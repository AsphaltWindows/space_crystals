//! The minimap as plain data: its frame in the Minimap section and the mappings each way, on square and letterboxed
//! maps; the camera's rectangle at two zoom levels; when the layers are redrawn; and the layers themselves, with the
//! fog of war on (`layers`).

mod layers;

use sc_sim::SU_PER_GU;

use super::*;
use crate::camera::view_rect;
use crate::hud::HUD_INSETS;
use crate::hud::layout::SECTION_PADDING_VH;

/// The default window's surface, 1280 × 720 logical pixels, at `scale` physical pixels per logical one.
fn surface(scale: f32) -> SurfaceSize {
    SurfaceSize {
        physical: (Vec2::new(1280.0, 720.0) * scale).as_uvec2(),
        scale_factor: scale,
    }
}

/// A map of `w` × `h` tiles, in space units.
fn map(w: u32, h: u32) -> Vec2 {
    Vec2::new(w as f32, h as f32) * SU_PER_GU
}

/// The Minimap section's content box on `size`: inside its border and padding.
fn content(size: SurfaceSize) -> Rect {
    let surface = size.logical();
    let inset = SECTION_BORDER_PX + SECTION_PADDING_VH * surface.y / 100.0;
    section_rect(HudSection::Minimap, surface).inflate(-inset)
}

/// The Director's decision for slice M5-MM: "the map is letterboxed in the square panel, aspect kept". Every map
/// fills the content box along its longer side, keeps its aspect to a pixel, and is centred, on whole pixels with one
/// texel each: a square map the whole box, `playtest.json`'s 64 × 48 with bars above and below, a tall map with bars
/// at the sides.
#[test]
fn the_map_fills_the_section_with_its_aspect_kept() {
    let size = surface(1.0);
    let content = content(size);
    for (w, h) in [(64, 64), (64, 48), (32, 64), (256, 64)] {
        let frame = MinimapFrame::on(size, map(w, h));
        let area = frame.area;
        assert_eq!(area.size(), frame.texels.as_vec2(), "{w} × {h}: a texel a pixel");
        assert_eq!(area.min, area.min.round(), "{w} × {h}: on whole pixels");
        assert!(
            content.contains(area.min) && content.contains(area.max),
            "{w} × {h}: {area:?} in {content:?}"
        );
        assert!(
            (content.size() - area.size()).min_element() < 1.0,
            "{w} × {h}: as large as fits"
        );
        assert!(area.center().distance(content.center()) < 0.71, "{w} × {h}: centred");
        let aspect = area.width() - area.height() * w as f32 / h as f32;
        assert!(aspect.abs() <= w as f32 / h as f32, "{w} × {h}: aspect kept, {aspect}");
    }
    assert_eq!(MinimapFrame::on(size, map(64, 64)).texels, UVec2::new(154, 154));
    assert_eq!(MinimapFrame::on(size, map(64, 48)).texels, UVec2::new(154, 116));
    assert_eq!(MinimapFrame::on(size, map(32, 64)).texels, UVec2::new(77, 154));
}

/// The Director's decision: "one tested function each way: panel pixel to world point, and world point to panel
/// pixel". There and back again, on a square map and on letterboxed ones: a ground point to the panel and back, a
/// point of the panel to the ground and back; the map area's corners are the map's.
#[test]
fn the_mappings_go_there_and_back() {
    for (w, h) in [(64, 64), (64, 48), (32, 64)] {
        let frame = MinimapFrame::on(surface(1.0), map(w, h));
        for fraction in [Vec2::ZERO, Vec2::new(0.25, 0.75), Vec2::splat(0.5), Vec2::new(0.9, 0.1)] {
            let ground = fraction * frame.map;
            let screen = frame.screen_at(ground);
            assert!(frame.contains(screen), "{w} × {h}: {ground} at {screen}");
            let back = frame.ground_at(screen);
            assert!(back.distance(ground) < 0.01, "{w} × {h}: {ground} came back as {back}");
        }
        let area = frame.area;
        for screen in [area.min, area.center(), area.min + Vec2::new(3.5, 7.25), area.max - 0.5] {
            let back = frame.screen_at(frame.ground_at(screen));
            assert!(back.distance(screen) < 0.001, "{w} × {h}: {screen} came back as {back}");
        }
        assert_eq!(frame.ground_at(area.min), Vec2::ZERO);
        let end = frame.ground_at(area.max);
        assert!(
            end.cmplt(frame.map).all() && end.distance(frame.map) < 0.01,
            "{w} × {h}: {end}"
        );
        assert_eq!(frame.texel_at(frame.map), frame.texels.as_vec2());
    }
}

/// A drag goes on off the map area: a point beside it is the nearest point on the map's edge, one beyond its corner
/// the corner; only the area takes presses.
#[test]
fn a_point_off_the_map_area_is_on_the_map_s_edge() {
    let frame = MinimapFrame::on(surface(1.0), map(64, 48));
    let area = frame.area;
    let middle = area.center();
    let left = frame.ground_at(Vec2::new(area.min.x - 40.0, middle.y));
    assert_eq!(left.x, 0.0);
    assert!((left.y - frame.map.y / 2.0).abs() < 0.01);
    let above = frame.ground_at(Vec2::new(middle.x, area.min.y - 5.0));
    assert_eq!(above.y, 0.0, "in the bar above the map");
    let corner = frame.ground_at(area.max + 100.0);
    assert!(corner.cmplt(frame.map).all() && corner.distance(frame.map) < 0.01);
    assert!(!frame.contains(Vec2::new(middle.x, area.min.y - 5.0)));
    assert!(!frame.contains(area.max), "the far edges are outside");
    assert!(frame.contains(area.min) && frame.contains(middle));
}

/// On a surface of twice the pixel density the image has twice the texels, one per physical pixel, in the same place
/// to half a logical pixel.
#[test]
fn at_twice_the_scale_the_image_has_twice_the_texels() {
    let one = MinimapFrame::on(surface(1.0), map(64, 48));
    let two = MinimapFrame::on(surface(2.0), map(64, 48));
    assert_eq!(two.scale(), 2.0);
    assert_eq!(two.area.size() * 2.0, two.texels.as_vec2());
    assert!(two.texels.as_vec2().distance(one.texels.as_vec2() * 2.0) <= 2.0);
    assert!(two.area.min.distance(one.area.min) <= 0.71);
}

/// "The camera's view as a rectangle outline": the world view's extent ([`view_rect`], as the camera has it) around
/// the ground point at its centre, at the game's scale and at the editor's zoom 2, twice as large; snapped to whole
/// pixels, each corner within half a pixel of the view's own corners on the panel. Near the map's corner the view goes
/// past the map, and the rectangle is cut to the map area, all four sides showing.
#[test]
fn the_camera_rectangle_is_the_camera_s_view_at_two_zoom_levels() {
    let size = surface(1.0);
    let frame = MinimapFrame::on(size, map(64, 48));
    let center = frame.map / 2.0;
    let mut sizes = Vec::new();
    for zoom in [1.0, 2.0] {
        let view = view_rect(size, HUD_INSETS, center, zoom);
        let rect = frame.camera_rect(center, view.size()).expect("on the map");
        let expected = Rect::from_corners(frame.screen_at(view.min), frame.screen_at(view.max));
        for (corner, wanted) in [(rect.min, expected.min), (rect.max, expected.max)] {
            assert!(corner.distance(wanted) <= 0.71, "zoom {zoom}: {corner} for {wanted}");
            assert_eq!(corner, corner.round(), "zoom {zoom}: whole pixels");
        }
        sizes.push(expected.size());
    }
    assert!(
        sizes[1].distance(sizes[0] * 2.0) < 0.01,
        "zoom 2 shows twice as much: {sizes:?}"
    );
    let view = view_rect(size, HUD_INSETS, Vec2::ZERO, 1.0);
    let cut = frame.camera_rect(Vec2::ZERO, view.size()).expect("on the map");
    assert_eq!(cut.min, frame.area.min, "cut at the map's corner");
    assert!(cut.max.distance(frame.screen_at(view.max)) <= 0.71);
    assert_eq!(frame.camera_rect(frame.map * 3.0, view.size()), None, "off the map");
}

/// The layers are redrawn only when what they show changed; at once the first time and while the sim stands still,
/// else at most [`REDRAWS_PER_SECOND`] times a second.
#[test]
fn the_layers_redraw_when_what_they_show_changes_at_most_fifteen_times_a_second() {
    let step = 1.0 / REDRAWS_PER_SECOND;
    assert!(redraw_due(None, &1, 0.0, false), "never drawn");
    assert!(!redraw_due(Some(&(1, 0.0)), &1, 10.0, false), "nothing changed");
    assert!(
        !redraw_due(Some(&(1, 0.0)), &1, 10.0, true),
        "nothing changed, the sim held"
    );
    assert!(!redraw_due(Some(&(1, 0.0)), &2, step * 0.9, false), "too soon");
    assert!(redraw_due(Some(&(1, 0.0)), &2, step, false), "due");
    assert!(
        redraw_due(Some(&(1, 0.0)), &2, 0.001, true),
        "the sim stands still: at once"
    );
}
