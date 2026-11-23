//! Utility functions for the entity definition system.

use bevy::prelude::*;
use crate::types::{ObjectEnum, Owner, SymmetryTypeEnum};

/// Insert visual components (Mesh3d, MeshMaterial3d, and child labels) for a spawned entity.
///
/// Entity definition `spawn()` functions insert gameplay components but not visuals.
/// This function adds the appropriate mesh, material, and structure labels based on
/// the entity's `ObjectEnum` type. Must be called after `EntityRegistry::spawn()`.
pub fn insert_entity_visuals(
    commands: &mut Commands,
    entity: Entity,
    object: &ObjectEnum,
    owner: &Owner,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    match object {
        // === Units ===
        ObjectEnum::Peacekeeper => {
            let mesh = meshes.add(Capsule3d::new(0.2, 0.6));
            let material = materials.add(StandardMaterial {
                base_color: owner.color(),
                ..default()
            });
            commands.entity(entity).insert((Mesh3d(mesh), MeshMaterial3d(material)));
        }
        ObjectEnum::SyndicateAgent => {
            let mesh = meshes.add(Capsule3d::new(0.28, 0.8));
            let material = materials.add(StandardMaterial {
                base_color: owner.color(),
                ..default()
            });
            commands.entity(entity).insert((Mesh3d(mesh), MeshMaterial3d(material)));
        }
        ObjectEnum::SyndicateGuard => {
            let mesh = meshes.add(Capsule3d::new(0.28, 0.8));
            let material = materials.add(StandardMaterial {
                base_color: owner.color(),
                ..default()
            });
            commands.entity(entity).insert((Mesh3d(mesh), MeshMaterial3d(material)));
        }
        ObjectEnum::SupplyChopper => {
            let mesh = meshes.add(Cuboid::new(0.6, 0.3, 0.6));
            let material = materials.add(StandardMaterial {
                base_color: owner.color(),
                ..default()
            });
            commands.entity(entity).insert((Mesh3d(mesh), MeshMaterial3d(material)));
        }
        ObjectEnum::CultsRecruit => {
            let mesh = meshes.add(Capsule3d::new(0.15, 0.5));
            let material = materials.add(StandardMaterial {
                base_color: Color::srgb(0.5, 0.2, 0.6),
                ..default()
            });
            commands.entity(entity).insert((Mesh3d(mesh), MeshMaterial3d(material)));
        }
        ObjectEnum::CultsSoldier => {
            let mesh = meshes.add(Capsule3d::new(0.18, 0.55));
            let material = materials.add(StandardMaterial {
                base_color: Color::srgb(0.6, 0.15, 0.5),
                ..default()
            });
            commands.entity(entity).insert((Mesh3d(mesh), MeshMaterial3d(material)));
        }
        ObjectEnum::CultsGunner => {
            let mesh = meshes.add(Capsule3d::new(0.16, 0.5));
            let material = materials.add(StandardMaterial {
                base_color: Color::srgb(0.45, 0.1, 0.65),
                ..default()
            });
            commands.entity(entity).insert((Mesh3d(mesh), MeshMaterial3d(material)));
        }

        // === GDO Structures ===
        ObjectEnum::DeploymentCenter => {
            insert_structure_visuals(
                commands, entity, meshes, materials,
                Cuboid::new(4.0, 1.5, 4.0), owner.color(), 0.6, 0.3, false,
                "Deployment Center", 1.05, Some(SymmetryTypeEnum::AAAA), 2.0, 2.0, 1.5,
            );
        }
        ObjectEnum::PowerPlant => {
            insert_structure_visuals(
                commands, entity, meshes, materials,
                Cuboid::new(2.0, 1.0, 2.0), Color::srgb(0.8, 0.8, 0.2), 0.4, 0.4, false,
                "Power Plant", 0.8, Some(SymmetryTypeEnum::AAAA), 1.0, 1.0, 1.0,
            );
        }
        ObjectEnum::Barracks => {
            insert_structure_visuals(
                commands, entity, meshes, materials,
                Cuboid::new(3.0, 0.8, 2.0), Color::srgb(0.3, 0.6, 0.3), 0.3, 0.4, false,
                "Barracks", 0.7, Some(SymmetryTypeEnum::ABAC), 1.5, 1.0, 0.8,
            );
        }
        ObjectEnum::ExtractionFacility => {
            insert_structure_visuals(
                commands, entity, meshes, materials,
                Cuboid::new(3.0, 1.2, 3.0), Color::srgb(0.6, 0.4, 0.2), 0.5, 0.4, false,
                "Extraction Facility", 0.9, Some(SymmetryTypeEnum::AAAA), 1.5, 1.5, 1.2,
            );
        }
        ObjectEnum::ExtractionPlate => {
            insert_structure_visuals(
                commands, entity, meshes, materials,
                Cuboid::new(0.8, 0.15, 0.8), Color::srgb(0.4, 0.5, 0.7), 0.7, 0.2, false,
                "Extraction Plate", 0.4, None, 0.0, 0.0, 0.0,
            );
        }
        ObjectEnum::SupplyTower => {
            insert_structure_visuals(
                commands, entity, meshes, materials,
                Cuboid::new(3.0, 1.2, 3.0), Color::srgb(0.5, 0.5, 0.8), 0.5, 0.3, false,
                "Supply Tower", 0.9, Some(SymmetryTypeEnum::AAAA), 1.5, 1.5, 1.2,
            );
        }

        // === Syndicate Structures ===
        ObjectEnum::Tunnel => {
            insert_structure_visuals(
                commands, entity, meshes, materials,
                Cuboid::new(4.0, 1.0, 4.0), Color::srgb(0.5, 0.2, 0.2), 0.5, 0.4, false,
                "Tunnel", 0.8, Some(SymmetryTypeEnum::ABCD), 2.0, 2.0, 1.0,
            );
        }
        ObjectEnum::Headquarters => {
            insert_structure_visuals(
                commands, entity, meshes, materials,
                Cuboid::new(2.0, 1.0, 2.0), Color::srgb(0.7, 0.2, 0.6), 0.5, 0.3, false,
                "Headquarters", 0.8, None, 0.0, 0.0, 0.0,
            );
        }

        // === Cults Structures ===
        ObjectEnum::RecruitmentCenter => {
            insert_structure_visuals(
                commands, entity, meshes, materials,
                Cuboid::new(4.0, 1.5, 4.0), Color::srgb(0.5, 0.2, 0.6), 0.6, 0.3, false,
                "Recruitment Center", 1.05, Some(SymmetryTypeEnum::AAAA), 2.0, 2.0, 1.5,
            );
        }
        ObjectEnum::CultsStorage => {
            insert_structure_visuals(
                commands, entity, meshes, materials,
                Cuboid::new(3.0, 0.6, 2.0), Color::srgb(0.4, 0.15, 0.5), 0.3, 0.4, false,
                "Storage", 0.5, Some(SymmetryTypeEnum::ABAB), 1.5, 1.0, 0.8,
            );
        }
        ObjectEnum::CultsArmory => {
            insert_structure_visuals(
                commands, entity, meshes, materials,
                Cuboid::new(3.0, 0.8, 2.0), Color::srgb(0.55, 0.2, 0.55), 0.3, 0.4, false,
                "Armory", 0.6, Some(SymmetryTypeEnum::ABCB), 1.5, 1.0, 0.8,
            );
        }

        // Resource entities and others — no visuals needed via this path
        _ => {}
    }
}

/// Helper to insert structure visual components and child labels.
#[allow(clippy::too_many_arguments)]
fn insert_structure_visuals(
    commands: &mut Commands,
    entity: Entity,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    cuboid: Cuboid,
    base_color: Color,
    metallic: f32,
    roughness: f32,
    _is_flipped: bool,
    label_name: &'static str,
    label_height: f32,
    symmetry: Option<SymmetryTypeEnum>,
    half_x: f32,
    half_z: f32,
    structure_height: f32,
) {
    let mesh = meshes.add(cuboid);
    let material = materials.add(StandardMaterial {
        base_color,
        metallic,
        perceptual_roughness: roughness,
        cull_mode: None, // Always double-sided to support flipped structures
        ..default()
    });

    commands.entity(entity).insert((Mesh3d(mesh), MeshMaterial3d(material)));

    commands.entity(entity).with_children(|parent| {
        // Structure name label
        parent.spawn((
            Text2d::new(label_name),
            TextFont { font_size: 36.0, ..default() },
            TextColor(Color::WHITE),
            TextLayout::new_with_justify(Justify::Center),
            Transform::from_xyz(0.0, label_height, 0.0)
                .with_scale(Vec3::splat(0.01)),
            crate::game::types::objects::StructureLabel,
        ));
        // Side labels
        if let Some(sym) = symmetry {
            crate::game::utils::spawn_ghost_side_labels(parent, sym, half_x, half_z, structure_height);
        }
    });
}

/// Compute world-space position from grid coordinates for entity spawning.
/// For 1x1 units: center is at (grid - 32) + 0.5.
/// For NxM structures: center is at (grid - 32) + N/2.
pub fn grid_to_spawn_position(grid_x: i32, grid_z: i32, size_x: f32, size_z: f32) -> Vec3 {
    let world_x = (grid_x as f32 - 32.0) + size_x / 2.0;
    let world_z = (grid_z as f32 - 32.0) + size_z / 2.0;
    Vec3::new(world_x, 0.0, world_z)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_to_spawn_position_unit_center() {
        // A 1x1 unit at grid (32, 32) should be at world (0.5, 0.0, 0.5)
        let pos = grid_to_spawn_position(32, 32, 1.0, 1.0);
        assert_eq!(pos, Vec3::new(0.5, 0.0, 0.5));
    }

    #[test]
    fn grid_to_spawn_position_4x4_structure() {
        // A 4x4 structure at grid (30, 30) should be at world (-0.0, 0.0, -0.0)
        let pos = grid_to_spawn_position(30, 30, 4.0, 4.0);
        assert_eq!(pos, Vec3::new(0.0, 0.0, 0.0));
    }

    #[test]
    fn grid_to_spawn_position_2x2_structure() {
        // A 2x2 structure at grid (33, 33) should be at world (2.0, 0.0, 2.0)
        let pos = grid_to_spawn_position(33, 33, 2.0, 2.0);
        assert_eq!(pos, Vec3::new(2.0, 0.0, 2.0));
    }

    #[test]
    fn registry_spawn_peacekeeper_creates_entity_with_components() {
        use crate::game::entities::registry::build_registry;
        use crate::game::entities::SpawnParams;
        use crate::game::types::ObjectInstance;
        use bevy::ecs::system::RunSystemOnce;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<StandardMaterial>>();
        app.insert_resource(build_registry());

        let entity = app.world_mut().run_system_once(
            |mut commands: Commands,
             mut meshes: ResMut<Assets<Mesh>>,
             mut materials: ResMut<Assets<StandardMaterial>>,
             registry: Res<crate::game::entities::EntityRegistry>| {
                let pos = grid_to_spawn_position(32, 32, 1.0, 1.0);
                let owner = Owner::player(0);
                let entity = registry.spawn(&ObjectEnum::Peacekeeper, &mut commands, SpawnParams {
                    position: pos,
                    owner,
                    player: 0,
                    rotation: None,
                    under_construction: false,
                    flip_horizontal: false,
                    flip_vertical: false,
                });
                insert_entity_visuals(&mut commands, entity, &ObjectEnum::Peacekeeper, &owner, &mut meshes, &mut materials);
                entity
            },
        ).unwrap();

        app.world_mut().flush();

        // Verify entity has expected components
        let obj = app.world().get::<ObjectInstance>(entity).unwrap();
        assert_eq!(obj.object_type, ObjectEnum::Peacekeeper);
        assert!(obj.is_destructible());

        // Verify visual components
        assert!(app.world().get::<Mesh3d>(entity).is_some(), "Should have Mesh3d");
        assert!(app.world().get::<MeshMaterial3d<StandardMaterial>>(entity).is_some(), "Should have MeshMaterial3d");
    }

    #[test]
    fn registry_spawn_structure_with_flip() {
        use crate::game::entities::registry::build_registry;
        use crate::game::entities::SpawnParams;
        use crate::game::types::{ObjectInstance, StructureInstance};
        use bevy::ecs::system::RunSystemOnce;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<StandardMaterial>>();
        app.insert_resource(build_registry());

        let entity = app.world_mut().run_system_once(
            |mut commands: Commands,
             mut meshes: ResMut<Assets<Mesh>>,
             mut materials: ResMut<Assets<StandardMaterial>>,
             registry: Res<crate::game::entities::EntityRegistry>| {
                let pos = grid_to_spawn_position(30, 30, 2.0, 2.0);
                let owner = Owner::player(0);
                let entity = registry.spawn(&ObjectEnum::PowerPlant, &mut commands, SpawnParams {
                    position: pos,
                    owner,
                    player: 0,
                    rotation: Some(crate::types::StructureRotation::R0),
                    under_construction: false,
                    flip_horizontal: true,
                    flip_vertical: false,
                });
                insert_entity_visuals(&mut commands, entity, &ObjectEnum::PowerPlant, &owner, &mut meshes, &mut materials);
                entity
            },
        ).unwrap();

        app.world_mut().flush();

        // Verify entity has expected components
        let obj = app.world().get::<ObjectInstance>(entity).unwrap();
        assert_eq!(obj.object_type, ObjectEnum::PowerPlant);

        let si = app.world().get::<StructureInstance>(entity).unwrap();
        assert!(si.flip_horizontal, "Flip horizontal should be true");
        assert!(!si.flip_vertical, "Flip vertical should be false");

        // Verify transform has flip scale
        let transform = app.world().get::<Transform>(entity).unwrap();
        assert_eq!(transform.scale.x, -1.0, "X scale should be -1 for flip_horizontal");
    }
}
