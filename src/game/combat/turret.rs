use bevy::prelude::*;
use crate::types::Unit;
use super::types::*;

/// System to set turret target angles based on attack targets
pub fn turret_aiming_system(
    mut units: Query<
        (&Transform, &mut Turret, &AttackState),
        With<Unit>
    >,
    targets: Query<&Transform, With<Unit>>,
) {
    for (unit_transform, mut turret, attack_state) in units.iter_mut() {
        if !matches!(attack_state.phase, AttackPhase::Aiming | AttackPhase::Reloading) {
            turret.target_angle = None;
            continue;
        }

        let Some(target_entity) = attack_state.target_entity() else {
            turret.target_angle = None;
            continue;
        };

        let Ok(target_transform) = targets.get(target_entity) else {
            turret.target_angle = None;
            continue;
        };

        let to_target = target_transform.translation - unit_transform.translation;
        let direction_2d = Vec2::new(to_target.x, to_target.z).normalize();

        let target_world_angle = direction_2d.y.atan2(direction_2d.x);

        let unit_forward = unit_transform.forward();
        let unit_direction_2d = Vec2::new(unit_forward.x, unit_forward.z).normalize();
        let unit_world_angle = unit_direction_2d.y.atan2(unit_direction_2d.x);

        let mut relative_angle = target_world_angle - unit_world_angle;

        while relative_angle > std::f32::consts::PI {
            relative_angle -= std::f32::consts::PI * 2.0;
        }
        while relative_angle < -std::f32::consts::PI {
            relative_angle += std::f32::consts::PI * 2.0;
        }

        let clamped_angle = turret.clamp_angle(relative_angle);
        turret.target_angle = Some(clamped_angle);
    }
}

/// System to rotate turrets toward their target angles
pub fn turret_rotation_system(
    time: Res<Time>,
    mut turrets: Query<&mut Turret, With<Unit>>,
) {
    let delta = time.delta_seconds();

    for mut turret in turrets.iter_mut() {
        let Some(target_angle) = turret.target_angle else {
            continue;
        };

        let angle_diff = target_angle - turret.current_angle;

        if angle_diff.abs() < 0.01 {
            turret.current_angle = target_angle;
        } else {
            let rotation_amount = turret.turn_rate * delta;
            let rotation_step = angle_diff.signum() * rotation_amount.min(angle_diff.abs());
            turret.current_angle += rotation_step;
        }
    }
}

/// System to update visual turret entity rotations
pub fn update_turret_visual_system(
    units: Query<(&Turret, &Children), With<Unit>>,
    mut turret_visuals: Query<&mut Transform, With<TurretVisual>>,
) {
    for (turret, children) in units.iter() {
        for &child in children.iter() {
            if let Ok(mut turret_transform) = turret_visuals.get_mut(child) {
                turret_transform.rotation = Quat::from_rotation_y(turret.current_angle);
            }
        }
    }
}
