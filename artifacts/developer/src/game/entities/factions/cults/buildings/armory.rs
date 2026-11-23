//! Cults Armory entity definition — unit training and storage structure.

use bevy::prelude::*;

use crate::types::{ObjectEnum, FactionEnum, SymmetryTypeEnum};
use crate::game::entities::types::{
    EntityDefinition, StructureDefinition, BaseStats, StructureStats, CommandSlotDef, SpawnParams,
};
use crate::game::types::objects::ObjectType;
use crate::game::types::structures::{
    StructureCost, ArmoryState,
};
use super::types::*;
use crate::game::types::ObjectInstance;
use crate::game::types::StructureInstance;
use crate::shared::types::{Selectable, SelectionBounds, GridPosition, SightRange};
use crate::game::combat::types::Armor;
use crate::ui::types::{ObjectInterfaceState, StructureMenuState, CommandButtonAction};

/// Command slots for Armory.
static ARMORY_COMMAND_SLOTS: &[CommandSlotDef] = &[
    CommandSlotDef::new(ObjectInterfaceState::StructureMenu(StructureMenuState::ArmoryMenu), 0, 0, CommandButtonAction::ArmoryTrainSoldier),
    CommandSlotDef::new(ObjectInterfaceState::StructureMenu(StructureMenuState::ArmoryMenu), 0, 1, CommandButtonAction::ArmoryTrainGunner),
    CommandSlotDef::new(ObjectInterfaceState::StructureMenu(StructureMenuState::ArmoryMenu), 0, 2, CommandButtonAction::ArmoryEjectAll),
    CommandSlotDef::new(ObjectInterfaceState::StructureMenu(StructureMenuState::ArmoryMenu), 2, 2, CommandButtonAction::SetRallyPoint),
];

/// Cults Armory entity definition marker.
pub struct CultsArmoryDef;

impl EntityDefinition for CultsArmoryDef {
    const OBJECT: ObjectEnum = ObjectEnum::CultsArmory;

    fn object_type() -> ObjectType {
        ObjectType {
            name: "Armory".to_string(),
            size: (3, 2),
            destructible: true,
            sight_range: 3,
            groupable: true,
        }
    }

    fn base_stats() -> BaseStats {
        BaseStats {
            max_hp: ARMORY_MAX_HP as u32,
            point_armor: ARMORY_POINT_ARMOR,
            full_armor: ARMORY_FULL_ARMOR,
            faction: FactionEnum::TheCults,
        }
    }

    fn cost() -> Option<StructureCost> {
        Some(StructureCost {
            space_crystals: ARMORY_SC_COST,
            build_frames: 0,
        })
    }

    fn command_slots() -> &'static [CommandSlotDef] {
        ARMORY_COMMAND_SLOTS
    }

    fn spawn(commands: &mut EntityCommands, params: SpawnParams) {
        let stats = Self::base_stats();
        let rotation = params.rotation.unwrap_or_default();

        let obj = if params.under_construction {
            ObjectInstance::under_construction(ObjectEnum::CultsArmory, stats.max_hp as f32)
        } else {
            ObjectInstance::destructible(ObjectEnum::CultsArmory, stats.max_hp as f32)
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
            SelectionBounds::from_dimensions(rot_x as f32, 0.8, rot_z as f32),
            GridPosition {
                x: ((params.position.x - 0.5) + 32.0) as i32,
                z: ((params.position.z - 0.5) + 32.0) as i32,
            },
            SightRange(3),
            ArmoryState::default(),
            Armor { point_armor: ARMORY_POINT_ARMOR as f32, full_armor: ARMORY_FULL_ARMOR as f32, directional_armor: false },
        ));
    }
}

impl StructureDefinition for CultsArmoryDef {
    fn structure_stats() -> StructureStats {
        StructureStats {
            power_value: 0,
            build_radius: 0,
            symmetry: SymmetryTypeEnum::ABCB,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::entities::types::EntityRegistry;

    #[test]
    fn armory_base_stats() {
        let stats = CultsArmoryDef::base_stats();
        assert_eq!(stats.max_hp, 300);
        assert_eq!(stats.point_armor, 1);
        assert_eq!(stats.full_armor, 4);
        assert_eq!(stats.faction, FactionEnum::TheCults);
    }

    #[test]
    fn armory_object_type() {
        let ot = CultsArmoryDef::object_type();
        assert_eq!(ot.name, "Armory");
        assert_eq!(ot.size, (3, 2));
        assert!(ot.destructible);
        assert_eq!(ot.sight_range, 3);
        assert!(ot.groupable);
    }

    #[test]
    fn armory_structure_stats() {
        let ss = CultsArmoryDef::structure_stats();
        assert_eq!(ss.power_value, 0);
        assert_eq!(ss.build_radius, 0);
        assert_eq!(ss.symmetry, SymmetryTypeEnum::ABCB);
    }

    #[test]
    fn armory_cost() {
        let cost = CultsArmoryDef::cost().expect("Armory has a cost");
        assert_eq!(cost.space_crystals, 150);
        assert_eq!(cost.build_frames, 0);
    }

    #[test]
    fn armory_command_slots_count() {
        let slots = CultsArmoryDef::command_slots();
        assert_eq!(slots.len(), 4);
    }

    #[test]
    fn armory_command_slots_train_soldier() {
        let slots = CultsArmoryDef::command_slots();
        let slot = slots.iter().find(|s| matches!(s.action, CommandButtonAction::ArmoryTrainSoldier));
        assert!(slot.is_some());
        let s = slot.unwrap();
        assert_eq!(s.row, 0);
        assert_eq!(s.col, 0);
    }

    #[test]
    fn armory_command_slots_train_gunner() {
        let slots = CultsArmoryDef::command_slots();
        let slot = slots.iter().find(|s| matches!(s.action, CommandButtonAction::ArmoryTrainGunner));
        assert!(slot.is_some());
        let s = slot.unwrap();
        assert_eq!(s.row, 0);
        assert_eq!(s.col, 1);
    }

    #[test]
    fn armory_command_slots_eject_all() {
        let slots = CultsArmoryDef::command_slots();
        let slot = slots.iter().find(|s| matches!(s.action, CommandButtonAction::ArmoryEjectAll));
        assert!(slot.is_some());
        let s = slot.unwrap();
        assert_eq!(s.row, 0);
        assert_eq!(s.col, 2);
    }

    #[test]
    fn armory_command_slots_rally_point() {
        let slots = CultsArmoryDef::command_slots();
        let slot = slots.iter().find(|s| matches!(s.action, CommandButtonAction::SetRallyPoint));
        assert!(slot.is_some());
        let s = slot.unwrap();
        assert_eq!(s.row, 2);
        assert_eq!(s.col, 2);
    }

    #[test]
    fn armory_registry_lookup() {
        let mut registry = EntityRegistry::new();
        registry.register_structure::<CultsArmoryDef>();
        let entry = registry.get(&ObjectEnum::CultsArmory);
        assert!(entry.is_some());
        let entry = entry.unwrap();
        assert_eq!(entry.object_type.name, "Armory");
        assert_eq!(entry.base_stats.max_hp, 300);
        assert!(entry.cost.is_some());
        assert_eq!(entry.cost.as_ref().unwrap().space_crystals, 150);
    }

    #[test]
    fn armory_object_enum() {
        assert_eq!(CultsArmoryDef::OBJECT, ObjectEnum::CultsArmory);
    }
}
