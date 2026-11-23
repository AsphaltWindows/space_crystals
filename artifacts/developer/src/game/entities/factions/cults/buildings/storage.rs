//! Cults Storage entity definition — passive space crystal drop-off point.

use bevy::prelude::*;

use crate::types::{ObjectEnum, FactionEnum, SymmetryTypeEnum};
use crate::game::entities::types::{
    EntityDefinition, StructureDefinition, BaseStats, StructureStats, CommandSlotDef, SpawnParams,
};
use crate::game::types::objects::ObjectType;
use crate::game::types::structures::{
    StructureCost, ConstructionHP, CultsConstructionState,
};
use super::types::*;
use crate::game::types::ObjectInstance;
use crate::game::types::StructureInstance;
use crate::shared::types::{Selectable, SelectionBounds, GridPosition, SightRange};

/// Cults Storage entity definition marker.
pub struct CultsStorageDef;

impl EntityDefinition for CultsStorageDef {
    const OBJECT: ObjectEnum = ObjectEnum::CultsStorage;

    fn object_type() -> ObjectType {
        ObjectType {
            name: "Storage".to_string(),
            size: (3, 2),
            destructible: true,
            sight_range: 3,
            groupable: true,
        }
    }

    fn base_stats() -> BaseStats {
        BaseStats {
            max_hp: STORAGE_MAX_HP as u32,
            point_armor: STORAGE_POINT_ARMOR,
            full_armor: STORAGE_FULL_ARMOR,
            faction: FactionEnum::TheCults,
        }
    }

    fn cost() -> Option<StructureCost> {
        Some(StructureCost {
            space_crystals: 0,
            build_frames: STORAGE_BUILD_FRAMES,
        })
    }

    fn command_slots() -> &'static [CommandSlotDef] {
        // Storage is an inert drop-off point — no command panel buttons
        &[]
    }

    fn spawn(commands: &mut EntityCommands, params: SpawnParams) {
        let stats = Self::base_stats();
        let rotation = params.rotation.unwrap_or_default();

        let obj = if params.under_construction {
            ObjectInstance::under_construction(ObjectEnum::CultsStorage, stats.max_hp as f32)
        } else {
            ObjectInstance::destructible(ObjectEnum::CultsStorage, stats.max_hp as f32)
        };

        let (rot_x, rot_z) = crate::game::world::utils::rotated_building_size(3, 2, &rotation);

        commands.insert((
            Transform::from_translation(params.position)
                .with_rotation(bevy::math::Quat::from_rotation_y(rotation.radians()))
                .with_scale(Vec3::new(
                    if params.flip_horizontal { -1.0 } else { 1.0 },
                    1.0,
                    if params.flip_vertical { -1.0 } else { 1.0 },
                )),
            obj,
            StructureInstance { rotation, flip_horizontal: params.flip_horizontal, flip_vertical: params.flip_vertical },
            params.owner,
            Selectable,
            SelectionBounds::from_dimensions(rot_x as f32, 0.6, rot_z as f32),
            GridPosition {
                x: ((params.position.x - 0.5) + 32.0) as i32,
                z: ((params.position.z - 0.5) + 32.0) as i32,
            },
            SightRange(3),
        ));

        if params.under_construction {
            commands.insert((
                ConstructionHP::new(STORAGE_BUILD_FRAMES),
                CultsConstructionState::new(STORAGE_BUILD_FRAMES),
            ));
        }
    }
}

impl StructureDefinition for CultsStorageDef {
    fn structure_stats() -> StructureStats {
        StructureStats {
            power_value: 0,
            build_radius: 0,
            symmetry: SymmetryTypeEnum::ABAB,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::entities::types::EntityRegistry;

    #[test]
    fn storage_base_stats() {
        let stats = CultsStorageDef::base_stats();
        assert_eq!(stats.max_hp, 200);
        assert_eq!(stats.point_armor, 1);
        assert_eq!(stats.full_armor, 4);
        assert_eq!(stats.faction, FactionEnum::TheCults);
    }

    #[test]
    fn storage_object_type() {
        let ot = CultsStorageDef::object_type();
        assert_eq!(ot.name, "Storage");
        assert_eq!(ot.size, (3, 2));
        assert!(ot.destructible);
        assert_eq!(ot.sight_range, 3);
        assert!(ot.groupable);
    }

    #[test]
    fn storage_structure_stats() {
        let ss = CultsStorageDef::structure_stats();
        assert_eq!(ss.power_value, 0);
        assert_eq!(ss.build_radius, 0);
        assert_eq!(ss.symmetry, SymmetryTypeEnum::ABAB);
    }

    #[test]
    fn storage_cost() {
        let cost = CultsStorageDef::cost().expect("Storage has a cost");
        assert_eq!(cost.space_crystals, 0);
        assert_eq!(cost.build_frames, 300);
    }

    #[test]
    fn storage_command_slots_empty() {
        assert!(CultsStorageDef::command_slots().is_empty());
    }

    #[test]
    fn storage_registry_lookup() {
        let mut registry = EntityRegistry::new();
        registry.register_structure::<CultsStorageDef>();
        let entry = registry.get(&ObjectEnum::CultsStorage);
        assert!(entry.is_some());
        let entry = entry.unwrap();
        assert_eq!(entry.object_type.name, "Storage");
        assert_eq!(entry.base_stats.max_hp, 200);
        assert!(entry.cost.is_some());
        assert_eq!(entry.cost.as_ref().unwrap().space_crystals, 0);
        assert_eq!(entry.cost.as_ref().unwrap().build_frames, 300);
    }

    #[test]
    fn storage_object_enum() {
        assert_eq!(CultsStorageDef::OBJECT, ObjectEnum::CultsStorage);
    }
}
