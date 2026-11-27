use super::*;
use sc_sim::{TilePos, scenarios};

fn surface(w: u32, h: u32) -> SurfaceSize {
    SurfaceSize {
        physical: UVec2::new(w, h),
        scale_factor: 1.0,
    }
}

#[test]
fn viewport_lies_between_the_bars() {
    let insets = ViewportInsets {
        top_vh: 4.0,
        bottom_vh: 24.0,
        ..default()
    };
    let (pos, size) = viewport_rect(surface(1280, 720), insets);
    assert_eq!(pos, UVec2::new(0, 28)); // 28.8 rounds up under the top bar
    assert_eq!(size, UVec2::new(1280, 548 - 28)); // 547.2 rounds down under the bottom bar
    let (pos, size) = viewport_rect(surface(800, 600), ViewportInsets::default());
    assert_eq!((pos, size), (UVec2::ZERO, UVec2::new(800, 600)));
    // Side panels: 10 vh is 60 px of a 600 px tall surface.
    let sides = ViewportInsets {
        left_vh: 10.0,
        right_vh: 20.0,
        ..default()
    };
    assert_eq!(
        viewport_rect(surface(800, 600), sides),
        (UVec2::new(60, 0), UVec2::new(800 - 60 - 120, 600))
    );
}

#[test]
fn view_is_28_gu_wide_and_as_tall_as_the_aspect_ratio_allows() {
    let visible = visible_extent_su(UVec2::new(1400, 700), 1400, 1.0);
    assert_eq!(visible, Vec2::new(28.0 * 64.0, 14.0 * 64.0));
    // A narrower viewport keeps the game's scale; zooming out shows more.
    assert_eq!(
        visible_extent_su(UVec2::new(700, 700), 1400, 1.0),
        Vec2::splat(14.0 * 64.0)
    );
    assert_eq!(
        visible_extent_su(UVec2::new(700, 700), 1400, 2.0),
        Vec2::splat(28.0 * 64.0)
    );
}

#[test]
fn the_cursor_maps_to_the_view_point_under_it() {
    let insets = ViewportInsets {
        top_vh: 10.0,
        left_vh: 20.0,
        ..default()
    };
    let size = surface(1000, 500); // viewport at (100, 50), 900 × 450 px
    let center = Vec2::new(2000.0, 1000.0);
    let su_per_px = gu(crate::tuning::camera::BUILTIN.view_width_gu) / 1000.0;
    let at = |x, y| cursor_to_view(Vec2::new(x, y), size, insets, center, 1.0);
    assert_eq!(at(550.0, 275.0), Some(center));
    assert_eq!(at(100.0, 50.0), Some(center - Vec2::new(450.0, 225.0) * su_per_px));
    assert_eq!(at(99.0, 300.0), None, "left of the viewport");
    assert_eq!(at(500.0, 40.0), None, "under the top bar");
    assert_eq!(at(f32::NAN, 300.0), None);
    let zoomed = cursor_to_view(Vec2::new(100.0, 275.0), size, insets, center, 2.0).unwrap();
    assert_eq!(zoomed, center - Vec2::new(450.0 * su_per_px * 2.0, 0.0));
}

#[test]
fn the_cursor_picks_the_ground_drawn_under_it() {
    let size = surface(1000, 500);
    let insets = ViewportInsets::default();
    // A map raised to level 4 everywhere: the ground under a view point lies a tile further south.
    let map = Map::from_ascii("....\n....\n\n44444\n44444\n44444").unwrap();
    let view = CameraView {
        center: Vec2::new(128.0, 64.0),
    };
    let ground = cursor_to_ground(Vec2::new(500.0, 250.0), size, insets, view, 1.0, &map).unwrap();
    assert_eq!(ground, Vec2::new(128.0, 64.0 + 64.0));
    assert_eq!(drawn_position(&map, ground), view.center);
}

#[test]
fn a_grab_keeps_the_grabbed_point_under_the_cursor() {
    let size = surface(1000, 500);
    let insets = ViewportInsets {
        top_vh: 10.0,
        ..default()
    };
    for zoom in [1.0, 2.5] {
        let grabbed = cursor_to_view(Vec2::new(300.0, 200.0), size, insets, Vec2::new(900.0, 700.0), zoom).unwrap();
        for cursor in [Vec2::new(310.0, 190.0), Vec2::new(900.0, 480.0), Vec2::new(-50.0, 10.0)] {
            let center = grab_center(grabbed, cursor, size, insets, zoom).unwrap();
            let under = center + cursor_offset(cursor, size, insets, zoom).unwrap();
            assert!((under - grabbed).length() < 1e-3, "{cursor}: {under} vs {grabbed}");
        }
    }
}

#[test]
fn the_start_falls_back_from_camera_start_to_start_position_to_map_center() {
    // (Between the two, the Deployment Center: see the client's camera tests.)
    let mut scenario = scenarios::empty(20, 10);
    scenario.map.slots[0].camera_start = Some(Vec2::new(100.0, 200.0));
    scenario.map.slots[1].start = TilePos::new(3, 4);
    let map = scenario.map.terrain.clone();
    assert_eq!(start_center(&scenario, PlayerId(1), &map), Vec2::new(100.0, 200.0));
    assert_eq!(start_center(&scenario, PlayerId(2), &map), TilePos::new(3, 4).center());
    // The player's slot decides, not the player's number.
    scenario.setup.players[0].slot = sc_sim::SlotId(2);
    scenario.setup.players[1].slot = sc_sim::SlotId(1);
    assert_eq!(start_center(&scenario, PlayerId(1), &map), TilePos::new(3, 4).center());
    assert_eq!(start_center(&scenario, PlayerId(9), &map), Vec2::new(640.0, 320.0));
    scenario.setup.players[0].slot = sc_sim::SlotId(7);
    assert_eq!(start_center(&scenario, PlayerId(1), &map), Vec2::new(640.0, 320.0));
}

#[test]
fn clamping_keeps_the_view_on_the_map() {
    let visible = Vec2::new(1000.0, 500.0);
    let map = Rect::new(0.0, 0.0, 4000.0, 3000.0);
    assert_eq!(
        clamp_center(Vec2::new(-50.0, 9999.0), visible, map),
        Vec2::new(500.0, 2750.0)
    );
    assert_eq!(
        clamp_center(Vec2::new(2000.0, 1000.0), visible, map),
        Vec2::new(2000.0, 1000.0)
    );
    // A map narrower than the view is centered.
    assert_eq!(
        clamp_center(Vec2::new(0.0, 0.0), visible, Rect::new(0.0, 0.0, 800.0, 3000.0)).x,
        400.0
    );
    // Raised ground at the north edge is drawn above y 0, and the view can reach it.
    let raised = Rect::new(0.0, -128.0, 4000.0, 3000.0);
    assert_eq!(clamp_center(Vec2::new(10.0, -500.0), visible, raised).y, -128.0 + 250.0);
}
