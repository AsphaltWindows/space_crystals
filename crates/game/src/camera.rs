//! The world camera and the UI camera (`core/camera.md`, `ARCHITECTURE.md > Client architecture > Camera`).
//!
//! The world camera is orthographic and shows exactly [`VIEW_WIDTH_GU`] horizontally. Its viewport is the strip
//! between the active mode's UI bars ([`ViewportInsets`]), so the visible height follows from the aspect ratio. A
//! second, full-surface camera renders only UI, because bevy_ui lays out each root node inside its camera's viewport.

use bevy::camera::visibility::RenderLayers;
use bevy::camera::{ScalingMode, Viewport};
use bevy::prelude::*;
use sc_sim::{Map, PlayerId, SU_PER_GU, Scenario, gu};

use crate::ClientSystems;
use crate::coords::sim_to_world;
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

/// The parts of the surface covered by the active mode's UI bars, in percent of the surface height. The world
/// view is the strip between them (`core/camera.md > Zoom`). The game HUD sets them; an editor sets its own.
#[derive(Resource, Copy, Clone, Debug, Default, PartialEq)]
pub struct ViewportInsets {
    pub top_vh: f32,
    pub bottom_vh: f32,
}

/// Where the camera looks: the view's center in sim space units. The camera's transform derives from it.
#[derive(Resource, Copy, Clone, Debug, PartialEq)]
pub struct CameraView {
    pub center: Vec2,
}

pub struct GameCameraPlugin;

impl Plugin for GameCameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ViewportInsets>()
            .add_systems(Startup, (init_camera_view, spawn_cameras))
            .add_systems(Update, pan_camera.in_set(ClientSystems::Actions))
            .add_systems(Update, apply_camera.in_set(ClientSystems::Present));
    }
}

/// `core/camera.md > Starting Position`: the map's starting camera position for the local player's slot, else the
/// player's primary starting structure. No starting structures exist yet (they arrive with the faction
/// milestones), so the map center stands in for them.
pub fn start_center(scenario: &Scenario, player: PlayerId, map: &Map) -> Vec2 {
    scenario
        .players
        .iter()
        .find(|p| p.id == player)
        .and_then(|p| p.camera_start)
        .unwrap_or_else(|| map_extent_su(map) / 2.0)
}

/// The map's size in space units.
pub fn map_extent_su(map: &Map) -> Vec2 {
    Vec2::new(map.width() as f32, map.height() as f32) * SU_PER_GU
}

/// The world camera's viewport (physical position and size): the full surface width, between the UI bars. Its
/// edges round outwards, under the bars, so no seam shows between the view and a bar.
pub fn viewport_rect(size: SurfaceSize, insets: ViewportInsets) -> (UVec2, UVec2) {
    let width = size.physical.x.max(1);
    let height = size.physical.y.max(1) as f32;
    let top = (height * insets.top_vh / 100.0).floor().clamp(0.0, height - 1.0);
    let bottom = (height * (1.0 - insets.bottom_vh / 100.0))
        .ceil()
        .clamp(top + 1.0, height);
    (UVec2::new(0, top as u32), UVec2::new(width, (bottom - top) as u32))
}

/// The world area a viewport of `viewport_size` shows, in space units: 28 GU wide, height by aspect ratio.
pub fn visible_extent_su(viewport_size: UVec2) -> Vec2 {
    let width = gu(VIEW_WIDTH_GU);
    let size = viewport_size.max(UVec2::ONE).as_vec2();
    Vec2::new(width, width * size.y / size.x)
}

/// Keeps the view on the map: each axis is clamped so the view stays inside the map, or centered on the map when
/// the map is smaller than the view.
pub fn clamp_center(center: Vec2, visible: Vec2, map: Vec2) -> Vec2 {
    let axis = |c: f32, v: f32, m: f32| if m <= v { m / 2.0 } else { c.clamp(v / 2.0, m - v / 2.0) };
    Vec2::new(axis(center.x, visible.x, map.x), axis(center.y, visible.y, map.y))
}

fn init_camera_view(mut commands: Commands, scenario: Res<LoadedScenario>, player: Res<LocalPlayer>, sim: Res<SimRes>) {
    commands.insert_resource(CameraView {
        center: start_center(&scenario.0, player.0, sim.map()),
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
/// or running fast.
fn pan_camera(actions: Res<ActionState>, time: Res<Time<Real>>, mut view: ResMut<CameraView>) {
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
        view.center += direction.normalize() * gu(PAN_SPEED_GU_PER_SECOND) * time.delta_secs();
    }
}

/// Clamps the view to the map and puts the world camera's viewport and transform where they belong.
fn apply_camera(
    size: Res<SurfaceSize>,
    insets: Res<ViewportInsets>,
    sim: Res<SimRes>,
    mut view: ResMut<CameraView>,
    mut camera: Query<(&mut Camera, &mut Transform), With<WorldCamera>>,
) {
    let (position, viewport_size) = viewport_rect(*size, *insets);
    let visible = visible_extent_su(viewport_size);
    let center = clamp_center(view.center, visible, map_extent_su(sim.map()));
    if center != view.center {
        view.center = center;
    }
    let Ok((mut camera, mut transform)) = camera.single_mut() else {
        return;
    };
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
    let snapped = (sim_to_world(center) * pixels_per_su).round() / pixels_per_su;
    let translation = snapped.extend(0.0);
    if transform.translation != translation {
        transform.translation = translation;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        };
        let (pos, size) = viewport_rect(surface(1280, 720), insets);
        assert_eq!(pos, UVec2::new(0, 28)); // 28.8 rounds up under the top bar
        assert_eq!(size, UVec2::new(1280, 548 - 28)); // 547.2 rounds down under the bottom bar
        let (pos, size) = viewport_rect(surface(800, 600), ViewportInsets::default());
        assert_eq!((pos, size), (UVec2::ZERO, UVec2::new(800, 600)));
    }

    #[test]
    fn view_is_28_gu_wide_and_as_tall_as_the_aspect_ratio_allows() {
        let visible = visible_extent_su(UVec2::new(1400, 700));
        assert_eq!(visible, Vec2::new(28.0 * 64.0, 14.0 * 64.0));
    }

    #[test]
    fn clamping_keeps_the_view_on_the_map() {
        let visible = Vec2::new(1000.0, 500.0);
        let map = Vec2::new(4000.0, 3000.0);
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
            clamp_center(Vec2::new(0.0, 0.0), visible, Vec2::new(800.0, 3000.0)).x,
            400.0
        );
    }
}
