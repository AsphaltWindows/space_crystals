use bevy::prelude::*;
use crate::types::*;
use crate::game::combat::types::{AttackState, Turret, Silhouette};
use crate::game::units::types::*;
use crate::game::units::types::movement::{TurnRateMovementParams, FixedTurnRadiusMovementParams, SpeedTurnRadiusMovementParams, DragMovementParams, GliderMovementParams};

/// Distance threshold for considering a waypoint reached during movement (grid units).
/// Must be large enough that a unit at max speed cannot overshoot in a single frame.
/// At 16 FPS fixed timestep (delta ~0.0625s) and max speed ~4-8 gu/s, per-frame movement is 0.25-0.5 gu.
pub const WAYPOINT_ARRIVAL_THRESHOLD: f32 = 0.5;

/// System to handle unit movement toward target (fallback for non-TurnRate units).
/// Includes ground collision: checks new position against OccupancyMap before applying.
pub fn unit_movement_system(
    mut commands: Commands,
    time: Res<Time>,
    occupancy: Res<OccupancyMap>,
    mut units: Query<
        (Entity, &mut Transform, &mut Velocity, &MovementSpeed, &MoveTarget, Option<&mut Path>,
         Option<&AttackState>, Option<&Turret>, &mut UnitCommand, Option<&Silhouette>, Option<&DomainEnum>),
        (With<Unit>, Without<HoldingPosition>, Without<TurnRateMovementParams>,
         Without<FixedTurnRadiusMovementParams>, Without<SpeedTurnRadiusMovementParams>,
         Without<DragMovementParams>, Without<GliderMovementParams>)
    >,
) {
    let delta = time.delta_secs();

    for (entity, mut transform, mut velocity, speed, _target, path_option,
         attack_state_opt, turret_opt, mut unit_command, silhouette_opt, domain_opt) in &mut units
    {
        // Check attack phase action constraints -- turret-source units can move freely
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
            // Very close to waypoint -- snap position and let the next frame advance waypoint
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

        // Ground collision check -- only for ground-domain units with a silhouette
        let is_ground = domain_opt.map_or(true, |d| *d == DomainEnum::Ground);
        if is_ground {
            if let Some(sil) = silhouette_opt {
                let half_w = sil.width / 2.0;
                let half_h = sil.height / 2.0;
                if occupancy.check_movement_collision(entity, proposed_pos.x, proposed_pos.z, half_w, half_h) {
                    // Collision detected -- stop and request repath
                    velocity.0 = Vec3::ZERO;
                    commands.entity(entity).remove::<Path>().insert(NeedsRepath);
                    continue;
                }
            }
        }

        transform.translation = proposed_pos;
        // Air units hover above ground; ground units stay at ground level
        let is_air_unit = domain_opt.map_or(false, |d| *d == DomainEnum::Air);
        transform.translation.y = if is_air_unit { 1.5 } else { 0.5 };
    }
}

/// System to handle unit rotation toward movement direction (fallback for non-TurnRate units)
pub fn unit_rotation_system(
    time: Res<Time>,
    mut units: Query<
        (&mut Transform, &Velocity, &RotationSpeed),
        (With<Unit>, Without<TurnRateMovementParams>,
         Without<FixedTurnRadiusMovementParams>, Without<SpeedTurnRadiusMovementParams>,
         Without<DragMovementParams>, Without<GliderMovementParams>)
    >,
) {
    let delta = time.delta_secs();

    for (mut transform, velocity, rotation_speed) in &mut units {
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

/// TurnRate movement system -- handles movement for entities with TurnRateMovementParams.
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
    let delta = time.delta_secs();
    if delta < 0.0001 {
        return;
    }

    for (entity, mut transform, mut velocity, params, _target, path_option,
         attack_state_opt, turret_opt, mut unit_command, silhouette_opt, domain_opt) in &mut units
    {
        // Check attack phase action constraints -- turret-source units can move freely
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
            (distance / decel_distance.max(0.5)) * params.max_speed
        } else if angle_to_target > std::f32::consts::FRAC_PI_2 {
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

        // Ground collision check
        let is_ground = domain_opt.map_or(true, |d| *d == DomainEnum::Ground);
        if is_ground {
            if let Some(sil) = silhouette_opt {
                let half_w = sil.width / 2.0;
                let half_h = sil.height / 2.0;
                if occupancy.check_movement_collision(entity, proposed_pos.x, proposed_pos.z, half_w, half_h) {
                    velocity.0 = Vec3::ZERO;
                    commands.entity(entity).remove::<Path>().insert(NeedsRepath);
                    continue;
                }
            }
        }

        transform.translation = proposed_pos;
        let is_air_unit = domain_opt.map_or(false, |d| *d == DomainEnum::Air);
        transform.translation.y = if is_air_unit { 1.5 } else { 0.5 };
    }
}

/// FixedTurnRadius movement system -- handles movement for wheeled vehicles.
/// Units cannot turn in place; turning requires forward (or reverse) motion.
/// Turn rate while moving = current_speed / minimum_turn_radius.
/// Supports forward and reverse movement with separate acceleration/speed params.
pub fn fixed_turn_radius_movement_system(
    mut commands: Commands,
    time: Res<Time>,
    occupancy: Res<OccupancyMap>,
    mut units: Query<
        (Entity, &mut Transform, &mut Velocity, &FixedTurnRadiusMovementParams, &MoveTarget,
         Option<&mut Path>, Option<&AttackState>, Option<&Turret>, &mut UnitCommand,
         Option<&Silhouette>, Option<&DomainEnum>),
        (With<Unit>, Without<HoldingPosition>)
    >,
) {
    let delta = time.delta_secs();
    if delta < 0.0001 {
        return;
    }

    for (entity, mut transform, mut velocity, params, _target, path_option,
         attack_state_opt, turret_opt, mut unit_command, silhouette_opt, domain_opt) in &mut units
    {
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

        let to_waypoint = next_waypoint - current_pos;
        let desired_dir_2d = Vec3::new(to_waypoint.x, 0.0, to_waypoint.z);
        let distance = desired_dir_2d.length();

        if distance < 0.1 {
            transform.translation.x = next_waypoint.x;
            transform.translation.z = next_waypoint.z;
            velocity.0 = Vec3::ZERO;
            continue;
        }

        let desired_dir = desired_dir_2d.normalize();

        let current_forward = transform.forward();
        let current_facing = Vec3::new(current_forward.x, 0.0, current_forward.z).normalize_or_zero();

        let dot_forward = current_facing.dot(desired_dir).clamp(-1.0, 1.0);
        let angle_to_target = dot_forward.acos();

        let reverse_threshold = std::f32::consts::FRAC_PI_2 + std::f32::consts::FRAC_PI_6;
        let should_reverse = angle_to_target > reverse_threshold;

        let current_speed = velocity.0.length();

        if should_reverse {
            let reverse_facing = -current_facing;
            let dot_reverse = reverse_facing.dot(desired_dir).clamp(-1.0, 1.0);
            let reverse_angle = dot_reverse.acos();

            let max_turn = if current_speed > 0.01 {
                params.max_turn_rate_at_speed(current_speed) * delta
            } else {
                0.0
            };

            if reverse_angle > 0.001 && max_turn > 0.0 {
                let cross = current_facing.cross(desired_dir);
                let turn_sign = if cross.y >= 0.0 { -1.0 } else { 1.0 };
                let actual_turn = reverse_angle.min(max_turn);
                let turn_quat = Quat::from_rotation_y(turn_sign * actual_turn);
                transform.rotation = turn_quat * transform.rotation;
            }

            let decel_distance = if params.deceleration < f32::MAX / 2.0 {
                (current_speed * current_speed) / (2.0 * params.deceleration)
            } else {
                0.0
            };

            let target_speed = if distance < decel_distance.max(0.5) {
                (distance / decel_distance.max(0.5)) * params.reverse_max_speed
            } else {
                params.reverse_max_speed
            };

            let new_speed = if target_speed > current_speed {
                (current_speed + params.reverse_acceleration * delta).min(target_speed)
            } else {
                (current_speed - params.deceleration * delta).max(target_speed).max(0.0)
            };

            let updated_forward = transform.forward();
            let facing_2d = Vec3::new(updated_forward.x, 0.0, updated_forward.z).normalize_or_zero();
            velocity.0 = -facing_2d * new_speed;
        } else {
            let max_turn = if current_speed > 0.01 {
                params.max_turn_rate_at_speed(current_speed) * delta
            } else {
                0.0
            };

            if angle_to_target > 0.001 && max_turn > 0.0 {
                let cross = current_facing.cross(desired_dir);
                let turn_sign = if cross.y >= 0.0 { 1.0 } else { -1.0 };
                let actual_turn = angle_to_target.min(max_turn);
                let turn_quat = Quat::from_rotation_y(turn_sign * actual_turn);
                transform.rotation = turn_quat * transform.rotation;
            }

            let decel_distance = if params.deceleration < f32::MAX / 2.0 {
                (current_speed * current_speed) / (2.0 * params.deceleration)
            } else {
                0.0
            };

            let target_speed = if distance < decel_distance.max(0.5) {
                (distance / decel_distance.max(0.5)) * params.forward_max_speed
            } else if angle_to_target > std::f32::consts::FRAC_PI_2 {
                params.forward_max_speed * 0.2
            } else {
                params.forward_max_speed
            };

            let min_creep_speed = if angle_to_target > 0.01 && current_speed < 0.1 {
                params.forward_max_speed * 0.1
            } else {
                0.0
            };

            let new_speed = if target_speed > current_speed {
                (current_speed + params.forward_acceleration * delta).min(target_speed)
            } else {
                (current_speed - params.deceleration * delta).max(target_speed).max(0.0)
            };
            let new_speed = new_speed.max(min_creep_speed);

            let updated_forward = transform.forward();
            let facing_2d = Vec3::new(updated_forward.x, 0.0, updated_forward.z).normalize_or_zero();
            velocity.0 = facing_2d * new_speed;
        }

        let proposed_pos = transform.translation + velocity.0 * delta;

        let is_ground = domain_opt.map_or(true, |d| *d == DomainEnum::Ground);
        if is_ground {
            if let Some(sil) = silhouette_opt {
                let half_w = sil.width / 2.0;
                let half_h = sil.height / 2.0;
                if occupancy.check_movement_collision(entity, proposed_pos.x, proposed_pos.z, half_w, half_h) {
                    velocity.0 = Vec3::ZERO;
                    commands.entity(entity).remove::<Path>().insert(NeedsRepath);
                    continue;
                }
            }
        }

        transform.translation = proposed_pos;
        let is_air_unit = domain_opt.map_or(false, |d| *d == DomainEnum::Air);
        transform.translation.y = if is_air_unit { 1.5 } else { 0.5 };
    }
}

/// SpeedTurnRadius movement system -- handles movement for tracked vehicles.
/// Units can rotate in place when stationary. Turn radius increases with speed:
/// turn_radius = speed * speed_to_turn_radius_ratio.
pub fn speed_turn_radius_movement_system(
    mut commands: Commands,
    time: Res<Time>,
    occupancy: Res<OccupancyMap>,
    mut units: Query<
        (Entity, &mut Transform, &mut Velocity, &SpeedTurnRadiusMovementParams, &MoveTarget,
         Option<&mut Path>, Option<&AttackState>, Option<&Turret>, &mut UnitCommand,
         Option<&Silhouette>, Option<&DomainEnum>),
        (With<Unit>, Without<HoldingPosition>)
    >,
) {
    let delta = time.delta_secs();
    if delta < 0.0001 {
        return;
    }

    for (entity, mut transform, mut velocity, params, _target, path_option,
         attack_state_opt, turret_opt, mut unit_command, silhouette_opt, domain_opt) in &mut units
    {
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

        let to_waypoint = next_waypoint - current_pos;
        let desired_dir_2d = Vec3::new(to_waypoint.x, 0.0, to_waypoint.z);
        let distance = desired_dir_2d.length();

        if distance < 0.1 {
            transform.translation.x = next_waypoint.x;
            transform.translation.z = next_waypoint.z;
            velocity.0 = Vec3::ZERO;
            continue;
        }

        let desired_dir = desired_dir_2d.normalize();

        let current_forward = transform.forward();
        let current_facing = Vec3::new(current_forward.x, 0.0, current_forward.z).normalize_or_zero();

        let dot = current_facing.dot(desired_dir).clamp(-1.0, 1.0);
        let angle_to_target = dot.acos();

        let current_speed = velocity.0.length();

        let max_turn = if current_speed < 0.01 {
            std::f32::consts::TAU * delta
        } else {
            params.max_turn_rate_at_speed(current_speed) * delta
        };

        if angle_to_target > 0.001 {
            let cross = current_facing.cross(desired_dir);
            let turn_sign = if cross.y >= 0.0 { 1.0 } else { -1.0 };
            let actual_turn = angle_to_target.min(max_turn);
            let turn_quat = Quat::from_rotation_y(turn_sign * actual_turn);
            transform.rotation = turn_quat * transform.rotation;
        }

        let reverse_threshold = std::f32::consts::FRAC_PI_2 + std::f32::consts::FRAC_PI_6;
        let should_reverse = angle_to_target > reverse_threshold && current_speed < 0.5;

        let decel_distance = if params.deceleration < f32::MAX / 2.0 {
            (current_speed * current_speed) / (2.0 * params.deceleration)
        } else {
            0.0
        };

        let target_speed = if should_reverse {
            0.0
        } else if distance < decel_distance.max(0.5) {
            (distance / decel_distance.max(0.5)) * params.max_speed
        } else if angle_to_target > std::f32::consts::FRAC_PI_2 {
            params.max_speed * 0.1
        } else if angle_to_target > std::f32::consts::FRAC_PI_4 {
            params.max_speed * 0.5
        } else {
            params.max_speed
        };

        let new_speed = if target_speed > current_speed {
            (current_speed + params.acceleration * delta).min(target_speed)
        } else {
            (current_speed - params.deceleration * delta).max(target_speed).max(0.0)
        };

        let updated_forward = transform.forward();
        let facing_2d = Vec3::new(updated_forward.x, 0.0, updated_forward.z).normalize_or_zero();
        velocity.0 = facing_2d * new_speed;

        let proposed_pos = transform.translation + velocity.0 * delta;

        let is_ground = domain_opt.map_or(true, |d| *d == DomainEnum::Ground);
        if is_ground {
            if let Some(sil) = silhouette_opt {
                let half_w = sil.width / 2.0;
                let half_h = sil.height / 2.0;
                if occupancy.check_movement_collision(entity, proposed_pos.x, proposed_pos.z, half_w, half_h) {
                    velocity.0 = Vec3::ZERO;
                    commands.entity(entity).remove::<Path>().insert(NeedsRepath);
                    continue;
                }
            }
        }

        transform.translation = proposed_pos;
        let is_air_unit = domain_opt.map_or(false, |d| *d == DomainEnum::Air);
        transform.translation.y = if is_air_unit { 1.5 } else { 0.5 };
    }
}

/// Drag movement system -- handles movement for entities with DragMovementParams (HoverVehicle, HoverCraft).
/// Physics-based: thrust (forward + omni-directional) and drag forces determine velocity.
pub fn drag_movement_system(
    mut commands: Commands,
    time: Res<Time>,
    occupancy: Res<OccupancyMap>,
    mut units: Query<
        (Entity, &mut Transform, &mut Velocity, &DragMovementParams, Option<&MoveTarget>,
         Option<&mut Path>, Option<&AttackState>, Option<&Turret>, &mut UnitCommand,
         Option<&Silhouette>, Option<&DomainEnum>),
        (With<Unit>, Without<HoldingPosition>)
    >,
) {
    let delta = time.delta_secs();
    if delta < 0.0001 {
        return;
    }

    for (entity, mut transform, mut velocity, params, move_target_opt, path_option,
         attack_state_opt, turret_opt, mut unit_command, silhouette_opt, domain_opt) in &mut units
    {
        if let Some(attack_state) = attack_state_opt {
            let is_turret_source = turret_opt.is_some();
            let constraints = attack_state.phase.base_action_constraints(is_turret_source);
            if !constraints.base_can_move {
                velocity.0 = Vec3::ZERO;
                continue;
            }
        }

        // Step 1: Apply drag to velocity
        let drag_force = velocity.0 * params.drag_ratio * delta;
        velocity.0 -= drag_force;

        // If no MoveTarget, just let drag decelerate
        if move_target_opt.is_none() {
            if velocity.0.length() < 0.01 {
                velocity.0 = Vec3::ZERO;
            }
            let proposed_pos = transform.translation + velocity.0 * delta;
            let is_ground = domain_opt.map_or(true, |d| *d == DomainEnum::Ground);
            if is_ground {
                if let Some(sil) = silhouette_opt {
                    let half_w = sil.width / 2.0;
                    let half_h = sil.height / 2.0;
                    if occupancy.check_movement_collision(entity, proposed_pos.x, proposed_pos.z, half_w, half_h) {
                        velocity.0 = Vec3::ZERO;
                        continue;
                    }
                }
            }
            transform.translation = proposed_pos;
            let is_air_unit = domain_opt.map_or(false, |d| *d == DomainEnum::Air);
            transform.translation.y = if is_air_unit { 1.5 } else { 0.5 };
            continue;
        }

        let current_pos = transform.translation;

        let next_waypoint = if let Some(mut path) = path_option {
            if path.current_waypoint >= path.waypoints.len() {
                commands.entity(entity).remove::<MoveTarget>().remove::<Path>();
                if matches!(*unit_command, UnitCommand::Move(_) | UnitCommand::AttackMove(_)) {
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
            commands.entity(entity).remove::<MoveTarget>();
            if matches!(*unit_command, UnitCommand::Move(_)) {
                *unit_command = UnitCommand::Idle;
            }
            continue;
        };

        let to_waypoint = next_waypoint - current_pos;
        let desired_dir_2d = Vec3::new(to_waypoint.x, 0.0, to_waypoint.z);
        let distance = desired_dir_2d.length();

        if distance < 0.1 {
            transform.translation.x = next_waypoint.x;
            transform.translation.z = next_waypoint.z;
            continue;
        }

        let desired_dir = desired_dir_2d.normalize();

        let current_forward = transform.forward();
        let current_facing = Vec3::new(current_forward.x, 0.0, current_forward.z).normalize_or_zero();

        let dot = current_facing.dot(desired_dir).clamp(-1.0, 1.0);
        let angle_to_target = dot.acos();
        let max_turn = params.turn_rate * delta;
        if angle_to_target > 0.001 {
            let cross = current_facing.cross(desired_dir);
            let turn_sign = if cross.y >= 0.0 { 1.0 } else { -1.0 };
            let actual_turn = angle_to_target.min(max_turn);
            let turn_quat = Quat::from_rotation_y(turn_sign * actual_turn);
            transform.rotation = turn_quat * transform.rotation;
        }

        // Step 2: Apply thrust
        let updated_forward = transform.forward();
        let facing_2d = Vec3::new(updated_forward.x, 0.0, updated_forward.z).normalize_or_zero();
        velocity.0 += facing_2d * params.forward_acceleration * delta;
        velocity.0 += desired_dir * params.non_forward_acceleration * delta;

        let max_speed = params.max_speed();
        let current_speed = velocity.0.length();
        if current_speed > max_speed {
            velocity.0 = velocity.0.normalize() * max_speed;
        }

        let proposed_pos = transform.translation + velocity.0 * delta;

        let is_ground = domain_opt.map_or(true, |d| *d == DomainEnum::Ground);
        if is_ground {
            if let Some(sil) = silhouette_opt {
                let half_w = sil.width / 2.0;
                let half_h = sil.height / 2.0;
                if occupancy.check_movement_collision(entity, proposed_pos.x, proposed_pos.z, half_w, half_h) {
                    velocity.0 = Vec3::ZERO;
                    commands.entity(entity).remove::<Path>().insert(NeedsRepath);
                    continue;
                }
            }
        }

        transform.translation = proposed_pos;
        let is_air_unit = domain_opt.map_or(false, |d| *d == DomainEnum::Air);
        transform.translation.y = if is_air_unit { 1.5 } else { 0.5 };
    }
}

/// Glider movement system -- handles movement for entities with GliderMovementParams.
/// The glider must always be moving to stay airborne. When idle, it circles at idle_speed.
pub fn glider_movement_system(
    mut commands: Commands,
    time: Res<Time>,
    mut units: Query<
        (Entity, &mut Transform, &mut Velocity, &GliderMovementParams, Option<&MoveTarget>,
         Option<&mut Path>, Option<&AttackState>, Option<&Turret>, &mut UnitCommand),
        (With<Unit>, Without<HoldingPosition>)
    >,
) {
    let delta = time.delta_secs();
    if delta < 0.0001 {
        return;
    }

    for (entity, mut transform, mut velocity, params, move_target_opt, path_option,
         attack_state_opt, turret_opt, mut unit_command) in &mut units
    {
        if let Some(attack_state) = attack_state_opt {
            let is_turret_source = turret_opt.is_some();
            let constraints = attack_state.phase.base_action_constraints(is_turret_source);
            if !constraints.base_can_move {
                let current_speed = velocity.0.length();
                if current_speed < params.idle_speed {
                    let forward = transform.forward();
                    let facing_2d = Vec3::new(forward.x, 0.0, forward.z).normalize_or_zero();
                    velocity.0 = facing_2d * params.idle_speed;
                }
                let proposed_pos = transform.translation + velocity.0 * delta;
                transform.translation = proposed_pos;
                transform.translation.y = 1.5;
                continue;
            }
        }

        let current_forward = transform.forward();
        let current_facing = Vec3::new(current_forward.x, 0.0, current_forward.z).normalize_or_zero();
        let current_speed = velocity.0.length().max(0.01);

        let has_move_target = move_target_opt.is_some();

        if !has_move_target {
            // Idle circling
            let target_speed = params.idle_speed;
            let new_speed = if current_speed > target_speed {
                (current_speed - params.deceleration * delta).max(target_speed)
            } else {
                (current_speed + params.acceleration * delta).min(target_speed)
            };

            let circle_turn_rate = params.max_centripetal_acceleration / new_speed.max(0.1);
            let actual_turn = circle_turn_rate * delta;
            let turn_quat = Quat::from_rotation_y(-actual_turn);
            transform.rotation = turn_quat * transform.rotation;

            let updated_forward = transform.forward();
            let facing_2d = Vec3::new(updated_forward.x, 0.0, updated_forward.z).normalize_or_zero();
            velocity.0 = facing_2d * new_speed;

            let proposed_pos = transform.translation + velocity.0 * delta;
            transform.translation = proposed_pos;
            transform.translation.y = 1.5;
            continue;
        }

        // Has move target: fly toward waypoints
        let current_pos = transform.translation;

        let next_waypoint = if let Some(mut path) = path_option {
            if path.current_waypoint >= path.waypoints.len() {
                commands.entity(entity).remove::<MoveTarget>().remove::<Path>();
                if matches!(*unit_command, UnitCommand::Move(_) | UnitCommand::AttackMove(_)) {
                    *unit_command = UnitCommand::Idle;
                }
                let updated_forward = transform.forward();
                let facing_2d = Vec3::new(updated_forward.x, 0.0, updated_forward.z).normalize_or_zero();
                velocity.0 = facing_2d * current_speed;
                let proposed_pos = transform.translation + velocity.0 * delta;
                transform.translation = proposed_pos;
                transform.translation.y = 1.5;
                continue;
            }

            let waypoint = path.waypoints[path.current_waypoint];
            let to_waypoint = waypoint - current_pos;
            let distance_to_waypoint = Vec3::new(to_waypoint.x, 0.0, to_waypoint.z).length();

            if distance_to_waypoint < WAYPOINT_ARRIVAL_THRESHOLD {
                path.current_waypoint += 1;

                if path.current_waypoint >= path.waypoints.len() {
                    commands.entity(entity).remove::<MoveTarget>().remove::<Path>();
                    if matches!(*unit_command, UnitCommand::Move(_)) {
                        *unit_command = UnitCommand::Idle;
                    }
                    let updated_forward = transform.forward();
                    let facing_2d = Vec3::new(updated_forward.x, 0.0, updated_forward.z).normalize_or_zero();
                    velocity.0 = facing_2d * current_speed;
                    let proposed_pos = transform.translation + velocity.0 * delta;
                    transform.translation = proposed_pos;
                    transform.translation.y = 1.5;
                    continue;
                }

                path.waypoints[path.current_waypoint]
            } else {
                waypoint
            }
        } else {
            commands.entity(entity).remove::<MoveTarget>();
            if matches!(*unit_command, UnitCommand::Move(_)) {
                *unit_command = UnitCommand::Idle;
            }
            let updated_forward = transform.forward();
            let facing_2d = Vec3::new(updated_forward.x, 0.0, updated_forward.z).normalize_or_zero();
            velocity.0 = facing_2d * current_speed;
            let proposed_pos = transform.translation + velocity.0 * delta;
            transform.translation = proposed_pos;
            transform.translation.y = 1.5;
            continue;
        };

        // Accelerate toward max_speed when given orders
        let target_speed = params.max_speed;
        let new_speed = if current_speed < target_speed {
            (current_speed + params.acceleration * delta).min(target_speed)
        } else {
            (current_speed - params.deceleration * delta).max(target_speed)
        };

        let to_waypoint = next_waypoint - current_pos;
        let desired_dir_2d = Vec3::new(to_waypoint.x, 0.0, to_waypoint.z);
        let distance = desired_dir_2d.length();

        if distance > 0.1 {
            let desired_dir = desired_dir_2d.normalize();

            let dot = current_facing.dot(desired_dir).clamp(-1.0, 1.0);
            let angle_to_target = dot.acos();

            let max_turn_rate = params.max_centripetal_acceleration / new_speed.max(0.1);
            let max_turn = max_turn_rate * delta;

            if angle_to_target > 0.001 {
                let cross = current_facing.cross(desired_dir);
                let turn_sign = if cross.y >= 0.0 { 1.0 } else { -1.0 };
                let actual_turn = angle_to_target.min(max_turn);
                let turn_quat = Quat::from_rotation_y(turn_sign * actual_turn);
                transform.rotation = turn_quat * transform.rotation;
            }
        }

        let updated_forward = transform.forward();
        let facing_2d = Vec3::new(updated_forward.x, 0.0, updated_forward.z).normalize_or_zero();
        velocity.0 = facing_2d * new_speed;

        let proposed_pos = transform.translation + velocity.0 * delta;
        transform.translation = proposed_pos;
        transform.translation.y = 1.5;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;
    use crate::game::combat::types::AttackPhase;

    // === FixedTurnRadius movement system tests ===

    fn default_fixed_turn_params() -> FixedTurnRadiusMovementParams {
        FixedTurnRadiusMovementParams {
            minimum_turn_radius: 2.0,
            forward_acceleration: 4.0,
            forward_max_speed: 6.0,
            reverse_acceleration: 2.0,
            reverse_max_speed: 3.0,
            deceleration: 8.0,
        }
    }

    fn spawn_fixed_turn_unit(world: &mut World, pos: Vec3, target: Vec3, facing_angle: f32) -> Entity {
        let rotation = Quat::from_rotation_y(facing_angle);
        world.spawn((
            Unit,
            Transform::from_translation(pos).with_rotation(rotation),
            Velocity(Vec3::ZERO),
            default_fixed_turn_params(),
            MoveTarget(target),
            Path { waypoints: vec![target], current_waypoint: 0 },
            UnitCommand::Move(target),
            GridPosition { x: 32, z: 32 },
        )).id()
    }

    #[test]
    fn fixed_turn_radius_moves_forward_when_facing_target() {
        let mut world = World::new();
        world.init_resource::<OccupancyMap>();
        world.init_resource::<Time<()>>();
        world.resource_mut::<Time<()>>().advance_by(std::time::Duration::from_millis(100));

        let entity = spawn_fixed_turn_unit(&mut world, Vec3::new(0.0, 0.5, 0.0), Vec3::new(0.0, 0.5, -10.0), 0.0);
        world.run_system_once(fixed_turn_radius_movement_system).unwrap();

        let vel = world.entity(entity).get::<Velocity>().unwrap();
        assert!(vel.0.length() > 0.0, "Unit should begin moving forward");
        assert!(vel.0.z < 0.0, "Velocity should be in -Z direction");
    }

    #[test]
    fn fixed_turn_radius_cannot_turn_when_stationary() {
        let mut world = World::new();
        world.init_resource::<OccupancyMap>();
        world.init_resource::<Time<()>>();
        world.resource_mut::<Time<()>>().advance_by(std::time::Duration::from_millis(100));

        let entity = spawn_fixed_turn_unit(&mut world, Vec3::new(0.0, 0.5, 0.0), Vec3::new(10.0, 0.5, 0.0), 0.0);
        world.run_system_once(fixed_turn_radius_movement_system).unwrap();

        let vel = world.entity(entity).get::<Velocity>().unwrap();
        assert!(vel.0.length() > 0.0, "Unit should creep forward to enable turning");
    }

    #[test]
    fn fixed_turn_radius_reverses_when_target_behind() {
        let mut world = World::new();
        world.init_resource::<OccupancyMap>();
        world.init_resource::<Time<()>>();
        world.resource_mut::<Time<()>>().advance_by(std::time::Duration::from_millis(100));

        let entity = spawn_fixed_turn_unit(&mut world, Vec3::new(0.0, 0.5, 0.0), Vec3::new(0.0, 0.5, 10.0), 0.0);
        world.entity_mut(entity).get_mut::<Velocity>().unwrap().0 = Vec3::new(0.0, 0.0, -0.5);
        world.run_system_once(fixed_turn_radius_movement_system).unwrap();

        let vel = world.entity(entity).get::<Velocity>().unwrap();
        assert!(vel.0.length() > 0.0, "Unit should have non-zero velocity");
    }

    #[test]
    fn fixed_turn_radius_completes_move_command() {
        let mut world = World::new();
        world.init_resource::<OccupancyMap>();
        world.init_resource::<Time<()>>();
        world.resource_mut::<Time<()>>().advance_by(std::time::Duration::from_millis(16));

        let target = Vec3::new(0.0, 0.5, -0.05);
        let entity = spawn_fixed_turn_unit(&mut world, Vec3::new(0.0, 0.5, 0.0), target, 0.0);
        world.run_system_once(fixed_turn_radius_movement_system).unwrap();

        let vel = world.entity(entity).get::<Velocity>().unwrap();
        assert!(vel.0.length() < 0.01, "Unit should stop near target");
    }

    #[test]
    fn fixed_turn_radius_stops_on_collision() {
        let mut world = World::new();
        world.init_resource::<Time<()>>();
        world.resource_mut::<Time<()>>().advance_by(std::time::Duration::from_millis(100));

        let mut occupancy = OccupancyMap::default();
        occupancy.ground_bodies.push(CollisionBody {
            entity: Entity::from_raw_u32(9999).unwrap(),
            x: 0.0, z: -2.0, half_w: 5.0, half_h: 5.0,
        });
        world.insert_resource(occupancy);

        let entity = spawn_fixed_turn_unit(&mut world, Vec3::new(0.0, 0.5, 0.0), Vec3::new(0.0, 0.5, -10.0), 0.0);
        world.entity_mut(entity).insert((
            Velocity(Vec3::new(0.0, 0.0, -4.0)),
            Silhouette { width: 1.0, height: 1.0 },
            DomainEnum::Ground,
        ));

        world.run_system_once(fixed_turn_radius_movement_system).unwrap();

        let vel = world.entity(entity).get::<Velocity>().unwrap();
        assert!(vel.0.length() < 0.01, "Unit should stop on collision");
        assert!(world.entity(entity).get::<NeedsRepath>().is_some(), "Unit should need repath");
    }

    // === SpeedTurnRadius movement system tests ===

    fn default_speed_turn_params() -> SpeedTurnRadiusMovementParams {
        SpeedTurnRadiusMovementParams {
            speed_to_turn_radius_ratio: 0.5,
            acceleration: 4.0,
            deceleration: 8.0,
            max_speed: 5.0,
        }
    }

    fn spawn_speed_turn_unit(world: &mut World, pos: Vec3, target: Vec3, facing_angle: f32) -> Entity {
        let rotation = Quat::from_rotation_y(facing_angle);
        world.spawn((
            Unit,
            Transform::from_translation(pos).with_rotation(rotation),
            Velocity(Vec3::ZERO),
            default_speed_turn_params(),
            MoveTarget(target),
            Path { waypoints: vec![target], current_waypoint: 0 },
            UnitCommand::Move(target),
            GridPosition { x: 32, z: 32 },
        )).id()
    }

    #[test]
    fn speed_turn_radius_moves_forward() {
        let mut world = World::new();
        world.init_resource::<OccupancyMap>();
        world.init_resource::<Time<()>>();
        world.resource_mut::<Time<()>>().advance_by(std::time::Duration::from_millis(100));

        let entity = spawn_speed_turn_unit(&mut world, Vec3::new(0.0, 0.5, 0.0), Vec3::new(0.0, 0.5, -10.0), 0.0);
        world.run_system_once(speed_turn_radius_movement_system).unwrap();

        let vel = world.entity(entity).get::<Velocity>().unwrap();
        assert!(vel.0.length() > 0.0, "Unit should begin moving");
        assert!(vel.0.z < 0.0, "Velocity should be in -Z direction");
    }

    #[test]
    fn speed_turn_radius_can_turn_in_place() {
        let mut world = World::new();
        world.init_resource::<OccupancyMap>();
        world.init_resource::<Time<()>>();
        world.resource_mut::<Time<()>>().advance_by(std::time::Duration::from_millis(100));

        let entity = spawn_speed_turn_unit(&mut world, Vec3::new(0.0, 0.5, 0.0), Vec3::new(10.0, 0.5, 0.0), 0.0);
        let rot_before = world.entity(entity).get::<Transform>().unwrap().rotation;
        world.run_system_once(speed_turn_radius_movement_system).unwrap();

        let rot_after = world.entity(entity).get::<Transform>().unwrap().rotation;
        let angle_changed = rot_before.angle_between(rot_after);
        assert!(angle_changed > 0.01, "TrackedVehicle should turn in place (angle changed: {})", angle_changed);
    }

    #[test]
    fn speed_turn_radius_slows_for_sharp_turns() {
        let mut world = World::new();
        world.init_resource::<OccupancyMap>();
        world.init_resource::<Time<()>>();
        world.resource_mut::<Time<()>>().advance_by(std::time::Duration::from_millis(100));

        let entity = spawn_speed_turn_unit(&mut world, Vec3::new(0.0, 0.5, 0.0), Vec3::new(10.0, 0.5, 0.0), 0.0);
        world.entity_mut(entity).get_mut::<Velocity>().unwrap().0 = Vec3::new(0.0, 0.0, -5.0);
        world.run_system_once(speed_turn_radius_movement_system).unwrap();

        let vel = world.entity(entity).get::<Velocity>().unwrap();
        assert!(vel.0.length() < 5.0, "Unit should slow down for sharp turn, speed: {}", vel.0.length());
    }

    #[test]
    fn speed_turn_radius_completes_move_command() {
        let mut world = World::new();
        world.init_resource::<OccupancyMap>();
        world.init_resource::<Time<()>>();
        world.resource_mut::<Time<()>>().advance_by(std::time::Duration::from_millis(16));

        let target = Vec3::new(0.0, 0.5, -0.05);
        let entity = spawn_speed_turn_unit(&mut world, Vec3::new(0.0, 0.5, 0.0), target, 0.0);
        world.run_system_once(speed_turn_radius_movement_system).unwrap();

        let vel = world.entity(entity).get::<Velocity>().unwrap();
        assert!(vel.0.length() < 0.01, "Unit should stop near target");
    }

    #[test]
    fn speed_turn_radius_stops_on_collision() {
        let mut world = World::new();
        world.init_resource::<Time<()>>();
        world.resource_mut::<Time<()>>().advance_by(std::time::Duration::from_millis(100));

        let mut occupancy = OccupancyMap::default();
        occupancy.ground_bodies.push(CollisionBody {
            entity: Entity::from_raw_u32(9999).unwrap(),
            x: 0.0, z: -2.0, half_w: 5.0, half_h: 5.0,
        });
        world.insert_resource(occupancy);

        let entity = spawn_speed_turn_unit(&mut world, Vec3::new(0.0, 0.5, 0.0), Vec3::new(0.0, 0.5, -10.0), 0.0);
        world.entity_mut(entity).insert((
            Velocity(Vec3::new(0.0, 0.0, -4.0)),
            Silhouette { width: 1.0, height: 1.0 },
            DomainEnum::Ground,
        ));
        world.run_system_once(speed_turn_radius_movement_system).unwrap();

        let vel = world.entity(entity).get::<Velocity>().unwrap();
        assert!(vel.0.length() < 0.01, "Unit should stop on collision");
        assert!(world.entity(entity).get::<NeedsRepath>().is_some(), "Unit should need repath");
    }

    #[test]
    fn fixed_turn_radius_attack_constraint_prevents_movement() {
        let mut world = World::new();
        world.init_resource::<OccupancyMap>();
        world.init_resource::<Time<()>>();
        world.resource_mut::<Time<()>>().advance_by(std::time::Duration::from_millis(100));

        let entity = spawn_fixed_turn_unit(&mut world, Vec3::new(0.0, 0.5, 0.0), Vec3::new(0.0, 0.5, -10.0), 0.0);
        world.entity_mut(entity).insert(AttackState {
            phase: AttackPhase::Firing, time_in_phase: 0.0, current_target: None,
        });
        world.run_system_once(fixed_turn_radius_movement_system).unwrap();

        let vel = world.entity(entity).get::<Velocity>().unwrap();
        assert!(vel.0.length() < 0.01, "Unit should not move while firing without turret");
    }

    #[test]
    fn speed_turn_radius_not_processed_by_unit_movement_system() {
        let mut world = World::new();
        world.init_resource::<OccupancyMap>();
        world.init_resource::<Time<()>>();
        world.resource_mut::<Time<()>>().advance_by(std::time::Duration::from_millis(100));

        let entity = spawn_speed_turn_unit(&mut world, Vec3::new(0.0, 0.5, 0.0), Vec3::new(0.0, 0.5, -10.0), 0.0);
        world.entity_mut(entity).insert(MovementSpeed(5.0));
        world.run_system_once(unit_movement_system).unwrap();

        let vel = world.entity(entity).get::<Velocity>().unwrap();
        assert!(vel.0.length() < 0.01, "unit_movement_system should not process SpeedTurnRadius entities");
    }

    // === Drag movement system tests ===

    fn default_drag_params() -> DragMovementParams {
        DragMovementParams { forward_acceleration: 6.0, non_forward_acceleration: 4.0, drag_ratio: 2.0, turn_rate: 2.5 }
    }

    fn spawn_drag_unit(world: &mut World, pos: Vec3, target: Option<Vec3>, facing_angle: f32) -> Entity {
        let rotation = Quat::from_rotation_y(facing_angle);
        let mut entity = world.spawn((
            Unit, Transform::from_translation(pos).with_rotation(rotation),
            Velocity(Vec3::ZERO), default_drag_params(), UnitCommand::Idle, GridPosition { x: 32, z: 32 },
        ));
        if let Some(t) = target {
            entity.insert((MoveTarget(t), Path { waypoints: vec![t], current_waypoint: 0 }));
            entity.insert(UnitCommand::Move(t));
        }
        entity.id()
    }

    #[test]
    fn drag_movement_accelerates_toward_target() {
        let mut world = World::new();
        world.init_resource::<OccupancyMap>();
        world.init_resource::<Time<()>>();
        world.resource_mut::<Time<()>>().advance_by(std::time::Duration::from_millis(100));

        let entity = spawn_drag_unit(&mut world, Vec3::new(0.0, 0.5, 0.0), Some(Vec3::new(0.0, 0.5, -10.0)), 0.0);
        world.run_system_once(drag_movement_system).unwrap();

        let vel = world.entity(entity).get::<Velocity>().unwrap();
        assert!(vel.0.length() > 0.0, "Unit should accelerate toward target");
        assert!(vel.0.z < 0.0, "Velocity should be in -Z direction");
    }

    #[test]
    fn drag_movement_drag_decelerates_when_idle() {
        let mut world = World::new();
        world.init_resource::<OccupancyMap>();
        world.init_resource::<Time<()>>();
        world.resource_mut::<Time<()>>().advance_by(std::time::Duration::from_millis(100));

        let entity = world.spawn((
            Unit, Transform::from_translation(Vec3::new(0.0, 0.5, 0.0)),
            Velocity(Vec3::new(0.0, 0.0, -3.0)), default_drag_params(),
            UnitCommand::Idle, GridPosition { x: 32, z: 32 },
        )).id();
        world.run_system_once(drag_movement_system).unwrap();

        let vel = world.entity(entity).get::<Velocity>().unwrap();
        assert!(vel.0.length() < 3.0, "Drag should slow the unit down");
        assert!(vel.0.length() > 0.0, "Unit should still be moving");
    }

    #[test]
    fn drag_movement_respects_max_speed() {
        let mut world = World::new();
        world.init_resource::<OccupancyMap>();
        world.init_resource::<Time<()>>();
        world.resource_mut::<Time<()>>().advance_by(std::time::Duration::from_secs(10));

        let entity = spawn_drag_unit(&mut world, Vec3::new(0.0, 0.5, 0.0), Some(Vec3::new(0.0, 0.5, -100.0)), 0.0);
        world.run_system_once(drag_movement_system).unwrap();

        let vel = world.entity(entity).get::<Velocity>().unwrap();
        let max_speed = default_drag_params().max_speed();
        assert!(vel.0.length() <= max_speed + 0.01, "Velocity should not exceed max speed ({} > {})", vel.0.length(), max_speed);
    }

    #[test]
    fn drag_movement_completes_move_command() {
        let mut world = World::new();
        world.init_resource::<OccupancyMap>();
        world.init_resource::<Time<()>>();
        world.resource_mut::<Time<()>>().advance_by(std::time::Duration::from_millis(100));

        let target = Vec3::new(0.0, 0.5, -0.05);
        let entity = spawn_drag_unit(&mut world, Vec3::new(0.0, 0.5, 0.0), Some(target), 0.0);
        world.run_system_once(drag_movement_system).unwrap();

        assert!(world.entity(entity).get::<MoveTarget>().is_none(), "MoveTarget should be removed");
        let cmd = world.entity(entity).get::<UnitCommand>().unwrap();
        assert!(matches!(*cmd, UnitCommand::Idle), "Command should be Idle after completing move");
    }

    #[test]
    fn drag_movement_not_processed_by_unit_movement_system() {
        let mut world = World::new();
        world.init_resource::<OccupancyMap>();
        world.init_resource::<Time<()>>();
        world.resource_mut::<Time<()>>().advance_by(std::time::Duration::from_millis(100));

        let entity = spawn_drag_unit(&mut world, Vec3::new(0.0, 0.5, 0.0), Some(Vec3::new(0.0, 0.5, -10.0)), 0.0);
        world.entity_mut(entity).insert(MovementSpeed(5.0));
        world.run_system_once(unit_movement_system).unwrap();

        let vel = world.entity(entity).get::<Velocity>().unwrap();
        assert!(vel.0.length() < 0.01, "unit_movement_system should not process Drag entities");
    }

    #[test]
    fn drag_movement_air_unit_at_air_height() {
        let mut world = World::new();
        world.init_resource::<OccupancyMap>();
        world.init_resource::<Time<()>>();
        world.resource_mut::<Time<()>>().advance_by(std::time::Duration::from_millis(100));

        let entity = spawn_drag_unit(&mut world, Vec3::new(0.0, 1.5, 0.0), Some(Vec3::new(0.0, 1.5, -10.0)), 0.0);
        world.entity_mut(entity).insert(DomainEnum::Air);
        world.run_system_once(drag_movement_system).unwrap();

        let transform = world.entity(entity).get::<Transform>().unwrap();
        assert!((transform.translation.y - 1.5).abs() < 0.01, "Air unit should stay at y=1.5");
    }

    // === Glider movement system tests ===

    fn default_glider_params() -> GliderMovementParams {
        GliderMovementParams { idle_speed: 5.0, max_speed: 15.0, acceleration: 3.0, deceleration: 6.0, max_centripetal_acceleration: 10.0 }
    }

    fn spawn_glider_unit(world: &mut World, pos: Vec3, target: Option<Vec3>, facing_angle: f32) -> Entity {
        let rotation = Quat::from_rotation_y(facing_angle);
        let mut entity = world.spawn((
            Unit, Transform::from_translation(pos).with_rotation(rotation),
            Velocity(Vec3::new(0.0, 0.0, -5.0)),
            default_glider_params(), UnitCommand::Idle, GridPosition { x: 32, z: 32 },
        ));
        if let Some(t) = target {
            entity.insert((MoveTarget(t), Path { waypoints: vec![t], current_waypoint: 0 }));
            entity.insert(UnitCommand::Move(t));
        }
        entity.id()
    }

    #[test]
    fn glider_circles_when_idle() {
        let mut world = World::new();
        world.init_resource::<Time<()>>();
        world.resource_mut::<Time<()>>().advance_by(std::time::Duration::from_millis(100));

        let entity = spawn_glider_unit(&mut world, Vec3::new(0.0, 1.5, 0.0), None, 0.0);
        let initial_rotation = world.entity(entity).get::<Transform>().unwrap().rotation;
        world.run_system_once(glider_movement_system).unwrap();

        let transform = world.entity(entity).get::<Transform>().unwrap();
        let vel = world.entity(entity).get::<Velocity>().unwrap();
        assert!(vel.0.length() > 4.0, "Glider should maintain speed when idle");
        assert_ne!(transform.rotation, initial_rotation, "Glider should turn when idle (circling)");
        assert!((transform.translation.y - 1.5).abs() < 0.01, "Glider should stay at y=1.5");
    }

    #[test]
    fn glider_accelerates_toward_max_speed_when_ordered() {
        let mut world = World::new();
        world.init_resource::<Time<()>>();
        world.resource_mut::<Time<()>>().advance_by(std::time::Duration::from_millis(100));

        let entity = spawn_glider_unit(&mut world, Vec3::new(0.0, 1.5, 0.0), Some(Vec3::new(0.0, 1.5, -50.0)), 0.0);
        let initial_speed = world.entity(entity).get::<Velocity>().unwrap().0.length();
        world.run_system_once(glider_movement_system).unwrap();

        let vel = world.entity(entity).get::<Velocity>().unwrap();
        assert!(vel.0.length() > initial_speed, "Glider should accelerate when given orders");
    }

    #[test]
    fn glider_never_stops_after_completing_waypoints() {
        let mut world = World::new();
        world.init_resource::<Time<()>>();
        world.resource_mut::<Time<()>>().advance_by(std::time::Duration::from_millis(100));

        let target = Vec3::new(0.0, 1.5, -0.1);
        let entity = spawn_glider_unit(&mut world, Vec3::new(0.0, 1.5, 0.0), Some(target), 0.0);
        world.run_system_once(glider_movement_system).unwrap();

        let vel = world.entity(entity).get::<Velocity>().unwrap();
        assert!(vel.0.length() > 1.0, "Glider should never stop");
        assert!(world.entity(entity).get::<MoveTarget>().is_none(), "MoveTarget should be removed");
    }

    #[test]
    fn glider_decelerates_toward_idle_speed_when_no_target() {
        let mut world = World::new();
        world.init_resource::<Time<()>>();
        world.resource_mut::<Time<()>>().advance_by(std::time::Duration::from_millis(100));

        let entity = world.spawn((
            Unit, Transform::from_translation(Vec3::new(0.0, 1.5, 0.0)),
            Velocity(Vec3::new(0.0, 0.0, -15.0)),
            default_glider_params(), UnitCommand::Idle, GridPosition { x: 32, z: 32 },
        )).id();
        world.run_system_once(glider_movement_system).unwrap();

        let vel = world.entity(entity).get::<Velocity>().unwrap();
        assert!(vel.0.length() < 15.0, "Glider should decelerate toward idle_speed");
        assert!(vel.0.length() > 4.0, "Glider should still be moving");
    }

    #[test]
    fn glider_speed_dependent_turn_radius() {
        let params = default_glider_params();
        let idle_radius = params.turn_radius(params.idle_speed);
        assert!((idle_radius - 2.5).abs() < 0.01, "Idle turn radius should be 2.5");
        let max_radius = params.turn_radius(params.max_speed);
        assert!((max_radius - 22.5).abs() < 0.01, "Max speed turn radius should be 22.5");
        assert!(max_radius > idle_radius, "Turn radius should increase with speed");
    }

    #[test]
    fn glider_not_processed_by_unit_movement_system() {
        let mut world = World::new();
        world.init_resource::<OccupancyMap>();
        world.init_resource::<Time<()>>();
        world.resource_mut::<Time<()>>().advance_by(std::time::Duration::from_millis(100));

        let entity = spawn_glider_unit(&mut world, Vec3::new(0.0, 1.5, 0.0), Some(Vec3::new(0.0, 1.5, -10.0)), 0.0);
        world.entity_mut(entity).insert(MovementSpeed(5.0));
        let initial_vel = world.entity(entity).get::<Velocity>().unwrap().0;
        world.run_system_once(unit_movement_system).unwrap();

        let vel = world.entity(entity).get::<Velocity>().unwrap();
        assert_eq!(vel.0, initial_vel, "unit_movement_system should not process Glider entities");
    }
}
