use bevy::prelude::*;
use crate::units::{Unit, UnitBase, DrillMode};
use crate::combat::{AttackState, AttackPhase};

/// Component defining turret properties
#[derive(Component)]
pub struct Turret {
    pub turn_angle: f32,        // Max rotation from center (radians)
    pub turn_rate: f32,         // Rotation speed (radians/sec)
    pub current_angle: f32,     // Current offset from center (radians)
    pub target_angle: Option<f32>, // Desired angle when aiming
}

impl Turret {
    /// Create a turret with full 360° rotation
    pub fn full_rotation(turn_rate: f32) -> Self {
        Self {
            turn_angle: std::f32::consts::PI * 2.0, // 360 degrees
            turn_rate,
            current_angle: 0.0,
            target_angle: None,
        }
    }

    /// Create a turret with limited rotation arc
    pub fn limited_rotation(max_angle: f32, turn_rate: f32) -> Self {
        Self {
            turn_angle: max_angle,
            turn_rate,
            current_angle: 0.0,
            target_angle: None,
        }
    }

    /// Check if a target angle is within turret's rotation limits
    pub fn can_reach_angle(&self, angle: f32) -> bool {
        let half_angle = self.turn_angle / 2.0;
        angle.abs() <= half_angle
    }

    /// Clamp angle to turret's rotation limits
    pub fn clamp_angle(&self, angle: f32) -> f32 {
        let half_angle = self.turn_angle / 2.0;
        angle.clamp(-half_angle, half_angle)
    }
}

/// Marker component for turret visual entity
#[derive(Component)]
pub struct TurretVisual {
    pub parent_unit: Entity,
}

/// Plugin for turret systems
pub struct TurretPlugin;

impl Plugin for TurretPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            turret_aiming_system,
            turret_rotation_system,
            update_turret_visual_system,
        ));
    }
}

/// System to set turret target angles based on attack targets
fn turret_aiming_system(
    mut units: Query<
        (&Transform, &mut Turret, &AttackState),
        With<Unit>
    >,
    targets: Query<&Transform, With<Unit>>,
) {
    for (unit_transform, mut turret, attack_state) in units.iter_mut() {
        // Only aim during Aiming and Reloading phases
        if !matches!(attack_state.phase, AttackPhase::Aiming | AttackPhase::Reloading) {
            turret.target_angle = None;
            continue;
        }

        // Check if we have a target
        let Some(target_entity) = attack_state.target else {
            turret.target_angle = None;
            continue;
        };

        // Get target position
        let Ok(target_transform) = targets.get(target_entity) else {
            turret.target_angle = None;
            continue;
        };

        // Calculate direction to target
        let to_target = target_transform.translation - unit_transform.translation;
        let direction_2d = Vec2::new(to_target.x, to_target.z).normalize();

        // Calculate target angle in world space
        let target_world_angle = direction_2d.y.atan2(direction_2d.x);

        // Get unit's forward direction (yaw rotation)
        let unit_forward = unit_transform.forward();
        let unit_direction_2d = Vec2::new(unit_forward.x, unit_forward.z).normalize();
        let unit_world_angle = unit_direction_2d.y.atan2(unit_direction_2d.x);

        // Calculate relative angle (turret angle relative to unit base)
        let mut relative_angle = target_world_angle - unit_world_angle;

        // Normalize to [-PI, PI]
        while relative_angle > std::f32::consts::PI {
            relative_angle -= std::f32::consts::PI * 2.0;
        }
        while relative_angle < -std::f32::consts::PI {
            relative_angle += std::f32::consts::PI * 2.0;
        }

        // Clamp to turret limits
        let clamped_angle = turret.clamp_angle(relative_angle);

        turret.target_angle = Some(clamped_angle);
    }
}

/// System to rotate turrets toward their target angles
fn turret_rotation_system(
    time: Res<Time>,
    mut turrets: Query<&mut Turret, With<Unit>>,
) {
    let delta = time.delta_seconds();

    for mut turret in turrets.iter_mut() {
        let Some(target_angle) = turret.target_angle else {
            continue;
        };

        let angle_diff = target_angle - turret.current_angle;

        // Rotate toward target at turn_rate
        if angle_diff.abs() < 0.01 {
            // Close enough
            turret.current_angle = target_angle;
        } else {
            let rotation_amount = turret.turn_rate * delta;
            let rotation_step = angle_diff.signum() * rotation_amount.min(angle_diff.abs());
            turret.current_angle += rotation_step;
        }
    }
}

/// System to update visual turret entity rotations
fn update_turret_visual_system(
    units: Query<(&Turret, &Children), With<Unit>>,
    mut turret_visuals: Query<&mut Transform, With<TurretVisual>>,
) {
    for (turret, children) in units.iter() {
        // Find turret visual child
        for &child in children.iter() {
            if let Ok(mut turret_transform) = turret_visuals.get_mut(child) {
                // Set turret rotation around Y axis
                turret_transform.rotation = Quat::from_rotation_y(turret.current_angle);
            }
        }
    }
}

/// Helper function to create turret based on unit base type
pub fn create_turret_for_unit(unit_base: &UnitBase) -> Option<Turret> {
    match unit_base {
        UnitBase::LightInfantry => None, // No turret
        UnitBase::WheeledVehicle { .. } => Some(Turret::full_rotation(
            std::f32::consts::PI // 180°/sec
        )),
        UnitBase::TrackedVehicle { .. } => Some(Turret::full_rotation(
            std::f32::consts::PI * 2.0 / 3.0 // 120°/sec
        )),
        UnitBase::DrillUnit { .. } => Some(Turret::full_rotation(
            std::f32::consts::PI * 0.75 // 135°/sec
        )),
        UnitBase::HoverVehicle { .. } => Some(Turret::limited_rotation(
            std::f32::consts::PI / 3.0, // 60° arc (narrow)
            std::f32::consts::PI * 0.8 // 144°/sec
        )),
        UnitBase::Mech { .. } => Some(Turret::limited_rotation(
            std::f32::consts::PI / 4.0, // 45° arc (very narrow)
            std::f32::consts::PI * 0.6 // 108°/sec
        )),
    }
}

/// Spawn visual turret entity as child of unit
pub fn spawn_turret_visual(
    commands: &mut Commands,
    parent_entity: Entity,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    unit_base: &UnitBase,
    owner_color: Color,
) {
    // Only spawn turret visual for units with turrets
    if create_turret_for_unit(unit_base).is_none() {
        return;
    }

    // Create turret mesh (small cylinder on top)
    let turret_mesh = meshes.add(Cylinder::new(0.2, 0.3));
    let turret_material = materials.add(StandardMaterial {
        base_color: owner_color,
        metallic: 0.5,
        ..default()
    });

    // Spawn turret as child of unit
    let turret_entity = commands.spawn((
        PbrBundle {
            mesh: turret_mesh,
            material: turret_material,
            transform: Transform::from_xyz(0.0, 0.3, 0.0), // Offset above base
            ..default()
        },
        TurretVisual {
            parent_unit: parent_entity,
        },
    )).id();

    // Add turret as child of unit
    commands.entity(parent_entity).add_child(turret_entity);
}
