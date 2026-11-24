//! The world camera and the UI camera (`core/camera.md`, `ARCHITECTURE.md > Client architecture > Camera`).
//!
//! The world camera is orthographic and shows exactly [`VIEW_WIDTH_GU`] horizontally. Its viewport is the area the
//! active mode's UI leaves free ([`ViewportInsets`]), so the visible height follows from the aspect ratio. A second,
//! full-surface camera renders only UI, because bevy_ui lays out each root node inside its camera's viewport.
//!
//! The game has no zoom (`core/camera.md > Zoom`). The map editor has its own: [`CameraZoom`] scales the view
//! relative to the game's pixel scale, 28 GU across the full surface width.
//!
//! The camera looks at view space (`crate::coords`): [`CameraView::center`] is a view point, the cursor maps to a
//! view point ([`cursor_to_view`]) and from there to the ground drawn under it ([`cursor_to_ground`]), and the view
//! stays inside the map's drawn area ([`drawn_bounds`]), so raised ground at the north edge stays reachable. It pans
//! with the pan actions and with the middle-mouse grab ([`GrabPan`]), in the game and in the editor alike.

use bevy::camera::visibility::RenderLayers;
use bevy::camera::{ScalingMode, Viewport};
use bevy::prelude::*;
use sc_sim::{Map, PlayerId, SU_PER_GU, Scenario, Slot, gu};

use crate::ClientSystems;
use crate::coords::{drawn_bounds, drawn_position, ground_under, view_to_world};
use crate::input::{ActionState, InputAction};
use crate::platform::{RenderSurface, SurfaceSize};
use crate::sim_driver::{LoadedScenario, LocalPlayer, SimRes};

/// `core/camera.md > Zoom`: the camera displays exactly 28 GridUnits horizontally.
pub const VIEW_WIDTH_GU: f32 = 28.0;
/// Placeholder pan speed until Q0002 settles the pan inputs and speed.
pub const PAN_SPEED_GU_PER_SECOND: f32 = 24.0;
/// Shown where the map doesn't fill the view (placeholder colour).
const BACKGROUND: Color = Color::srgb(0.02, 0.02, 0.03);

#[derive(Component)]
pub struct WorldCamera;

#[derive(Component)]
pub struct UiCamera;

/// The parts of the surface covered by the active mode's UI, in percent of the surface height (`vh`, also for the
/// sides). The world view is the area between them (`core/camera.md > Zoom`). The game HUD sets its top and bottom
/// bars; the map editor also covers the sides with its panels.
#[derive(Resource, Copy, Clone, Debug, Default, PartialEq)]
pub struct ViewportInsets {
    pub top_vh: f32,
    pub bottom_vh: f32,
    pub left_vh: f32,
    pub right_vh: f32,
}

/// Map editor zoom: how many times more world the view shows than the game, whose scale is 28 GU across the full
/// surface width (`core/camera.md > Zoom`). Always 1 in the game, which has no zoom.
#[derive(Resource, Copy, Clone, Debug, PartialEq)]
pub struct CameraZoom(pub f32);

impl Default for CameraZoom {
    fn default() -> Self {
        Self(1.0)
    }
}

/// Where the camera looks: the view's center, a view-space point (space units, `crate::coords`). The camera's
/// transform derives from it. To look at a ground point, center on its [`drawn_position`].
#[derive(Resource, Copy, Clone, Debug, PartialEq)]
pub struct CameraView {
    pub center: Vec2,
}

/// The middle-mouse grab (`core/camera.md > Panning > Middle-mouse grab`): the view point grabbed when the button
/// went down over the map, kept under the cursor until the button is released.
#[derive(Resource, Default, Debug)]
pub struct GrabPan {
    pub grabbed: Option<Vec2>,
}

pub struct GameCameraPlugin;

impl Plugin for GameCameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ViewportInsets>()
            .init_resource::<CameraZoom>()
            .init_resource::<GrabPan>()
            .add_systems(Startup, (init_camera_view, spawn_cameras))
            // The grab sets the view outright, so it comes last.
            .add_systems(Update, (pan_camera, grab_pan).chain().in_set(ClientSystems::Actions))
            .add_systems(Update, apply_camera.in_set(ClientSystems::Present));
    }
}

/// `core/camera.md > Starting Position`: the map's starting camera position for the local player's slot, else the
/// player's primary starting structure (the GDO's Deployment Center), where their Starting Layout places it
/// ([`Scenario::primary_structure`]). A player without one (a faction whose Starting Conditions aren't implemented
/// yet, or a layout without it) starts at their slot's start position, and a player the setup doesn't have (or
/// whose slot the map lacks) at the map center. A ground point: the camera centers on its [`start_view`].
pub fn start_center(scenario: &Scenario, player: PlayerId, map: &Map) -> Vec2 {
    let slot = scenario.setup.player(player).and_then(|p| scenario.map.slot(p.slot));
    match (slot.and_then(|s| s.camera_start), scenario.primary_structure(player)) {
        (Some(camera_start), _) => camera_start,
        (None, Some(structure)) => structure,
        (None, None) => slot_center(slot, map),
    }
}

/// [`start_center`] for a slot: its camera start, else its start position, else (no slot) the map center.
pub fn slot_center(slot: Option<&Slot>, map: &Map) -> Vec2 {
    slot.map_or_else(
        || map_extent_su(map) / 2.0,
        |s| s.camera_start.unwrap_or_else(|| s.start.center()),
    )
}

/// The view center that shows [`start_center`]: where that ground point is drawn.
pub fn start_view(scenario: &Scenario, player: PlayerId, map: &Map) -> Vec2 {
    drawn_position(map, start_center(scenario, player, map))
}

/// The map's size in space units.
pub fn map_extent_su(map: &Map) -> Vec2 {
    Vec2::new(map.width() as f32, map.height() as f32) * SU_PER_GU
}

/// The world camera's viewport (physical position and size): the area between the UI's insets. Its edges round
/// outwards, under the UI, so no seam shows between the view and a bar.
pub fn viewport_rect(size: SurfaceSize, insets: ViewportInsets) -> (UVec2, UVec2) {
    let width = size.physical.x.max(1) as f32;
    let height = size.physical.y.max(1) as f32;
    let vh = height / 100.0;
    let top = (insets.top_vh * vh).floor().clamp(0.0, height - 1.0);
    let bottom = (height - insets.bottom_vh * vh).ceil().clamp(top + 1.0, height);
    let left = (insets.left_vh * vh).floor().clamp(0.0, width - 1.0);
    let right = (width - insets.right_vh * vh).ceil().clamp(left + 1.0, width);
    (
        UVec2::new(left as u32, top as u32),
        UVec2::new((right - left) as u32, (bottom - top) as u32),
    )
}

/// The world area a viewport of `viewport_size` shows, in space units, on a surface `surface_width` pixels wide. At
/// zoom 1 that is the game's scale, 28 GU across the full surface width, so a full-width viewport shows exactly
/// 28 GU. The height follows from the aspect ratio.
pub fn visible_extent_su(viewport_size: UVec2, surface_width: u32, zoom: f32) -> Vec2 {
    let size = viewport_size.max(UVec2::ONE).as_vec2();
    let width = gu(VIEW_WIDTH_GU) * zoom * size.x / surface_width.max(1) as f32;
    Vec2::new(width, width * size.y / size.x)
}

/// The area the world view shows, in view space: centered on `center`, as large as the viewport shows at `zoom`.
pub fn view_rect(size: SurfaceSize, insets: ViewportInsets, center: Vec2, zoom: f32) -> Rect {
    let (_, viewport) = viewport_rect(size, insets);
    Rect::from_center_half_size(center, visible_extent_su(viewport, size.physical.x, zoom) / 2.0)
}

/// How far from the view's center a cursor at `cursor` (logical pixels from the surface's top-left corner) points,
/// in space units, wherever it is; `None` for a non-finite cursor.
pub fn cursor_offset(cursor: Vec2, size: SurfaceSize, insets: ViewportInsets, zoom: f32) -> Option<Vec2> {
    let (position, viewport) = viewport_rect(size, insets);
    let local = cursor * size.scale_factor - position.as_vec2();
    if !local.is_finite() {
        return None;
    }
    let su_per_pixel = visible_extent_su(viewport, size.physical.x, zoom).x / viewport.x as f32;
    // Screen +y points down, like view +y.
    Some((local - viewport.as_vec2() / 2.0) * su_per_pixel)
}

/// The view point under a cursor at `cursor` (logical pixels from the surface's top-left corner), or `None`
/// outside the world viewport. `center` is the view's clamped center ([`CameraView`]).
pub fn cursor_to_view(
    cursor: Vec2,
    size: SurfaceSize,
    insets: ViewportInsets,
    center: Vec2,
    zoom: f32,
) -> Option<Vec2> {
    let (position, viewport) = viewport_rect(size, insets);
    let local = cursor * size.scale_factor - position.as_vec2();
    let inside = local.cmpge(Vec2::ZERO).all() && local.cmplt(viewport.as_vec2()).all();
    if !inside {
        return None;
    }
    cursor_offset(cursor, size, insets, zoom).map(|offset| center + offset)
}

/// The ground point visible under a cursor ([`cursor_to_view`], then [`ground_under`]): what every click and hover
/// on the map refers to.
pub fn cursor_to_ground(
    cursor: Vec2,
    size: SurfaceSize,
    insets: ViewportInsets,
    view: CameraView,
    zoom: f32,
    map: &Map,
) -> Option<Vec2> {
    cursor_to_view(cursor, size, insets, view.center, zoom).map(|v| ground_under(map, v))
}

/// The view center that puts the view point `grabbed` under a cursor at `cursor`: the grab pan's view.
pub fn grab_center(grabbed: Vec2, cursor: Vec2, size: SurfaceSize, insets: ViewportInsets, zoom: f32) -> Option<Vec2> {
    cursor_offset(cursor, size, insets, zoom).map(|offset| grabbed - offset)
}

/// Keeps the view on the map's drawn area `bounds` (view space, [`drawn_bounds`]): each axis is clamped so the
/// view stays inside it, or centered on it when it is smaller than the view.
pub fn clamp_center(center: Vec2, visible: Vec2, bounds: Rect) -> Vec2 {
    let axis = |c: f32, v: f32, lo: f32, hi: f32| {
        if hi - lo <= v {
            (lo + hi) / 2.0
        } else {
            c.clamp(lo + v / 2.0, hi - v / 2.0)
        }
    };
    Vec2::new(
        axis(center.x, visible.x, bounds.min.x, bounds.max.x),
        axis(center.y, visible.y, bounds.min.y, bounds.max.y),
    )
}

fn init_camera_view(mut commands: Commands, scenario: Res<LoadedScenario>, player: Res<LocalPlayer>, sim: Res<SimRes>) {
    commands.insert_resource(CameraView {
        center: start_view(&scenario.0, player.0, sim.map()),
    });
}

fn spawn_cameras(mut commands: Commands, surface: Res<RenderSurface>) {
    let projection = OrthographicProjection {
        scaling_mode: ScalingMode::FixedHorizontal {
            viewport_width: gu(VIEW_WIDTH_GU),
        },
        ..OrthographicProjection::default_2d()
    };
    commands.spawn((
        Name::new("World camera"),
        WorldCamera,
        Camera2d,
        Camera {
            order: 0,
            clear_color: ClearColorConfig::Custom(BACKGROUND),
            ..default()
        },
        Projection::Orthographic(projection),
        surface.target.clone(),
    ));
    commands.spawn((
        Name::new("UI camera"),
        UiCamera,
        Camera2d,
        Camera {
            order: 1,
            clear_color: ClearColorConfig::None,
            ..default()
        },
        // Draws no world entities: only the UI, over the whole surface.
        RenderLayers::none(),
        IsDefaultUiCamera,
        surface.target.clone(),
    ));
}

/// `core/camera.md > Panning`: pans while a pan action is held. Real time, so panning works while the sim is paused
/// or running fast. A zoomed-out editor view pans as many pixels per second as the game's view.
fn pan_camera(actions: Res<ActionState>, time: Res<Time<Real>>, zoom: Res<CameraZoom>, mut view: ResMut<CameraView>) {
    let mut direction = Vec2::ZERO; // sim space: +y is south
    for (action, step) in [
        (InputAction::PanLeft, Vec2::NEG_X),
        (InputAction::PanRight, Vec2::X),
        (InputAction::PanUp, Vec2::NEG_Y),
        (InputAction::PanDown, Vec2::Y),
    ] {
        if actions.held(action) {
            direction += step;
        }
    }
    if direction != Vec2::ZERO {
        view.center += direction.normalize() * gu(PAN_SPEED_GU_PER_SECOND) * zoom.0 * time.delta_secs();
    }
}

/// `core/camera.md > Panning > Middle-mouse grab`: the press grabs the view point under the cursor, if the cursor
/// is over the world view; while the button is held, the view follows so that point stays exactly under the cursor,
/// wherever the cursor goes; the release ends it on the spot, without drift.
fn grab_pan(
    actions: Res<ActionState>,
    size: Res<SurfaceSize>,
    insets: Res<ViewportInsets>,
    zoom: Res<CameraZoom>,
    mut grab: ResMut<GrabPan>,
    mut view: ResMut<CameraView>,
) {
    if !actions.held(InputAction::GrabPan) {
        grab.grabbed = None;
        return;
    }
    let Some(cursor) = actions.cursor() else {
        return;
    };
    if actions.just_pressed(InputAction::GrabPan) {
        grab.grabbed = cursor_to_view(cursor, *size, *insets, view.center, zoom.0);
        return;
    }
    if let Some(grabbed) = grab.grabbed
        && let Some(center) = grab_center(grabbed, cursor, *size, *insets, zoom.0)
        && center != view.center
    {
        view.center = center;
    }
}

/// Clamps the view to the map and puts the world camera's viewport, scale and transform where they belong.
fn apply_camera(
    size: Res<SurfaceSize>,
    insets: Res<ViewportInsets>,
    zoom: Res<CameraZoom>,
    sim: Res<SimRes>,
    mut view: ResMut<CameraView>,
    mut camera: Query<(&mut Camera, &mut Projection, &mut Transform), With<WorldCamera>>,
) {
    let (position, viewport_size) = viewport_rect(*size, *insets);
    let visible = visible_extent_su(viewport_size, size.physical.x, zoom.0);
    let center = clamp_center(view.center, visible, drawn_bounds(sim.map()));
    if center != view.center {
        view.center = center;
    }
    let Ok((mut camera, mut projection, mut transform)) = camera.single_mut() else {
        return;
    };
    // Compared before writing, so an unchanged projection isn't marked changed.
    let stale = match &*projection {
        Projection::Orthographic(o) => !matches!(
            o.scaling_mode,
            ScalingMode::FixedHorizontal { viewport_width } if viewport_width == visible.x
        ),
        _ => false,
    };
    if stale && let Projection::Orthographic(ortho) = &mut *projection {
        ortho.scaling_mode = ScalingMode::FixedHorizontal {
            viewport_width: visible.x,
        };
    }
    let unchanged = camera
        .viewport
        .as_ref()
        .is_some_and(|v| v.physical_position == position && v.physical_size == viewport_size);
    if !unchanged {
        camera.viewport = Some(Viewport {
            physical_position: position,
            physical_size: viewport_size,
            ..default()
        });
    }
    // Snap to whole physical pixels, so tile edges don't shimmer while panning.
    let pixels_per_su = viewport_size.x as f32 / visible.x;
    let snapped = (view_to_world(center) * pixels_per_su).round() / pixels_per_su;
    let translation = snapped.extend(0.0);
    if transform.translation != translation {
        transform.translation = translation;
    }
}

#[cfg(test)]
mod tests {
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
        let su_per_px = gu(VIEW_WIDTH_GU) / 1000.0;
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
}
