use bevy::prelude::*;
use crate::types::*;
use super::types::objects::StructureLabel;

/// Billboard system: rotates all StructureLabel entities to face the main camera.
/// Labels are children of structure entities, so we use GlobalTransform for world position.
pub fn billboard_label_system(
    camera_query: Query<&GlobalTransform, With<MainCamera>>,
    mut label_query: Query<(&mut Transform, &GlobalTransform), (With<StructureLabel>, Without<MainCamera>)>,
) {
    let Ok(camera_global) = camera_query.single() else { return };
    let camera_pos: Vec3 = camera_global.translation();

    for (mut label_transform, label_global) in label_query.iter_mut() {
        let label_world_pos = label_global.translation();
        let direction = camera_pos - label_world_pos;
        if direction.length_squared() > 0.001 {
            // Face towards camera — use full direction for proper billboard tilt
            let scale = label_transform.scale;
            label_transform.look_to(direction.normalize(), Vec3::Y);
            label_transform.scale = scale;
        }
    }
}

/// Get the side label characters for a given symmetry type.
/// Returns [North(+Z), East(+X), South(-Z), West(-X)] at R0 rotation.
fn side_labels(sym: SymmetryTypeEnum) -> [char; 4] {
    match sym {
        SymmetryTypeEnum::AAAA => ['A', 'A', 'A', 'A'],
        SymmetryTypeEnum::AAAB => ['A', 'A', 'A', 'B'],
        SymmetryTypeEnum::AABB => ['A', 'A', 'B', 'B'],
        SymmetryTypeEnum::ABAB => ['A', 'B', 'A', 'B'],
        SymmetryTypeEnum::AABC => ['A', 'A', 'B', 'C'],
        SymmetryTypeEnum::ABAC => ['A', 'B', 'A', 'C'],
        SymmetryTypeEnum::ABCB => ['A', 'B', 'C', 'B'],
        SymmetryTypeEnum::ABCD => ['A', 'B', 'C', 'D'],
    }
}

/// Spawn side labels (A/B/C/D) on a building's faces as children.
/// Labels are positioned at the midpoint of each face.
/// The building is centered at local (0,0,0) with given half-extents.
/// Since labels are children, they rotate with the parent automatically.
fn spawn_side_labels(
    parent: &mut ChildSpawnerCommands,
    sym: SymmetryTypeEnum,
    half_x: f32,
    half_z: f32,
    height: f32,
) {
    let labels = side_labels(sym);
    // Skip AAAA buildings (all sides identical — labels unnecessary)
    if sym == SymmetryTypeEnum::AAAA {
        return;
    }

    let label_y = height * 0.5; // Mid-height of the building
    let offset = 0.02; // Slight offset from face to avoid z-fighting

    // Side positions in local space: [North(+Z), East(+X), South(-Z), West(-X)]
    let positions = [
        Vec3::new(0.0, label_y, half_z + offset),    // North (+Z)
        Vec3::new(half_x + offset, label_y, 0.0),    // East (+X)
        Vec3::new(0.0, label_y, -(half_z + offset)),  // South (-Z)
        Vec3::new(-(half_x + offset), label_y, 0.0),  // West (-X)
    ];

    // Rotation for each label to face outward from the building
    let rotations = [
        Quat::IDENTITY,                                          // North: face +Z
        Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2),     // East: face +X
        Quat::from_rotation_y(std::f32::consts::PI),             // South: face -Z
        Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),     // West: face -X
    ];

    for i in 0..4 {
        let label_char = labels[i];
        let is_b_side = label_char == 'B';

        let text_color = if is_b_side {
            Color::srgb(1.0, 0.8, 0.2) // Highlighted yellow for B side (unit exit)
        } else {
            Color::srgba(0.9, 0.9, 0.9, 0.8)
        };

        let font_size = if is_b_side { 48.0 } else { 36.0 };

        parent.spawn((
            Text2d::new(label_char.to_string()),
            TextFont { font_size, ..default() },
            TextColor(text_color),
            TextLayout::new_with_justify(Justify::Center),
            Transform::from_translation(positions[i])
                .with_rotation(rotations[i])
                .with_scale(Vec3::splat(0.008)),
            StructureLabel, // Reuse for billboard system
        ));
    }
}

/// Public interface for spawning side labels on ghost preview entities.
pub fn spawn_ghost_side_labels(
    parent: &mut ChildSpawnerCommands,
    sym: SymmetryTypeEnum,
    half_x: f32,
    half_z: f32,
    height: f32,
) {
    spawn_side_labels(parent, sym, half_x, half_z, height);
}
