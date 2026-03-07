use bevy::prelude::*;
use crate::types::{Unit, Owner, DomainEnum, GridPosition, UnitBaseEnum, SightRange, MovementModelEnum};
use crate::game::types::ObjectInstance;
use crate::utils::is_enemy;
use crate::game::units::types::*;
use crate::game::units::utils::{world_to_grid, smooth_path};
use crate::game::world::types::{Tile, TilePreset, GridMap, ElevationMap, elevation_modifier};
use crate::game::combat::types::*;

/// Attacking object behavior system.
///
/// Handles `UnitCommand::AttackTarget(entity)` — drives approach movement when out of range
/// and delegates to attack_phase_system for engagement when in range.
/// Infantry stops to fire; turret units continue moving; gliders strafe.
pub fn attacking_object_behavior_system(
    mut commands: Commands,
    mut units: Query<
        (Entity, &Transform, &AttackCapability, &mut AttackState, &UnitCommand,
         &UnitBaseEnum, &Owner, Option<&DomainEnum>, &GridPosition),
        With<Unit>
    >,
    targets: Query<(&Transform, &Owner, Option<&DomainEnum>, &GridPosition), With<ObjectInstance>>,
    tiles: Query<(&GridPosition, &TilePreset), With<Tile>>,
    grid: Res<GridMap>,
    elevation_map: Res<ElevationMap>,
    occupancy: Res<OccupancyMap>,
) {
    for (entity, transform, attack_cap, mut attack_state, command, unit_base, _owner,
         source_domain, source_grid_pos) in units.iter_mut()
    {
        let target_entity = match command {
            UnitCommand::AttackTarget(t) => *t,
            _ => continue,
        };

        // Check if target still exists
        let Ok((target_transform, _target_owner, target_domain, target_grid_pos)) = targets.get(target_entity) else {
            // Target destroyed — go idle, check command queue
            commands.entity(entity)
                .remove::<MoveTarget>()
                .remove::<Path>()
                .insert(UnitCommand::Idle);
            attack_state.current_target = None;
            attack_state.phase = AttackPhase::None;
            continue;
        };

        let distance = transform.translation.distance(target_transform.translation);
        let base_data = unit_base.data();

        // Compute elevation-adjusted effective range
        let src_domain = source_domain.copied().unwrap_or(DomainEnum::Ground);
        let tgt_domain = target_domain.copied().unwrap_or(DomainEnum::Ground);
        let src_elev = elevation_map.get(source_grid_pos.x, source_grid_pos.z);
        let tgt_elev = elevation_map.get(target_grid_pos.x, target_grid_pos.z);
        let elev_mod = if attack_cap.is_melee() { 0 } else {
            elevation_modifier(src_domain, src_elev, tgt_domain, tgt_elev)
        };
        let effective_range = attack_cap.range + elev_mod as f32;

        if distance <= effective_range {
            // In range — engage
            if attack_state.target_entity() != Some(target_entity) {
                if attack_state.phase.is_interruptible() {
                    attack_state.current_target = Some(AttackTarget::UnitTarget(target_entity));
                    attack_state.phase = AttackPhase::None;
                    attack_state.time_in_phase = 0.0;
                }
            }

            // Infantry: stop movement when in range
            if !base_data.has_turret && base_data.movement_model != MovementModelEnum::Glider {
                commands.entity(entity)
                    .remove::<MoveTarget>()
                    .remove::<Path>();
            }
            // Turret units and gliders: keep moving, turret/phase system handles firing
        } else {
            // Out of range — approach target
            let start_grid = world_to_grid(transform.translation);
            let target_grid = world_to_grid(target_transform.translation);

            if let Some(path) = crate::game::units::pathfinding::find_path(
                start_grid, target_grid, &tiles, unit_base,
                grid.width as i32, grid.height as i32,
                &occupancy, (start_grid.x, start_grid.z),
            ) {
                let smoothed = smooth_path(path);
                commands.entity(entity).insert((
                    MoveTarget(target_transform.translation),
                    Path { waypoints: smoothed, current_waypoint: 0 },
                ));
            }
        }
    }
}

/// Attacking location behavior system.
///
/// Handles `UnitCommand::AttackLocation(pos)` — approaches the location until in range,
/// then sets the attack target to that location. Completes after one attack cycle.
/// TODO: Gate on `can_target_ground` when the field is added to AttackCapability.
pub fn attacking_location_behavior_system(
    mut commands: Commands,
    mut units: Query<
        (Entity, &Transform, &AttackCapability, &mut AttackState, &UnitCommand,
         &UnitBaseEnum, Option<&DomainEnum>, &GridPosition),
        With<Unit>
    >,
    tiles: Query<(&GridPosition, &TilePreset), With<Tile>>,
    grid: Res<GridMap>,
    elevation_map: Res<ElevationMap>,
    occupancy: Res<OccupancyMap>,
) {
    for (entity, transform, attack_cap, mut attack_state, command, unit_base,
         source_domain, source_grid_pos) in units.iter_mut()
    {
        let target_pos = match command {
            UnitCommand::AttackLocation(pos) => *pos,
            _ => continue,
        };

        let distance = transform.translation.distance(target_pos);
        let base_data = unit_base.data();

        // Compute elevation-adjusted effective range (use ground elevation at target)
        let src_domain = source_domain.copied().unwrap_or(DomainEnum::Ground);
        let target_grid = world_to_grid(target_pos);
        let src_elev = elevation_map.get(source_grid_pos.x, source_grid_pos.z);
        let tgt_elev = elevation_map.get(target_grid.x, target_grid.z);
        let elev_mod = if attack_cap.is_melee() { 0 } else {
            elevation_modifier(src_domain, src_elev, DomainEnum::Ground, tgt_elev)
        };
        let effective_range = attack_cap.range + elev_mod as f32;

        if distance <= effective_range {
            // In range — fire at location
            if attack_state.current_target.is_none() || attack_state.phase.is_interruptible() {
                attack_state.current_target = Some(AttackTarget::LocationTarget(target_pos));
                attack_state.phase = AttackPhase::None;
                attack_state.time_in_phase = 0.0;
            }

            // Infantry: stop movement
            if !base_data.has_turret && base_data.movement_model != MovementModelEnum::Glider {
                commands.entity(entity)
                    .remove::<MoveTarget>()
                    .remove::<Path>();
            }

            // After one full attack cycle (reached Reloading), complete the command
            if matches!(attack_state.phase, AttackPhase::Reloading) {
                commands.entity(entity)
                    .remove::<MoveTarget>()
                    .remove::<Path>()
                    .insert(UnitCommand::Idle);
                attack_state.current_target = None;
                attack_state.phase = AttackPhase::None;
            }
        } else {
            // Out of range — approach
            let start_grid = world_to_grid(transform.translation);
            let target_grid_pos = world_to_grid(target_pos);

            if let Some(path) = crate::game::units::pathfinding::find_path(
                start_grid, target_grid_pos, &tiles, unit_base,
                grid.width as i32, grid.height as i32,
                &occupancy, (start_grid.x, start_grid.z),
            ) {
                let smoothed = smooth_path(path);
                commands.entity(entity).insert((
                    MoveTarget(target_pos),
                    Path { waypoints: smoothed, current_waypoint: 0 },
                ));
            }
        }
    }
}

/// Attack-move behavior system.
///
/// Handles `UnitCommand::AttackMove(destination)` — moves toward destination along a path,
/// scanning for enemies within sight range during movement. Engages found enemies,
/// but disengages if perpendicular distance from original path exceeds ATTACK_MOVE_LEASH_DISTANCE.
pub fn attack_move_behavior_system(
    mut commands: Commands,
    mut units: Query<
        (Entity, &Transform, &AttackCapability, &mut AttackState, &mut UnitCommand,
         &UnitBaseEnum, &Owner, Option<&SightRange>, Option<&DomainEnum>, &GridPosition),
        With<Unit>
    >,
    potential_targets: Query<(Entity, &Transform, &Owner, Option<&DomainEnum>, &GridPosition), With<ObjectInstance>>,
    tiles: Query<(&GridPosition, &TilePreset), With<Tile>>,
    grid: Res<GridMap>,
    _elevation_map: Res<ElevationMap>,
    attack_move_origins: Query<&AttackMoveOrigin>,
    occupancy: Res<OccupancyMap>,
) {
    for (entity, transform, _attack_cap, mut attack_state, mut command, unit_base, owner,
         sight_range_opt, _source_domain, _source_grid_pos) in units.iter_mut()
    {
        let destination = match command.as_ref() {
            UnitCommand::AttackMove(dest) => *dest,
            _ => continue,
        };

        let pos = transform.translation;
        let base_data = unit_base.data();

        // Check if we've arrived at destination
        let dist_to_dest = Vec3::new(pos.x - destination.x, 0.0, pos.z - destination.z).length();
        if dist_to_dest < 0.5 {
            // Arrived — go idle
            commands.entity(entity)
                .remove::<MoveTarget>()
                .remove::<Path>()
                .remove::<AttackMoveOrigin>();
            *command = UnitCommand::Idle;
            attack_state.current_target = None;
            attack_state.phase = AttackPhase::None;
            continue;
        }

        // If currently engaging a target, check leash
        if let Some(target_entity) = attack_state.target_entity() {
            if let Ok(origin) = attack_move_origins.get(entity) {
                let dist_from_origin = Vec3::new(
                    pos.x - origin.0.x, 0.0, pos.z - origin.0.z,
                ).length();

                if dist_from_origin > ATTACK_MOVE_LEASH_DISTANCE {
                    // Too far from path — disengage and resume move
                    attack_state.current_target = None;
                    attack_state.phase = AttackPhase::None;
                    commands.entity(entity).remove::<AttackMoveOrigin>();

                    // Re-pathfind to destination
                    let start_grid = world_to_grid(pos);
                    let target_grid = world_to_grid(destination);
                    if let Some(path) = crate::game::units::pathfinding::find_path(
                        start_grid, target_grid, &tiles, unit_base,
                        grid.width as i32, grid.height as i32,
                        &occupancy, (start_grid.x, start_grid.z),
                    ) {
                        let smoothed = smooth_path(path);
                        commands.entity(entity).insert((
                            MoveTarget(destination),
                            Path { waypoints: smoothed, current_waypoint: 0 },
                        ));
                    }
                    continue;
                }
            }

            // Check if target is still alive
            if potential_targets.get(target_entity).is_err() {
                // Target destroyed — resume path
                attack_state.current_target = None;
                attack_state.phase = AttackPhase::None;
                commands.entity(entity).remove::<AttackMoveOrigin>();

                let start_grid = world_to_grid(pos);
                let target_grid = world_to_grid(destination);
                if let Some(path) = crate::game::units::pathfinding::find_path(
                    start_grid, target_grid, &tiles, unit_base,
                    grid.width as i32, grid.height as i32,
                    &occupancy, (start_grid.x, start_grid.z),
                ) {
                    let smoothed = smooth_path(path);
                    commands.entity(entity).insert((
                        MoveTarget(destination),
                        Path { waypoints: smoothed, current_waypoint: 0 },
                    ));
                }
            }
            continue; // Let attack systems handle the engagement
        }

        // Not currently engaged — scan for enemies
        let scan_range = sight_range_opt.map(|sr| sr.0 as f32).unwrap_or(5.0);

        let mut nearest_enemy = None;
        let mut nearest_distance = f32::MAX;

        for (target_entity, target_transform, target_owner, _target_domain, _target_grid_pos) in potential_targets.iter() {
            if target_entity == entity { continue; }
            if !is_enemy(owner, target_owner) { continue; }

            let distance = pos.distance(target_transform.translation);
            if distance > scan_range { continue; }

            if distance < nearest_distance {
                nearest_enemy = Some(target_entity);
                nearest_distance = distance;
            }
        }

        if let Some(target) = nearest_enemy {
            // Found enemy — engage
            attack_state.current_target = Some(AttackTarget::UnitTarget(target));
            attack_state.phase = AttackPhase::Aiming;
            attack_state.time_in_phase = 0.0;

            // Record where we started the detour
            commands.entity(entity).insert(AttackMoveOrigin(pos));

            // Infantry: stop to engage
            if !base_data.has_turret && base_data.movement_model != MovementModelEnum::Glider {
                commands.entity(entity)
                    .remove::<MoveTarget>()
                    .remove::<Path>();
            }
        } else {
            // No enemies — ensure we're moving toward destination
            // Only re-pathfind if we don't already have a path
            // (checking for MoveTarget component presence would require another query,
            //  so we rely on the movement system to handle path following)
            let start_grid = world_to_grid(pos);
            let target_grid = world_to_grid(destination);

            if let Some(path) = crate::game::units::pathfinding::find_path(
                start_grid, target_grid, &tiles, unit_base,
                grid.width as i32, grid.height as i32,
                &occupancy, (start_grid.x, start_grid.z),
            ) {
                let smoothed = smooth_path(path);
                commands.entity(entity).insert((
                    MoveTarget(destination),
                    Path { waypoints: smoothed, current_waypoint: 0 },
                ));
            }
        }
    }
}

/// Hold position behavior system.
///
/// Handles `UnitCommand::HoldPosition` units — ensures they NEVER move but can engage
/// enemies that come within range. For turret units, the turret_autonomous_scanning_system
/// handles targeting. For non-turret CanTurnInPlace infantry, rotates toward enemies.
/// For non-turning infantry, only engages targets within the facing arc.
pub fn hold_position_behavior_system(
    mut commands: Commands,
    mut units: Query<
        (Entity, &Transform, &AttackCapability, &mut AttackState, &UnitCommand,
         &UnitBaseEnum, &Owner, Option<&DomainEnum>, &GridPosition),
        (With<Unit>, Without<Turret>)
    >,
    potential_targets: Query<(Entity, &Transform, &Owner, Option<&DomainEnum>, &GridPosition), With<ObjectInstance>>,
    elevation_map: Res<ElevationMap>,
) {
    for (entity, transform, attack_cap, mut attack_state, command, unit_base, owner,
         source_domain, source_grid_pos) in units.iter_mut()
    {
        if !matches!(command, UnitCommand::HoldPosition) {
            continue;
        }

        // Ensure movement components are removed — never move
        commands.entity(entity)
            .remove::<MoveTarget>()
            .remove::<Path>();

        // If already have a target, let attack_phase_system handle it
        if attack_state.target_entity().is_some() {
            continue;
        }

        let base_data = unit_base.data();
        let src_domain = source_domain.copied().unwrap_or(DomainEnum::Ground);
        let src_elev = elevation_map.get(source_grid_pos.x, source_grid_pos.z);
        let pos = transform.translation;

        let mut best_target: Option<(Entity, f32)> = None;

        for (target_entity, target_transform, target_owner, target_domain, target_grid_pos) in potential_targets.iter() {
            if target_entity == entity { continue; }
            if !is_enemy(owner, target_owner) { continue; }

            let distance = pos.distance(target_transform.translation);

            // Compute elevation-adjusted effective range
            let tgt_domain = target_domain.copied().unwrap_or(DomainEnum::Ground);
            let tgt_elev = elevation_map.get(target_grid_pos.x, target_grid_pos.z);
            let elev_mod = if attack_cap.is_melee() { 0 } else {
                elevation_modifier(src_domain, src_elev, tgt_domain, tgt_elev)
            };
            let effective_range = attack_cap.range + elev_mod as f32;

            if distance > effective_range { continue; }

            // Non-turning infantry: check facing arc
            if !base_data.can_turn_in_place {
                let to_target = (target_transform.translation - pos).normalize_or_zero();
                let forward = transform.forward();
                let forward_2d = Vec3::new(forward.x, 0.0, forward.z).normalize_or_zero();
                let to_target_2d = Vec3::new(to_target.x, 0.0, to_target.z).normalize_or_zero();
                let angle = forward_2d.dot(to_target_2d).acos();

                if angle > HOLD_POSITION_FACING_ARC {
                    continue; // Target is outside facing arc
                }
            }

            if let Some((_, best_dist)) = &best_target {
                if distance < *best_dist {
                    best_target = Some((target_entity, distance));
                }
            } else {
                best_target = Some((target_entity, distance));
            }
        }

        if let Some((target, _)) = best_target {
            attack_state.current_target = Some(AttackTarget::UnitTarget(target));
            attack_state.phase = AttackPhase::Aiming;
            attack_state.time_in_phase = 0.0;
        }
    }
}

/// Patrol scanning system.
///
/// Enhances the existing patrol_command_system by adding enemy scanning during patrol movement.
/// When a patrolling unit detects an enemy within sight range, it temporarily engages via
/// AttackTarget, recording patrol state in PatrolEngaged. When the target is destroyed,
/// patrol resumes from the current position.
pub fn patrol_scanning_system(
    mut commands: Commands,
    mut units: Query<
        (Entity, &Transform, &mut UnitCommand, &AttackCapability, &mut AttackState,
         &Owner, Option<&SightRange>, Option<&DomainEnum>, &GridPosition),
        With<Unit>
    >,
    potential_targets: Query<(Entity, &Transform, &Owner, Option<&DomainEnum>, &GridPosition), With<ObjectInstance>>,
    patrol_engaged: Query<&PatrolEngaged>,
    _elevation_map: Res<ElevationMap>,
) {
    for (entity, transform, mut command, _attack_cap, mut attack_state, owner,
         sight_range_opt, _source_domain, _source_grid_pos) in units.iter_mut()
    {
        // Check if unit was engaged and target died — resume patrol
        if let Ok(engaged) = patrol_engaged.get(entity) {
            if attack_state.target_entity().is_none() || matches!(attack_state.phase, AttackPhase::None) {
                // Target gone — resume patrol from current position
                let patrol_start = engaged.patrol_start;
                let patrol_end = engaged.patrol_end;
                let going_to_end = engaged.going_to_end;
                *command = UnitCommand::Patrol {
                    start: patrol_start,
                    end: patrol_end,
                    going_to_end,
                };
                attack_state.current_target = None;
                attack_state.phase = AttackPhase::None;
                commands.entity(entity).remove::<PatrolEngaged>();
            }
            continue;
        }

        // Only scan during active patrol
        let (start, end, going_to_end) = match command.as_ref() {
            UnitCommand::Patrol { start, end, going_to_end } => (*start, *end, *going_to_end),
            _ => continue,
        };

        // Already engaged — skip
        if attack_state.target_entity().is_some() {
            continue;
        }

        let scan_range = sight_range_opt.map(|sr| sr.0 as f32).unwrap_or(5.0);
        let pos = transform.translation;

        let mut nearest_enemy = None;
        let mut nearest_distance = f32::MAX;

        for (target_entity, target_transform, target_owner, _target_domain, _target_grid_pos) in potential_targets.iter() {
            if target_entity == entity { continue; }
            if !is_enemy(owner, target_owner) { continue; }

            let distance = pos.distance(target_transform.translation);
            if distance > scan_range { continue; }

            if distance < nearest_distance {
                nearest_enemy = Some(target_entity);
                nearest_distance = distance;
            }
        }

        if let Some(target) = nearest_enemy {
            // Engage enemy — save patrol state
            commands.entity(entity).insert(PatrolEngaged {
                patrol_start: start,
                patrol_end: end,
                going_to_end,
            });

            attack_state.current_target = Some(AttackTarget::UnitTarget(target));
            attack_state.phase = AttackPhase::Aiming;
            attack_state.time_in_phase = 0.0;

            // Temporarily switch to attack command
            *command = UnitCommand::AttackTarget(target);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === ATTACK_MOVE_LEASH_DISTANCE ===

    #[test]
    fn attack_move_leash_distance_is_6_gu() {
        assert!((ATTACK_MOVE_LEASH_DISTANCE - 6.0).abs() < f32::EPSILON);
    }

    #[test]
    fn attack_move_leash_greater_than_idle_leash() {
        assert!(ATTACK_MOVE_LEASH_DISTANCE > IDLE_LEASH_DISTANCE);
    }

    // === HOLD_POSITION_FACING_ARC ===

    #[test]
    fn hold_position_facing_arc_is_pi_over_6() {
        assert!((HOLD_POSITION_FACING_ARC - std::f32::consts::FRAC_PI_6).abs() < f32::EPSILON);
    }

    #[test]
    fn hold_position_facing_arc_is_30_degrees() {
        let degrees = HOLD_POSITION_FACING_ARC.to_degrees();
        assert!((degrees - 30.0).abs() < 0.01);
    }

    // === AttackMoveOrigin ===

    #[test]
    fn attack_move_origin_stores_position() {
        let origin = AttackMoveOrigin(Vec3::new(5.0, 0.0, 10.0));
        assert_eq!(origin.0, Vec3::new(5.0, 0.0, 10.0));
    }

    // === PatrolEngaged ===

    #[test]
    fn patrol_engaged_stores_patrol_state() {
        let engaged = PatrolEngaged {
            patrol_start: Vec3::new(0.0, 0.0, 0.0),
            patrol_end: Vec3::new(10.0, 0.0, 10.0),
            going_to_end: true,
        };
        assert_eq!(engaged.patrol_start, Vec3::ZERO);
        assert_eq!(engaged.patrol_end, Vec3::new(10.0, 0.0, 10.0));
        assert!(engaged.going_to_end);
    }

    #[test]
    fn patrol_engaged_going_to_start() {
        let engaged = PatrolEngaged {
            patrol_start: Vec3::new(1.0, 0.0, 1.0),
            patrol_end: Vec3::new(5.0, 0.0, 5.0),
            going_to_end: false,
        };
        assert!(!engaged.going_to_end);
    }

    // === Infantry vs Turret distinction tests ===

    #[test]
    fn light_infantry_no_turret_stops_to_fire() {
        let base_data = UnitBaseEnum::LightInfantry.data();
        assert!(!base_data.has_turret);
        assert!(base_data.can_turn_in_place);
        // Light infantry should stop movement when in range (no turret)
    }

    #[test]
    fn heavy_infantry_no_turret_stops_to_fire() {
        let base_data = UnitBaseEnum::HeavyInfantry.data();
        assert!(!base_data.has_turret);
        assert!(base_data.can_turn_in_place);
    }

    #[test]
    fn wheeled_vehicle_has_turret_keeps_moving() {
        let base_data = UnitBaseEnum::WheeledVehicle.data();
        assert!(base_data.has_turret);
        // Turret units continue moving while turret engages
    }

    #[test]
    fn tracked_vehicle_has_turret_keeps_moving() {
        let base_data = UnitBaseEnum::TrackedVehicle.data();
        assert!(base_data.has_turret);
    }

    #[test]
    fn glider_always_keeps_moving() {
        let base_data = UnitBaseEnum::Glider.data();
        assert_eq!(base_data.movement_model, MovementModelEnum::Glider);
        // Gliders never stop — strafing behavior
    }

    // === Hold position facing arc tests ===

    #[test]
    fn non_turning_infantry_facing_check_within_arc() {
        // WheeledVehicle is the only non-turning unit base
        // But for hold position, the constraint is can_turn_in_place
        let base_data = UnitBaseEnum::WheeledVehicle.data();
        assert!(!base_data.can_turn_in_place);
        // Non-turning: target must be within HOLD_POSITION_FACING_ARC

        // Simulate: forward = (0, 0, -1), target at slight angle
        let forward = Vec3::new(0.0, 0.0, -1.0);
        let to_target = Vec3::new(0.1, 0.0, -1.0).normalize();
        let angle = forward.dot(to_target).acos();
        assert!(angle < HOLD_POSITION_FACING_ARC);
    }

    #[test]
    fn non_turning_infantry_target_outside_arc() {
        // Target far to the side — angle > PI/6
        let forward = Vec3::new(0.0, 0.0, -1.0);
        let to_target = Vec3::new(1.0, 0.0, -0.5).normalize();
        let angle = forward.dot(to_target).acos();
        assert!(angle > HOLD_POSITION_FACING_ARC);
    }

    #[test]
    fn turning_infantry_can_engage_any_direction() {
        let base_data = UnitBaseEnum::LightInfantry.data();
        assert!(base_data.can_turn_in_place);
        // Can turn in place — no facing arc restriction
    }

    // === Attack move scanning behavior ===

    #[test]
    fn attack_move_destination_reached_threshold() {
        // Destination reached when dist < 0.5 GU
        let pos = Vec3::new(10.0, 0.0, 10.0);
        let dest = Vec3::new(10.3, 0.0, 10.3);
        let dist = Vec3::new(pos.x - dest.x, 0.0, pos.z - dest.z).length();
        assert!(dist < 0.5);
    }

    #[test]
    fn attack_move_not_reached_at_1_gu() {
        let pos = Vec3::new(10.0, 0.0, 10.0);
        let dest = Vec3::new(11.0, 0.0, 10.0);
        let dist = Vec3::new(pos.x - dest.x, 0.0, pos.z - dest.z).length();
        assert!(dist >= 0.5);
    }

    // === Leash distance tests ===

    #[test]
    fn within_leash_distance() {
        let origin = Vec3::new(0.0, 0.0, 0.0);
        let pos = Vec3::new(3.0, 0.0, 4.0); // distance = 5.0
        let dist = Vec3::new(pos.x - origin.x, 0.0, pos.z - origin.z).length();
        assert!(dist < ATTACK_MOVE_LEASH_DISTANCE);
    }

    #[test]
    fn beyond_leash_distance() {
        let origin = Vec3::new(0.0, 0.0, 0.0);
        let pos = Vec3::new(5.0, 0.0, 5.0); // distance ~7.07
        let dist = Vec3::new(pos.x - origin.x, 0.0, pos.z - origin.z).length();
        assert!(dist > ATTACK_MOVE_LEASH_DISTANCE);
    }

    // === AttackTarget command variants ===

    #[test]
    fn attack_target_command_matches() {
        let cmd = UnitCommand::AttackTarget(Entity::from_raw(5));
        assert!(matches!(cmd, UnitCommand::AttackTarget(_)));
    }

    #[test]
    fn attack_location_command_matches() {
        let cmd = UnitCommand::AttackLocation(Vec3::new(5.0, 0.0, 5.0));
        assert!(matches!(cmd, UnitCommand::AttackLocation(_)));
    }

    #[test]
    fn attack_move_command_matches() {
        let cmd = UnitCommand::AttackMove(Vec3::new(5.0, 0.0, 5.0));
        assert!(matches!(cmd, UnitCommand::AttackMove(_)));
    }

    #[test]
    fn hold_position_command_matches() {
        let cmd = UnitCommand::HoldPosition;
        assert!(matches!(cmd, UnitCommand::HoldPosition));
    }

    #[test]
    fn patrol_command_matches() {
        let cmd = UnitCommand::Patrol {
            start: Vec3::ZERO,
            end: Vec3::ONE,
            going_to_end: true,
        };
        assert!(matches!(cmd, UnitCommand::Patrol { .. }));
    }
}
