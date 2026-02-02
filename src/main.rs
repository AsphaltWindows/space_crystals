use bevy::prelude::*;

mod map;
mod resources;
mod units;
mod pathfinding;
mod commands;
mod combat;
mod turret;
mod projectile;
mod faction;

fn main() {
    App::new()
        // Default plugins include windowing, rendering, input, etc.
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Space Crystals RTS".to_string(),
                resolution: (1280.0, 720.0).into(),
                ..default()
            }),
            ..default()
        }))
        // Our custom game plugins
        .add_plugins((MapPlugin, ResourcesPlugin, UnitsPlugin, CommandsPlugin, CombatPlugin, TurretPlugin, ProjectilePlugin, FactionPlugin, GamePlugin))
        .run();
}

use map::MapPlugin;
use resources::ResourcesPlugin;
use units::UnitsPlugin;
use commands::CommandsPlugin;
use combat::CombatPlugin;
use turret::TurretPlugin;
use projectile::ProjectilePlugin;
use faction::FactionPlugin;

// Main game plugin - organizes all game systems
struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup)
            .add_systems(Update, (
                camera_movement,
                camera_zoom,
            ));
    }
}

// Components
#[derive(Component)]
struct MainCamera;

// Startup system - runs once at the beginning
fn setup(
    mut commands: Commands,
) {
    // Spawn a 3D camera for our RTS view, positioned to see the entire 20x20 grid
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 25.0, 15.0)
                .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
            ..default()
        },
        MainCamera,
    ));

    // Add a simple light
    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_rotation(Quat::from_euler(
            EulerRot::XYZ,
            -0.5,
            0.5,
            0.0,
        )),
        ..default()
    });

    info!("Space Crystals RTS initialized!");
}

// RTS camera movement system (WASD or arrow keys)
fn camera_movement(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
) {
    let mut camera_transform = camera_query.single_mut();
    let speed = 10.0;
    let delta = time.delta_seconds();

    // Move camera forward/back (W/S or Up/Down)
    if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
        camera_transform.translation.z -= speed * delta;
    }
    if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
        camera_transform.translation.z += speed * delta;
    }

    // Move camera left/right (A/D or Left/Right)
    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        camera_transform.translation.x -= speed * delta;
    }
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        camera_transform.translation.x += speed * delta;
    }
}

// Camera zoom system (Q/E keys)
fn camera_zoom(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
) {
    let mut camera_transform = camera_query.single_mut();
    let zoom_speed = 10.0;
    let delta = time.delta_seconds();

    if keyboard.pressed(KeyCode::KeyQ) {
        camera_transform.translation.y += zoom_speed * delta;
    }
    if keyboard.pressed(KeyCode::KeyE) {
        camera_transform.translation.y -= zoom_speed * delta;
    }
}
