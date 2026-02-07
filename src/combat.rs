use bevy::prelude::*;
use crate::units::{Unit, UnitHealth, Owner};

/// Component defining a unit's attack capabilities
#[derive(Component, Clone)]
pub struct AttackCapability {
    pub damage: f32,
    pub range: f32,
    pub aim_time: f32,
    pub fire_time: f32,
    pub cooldown_time: f32,
    pub reload_time: f32,
    pub attack_type: AttackType,
}

impl Default for AttackCapability {
    fn default() -> Self {
        Self {
            damage: 10.0,
            range: 5.0,
            aim_time: 0.3,
            fire_time: 0.1,
            cooldown_time: 0.1,
            reload_time: 1.0,
            attack_type: AttackType::FullyConnected,
        }
    }
}

/// Types of attacks (from design doc)
#[derive(Clone, Debug)]
pub enum AttackType {
    FullyConnected,     // Instant hit, cannot miss
    TailDisjointed {    // Projectile after firing, homing
        projectile_speed: f32,
        projectile_visual: crate::projectile::ProjectileVisual,
    },
    HeadDisjointed {    // Attack target location, instant
        effect_radius: f32,
    },
    DoublyDisjointed {  // Projectile to location
        projectile_speed: f32,
        projectile_visual: crate::projectile::ProjectileVisual,
        effect_radius: f32,
    },
}

/// Attack phase states
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AttackPhase {
    None,
    Aiming,
    Firing,
    Cooldown,
    Reloading,
}

/// Component tracking current attack state
#[derive(Component)]
pub struct AttackState {
    pub phase: AttackPhase,
    pub time_in_phase: f32,
    pub target: Option<Entity>,
    pub target_location: Option<Vec3>, // For Head/Doubly Disjointed attacks
}

impl Default for AttackState {
    fn default() -> Self {
        Self {
            phase: AttackPhase::None,
            time_in_phase: 0.0,
            target: None,
            target_location: None,
        }
    }
}

/// Plugin for combat systems
pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            attack_command_system,
            attack_phase_system,
            auto_target_system,
            apply_damage_system,
            remove_dead_units_system,
        ));
    }
}

/// System to handle attack commands
fn attack_command_system(
    mut units: Query<
        (Entity, &Transform, &AttackCapability, &mut AttackState, &crate::commands::UnitCommand),
        With<Unit>
    >,
    targets: Query<(&Transform, &Owner), With<Unit>>,
) {
    for (entity, transform, attack_cap, mut attack_state, unit_command) in units.iter_mut() {
        // Check if unit has an attack target command
        if let crate::commands::UnitCommand::AttackTarget(target_entity) = unit_command {
            // Validate target exists
            if targets.get(*target_entity).is_ok() {
                // Set attack target
                if attack_state.target.is_none() {
                    attack_state.target = Some(*target_entity);
                    attack_state.phase = AttackPhase::None;
                    attack_state.time_in_phase = 0.0;
                }
            }
        }
    }
}

/// System to progress through attack phases
fn attack_phase_system(
    mut commands: Commands,
    time: Res<Time>,
    mut units: Query<
        (Entity, &Transform, &AttackCapability, &mut AttackState, &Owner),
        With<Unit>
    >,
    targets: Query<(&Transform, &Owner), With<Unit>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let delta = time.delta_seconds();

    for (entity, transform, attack_cap, mut attack_state, owner) in units.iter_mut() {
        // Skip if no target
        let Some(target_entity) = attack_state.target else {
            continue;
        };

        // Validate target still exists and is enemy
        let Ok((target_transform, target_owner)) = targets.get(target_entity) else {
            // Target no longer valid
            attack_state.target = None;
            attack_state.phase = AttackPhase::None;
            continue;
        };

        // Check if target is enemy
        if !is_enemy(owner, target_owner) {
            attack_state.target = None;
            attack_state.phase = AttackPhase::None;
            continue;
        }

        // Calculate distance to target
        let distance = transform.translation.distance(target_transform.translation);

        // Progress time in current phase
        attack_state.time_in_phase += delta;

        match attack_state.phase {
            AttackPhase::None => {
                // Start attacking if in range
                if distance <= attack_cap.range {
                    attack_state.phase = AttackPhase::Aiming;
                    attack_state.time_in_phase = 0.0;
                    info!("Unit {:?} starting attack on target", entity);
                }
            }

            AttackPhase::Aiming => {
                // Check if target still in range
                if distance > attack_cap.range {
                    attack_state.phase = AttackPhase::None;
                    continue;
                }

                // Complete aiming phase
                if attack_state.time_in_phase >= attack_cap.aim_time {
                    // Capture target location for Head/Doubly Disjointed attacks
                    match &attack_cap.attack_type {
                        AttackType::HeadDisjointed { .. } | AttackType::DoublyDisjointed { .. } => {
                            attack_state.target_location = Some(target_transform.translation);
                        }
                        _ => {}
                    }

                    attack_state.phase = AttackPhase::Firing;
                    attack_state.time_in_phase = 0.0;
                }
            }

            AttackPhase::Firing => {
                // Handle attack type when entering firing phase
                if attack_state.time_in_phase <= delta {
                    // Just entered firing phase
                    match &attack_cap.attack_type {
                        AttackType::FullyConnected => {
                            // Instant hit
                            if let Ok(_) = targets.get(target_entity) {
                                commands.entity(target_entity).insert(DamageEvent {
                                    amount: attack_cap.damage,
                                    source: entity,
                                });
                                info!("Unit {:?} fired (instant hit), damage: {}", entity, attack_cap.damage);
                            }
                        }

                        AttackType::TailDisjointed { projectile_speed, projectile_visual } => {
                            // Spawn homing projectile
                            if let Ok((target_transform, _)) = targets.get(target_entity) {
                                crate::projectile::spawn_projectile(
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
                            // Instant AOE at target location
                            if let Some(target_loc) = attack_state.target_location {
                                // Apply AOE damage at location
                                apply_aoe_damage(
                                    &mut commands,
                                    &targets,
                                    target_loc,
                                    *effect_radius,
                                    attack_cap.damage,
                                    entity,
                                    owner,
                                );

                                // Spawn explosion effect
                                crate::projectile::spawn_projectile(
                                    &mut commands,
                                    &mut meshes,
                                    &mut materials,
                                    target_loc,
                                    target_loc,
                                    1.0,
                                    0.0,
                                    Some(*effect_radius),
                                    crate::projectile::ProjectileVisual::Sphere { radius: 0.1 },
                                    *owner,
                                );

                                info!("Unit {:?} fired AOE attack (instant)", entity);
                            }
                        }

                        AttackType::DoublyDisjointed { projectile_speed, projectile_visual, effect_radius } => {
                            // Spawn projectile to location
                            if let Some(target_loc) = attack_state.target_location {
                                crate::projectile::spawn_projectile(
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

                // Complete firing phase
                if attack_state.time_in_phase >= attack_cap.fire_time {
                    attack_state.phase = AttackPhase::Cooldown;
                    attack_state.time_in_phase = 0.0;
                }
            }

            AttackPhase::Cooldown => {
                // Complete cooldown phase
                if attack_state.time_in_phase >= attack_cap.cooldown_time {
                    attack_state.phase = AttackPhase::Reloading;
                    attack_state.time_in_phase = 0.0;
                }
            }

            AttackPhase::Reloading => {
                // Complete reloading phase
                if attack_state.time_in_phase >= attack_cap.reload_time {
                    // Check if target still in range, restart attack
                    if distance <= attack_cap.range {
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

/// System to auto-target nearby enemies for idle units
fn auto_target_system(
    mut units: Query<
        (Entity, &Transform, &AttackCapability, &mut AttackState, &Owner),
        (With<Unit>, Without<crate::commands::HoldingPosition>)
    >,
    potential_targets: Query<(Entity, &Transform, &Owner), With<Unit>>,
    unit_commands: Query<&crate::commands::UnitCommand>,
) {
    for (entity, transform, attack_cap, mut attack_state, owner) in units.iter_mut() {
        // Skip if already attacking
        if attack_state.target.is_some() {
            continue;
        }

        // Skip if unit has an explicit attack target or stop command
        if let Ok(command) = unit_commands.get(entity) {
            if matches!(command, crate::commands::UnitCommand::AttackTarget(_) | crate::commands::UnitCommand::Stop) {
                continue;
            }
        }

        // Find nearest enemy in range
        let mut nearest_enemy = None;
        let mut nearest_distance = f32::MAX;

        for (target_entity, target_transform, target_owner) in potential_targets.iter() {
            // Skip self
            if target_entity == entity {
                continue;
            }

            // Skip allies
            if !is_enemy(owner, target_owner) {
                continue;
            }

            let distance = transform.translation.distance(target_transform.translation);

            // Check if in range
            if distance <= attack_cap.range && distance < nearest_distance {
                nearest_enemy = Some(target_entity);
                nearest_distance = distance;
            }
        }

        // Set target if found
        if let Some(target) = nearest_enemy {
            attack_state.target = Some(target);
            attack_state.phase = AttackPhase::Aiming;
            attack_state.time_in_phase = 0.0;
            info!("Unit {:?} auto-targeting enemy at distance {:.1}", entity, nearest_distance);
        }
    }
}

/// Component for damage events
#[derive(Component)]
pub struct DamageEvent {
    pub amount: f32,
    pub source: Entity,
}

/// System to apply damage to units
fn apply_damage_system(
    mut commands: Commands,
    mut units: Query<(Entity, &mut UnitHealth), With<DamageEvent>>,
    damage_events: Query<&DamageEvent>,
) {
    for (entity, mut health) in units.iter_mut() {
        if let Ok(damage_event) = damage_events.get(entity) {
            health.current -= damage_event.amount;
            health.current = health.current.max(0.0);

            info!("Unit {:?} took {} damage, health: {}/{}",
                entity, damage_event.amount, health.current, health.max);

            // Remove damage event
            commands.entity(entity).remove::<DamageEvent>();
        }
    }
}

/// System to remove dead units
fn remove_dead_units_system(
    mut commands: Commands,
    units: Query<(Entity, &UnitHealth), With<Unit>>,
) {
    for (entity, health) in units.iter() {
        if health.current <= 0.0 {
            info!("Unit {:?} destroyed", entity);
            commands.entity(entity).despawn_recursive();
        }
    }
}

/// Check if two owners are enemies
fn is_enemy(owner1: &Owner, owner2: &Owner) -> bool {
    match (owner1, owner2) {
        (Owner::Player(p1), Owner::Player(p2)) => p1 != p2,
        (Owner::Neutral, _) | (_, Owner::Neutral) => false,
    }
}

/// Apply AOE damage at a location
fn apply_aoe_damage(
    commands: &mut Commands,
    targets: &Query<(&Transform, &Owner), With<Unit>>,
    location: Vec3,
    radius: f32,
    damage: f32,
    source: Entity,
    source_owner: &Owner,
) {
    for (target_entity, (target_transform, target_owner)) in targets.iter().enumerate() {
        let distance = target_transform.translation.distance(location);

        if distance <= radius && is_enemy(source_owner, target_owner) {
            // Apply damage with falloff
            let damage_multiplier = 1.0 - (distance / radius);
            let actual_damage = damage * damage_multiplier;

            // This is a workaround - in real code we'd need the entity ID from the query
            // For now, AOE damage is handled in projectile.rs
        }
    }
}
