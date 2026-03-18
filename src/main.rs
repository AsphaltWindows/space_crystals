use bevy::prelude::*;
use bevy::window::WindowResolution;

use space_crystals::game::{MapPlugin, ResourcesPlugin, UnitsPlugin, CommandsPlugin, CombatPlugin, TurretPlugin, ProjectilePlugin, FactionPlugin};
use space_crystals::simulation::SimulationCorePlugin;
#[cfg(feature = "diagnostics")]
use space_crystals::simulation::diagnostics::PerformanceDiagnosticsPlugin;
#[cfg(feature = "diagnostics")]
use bevy::diagnostic::{EntityCountDiagnosticsPlugin, LogDiagnosticsPlugin};
use space_crystals::ui;
use space_crystals::ui::HudPlugin;
use bevy::camera::Viewport;
use space_crystals::types::{MainCamera, AppState};
use space_crystals::simulation::types::DiagCategory;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Space Crystals RTS".to_string(),
                resolution: WindowResolution::new(1280, 720),
                ..default()
            }),
            ..default()
        }))
        .add_plugins((SimulationCorePlugin, MapPlugin, ResourcesPlugin, UnitsPlugin, CommandsPlugin, CombatPlugin, TurretPlugin, ProjectilePlugin, FactionPlugin, HudPlugin, GamePlugin));

    #[cfg(feature = "diagnostics")]
    app.add_plugins((
        PerformanceDiagnosticsPlugin,
        EntityCountDiagnosticsPlugin::default(),
        LogDiagnosticsPlugin::default(),
    ));

    app.run();
}

// Main game plugin - organizes all game systems
struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            .init_state::<AppState>()
            .add_systems(Startup, setup)
            .add_systems(Update, (
                camera_movement,
                camera_zoom,
                update_camera_viewport,
            ).in_set(DiagCategory::Camera));
    }
}

// Startup system - runs once at the beginning
fn setup(
    mut commands: Commands,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 40.0, 25.0)
            .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
        MainCamera,
    ));

    commands.spawn((
        DirectionalLight::default(),
        Transform::from_rotation(Quat::from_euler(
            EulerRot::XYZ,
            -0.5,
            0.5,
            0.0,
        )),
    ));

    info!("Space Crystals RTS initialized!");
}

// RTS camera movement system (arrow keys only)
fn camera_movement(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
) {
    let Ok(mut camera_transform) = camera_query.single_mut() else { return; };
    let speed = 10.0;
    let delta = time.delta_secs();

    if keyboard.pressed(KeyCode::ArrowUp) {
        camera_transform.translation.z -= speed * delta;
    }
    if keyboard.pressed(KeyCode::ArrowDown) {
        camera_transform.translation.z += speed * delta;
    }
    if keyboard.pressed(KeyCode::ArrowLeft) {
        camera_transform.translation.x -= speed * delta;
    }
    if keyboard.pressed(KeyCode::ArrowRight) {
        camera_transform.translation.x += speed * delta;
    }
}

// Camera zoom system (Q/E keys or scroll wheel)
// When the command panel is visible, Q/E keys are reserved for grid hotkeys.
fn camera_zoom(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
    interface_state: Res<ui::types::ObjectInterfaceState>,
    selection: Res<space_crystals::types::Selection>,
) {
    let Ok(mut camera_transform) = camera_query.single_mut() else { return; };
    let zoom_speed = 10.0;
    let delta = time.delta_secs();

    // Panel is hidden when in Default state with empty selection
    let panel_hidden = matches!(*interface_state, ui::types::ObjectInterfaceState::Default) && selection.groups.is_empty();

    if panel_hidden && keyboard.pressed(KeyCode::KeyQ) {
        camera_transform.translation.y += zoom_speed * delta;
    }
    if panel_hidden && keyboard.pressed(KeyCode::KeyE) {
        camera_transform.translation.y -= zoom_speed * delta;
    }
}

// Update camera viewport to exclude HUD regions (top bar + bottom panel)
fn update_camera_viewport(
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut cameras: Query<&mut Camera, With<MainCamera>>,
) {
    let Ok(window) = windows.single() else { return; };
    let scale_factor = window.scale_factor();
    let physical_width = window.physical_width();
    let physical_height = window.physical_height();

    let top_px = (ui::types::HUD_TOP_BAR_HEIGHT * scale_factor).ceil() as u32;
    let bottom_px = (ui::types::HUD_BOTTOM_PANEL_HEIGHT * scale_factor).ceil() as u32;

    let viewport_height = physical_height.saturating_sub(top_px + bottom_px);

    if viewport_height == 0 {
        return;
    }

    let Ok(mut camera) = cameras.single_mut() else { return; };
    camera.viewport = Some(Viewport {
        physical_position: UVec2::new(0, top_px),
        physical_size: UVec2::new(physical_width, viewport_height),
        ..default()
    });
}
