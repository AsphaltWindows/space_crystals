use bevy::prelude::*;
use crate::map::{GridPosition, Tile, TileProperties};
use crate::resources::{Selectable, Selected};
use crate::pathfinding::{Path, find_path, world_to_grid, smooth_path};
use crate::commands::{CommandMode, CommandType, UnitCommand};
use crate::combat::{AttackCapability, AttackState, AttackType};
use crate::turret::{create_turret_for_unit, spawn_turret_visual};

/// Component marking an entity as a unit
#[derive(Component)]
pub struct Unit;

/// Component storing unit health
#[derive(Component)]
pub struct UnitHealth {
    pub current: f32,
    pub max: f32,
}

/// Component tracking unit ownership
#[derive(Component, Clone, Copy, Debug)]
pub enum Owner {
    Player(u8),
    Neutral,
}

impl Owner {
    /// Get visual color for this owner
    pub fn color(&self) -> Color {
        match self {
            Owner::Player(0) => Color::srgb(0.2, 0.4, 0.8),  // Blue
            Owner::Player(1) => Color::srgb(0.8, 0.2, 0.2),  // Red
            Owner::Player(2) => Color::srgb(0.2, 0.8, 0.3),  // Green
            Owner::Player(3) => Color::srgb(0.8, 0.8, 0.2),  // Yellow
            _ => Color::srgb(0.6, 0.6, 0.6),                 // Gray for other players/neutral
        }
    }
}

/// Component storing unit type information
#[derive(Component)]
pub struct UnitType {
    pub name: String,
}

/// Component storing movement target position
#[derive(Component)]
pub struct MoveTarget(pub Vec3);

/// Component storing unit velocity
#[derive(Component)]
pub struct Velocity(pub Vec3);

/// Component storing movement speed
#[derive(Component)]
pub struct MovementSpeed(pub f32);

/// Component storing rotation speed (radians per second)
#[derive(Component)]
pub struct RotationSpeed(pub f32);

/// Component for visual move target marker
#[derive(Component)]
struct MoveTargetMarker;

/// Drill unit mode (underground or above-ground)
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DrillMode {
    Underground,
    AboveGround,
}

/// Unit base types with different movement characteristics
#[derive(Component, Clone, Copy, Debug)]
pub enum UnitBase {
    LightInfantry,
    WheeledVehicle {
        min_turn_radius: f32,
        forward_speed: f32,
        reverse_speed: f32,
    },
    TrackedVehicle {
        speed_to_turn_ratio: f32,
    },
    DrillUnit {
        mode: DrillMode,
        speed_to_turn_ratio: f32,
        acceleration: f32,
        deceleration: f32,
        max_speed: f32,
    },
    HoverVehicle {
        turn_rate: f32,
        forward_accel: f32,
        non_forward_accel: f32,
        drag_ratio: f32,
    },
    Mech {
        turn_rate: f32,
        max_speed: f32,
        acceleration: f32,
        deceleration: f32,
    },
}

impl UnitBase {
    /// Check if this unit type can traverse rugged terrain
    pub fn can_traverse_rugged(&self) -> bool {
        matches!(self, UnitBase::LightInfantry | UnitBase::Mech { .. })
    }

    /// Check if this unit type can traverse drillable terrain (underground mode)
    pub fn can_traverse_drillable(&self) -> bool {
        if let UnitBase::DrillUnit { mode, .. } = self {
            *mode == DrillMode::Underground
        } else {
            false
        }
    }

    /// Get movement speed for this unit base
    pub fn get_speed(&self) -> f32 {
        match self {
            UnitBase::LightInfantry => 3.0,
            UnitBase::WheeledVehicle { forward_speed, .. } => *forward_speed,
            UnitBase::TrackedVehicle { .. } => 2.5,
            UnitBase::DrillUnit { max_speed, .. } => *max_speed,
            UnitBase::HoverVehicle { forward_accel, drag_ratio, .. } => {
                // Effective max speed = forward_accel / drag_ratio
                forward_accel / drag_ratio
            },
            UnitBase::Mech { max_speed, .. } => *max_speed,
        }
    }

    /// Get rotation speed for this unit base (radians/sec)
    pub fn get_rotation_speed(&self) -> f32 {
        match self {
            UnitBase::LightInfantry => 10.0, // Very fast turning
            UnitBase::WheeledVehicle { .. } => 3.0, // Slower turning
            UnitBase::TrackedVehicle { .. } => 1.57, // ~90 degrees/sec
            UnitBase::DrillUnit { .. } => 1.2, // Slow turning
            UnitBase::HoverVehicle { turn_rate, .. } => *turn_rate,
            UnitBase::Mech { turn_rate, .. } => *turn_rate,
        }
    }
}

/// Plugin for unit-related systems
pub struct UnitsPlugin;

impl Plugin for UnitsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_test_units.after(crate::map::spawn_grid))
            .add_systems(Update, (
                unit_selection_display,
                right_click_move_command,
                unit_movement_system,
                unit_rotation_system,
            ));
    }
}

/// Spawn test units on the map
fn spawn_test_units(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Define test units: (grid_x, grid_z, owner, unit_name, health, mesh_type, unit_base)
    let unit_data = [
        (5, 10, Owner::Player(0), "Infantry Alpha", 100.0, UnitMeshType::Capsule,
            UnitBase::LightInfantry),
        (6, 10, Owner::Player(0), "Infantry Beta", 100.0, UnitMeshType::Capsule,
            UnitBase::LightInfantry),
        (7, 10, Owner::Player(0), "Hover Vehicle", 140.0, UnitMeshType::Cube,
            UnitBase::HoverVehicle {
                turn_rate: 2.5,
                forward_accel: 12.0,
                non_forward_accel: 5.0,
                drag_ratio: 2.5
            }),
        (8, 10, Owner::Player(0), "Mech Walker", 250.0, UnitMeshType::Cube,
            UnitBase::Mech {
                turn_rate: 1.8,
                max_speed: 2.2,
                acceleration: 3.5,
                deceleration: 5.0
            }),
        (14, 10, Owner::Player(1), "Wheeled APC", 150.0, UnitMeshType::Cube,
            UnitBase::WheeledVehicle { min_turn_radius: 2.0, forward_speed: 7.0, reverse_speed: 3.0 }),
        (15, 10, Owner::Player(1), "Heavy Tank", 200.0, UnitMeshType::Cube,
            UnitBase::TrackedVehicle { speed_to_turn_ratio: 1.2 }),
        (16, 10, Owner::Player(1), "Drill Unit (Above)", 180.0, UnitMeshType::Cube,
            UnitBase::DrillUnit {
                mode: DrillMode::AboveGround,
                speed_to_turn_ratio: 1.3,
                acceleration: 2.5,
                deceleration: 4.0,
                max_speed: 2.0
            }),
        (10, 10, Owner::Neutral, "Neutral Infantry", 80.0, UnitMeshType::Capsule,
            UnitBase::LightInfantry),
    ];

    let unit_count = unit_data.len();

    for (grid_x, grid_z, owner, unit_name, max_health, mesh_type, unit_base) in unit_data {
        // Convert grid position to world position (centered grid)
        let world_x = (grid_x as f32 - 10.0) + 0.5;
        let world_z = (grid_z as f32 - 10.0) + 0.5;

        // Create appropriate mesh based on unit type
        let mesh = match mesh_type {
            UnitMeshType::Capsule => meshes.add(Capsule3d::new(0.2, 0.6)),
            UnitMeshType::Cube => meshes.add(Cuboid::new(0.5, 0.5, 0.5)),
        };

        let material = materials.add(StandardMaterial {
            base_color: owner.color(),
            ..default()
        });

        let mut entity_commands = commands.spawn((
            PbrBundle {
                mesh,
                material,
                transform: Transform::from_xyz(world_x, 0.5, world_z),
                ..default()
            },
            Unit,
            UnitHealth {
                current: max_health,
                max: max_health,
            },
            owner,
            UnitType {
                name: unit_name.to_string(),
            },
            Selectable,
            GridPosition {
                x: grid_x,
                z: grid_z,
            },
            unit_base,
            MovementSpeed(unit_base.get_speed()),
            RotationSpeed(unit_base.get_rotation_speed()),
            Velocity(Vec3::ZERO),
            create_attack_capability(&unit_base),
            AttackState::default(),
            UnitCommand::Idle,
        ));

        let entity_id = entity_commands.id();

        // Add turret component if unit has a turret
        if let Some(turret) = create_turret_for_unit(&unit_base) {
            entity_commands.insert(turret);
        }

        // Spawn turret visual (if applicable)
        spawn_turret_visual(&mut commands, entity_id, &mut meshes, &mut materials, &unit_base, owner.color());
    }

    info!("Spawned {} test units", unit_count);
}

/// Mesh type for units
enum UnitMeshType {
    Capsule,
    Cube,
}

/// Check if two owners are enemies
fn is_enemy_owner(owner1: &Owner, owner2: &Owner) -> bool {
    match (owner1, owner2) {
        (Owner::Player(p1), Owner::Player(p2)) => p1 != p2,
        (Owner::Neutral, _) | (_, Owner::Neutral) => false,
    }
}

/// Create attack capability based on unit base type
fn create_attack_capability(unit_base: &UnitBase) -> AttackCapability {
    match unit_base {
        UnitBase::LightInfantry => AttackCapability {
            damage: 10.0,
            range: 5.0,
            aim_time: 0.2,
            fire_time: 0.1,
            cooldown_time: 0.05,
            reload_time: 1.0,
            attack_type: AttackType::FullyConnected, // Instant hit rifle
        },
        UnitBase::WheeledVehicle { .. } => AttackCapability {
            damage: 20.0,
            range: 8.0,
            aim_time: 0.4,
            fire_time: 0.15,
            cooldown_time: 0.1,
            reload_time: 2.5,
            attack_type: AttackType::TailDisjointed {
                projectile_speed: 20.0,
                projectile_visual: crate::projectile::ProjectileVisual::Cylinder {
                    radius: 0.1,
                    length: 0.3,
                },
            }, // Cannon shell projectile
        },
        UnitBase::TrackedVehicle { .. } => AttackCapability {
            damage: 30.0,
            range: 7.0,
            aim_time: 0.5,
            fire_time: 0.2,
            cooldown_time: 0.15,
            reload_time: 3.0,
            attack_type: AttackType::DoublyDisjointed {
                projectile_speed: 15.0,
                projectile_visual: crate::projectile::ProjectileVisual::Sphere {
                    radius: 0.15,
                },
                effect_radius: 2.0,
            }, // AOE shell
        },
        UnitBase::DrillUnit { mode, .. } => {
            // Drill units can only attack in above-ground mode
            if *mode == DrillMode::AboveGround {
                AttackCapability {
                    damage: 25.0,
                    range: 6.5,
                    aim_time: 0.45,
                    fire_time: 0.18,
                    cooldown_time: 0.12,
                    reload_time: 2.8,
                    attack_type: AttackType::DoublyDisjointed {
                        projectile_speed: 18.0,
                        projectile_visual: crate::projectile::ProjectileVisual::Sphere {
                            radius: 0.12,
                        },
                        effect_radius: 1.5,
                    }, // Medium AOE shell
                }
            } else {
                // Underground mode - cannot attack
                AttackCapability {
                    damage: 0.0,
                    range: 0.0,
                    aim_time: 0.0,
                    fire_time: 0.0,
                    cooldown_time: 0.0,
                    reload_time: 0.0,
                    attack_type: AttackType::FullyConnected,
                }
            }
        },
        UnitBase::HoverVehicle { .. } => AttackCapability {
            damage: 22.0,
            range: 8.5,
            aim_time: 0.35,
            fire_time: 0.12,
            cooldown_time: 0.08,
            reload_time: 2.2,
            attack_type: AttackType::TailDisjointed {
                projectile_speed: 25.0,
                projectile_visual: crate::projectile::ProjectileVisual::Cylinder {
                    radius: 0.08,
                    length: 0.25,
                },
            }, // Fast projectile
        },
        UnitBase::Mech { .. } => AttackCapability {
            damage: 35.0,
            range: 6.0,
            aim_time: 0.6,
            fire_time: 0.25,
            cooldown_time: 0.2,
            reload_time: 3.5,
            attack_type: AttackType::DoublyDisjointed {
                projectile_speed: 12.0,
                projectile_visual: crate::projectile::ProjectileVisual::Sphere {
                    radius: 0.18,
                },
                effect_radius: 2.5,
            }, // Heavy AOE weapon
        },
    }
}

/// System to display unit info when selected
fn unit_selection_display(
    units: Query<
        (&UnitType, &UnitHealth, &Owner),
        (With<Unit>, Added<Selected>)
    >,
) {
    for (unit_type, health, owner) in units.iter() {
        info!(
            "Unit selected: {} | Health: {}/{} | Owner: {:?}",
            unit_type.name,
            health.current,
            health.max,
            owner
        );
    }
}

/// System to handle move commands (right-click or left-click in command mode)
fn right_click_move_command(
    mut commands_ecs: Commands,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut command_mode: ResMut<CommandMode>,
    selected_units: Query<(Entity, &Transform, &UnitBase, &Owner), (With<Unit>, With<Selected>)>,
    potential_targets: Query<(Entity, &Transform, &Owner), With<Unit>>,
    tiles: Query<(&GridPosition, &TileProperties), With<Tile>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    move_markers: Query<Entity, With<MoveTargetMarker>>,
) {
    // Handle right-click (always works) or left-click when in a command mode
    let is_right_click = buttons.just_pressed(MouseButton::Right);
    let is_left_click = buttons.just_pressed(MouseButton::Left);
    let in_command_mode = command_mode.mode != CommandType::Default;

    if !is_right_click && !(is_left_click && in_command_mode) {
        return;
    }

    let window = windows.single();
    let (camera, camera_transform) = cameras.single();

    if let Some(cursor_pos) = window.cursor_position() {
        if let Some(ray) = camera.viewport_to_world(camera_transform, cursor_pos) {
            // First check if clicking on an enemy unit (for Attack command)
            let mut clicked_enemy = None;

            if command_mode.mode == CommandType::Attack {
                let selected_count = selected_units.iter().count();
                if selected_count > 0 {
                    // Get owner of first selected unit to determine enemies
                    if let Some((_, _, _, owner)) = selected_units.iter().next() {
                        // Check each potential target
                        for (target_entity, target_transform, target_owner) in potential_targets.iter() {
                            // Skip if not enemy
                            if !is_enemy_owner(owner, target_owner) {
                                continue;
                            }

                            let target_pos = target_transform.translation;

                            // Simple sphere intersection check
                            let to_target = target_pos - ray.origin;
                            let projection = to_target.dot(*ray.direction);

                            if projection > 0.0 {
                                let closest_point = ray.origin + *ray.direction * projection;
                                let distance_to_target = (closest_point - target_pos).length();

                                if distance_to_target < 0.5 {
                                    clicked_enemy = Some(target_entity);
                                    break;
                                }
                            }
                        }
                    }
                }
            }

            // Handle attack on clicked enemy
            if let Some(target_entity) = clicked_enemy {
                let selected_count = selected_units.iter().count();
                for (entity, _, _, _) in selected_units.iter() {
                    commands_ecs.entity(entity)
                        .remove::<crate::commands::HoldingPosition>()
                        .insert(UnitCommand::AttackTarget(target_entity));
                }

                info!("Attack command: {} unit(s) targeting enemy", selected_count);
                command_mode.mode = CommandType::Default;
                return;
            }

            // Raycast to ground plane (y = 0)
            if ray.direction.y.abs() > 0.001 {
                let t = -ray.origin.y / ray.direction.y;
                if t > 0.0 {
                    let target_pos = ray.origin + *ray.direction * t;
                    let target_grid = world_to_grid(target_pos);

                    // Issue command based on current command mode
                    let selected_count = selected_units.iter().count();
                    if selected_count > 0 {
                        match command_mode.mode {
                            CommandType::Move | CommandType::Default => {
                                // Move command
                                for (entity, transform, unit_base, _owner) in selected_units.iter() {
                                    let start_grid = world_to_grid(transform.translation);

                                    // Calculate path using A* with unit type awareness
                                    if let Some(path) = find_path(start_grid, target_grid, &tiles, unit_base) {
                                        let smoothed_waypoints = smooth_path(path);

                                        commands_ecs.entity(entity)
                                            .remove::<crate::commands::HoldingPosition>()
                                            .insert((
                                                MoveTarget(target_pos),
                                                Path {
                                                    waypoints: smoothed_waypoints,
                                                    current_waypoint: 0,
                                                },
                                                UnitCommand::Move(target_pos),
                                            ));
                                    } else {
                                        warn!("No path found for unit to ({}, {})", target_grid.x, target_grid.z);
                                    }
                                }

                                info!("Move command: {} unit(s) to ({:.1}, {:.1})",
                                    selected_count, target_pos.x, target_pos.z);

                                // Reset to default mode
                                command_mode.mode = CommandType::Default;
                            }

                            CommandType::Patrol => {
                                // Patrol command - set up patrol between current position and target
                                for (entity, transform, unit_base, _owner) in selected_units.iter() {
                                    let start_pos = transform.translation;
                                    let start_grid = world_to_grid(start_pos);

                                    // Calculate initial path to target
                                    if let Some(path) = find_path(start_grid, target_grid, &tiles, unit_base) {
                                        let smoothed_waypoints = smooth_path(path);

                                        commands_ecs.entity(entity)
                                            .remove::<crate::commands::HoldingPosition>()
                                            .insert((
                                                MoveTarget(target_pos),
                                                Path {
                                                    waypoints: smoothed_waypoints,
                                                    current_waypoint: 0,
                                                },
                                                UnitCommand::Patrol {
                                                    start: start_pos,
                                                    end: target_pos,
                                                    going_to_end: true,
                                                },
                                            ));
                                    }
                                }

                                info!("Patrol command: {} unit(s) to ({:.1}, {:.1})",
                                    selected_count, target_pos.x, target_pos.z);

                                // Reset to default mode
                                command_mode.mode = CommandType::Default;
                            }

                            CommandType::Attack => {
                                // Attack mode clicked on ground (no enemy found)
                                // Just reset to default mode - attack on enemy was handled above
                                info!("Attack mode: No enemy at click location, returning to default mode");
                                command_mode.mode = CommandType::Default;
                            }

                            CommandType::AttackGround => {
                                // Attack Ground command - issue AttackLocation to selected units
                                for (entity, transform, unit_base, _owner) in selected_units.iter() {
                                    let start_grid = world_to_grid(transform.translation);

                                    // Calculate path to the attack location
                                    if let Some(path) = find_path(start_grid, target_grid, &tiles, unit_base) {
                                        let smoothed_waypoints = smooth_path(path);

                                        commands_ecs.entity(entity)
                                            .remove::<crate::commands::HoldingPosition>()
                                            .insert((
                                                MoveTarget(target_pos),
                                                Path {
                                                    waypoints: smoothed_waypoints,
                                                    current_waypoint: 0,
                                                },
                                                UnitCommand::AttackLocation(target_pos),
                                            ));
                                    } else {
                                        warn!("No path found for unit to ({}, {})", target_grid.x, target_grid.z);
                                    }
                                }

                                info!("Attack Ground command: {} unit(s) to ({:.1}, {:.1})",
                                    selected_count, target_pos.x, target_pos.z);

                                // Reset to default mode
                                command_mode.mode = CommandType::Default;
                            }
                        }

                        // Remove old move markers
                        for marker_entity in move_markers.iter() {
                            commands_ecs.entity(marker_entity).despawn();
                        }

                        // Spawn visual move target marker
                        let marker_mesh = meshes.add(Cylinder::new(0.3, 0.05));
                        let marker_material = materials.add(StandardMaterial {
                            base_color: Color::srgb(0.0, 1.0, 0.0),
                            emissive: Color::srgb(0.0, 0.8, 0.0).into(),
                            ..default()
                        });

                        commands_ecs.spawn((
                            PbrBundle {
                                mesh: marker_mesh,
                                material: marker_material,
                                transform: Transform::from_xyz(target_pos.x, 0.05, target_pos.z),
                                ..default()
                            },
                            MoveTargetMarker,
                        ));
                    }
                }
            }
        }
    }
}

/// System to handle unit movement toward target
fn unit_movement_system(
    mut commands: Commands,
    time: Res<Time>,
    mut units: Query<
        (Entity, &mut Transform, &mut Velocity, &MovementSpeed, &MoveTarget, Option<&mut Path>, Option<&AttackState>, &mut UnitCommand),
        (With<Unit>, Without<crate::commands::HoldingPosition>)
    >,
) {
    let delta = time.delta_seconds();

    for (entity, mut transform, mut velocity, speed, _target, path_option, attack_state_opt, mut unit_command) in units.iter_mut() {
        // Stop moving if in Firing or Cooldown phase
        if let Some(attack_state) = attack_state_opt {
            if matches!(attack_state.phase, crate::combat::AttackPhase::Firing | crate::combat::AttackPhase::Cooldown) {
                velocity.0 = Vec3::ZERO;
                continue;
            }
        }

        let current_pos = transform.translation;

        // Determine next waypoint
        let next_waypoint = if let Some(mut path) = path_option {
            // Following a path - get current waypoint
            if path.current_waypoint >= path.waypoints.len() {
                // Reached end of path
                velocity.0 = Vec3::ZERO;
                commands.entity(entity).remove::<MoveTarget>().remove::<Path>();
                if matches!(*unit_command, UnitCommand::Move(_)) {
                    *unit_command = UnitCommand::Idle;
                }
                continue;
            }

            let waypoint = path.waypoints[path.current_waypoint];

            // Check if we reached this waypoint
            let to_waypoint = waypoint - current_pos;
            let distance_to_waypoint = Vec3::new(to_waypoint.x, 0.0, to_waypoint.z).length();

            if distance_to_waypoint < 0.3 {
                // Reached waypoint, move to next
                path.current_waypoint += 1;

                if path.current_waypoint >= path.waypoints.len() {
                    // Reached end of path
                    velocity.0 = Vec3::ZERO;
                    commands.entity(entity).remove::<MoveTarget>().remove::<Path>();
                    if matches!(*unit_command, UnitCommand::Move(_)) {
                        *unit_command = UnitCommand::Idle;
                    }
                    continue;
                }

                path.waypoints[path.current_waypoint]
            } else {
                waypoint
            }
        } else {
            // No path - this shouldn't happen but handle gracefully
            velocity.0 = Vec3::ZERO;
            commands.entity(entity).remove::<MoveTarget>();
            if matches!(*unit_command, UnitCommand::Move(_)) {
                *unit_command = UnitCommand::Idle;
            }
            continue;
        };

        // Calculate direction to next waypoint (ignore y axis)
        let direction_3d = next_waypoint - current_pos;
        let direction_2d = Vec3::new(direction_3d.x, 0.0, direction_3d.z);
        let distance = direction_2d.length();

        // Check if we've reached the waypoint
        if distance < 0.1 {
            continue;
        }

        let normalized_direction = direction_2d.normalize();

        // Simple acceleration toward waypoint
        let acceleration = 8.0; // units/sec^2
        let decel_distance = 1.5; // Start decelerating when this close

        if distance < decel_distance {
            // Decelerate when close
            let target_speed = (distance / decel_distance) * speed.0;
            let desired_velocity = normalized_direction * target_speed;
            velocity.0 = velocity.0.lerp(desired_velocity, acceleration * delta);
        } else {
            // Accelerate to max speed
            let desired_velocity = normalized_direction * speed.0;
            velocity.0 = velocity.0.lerp(desired_velocity, acceleration * delta);
        }

        // Update position
        transform.translation += velocity.0 * delta;
        transform.translation.y = 0.5; // Keep units at constant height
    }
}

/// System to handle unit rotation toward movement direction
fn unit_rotation_system(
    time: Res<Time>,
    mut units: Query<
        (&mut Transform, &Velocity, &RotationSpeed),
        With<Unit>
    >,
) {
    let delta = time.delta_seconds();

    for (mut transform, velocity, rotation_speed) in units.iter_mut() {
        // Only rotate if moving
        if velocity.0.length() > 0.1 {
            // Calculate target direction (ignore y)
            let direction = Vec3::new(velocity.0.x, 0.0, velocity.0.z).normalize();

            // Calculate target rotation
            let target_rotation = Quat::from_rotation_y(
                direction.x.atan2(direction.z)
            );

            // Smoothly interpolate rotation
            let rotation_speed_factor = rotation_speed.0 * delta;
            transform.rotation = transform.rotation.slerp(target_rotation, rotation_speed_factor);
        }
    }
}
