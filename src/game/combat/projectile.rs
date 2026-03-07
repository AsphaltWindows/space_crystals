use bevy::prelude::*;
use crate::types::{Unit, Owner};
use crate::utils::is_enemy;
use super::types::*;
use super::utils::spawn_explosion_effect;

/// System to move projectiles toward targets
pub fn projectile_movement_system(
    time: Res<Time>,
    mut projectiles: Query<(&mut Transform, &Projectile)>,
) {
    let delta = time.delta_seconds();

    for (mut transform, projectile) in projectiles.iter_mut() {
        let current_pos = transform.translation;
        let target_pos = projectile.target_position;

        let direction = (target_pos - current_pos).normalize_or_zero();
        let movement = direction * projectile.speed * delta;
        transform.translation += movement;

        if direction.length() > 0.01 {
            let look_at = current_pos + direction;
            transform.look_at(look_at, Vec3::Y);
        }
    }
}

/// System to detect projectile impacts and apply damage
pub fn projectile_impact_system(
    mut commands: Commands,
    projectiles: Query<(Entity, &Transform, &Projectile)>,
    mut units: Query<(Entity, &Transform, &Owner), With<Unit>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (projectile_entity, projectile_transform, projectile) in projectiles.iter() {
        let projectile_pos = projectile_transform.translation;
        let target_pos = projectile.target_position;

        let distance = projectile_pos.distance(target_pos);

        if distance < 0.2 {
            if let Some(effect_radius) = projectile.effect_radius {
                // AoE projectile: insert AreaOfEffect damage on all enemy units in radius
                // The apply_damage_system handles area-overlap calculations
                for (unit_entity, unit_transform, unit_owner) in units.iter_mut() {
                    let unit_pos = unit_transform.translation;
                    let distance_to_impact = unit_pos.distance(target_pos);

                    if distance_to_impact <= effect_radius {
                        if is_enemy(&projectile.source_owner, unit_owner) {
                            commands.entity(unit_entity).insert(DamageEvent::AreaOfEffect {
                                damage: projectile.damage,
                                source: projectile_entity,
                                center: target_pos,
                                radius: effect_radius,
                                source_owner: projectile.source_owner,
                            });
                        }
                    }
                }

                spawn_explosion_effect(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    target_pos,
                    effect_radius,
                );
            } else {
                // Single-target projectile: find closest enemy unit near impact
                let mut closest_unit = None;
                let mut closest_distance = f32::MAX;

                for (unit_entity, unit_transform, unit_owner) in units.iter() {
                    let unit_pos = unit_transform.translation;
                    let distance_to_unit = unit_pos.distance(target_pos);

                    if distance_to_unit < 1.0 && distance_to_unit < closest_distance {
                        if is_enemy(&projectile.source_owner, unit_owner) {
                            closest_unit = Some((unit_entity, unit_transform.translation));
                            closest_distance = distance_to_unit;
                        }
                    }
                }

                if let Some((unit_entity, _unit_pos)) = closest_unit {
                    commands.entity(unit_entity).insert(DamageEvent::SingleTarget {
                        damage: projectile.damage,
                        source: projectile_entity,
                        source_position: projectile_pos,
                    });
                }
            }

            commands.entity(projectile_entity).despawn_recursive();
        }
    }
}

/// System to decay and remove target highlight effects
pub fn target_highlight_decay_system(
    mut commands: Commands,
    time: Res<Time>,
    mut highlights: Query<(Entity, &mut TargetHighlight, &mut Transform)>,
) {
    let delta = time.delta_seconds();

    for (entity, mut highlight, mut transform) in highlights.iter_mut() {
        highlight.lifetime += delta;

        // Pulse scale for visual pop
        let pulse = 1.0 + 0.2 * (highlight.lifetime * 10.0).sin();
        transform.scale = Vec3::splat(pulse);

        if highlight.lifetime >= highlight.max_lifetime {
            commands.entity(entity).despawn_recursive();
        }
    }
}

/// System to decay and remove attack line tracers
pub fn attack_line_decay_system(
    mut commands: Commands,
    time: Res<Time>,
    mut lines: Query<(Entity, &mut AttackLine)>,
) {
    let delta = time.delta_seconds();

    for (entity, mut line) in lines.iter_mut() {
        line.lifetime += delta;

        if line.lifetime >= line.max_lifetime {
            commands.entity(entity).despawn_recursive();
        }
    }
}

/// System to animate and remove explosion effects
pub fn explosion_effect_system(
    mut commands: Commands,
    time: Res<Time>,
    mut explosions: Query<(Entity, &mut Transform, &mut ExplosionEffect)>,
) {
    let delta = time.delta_seconds();

    for (entity, mut transform, mut effect) in explosions.iter_mut() {
        effect.lifetime += delta;

        let progress = effect.lifetime / effect.max_lifetime;
        let scale = 1.0 + progress * 2.0;
        transform.scale = Vec3::splat(scale);

        if effect.lifetime >= effect.max_lifetime {
            commands.entity(entity).despawn_recursive();
        }
    }
}
