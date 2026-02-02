use bevy::prelude::*;
use crate::units::{Unit, Owner};
use crate::combat::DamageEvent;

/// Visual style for projectiles
#[derive(Clone, Copy, Debug)]
pub enum ProjectileVisual {
    Sphere { radius: f32 },
    Cylinder { radius: f32, length: f32 },
}

/// Component for projectile entities
#[derive(Component)]
pub struct Projectile {
    pub target_position: Vec3,
    pub speed: f32,
    pub damage: f32,
    pub effect_radius: Option<f32>, // For AOE
    pub source_owner: Owner,
}

/// Component for visual explosion effects
#[derive(Component)]
struct ExplosionEffect {
    lifetime: f32,
    max_lifetime: f32,
}

/// Plugin for projectile systems
pub struct ProjectilePlugin;

impl Plugin for ProjectilePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            projectile_movement_system,
            projectile_impact_system,
            explosion_effect_system,
        ));
    }
}

/// System to move projectiles toward targets
fn projectile_movement_system(
    time: Res<Time>,
    mut projectiles: Query<(&mut Transform, &Projectile)>,
) {
    let delta = time.delta_seconds();

    for (mut transform, projectile) in projectiles.iter_mut() {
        let current_pos = transform.translation;
        let target_pos = projectile.target_position;

        // Calculate direction
        let direction = (target_pos - current_pos).normalize_or_zero();

        // Move projectile
        let movement = direction * projectile.speed * delta;
        transform.translation += movement;

        // Orient projectile toward movement direction
        if direction.length() > 0.01 {
            let look_at = current_pos + direction;
            transform.look_at(look_at, Vec3::Y);
        }
    }
}

/// System to detect projectile impacts and apply damage
fn projectile_impact_system(
    mut commands: Commands,
    projectiles: Query<(Entity, &Transform, &Projectile)>,
    mut units: Query<(Entity, &Transform, &Owner), With<Unit>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (projectile_entity, projectile_transform, projectile) in projectiles.iter() {
        let projectile_pos = projectile_transform.translation;
        let target_pos = projectile.target_position;

        // Check if projectile reached target
        let distance = projectile_pos.distance(target_pos);

        if distance < 0.2 {
            // Projectile hit target location

            if let Some(effect_radius) = projectile.effect_radius {
                // AOE damage
                for (unit_entity, unit_transform, unit_owner) in units.iter_mut() {
                    let unit_pos = unit_transform.translation;
                    let distance_to_impact = unit_pos.distance(target_pos);

                    if distance_to_impact <= effect_radius {
                        // Check if enemy
                        if is_enemy(&projectile.source_owner, unit_owner) {
                            // Apply damage with falloff
                            let damage_multiplier = 1.0 - (distance_to_impact / effect_radius);
                            let actual_damage = projectile.damage * damage_multiplier;

                            commands.entity(unit_entity).insert(DamageEvent {
                                amount: actual_damage,
                                source: projectile_entity,
                            });
                        }
                    }
                }

                // Spawn explosion effect
                spawn_explosion_effect(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    target_pos,
                    effect_radius,
                );
            } else {
                // Single target damage - find closest unit
                let mut closest_unit = None;
                let mut closest_distance = f32::MAX;

                for (unit_entity, unit_transform, unit_owner) in units.iter() {
                    let unit_pos = unit_transform.translation;
                    let distance_to_unit = unit_pos.distance(target_pos);

                    if distance_to_unit < 1.0 && distance_to_unit < closest_distance {
                        if is_enemy(&projectile.source_owner, unit_owner) {
                            closest_unit = Some(unit_entity);
                            closest_distance = distance_to_unit;
                        }
                    }
                }

                // Apply damage to closest unit
                if let Some(unit_entity) = closest_unit {
                    commands.entity(unit_entity).insert(DamageEvent {
                        amount: projectile.damage,
                        source: projectile_entity,
                    });
                }
            }

            // Despawn projectile
            commands.entity(projectile_entity).despawn_recursive();
        }
    }
}

/// System to animate and remove explosion effects
fn explosion_effect_system(
    mut commands: Commands,
    time: Res<Time>,
    mut explosions: Query<(Entity, &mut Transform, &mut ExplosionEffect)>,
) {
    let delta = time.delta_seconds();

    for (entity, mut transform, mut effect) in explosions.iter_mut() {
        effect.lifetime += delta;

        // Scale up explosion over lifetime
        let progress = effect.lifetime / effect.max_lifetime;
        let scale = 1.0 + progress * 2.0;
        transform.scale = Vec3::splat(scale);

        // Remove when lifetime exceeded
        if effect.lifetime >= effect.max_lifetime {
            commands.entity(entity).despawn_recursive();
        }
    }
}

/// Spawn visual explosion effect
fn spawn_explosion_effect(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    radius: f32,
) {
    let explosion_mesh = meshes.add(Sphere::new(radius));
    let explosion_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.5, 0.0),
        emissive: Color::srgb(1.0, 0.3, 0.0).into(),
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    commands.spawn((
        PbrBundle {
            mesh: explosion_mesh,
            material: explosion_material,
            transform: Transform::from_translation(position),
            ..default()
        },
        ExplosionEffect {
            lifetime: 0.0,
            max_lifetime: 0.5,
        },
    ));
}

/// Spawn a projectile entity
pub fn spawn_projectile(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    start_position: Vec3,
    target_position: Vec3,
    speed: f32,
    damage: f32,
    effect_radius: Option<f32>,
    visual: ProjectileVisual,
    source_owner: Owner,
) {
    let (mesh, material) = match visual {
        ProjectileVisual::Sphere { radius } => {
            let mesh = meshes.add(Sphere::new(radius));
            let material = materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 0.8, 0.0),
                emissive: Color::srgb(1.0, 0.8, 0.0).into(),
                ..default()
            });
            (mesh, material)
        }
        ProjectileVisual::Cylinder { radius, length } => {
            let mesh = meshes.add(Capsule3d::new(radius, length));
            let material = materials.add(StandardMaterial {
                base_color: Color::srgb(0.8, 0.8, 0.8),
                metallic: 0.8,
                ..default()
            });
            (mesh, material)
        }
    };

    commands.spawn((
        PbrBundle {
            mesh,
            material,
            transform: Transform::from_translation(start_position),
            ..default()
        },
        Projectile {
            target_position,
            speed,
            damage,
            effect_radius,
            source_owner,
        },
    ));
}

/// Check if two owners are enemies
fn is_enemy(owner1: &Owner, owner2: &Owner) -> bool {
    match (owner1, owner2) {
        (Owner::Player(p1), Owner::Player(p2)) => p1 != p2,
        (Owner::Neutral, _) | (_, Owner::Neutral) => false,
    }
}
