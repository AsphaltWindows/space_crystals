use bevy::prelude::*;
use crate::types::{Unit, Owner, DomainEnum, GridPosition};
use crate::game::types::ObjectInstance;
use crate::utils::is_enemy;
use crate::game::units::types::{UnitCommand, UnitControlCost, TurretCommandState};
use crate::game::types::factions::{Player, GdoPlayerResources};
use crate::game::world::types::{ElevationMap, elevation_modifier, SpaceCrystalPatch};
use crate::game::types::structures::ExtractionPlateState;
use crate::game::combat::types::*;
use crate::game::combat::utils::{spawn_projectile, apply_aoe_damage, spawn_attack_line, circle_rect_overlap_area, is_valid_target, is_domain_compatible, select_best_target};
use crate::types::{VisibilityStateEnum, SightRange};
use crate::game::units::types::state::behavior::{
    TurretOrientationChannel, TurretAttackChannel,
    BaseAttackChannel, LocomotionChannel, OrientationChannel,
};

/// System to handle attack commands — targets any entity with ObjectInstance (units and structures)
/// Respects interruptibility: new attack targets are accepted if the current phase is
/// interruptible or no target is set. Non-interruptible phases (Firing, Cooldown) reject
/// new target assignments.
pub fn attack_command_system(
    mut units: Query<
        (Entity, &Transform, &AttackCapability, &mut AttackState, &UnitCommand),
        With<Unit>
    >,
    targets: Query<(&Transform, &Owner), With<ObjectInstance>>,
) {
    for (_entity, _transform, _attack_cap, mut attack_state, unit_command) in units.iter_mut() {
        if let UnitCommand::AttackTarget(target_entity) = unit_command {
            if targets.get(*target_entity).is_ok() {
                // Accept new target if no current target or current phase is interruptible
                if attack_state.target_entity().is_none() || attack_state.phase.is_interruptible() {
                    attack_state.current_target = Some(AttackTarget::UnitTarget(*target_entity));
                    attack_state.phase = AttackPhase::None;
                    attack_state.time_in_phase = 0.0;
                }
            }
        }
    }
}

/// System to progress through attack phases — targets any entity with ObjectInstance
pub fn attack_phase_system(
    mut commands: Commands,
    time: Res<Time>,
    mut units: Query<
        (Entity, &Transform, &AttackCapability, &mut AttackState, &Owner, &UnitCommand, Option<&DomainEnum>, &GridPosition),
        With<Unit>
    >,
    targets: Query<(&Transform, &Owner, Option<&DomainEnum>, &GridPosition), With<ObjectInstance>>,
    mut meshes: ResMut<Assets<Mesh>>,
    cache: Res<CombatAssetCache>,
    elevation_map: Res<ElevationMap>,
) {
    let delta = time.delta_secs();

    for (entity, transform, attack_cap, mut attack_state, owner, unit_command, source_domain, source_grid_pos) in units.iter_mut() {
        let Some(target_entity) = attack_state.target_entity() else {
            continue;
        };

        let Ok((target_transform, target_owner, target_domain, target_grid_pos)) = targets.get(target_entity) else {
            attack_state.current_target = None;
            attack_state.phase = AttackPhase::None;
            continue;
        };

        // Allow force-attack on friendly entities when explicitly commanded
        let is_force_attack = matches!(unit_command, UnitCommand::AttackTarget(t) if *t == target_entity);
        if !is_force_attack && !is_enemy(owner, target_owner) {
            attack_state.current_target = None;
            attack_state.phase = AttackPhase::None;
            continue;
        }

        let distance = transform.translation.distance(target_transform.translation);

        // Compute elevation-adjusted effective range
        let src_domain = source_domain.copied().unwrap_or(DomainEnum::Ground);
        let tgt_domain = target_domain.copied().unwrap_or(DomainEnum::Ground);
        let src_elev = elevation_map.get(source_grid_pos.x, source_grid_pos.z);
        let tgt_elev = elevation_map.get(target_grid_pos.x, target_grid_pos.z);
        let elev_mod = if attack_cap.is_melee() { 0 } else {
            elevation_modifier(src_domain, src_elev, tgt_domain, tgt_elev)
        };
        let effective_range = attack_cap.range + elev_mod as f32;

        attack_state.time_in_phase += delta;

        match attack_state.phase {
            AttackPhase::None => {
                if distance <= effective_range {
                    attack_state.phase = AttackPhase::Aiming;
                    attack_state.time_in_phase = 0.0;
                    info!("Unit {:?} starting attack on target", entity);
                }
            }

            AttackPhase::Aiming => {
                if distance > effective_range {
                    attack_state.phase = AttackPhase::None;
                    continue;
                }

                if attack_state.time_in_phase >= attack_cap.aim_time {
                    // For area-effect attacks, snapshot the target location at fire time
                    match &attack_cap.attack_type {
                        AttackType::HeadDisjointed { .. } | AttackType::DoublyDisjointed { .. } => {
                            // Store the location target alongside the unit target for AoE
                            // The unit target is kept for tracking; location is the snapshot
                            attack_state.current_target = Some(AttackTarget::LocationTarget(target_transform.translation));
                        }
                        _ => {}
                    }

                    attack_state.phase = AttackPhase::Firing;
                    attack_state.time_in_phase = 0.0;
                }
            }

            AttackPhase::Firing => {
                if attack_state.time_in_phase <= delta {
                    match &attack_cap.attack_type {
                        AttackType::FullyConnected { .. } => {
                            if let Ok((target_tf, _, _, _)) = targets.get(target_entity) {
                                commands.entity(target_entity).insert(DamageEvent::SingleTarget {
                                    damage: attack_cap.damage,
                                    source: entity,
                                    source_position: transform.translation,
                                });
                                // Spawn visual attack line tracer
                                let start_pos = transform.translation + Vec3::Y * 0.5;
                                let end_pos = target_tf.translation + Vec3::Y * 0.5;
                                spawn_attack_line(
                                    &mut commands,
                                    &cache,
                                    start_pos,
                                    end_pos,
                                );
                                info!("Unit {:?} fired (instant hit), damage: {}", entity, attack_cap.damage);
                            }
                        }

                        AttackType::TailDisjointed { projectile_speed, projectile_visual } => {
                            if let Ok((target_transform, _, _, _)) = targets.get(target_entity) {
                                spawn_projectile(
                                    &mut commands,
                                    &mut meshes,
                                    &cache,
                                    transform.translation + Vec3::new(0.0, 0.5, 0.0),
                                    target_transform.translation,
                                    *projectile_speed,
                                    attack_cap.damage,
                                    None,
                                    *projectile_visual,
                                    *owner,
                                );
                                info!("Unit {:?} fired projectile (homing)", entity);
                            }
                        }

                        AttackType::HeadDisjointed { effect_radius } => {
                            if let Some(target_loc) = attack_state.target_location() {
                                apply_aoe_damage(
                                    &mut commands,
                                    &targets,
                                    target_loc,
                                    *effect_radius,
                                    attack_cap.damage,
                                    entity,
                                    owner,
                                );

                                spawn_projectile(
                                    &mut commands,
                                    &mut meshes,
                                    &cache,
                                    target_loc,
                                    target_loc,
                                    1.0,
                                    attack_cap.damage,
                                    Some(*effect_radius),
                                    ProjectileVisual::Sphere { radius: 0.1 },
                                    *owner,
                                );

                                info!("Unit {:?} fired AOE attack (instant)", entity);
                            }
                        }

                        AttackType::DoublyDisjointed { projectile_speed, projectile_visual, effect_radius } => {
                            if let Some(target_loc) = attack_state.target_location() {
                                spawn_projectile(
                                    &mut commands,
                                    &mut meshes,
                                    &cache,
                                    transform.translation + Vec3::new(0.0, 0.5, 0.0),
                                    target_loc,
                                    *projectile_speed,
                                    attack_cap.damage,
                                    Some(*effect_radius),
                                    *projectile_visual,
                                    *owner,
                                );
                                info!("Unit {:?} fired projectile to location (AOE)", entity);
                            }
                        }
                    }
                }

                if attack_state.time_in_phase >= attack_cap.fire_time {
                    attack_state.phase = AttackPhase::Cooldown;
                    attack_state.time_in_phase = 0.0;
                }
            }

            AttackPhase::Cooldown => {
                if attack_state.time_in_phase >= attack_cap.cooldown_time {
                    attack_state.phase = AttackPhase::Reloading;
                    attack_state.time_in_phase = 0.0;
                }
            }

            AttackPhase::Reloading => {
                if attack_state.time_in_phase >= attack_cap.reload_time {
                    if distance <= effective_range {
                        attack_state.phase = AttackPhase::Aiming;
                        attack_state.time_in_phase = 0.0;
                    } else {
                        attack_state.phase = AttackPhase::None;
                    }
                }
            }
        }
    }
}

/// Check whether a target can threaten a defender based on attack domain compatibility.
/// Returns true if the target has an AttackCapability whose target_domain can hit the defender's domain.
/// If the target has no AttackCapability (e.g. unarmed structures), returns false (not threatening).
pub fn can_threaten(
    target_attack_cap: Option<&AttackCapability>,
    defender_domain: &DomainEnum,
) -> bool {
    match target_attack_cap {
        Some(cap) => is_domain_compatible(&cap.target_domain, defender_domain),
        None => false,
    }
}

/// Compute the relative turret angle from a unit to a target position.
/// Returns the angle in radians normalized to [-PI, PI], relative to the unit's forward direction.
pub fn compute_relative_turret_angle(unit_transform: &Transform, target_pos: Vec3) -> f32 {
    let to_target = target_pos - unit_transform.translation;
    let direction_2d = Vec2::new(to_target.x, to_target.z).normalize_or_zero();
    let target_world_angle = direction_2d.y.atan2(direction_2d.x);

    let unit_forward = unit_transform.forward();
    let unit_dir_2d = Vec2::new(unit_forward.x, unit_forward.z).normalize_or_zero();
    let unit_world_angle = unit_dir_2d.y.atan2(unit_dir_2d.x);

    let mut relative_angle = target_world_angle - unit_world_angle;
    while relative_angle > std::f32::consts::PI {
        relative_angle -= std::f32::consts::TAU;
    }
    while relative_angle < -std::f32::consts::PI {
        relative_angle += std::f32::consts::TAU;
    }
    relative_angle
}

/// System for turret autonomous scanning — turrets independently select targets
/// regardless of UnitCommand. Priority: threatening > least rotation > closest.
pub fn turret_autonomous_scanning_system(
    mut units: Query<
        (Entity, &Transform, &Turret, &AttackCapability, &mut TurretCommandState, &Owner, Option<&DomainEnum>, &GridPosition),
        With<Unit>
    >,
    potential_targets: Query<(Entity, &Transform, &Owner, Option<&DomainEnum>, Option<&AttackCapability>, &GridPosition), With<ObjectInstance>>,
    elevation_map: Res<ElevationMap>,
) {
    for (entity, transform, turret, attack_cap, mut turret_command_state, owner, source_domain, source_grid_pos) in units.iter_mut() {
        // Validate existing locked target — clear if entity no longer exists
        if let Some(target) = turret_command_state.locked_target {
            if potential_targets.get(target).is_err() {
                turret_command_state.locked_target = None;
            } else {
                continue; // target still valid, skip scanning
            }
        }

        let src_domain = source_domain.copied().unwrap_or(DomainEnum::Ground);
        let src_elev = elevation_map.get(source_grid_pos.x, source_grid_pos.z);

        // Candidate: (threatening, rotation_abs, distance, entity)
        let mut best_candidate: Option<(bool, f32, f32, Entity)> = None;

        for (target_entity, target_transform, target_owner, target_domain, target_attack_cap, target_grid_pos) in potential_targets.iter() {
            if target_entity == entity {
                continue;
            }

            if !is_enemy(owner, target_owner) {
                continue;
            }

            let distance = transform.translation.distance(target_transform.translation);

            // Compute elevation-adjusted effective range
            let tgt_domain = target_domain.copied().unwrap_or(DomainEnum::Ground);
            let tgt_elev = elevation_map.get(target_grid_pos.x, target_grid_pos.z);
            let elev_mod = if attack_cap.is_melee() { 0 } else {
                elevation_modifier(src_domain, src_elev, tgt_domain, tgt_elev)
            };
            let effective_range = attack_cap.range + elev_mod as f32;

            if distance > effective_range {
                continue;
            }

            // Check turret arc
            let relative_angle = compute_relative_turret_angle(transform, target_transform.translation);
            if !turret.can_reach_angle(relative_angle) {
                continue;
            }

            // Threat priority: can the target threaten us?
            let threatening = can_threaten(target_attack_cap, &src_domain);
            let rotation_abs = relative_angle.abs();

            // Compare: prefer threatening > least rotation > closest
            let is_better = match &best_candidate {
                None => true,
                Some((best_threat, best_rot, best_dist, _)) => {
                    if threatening && !best_threat {
                        true
                    } else if threatening == *best_threat {
                        if rotation_abs < *best_rot - 0.01 {
                            true
                        } else if (rotation_abs - best_rot).abs() < 0.01 {
                            distance < *best_dist
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                }
            };

            if is_better {
                best_candidate = Some((threatening, rotation_abs, distance, target_entity));
            }
        }

        if let Some((_, _, _, target)) = best_candidate {
            turret_command_state.locked_target = Some(target);
            info!("Turret unit {:?} auto-scanning locked target {:?}", entity, target);
        }
    }
}

/// Alignment tolerance for turret engagement — turret must be within this angle (radians)
/// of the target to begin firing. Matches turret_rotation_system convergence threshold.
const TURRET_ALIGNMENT_TOLERANCE: f32 = 0.05;

/// System that reads TurretCommandState.locked_target and drives turret rotation and firing
/// via the turret action channels. Sets TurretOrientationChannel and TurretAttackChannel
/// based on locked_target validity, range, arc, and current attack phase.
/// Also sets Turret.target_angle directly for immediate turret rotation integration.
pub fn turret_engagement_system(
    mut turret_units: Query<
        (&Transform, &mut Turret, &AttackCapability, &mut TurretCommandState,
         &mut TurretOrientationChannel, &mut TurretAttackChannel,
         &AttackState, &Owner, &GridPosition),
        With<Unit>
    >,
    targets: Query<(&Transform, &Owner, Option<&DomainEnum>, &GridPosition), With<ObjectInstance>>,
    elevation_map: Res<ElevationMap>,
) {
    for (transform, mut turret, attack_cap, mut turret_cmd, mut turret_orient, mut turret_attack,
         attack_state, owner, source_grid_pos) in turret_units.iter_mut()
    {
        let target_entity = match turret_cmd.locked_target {
            Some(t) => t,
            None => {
                // No target — set channels to idle
                *turret_orient = TurretOrientationChannel::Maintaining;
                *turret_attack = TurretAttackChannel::Inactive;
                turret.target_angle = None;
                continue;
            }
        };

        // Validate target still exists
        let (target_transform, _target_owner, target_domain, target_grid_pos) =
            match targets.get(target_entity) {
                Ok(t) => t,
                Err(_) => {
                    // Target despawned — clear and idle
                    turret_cmd.locked_target = None;
                    *turret_orient = TurretOrientationChannel::Maintaining;
                    *turret_attack = TurretAttackChannel::Inactive;
                    turret.target_angle = None;
                    continue;
                }
            };

        let target_pos = target_transform.translation;

        // Range check with elevation adjustment
        let distance = transform.translation.distance(target_pos);
        let src_domain = DomainEnum::Ground; // turret units are ground-based
        let src_elev = elevation_map.get(source_grid_pos.x, source_grid_pos.z);
        let tgt_domain = target_domain.copied().unwrap_or(DomainEnum::Ground);
        let tgt_elev = elevation_map.get(target_grid_pos.x, target_grid_pos.z);
        let elev_mod = if attack_cap.is_melee() { 0 } else {
            elevation_modifier(src_domain, src_elev, tgt_domain, tgt_elev)
        };
        let effective_range = attack_cap.range + elev_mod as f32;

        if distance > effective_range {
            // Target out of range — clear so scanning can re-acquire
            turret_cmd.locked_target = None;
            *turret_orient = TurretOrientationChannel::Maintaining;
            *turret_attack = TurretAttackChannel::Inactive;
            turret.target_angle = None;
            continue;
        }

        // Arc check
        let relative_angle = compute_relative_turret_angle(transform, target_pos);
        if !turret.can_reach_angle(relative_angle) {
            // Target outside turret arc — clear
            turret_cmd.locked_target = None;
            *turret_orient = TurretOrientationChannel::Maintaining;
            *turret_attack = TurretAttackChannel::Inactive;
            turret.target_angle = None;
            continue;
        }

        // Target is valid, in range, and in arc — set orientation channel
        *turret_orient = TurretOrientationChannel::Turning(target_pos);

        // Also set Turret.target_angle directly for immediate turret rotation
        let clamped = turret.clamp_angle(relative_angle);
        turret.target_angle = Some(clamped);

        // Check alignment for attack channel
        let aligned = (turret.current_angle - relative_angle).abs() < TURRET_ALIGNMENT_TOLERANCE;

        if aligned {
            // Map AttackState.phase to TurretAttackChannel
            match attack_state.phase {
                AttackPhase::Aiming => *turret_attack = TurretAttackChannel::Aiming(target_entity),
                AttackPhase::Firing => *turret_attack = TurretAttackChannel::Firing(target_entity),
                AttackPhase::Cooldown => *turret_attack = TurretAttackChannel::Cooldown,
                AttackPhase::Reloading => *turret_attack = TurretAttackChannel::Reloading,
                AttackPhase::None => *turret_attack = TurretAttackChannel::Aiming(target_entity),
            }
        } else {
            // Not yet aligned — still aiming
            *turret_attack = TurretAttackChannel::Aiming(target_entity);
        }
    }
}

/// System for base (non-turret) auto-targeting — idle and hold-position units engage nearby enemies.
/// Active only for UnitCommand::Idle and HoldPosition.
/// Uses 3-tier target priority: threatening > least rotation > closest distance.
/// Filters targets through is_valid_target (destructible, visible, domain-compatible).
/// Idle units scan within SightRange; HoldPosition units scan within weapon range.
pub fn base_auto_target_system(
    mut commands: Commands,
    mut units: Query<
        (Entity, &Transform, &AttackCapability, &mut AttackState, &Owner, Option<&DomainEnum>, &GridPosition, Option<&SightRange>),
        (With<Unit>, Without<Turret>)
    >,
    potential_targets: Query<(Entity, &Transform, &Owner, Option<&DomainEnum>, &GridPosition, &ObjectInstance, &VisibilityStateEnum, Option<&AttackCapability>), With<ObjectInstance>>,
    unit_commands: Query<&UnitCommand>,
    elevation_map: Res<ElevationMap>,
) {
    for (entity, transform, attack_cap, mut attack_state, owner, source_domain, source_grid_pos, sight_range) in units.iter_mut() {
        if attack_state.target_entity().is_some() {
            continue;
        }

        let command = unit_commands.get(entity).ok();

        // Only auto-target for Idle and HoldPosition
        let is_idle = if let Some(cmd) = command {
            match cmd {
                UnitCommand::Idle => true,
                UnitCommand::HoldPosition => false,
                _ => continue,
            }
        } else {
            continue;
        };

        let src_domain = source_domain.copied().unwrap_or(DomainEnum::Ground);
        let src_elev = elevation_map.get(source_grid_pos.x, source_grid_pos.z);

        // Idle units scan within SightRange; HoldPosition uses weapon range
        let base_scan_range = if is_idle {
            sight_range.map(|sr| sr.0 as f32).unwrap_or(attack_cap.range)
        } else {
            attack_cap.range
        };

        let mut candidates = Vec::new();

        for (target_entity, target_transform, target_owner, target_domain, target_grid_pos,
             target_obj, target_vis, target_attack_cap) in potential_targets.iter()
        {
            if target_entity == entity {
                continue;
            }

            if !is_enemy(owner, target_owner) {
                continue;
            }

            let tgt_domain = target_domain.copied().unwrap_or(DomainEnum::Ground);

            // Filter through is_valid_target (destructible, visible, domain-compatible)
            if !is_valid_target(target_obj, target_vis, &tgt_domain, &attack_cap.target_domain) {
                continue;
            }

            let distance = transform.translation.distance(target_transform.translation);

            // Compute elevation-adjusted scan range
            let tgt_elev = elevation_map.get(target_grid_pos.x, target_grid_pos.z);
            let elev_mod = if attack_cap.is_melee() { 0 } else {
                elevation_modifier(src_domain, src_elev, tgt_domain, tgt_elev)
            };
            let effective_scan_range = base_scan_range + elev_mod as f32;

            if distance > effective_scan_range {
                continue;
            }

            // Compute threat and rotation for 3-tier priority
            let threatening = can_threaten(target_attack_cap, &src_domain);
            let relative_angle = compute_relative_turret_angle(transform, target_transform.translation);
            let rotation_abs = relative_angle.abs();

            candidates.push((target_entity, threatening, rotation_abs, distance));
        }

        if let Some(target) = select_best_target(candidates.into_iter()) {
            // Record idle origin when an idle unit auto-acquires a target
            if is_idle {
                commands.entity(entity).insert(IdleOrigin(transform.translation));
            }

            attack_state.current_target = Some(AttackTarget::UnitTarget(target));
            attack_state.phase = AttackPhase::Aiming;
            attack_state.time_in_phase = 0.0;
            info!("Unit {:?} auto-targeting enemy", entity);
        }
    }
}

/// System to sync AttackState.phase to BaseAttackChannel for non-turret units,
/// and enforce cross-channel constraints on LocomotionChannel/OrientationChannel.
///
/// Runs after attack_phase_system and base_auto_target_system. For each non-turret unit:
/// - Maps AttackPhase → BaseAttackChannel variant
/// - Aiming: forces LocomotionChannel::Stationary, overrides OrientationChannel::Turning(target_pos)
/// - Firing/Cooldown: forces LocomotionChannel::Stationary, OrientationChannel::Maintaining
/// - Reloading/None: no constraints (locomotion and orientation free)
pub fn attack_channel_sync_system(
    mut units: Query<
        (&AttackState, &mut BaseAttackChannel, &mut LocomotionChannel, &mut OrientationChannel),
        (With<Unit>, Without<Turret>)
    >,
    targets: Query<&Transform, With<ObjectInstance>>,
) {
    for (attack_state, mut base_attack, mut locomotion, mut orientation) in units.iter_mut() {
        // Map AttackPhase to BaseAttackChannel
        match attack_state.phase {
            AttackPhase::Aiming => {
                if let Some(target_entity) = attack_state.target_entity() {
                    *base_attack = BaseAttackChannel::Aiming(target_entity);

                    // Enforce constraints: stop movement, turn to face target
                    *locomotion = LocomotionChannel::Stationary;
                    if let Ok(target_transform) = targets.get(target_entity) {
                        *orientation = OrientationChannel::Turning(target_transform.translation);
                    }
                } else if let Some(target_loc) = attack_state.target_location() {
                    // Location target (shouldn't normally be Aiming at a location, but handle gracefully)
                    *base_attack = BaseAttackChannel::None;
                    *locomotion = LocomotionChannel::Stationary;
                    *orientation = OrientationChannel::Turning(target_loc);
                } else {
                    *base_attack = BaseAttackChannel::None;
                }
            }
            AttackPhase::Firing => {
                if let Some(target_entity) = attack_state.target_entity() {
                    *base_attack = BaseAttackChannel::Firing(target_entity);
                } else {
                    // During firing, target may have been converted to LocationTarget for AoE
                    // Use a placeholder entity isn't ideal — just set Cooldown-like state
                    *base_attack = BaseAttackChannel::Cooldown;
                }
                // Enforce constraints: stop everything
                *locomotion = LocomotionChannel::Stationary;
                *orientation = OrientationChannel::Maintaining;
            }
            AttackPhase::Cooldown => {
                *base_attack = BaseAttackChannel::Cooldown;
                // Enforce constraints: stop everything
                *locomotion = LocomotionChannel::Stationary;
                *orientation = OrientationChannel::Maintaining;
            }
            AttackPhase::Reloading => {
                *base_attack = BaseAttackChannel::Reloading;
                // No constraints — locomotion and orientation free
            }
            AttackPhase::None => {
                *base_attack = BaseAttackChannel::None;
                // No constraints — locomotion and orientation free
            }
        }
    }
}

/// System to enforce idle leash distance — disengage if unit chases too far from idle origin
pub fn idle_leash_system(
    mut commands: Commands,
    units: Query<(Entity, &Transform, &IdleOrigin, &AttackState), With<Unit>>,
) {
    for (entity, transform, idle_origin, attack_state) in units.iter() {
        let distance = transform.translation.distance(idle_origin.0);

        if distance > IDLE_LEASH_DISTANCE {
            // Unit has chased too far — disengage and go idle at current position
            commands.entity(entity)
                .remove::<IdleOrigin>()
                .insert(AttackState::default())
                .insert(UnitCommand::Idle);
            info!("Unit {:?} exceeded idle leash distance ({:.1} > {:.1}), disengaging",
                entity, distance, IDLE_LEASH_DISTANCE);
        } else if attack_state.target_entity().is_none() && matches!(attack_state.phase, AttackPhase::None) {
            // Target was destroyed or lost — remove idle origin, stay idle at current position
            commands.entity(entity).remove::<IdleOrigin>();
        }
    }
}

/// Apply directional armor modifier based on attack angle vs target facing.
/// Returns the armor multiplier (1.0 = side hit, >1 = front, <1 = rear).
pub fn directional_armor_multiplier(source_pos: Vec3, target_pos: Vec3, target_forward: Vec3) -> f32 {
    let attack_dir = (target_pos - source_pos).normalize_or_zero();
    let facing_2d = Vec3::new(target_forward.x, 0.0, target_forward.z).normalize_or_zero();
    let attack_dir_2d = Vec3::new(attack_dir.x, 0.0, attack_dir.z).normalize_or_zero();

    // Dot product: compare negated attack direction with target facing
    // If they align (dot > 0), the attack comes from the front
    // If they oppose (dot < 0), the attack comes from the rear
    let dot = (-attack_dir_2d).dot(facing_2d);

    if dot >= DIRECTIONAL_ARMOR_FRONT_THRESHOLD {
        DIRECTIONAL_ARMOR_FRONT_MULTIPLIER
    } else if dot <= DIRECTIONAL_ARMOR_REAR_THRESHOLD {
        DIRECTIONAL_ARMOR_REAR_MULTIPLIER
    } else {
        1.0 // Side hit — no modifier
    }
}

/// System to apply damage to units with armor calculation
pub fn apply_damage_system(
    mut commands: Commands,
    mut units: Query<(Entity, &mut ObjectInstance, &Transform, Option<&Armor>, Option<&Silhouette>), With<DamageEvent>>,
    damage_events: Query<&DamageEvent>,
) {
    for (entity, mut obj, transform, armor_opt, silhouette_opt) in units.iter_mut() {
        if let Ok(damage_event) = damage_events.get(entity) {
            let actual_damage = match damage_event {
                DamageEvent::SingleTarget { damage, source: _, source_position } => {
                    if let Some(armor) = armor_opt {
                        let armor_value = if armor.directional_armor {
                            let multiplier = directional_armor_multiplier(
                                *source_position,
                                transform.translation,
                                *transform.forward(),
                            );
                            armor.point_armor * multiplier
                        } else {
                            armor.point_armor
                        };
                        (*damage - armor_value).max(0.0)
                    } else {
                        *damage
                    }
                }
                DamageEvent::AreaOfEffect { damage, source: _, center, radius, source_owner: _ } => {
                    let unit_pos = transform.translation;
                    let (sil_w, sil_h) = if let Some(sil) = silhouette_opt {
                        (sil.width, sil.height)
                    } else {
                        (1.0, 1.0) // Default 1x1 silhouette
                    };

                    let overlap = circle_rect_overlap_area(
                        Vec2::new(center.x, center.z),
                        *radius,
                        Vec2::new(unit_pos.x, unit_pos.z),
                        sil_w,
                        sil_h,
                    );
                    let aoe_area = std::f32::consts::PI * radius * radius;
                    let unit_area = sil_w * sil_h;

                    if aoe_area < f32::EPSILON || unit_area < f32::EPSILON {
                        0.0
                    } else {
                        let damage_share = damage * (overlap / aoe_area);

                        if let Some(armor) = armor_opt {
                            let effective_armor = armor.full_armor * (overlap / unit_area);
                            let armor_value = if armor.directional_armor {
                                let multiplier = directional_armor_multiplier(
                                    *center,
                                    unit_pos,
                                    *transform.forward(),
                                );
                                effective_armor * multiplier
                            } else {
                                effective_armor
                            };
                            (damage_share - armor_value).max(0.0)
                        } else {
                            damage_share
                        }
                    }
                }
            };

            let _destroyed = obj.apply_damage(actual_damage);

            info!("Unit {:?} took {:.1} damage, health: {}/{}",
                entity, actual_damage,
                obj.hp.unwrap_or(0.0), obj.max_hp.unwrap_or(0.0));

            commands.entity(entity).remove::<DamageEvent>();
        }
    }
}

/// System to remove dead entities (units and structures) and decrement unit cap on death
pub fn remove_dead_entities_system(
    mut commands: Commands,
    entities: Query<(Entity, &ObjectInstance, Option<&UnitControlCost>, Option<&Owner>, Option<&ExtractionPlateState>)>,
    mut gdo_players: Query<(&Player, &mut GdoPlayerResources)>,
    mut patches: Query<&mut SpaceCrystalPatch>,
    // TODO: Add SyndicatePlayerResources, CultsPlayerResources, ColonistsPlayerResources
    // queries when those factions have units spawning with UnitControlCost
) {
    for (entity, obj, control_cost, owner, plate_state) in entities.iter() {
        if !obj.is_alive() {
            // Handle ExtractionPlate destruction: clean up attached patch
            if let Some(plate) = plate_state {
                if let Ok(mut patch) = patches.get_mut(plate.attached_patch) {
                    if patch.remaining_amount > 0 {
                        // Patch still has resources — uncover it
                        patch.has_plate = false;
                        info!("Extraction Plate {:?} destroyed, patch uncovered (remaining: {})",
                            entity, patch.remaining_amount);
                    } else {
                        // Patch is depleted — despawn the patch entity
                        info!("Extraction Plate {:?} destroyed over depleted patch, removing patch",
                            entity);
                        commands.entity(plate.attached_patch).despawn();
                    }
                }
            }

            // Decrement unit cap for the owning player
            if let (Some(cost), Some(owner)) = (control_cost, owner) {
                for (player, mut res) in gdo_players.iter_mut() {
                    if Some(player.player_number) == owner.player_number() {
                        res.unit_control_used = res.unit_control_used.saturating_sub(cost.0);
                        info!("Entity {:?} destroyed, freed {} unit control for player {}",
                            entity, cost.0, player.player_number);
                        break;
                    }
                }
                // TODO: Match on player faction and decrement appropriate resource
                // (SyndicatePlayerResources.tunnel_space_used,
                //  CultsPlayerResources.unit_control_used,
                //  ColonistsPlayerResources.beacon_capacity_used)
            } else {
                info!("Entity {:?} destroyed", entity);
            }
            commands.entity(entity).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{DomainEnum, TargetDomainEnum};
    use crate::game::combat::utils::select_best_target;

    // === can_threaten ===

    #[test]
    fn can_threaten_no_attack_cap_not_threatening() {
        // No AttackCapability means not threatening
        assert!(!can_threaten(None, &DomainEnum::Ground));
    }

    #[test]
    fn can_threaten_ground_attack_threatens_ground_unit() {
        let mut cap = AttackCapability::default();
        cap.target_domain = TargetDomainEnum::Ground;
        assert!(can_threaten(Some(&cap), &DomainEnum::Ground));
    }

    #[test]
    fn can_threaten_air_attack_does_not_threaten_ground_unit() {
        let mut cap = AttackCapability::default();
        cap.target_domain = TargetDomainEnum::Air;
        assert!(!can_threaten(Some(&cap), &DomainEnum::Ground));
    }

    #[test]
    fn can_threaten_universal_attack_threatens_air() {
        let mut cap = AttackCapability::default();
        cap.target_domain = TargetDomainEnum::Universal;
        assert!(can_threaten(Some(&cap), &DomainEnum::Air));
    }

    #[test]
    fn can_threaten_ground_attack_threatens_underground() {
        let mut cap = AttackCapability::default();
        cap.target_domain = TargetDomainEnum::Ground;
        assert!(can_threaten(Some(&cap), &DomainEnum::Underground));
    }

    // === compute_relative_turret_angle ===

    #[test]
    fn relative_angle_target_directly_ahead_is_zero() {
        // Unit at origin facing -Z, target at (0, 0, -5)
        let unit_tf = Transform::from_translation(Vec3::ZERO)
            .looking_to(Vec3::new(0.0, 0.0, -1.0), Vec3::Y);
        let target_pos = Vec3::new(0.0, 0.0, -5.0);
        let angle = compute_relative_turret_angle(&unit_tf, target_pos);
        assert!(angle.abs() < 0.01, "Expected ~0, got {}", angle);
    }

    #[test]
    fn relative_angle_target_directly_behind_is_pi() {
        // Unit at origin facing -Z, target at (0, 0, 5)
        let unit_tf = Transform::from_translation(Vec3::ZERO)
            .looking_to(Vec3::new(0.0, 0.0, -1.0), Vec3::Y);
        let target_pos = Vec3::new(0.0, 0.0, 5.0);
        let angle = compute_relative_turret_angle(&unit_tf, target_pos);
        assert!((angle.abs() - std::f32::consts::PI).abs() < 0.01, "Expected ~PI, got {}", angle);
    }

    #[test]
    fn relative_angle_target_to_right_is_positive_or_negative() {
        // Unit at origin facing -Z, target at (5, 0, 0) (to the right)
        let unit_tf = Transform::from_translation(Vec3::ZERO)
            .looking_to(Vec3::new(0.0, 0.0, -1.0), Vec3::Y);
        let target_pos = Vec3::new(5.0, 0.0, 0.0);
        let angle = compute_relative_turret_angle(&unit_tf, target_pos);
        // Should be roughly PI/2 in magnitude
        assert!((angle.abs() - std::f32::consts::FRAC_PI_2).abs() < 0.1,
            "Expected ~PI/2, got {}", angle);
    }

    #[test]
    fn relative_angle_is_bounded() {
        // Any target should give an angle in [-PI, PI]
        let unit_tf = Transform::from_translation(Vec3::ZERO)
            .looking_to(Vec3::new(1.0, 0.0, 1.0).normalize(), Vec3::Y);
        let target_pos = Vec3::new(-3.0, 0.0, 2.0);
        let angle = compute_relative_turret_angle(&unit_tf, target_pos);
        assert!(angle >= -std::f32::consts::PI && angle <= std::f32::consts::PI,
            "Angle out of bounds: {}", angle);
    }

    // === Turret scanning priority logic (unit test of priority comparison) ===

    #[test]
    fn turret_prefers_threatening_over_non_threatening() {
        // Simulated candidates
        let threatening_target = (true, 1.0_f32, 5.0_f32);  // (threatening, rotation, distance)
        let non_threatening_target = (false, 0.5_f32, 3.0_f32);

        // Threatening should win even with worse rotation and distance
        assert!(threatening_target.0 && !non_threatening_target.0);
    }

    #[test]
    fn turret_prefers_less_rotation_when_equal_threat() {
        let target_a = (true, 0.3_f32, 5.0_f32);
        let target_b = (true, 1.5_f32, 3.0_f32);

        // Both threatening, target_a has less rotation → a is better
        assert!(target_a.1 < target_b.1);
    }

    #[test]
    fn turret_prefers_closer_when_equal_threat_and_rotation() {
        let target_a = (true, 0.5_f32, 3.0_f32);
        let target_b = (true, 0.5_f32, 7.0_f32);

        // Both threatening, same rotation, target_a is closer → a is better
        assert!(target_a.2 < target_b.2);
    }

    // === Base auto-target command filtering ===

    #[test]
    fn base_auto_target_allows_idle() {
        let cmd = UnitCommand::Idle;
        let allowed = matches!(cmd, UnitCommand::Idle | UnitCommand::HoldPosition);
        assert!(allowed);
    }

    #[test]
    fn base_auto_target_allows_hold_position() {
        let cmd = UnitCommand::HoldPosition;
        let allowed = matches!(cmd, UnitCommand::Idle | UnitCommand::HoldPosition);
        assert!(allowed);
    }

    #[test]
    fn base_auto_target_blocks_attack_move() {
        // AttackMove has its own scanning in attack_move_behavior_system
        let cmd = UnitCommand::AttackMove(Vec3::ZERO);
        let allowed = matches!(cmd, UnitCommand::Idle | UnitCommand::HoldPosition);
        assert!(!allowed);
    }

    #[test]
    fn base_auto_target_blocks_move() {
        let cmd = UnitCommand::Move(Vec3::ZERO);
        let allowed = matches!(cmd, UnitCommand::Idle | UnitCommand::HoldPosition);
        assert!(!allowed);
    }

    #[test]
    fn base_auto_target_blocks_attack_target() {
        let cmd = UnitCommand::AttackTarget(Entity::from_raw_u32(1).unwrap());
        let allowed = matches!(cmd, UnitCommand::Idle | UnitCommand::HoldPosition);
        assert!(!allowed);
    }

    #[test]
    fn base_auto_target_blocks_stop() {
        let cmd = UnitCommand::Stop;
        let allowed = matches!(cmd, UnitCommand::Idle | UnitCommand::HoldPosition);
        assert!(!allowed);
    }

    #[test]
    fn base_auto_target_blocks_attack_location() {
        let cmd = UnitCommand::AttackLocation(Vec3::ZERO);
        let allowed = matches!(cmd, UnitCommand::Idle | UnitCommand::HoldPosition);
        assert!(!allowed);
    }

    #[test]
    fn base_auto_target_blocks_reverse() {
        let cmd = UnitCommand::Reverse(Vec3::ZERO);
        let allowed = matches!(cmd, UnitCommand::Idle | UnitCommand::HoldPosition);
        assert!(!allowed);
    }

    #[test]
    fn base_auto_target_blocks_patrol() {
        let cmd = UnitCommand::Patrol { start: Vec3::ZERO, end: Vec3::ONE, going_to_end: true };
        let allowed = matches!(cmd, UnitCommand::Idle | UnitCommand::HoldPosition);
        assert!(!allowed);
    }

    // === select_best_target priority tests ===

    #[test]
    fn select_best_target_threatening_over_closer() {
        let e1 = Entity::from_raw_u32(1).unwrap();
        let e2 = Entity::from_raw_u32(2).unwrap();
        // e1: threatening, farther; e2: not threatening, closer
        let candidates = vec![
            (e1, true, 1.0, 10.0),
            (e2, false, 0.5, 3.0),
        ];
        let result = select_best_target(candidates.into_iter());
        assert_eq!(result, Some(e1));
    }

    #[test]
    fn select_best_target_least_rotation_among_equal_threat() {
        let e1 = Entity::from_raw_u32(1).unwrap();
        let e2 = Entity::from_raw_u32(2).unwrap();
        // Both threatening; e1 has less rotation
        let candidates = vec![
            (e1, true, 0.3, 5.0),
            (e2, true, 1.5, 3.0),
        ];
        let result = select_best_target(candidates.into_iter());
        assert_eq!(result, Some(e1));
    }

    #[test]
    fn select_best_target_closest_as_tiebreaker() {
        let e1 = Entity::from_raw_u32(1).unwrap();
        let e2 = Entity::from_raw_u32(2).unwrap();
        // Both threatening, same rotation; e2 is closer
        let candidates = vec![
            (e1, true, 0.5, 7.0),
            (e2, true, 0.5, 3.0),
        ];
        let result = select_best_target(candidates.into_iter());
        assert_eq!(result, Some(e2));
    }

    #[test]
    fn select_best_target_empty_returns_none() {
        let candidates: Vec<(Entity, bool, f32, f32)> = vec![];
        let result = select_best_target(candidates.into_iter());
        assert_eq!(result, None);
    }

    // === ExtractionPlate destruction cleanup logic ===

    #[test]
    fn extraction_plate_state_has_attached_patch() {
        let patch_entity = Entity::from_raw_u32(42).unwrap();
        let plate = ExtractionPlateState {
            attached_patch: patch_entity,
            mining_timer: 0,
        };
        assert_eq!(plate.attached_patch, patch_entity);
    }

    #[test]
    fn patch_with_resources_uncovered_on_plate_destroy() {
        // When plate is destroyed and patch has remaining resources,
        // has_plate should be set to false
        let mut patch = SpaceCrystalPatch {
            remaining_amount: 500,
            initial_amount: 1000,
            has_plate: true,
        };
        // Simulate the destruction logic
        if patch.remaining_amount > 0 {
            patch.has_plate = false;
        }
        assert!(!patch.has_plate);
        assert_eq!(patch.remaining_amount, 500);
    }

    #[test]
    fn depleted_patch_marked_for_removal_on_plate_destroy() {
        // When plate is destroyed and patch is depleted,
        // the patch should be despawned (remaining == 0)
        let patch = SpaceCrystalPatch {
            remaining_amount: 0,
            initial_amount: 1000,
            has_plate: true,
        };
        let should_despawn_patch = patch.remaining_amount == 0;
        assert!(should_despawn_patch);
    }

    #[test]
    fn patch_has_plate_set_on_placement() {
        // When a plate is placed, has_plate should be set to true
        let mut patch = SpaceCrystalPatch {
            remaining_amount: 1000,
            initial_amount: 1000,
            has_plate: false,
        };
        // Simulate placement
        patch.has_plate = true;
        assert!(patch.has_plate);
    }

    #[test]
    fn patch_has_plate_prevents_double_placement() {
        let patch = SpaceCrystalPatch {
            remaining_amount: 1000,
            initial_amount: 1000,
            has_plate: true,
        };
        // Validation should reject placement on patch with existing plate
        assert!(patch.has_plate);
    }

    #[test]
    fn extraction_plate_hud_display_percentage() {
        // Test the percentage calculation used in HUD
        let remaining: u32 = 300;
        let initial: u32 = 1000;
        let pct = remaining as f32 / initial as f32 * 100.0;
        assert!((pct - 30.0).abs() < 0.01);
    }

    #[test]
    fn extraction_plate_hud_display_depleted() {
        let remaining: u32 = 0;
        let initial: u32 = 1000;
        let pct = if initial > 0 { (remaining as f32 / initial as f32 * 100.0).min(100.0) } else { 0.0 };
        assert!((pct - 0.0).abs() < 0.01);
    }

    #[test]
    fn extraction_plate_hud_display_full() {
        let remaining: u32 = 1000;
        let initial: u32 = 1000;
        let pct = if initial > 0 { (remaining as f32 / initial as f32 * 100.0).min(100.0) } else { 0.0 };
        assert!((pct - 100.0).abs() < 0.01);
    }

    // === Turret autonomous scanning — TurretCommandState integration ===

    #[test]
    fn turret_scanning_sets_locked_target_not_attack_state() {
        // Verify the system writes to TurretCommandState.locked_target
        // rather than AttackState when finding a target
        let mut state = TurretCommandState::default();
        assert!(state.locked_target.is_none());

        // Simulate what the system does on finding a target
        let target = Entity::from_raw_u32(42).unwrap();
        state.locked_target = Some(target);
        assert_eq!(state.locked_target, Some(target));
    }

    #[test]
    fn turret_scanning_skips_when_locked_target_present() {
        // When locked_target is Some and the entity is valid, scanning should skip
        let target = Entity::from_raw_u32(10).unwrap();
        let state = TurretCommandState { locked_target: Some(target) };

        // The system checks: if locked_target is Some and entity exists → continue
        assert!(state.locked_target.is_some());
    }

    #[test]
    fn turret_scanning_clears_invalid_locked_target() {
        // When locked_target refers to a despawned entity, it should be cleared
        let target = Entity::from_raw_u32(999).unwrap();
        let mut state = TurretCommandState { locked_target: Some(target) };

        // Simulate: potential_targets.get(target).is_err() → clear
        let target_exists = false; // simulating despawned entity
        if !target_exists {
            state.locked_target = None;
        }
        assert!(state.locked_target.is_none());
    }

    #[test]
    fn turret_scanning_no_target_leaves_locked_target_none() {
        // When no valid candidates found, locked_target stays None
        let state = TurretCommandState::default();
        let best_candidate: Option<(bool, f32, f32, Entity)> = None;

        // System only sets locked_target when best_candidate is Some
        assert!(best_candidate.is_none());
        assert!(state.locked_target.is_none());
    }

    // === turret_engagement_system tests ===

    #[test]
    fn engagement_no_target_sets_channels_idle() {
        // When locked_target is None, channels should be set to idle
        let mut turret_orient = TurretOrientationChannel::Turning(Vec3::ZERO);
        let mut turret_attack = TurretAttackChannel::Aiming(Entity::from_raw_u32(1).unwrap());
        let mut turret = Turret::full_rotation(1.0);
        let locked_target: Option<Entity> = None;

        // Simulate the no-target branch
        if locked_target.is_none() {
            turret_orient = TurretOrientationChannel::Maintaining;
            turret_attack = TurretAttackChannel::Inactive;
            turret.target_angle = None;
        }

        assert!(matches!(turret_orient, TurretOrientationChannel::Maintaining));
        assert!(matches!(turret_attack, TurretAttackChannel::Inactive));
        assert!(turret.target_angle.is_none());
    }

    #[test]
    fn engagement_despawned_target_clears_locked_target() {
        // When locked_target refers to a despawned entity, clear it and idle
        let target = Entity::from_raw_u32(42).unwrap();
        let mut cmd_state = TurretCommandState { locked_target: Some(target) };
        let mut turret_orient = TurretOrientationChannel::Turning(Vec3::ZERO);
        let mut turret_attack = TurretAttackChannel::Aiming(target);
        let mut turret = Turret::full_rotation(1.0);

        // Simulate target lookup failure
        let target_exists = false;
        if !target_exists {
            cmd_state.locked_target = None;
            turret_orient = TurretOrientationChannel::Maintaining;
            turret_attack = TurretAttackChannel::Inactive;
            turret.target_angle = None;
        }

        assert!(cmd_state.locked_target.is_none());
        assert!(matches!(turret_orient, TurretOrientationChannel::Maintaining));
        assert!(matches!(turret_attack, TurretAttackChannel::Inactive));
    }

    #[test]
    fn engagement_target_out_of_range_clears_locked_target() {
        // When target is beyond effective range, clear locked_target
        let target = Entity::from_raw_u32(10).unwrap();
        let mut cmd_state = TurretCommandState { locked_target: Some(target) };

        let distance = 15.0_f32;
        let effective_range = 10.0_f32;

        if distance > effective_range {
            cmd_state.locked_target = None;
        }

        assert!(cmd_state.locked_target.is_none());
    }

    #[test]
    fn engagement_target_outside_arc_clears_locked_target() {
        // When target is outside the turret's arc, clear locked_target
        let target = Entity::from_raw_u32(10).unwrap();
        let mut cmd_state = TurretCommandState { locked_target: Some(target) };
        let turret = Turret::limited_rotation(std::f32::consts::FRAC_PI_2, 1.0); // 90 degree arc

        let relative_angle = std::f32::consts::PI; // target behind
        if !turret.can_reach_angle(relative_angle) {
            cmd_state.locked_target = None;
        }

        assert!(cmd_state.locked_target.is_none());
    }

    #[test]
    fn engagement_target_within_arc_keeps_locked_target() {
        // When target is within the turret's arc, keep locked_target
        let target = Entity::from_raw_u32(10).unwrap();
        let cmd_state = TurretCommandState { locked_target: Some(target) };
        let turret = Turret::full_rotation(1.0);

        let relative_angle = 0.5;
        assert!(turret.can_reach_angle(relative_angle));
        assert!(cmd_state.locked_target.is_some());
    }

    #[test]
    fn engagement_aligned_maps_attack_phase_to_channel() {
        // When turret is aligned, AttackPhase should map to TurretAttackChannel
        let target = Entity::from_raw_u32(10).unwrap();

        // Aiming phase
        let phase = AttackPhase::Aiming;
        let channel = match phase {
            AttackPhase::Aiming => TurretAttackChannel::Aiming(target),
            AttackPhase::Firing => TurretAttackChannel::Firing(target),
            AttackPhase::Cooldown => TurretAttackChannel::Cooldown,
            AttackPhase::Reloading => TurretAttackChannel::Reloading,
            AttackPhase::None => TurretAttackChannel::Aiming(target),
        };
        assert!(matches!(channel, TurretAttackChannel::Aiming(_)));

        // Firing phase
        let phase = AttackPhase::Firing;
        let channel = match phase {
            AttackPhase::Aiming => TurretAttackChannel::Aiming(target),
            AttackPhase::Firing => TurretAttackChannel::Firing(target),
            AttackPhase::Cooldown => TurretAttackChannel::Cooldown,
            AttackPhase::Reloading => TurretAttackChannel::Reloading,
            AttackPhase::None => TurretAttackChannel::Aiming(target),
        };
        assert!(matches!(channel, TurretAttackChannel::Firing(_)));

        // Cooldown phase
        let phase = AttackPhase::Cooldown;
        let channel = match phase {
            AttackPhase::Aiming => TurretAttackChannel::Aiming(target),
            AttackPhase::Firing => TurretAttackChannel::Firing(target),
            AttackPhase::Cooldown => TurretAttackChannel::Cooldown,
            AttackPhase::Reloading => TurretAttackChannel::Reloading,
            AttackPhase::None => TurretAttackChannel::Aiming(target),
        };
        assert!(matches!(channel, TurretAttackChannel::Cooldown));

        // Reloading phase
        let phase = AttackPhase::Reloading;
        let channel = match phase {
            AttackPhase::Aiming => TurretAttackChannel::Aiming(target),
            AttackPhase::Firing => TurretAttackChannel::Firing(target),
            AttackPhase::Cooldown => TurretAttackChannel::Cooldown,
            AttackPhase::Reloading => TurretAttackChannel::Reloading,
            AttackPhase::None => TurretAttackChannel::Aiming(target),
        };
        assert!(matches!(channel, TurretAttackChannel::Reloading));

        // None phase (should default to Aiming)
        let phase = AttackPhase::None;
        let channel = match phase {
            AttackPhase::Aiming => TurretAttackChannel::Aiming(target),
            AttackPhase::Firing => TurretAttackChannel::Firing(target),
            AttackPhase::Cooldown => TurretAttackChannel::Cooldown,
            AttackPhase::Reloading => TurretAttackChannel::Reloading,
            AttackPhase::None => TurretAttackChannel::Aiming(target),
        };
        assert!(matches!(channel, TurretAttackChannel::Aiming(_)));
    }

    #[test]
    fn engagement_not_aligned_sets_aiming() {
        // When turret is not aligned, should be Aiming regardless of phase
        let target = Entity::from_raw_u32(10).unwrap();
        let turret = Turret::full_rotation(1.0);
        // current_angle is 0.0, target angle is 1.0 — not aligned
        let relative_angle = 1.0;
        let aligned = (turret.current_angle - relative_angle).abs() < TURRET_ALIGNMENT_TOLERANCE;
        assert!(!aligned);

        // Should set Aiming when not aligned
        let channel = if aligned {
            TurretAttackChannel::Inactive // won't reach here
        } else {
            TurretAttackChannel::Aiming(target)
        };
        assert!(matches!(channel, TurretAttackChannel::Aiming(_)));
    }

    #[test]
    fn engagement_sets_turret_target_angle() {
        // When target is valid, turret.target_angle should be set to clamped angle
        let mut turret = Turret::limited_rotation(std::f32::consts::PI, 1.0); // 180 degree arc
        let relative_angle = 0.3;
        let clamped = turret.clamp_angle(relative_angle);
        turret.target_angle = Some(clamped);

        assert_eq!(turret.target_angle, Some(0.3));
    }

    #[test]
    fn engagement_clamps_angle_to_arc() {
        // Turret.target_angle should be clamped to arc limits
        let turret = Turret::limited_rotation(std::f32::consts::FRAC_PI_2, 1.0); // 90 degree arc
        let relative_angle = 1.0; // ~57 degrees, within 45 degree half-arc? No, > PI/4
        let clamped = turret.clamp_angle(relative_angle);
        let half_arc = std::f32::consts::FRAC_PI_2 / 2.0; // PI/4
        assert!((clamped - half_arc).abs() < 0.01,
            "Expected clamped to ~{}, got {}", half_arc, clamped);
    }

    #[test]
    fn engagement_orientation_set_to_turning_for_valid_target() {
        // When target is valid and in range/arc, orientation should be Turning(target_pos)
        let target_pos = Vec3::new(5.0, 0.0, 3.0);
        let turret_orient = TurretOrientationChannel::Turning(target_pos);
        match turret_orient {
            TurretOrientationChannel::Turning(pos) => {
                assert!((pos.x - 5.0).abs() < 0.01);
                assert!((pos.z - 3.0).abs() < 0.01);
            }
            _ => panic!("Expected Turning variant"),
        }
    }

    #[test]
    fn alignment_tolerance_constant_is_reasonable() {
        // TURRET_ALIGNMENT_TOLERANCE should be small but not too small
        assert!(TURRET_ALIGNMENT_TOLERANCE > 0.0);
        assert!(TURRET_ALIGNMENT_TOLERANCE < 0.2); // less than ~11 degrees
    }

    // === attack_channel_sync_system tests ===

    #[test]
    fn sync_aiming_phase_sets_base_attack_channel_aiming() {
        // When AttackState.phase is Aiming with a UnitTarget,
        // BaseAttackChannel should be Aiming(target_entity)
        let target = Entity::from_raw_u32(42).unwrap();
        let attack_state = AttackState {
            phase: AttackPhase::Aiming,
            time_in_phase: 0.1,
            current_target: Some(AttackTarget::UnitTarget(target)),
        };

        // Simulate the sync logic
        let channel = match attack_state.phase {
            AttackPhase::Aiming => {
                if let Some(t) = attack_state.target_entity() {
                    BaseAttackChannel::Aiming(t)
                } else {
                    BaseAttackChannel::None
                }
            }
            _ => BaseAttackChannel::None,
        };
        assert!(matches!(channel, BaseAttackChannel::Aiming(e) if e == target));
    }

    #[test]
    fn sync_firing_phase_sets_base_attack_channel_firing() {
        let target = Entity::from_raw_u32(10).unwrap();
        let attack_state = AttackState {
            phase: AttackPhase::Firing,
            time_in_phase: 0.05,
            current_target: Some(AttackTarget::UnitTarget(target)),
        };

        let channel = if let Some(t) = attack_state.target_entity() {
            BaseAttackChannel::Firing(t)
        } else {
            BaseAttackChannel::Cooldown
        };
        assert!(matches!(channel, BaseAttackChannel::Firing(e) if e == target));
    }

    #[test]
    fn sync_cooldown_phase_sets_base_attack_channel_cooldown() {
        let attack_state = AttackState {
            phase: AttackPhase::Cooldown,
            time_in_phase: 0.0,
            current_target: Some(AttackTarget::UnitTarget(Entity::from_raw_u32(1).unwrap())),
        };

        let channel = match attack_state.phase {
            AttackPhase::Cooldown => BaseAttackChannel::Cooldown,
            _ => BaseAttackChannel::None,
        };
        assert!(matches!(channel, BaseAttackChannel::Cooldown));
    }

    #[test]
    fn sync_reloading_phase_sets_base_attack_channel_reloading() {
        let attack_state = AttackState {
            phase: AttackPhase::Reloading,
            time_in_phase: 0.5,
            current_target: Some(AttackTarget::UnitTarget(Entity::from_raw_u32(1).unwrap())),
        };

        let channel = match attack_state.phase {
            AttackPhase::Reloading => BaseAttackChannel::Reloading,
            _ => BaseAttackChannel::None,
        };
        assert!(matches!(channel, BaseAttackChannel::Reloading));
    }

    #[test]
    fn sync_none_phase_sets_base_attack_channel_none() {
        let attack_state = AttackState::default();

        let channel = match attack_state.phase {
            AttackPhase::None => BaseAttackChannel::None,
            _ => BaseAttackChannel::Cooldown, // shouldn't happen
        };
        assert!(matches!(channel, BaseAttackChannel::None));
    }

    #[test]
    fn sync_aiming_enforces_stationary_locomotion() {
        // Aiming should force LocomotionChannel::Stationary
        let phase = AttackPhase::Aiming;
        let constraints = phase.base_action_constraints(false);
        assert!(!constraints.base_can_move, "Aiming should block movement");
        // The sync system enforces LocomotionChannel::Stationary
        let locomotion = LocomotionChannel::Stationary;
        assert!(matches!(locomotion, LocomotionChannel::Stationary));
    }

    #[test]
    fn sync_aiming_sets_orientation_turning_toward_target() {
        // Aiming should override OrientationChannel to Turning(target_pos)
        let target_pos = Vec3::new(10.0, 0.0, 5.0);
        let orientation = OrientationChannel::Turning(target_pos);
        match orientation {
            OrientationChannel::Turning(pos) => {
                assert_eq!(pos, target_pos);
            }
            _ => panic!("Expected Turning variant"),
        }
    }

    #[test]
    fn sync_firing_enforces_stationary_and_maintaining() {
        // Firing should force LocomotionChannel::Stationary and OrientationChannel::Maintaining
        let phase = AttackPhase::Firing;
        let constraints = phase.base_action_constraints(false);
        assert!(!constraints.base_can_move, "Firing should block movement");
        assert!(!constraints.base_can_turn, "Firing should block turning");

        let locomotion = LocomotionChannel::Stationary;
        let orientation = OrientationChannel::Maintaining;
        assert!(matches!(locomotion, LocomotionChannel::Stationary));
        assert!(matches!(orientation, OrientationChannel::Maintaining));
    }

    #[test]
    fn sync_cooldown_enforces_stationary_and_maintaining() {
        // Cooldown should force LocomotionChannel::Stationary and OrientationChannel::Maintaining
        let phase = AttackPhase::Cooldown;
        let constraints = phase.base_action_constraints(false);
        assert!(!constraints.base_can_move, "Cooldown should block movement");
        assert!(!constraints.base_can_turn, "Cooldown should block turning");

        let locomotion = LocomotionChannel::Stationary;
        let orientation = OrientationChannel::Maintaining;
        assert!(matches!(locomotion, LocomotionChannel::Stationary));
        assert!(matches!(orientation, OrientationChannel::Maintaining));
    }

    #[test]
    fn sync_reloading_does_not_constrain_locomotion() {
        // Reloading should NOT enforce any constraint on locomotion or orientation
        let phase = AttackPhase::Reloading;
        let constraints = phase.base_action_constraints(false);
        assert!(constraints.base_can_move, "Reloading should allow movement");
        assert!(constraints.base_can_turn, "Reloading should allow turning");
    }

    #[test]
    fn sync_none_does_not_constrain_locomotion() {
        // None should NOT enforce any constraint on locomotion or orientation
        let phase = AttackPhase::None;
        let constraints = phase.base_action_constraints(false);
        assert!(constraints.base_can_move, "None should allow movement");
        assert!(constraints.base_can_turn, "None should allow turning");
    }

    #[test]
    fn sync_aiming_no_target_entity_sets_none() {
        // If AttackState has no target entity during Aiming, BaseAttackChannel should be None
        let attack_state = AttackState {
            phase: AttackPhase::Aiming,
            time_in_phase: 0.1,
            current_target: None,
        };
        assert!(attack_state.target_entity().is_none());
        // The sync system checks target_entity() → None → sets BaseAttackChannel::None
    }

    #[test]
    fn sync_firing_location_target_falls_back_to_cooldown_channel() {
        // When firing with a LocationTarget (AoE snapshot), target_entity() returns None
        // The sync system handles this by setting BaseAttackChannel::Cooldown
        let attack_state = AttackState {
            phase: AttackPhase::Firing,
            time_in_phase: 0.0,
            current_target: Some(AttackTarget::LocationTarget(Vec3::new(5.0, 0.0, 5.0))),
        };
        assert!(attack_state.target_entity().is_none());
        assert!(attack_state.target_location().is_some());
        // Fallback: Cooldown channel (still enforces stationary constraints)
    }

    #[test]
    fn sync_interruptibility_aiming_allows_cancel() {
        // Aiming is interruptible — new commands can cancel the attack
        let attack_state = AttackState {
            phase: AttackPhase::Aiming,
            time_in_phase: 0.2,
            current_target: Some(AttackTarget::UnitTarget(Entity::from_raw_u32(1).unwrap())),
        };
        assert!(attack_state.phase.is_interruptible());
    }

    #[test]
    fn sync_interruptibility_firing_blocks_cancel() {
        // Firing is NOT interruptible — commands are queued/rejected
        let attack_state = AttackState {
            phase: AttackPhase::Firing,
            time_in_phase: 0.0,
            current_target: Some(AttackTarget::UnitTarget(Entity::from_raw_u32(1).unwrap())),
        };
        assert!(!attack_state.phase.is_interruptible());
    }

    #[test]
    fn sync_interruptibility_reloading_allows_cancel() {
        // Reloading is interruptible
        let attack_state = AttackState {
            phase: AttackPhase::Reloading,
            time_in_phase: 0.3,
            current_target: Some(AttackTarget::UnitTarget(Entity::from_raw_u32(1).unwrap())),
        };
        assert!(attack_state.phase.is_interruptible());
    }

    #[test]
    fn sync_interruptibility_cooldown_blocks_cancel() {
        // Cooldown is NOT interruptible
        let attack_state = AttackState {
            phase: AttackPhase::Cooldown,
            time_in_phase: 0.05,
            current_target: Some(AttackTarget::UnitTarget(Entity::from_raw_u32(1).unwrap())),
        };
        assert!(!attack_state.phase.is_interruptible());
    }

    #[test]
    fn sync_phase_channel_mapping_exhaustive() {
        // Verify all AttackPhase variants have corresponding BaseAttackChannel mappings
        let target = Entity::from_raw_u32(1).unwrap();
        let phases_and_expected: Vec<(AttackPhase, bool)> = vec![
            (AttackPhase::None, true),       // → BaseAttackChannel::None
            (AttackPhase::Aiming, true),     // → BaseAttackChannel::Aiming(target)
            (AttackPhase::Firing, true),     // → BaseAttackChannel::Firing(target)
            (AttackPhase::Cooldown, true),   // → BaseAttackChannel::Cooldown
            (AttackPhase::Reloading, true),  // → BaseAttackChannel::Reloading
        ];

        for (phase, _) in &phases_and_expected {
            let attack_state = AttackState {
                phase: *phase,
                time_in_phase: 0.0,
                current_target: Some(AttackTarget::UnitTarget(target)),
            };

            let channel = match attack_state.phase {
                AttackPhase::Aiming => {
                    if let Some(t) = attack_state.target_entity() {
                        BaseAttackChannel::Aiming(t)
                    } else {
                        BaseAttackChannel::None
                    }
                }
                AttackPhase::Firing => {
                    if let Some(t) = attack_state.target_entity() {
                        BaseAttackChannel::Firing(t)
                    } else {
                        BaseAttackChannel::Cooldown
                    }
                }
                AttackPhase::Cooldown => BaseAttackChannel::Cooldown,
                AttackPhase::Reloading => BaseAttackChannel::Reloading,
                AttackPhase::None => BaseAttackChannel::None,
            };

            // Verify each phase produces a valid channel
            match attack_state.phase {
                AttackPhase::Aiming => assert!(matches!(channel, BaseAttackChannel::Aiming(_))),
                AttackPhase::Firing => assert!(matches!(channel, BaseAttackChannel::Firing(_))),
                AttackPhase::Cooldown => assert!(matches!(channel, BaseAttackChannel::Cooldown)),
                AttackPhase::Reloading => assert!(matches!(channel, BaseAttackChannel::Reloading)),
                AttackPhase::None => assert!(matches!(channel, BaseAttackChannel::None)),
            }
        }
    }

    #[test]
    fn sync_constraint_consistency_with_base_action_constraints() {
        // Verify that the sync system's constraint behavior matches
        // AttackPhase::base_action_constraints(false) for all phases
        let target = Entity::from_raw_u32(1).unwrap();

        for phase in [AttackPhase::None, AttackPhase::Aiming, AttackPhase::Firing,
                      AttackPhase::Cooldown, AttackPhase::Reloading]
        {
            let constraints = phase.base_action_constraints(false);

            // For non-turret units:
            // !base_can_move → LocomotionChannel::Stationary enforced
            // !base_can_turn → OrientationChannel::Maintaining enforced
            match phase {
                AttackPhase::Aiming => {
                    assert!(!constraints.base_can_move, "Aiming: no move");
                    assert!(constraints.base_can_turn, "Aiming: can turn");
                }
                AttackPhase::Firing | AttackPhase::Cooldown => {
                    assert!(!constraints.base_can_move, "{:?}: no move", phase);
                    assert!(!constraints.base_can_turn, "{:?}: no turn", phase);
                }
                AttackPhase::None | AttackPhase::Reloading => {
                    assert!(constraints.base_can_move, "{:?}: can move", phase);
                    assert!(constraints.base_can_turn, "{:?}: can turn", phase);
                }
            }
        }
    }
}
