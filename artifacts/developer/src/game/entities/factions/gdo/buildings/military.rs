//! Barracks and Supply Tower entity definitions — GDO military production structures.

use bevy::prelude::*;

use crate::types::{ObjectEnum, FactionEnum, SymmetryTypeEnum};
use crate::game::entities::types::{
    EntityDefinition, StructureDefinition, BaseStats, StructureStats, CommandSlotDef, SpawnParams,
    SlotCondition,
};
use crate::game::types::objects::ObjectType;
use crate::game::types::structures::{
    StructureCost, PowerValue, BuildRadiusExtension,
    BarracksState, SupplyTowerState,
};
use super::types::*;
use crate::game::types::ObjectInstance;
use crate::shared::types::{
    Selectable, SelectionBounds, GridPosition, SightRange,
};
use crate::game::types::StructureInstance;
use crate::ui::types::{ObjectInterfaceState, StructureMenuState, CommandButtonAction};

// === Barracks ===

/// Command slots for Barracks.
static BK_COMMAND_SLOTS: &[CommandSlotDef] = &[
    CommandSlotDef::new(ObjectInterfaceState::StructureMenu(StructureMenuState::BarracksMenu), 0, 0, CommandButtonAction::BkTrain(ObjectEnum::Peacekeeper)),
    // Cancel — conditional on bk_has_queue
    CommandSlotDef::with_condition(ObjectInterfaceState::StructureMenu(StructureMenuState::BarracksMenu), 2, 1, CommandButtonAction::BkCancel, SlotCondition::HasQueue),
    CommandSlotDef::new(ObjectInterfaceState::StructureMenu(StructureMenuState::BarracksMenu), 2, 2, CommandButtonAction::SetRallyPoint),
];

/// Barracks entity definition marker.
pub struct Barracks;

impl EntityDefinition for Barracks {
    const OBJECT: ObjectEnum = ObjectEnum::Barracks;

    fn object_type() -> ObjectType {
        ObjectType {
            name: "Barracks".to_string(),
            size: (3, 2),
            destructible: true,
            sight_range: 4,
            groupable: true,
        }
    }

    fn base_stats() -> BaseStats {
        BaseStats {
            max_hp: BK_MAX_HP as u32,
            point_armor: BK_POINT_ARMOR,
            full_armor: BK_FULL_ARMOR,
            faction: FactionEnum::GlobalDefenseOrdinance,
        }
    }

    fn cost() -> Option<StructureCost> {
        Some(StructureCost {
            space_crystals: 200,
            build_frames: 160,
        })
    }

    fn command_slots() -> &'static [CommandSlotDef] {
        BK_COMMAND_SLOTS
    }

    fn spawn(commands: &mut EntityCommands, params: SpawnParams) {
        let stats = Self::base_stats();
        let rotation = params.rotation.unwrap_or_default();
        let obj = if params.under_construction {
            ObjectInstance::under_construction(ObjectEnum::Barracks, stats.max_hp as f32)
        } else {
            ObjectInstance::destructible(ObjectEnum::Barracks, stats.max_hp as f32)
        };

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
            SelectionBounds::from_dimensions(3.0, 0.8, 2.0),
            GridPosition {
                x: ((params.position.x - 0.5) + 32.0) as i32,
                z: ((params.position.z - 0.5) + 32.0) as i32,
            },
            PowerValue(BK_POWER),
            BuildRadiusExtension(BK_BUILD_RADIUS),
            BarracksState::default(),
            SightRange(4),
        ));
    }
}

impl StructureDefinition for Barracks {
    fn structure_stats() -> StructureStats {
        StructureStats {
            power_value: BK_POWER,
            build_radius: BK_BUILD_RADIUS,
            symmetry: SymmetryTypeEnum::ABAC,
        }
    }
}

// === Supply Tower ===

/// Command slots for Supply Tower.
static ST_COMMAND_SLOTS: &[CommandSlotDef] = &[
    CommandSlotDef::new(ObjectInterfaceState::StructureMenu(StructureMenuState::SupplyTowerMenu), 0, 0, CommandButtonAction::StTrain(ObjectEnum::SupplyChopper)),
    CommandSlotDef::new(ObjectInterfaceState::StructureMenu(StructureMenuState::SupplyTowerMenu), 1, 0, CommandButtonAction::StScheduleDeliveries),
    // Cancel — conditional on bk_has_queue
    CommandSlotDef::with_condition(ObjectInterfaceState::StructureMenu(StructureMenuState::SupplyTowerMenu), 2, 1, CommandButtonAction::StCancel, SlotCondition::HasQueue),
    CommandSlotDef::new(ObjectInterfaceState::StructureMenu(StructureMenuState::SupplyTowerMenu), 2, 2, CommandButtonAction::SetRallyPoint),
];

/// Supply Tower entity definition marker.
pub struct SupplyTowerDef;

impl EntityDefinition for SupplyTowerDef {
    const OBJECT: ObjectEnum = ObjectEnum::SupplyTower;

    fn object_type() -> ObjectType {
        ObjectType {
            name: "Supply Tower".to_string(),
            size: (3, 3),
            destructible: true,
            sight_range: 4,
            groupable: false,
        }
    }

    fn base_stats() -> BaseStats {
        BaseStats {
            max_hp: ST_MAX_HP as u32,
            point_armor: ST_POINT_ARMOR,
            full_armor: ST_FULL_ARMOR,
            faction: FactionEnum::GlobalDefenseOrdinance,
        }
    }

    fn cost() -> Option<StructureCost> {
        Some(StructureCost {
            space_crystals: 200,
            build_frames: 240,
        })
    }

    fn command_slots() -> &'static [CommandSlotDef] {
        ST_COMMAND_SLOTS
    }

    fn spawn(commands: &mut EntityCommands, params: SpawnParams) {
        let stats = Self::base_stats();
        let rotation = params.rotation.unwrap_or_default();
        let obj = if params.under_construction {
            ObjectInstance::under_construction(ObjectEnum::SupplyTower, stats.max_hp as f32)
        } else {
            ObjectInstance::destructible(ObjectEnum::SupplyTower, stats.max_hp as f32)
        };

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
            SelectionBounds::from_dimensions(3.0, 1.2, 3.0),
            GridPosition {
                x: ((params.position.x - 0.5) + 32.0) as i32,
                z: ((params.position.z - 0.5) + 32.0) as i32,
            },
            PowerValue(ST_POWER),
            BuildRadiusExtension(ST_BUILD_RADIUS),
            SupplyTowerState::default(),
            SightRange(4),
        ));
    }
}

impl StructureDefinition for SupplyTowerDef {
    fn structure_stats() -> StructureStats {
        StructureStats {
            power_value: ST_POWER,
            build_radius: ST_BUILD_RADIUS,
            symmetry: SymmetryTypeEnum::AAAA,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::entities::types::EntityRegistry;

    // === Barracks Tests ===

    #[test]
    fn bk_base_stats() {
        let stats = Barracks::base_stats();
        assert_eq!(stats.max_hp, 300);
        assert_eq!(stats.point_armor, 1);
        assert_eq!(stats.full_armor, 6);
        assert_eq!(stats.faction, FactionEnum::GlobalDefenseOrdinance);
    }

    #[test]
    fn bk_object_type() {
        let ot = Barracks::object_type();
        assert_eq!(ot.name, "Barracks");
        assert_eq!(ot.size, (3, 2));
        assert!(ot.destructible);
        assert_eq!(ot.sight_range, 4);
        assert!(ot.groupable);
    }

    #[test]
    fn bk_structure_stats() {
        let ss = Barracks::structure_stats();
        assert_eq!(ss.power_value, -30);
        assert_eq!(ss.build_radius, 2);
        assert_eq!(ss.symmetry, SymmetryTypeEnum::ABAC);
    }

    #[test]
    fn bk_cost() {
        let cost = Barracks::cost().expect("Barracks has a cost");
        assert_eq!(cost.space_crystals, 200);
        assert_eq!(cost.build_frames, 160);
    }

    #[test]
    fn bk_command_slots_count() {
        let slots = Barracks::command_slots();
        assert_eq!(slots.len(), 3);
    }

    #[test]
    fn bk_command_slots_train_peacekeeper() {
        let slots = Barracks::command_slots();
        let slot = slots.iter().find(|s| matches!(s.action, CommandButtonAction::BkTrain(ObjectEnum::Peacekeeper)));
        assert!(slot.is_some());
        let s = slot.unwrap();
        assert_eq!(s.row, 0);
        assert_eq!(s.col, 0);
    }

    #[test]
    fn bk_command_slots_set_rally_point() {
        let slots = Barracks::command_slots();
        let slot = slots.iter().find(|s| matches!(s.action, CommandButtonAction::SetRallyPoint));
        assert!(slot.is_some());
        let s = slot.unwrap();
        assert_eq!(s.row, 2);
        assert_eq!(s.col, 2);
    }

    #[test]
    fn bk_registry_lookup() {
        let mut registry = EntityRegistry::new();
        registry.register_structure::<Barracks>();
        let entry = registry.get(&ObjectEnum::Barracks);
        assert!(entry.is_some());
        let entry = entry.unwrap();
        assert_eq!(entry.object_type.name, "Barracks");
        assert_eq!(entry.base_stats.max_hp, 300);
        assert!(entry.cost.is_some());
        assert_eq!(entry.cost.as_ref().unwrap().space_crystals, 200);
    }

    #[test]
    fn bk_object_enum() {
        assert_eq!(Barracks::OBJECT, ObjectEnum::Barracks);
    }

    // === Supply Tower Tests ===

    #[test]
    fn st_base_stats() {
        let stats = SupplyTowerDef::base_stats();
        assert_eq!(stats.max_hp, 400);
        assert_eq!(stats.point_armor, 1);
        assert_eq!(stats.full_armor, 9);
        assert_eq!(stats.faction, FactionEnum::GlobalDefenseOrdinance);
    }

    #[test]
    fn st_object_type() {
        let ot = SupplyTowerDef::object_type();
        assert_eq!(ot.name, "Supply Tower");
        assert_eq!(ot.size, (3, 3));
        assert!(ot.destructible);
        assert_eq!(ot.sight_range, 4);
        assert!(!ot.groupable);
    }

    #[test]
    fn st_structure_stats() {
        let ss = SupplyTowerDef::structure_stats();
        assert_eq!(ss.power_value, -15);
        assert_eq!(ss.build_radius, 1);
        assert_eq!(ss.symmetry, SymmetryTypeEnum::AAAA);
    }

    #[test]
    fn st_cost() {
        let cost = SupplyTowerDef::cost().expect("SupplyTower has a cost");
        assert_eq!(cost.space_crystals, 200);
        assert_eq!(cost.build_frames, 240);
    }

    #[test]
    fn st_command_slots_count() {
        let slots = SupplyTowerDef::command_slots();
        assert_eq!(slots.len(), 4);
    }

    #[test]
    fn st_command_slots_train_chopper() {
        let slots = SupplyTowerDef::command_slots();
        let slot = slots.iter().find(|s| matches!(s.action, CommandButtonAction::StTrain(ObjectEnum::SupplyChopper)));
        assert!(slot.is_some());
        let s = slot.unwrap();
        assert_eq!(s.row, 0);
        assert_eq!(s.col, 0);
    }

    #[test]
    fn st_command_slots_schedule_deliveries() {
        let slots = SupplyTowerDef::command_slots();
        let slot = slots.iter().find(|s| matches!(s.action, CommandButtonAction::StScheduleDeliveries));
        assert!(slot.is_some());
        let s = slot.unwrap();
        assert_eq!(s.row, 1);
        assert_eq!(s.col, 0);
    }

    #[test]
    fn st_command_slots_set_rally_point() {
        let slots = SupplyTowerDef::command_slots();
        let slot = slots.iter().find(|s| matches!(s.action, CommandButtonAction::SetRallyPoint));
        assert!(slot.is_some());
        let s = slot.unwrap();
        assert_eq!(s.row, 2);
        assert_eq!(s.col, 2);
    }

    #[test]
    fn st_registry_lookup() {
        let mut registry = EntityRegistry::new();
        registry.register_structure::<SupplyTowerDef>();
        let entry = registry.get(&ObjectEnum::SupplyTower);
        assert!(entry.is_some());
        let entry = entry.unwrap();
        assert_eq!(entry.object_type.name, "Supply Tower");
        assert_eq!(entry.base_stats.max_hp, 400);
        assert!(entry.cost.is_some());
        assert_eq!(entry.cost.as_ref().unwrap().space_crystals, 200);
    }

    #[test]
    fn st_object_enum() {
        assert_eq!(SupplyTowerDef::OBJECT, ObjectEnum::SupplyTower);
    }
}
