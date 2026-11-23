//! Extraction Facility and Extraction Plate entity definitions — GDO resource extraction structures.

use bevy::prelude::*;

use crate::types::{ObjectEnum, FactionEnum, SymmetryTypeEnum};
use crate::game::entities::types::{
    EntityDefinition, StructureDefinition, BaseStats, StructureStats, CommandSlotDef, SpawnParams,
    SlotCondition,
};
use crate::game::types::objects::ObjectType;
use crate::game::types::structures::{
    StructureCost, PowerValue, BuildRadiusExtension,
    ExtractionFacilityState, ExtractionPlateState,
};
use super::types::*;
use crate::game::types::ObjectInstance;
use crate::shared::types::{
    Selectable, SelectionBounds, GridPosition, SightRange,
};
use crate::game::types::StructureInstance;
use crate::ui::types::{ObjectInterfaceState, StructureMenuState, CommandButtonAction};

// === Extraction Facility ===

/// Command slots for Extraction Facility.
static EF_COMMAND_SLOTS: &[CommandSlotDef] = &[
    // EnterPlacement when plate is ready (takes priority over EfBuildPlate at same position)
    CommandSlotDef::with_condition(ObjectInterfaceState::StructureMenu(StructureMenuState::EfIdle), 0, 0, CommandButtonAction::EnterPlacement, SlotCondition::HasReadyPlate),
    // EfBuildPlate — fallback when no plate is ready
    CommandSlotDef::new(ObjectInterfaceState::StructureMenu(StructureMenuState::EfIdle), 0, 0, CommandButtonAction::EfBuildPlate),
    // Cancel — conditional on has_active_construction
    CommandSlotDef::with_condition(ObjectInterfaceState::StructureMenu(StructureMenuState::EfIdle), 2, 1, CommandButtonAction::EfCancel, SlotCondition::HasActiveConstruction),
];

/// Extraction Facility entity definition marker.
pub struct ExtractionFacility;

impl EntityDefinition for ExtractionFacility {
    const OBJECT: ObjectEnum = ObjectEnum::ExtractionFacility;

    fn object_type() -> ObjectType {
        ObjectType {
            name: "Extraction Facility".to_string(),
            size: (3, 3),
            destructible: true,
            sight_range: 3,
            groupable: false,
        }
    }

    fn base_stats() -> BaseStats {
        BaseStats {
            max_hp: EF_MAX_HP as u32,
            point_armor: EF_POINT_ARMOR,
            full_armor: EF_FULL_ARMOR,
            faction: FactionEnum::GlobalDefenseOrdinance,
        }
    }

    fn cost() -> Option<StructureCost> {
        Some(StructureCost {
            space_crystals: 200,
            build_frames: 320,
        })
    }

    fn command_slots() -> &'static [CommandSlotDef] {
        EF_COMMAND_SLOTS
    }

    fn spawn(commands: &mut EntityCommands, params: SpawnParams) {
        let stats = Self::base_stats();
        let rotation = params.rotation.unwrap_or_default();
        let obj = if params.under_construction {
            ObjectInstance::under_construction(ObjectEnum::ExtractionFacility, stats.max_hp as f32)
        } else {
            ObjectInstance::destructible(ObjectEnum::ExtractionFacility, stats.max_hp as f32)
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
            PowerValue(EF_POWER),
            BuildRadiusExtension(EF_BUILD_RADIUS),
            ExtractionFacilityState::default(),
            SightRange(3),
        ));
    }
}

impl StructureDefinition for ExtractionFacility {
    fn structure_stats() -> StructureStats {
        StructureStats {
            power_value: EF_POWER,
            build_radius: EF_BUILD_RADIUS,
            symmetry: SymmetryTypeEnum::AAAA,
        }
    }
}

// === Extraction Plate ===

/// Extraction Plate entity definition marker.
pub struct ExtractionPlate;

impl EntityDefinition for ExtractionPlate {
    const OBJECT: ObjectEnum = ObjectEnum::ExtractionPlate;

    fn object_type() -> ObjectType {
        ObjectType {
            name: "Extraction Plate".to_string(),
            size: (1, 1),
            destructible: true,
            sight_range: 0,
            groupable: true,
        }
    }

    fn base_stats() -> BaseStats {
        BaseStats {
            max_hp: EP_MAX_HP as u32,
            point_armor: EP_POINT_ARMOR,
            full_armor: EP_FULL_ARMOR,
            faction: FactionEnum::GlobalDefenseOrdinance,
        }
    }

    fn cost() -> Option<StructureCost> {
        // Extraction Plate is built by Extraction Facility, not DC
        Some(StructureCost {
            space_crystals: 75,
            build_frames: 96,
        })
    }

    fn command_slots() -> &'static [CommandSlotDef] {
        // Extraction Plate is inert — no command slots
        &[]
    }

    fn spawn(commands: &mut EntityCommands, params: SpawnParams) {
        let stats = Self::base_stats();
        let obj = if params.under_construction {
            ObjectInstance::under_construction(ObjectEnum::ExtractionPlate, stats.max_hp as f32)
        } else {
            ObjectInstance::destructible(ObjectEnum::ExtractionPlate, stats.max_hp as f32)
        };

        // Note: ExtractionPlateState requires an attached_patch Entity.
        // SpawnParams doesn't carry this; use Entity::PLACEHOLDER as default.
        // Callers that need a real attached_patch should insert ExtractionPlateState
        // separately after spawn.
        commands.insert((
            Transform::from_translation(params.position),
            obj,
            StructureInstance::default(),
            params.owner,
            Selectable,
            SelectionBounds::from_dimensions(0.8, 0.15, 0.8),
            GridPosition {
                x: ((params.position.x - 0.5) + 32.0) as i32,
                z: ((params.position.z - 0.5) + 32.0) as i32,
            },
            BuildRadiusExtension(EP_BUILD_RADIUS),
            PowerValue(EP_POWER),
            ExtractionPlateState {
                attached_patch: Entity::PLACEHOLDER,
                mining_timer: 0.0,
            },
        ));
    }
}

impl StructureDefinition for ExtractionPlate {
    fn structure_stats() -> StructureStats {
        StructureStats {
            power_value: EP_POWER,
            build_radius: EP_BUILD_RADIUS,
            symmetry: SymmetryTypeEnum::AAAA,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::entities::types::EntityRegistry;

    // === Extraction Facility Tests ===

    #[test]
    fn ef_base_stats() {
        let stats = ExtractionFacility::base_stats();
        assert_eq!(stats.max_hp, 500);
        assert_eq!(stats.point_armor, 1);
        assert_eq!(stats.full_armor, 9);
        assert_eq!(stats.faction, FactionEnum::GlobalDefenseOrdinance);
    }

    #[test]
    fn ef_object_type() {
        let ot = ExtractionFacility::object_type();
        assert_eq!(ot.name, "Extraction Facility");
        assert_eq!(ot.size, (3, 3));
        assert!(ot.destructible);
        assert_eq!(ot.sight_range, 3);
        assert!(!ot.groupable);
    }

    #[test]
    fn ef_structure_stats() {
        let ss = ExtractionFacility::structure_stats();
        assert_eq!(ss.power_value, -15);
        assert_eq!(ss.build_radius, 2);
        assert_eq!(ss.symmetry, SymmetryTypeEnum::AAAA);
    }

    #[test]
    fn ef_cost() {
        let cost = ExtractionFacility::cost().expect("EF has a cost");
        assert_eq!(cost.space_crystals, 200);
        assert_eq!(cost.build_frames, 320);
    }

    #[test]
    fn ef_command_slots_count() {
        let slots = ExtractionFacility::command_slots();
        assert_eq!(slots.len(), 3); // EnterPlacement (conditional) + EfBuildPlate + EfCancel (conditional)
    }

    #[test]
    fn ef_command_slots_build_plate() {
        let slots = ExtractionFacility::command_slots();
        let slot = slots.iter().find(|s| matches!(s.action, CommandButtonAction::EfBuildPlate));
        assert!(slot.is_some());
        let s = slot.unwrap();
        assert_eq!(s.row, 0);
        assert_eq!(s.col, 0);
    }

    #[test]
    fn ef_command_slots_cancel() {
        let slots = ExtractionFacility::command_slots();
        let slot = slots.iter().find(|s| matches!(s.action, CommandButtonAction::EfCancel));
        assert!(slot.is_some());
        let s = slot.unwrap();
        assert_eq!(s.row, 2);
        assert_eq!(s.col, 1);
    }

    #[test]
    fn ef_registry_lookup() {
        let mut registry = EntityRegistry::new();
        registry.register_structure::<ExtractionFacility>();
        let entry = registry.get(&ObjectEnum::ExtractionFacility);
        assert!(entry.is_some());
        let entry = entry.unwrap();
        assert_eq!(entry.object_type.name, "Extraction Facility");
        assert_eq!(entry.base_stats.max_hp, 500);
        assert!(entry.cost.is_some());
        assert_eq!(entry.cost.as_ref().unwrap().space_crystals, 200);
    }

    #[test]
    fn ef_object_enum() {
        assert_eq!(ExtractionFacility::OBJECT, ObjectEnum::ExtractionFacility);
    }

    // === Extraction Plate Tests ===

    #[test]
    fn ep_base_stats() {
        let stats = ExtractionPlate::base_stats();
        assert_eq!(stats.max_hp, 85);
        assert_eq!(stats.point_armor, 2);
        assert_eq!(stats.full_armor, 2);
        assert_eq!(stats.faction, FactionEnum::GlobalDefenseOrdinance);
    }

    #[test]
    fn ep_object_type() {
        let ot = ExtractionPlate::object_type();
        assert_eq!(ot.name, "Extraction Plate");
        assert_eq!(ot.size, (1, 1));
        assert!(ot.destructible);
        assert_eq!(ot.sight_range, 0);
        assert!(ot.groupable);
    }

    #[test]
    fn ep_structure_stats() {
        let ss = ExtractionPlate::structure_stats();
        assert_eq!(ss.power_value, -3);
        assert_eq!(ss.build_radius, 0);
        assert_eq!(ss.symmetry, SymmetryTypeEnum::AAAA);
    }

    #[test]
    fn ep_cost() {
        let cost = ExtractionPlate::cost().expect("EP has a cost");
        assert_eq!(cost.space_crystals, 75);
        assert_eq!(cost.build_frames, 96);
    }

    #[test]
    fn ep_command_slots_empty() {
        assert!(ExtractionPlate::command_slots().is_empty());
    }

    #[test]
    fn ep_registry_lookup() {
        let mut registry = EntityRegistry::new();
        registry.register_structure::<ExtractionPlate>();
        let entry = registry.get(&ObjectEnum::ExtractionPlate);
        assert!(entry.is_some());
        let entry = entry.unwrap();
        assert_eq!(entry.object_type.name, "Extraction Plate");
        assert_eq!(entry.base_stats.max_hp, 85);
        assert!(entry.cost.is_some());
        assert_eq!(entry.cost.as_ref().unwrap().space_crystals, 75);
    }

    #[test]
    fn ep_object_enum() {
        assert_eq!(ExtractionPlate::OBJECT, ObjectEnum::ExtractionPlate);
    }
}
