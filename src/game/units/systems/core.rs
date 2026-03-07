use bevy::prelude::*;
use crate::types::*;
use crate::game::types::{ObjectInstance, StructureInstance};
use crate::game::world::types::{Tile, TilePreset, GridMap};
use crate::game::combat::types::{AttackState, Turret, IdleOrigin, Armor, Silhouette, SeparationRadius, SEPARATION_FORCE_SCALE};
use crate::game::units::types::types::{CommandIndicator, CommandIndicatorType, command_indicator_color, command_has_indicator};
use crate::game::combat::utils::{create_turret_for_unit, spawn_turret_visual};
use crate::game::units::types::movement::TurnRateMovementParams;
use crate::game::utils::spawn_peacekeeper;
use crate::ui::types::{CursorTarget, CursorTargetEnum, ObjectInterfaceState};
use crate::game::units::types::*;
use crate::game::units::utils::{world_to_grid, create_attack_capability, smooth_path, clear_movement_state_full};
use crate::game::types::{SupplyTowerState, SupplyChopperState, TunnelState};
use crate::game::world::types::{SupplyDeliveryStation, SpaceCrystalPatch};
use crate::game::units::types::state::AgentCarryState;

/// Distance threshold for considering a waypoint reached during movement (grid units).
/// Must be large enough that a unit at max speed cannot overshoot in a single frame.
/// At 16 FPS fixed timestep (delta ~0.0625s) and max speed ~4-8 gu/s, per-frame movement is 0.25-0.5 gu.
const WAYPOINT_ARRIVAL_THRESHOLD: f32 = 0.5;

/// Spawn test units on the map — GDO player units are now proper Peacekeepers
#[allow(dead_code)]
pub fn spawn_test_units(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Spawn GDO Peacekeepers for Player 0 (near DC at 30,30)
    let peacekeeper_positions = [(28, 32), (29, 32), (30, 32), (31, 32)];
    for (grid_x, grid_z) in peacekeeper_positions {
        spawn_peacekeeper(
            &mut commands, &mut meshes, &mut materials,
            grid_x, grid_z, Owner::player(0),
        );
    }

    // Enemy test units (Player 1) — still using placeholder stats until their unit types are defined
    let enemy_data: [(i32, i32, &str, f32, UnitMeshType, UnitBaseEnum, f32, f32); 3] = [
        (44, 32, "Wheeled APC", 150.0, UnitMeshType::Cube,
            UnitBaseEnum::WheeledVehicle, 7.0, 3.0),
        (45, 32, "Heavy Tank", 200.0, UnitMeshType::Cube,
            UnitBaseEnum::TrackedVehicle, 2.5, 1.57),
        (46, 32, "Drill Unit", 180.0, UnitMeshType::Cube,
            UnitBaseEnum::DrillUnit, 2.0, 1.2),
    ];

    let owner = Owner::player(1);

    for (grid_x, grid_z, unit_name, max_health, mesh_type, unit_base, speed, rot_speed) in enemy_data {
        let world_x = (grid_x as f32 - 32.0) + 0.5;
        let world_z = (grid_z as f32 - 32.0) + 0.5;

        let mesh = match mesh_type {
            UnitMeshType::Capsule => meshes.add(Capsule3d::new(0.2, 0.6)),
            UnitMeshType::Cube => meshes.add(Cuboid::new(0.5, 0.5, 0.5)),
        };

        let material = materials.add(StandardMaterial {
            base_color: owner.color(),
            ..default()
        });

        let has_turret = unit_base.data().has_turret;

        let mut entity_commands = commands.spawn((
            PbrBundle {
                mesh,
                material,
                transform: Transform::from_xyz(world_x, 0.5, world_z),
                ..default()
            },
            Unit,
            ObjectInstance::destructible(ObjectEnum::Peacekeeper, max_health),
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
            unit_base.data().domain,
            MovementSpeed(speed),
            RotationSpeed(rot_speed),
            Velocity(Vec3::ZERO),
            create_attack_capability(&unit_base),
            AttackState::default(),
            UnitCommand::Idle,
        ));

        let base_data = unit_base.data();
        entity_commands.insert((
            CommandQueue::new(),
            BaseCommandState::default(),
            BaseBehaviorState::default(),
            LocomotionChannel::default(),
            OrientationChannel::default(),
            UnitControlCost(1), // TODO: Use actual cost from unit type data when defined
        ));

        // Armor and silhouette — placeholder values until unit type data is defined
        entity_commands.insert((
            Armor {
                point_armor: 5.0, // Placeholder
                full_armor: 3.0,  // Placeholder
                directional_armor: base_data.directional_armor,
            },
            Silhouette {
                width: 0.5,  // Placeholder
                height: 0.5, // Placeholder
            },
        ));

        if has_turret {
            // Turret units: turret channels handle attacks
            entity_commands.insert((
                TurretCommandState::default(),
                TurretBehaviorState::default(),
                TurretOrientationChannel::default(),
                TurretAttackChannel::default(),
            ));
        } else {
            // Non-turret units: base handles attacks
            entity_commands.insert(BaseAttackChannel::default());
        }

        let entity_id = entity_commands.id();

        if let Some(turret) = create_turret_for_unit(&unit_base) {
            entity_commands.insert(turret);
        }

        spawn_turret_visual(&mut commands, entity_id, &mut meshes, &mut materials, &unit_base, owner.color());
    }

    info!("Spawned 4 Peacekeepers (Player 0) and 3 enemy test units (Player 1)");
}

/// System to sync GridPosition from Transform for all units each frame.
/// Movement systems update Transform but not GridPosition; this system keeps
/// GridPosition accurate so fog-of-war, combat range checks, and other
/// grid-based queries work correctly for moving units.
pub fn grid_position_sync_system(
    mut units: Query<(&Transform, &mut GridPosition), With<Unit>>,
) {
    for (transform, mut grid_pos) in units.iter_mut() {
        let new_pos = world_to_grid(transform.translation);
        if grid_pos.x != new_pos.x || grid_pos.z != new_pos.z {
            grid_pos.x = new_pos.x;
            grid_pos.z = new_pos.z;
        }
    }
}

/// System to display unit info when selected
pub fn unit_selection_display(
    units: Query<
        (&UnitType, &ObjectInstance, &Owner),
        (With<Unit>, Added<Selected>)
    >,
) {
    for (unit_type, obj, owner) in units.iter() {
        info!(
            "Unit selected: {} | Health: {}/{} | Owner: {:?}",
            unit_type.name,
            obj.hp.unwrap_or(0.0),
            obj.max_hp.unwrap_or(0.0),
            owner
        );
    }
}

/// System to handle move commands (right-click or left-click in command mode).
/// Uses CursorTarget resource instead of inline raycasting, and ObjectInterfaceState
/// instead of CommandMode resource.
pub fn right_click_move_command(
    mut commands_ecs: Commands,
    buttons: Res<ButtonInput<MouseButton>>,
    mut interface_state: ResMut<ObjectInterfaceState>,
    cursor_target: Res<CursorTarget>,
    selected_units: Query<(Entity, &Transform, &UnitBaseEnum, &Owner, Option<&AttackState>, Option<&SupplyChopperState>, &ObjectInstance, Option<&AgentCarryState>), (With<Unit>, With<Selected>)>,
    target_info: Query<(Option<&SupplyDeliveryStation>, Option<&SupplyTowerState>, &Owner, Option<&SpaceCrystalPatch>, Option<&TunnelState>), With<ObjectInstance>>,
    tiles: Query<(&GridPosition, &TilePreset), With<Tile>>,
    grid: Res<GridMap>,
    local_player: Res<LocalPlayer>,
    occupancy: Res<OccupancyMap>,
) {
    let is_right_click = buttons.just_pressed(MouseButton::Right);
    let is_left_click = buttons.just_pressed(MouseButton::Left);
    let in_command_mode = interface_state.is_awaiting_target();

    if !is_right_click && !(is_left_click && in_command_mode) {
        return;
    }

    // CursorTarget is already None when cursor is over UI
    if cursor_target.kind == CursorTargetEnum::None {
        return;
    }

    // Don't process move commands during placement mode — right-click cancels placement instead
    if interface_state.is_placement_mode() {
        return;
    }

    let command_type = interface_state.awaiting_command_type().unwrap_or(CommandType::Default);

    // Check if any selected unit is a Supply Chopper
    let has_selected_choppers = selected_units.iter().any(|(_, _, _, _, _, chopper_state, _, _)| chopper_state.is_some());
    let has_selected_agents = selected_units.iter().any(|(_, _, _, _, _, _, obj, _)| obj.object_type == ObjectEnum::SyndicateAgent);

    // Handle entity click (Attack mode left-click or right-click entity detection)
    if let Some(target_entity) = cursor_target.entity {
        let is_enemy = cursor_target.kind == CursorTargetEnum::EnemyObject;

        let should_attack = match command_type {
            CommandType::Attack => true,
            CommandType::Default if is_right_click && is_enemy => true,
            _ => false,
        };

        if should_attack {
            let selected_count = selected_units.iter().count();
            for (entity, _, _, _, attack_state_opt, _, _, _) in selected_units.iter() {
                if let Some(attack_state) = attack_state_opt {
                    if !attack_state.phase.is_interruptible() {
                        continue;
                    }
                }
                let mut entity_cmds = commands_ecs.entity(entity);
                clear_movement_state_full(&mut entity_cmds);
                entity_cmds.insert(UnitCommand::AttackTarget(target_entity));
            }

            // Attack target highlights are now handled by command_indicator_sync_system

            info!("Attack command: {} unit(s) targeting entity", selected_count);
            *interface_state = ObjectInterfaceState::Default;
            return;
        }

        // Right-click on non-enemy entity: check for chopper-specific targets
        if is_right_click && command_type == CommandType::Default && has_selected_choppers {
            if let Ok((sds_opt, st_opt, target_owner, _, _)) = target_info.get(target_entity) {
                if sds_opt.is_some() {
                    for (entity, _, _, _, attack_state_opt, chopper_opt, _, _) in selected_units.iter() {
                        if chopper_opt.is_some() {
                            if let Some(attack_state) = attack_state_opt {
                                if !attack_state.phase.is_interruptible() {
                                    continue;
                                }
                            }
                            let mut entity_cmds = commands_ecs.entity(entity);
                            clear_movement_state_full(&mut entity_cmds);
                            entity_cmds.insert(UnitCommand::PickUpSupplies(target_entity));
                        }
                    }
                    info!("Supply Chopper: PickUpSupplies from SDS");
                    *interface_state = ObjectInterfaceState::Default;
                    return;
                }
                if st_opt.is_some() && target_owner.player_number() == Some(local_player.0) {
                    for (entity, _, _, _, attack_state_opt, chopper_opt, _, _) in selected_units.iter() {
                        if chopper_opt.is_some() {
                            if let Some(attack_state) = attack_state_opt {
                                if !attack_state.phase.is_interruptible() {
                                    continue;
                                }
                            }
                            let mut entity_cmds = commands_ecs.entity(entity);
                            clear_movement_state_full(&mut entity_cmds);
                            entity_cmds.insert(UnitCommand::AttachToTower(target_entity));
                        }
                    }
                    info!("Supply Chopper: AttachToTower");
                    *interface_state = ObjectInterfaceState::Default;
                    return;
                }
            }
        }
        // Right-click on non-enemy entity: check for Agent-specific targets
        if is_right_click && command_type == CommandType::Default && has_selected_agents {
            if let Ok((sds_opt, _st_opt, target_owner, crystal_opt, tunnel_opt)) = target_info.get(target_entity) {
                // Crystal patch → Gather
                if crystal_opt.is_some() {
                    for (entity, _, _, _, attack_state_opt, _, obj, _) in selected_units.iter() {
                        if obj.object_type == ObjectEnum::SyndicateAgent {
                            if let Some(attack_state) = attack_state_opt {
                                if !attack_state.phase.is_interruptible() {
                                    continue;
                                }
                            }
                            let mut entity_cmds = commands_ecs.entity(entity);
                            clear_movement_state_full(&mut entity_cmds);
                            entity_cmds.insert(UnitCommand::Gather(target_entity));
                        }
                    }
                    info!("Agent: Gather crystals");
                    *interface_state = ObjectInterfaceState::Default;
                    return;
                }
                // Supply Delivery Station → Gather (supplies)
                if sds_opt.is_some() {
                    for (entity, _, _, _, attack_state_opt, _, obj, _) in selected_units.iter() {
                        if obj.object_type == ObjectEnum::SyndicateAgent {
                            if let Some(attack_state) = attack_state_opt {
                                if !attack_state.phase.is_interruptible() {
                                    continue;
                                }
                            }
                            let mut entity_cmds = commands_ecs.entity(entity);
                            clear_movement_state_full(&mut entity_cmds);
                            entity_cmds.insert(UnitCommand::Gather(target_entity));
                        }
                    }
                    info!("Agent: Gather supplies");
                    *interface_state = ObjectInterfaceState::Default;
                    return;
                }
                // Own Tunnel → DropOff (if carrying) or Enter (if not)
                if tunnel_opt.is_some() && target_owner.player_number() == Some(local_player.0) {
                    for (entity, _, _, _, attack_state_opt, _, obj, carry_opt) in selected_units.iter() {
                        if obj.object_type == ObjectEnum::SyndicateAgent {
                            if let Some(attack_state) = attack_state_opt {
                                if !attack_state.phase.is_interruptible() {
                                    continue;
                                }
                            }
                            let mut entity_cmds = commands_ecs.entity(entity);
                            clear_movement_state_full(&mut entity_cmds);
                            if carry_opt.map(|cs| cs.is_carrying()).unwrap_or(false) {
                                entity_cmds.insert(UnitCommand::DropOffResources(target_entity));
                            } else {
                                entity_cmds.insert(UnitCommand::Enter(target_entity));
                            }
                        }
                    }
                    info!("Agent: Tunnel interaction (drop-off or enter)");
                    *interface_state = ObjectInterfaceState::Default;
                    return;
                }
            }
        }
        // If right-click on non-enemy entity, fall through to ground Move below
    }

    // Ground-based commands using CursorTarget.location
    let Some(target_pos) = cursor_target.location else { return };
    let target_grid = world_to_grid(target_pos);

    let selected_count = selected_units.iter().count();
    if selected_count == 0 {
        return;
    }

    match command_type {
        CommandType::Move | CommandType::Default => {
            for (entity, transform, unit_base, _owner, attack_state_opt, _, _, _) in selected_units.iter() {
                if let Some(attack_state) = attack_state_opt {
                    if !attack_state.phase.is_interruptible() {
                        continue;
                    }
                }
                let start_grid = world_to_grid(transform.translation);
                if let Some(path) = crate::game::units::pathfinding::find_path(start_grid, target_grid, &tiles, unit_base, grid.width as i32, grid.height as i32, &occupancy, (start_grid.x, start_grid.z)) {
                    let smoothed_waypoints = smooth_path(path);
                    commands_ecs.entity(entity)
                        .remove::<HoldingPosition>()
                        .remove::<IdleOrigin>()
                        .insert((
                            MoveTarget(target_pos),
                            Path { waypoints: smoothed_waypoints, current_waypoint: 0 },
                            UnitCommand::Move(target_pos),
                        ));
                } else {
                    warn!("No path found for unit to ({}, {})", target_grid.x, target_grid.z);
                }
            }
            info!("Move command: {} unit(s) to ({:.1}, {:.1})", selected_count, target_pos.x, target_pos.z);
            *interface_state = ObjectInterfaceState::Default;
        }

        CommandType::Patrol => {
            for (entity, transform, unit_base, _owner, attack_state_opt, _, _, _) in selected_units.iter() {
                if let Some(attack_state) = attack_state_opt {
                    if !attack_state.phase.is_interruptible() {
                        continue;
                    }
                }
                let start_pos = transform.translation;
                let start_grid = world_to_grid(start_pos);
                if let Some(path) = crate::game::units::pathfinding::find_path(start_grid, target_grid, &tiles, unit_base, grid.width as i32, grid.height as i32, &occupancy, (start_grid.x, start_grid.z)) {
                    let smoothed_waypoints = smooth_path(path);
                    commands_ecs.entity(entity)
                        .remove::<HoldingPosition>()
                        .remove::<IdleOrigin>()
                        .insert((
                            MoveTarget(target_pos),
                            Path { waypoints: smoothed_waypoints, current_waypoint: 0 },
                            UnitCommand::Patrol { start: start_pos, end: target_pos, going_to_end: true },
                        ));
                }
            }
            info!("Patrol command: {} unit(s) to ({:.1}, {:.1})", selected_count, target_pos.x, target_pos.z);
            *interface_state = ObjectInterfaceState::Default;
        }

        CommandType::Attack => {
            // Attack + ground click = AttackMove to that location
            for (entity, transform, unit_base, _owner, attack_state_opt, _, _, _) in selected_units.iter() {
                if let Some(attack_state) = attack_state_opt {
                    if !attack_state.phase.is_interruptible() {
                        continue;
                    }
                }
                let start_grid = world_to_grid(transform.translation);
                if let Some(path) = crate::game::units::pathfinding::find_path(start_grid, target_grid, &tiles, unit_base, grid.width as i32, grid.height as i32, &occupancy, (start_grid.x, start_grid.z)) {
                    let smoothed_waypoints = smooth_path(path);
                    commands_ecs.entity(entity)
                        .remove::<HoldingPosition>()
                        .remove::<IdleOrigin>()
                        .insert((
                            MoveTarget(target_pos),
                            Path { waypoints: smoothed_waypoints, current_waypoint: 0 },
                            UnitCommand::AttackMove(target_pos),
                        ));
                }
            }
            info!("Attack Move (from Attack+ground): {} unit(s) to ({:.1}, {:.1})", selected_count, target_pos.x, target_pos.z);
            *interface_state = ObjectInterfaceState::Default;
        }

        CommandType::AttackMove => {
            for (entity, transform, unit_base, _owner, attack_state_opt, _, _, _) in selected_units.iter() {
                if let Some(attack_state) = attack_state_opt {
                    if !attack_state.phase.is_interruptible() {
                        continue;
                    }
                }
                let start_grid = world_to_grid(transform.translation);
                if let Some(path) = crate::game::units::pathfinding::find_path(start_grid, target_grid, &tiles, unit_base, grid.width as i32, grid.height as i32, &occupancy, (start_grid.x, start_grid.z)) {
                    let smoothed_waypoints = smooth_path(path);
                    commands_ecs.entity(entity)
                        .remove::<HoldingPosition>()
                        .remove::<IdleOrigin>()
                        .insert((
                            MoveTarget(target_pos),
                            Path { waypoints: smoothed_waypoints, current_waypoint: 0 },
                            UnitCommand::AttackMove(target_pos),
                        ));
                }
            }
            info!("Attack Move command: {} unit(s) to ({:.1}, {:.1})", selected_count, target_pos.x, target_pos.z);
            *interface_state = ObjectInterfaceState::Default;
        }

        CommandType::AttackGround => {
            for (entity, transform, unit_base, _owner, attack_state_opt, _, _, _) in selected_units.iter() {
                if let Some(attack_state) = attack_state_opt {
                    if !attack_state.phase.is_interruptible() {
                        continue;
                    }
                }
                let start_grid = world_to_grid(transform.translation);
                if let Some(path) = crate::game::units::pathfinding::find_path(start_grid, target_grid, &tiles, unit_base, grid.width as i32, grid.height as i32, &occupancy, (start_grid.x, start_grid.z)) {
                    let smoothed_waypoints = smooth_path(path);
                    commands_ecs.entity(entity)
                        .remove::<HoldingPosition>()
                        .remove::<IdleOrigin>()
                        .insert((
                            MoveTarget(target_pos),
                            Path { waypoints: smoothed_waypoints, current_waypoint: 0 },
                            UnitCommand::AttackLocation(target_pos),
                        ));
                }
            }
            info!("Attack Ground command: {} unit(s) to ({:.1}, {:.1})", selected_count, target_pos.x, target_pos.z);
            *interface_state = ObjectInterfaceState::Default;
        }

        CommandType::Reverse => {
            for (entity, transform, unit_base, _owner, attack_state_opt, _, _, _) in selected_units.iter() {
                if let Some(attack_state) = attack_state_opt {
                    if !attack_state.phase.is_interruptible() {
                        continue;
                    }
                }
                if !unit_base.data().can_reverse {
                    continue;
                }
                let start_grid = world_to_grid(transform.translation);
                if let Some(path) = crate::game::units::pathfinding::find_path(start_grid, target_grid, &tiles, unit_base, grid.width as i32, grid.height as i32, &occupancy, (start_grid.x, start_grid.z)) {
                    let smoothed_waypoints = smooth_path(path);
                    commands_ecs.entity(entity)
                        .remove::<HoldingPosition>()
                        .remove::<IdleOrigin>()
                        .insert((
                            MoveTarget(target_pos),
                            Path { waypoints: smoothed_waypoints, current_waypoint: 0 },
                            UnitCommand::Reverse(target_pos),
                        ));
                }
            }
            info!("Reverse command: {} unit(s) to ({:.1}, {:.1})", selected_count, target_pos.x, target_pos.z);
            *interface_state = ObjectInterfaceState::Default;
        }

        CommandType::Enter => {
            // Enter mode requires clicking a tunnel entity, not ground.
            // Ground click in Enter mode does nothing — just reset mode.
            *interface_state = ObjectInterfaceState::Default;
        }

        CommandType::Build | CommandType::BuildTunnel => {
            // Build mode requires a placement click through the placement system,
            // not a ground click in the right-click handler. Reset mode.
            *interface_state = ObjectInterfaceState::Default;
        }

        CommandType::Gather | CommandType::DropOff => {
            // Gather and DropOff require clicking a target entity, not ground.
            // Ground click resets mode.
            *interface_state = ObjectInterfaceState::Default;
        }
    }

    // Move target markers are now handled by command_indicator_sync_system
}

/// System to handle unit movement toward target (fallback for non-TurnRate units).
/// Includes ground collision: checks new position against OccupancyMap before applying.
pub fn unit_movement_system(
    mut commands: Commands,
    time: Res<Time>,
    occupancy: Res<OccupancyMap>,
    mut units: Query<
        (Entity, &mut Transform, &mut Velocity, &MovementSpeed, &MoveTarget, Option<&mut Path>,
         Option<&AttackState>, Option<&Turret>, &mut UnitCommand, Option<&Silhouette>, Option<&DomainEnum>),
        (With<Unit>, Without<HoldingPosition>, Without<TurnRateMovementParams>)
    >,
) {
    let delta = time.delta_seconds();

    for (entity, mut transform, mut velocity, speed, _target, path_option,
         attack_state_opt, turret_opt, mut unit_command, silhouette_opt, domain_opt) in units.iter_mut()
    {
        // Check attack phase action constraints — turret-source units can move freely
        if let Some(attack_state) = attack_state_opt {
            let is_turret_source = turret_opt.is_some();
            let constraints = attack_state.phase.base_action_constraints(is_turret_source);
            if !constraints.base_can_move {
                velocity.0 = Vec3::ZERO;
                continue;
            }
        }

        let current_pos = transform.translation;

        let next_waypoint = if let Some(mut path) = path_option {
            if path.current_waypoint >= path.waypoints.len() {
                velocity.0 = Vec3::ZERO;
                commands.entity(entity).remove::<MoveTarget>().remove::<Path>();
                if matches!(*unit_command, UnitCommand::Move(_) | UnitCommand::AttackMove(_) | UnitCommand::Reverse(_)) {
                    *unit_command = UnitCommand::Idle;
                }
                continue;
            }

            let waypoint = path.waypoints[path.current_waypoint];
            let to_waypoint = waypoint - current_pos;
            let distance_to_waypoint = Vec3::new(to_waypoint.x, 0.0, to_waypoint.z).length();

            if distance_to_waypoint < WAYPOINT_ARRIVAL_THRESHOLD {
                path.current_waypoint += 1;

                if path.current_waypoint >= path.waypoints.len() {
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
            velocity.0 = Vec3::ZERO;
            commands.entity(entity).remove::<MoveTarget>();
            if matches!(*unit_command, UnitCommand::Move(_)) {
                *unit_command = UnitCommand::Idle;
            }
            continue;
        };

        let direction_3d = next_waypoint - current_pos;
        let direction_2d = Vec3::new(direction_3d.x, 0.0, direction_3d.z);
        let distance = direction_2d.length();

        if distance < 0.1 {
            // Very close to waypoint — snap position and let the next frame advance waypoint
            transform.translation.x = next_waypoint.x;
            transform.translation.z = next_waypoint.z;
            velocity.0 = Vec3::ZERO;
            continue;
        }

        let normalized_direction = direction_2d.normalize();
        let acceleration = 8.0;
        let decel_distance = 1.5;

        if distance < decel_distance {
            let target_speed = (distance / decel_distance) * speed.0;
            let desired_velocity = normalized_direction * target_speed;
            velocity.0 = velocity.0.lerp(desired_velocity, acceleration * delta);
        } else {
            let desired_velocity = normalized_direction * speed.0;
            velocity.0 = velocity.0.lerp(desired_velocity, acceleration * delta);
        }

        let proposed_pos = transform.translation + velocity.0 * delta;

        // Ground collision check — only for ground-domain units with a silhouette
        let is_ground = domain_opt.map_or(true, |d| *d == DomainEnum::Ground);
        if is_ground {
            if let Some(sil) = silhouette_opt {
                let half_w = sil.width / 2.0;
                let half_h = sil.height / 2.0;
                if occupancy.check_movement_collision(entity, proposed_pos.x, proposed_pos.z, half_w, half_h) {
                    // Collision detected — stop and request repath
                    velocity.0 = Vec3::ZERO;
                    commands.entity(entity).remove::<Path>().insert(NeedsRepath);
                    continue;
                }
            }
        }

        transform.translation = proposed_pos;
        transform.translation.y = 0.5;
    }
}

/// System to handle unit rotation toward movement direction (fallback for non-TurnRate units)
pub fn unit_rotation_system(
    time: Res<Time>,
    mut units: Query<
        (&mut Transform, &Velocity, &RotationSpeed),
        (With<Unit>, Without<TurnRateMovementParams>)
    >,
) {
    let delta = time.delta_seconds();

    for (mut transform, velocity, rotation_speed) in units.iter_mut() {
        if velocity.0.length() > 0.1 {
            let direction = Vec3::new(velocity.0.x, 0.0, velocity.0.z).normalize();
            let target_rotation = Quat::from_rotation_y(
                direction.x.atan2(direction.z)
            );
            let rotation_speed_factor = rotation_speed.0 * delta;
            transform.rotation = transform.rotation.slerp(target_rotation, rotation_speed_factor);
        }
    }
}

/// TurnRate movement system — handles movement for entities with TurnRateMovementParams.
/// Units turn toward waypoints and move forward in their facing direction.
/// Includes ground collision checks via OccupancyMap.
pub fn turn_rate_movement_system(
    mut commands: Commands,
    time: Res<Time>,
    occupancy: Res<OccupancyMap>,
    mut units: Query<
        (Entity, &mut Transform, &mut Velocity, &TurnRateMovementParams, &MoveTarget,
         Option<&mut Path>, Option<&AttackState>, Option<&Turret>, &mut UnitCommand,
         Option<&Silhouette>, Option<&DomainEnum>),
        (With<Unit>, Without<HoldingPosition>)
    >,
) {
    let delta = time.delta_seconds();
    if delta < 0.0001 {
        return;
    }

    for (entity, mut transform, mut velocity, params, _target, path_option,
         attack_state_opt, turret_opt, mut unit_command, silhouette_opt, domain_opt) in units.iter_mut()
    {
        // Check attack phase action constraints — turret-source units can move freely
        if let Some(attack_state) = attack_state_opt {
            let is_turret_source = turret_opt.is_some();
            let constraints = attack_state.phase.base_action_constraints(is_turret_source);
            if !constraints.base_can_move {
                velocity.0 = Vec3::ZERO;
                continue;
            }
        }

        let current_pos = transform.translation;

        // Get next waypoint from path
        let next_waypoint = if let Some(mut path) = path_option {
            if path.current_waypoint >= path.waypoints.len() {
                velocity.0 = Vec3::ZERO;
                commands.entity(entity).remove::<MoveTarget>().remove::<Path>();
                if matches!(*unit_command, UnitCommand::Move(_) | UnitCommand::AttackMove(_) | UnitCommand::Reverse(_)) {
                    *unit_command = UnitCommand::Idle;
                }
                continue;
            }

            let waypoint = path.waypoints[path.current_waypoint];
            let to_waypoint = waypoint - current_pos;
            let distance_to_waypoint = Vec3::new(to_waypoint.x, 0.0, to_waypoint.z).length();

            if distance_to_waypoint < WAYPOINT_ARRIVAL_THRESHOLD {
                path.current_waypoint += 1;

                if path.current_waypoint >= path.waypoints.len() {
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
            velocity.0 = Vec3::ZERO;
            commands.entity(entity).remove::<MoveTarget>();
            if matches!(*unit_command, UnitCommand::Move(_)) {
                *unit_command = UnitCommand::Idle;
            }
            continue;
        };

        // Compute desired direction (2D, ignoring Y)
        let to_waypoint = next_waypoint - current_pos;
        let desired_dir_2d = Vec3::new(to_waypoint.x, 0.0, to_waypoint.z);
        let distance = desired_dir_2d.length();

        if distance < 0.1 {
            // Very close to waypoint — snap position and let the next frame advance waypoint
            transform.translation.x = next_waypoint.x;
            transform.translation.z = next_waypoint.z;
            velocity.0 = Vec3::ZERO;
            continue;
        }

        let desired_dir = desired_dir_2d.normalize();

        // Get current facing direction from rotation
        let current_forward = transform.forward();
        let current_facing = Vec3::new(current_forward.x, 0.0, current_forward.z).normalize_or_zero();

        // Compute angle between current facing and desired direction
        let dot = current_facing.dot(desired_dir).clamp(-1.0, 1.0);
        let angle_to_target = dot.acos(); // radians

        // Turn toward desired direction, capped by turn_rate
        let max_turn = params.turn_rate * delta;
        if angle_to_target > 0.001 {
            // Determine turn direction (clockwise or counter-clockwise)
            let cross = current_facing.cross(desired_dir);
            let turn_sign = if cross.y >= 0.0 { 1.0 } else { -1.0 };
            let actual_turn = angle_to_target.min(max_turn);

            let turn_quat = Quat::from_rotation_y(turn_sign * actual_turn);
            transform.rotation = turn_quat * transform.rotation;
        }

        // Compute desired speed
        let current_speed = velocity.0.length();
        let decel_distance = if params.deceleration < f32::MAX / 2.0 {
            (current_speed * current_speed) / (2.0 * params.deceleration)
        } else {
            0.0
        };

        let target_speed = if distance < decel_distance.max(0.5) {
            // Approaching destination — decelerate
            (distance / decel_distance.max(0.5)) * params.max_speed
        } else if angle_to_target > std::f32::consts::FRAC_PI_2 {
            // Facing away from waypoint — slow down to turn
            params.max_speed * 0.1
        } else {
            params.max_speed
        };

        // Apply acceleration/deceleration
        let new_speed = if target_speed > current_speed {
            (current_speed + params.acceleration * delta).min(target_speed)
        } else {
            (current_speed - params.deceleration * delta).max(target_speed).max(0.0)
        };

        // Move forward in facing direction
        let updated_forward = transform.forward();
        let facing_2d = Vec3::new(updated_forward.x, 0.0, updated_forward.z).normalize_or_zero();
        velocity.0 = facing_2d * new_speed;

        let proposed_pos = transform.translation + velocity.0 * delta;

        // Ground collision check — only for ground-domain units with a silhouette
        let is_ground = domain_opt.map_or(true, |d| *d == DomainEnum::Ground);
        if is_ground {
            if let Some(sil) = silhouette_opt {
                let half_w = sil.width / 2.0;
                let half_h = sil.height / 2.0;
                if occupancy.check_movement_collision(entity, proposed_pos.x, proposed_pos.z, half_w, half_h) {
                    // Collision detected — stop and request repath
                    velocity.0 = Vec3::ZERO;
                    commands.entity(entity).remove::<Path>().insert(NeedsRepath);
                    continue;
                }
            }
        }

        transform.translation = proposed_pos;
        transform.translation.y = 0.5; // Keep on ground
    }
}

/// System to rebuild the OccupancyMap each frame.
/// Marks tiles occupied by ground units and structure footprints.
/// Must run before pathfinding and movement systems.
pub fn rebuild_occupancy_map(
    mut occupancy: ResMut<OccupancyMap>,
    ground_units: Query<(Entity, &GridPosition, &Transform, Option<&DomainEnum>, Option<&Silhouette>), With<Unit>>,
    structures: Query<(&GridPosition, &ObjectInstance), With<StructureInstance>>,
) {
    occupancy.clear();

    // Mark ground units
    for (entity, grid_pos, transform, domain_opt, silhouette_opt) in ground_units.iter() {
        let is_ground = domain_opt.map_or(true, |d| *d == DomainEnum::Ground);
        if !is_ground {
            continue;
        }
        // Pathfinding layer: mark the unit's tile
        occupancy.blocked_tiles.insert((grid_pos.x, grid_pos.z));
        // Movement layer: store AABB collision body
        let (half_w, half_h) = if let Some(sil) = silhouette_opt {
            (sil.width / 2.0, sil.height / 2.0)
        } else {
            (0.25, 0.25) // Default small body for units without silhouette
        };
        occupancy.ground_bodies.push(CollisionBody {
            entity,
            x: transform.translation.x,
            z: transform.translation.z,
            half_w,
            half_h,
        });
    }

    // Mark structures
    for (grid_pos, obj_instance) in structures.iter() {
        let (size_w, size_h) = obj_instance.object_type.object_type().size;
        for dx in 0..size_w as i32 {
            for dz in 0..size_h as i32 {
                let tile = (grid_pos.x + dx, grid_pos.z + dz);
                occupancy.blocked_tiles.insert(tile);
                occupancy.structure_tiles.insert(tile);
            }
        }
    }
}

/// System to recompute paths for units blocked by collisions.
/// Runs after rebuild_occupancy_map. Units with NeedsRepath get a fresh path
/// using the updated occupancy data. If no path exists, the marker stays for retry.
pub fn collision_repath_system(
    mut commands: Commands,
    units: Query<
        (Entity, &Transform, &UnitBaseEnum, &MoveTarget),
        (With<Unit>, With<NeedsRepath>)
    >,
    tiles: Query<(&GridPosition, &TilePreset), With<Tile>>,
    grid: Res<GridMap>,
    occupancy: Res<OccupancyMap>,
) {
    use crate::game::units::utils::{world_to_grid, smooth_path};

    for (entity, transform, unit_base, move_target) in units.iter() {
        let start = world_to_grid(transform.translation);
        let target = world_to_grid(move_target.0);
        let self_pos = (start.x, start.z);

        if let Some(path) = crate::game::units::pathfinding::find_path(
            start, target, &tiles, unit_base, grid.width as i32, grid.height as i32,
            &occupancy, self_pos,
        ) {
            let smoothed = smooth_path(path);
            commands.entity(entity)
                .remove::<NeedsRepath>()
                .insert(Path { waypoints: smoothed, current_waypoint: 0 });
        }
        // If no path found, NeedsRepath stays — will retry next frame
    }
}

/// Create an emissive color variant from a base indicator color (scaled down for glow).
fn emissive_from_color(color: Color) -> Color {
    match color {
        Color::Srgba(c) => Color::srgb(c.red * 0.8, c.green * 0.8, c.blue * 0.8),
        _ => color,
    }
}

/// Describes a desired indicator for a specific unit+command combination.
struct DesiredIndicator {
    owner_unit: Entity,
    indicator_type: CommandIndicatorType,
    target_entity: Option<Entity>,
    target_position: Option<Vec3>,
    color: Color,
    patrol_index: u8,
}

/// System to synchronize command indicators for all selected units.
/// Runs every frame: diffs existing indicators against desired state,
/// despawning stale ones and spawning new ones.
pub fn command_indicator_sync_system(
    mut commands: Commands,
    selected_units: Query<(Entity, &UnitCommand), (With<Unit>, With<Selected>)>,
    existing_indicators: Query<(Entity, &CommandIndicator)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Build desired indicator set from selected units' commands
    let mut desired: Vec<DesiredIndicator> = Vec::new();

    for (unit_entity, cmd) in selected_units.iter() {
        if !command_has_indicator(cmd) {
            continue;
        }

        let color = command_indicator_color(cmd);

        match cmd {
            UnitCommand::Move(pos) | UnitCommand::AttackMove(pos)
            | UnitCommand::AttackLocation(pos) | UnitCommand::Reverse(pos) => {
                desired.push(DesiredIndicator {
                    owner_unit: unit_entity,
                    indicator_type: CommandIndicatorType::Location,
                    target_entity: None,
                    target_position: Some(*pos),
                    color,
                    patrol_index: 0,
                });
            }
            UnitCommand::AttackTarget(target) | UnitCommand::Enter(target) => {
                desired.push(DesiredIndicator {
                    owner_unit: unit_entity,
                    indicator_type: CommandIndicatorType::Object,
                    target_entity: Some(*target),
                    target_position: None,
                    color,
                    patrol_index: 0,
                });
            }
            UnitCommand::Patrol { start, end, .. } => {
                // Two location indicators: start and end
                desired.push(DesiredIndicator {
                    owner_unit: unit_entity,
                    indicator_type: CommandIndicatorType::Location,
                    target_entity: None,
                    target_position: Some(*start),
                    color,
                    patrol_index: 0,
                });
                desired.push(DesiredIndicator {
                    owner_unit: unit_entity,
                    indicator_type: CommandIndicatorType::Location,
                    target_entity: None,
                    target_position: Some(*end),
                    color,
                    patrol_index: 1,
                });
            }
            _ => {}
        }
    }

    // Diff: remove indicators not matching desired set
    let mut kept: Vec<Entity> = Vec::new();
    for (indicator_entity, indicator) in existing_indicators.iter() {
        let still_desired = desired.iter().any(|d| {
            d.owner_unit == indicator.owner_unit
                && d.indicator_type == indicator.indicator_type
                && d.target_entity == indicator.target_entity
                && d.patrol_index == indicator.patrol_index
        });

        if still_desired {
            kept.push(indicator_entity);
        } else {
            commands.entity(indicator_entity).despawn_recursive();
        }
    }

    // Spawn missing indicators
    for d in desired.iter() {
        let already_exists = existing_indicators.iter().any(|(ent, ind)| {
            kept.contains(&ent)
                && ind.owner_unit == d.owner_unit
                && ind.indicator_type == d.indicator_type
                && ind.target_entity == d.target_entity
                && ind.patrol_index == d.patrol_index
        });

        if already_exists {
            continue;
        }

        let indicator_component = CommandIndicator {
            owner_unit: d.owner_unit,
            indicator_type: d.indicator_type,
            target_entity: d.target_entity,
            patrol_index: d.patrol_index,
        };

        match d.indicator_type {
            CommandIndicatorType::Location => {
                let pos = d.target_position.unwrap_or(Vec3::ZERO);
                let mesh = meshes.add(Cylinder::new(0.3, 0.05));
                let material = materials.add(StandardMaterial {
                    base_color: d.color,
                    emissive: emissive_from_color(d.color).into(),
                    unlit: true,
                    ..default()
                });
                commands.spawn((
                    PbrBundle {
                        mesh,
                        material,
                        transform: Transform::from_xyz(pos.x, 0.05, pos.z),
                        ..default()
                    },
                    indicator_component,
                ));
            }
            CommandIndicatorType::Object => {
                if let Some(target_entity) = d.target_entity {
                    let mesh = meshes.add(Torus::new(0.4, 0.56));
                    let material = materials.add(StandardMaterial {
                        base_color: d.color,
                        emissive: emissive_from_color(d.color).into(),
                        unlit: true,
                        ..default()
                    });
                    let indicator_id = commands.spawn((
                        PbrBundle {
                            mesh,
                            material,
                            transform: Transform::from_xyz(0.0, -0.3, 0.0)
                                .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
                            ..default()
                        },
                        indicator_component,
                    )).id();
                    commands.entity(target_entity).add_child(indicator_id);
                }
            }
        }
    }
}

/// Air unit soft separation system — applies gentle repulsion between nearby air units
/// to prevent stacking. Runs after movement systems so it doesn't interfere with pathfinding.
/// Uses linear force falloff: full force at overlap, zero force at SeparationRadius edge.
/// Only affects units with DomainEnum::Air and a SeparationRadius component.
pub fn air_unit_separation_system(
    time: Res<Time>,
    mut air_units: Query<
        (Entity, &mut Transform, &SeparationRadius, &DomainEnum),
        (With<Unit>, Without<InTunnelNetwork>)
    >,
) {
    let delta = time.delta_seconds();
    if delta < 0.0001 {
        return;
    }

    // Collect air unit positions first to avoid borrow issues with mutable iteration
    let air_positions: Vec<(Entity, Vec3, f32)> = air_units.iter()
        .filter(|(_, _, _, domain)| **domain == DomainEnum::Air)
        .map(|(e, t, sr, _)| (e, t.translation, sr.0))
        .collect();

    for (entity, mut transform, sep_radius, domain) in air_units.iter_mut() {
        if *domain != DomainEnum::Air {
            continue;
        }

        let current_pos = transform.translation;
        let mut repulsion = Vec3::ZERO;

        for &(other_entity, other_pos, other_radius) in &air_positions {
            if entity == other_entity {
                continue;
            }

            // 2D distance (XZ plane only — ignore Y)
            let diff = Vec3::new(
                current_pos.x - other_pos.x,
                0.0,
                current_pos.z - other_pos.z,
            );
            let dist = diff.length();

            // Use the maximum of both radii as the activation threshold
            let threshold = sep_radius.0.max(other_radius);

            if dist < threshold && dist > 0.01 {
                let direction = diff / dist; // normalize
                let strength = 1.0 - (dist / threshold); // linear falloff
                repulsion += direction * strength * SEPARATION_FORCE_SCALE;
            }
        }

        // Apply as direct position nudge (not velocity — velocity is managed by movement systems)
        if repulsion.length_squared() > 0.0001 {
            transform.translation.x += repulsion.x * delta;
            transform.translation.z += repulsion.z * delta;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    /// Helper: spawn a Unit entity with Transform and GridPosition
    fn spawn_unit_at(world: &mut World, world_x: f32, world_z: f32, grid_x: i32, grid_z: i32) -> Entity {
        world.spawn((
            Unit,
            Transform::from_xyz(world_x, 0.5, world_z),
            GridPosition { x: grid_x, z: grid_z },
        )).id()
    }

    #[test]
    fn grid_position_sync_updates_moved_unit() {
        let mut world = World::new();
        // Spawn at grid (32,32) = world (0.5, 0.5), then move transform to world (5.5, 0.5, 3.5) = grid (37, 35)
        let entity = spawn_unit_at(&mut world, 5.5, 3.5, 32, 32);
        world.run_system_once(grid_position_sync_system);
        let gp = world.entity(entity).get::<GridPosition>().unwrap();
        assert_eq!(gp.x, 37);
        assert_eq!(gp.z, 35);
    }

    #[test]
    fn grid_position_sync_no_change_when_stationary() {
        let mut world = World::new();
        // world (0.5, 0.5) maps to grid (32, 32) — already correct
        let entity = spawn_unit_at(&mut world, 0.5, 0.5, 32, 32);
        world.run_system_once(grid_position_sync_system);
        let gp = world.entity(entity).get::<GridPosition>().unwrap();
        assert_eq!(gp.x, 32);
        assert_eq!(gp.z, 32);
    }

    #[test]
    fn grid_position_sync_negative_world_coords() {
        let mut world = World::new();
        // world (-31.5, -31.5) = grid (0, 0)
        let entity = spawn_unit_at(&mut world, -31.5, -31.5, 32, 32);
        world.run_system_once(grid_position_sync_system);
        let gp = world.entity(entity).get::<GridPosition>().unwrap();
        assert_eq!(gp.x, 0);
        assert_eq!(gp.z, 0);
    }

    #[test]
    fn grid_position_sync_multiple_units() {
        let mut world = World::new();
        let e1 = spawn_unit_at(&mut world, 0.5, 0.5, 32, 32); // stays at (32,32)
        let e2 = spawn_unit_at(&mut world, 10.5, -5.5, 0, 0); // should become (42, 26)
        world.run_system_once(grid_position_sync_system);

        let gp1 = world.entity(e1).get::<GridPosition>().unwrap();
        assert_eq!((gp1.x, gp1.z), (32, 32));

        let gp2 = world.entity(e2).get::<GridPosition>().unwrap();
        assert_eq!((gp2.x, gp2.z), (42, 26));
    }

    #[test]
    fn grid_position_sync_ignores_non_unit_entities() {
        let mut world = World::new();
        // Entity without Unit marker — should NOT be synced
        let entity = world.spawn((
            Transform::from_xyz(10.0, 0.5, 10.0),
            GridPosition { x: 5, z: 5 },
        )).id();
        world.run_system_once(grid_position_sync_system);
        let gp = world.entity(entity).get::<GridPosition>().unwrap();
        // Should remain unchanged
        assert_eq!(gp.x, 5);
        assert_eq!(gp.z, 5);
    }

    #[test]
    fn grid_position_sync_sub_tile_movement_no_update() {
        let mut world = World::new();
        // world (0.1, 0.1) still maps to grid (32, 32) via floor
        let entity = spawn_unit_at(&mut world, 0.1, 0.1, 32, 32);
        world.run_system_once(grid_position_sync_system);
        let gp = world.entity(entity).get::<GridPosition>().unwrap();
        assert_eq!((gp.x, gp.z), (32, 32));
    }

    #[test]
    fn grid_position_sync_tile_boundary_crossing() {
        let mut world = World::new();
        // world (1.0, 0.5) = grid (33, 32) — just crossed tile boundary
        let entity = spawn_unit_at(&mut world, 1.0, 0.5, 32, 32);
        world.run_system_once(grid_position_sync_system);
        let gp = world.entity(entity).get::<GridPosition>().unwrap();
        assert_eq!((gp.x, gp.z), (33, 32));
    }

    #[test]
    fn grid_position_sync_change_detection_guard() {
        let mut world = World::new();
        // Position already matches — change detection should NOT fire
        // world (0.5, 0.5) = grid (32, 32)
        let entity = spawn_unit_at(&mut world, 0.5, 0.5, 32, 32);

        // Clear change ticks by running a frame
        world.increment_change_tick();
        world.clear_trackers();

        world.run_system_once(grid_position_sync_system);

        // GridPosition should not have been mutated (change detection guard)
        // We verify the values are still correct
        let gp = world.entity(entity).get::<GridPosition>().unwrap();
        assert_eq!((gp.x, gp.z), (32, 32));
    }

    #[test]
    fn grid_position_sync_uses_world_to_grid_correctly() {
        // Verify the system produces the same result as world_to_grid
        let mut world = World::new();
        let test_positions = [
            (0.0, 0.0),    // grid center
            (-32.0, -32.0), // corner
            (31.0, 31.0),  // opposite corner area
            (15.5, -10.5), // arbitrary
        ];

        let entities: Vec<Entity> = test_positions
            .iter()
            .map(|&(x, z)| spawn_unit_at(&mut world, x, z, 0, 0))
            .collect();

        world.run_system_once(grid_position_sync_system);

        for (i, &(x, z)) in test_positions.iter().enumerate() {
            let expected = world_to_grid(Vec3::new(x, 0.5, z));
            let gp = world.entity(entities[i]).get::<GridPosition>().unwrap();
            assert_eq!(
                (gp.x, gp.z), (expected.x, expected.z),
                "Mismatch at world ({}, {})", x, z
            );
        }
    }

    // === Command Indicator Sync Tests ===

    #[test]
    fn emissive_from_green_color() {
        let green = Color::srgb(0.0, 1.0, 0.0);
        let emissive = emissive_from_color(green);
        assert_eq!(emissive, Color::srgb(0.0, 0.8, 0.0));
    }

    #[test]
    fn emissive_from_red_color() {
        let red = Color::srgb(1.0, 0.2, 0.0);
        let emissive = emissive_from_color(red);
        // 0.2 * 0.8 = 0.16000... (float precision)
        match emissive {
            Color::Srgba(c) => {
                assert!((c.red - 0.8).abs() < 0.001);
                assert!((c.green - 0.16).abs() < 0.001);
                assert!((c.blue - 0.0).abs() < 0.001);
            }
            _ => panic!("Expected Srgba color"),
        }
    }

    #[test]
    fn emissive_from_orange_color() {
        let orange = Color::srgb(1.0, 0.6, 0.0);
        let emissive = emissive_from_color(orange);
        match emissive {
            Color::Srgba(c) => {
                assert!((c.red - 0.8).abs() < 0.001);
                assert!((c.green - 0.48).abs() < 0.001);
                assert!((c.blue - 0.0).abs() < 0.001);
            }
            _ => panic!("Expected Srgba color"),
        }
    }

    /// Helper: create a minimal World with Assets for indicator sync testing
    fn create_indicator_test_world() -> World {
        let mut world = World::new();
        world.init_resource::<Assets<Mesh>>();
        world.init_resource::<Assets<StandardMaterial>>();
        world
    }

    /// Helper: spawn a selected unit with a command
    fn spawn_selected_unit_with_command(world: &mut World, cmd: UnitCommand) -> Entity {
        world.spawn((
            Unit,
            Selected,
            Transform::from_xyz(0.0, 0.5, 0.0),
            cmd,
        )).id()
    }

    #[test]
    fn sync_spawns_location_indicator_for_move() {
        let mut world = create_indicator_test_world();
        spawn_selected_unit_with_command(&mut world, UnitCommand::Move(Vec3::new(5.0, 0.0, 5.0)));

        world.run_system_once(command_indicator_sync_system);

        let indicators: Vec<_> = world.query::<&CommandIndicator>().iter(&world).collect();
        assert_eq!(indicators.len(), 1);
        assert_eq!(indicators[0].indicator_type, CommandIndicatorType::Location);
    }

    #[test]
    fn sync_spawns_object_indicator_for_attack() {
        let mut world = create_indicator_test_world();
        let target = world.spawn(Transform::from_xyz(10.0, 0.5, 10.0)).id();
        spawn_selected_unit_with_command(&mut world, UnitCommand::AttackTarget(target));

        world.run_system_once(command_indicator_sync_system);

        let indicators: Vec<_> = world.query::<&CommandIndicator>().iter(&world).collect();
        assert_eq!(indicators.len(), 1);
        assert_eq!(indicators[0].indicator_type, CommandIndicatorType::Object);
        assert_eq!(indicators[0].target_entity, Some(target));
    }

    #[test]
    fn sync_spawns_two_indicators_for_patrol() {
        let mut world = create_indicator_test_world();
        spawn_selected_unit_with_command(&mut world, UnitCommand::Patrol {
            start: Vec3::new(0.0, 0.0, 0.0),
            end: Vec3::new(10.0, 0.0, 10.0),
            going_to_end: true,
        });

        world.run_system_once(command_indicator_sync_system);

        let indicators: Vec<_> = world.query::<&CommandIndicator>().iter(&world).collect();
        assert_eq!(indicators.len(), 2);
        // One start (index 0) and one end (index 1)
        let indices: Vec<u8> = indicators.iter().map(|i| i.patrol_index).collect();
        assert!(indices.contains(&0));
        assert!(indices.contains(&1));
    }

    #[test]
    fn sync_no_indicator_for_idle() {
        let mut world = create_indicator_test_world();
        spawn_selected_unit_with_command(&mut world, UnitCommand::Idle);

        world.run_system_once(command_indicator_sync_system);

        let count = world.query::<&CommandIndicator>().iter(&world).count();
        assert_eq!(count, 0);
    }

    #[test]
    fn sync_no_indicator_for_hold_position() {
        let mut world = create_indicator_test_world();
        spawn_selected_unit_with_command(&mut world, UnitCommand::HoldPosition);

        world.run_system_once(command_indicator_sync_system);

        let count = world.query::<&CommandIndicator>().iter(&world).count();
        assert_eq!(count, 0);
    }

    #[test]
    fn sync_removes_indicator_when_deselected() {
        let mut world = create_indicator_test_world();
        let unit = spawn_selected_unit_with_command(&mut world, UnitCommand::Move(Vec3::new(5.0, 0.0, 5.0)));

        world.run_system_once(command_indicator_sync_system);
        assert_eq!(world.query::<&CommandIndicator>().iter(&world).count(), 1);

        // Deselect the unit
        world.entity_mut(unit).remove::<Selected>();
        world.run_system_once(command_indicator_sync_system);

        assert_eq!(world.query::<&CommandIndicator>().iter(&world).count(), 0);
    }

    #[test]
    fn sync_multiple_selected_units_show_all_indicators() {
        let mut world = create_indicator_test_world();
        spawn_selected_unit_with_command(&mut world, UnitCommand::Move(Vec3::new(5.0, 0.0, 5.0)));
        spawn_selected_unit_with_command(&mut world, UnitCommand::Move(Vec3::new(10.0, 0.0, 10.0)));
        let target = world.spawn(Transform::from_xyz(15.0, 0.5, 15.0)).id();
        spawn_selected_unit_with_command(&mut world, UnitCommand::AttackTarget(target));

        world.run_system_once(command_indicator_sync_system);

        let count = world.query::<&CommandIndicator>().iter(&world).count();
        assert_eq!(count, 3); // 2 location + 1 object
    }

    #[test]
    fn sync_indicator_tracks_owner_unit() {
        let mut world = create_indicator_test_world();
        let unit = spawn_selected_unit_with_command(&mut world, UnitCommand::Move(Vec3::ZERO));

        world.run_system_once(command_indicator_sync_system);

        let indicators: Vec<_> = world.query::<&CommandIndicator>().iter(&world).collect();
        assert_eq!(indicators.len(), 1);
        assert_eq!(indicators[0].owner_unit, unit);
    }

    #[test]
    fn sync_enter_command_creates_object_indicator() {
        let mut world = create_indicator_test_world();
        let tunnel = world.spawn(Transform::from_xyz(5.0, 0.5, 5.0)).id();
        spawn_selected_unit_with_command(&mut world, UnitCommand::Enter(tunnel));

        world.run_system_once(command_indicator_sync_system);

        let indicators: Vec<_> = world.query::<&CommandIndicator>().iter(&world).collect();
        assert_eq!(indicators.len(), 1);
        assert_eq!(indicators[0].indicator_type, CommandIndicatorType::Object);
        assert_eq!(indicators[0].target_entity, Some(tunnel));
    }

    #[test]
    fn sync_attack_location_creates_location_indicator() {
        let mut world = create_indicator_test_world();
        spawn_selected_unit_with_command(&mut world, UnitCommand::AttackLocation(Vec3::new(8.0, 0.0, 3.0)));

        world.run_system_once(command_indicator_sync_system);

        let indicators: Vec<_> = world.query::<&CommandIndicator>().iter(&world).collect();
        assert_eq!(indicators.len(), 1);
        assert_eq!(indicators[0].indicator_type, CommandIndicatorType::Location);
    }

    #[test]
    fn sync_reverse_creates_location_indicator() {
        let mut world = create_indicator_test_world();
        spawn_selected_unit_with_command(&mut world, UnitCommand::Reverse(Vec3::new(3.0, 0.0, 2.0)));

        world.run_system_once(command_indicator_sync_system);

        let indicators: Vec<_> = world.query::<&CommandIndicator>().iter(&world).collect();
        assert_eq!(indicators.len(), 1);
        assert_eq!(indicators[0].indicator_type, CommandIndicatorType::Location);
    }

    #[test]
    fn sync_attack_move_creates_location_indicator() {
        let mut world = create_indicator_test_world();
        spawn_selected_unit_with_command(&mut world, UnitCommand::AttackMove(Vec3::new(7.0, 0.0, 7.0)));

        world.run_system_once(command_indicator_sync_system);

        let indicators: Vec<_> = world.query::<&CommandIndicator>().iter(&world).collect();
        assert_eq!(indicators.len(), 1);
        assert_eq!(indicators[0].indicator_type, CommandIndicatorType::Location);
    }

    // === Air Unit Soft Separation Tests ===

    fn spawn_air_unit_at(world: &mut World, x: f32, z: f32, radius: f32) -> Entity {
        world.spawn((
            TransformBundle::from_transform(Transform::from_xyz(x, 1.5, z)),
            Unit,
            DomainEnum::Air,
            SeparationRadius(radius),
        )).id()
    }

    fn spawn_ground_unit_at(world: &mut World, x: f32, z: f32) -> Entity {
        world.spawn((
            TransformBundle::from_transform(Transform::from_xyz(x, 0.5, z)),
            Unit,
            DomainEnum::Ground,
        )).id()
    }

    fn create_separation_test_world() -> World {
        let mut world = World::new();
        world.insert_resource(Time::<()>::default());
        world
    }

    #[test]
    fn separation_radius_component_stores_value() {
        let sr = SeparationRadius(1.5);
        assert!((sr.0 - 1.5).abs() < f32::EPSILON);
    }

    #[test]
    fn separation_force_scale_is_positive() {
        assert!(SEPARATION_FORCE_SCALE > 0.0);
    }

    #[test]
    fn two_overlapping_air_units_drift_apart() {
        let mut world = create_separation_test_world();
        let e1 = spawn_air_unit_at(&mut world, 0.0, 0.0, 1.25);
        let e2 = spawn_air_unit_at(&mut world, 0.3, 0.0, 1.25);

        // Manually advance time so delta_seconds > 0
        world.resource_mut::<Time<()>>().advance_by(std::time::Duration::from_millis(100));

        world.run_system_once(air_unit_separation_system);

        let pos1 = world.entity(e1).get::<Transform>().unwrap().translation;
        let pos2 = world.entity(e2).get::<Transform>().unwrap().translation;

        // Units should have drifted further apart (e1 pushed left, e2 pushed right)
        let separation = (pos2.x - pos1.x).abs();
        assert!(separation > 0.3, "Units should have drifted apart: separation={}", separation);
    }

    #[test]
    fn air_units_beyond_radius_no_repulsion() {
        let mut world = create_separation_test_world();
        let e1 = spawn_air_unit_at(&mut world, 0.0, 0.0, 1.25);
        let e2 = spawn_air_unit_at(&mut world, 5.0, 0.0, 1.25);

        world.resource_mut::<Time<()>>().advance_by(std::time::Duration::from_millis(100));
        world.run_system_once(air_unit_separation_system);

        let pos1 = world.entity(e1).get::<Transform>().unwrap().translation;
        let pos2 = world.entity(e2).get::<Transform>().unwrap().translation;

        // Positions should be unchanged (beyond separation radius)
        assert!((pos1.x - 0.0).abs() < 0.001);
        assert!((pos2.x - 5.0).abs() < 0.001);
    }

    #[test]
    fn air_units_ignore_ground_units() {
        let mut world = create_separation_test_world();
        let air = spawn_air_unit_at(&mut world, 0.0, 0.0, 1.25);
        let _ground = spawn_ground_unit_at(&mut world, 0.1, 0.0);

        world.resource_mut::<Time<()>>().advance_by(std::time::Duration::from_millis(100));
        world.run_system_once(air_unit_separation_system);

        let pos = world.entity(air).get::<Transform>().unwrap().translation;
        // Air unit should not have moved — ground units don't affect air separation
        assert!((pos.x - 0.0).abs() < 0.001, "Air unit should not be pushed by ground unit");
    }

    #[test]
    fn separation_preserves_y_height() {
        let mut world = create_separation_test_world();
        let e1 = spawn_air_unit_at(&mut world, 0.0, 0.0, 1.25);
        let _e2 = spawn_air_unit_at(&mut world, 0.3, 0.0, 1.25);

        world.resource_mut::<Time<()>>().advance_by(std::time::Duration::from_millis(100));
        world.run_system_once(air_unit_separation_system);

        let pos = world.entity(e1).get::<Transform>().unwrap().translation;
        assert!((pos.y - 1.5).abs() < 0.001, "Y should be unchanged");
    }

    #[test]
    fn separation_uses_max_radius() {
        let mut world = create_separation_test_world();
        // e1 has small radius, e2 has large radius
        let e1 = spawn_air_unit_at(&mut world, 0.0, 0.0, 0.5);
        let e2 = spawn_air_unit_at(&mut world, 1.0, 0.0, 2.0);

        world.resource_mut::<Time<()>>().advance_by(std::time::Duration::from_millis(100));
        world.run_system_once(air_unit_separation_system);

        let pos1 = world.entity(e1).get::<Transform>().unwrap().translation;
        let pos2 = world.entity(e2).get::<Transform>().unwrap().translation;

        // Distance is 1.0, max(0.5, 2.0) = 2.0, so within threshold — should repel
        let separation = (pos2.x - pos1.x).abs();
        assert!(separation > 1.0, "Units should drift apart when within max radius");
    }

    #[test]
    fn three_air_units_spread_evenly() {
        let mut world = create_separation_test_world();
        let e1 = spawn_air_unit_at(&mut world, 0.0, 0.0, 1.25);
        let e2 = spawn_air_unit_at(&mut world, 0.1, 0.0, 1.25);
        let e3 = spawn_air_unit_at(&mut world, 0.2, 0.0, 1.25);

        world.resource_mut::<Time<()>>().advance_by(std::time::Duration::from_millis(100));
        world.run_system_once(air_unit_separation_system);

        let pos1 = world.entity(e1).get::<Transform>().unwrap().translation;
        let pos2 = world.entity(e2).get::<Transform>().unwrap().translation;
        let pos3 = world.entity(e3).get::<Transform>().unwrap().translation;

        // All three should have spread out — e1 left, e3 right, e2 stays roughly center
        assert!(pos1.x < 0.0, "Leftmost unit should move further left");
        assert!(pos3.x > 0.2, "Rightmost unit should move further right");
    }

    #[test]
    fn in_tunnel_units_excluded_from_separation() {
        let mut world = create_separation_test_world();
        let air = spawn_air_unit_at(&mut world, 0.0, 0.0, 1.25);
        let _tunneled = world.spawn((
            TransformBundle::from_transform(Transform::from_xyz(0.1, 1.5, 0.0)),
            Unit,
            DomainEnum::Air,
            SeparationRadius(1.25),
            InTunnelNetwork { owner_player: 0 },
        )).id();

        world.resource_mut::<Time<()>>().advance_by(std::time::Duration::from_millis(100));
        world.run_system_once(air_unit_separation_system);

        let pos = world.entity(air).get::<Transform>().unwrap().translation;
        // Air unit should not have moved — the other unit is in tunnel network
        assert!((pos.x - 0.0).abs() < 0.001, "Should not be pushed by in-tunnel unit");
    }

    #[test]
    fn separation_linear_falloff_stronger_when_closer() {
        let mut world = create_separation_test_world();
        // Very close (0.1 apart)
        let e1_close = spawn_air_unit_at(&mut world, 0.0, 0.0, 1.25);
        let _e2_close = spawn_air_unit_at(&mut world, 0.1, 0.0, 1.25);

        world.resource_mut::<Time<()>>().advance_by(std::time::Duration::from_millis(100));
        world.run_system_once(air_unit_separation_system);
        let push_close = (world.entity(e1_close).get::<Transform>().unwrap().translation.x - 0.0).abs();

        // Now test farther apart (1.0 apart)
        let mut world2 = create_separation_test_world();
        let e1_far = spawn_air_unit_at(&mut world2, 0.0, 0.0, 1.25);
        let _e2_far = spawn_air_unit_at(&mut world2, 1.0, 0.0, 1.25);

        world2.resource_mut::<Time<()>>().advance_by(std::time::Duration::from_millis(100));
        world2.run_system_once(air_unit_separation_system);
        let push_far = (world2.entity(e1_far).get::<Transform>().unwrap().translation.x - 0.0).abs();

        assert!(push_close > push_far, "Closer units should receive stronger repulsion: close={}, far={}", push_close, push_far);
    }

    #[test]
    fn no_separation_when_zero_delta() {
        let mut world = create_separation_test_world();
        let e1 = spawn_air_unit_at(&mut world, 0.0, 0.0, 1.25);
        let _e2 = spawn_air_unit_at(&mut world, 0.1, 0.0, 1.25);

        // Don't advance time — delta is 0
        world.run_system_once(air_unit_separation_system);

        let pos = world.entity(e1).get::<Transform>().unwrap().translation;
        assert!((pos.x - 0.0).abs() < 0.001, "No movement with zero delta");
    }

    #[test]
    fn separation_works_on_z_axis() {
        let mut world = create_separation_test_world();
        let e1 = spawn_air_unit_at(&mut world, 0.0, 0.0, 1.25);
        let _e2 = world.spawn((
            TransformBundle::from_transform(Transform::from_xyz(0.0, 1.5, 0.3)),
            Unit,
            DomainEnum::Air,
            SeparationRadius(1.25),
        )).id();

        world.resource_mut::<Time<()>>().advance_by(std::time::Duration::from_millis(100));
        world.run_system_once(air_unit_separation_system);

        let pos = world.entity(e1).get::<Transform>().unwrap().translation;
        assert!(pos.z < 0.0, "Unit should be pushed in negative Z direction");
    }
}

