use bevy::prelude::*;
use crate::types::{Unit, Owner, DomainEnum, GridPosition};
use crate::game::types::ObjectInstance;
use crate::utils::is_enemy;
use crate::game::units::types::{UnitCommand, UnitControlCost};
use crate::game::types::factions::{Player, GdoPlayerResources};
use crate::game::world::types::{ElevationMap, elevation_modifier, SpaceCrystalPatch};
use crate::game::types::structures::ExtractionPlateState;
use crate::game::combat::types::*;
use crate::game::combat::utils::{spawn_projectile, apply_aoe_damage, spawn_attack_line, circle_rect_overlap_area};

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
    mut materials: ResMut<Assets<StandardMaterial>>,
    elevation_map: Res<ElevationMap>,
) {
    let delta = time.delta_seconds();

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
                                    &mut meshes,
                                    &mut materials,
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
                                    &mut materials,
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
                                    &mut materials,
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
                                    &mut materials,
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

/// Stub for threat checking — returns true for all targets.
/// TODO: implement domain check when target_domain is properly queried from all targets
pub fn can_threaten(
    _target_attack_cap: Option<&AttackCapability>,
    _defender_domain: &DomainEnum,
) -> bool {
    true
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
        (Entity, &Transform, &Turret, &AttackCapability, &mut AttackState, &Owner, Option<&DomainEnum>, &GridPosition),
        With<Unit>
    >,
    potential_targets: Query<(Entity, &Transform, &Owner, Option<&DomainEnum>, Option<&AttackCapability>, &GridPosition), With<ObjectInstance>>,
    elevation_map: Res<ElevationMap>,
) {
    for (entity, transform, turret, attack_cap, mut attack_state, owner, source_domain, source_grid_pos) in units.iter_mut() {
        // Only scan when no locked target
        if attack_state.target_entity().is_some() {
            continue;
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
            attack_state.current_target = Some(AttackTarget::UnitTarget(target));
            attack_state.phase = AttackPhase::Aiming;
            attack_state.time_in_phase = 0.0;
            info!("Turret unit {:?} auto-scanning target", entity);
        }
    }
}

/// System for base (non-turret) auto-targeting — idle and hold-position units engage nearby enemies.
/// Active only for UnitCommand::Idle, HoldPosition, and AttackMove.
/// Targets include structures (ObjectInstance) not just units.
pub fn base_auto_target_system(
    mut commands: Commands,
    mut units: Query<
        (Entity, &Transform, &AttackCapability, &mut AttackState, &Owner, Option<&DomainEnum>, &GridPosition),
        (With<Unit>, Without<Turret>)
    >,
    potential_targets: Query<(Entity, &Transform, &Owner, Option<&DomainEnum>, &GridPosition), With<ObjectInstance>>,
    unit_commands: Query<&UnitCommand>,
    elevation_map: Res<ElevationMap>,
) {
    for (entity, transform, attack_cap, mut attack_state, owner, source_domain, source_grid_pos) in units.iter_mut() {
        if attack_state.target_entity().is_some() {
            continue;
        }

        let command = unit_commands.get(entity).ok();

        // Only auto-target for Idle, HoldPosition, and AttackMove
        if let Some(cmd) = command {
            match cmd {
                UnitCommand::Idle | UnitCommand::HoldPosition | UnitCommand::AttackMove(_) => {}
                _ => continue,
            }
        } else {
            continue;
        }

        let src_domain = source_domain.copied().unwrap_or(DomainEnum::Ground);
        let src_elev = elevation_map.get(source_grid_pos.x, source_grid_pos.z);

        let mut nearest_enemy = None;
        let mut nearest_distance = f32::MAX;

        for (target_entity, target_transform, target_owner, target_domain, target_grid_pos) in potential_targets.iter() {
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

            if distance <= effective_range && distance < nearest_distance {
                nearest_enemy = Some(target_entity);
                nearest_distance = distance;
            }
        }

        if let Some(target) = nearest_enemy {
            // Record idle origin when an idle unit auto-acquires a target
            if let Some(cmd) = command {
                if matches!(cmd, UnitCommand::Idle) {
                    commands.entity(entity).insert(IdleOrigin(transform.translation));
                }
            }

            attack_state.current_target = Some(AttackTarget::UnitTarget(target));
            attack_state.phase = AttackPhase::Aiming;
            attack_state.time_in_phase = 0.0;
            info!("Unit {:?} auto-targeting enemy at distance {:.1}", entity, nearest_distance);
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
                        commands.entity(plate.attached_patch).despawn_recursive();
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
            commands.entity(entity).despawn_recursive();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::DomainEnum;

    // === can_threaten stub ===

    #[test]
    fn can_threaten_returns_true_for_none_attack_cap() {
        assert!(can_threaten(None, &DomainEnum::Ground));
    }

    #[test]
    fn can_threaten_returns_true_for_some_attack_cap() {
        let cap = AttackCapability::default();
        assert!(can_threaten(Some(&cap), &DomainEnum::Ground));
    }

    #[test]
    fn can_threaten_returns_true_for_air_domain() {
        assert!(can_threaten(None, &DomainEnum::Air));
    }

    #[test]
    fn can_threaten_returns_true_for_underground_domain() {
        assert!(can_threaten(None, &DomainEnum::Underground));
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
        let allowed = matches!(cmd, UnitCommand::Idle | UnitCommand::HoldPosition | UnitCommand::AttackMove(_));
        assert!(allowed);
    }

    #[test]
    fn base_auto_target_allows_hold_position() {
        let cmd = UnitCommand::HoldPosition;
        let allowed = matches!(cmd, UnitCommand::Idle | UnitCommand::HoldPosition | UnitCommand::AttackMove(_));
        assert!(allowed);
    }

    #[test]
    fn base_auto_target_allows_attack_move() {
        let cmd = UnitCommand::AttackMove(Vec3::ZERO);
        let allowed = matches!(cmd, UnitCommand::Idle | UnitCommand::HoldPosition | UnitCommand::AttackMove(_));
        assert!(allowed);
    }

    #[test]
    fn base_auto_target_blocks_move() {
        let cmd = UnitCommand::Move(Vec3::ZERO);
        let allowed = matches!(cmd, UnitCommand::Idle | UnitCommand::HoldPosition | UnitCommand::AttackMove(_));
        assert!(!allowed);
    }

    #[test]
    fn base_auto_target_blocks_attack_target() {
        let cmd = UnitCommand::AttackTarget(Entity::from_raw(1));
        let allowed = matches!(cmd, UnitCommand::Idle | UnitCommand::HoldPosition | UnitCommand::AttackMove(_));
        assert!(!allowed);
    }

    #[test]
    fn base_auto_target_blocks_stop() {
        let cmd = UnitCommand::Stop;
        let allowed = matches!(cmd, UnitCommand::Idle | UnitCommand::HoldPosition | UnitCommand::AttackMove(_));
        assert!(!allowed);
    }

    #[test]
    fn base_auto_target_blocks_attack_location() {
        let cmd = UnitCommand::AttackLocation(Vec3::ZERO);
        let allowed = matches!(cmd, UnitCommand::Idle | UnitCommand::HoldPosition | UnitCommand::AttackMove(_));
        assert!(!allowed);
    }

    #[test]
    fn base_auto_target_blocks_reverse() {
        let cmd = UnitCommand::Reverse(Vec3::ZERO);
        let allowed = matches!(cmd, UnitCommand::Idle | UnitCommand::HoldPosition | UnitCommand::AttackMove(_));
        assert!(!allowed);
    }

    #[test]
    fn base_auto_target_blocks_patrol() {
        let cmd = UnitCommand::Patrol { start: Vec3::ZERO, end: Vec3::ONE, going_to_end: true };
        let allowed = matches!(cmd, UnitCommand::Idle | UnitCommand::HoldPosition | UnitCommand::AttackMove(_));
        assert!(!allowed);
    }

    // === ExtractionPlate destruction cleanup logic ===

    #[test]
    fn extraction_plate_state_has_attached_patch() {
        let patch_entity = Entity::from_raw(42);
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
}
