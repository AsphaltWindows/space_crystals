use bevy::prelude::*;
use crate::types::*;
use crate::game::types::ObjectInstance;
use crate::game::combat::types::Silhouette;
use crate::game::units::types::*;
use crate::game::units::types::movement::{TurnRateMovementParams, FixedTurnRadiusMovementParams, SpeedTurnRadiusMovementParams, DragMovementParams, GliderMovementParams};

/// Channel-driven locomotion consumer for TurnRate units.
/// Reads LocomotionChannel (written by behavior systems in Phase 2) and drives
/// Transform/Velocity. Mirrors turn_rate_movement_system logic but reads from
/// channels instead of MoveTarget/Path components.
///
/// Coexistence: only processes entities WITHOUT MoveTarget (channel-driven path).
pub fn channel_turnrate_locomotion_system(
    time: Res<Time>,
    occupancy: Res<OccupancyMap>,
    mut units: Query<
        (Entity, &mut Transform, &mut Velocity, &LocomotionChannel,
         &TurnRateMovementParams, Option<&Silhouette>, Option<&DomainEnum>),
        (With<Unit>, Without<MoveTarget>, Without<HoldingPosition>)
    >,
) {
    let delta = time.delta_secs();
    if delta < 0.0001 {
        return;
    }

    for (entity, mut transform, mut velocity, locomotion, params,
         silhouette_opt, domain_opt) in &mut units
    {
        match locomotion {
            LocomotionChannel::Moving(path) => {
                if path.is_empty() {
                    velocity.0 = Vec3::ZERO;
                    continue;
                }
                let target = path[0];
                let current_pos = transform.translation;
                let to_target = Vec3::new(target.x - current_pos.x, 0.0, target.z - current_pos.z);
                let distance = to_target.length();

                if distance < 0.1 {
                    transform.translation.x = target.x;
                    transform.translation.z = target.z;
                    velocity.0 = Vec3::ZERO;
                    continue;
                }

                let desired_dir = to_target.normalize();
                let current_forward = transform.forward();
                let current_facing = Vec3::new(current_forward.x, 0.0, current_forward.z).normalize_or_zero();
                let dot = current_facing.dot(desired_dir).clamp(-1.0, 1.0);
                let angle_to_target = dot.acos();

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
                            continue;
                        }
                    }
                }

                transform.translation = proposed_pos;
                let is_air = domain_opt.map_or(false, |d| *d == DomainEnum::Air);
                transform.translation.y = if is_air { 1.5 } else { 0.5 };
            }
            LocomotionChannel::Reversing(path) => {
                if path.is_empty() {
                    velocity.0 = Vec3::ZERO;
                    continue;
                }
                let target = path[0];
                let current_pos = transform.translation;
                let to_target = Vec3::new(target.x - current_pos.x, 0.0, target.z - current_pos.z);
                let distance = to_target.length();

                if distance < 0.1 {
                    transform.translation.x = target.x;
                    transform.translation.z = target.z;
                    velocity.0 = Vec3::ZERO;
                    continue;
                }

                let current_forward = transform.forward();
                let facing_2d = Vec3::new(current_forward.x, 0.0, current_forward.z).normalize_or_zero();
                let reverse_dir = -facing_2d;

                let current_speed = velocity.0.length();
                let reverse_max = params.max_speed * 0.5;
                let target_speed = if distance < 1.0 {
                    reverse_max * (distance / 1.0)
                } else {
                    reverse_max
                };

                let new_speed = if target_speed > current_speed {
                    (current_speed + params.acceleration * delta).min(target_speed)
                } else {
                    (current_speed - params.deceleration * delta).max(target_speed).max(0.0)
                };

                velocity.0 = reverse_dir * new_speed;

                let proposed_pos = transform.translation + velocity.0 * delta;

                let is_ground = domain_opt.map_or(true, |d| *d == DomainEnum::Ground);
                if is_ground {
                    if let Some(sil) = silhouette_opt {
                        if occupancy.check_movement_collision(entity, proposed_pos.x, proposed_pos.z, sil.width / 2.0, sil.height / 2.0) {
                            velocity.0 = Vec3::ZERO;
                            continue;
                        }
                    }
                }

                transform.translation = proposed_pos;
                let is_air = domain_opt.map_or(false, |d| *d == DomainEnum::Air);
                transform.translation.y = if is_air { 1.5 } else { 0.5 };
            }
            LocomotionChannel::Stopping => {
                let current_speed = velocity.0.length();
                if current_speed < 0.01 {
                    velocity.0 = Vec3::ZERO;
                    continue;
                }
                let new_speed = if params.deceleration < f32::MAX / 2.0 {
                    (current_speed - params.deceleration * delta).max(0.0)
                } else {
                    0.0
                };
                velocity.0 = velocity.0.normalize_or_zero() * new_speed;

                let proposed_pos = transform.translation + velocity.0 * delta;

                let is_ground = domain_opt.map_or(true, |d| *d == DomainEnum::Ground);
                if is_ground {
                    if let Some(sil) = silhouette_opt {
                        if occupancy.check_movement_collision(entity, proposed_pos.x, proposed_pos.z, sil.width / 2.0, sil.height / 2.0) {
                            velocity.0 = Vec3::ZERO;
                            continue;
                        }
                    }
                }

                transform.translation = proposed_pos;
                let is_air = domain_opt.map_or(false, |d| *d == DomainEnum::Air);
                transform.translation.y = if is_air { 1.5 } else { 0.5 };
            }
            LocomotionChannel::Stationary => {
                // No channel-driven movement
            }
        }
    }
}

/// Channel-driven locomotion consumer for non-TurnRate units (fallback).
/// Only processes entities WITHOUT MoveTarget and WITHOUT TurnRateMovementParams.
pub fn channel_fallback_locomotion_system(
    time: Res<Time>,
    occupancy: Res<OccupancyMap>,
    mut units: Query<
        (Entity, &mut Transform, &mut Velocity, &LocomotionChannel,
         &MovementSpeed, Option<&Silhouette>, Option<&DomainEnum>),
        (With<Unit>, Without<MoveTarget>, Without<TurnRateMovementParams>,
         Without<FixedTurnRadiusMovementParams>, Without<SpeedTurnRadiusMovementParams>,
         Without<DragMovementParams>, Without<GliderMovementParams>, Without<HoldingPosition>)
    >,
) {
    let delta = time.delta_secs();
    if delta < 0.0001 {
        return;
    }

    for (entity, mut transform, mut velocity, locomotion, speed,
         silhouette_opt, domain_opt) in &mut units
    {
        match locomotion {
            LocomotionChannel::Moving(path) | LocomotionChannel::Reversing(path) => {
                if path.is_empty() {
                    velocity.0 = Vec3::ZERO;
                    continue;
                }
                let target = path[0];
                let current_pos = transform.translation;
                let direction_3d = target - current_pos;
                let direction_2d = Vec3::new(direction_3d.x, 0.0, direction_3d.z);
                let distance = direction_2d.length();

                if distance < 0.1 {
                    transform.translation.x = target.x;
                    transform.translation.z = target.z;
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
                let is_air = domain_opt.map_or(false, |d| *d == DomainEnum::Air);
                transform.translation.y = if is_air { 1.5 } else { 0.5 };
            }
            LocomotionChannel::Stopping => {
                let current_speed = velocity.0.length();
                if current_speed < 0.01 {
                    velocity.0 = Vec3::ZERO;
                    continue;
                }
                let new_speed = (current_speed - 8.0 * delta).max(0.0);
                velocity.0 = velocity.0.normalize_or_zero() * new_speed;

                let proposed_pos = transform.translation + velocity.0 * delta;

                let is_ground = domain_opt.map_or(true, |d| *d == DomainEnum::Ground);
                if is_ground {
                    if let Some(sil) = silhouette_opt {
                        if occupancy.check_movement_collision(entity, proposed_pos.x, proposed_pos.z, sil.width / 2.0, sil.height / 2.0) {
                            velocity.0 = Vec3::ZERO;
                            continue;
                        }
                    }
                }

                transform.translation = proposed_pos;
                let is_air = domain_opt.map_or(false, |d| *d == DomainEnum::Air);
                transform.translation.y = if is_air { 1.5 } else { 0.5 };
            }
            LocomotionChannel::Stationary => {
                // No channel-driven movement
            }
        }
    }
}

/// Channel-driven orientation consumer system.
/// Reads OrientationChannel and rotates the unit base toward the target.
pub fn channel_orientation_system(
    time: Res<Time>,
    mut units: Query<
        (&mut Transform, &OrientationChannel,
         Option<&TurnRateMovementParams>, Option<&RotationSpeed>),
        (With<Unit>, Without<MoveTarget>)
    >,
) {
    let delta = time.delta_secs();
    if delta < 0.0001 {
        return;
    }

    for (mut transform, orientation, turn_params_opt, rot_speed_opt) in &mut units {
        match orientation {
            OrientationChannel::Turning(target) => {
                let current_pos = transform.translation;
                let to_target = Vec3::new(target.x - current_pos.x, 0.0, target.z - current_pos.z);

                if to_target.length_squared() < 0.001 {
                    continue;
                }

                let desired_dir = to_target.normalize();

                if let Some(params) = turn_params_opt {
                    let current_forward = transform.forward();
                    let current_facing = Vec3::new(current_forward.x, 0.0, current_forward.z).normalize_or_zero();
                    let dot = current_facing.dot(desired_dir).clamp(-1.0, 1.0);
                    let angle_to_target = dot.acos();

                    if angle_to_target > 0.001 {
                        let max_turn = params.turn_rate * delta;
                        let cross = current_facing.cross(desired_dir);
                        let turn_sign = if cross.y >= 0.0 { 1.0 } else { -1.0 };
                        let actual_turn = angle_to_target.min(max_turn);
                        let turn_quat = Quat::from_rotation_y(turn_sign * actual_turn);
                        transform.rotation = turn_quat * transform.rotation;
                    }
                } else if let Some(rot_speed) = rot_speed_opt {
                    let target_rotation = Quat::from_rotation_y(
                        desired_dir.x.atan2(desired_dir.z)
                    );
                    let rotation_speed_factor = rot_speed.0 * delta;
                    transform.rotation = transform.rotation.slerp(target_rotation, rotation_speed_factor.min(1.0));
                } else {
                    let target_rotation = Quat::from_rotation_y(
                        desired_dir.x.atan2(desired_dir.z)
                    );
                    transform.rotation = target_rotation;
                }
            }
            OrientationChannel::Maintaining => {
                // Hold current facing
            }
        }
    }
}

/// Crushing system: TrackedVehicle and Mech units crush enemy LightInfantry on contact.
/// Runs after all movement systems so positions are final for the frame.
pub fn crushing_system(
    crushers: Query<
        (&Transform, &UnitBaseEnum, &Owner, &Silhouette),
        (With<Unit>, Without<InTunnelNetwork>),
    >,
    mut crushables: Query<
        (Entity, &Transform, &UnitBaseEnum, &Owner, &Silhouette, &mut ObjectInstance),
        (With<Unit>, Without<InTunnelNetwork>),
    >,
) {
    for (c_transform, c_base, c_owner, c_sil) in crushers.iter() {
        if !c_base.data().can_crush {
            continue;
        }
        for (_entity, t_transform, t_base, t_owner, t_sil, mut obj) in crushables.iter_mut() {
            if !t_base.data().crushable {
                continue;
            }
            if c_owner == t_owner {
                continue;
            }
            if !obj.is_alive() {
                continue;
            }
            let cx = c_transform.translation.x;
            let cz = c_transform.translation.z;
            let tx = t_transform.translation.x;
            let tz = t_transform.translation.z;
            let overlap_x = (c_sil.width / 2.0 + t_sil.width / 2.0) - (cx - tx).abs();
            let overlap_z = (c_sil.height / 2.0 + t_sil.height / 2.0) - (cz - tz).abs();
            if overlap_x > 0.0 && overlap_z > 0.0 {
                obj.apply_damage(f32::MAX);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    fn test_turnrate_params() -> TurnRateMovementParams {
        TurnRateMovementParams {
            turn_rate: 10.0,
            acceleration: 100.0,
            deceleration: 100.0,
            max_speed: 5.0,
        }
    }

    fn spawn_channel_turnrate_unit(world: &mut World, pos: Vec3, facing_angle: f32) -> Entity {
        world.spawn((
            Unit,
            Transform::from_translation(pos)
                .with_rotation(Quat::from_rotation_y(facing_angle)),
            Velocity(Vec3::ZERO),
            test_turnrate_params(),
            LocomotionChannel::default(),
            OrientationChannel::default(),
            GridPosition { x: 32, z: 32 },
        )).id()
    }

    #[test]
    fn channel_turnrate_moving_sets_velocity() {
        let mut world = World::new();
        world.insert_resource(OccupancyMap::default());
        world.insert_resource(Time::<()>::default());

        let entity = spawn_channel_turnrate_unit(&mut world, Vec3::new(0.0, 0.5, 0.0), 0.0);
        world.entity_mut(entity).insert(LocomotionChannel::Moving(vec![Vec3::new(0.0, 0.0, -10.0)]));
        world.resource_mut::<Time<()>>().advance_by(std::time::Duration::from_millis(16));
        world.run_system_once(channel_turnrate_locomotion_system).unwrap();

        let velocity = world.entity(entity).get::<Velocity>().unwrap();
        assert!(velocity.0.length() > 0.0, "Unit should have non-zero velocity when Moving");
        assert!(velocity.0.z < 0.0, "Velocity z should be negative");
    }

    #[test]
    fn channel_turnrate_stationary_no_movement() {
        let mut world = World::new();
        world.insert_resource(OccupancyMap::default());
        world.insert_resource(Time::<()>::default());

        let entity = spawn_channel_turnrate_unit(&mut world, Vec3::new(0.0, 0.5, 0.0), 0.0);
        world.resource_mut::<Time<()>>().advance_by(std::time::Duration::from_millis(16));
        world.run_system_once(channel_turnrate_locomotion_system).unwrap();

        let velocity = world.entity(entity).get::<Velocity>().unwrap();
        assert_eq!(velocity.0, Vec3::ZERO, "Stationary channel should not produce movement");
    }

    #[test]
    fn channel_turnrate_stopping_decelerates() {
        let mut world = World::new();
        world.insert_resource(OccupancyMap::default());
        world.insert_resource(Time::<()>::default());

        let entity = spawn_channel_turnrate_unit(&mut world, Vec3::new(0.0, 0.5, 0.0), 0.0);
        world.entity_mut(entity).insert((
            Velocity(Vec3::new(0.0, 0.0, 5.0)),
            LocomotionChannel::Stopping,
        ));
        world.resource_mut::<Time<()>>().advance_by(std::time::Duration::from_millis(16));
        world.run_system_once(channel_turnrate_locomotion_system).unwrap();

        let velocity = world.entity(entity).get::<Velocity>().unwrap();
        assert!(velocity.0.length() < 5.0, "Stopping should decelerate");
    }

    #[test]
    fn channel_turnrate_moving_empty_path_stops() {
        let mut world = World::new();
        world.insert_resource(OccupancyMap::default());
        world.insert_resource(Time::<()>::default());

        let entity = spawn_channel_turnrate_unit(&mut world, Vec3::new(0.0, 0.5, 0.0), 0.0);
        world.entity_mut(entity).insert((
            Velocity(Vec3::new(0.0, 0.0, 3.0)),
            LocomotionChannel::Moving(vec![]),
        ));
        world.resource_mut::<Time<()>>().advance_by(std::time::Duration::from_millis(16));
        world.run_system_once(channel_turnrate_locomotion_system).unwrap();

        let velocity = world.entity(entity).get::<Velocity>().unwrap();
        assert_eq!(velocity.0, Vec3::ZERO, "Empty path should zero velocity");
    }

    #[test]
    fn channel_turnrate_snap_when_close() {
        let mut world = World::new();
        world.insert_resource(OccupancyMap::default());
        world.insert_resource(Time::<()>::default());

        let target = Vec3::new(1.0, 0.0, 1.0);
        let entity = spawn_channel_turnrate_unit(&mut world, Vec3::new(1.05, 0.5, 1.02), 0.0);
        world.entity_mut(entity).insert(LocomotionChannel::Moving(vec![target]));
        world.resource_mut::<Time<()>>().advance_by(std::time::Duration::from_millis(16));
        world.run_system_once(channel_turnrate_locomotion_system).unwrap();

        let transform = world.entity(entity).get::<Transform>().unwrap();
        assert!((transform.translation.x - target.x).abs() < 0.001, "Should snap X to target");
        assert!((transform.translation.z - target.z).abs() < 0.001, "Should snap Z to target");
    }

    #[test]
    fn channel_turnrate_reversing_moves_backward() {
        let mut world = World::new();
        world.insert_resource(OccupancyMap::default());
        world.insert_resource(Time::<()>::default());

        let entity = spawn_channel_turnrate_unit(&mut world, Vec3::new(0.0, 0.5, 0.0), 0.0);
        world.entity_mut(entity).insert(LocomotionChannel::Reversing(vec![Vec3::new(0.0, 0.0, 5.0)]));
        world.resource_mut::<Time<()>>().advance_by(std::time::Duration::from_millis(16));
        world.run_system_once(channel_turnrate_locomotion_system).unwrap();

        let velocity = world.entity(entity).get::<Velocity>().unwrap();
        assert!(velocity.0.length() > 0.0, "Should have velocity when reversing");
        assert!(velocity.0.z > 0.0, "Should move in +Z (reverse of -Z facing)");
    }

    #[test]
    fn channel_turnrate_skips_movetarget_entities() {
        let mut world = World::new();
        world.insert_resource(OccupancyMap::default());
        world.insert_resource(Time::<()>::default());

        let entity = spawn_channel_turnrate_unit(&mut world, Vec3::new(0.0, 0.5, 0.0), 0.0);
        world.entity_mut(entity).insert((
            LocomotionChannel::Moving(vec![Vec3::new(0.0, 0.0, 10.0)]),
            MoveTarget(Vec3::new(0.0, 0.0, 10.0)),
        ));
        world.resource_mut::<Time<()>>().advance_by(std::time::Duration::from_millis(16));
        world.run_system_once(channel_turnrate_locomotion_system).unwrap();

        let velocity = world.entity(entity).get::<Velocity>().unwrap();
        assert_eq!(velocity.0, Vec3::ZERO, "Entity with MoveTarget should not be processed by channel system");
    }

    #[test]
    fn channel_orientation_turns_toward_target() {
        let mut world = World::new();
        world.insert_resource(Time::<()>::default());

        let entity = spawn_channel_turnrate_unit(&mut world, Vec3::new(0.0, 0.5, 0.0), 0.0);
        world.entity_mut(entity).insert(OrientationChannel::Turning(Vec3::new(10.0, 0.0, 0.0)));
        let initial_rotation = world.entity(entity).get::<Transform>().unwrap().rotation;
        world.resource_mut::<Time<()>>().advance_by(std::time::Duration::from_millis(16));
        world.run_system_once(channel_orientation_system).unwrap();

        let final_rotation = world.entity(entity).get::<Transform>().unwrap().rotation;
        assert_ne!(initial_rotation, final_rotation, "Unit should have rotated toward target");
    }

    #[test]
    fn channel_orientation_maintaining_no_rotation() {
        let mut world = World::new();
        world.insert_resource(Time::<()>::default());

        let entity = spawn_channel_turnrate_unit(&mut world, Vec3::new(0.0, 0.5, 0.0), 0.0);
        let initial_rotation = world.entity(entity).get::<Transform>().unwrap().rotation;
        world.resource_mut::<Time<()>>().advance_by(std::time::Duration::from_millis(16));
        world.run_system_once(channel_orientation_system).unwrap();

        let final_rotation = world.entity(entity).get::<Transform>().unwrap().rotation;
        assert_eq!(initial_rotation, final_rotation, "Maintaining should not change rotation");
    }

    #[test]
    fn channel_orientation_skips_movetarget_entities() {
        let mut world = World::new();
        world.insert_resource(Time::<()>::default());

        let entity = spawn_channel_turnrate_unit(&mut world, Vec3::new(0.0, 0.5, 0.0), 0.0);
        world.entity_mut(entity).insert((
            OrientationChannel::Turning(Vec3::new(10.0, 0.0, 0.0)),
            MoveTarget(Vec3::new(10.0, 0.0, 0.0)),
        ));
        let initial_rotation = world.entity(entity).get::<Transform>().unwrap().rotation;
        world.resource_mut::<Time<()>>().advance_by(std::time::Duration::from_millis(16));
        world.run_system_once(channel_orientation_system).unwrap();

        let final_rotation = world.entity(entity).get::<Transform>().unwrap().rotation;
        assert_eq!(initial_rotation, final_rotation, "Entity with MoveTarget should be skipped");
    }

    #[test]
    fn channel_fallback_locomotion_moving() {
        let mut world = World::new();
        world.insert_resource(OccupancyMap::default());
        world.insert_resource(Time::<()>::default());

        let entity = world.spawn((
            Unit,
            Transform::from_xyz(0.0, 0.5, 0.0),
            Velocity(Vec3::ZERO),
            MovementSpeed(5.0),
            LocomotionChannel::Moving(vec![Vec3::new(5.0, 0.0, 5.0)]),
            GridPosition { x: 32, z: 32 },
        )).id();
        world.resource_mut::<Time<()>>().advance_by(std::time::Duration::from_millis(16));
        world.run_system_once(channel_fallback_locomotion_system).unwrap();

        let velocity = world.entity(entity).get::<Velocity>().unwrap();
        assert!(velocity.0.length() > 0.0, "Non-TurnRate unit should move via fallback system");
    }

    // === Crushing system tests ===

    fn spawn_crusher(world: &mut World, base: UnitBaseEnum, pos: Vec3, owner: u8) -> Entity {
        world.spawn((
            Unit,
            Transform::from_translation(pos),
            base,
            Owner(Some(owner)),
            Silhouette { width: 1.0, height: 1.0 },
            ObjectInstance::destructible(ObjectEnum::Peacekeeper, 200.0),
        )).id()
    }

    fn spawn_crushable(world: &mut World, base: UnitBaseEnum, pos: Vec3, owner: u8) -> Entity {
        world.spawn((
            Unit,
            Transform::from_translation(pos),
            base,
            Owner(Some(owner)),
            Silhouette { width: 1.0, height: 1.0 },
            ObjectInstance::destructible(ObjectEnum::Peacekeeper, 100.0),
        )).id()
    }

    #[test]
    fn crushing_tracked_vehicle_kills_enemy_light_infantry() {
        let mut world = World::new();
        let _crusher = spawn_crusher(&mut world, UnitBaseEnum::TrackedVehicle, Vec3::new(0.0, 0.5, 0.0), 0);
        let victim = spawn_crushable(&mut world, UnitBaseEnum::LightInfantry, Vec3::new(0.3, 0.5, 0.3), 1);
        world.run_system_once(crushing_system).unwrap();
        assert!(!world.entity(victim).get::<ObjectInstance>().unwrap().is_alive(), "LightInfantry should be crushed");
    }

    #[test]
    fn crushing_mech_kills_enemy_light_infantry() {
        let mut world = World::new();
        let _crusher = spawn_crusher(&mut world, UnitBaseEnum::Mech, Vec3::new(0.0, 0.5, 0.0), 0);
        let victim = spawn_crushable(&mut world, UnitBaseEnum::LightInfantry, Vec3::new(0.3, 0.5, 0.3), 1);
        world.run_system_once(crushing_system).unwrap();
        assert!(!world.entity(victim).get::<ObjectInstance>().unwrap().is_alive(), "LightInfantry should be crushed by Mech");
    }

    #[test]
    fn crushing_no_friendly_fire() {
        let mut world = World::new();
        let _crusher = spawn_crusher(&mut world, UnitBaseEnum::TrackedVehicle, Vec3::new(0.0, 0.5, 0.0), 0);
        let friendly = spawn_crushable(&mut world, UnitBaseEnum::LightInfantry, Vec3::new(0.3, 0.5, 0.3), 0);
        world.run_system_once(crushing_system).unwrap();
        assert!(world.entity(friendly).get::<ObjectInstance>().unwrap().is_alive(), "Friendly should not be crushed");
    }

    #[test]
    fn crushing_heavy_infantry_survives() {
        let mut world = World::new();
        let _crusher = spawn_crusher(&mut world, UnitBaseEnum::TrackedVehicle, Vec3::new(0.0, 0.5, 0.0), 0);
        let heavy = spawn_crushable(&mut world, UnitBaseEnum::HeavyInfantry, Vec3::new(0.3, 0.5, 0.3), 1);
        world.run_system_once(crushing_system).unwrap();
        assert!(world.entity(heavy).get::<ObjectInstance>().unwrap().is_alive(), "HeavyInfantry should not be crushable");
    }

    #[test]
    fn crushing_wheeled_vehicle_cannot_crush() {
        let mut world = World::new();
        let _non_crusher = spawn_crusher(&mut world, UnitBaseEnum::WheeledVehicle, Vec3::new(0.0, 0.5, 0.0), 0);
        let victim = spawn_crushable(&mut world, UnitBaseEnum::LightInfantry, Vec3::new(0.3, 0.5, 0.3), 1);
        world.run_system_once(crushing_system).unwrap();
        assert!(world.entity(victim).get::<ObjectInstance>().unwrap().is_alive(), "WheeledVehicle should not crush");
    }

    #[test]
    fn crushing_no_overlap_no_crush() {
        let mut world = World::new();
        let _crusher = spawn_crusher(&mut world, UnitBaseEnum::TrackedVehicle, Vec3::new(0.0, 0.5, 0.0), 0);
        let far_unit = spawn_crushable(&mut world, UnitBaseEnum::LightInfantry, Vec3::new(5.0, 0.5, 5.0), 1);
        world.run_system_once(crushing_system).unwrap();
        assert!(world.entity(far_unit).get::<ObjectInstance>().unwrap().is_alive(), "Distant unit should not be crushed");
    }

    #[test]
    fn crushing_in_tunnel_excluded() {
        let mut world = World::new();
        let crusher = spawn_crusher(&mut world, UnitBaseEnum::TrackedVehicle, Vec3::new(0.0, 0.5, 0.0), 0);
        world.entity_mut(crusher).insert(InTunnelNetwork { owner_player: 0 });
        let victim = spawn_crushable(&mut world, UnitBaseEnum::LightInfantry, Vec3::new(0.3, 0.5, 0.3), 1);
        world.run_system_once(crushing_system).unwrap();
        assert!(world.entity(victim).get::<ObjectInstance>().unwrap().is_alive(), "In-tunnel crusher should not crush");
    }
}
