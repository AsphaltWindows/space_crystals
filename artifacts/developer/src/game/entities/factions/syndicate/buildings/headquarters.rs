//! Headquarters entity definition — Syndicate underground expansion structure.

use bevy::prelude::*;

use crate::types::{ObjectEnum, FactionEnum, SymmetryTypeEnum};
use crate::game::entities::types::{
    EntityDefinition, StructureDefinition, BaseStats, StructureStats, CommandSlotDef, SpawnParams,
    SlotCondition,
};
use crate::game::types::objects::ObjectType;
use crate::game::types::structures::{
    StructureCost, HeadquartersState,
};
use super::types::*;
use crate::game::types::ObjectInstance;
use crate::game::types::StructureInstance;
use crate::shared::types::{Selectable, SelectionBounds, GridPosition, SightRange, DomainEnum};
use crate::ui::types::{ObjectInterfaceState, StructureMenuState, CommandButtonAction};

/// Command slots for Headquarters.
static HQ_COMMAND_SLOTS: &[CommandSlotDef] = &[
    CommandSlotDef::new(ObjectInterfaceState::StructureMenu(StructureMenuState::HeadquartersMenu), 0, 0, CommandButtonAction::HqTrain(ObjectEnum::SyndicateAgent)),
    CommandSlotDef::new(ObjectInterfaceState::StructureMenu(StructureMenuState::HeadquartersMenu), 0, 1, CommandButtonAction::HqTrain(ObjectEnum::SyndicateGuard)),
    // Cancel — conditional on bk_has_queue (TODO: condition system Phase 2)
    CommandSlotDef::with_condition(ObjectInterfaceState::StructureMenu(StructureMenuState::HeadquartersMenu), 2, 1, CommandButtonAction::HqCancel, SlotCondition::HasQueue),
    CommandSlotDef::new(ObjectInterfaceState::StructureMenu(StructureMenuState::HeadquartersMenu), 2, 2, CommandButtonAction::SetRallyPoint),
];

/// Headquarters entity definition marker.
pub struct HeadquartersDef;

impl EntityDefinition for HeadquartersDef {
    const OBJECT: ObjectEnum = ObjectEnum::Headquarters;

    fn object_type() -> ObjectType {
        ObjectType {
            name: "Headquarters".to_string(),
            size: (2, 2),
            destructible: true,
            sight_range: 0, // Underground, no surface sight
            groupable: false,
        }
    }

    fn base_stats() -> BaseStats {
        BaseStats {
            max_hp: HQ_MAX_HP as u32,
            point_armor: HQ_POINT_ARMOR,
            full_armor: HQ_FULL_ARMOR,
            faction: FactionEnum::TheSyndicate,
        }
    }

    fn cost() -> Option<StructureCost> {
        Some(StructureCost {
            space_crystals: HQ_SC_COST,
            build_frames: HQ_BUILD_FRAMES,
        })
    }

    fn command_slots() -> &'static [CommandSlotDef] {
        HQ_COMMAND_SLOTS
    }

    fn spawn(commands: &mut EntityCommands, params: SpawnParams) {
        let stats = Self::base_stats();
        let obj = if params.under_construction {
            ObjectInstance::under_construction(ObjectEnum::Headquarters, stats.max_hp as f32)
        } else {
            ObjectInstance::destructible(ObjectEnum::Headquarters, stats.max_hp as f32)
        };

        commands.insert((
            Transform::from_translation(params.position),
            obj,
            StructureInstance::default(),
            params.owner,
            Selectable,
            SelectionBounds::from_dimensions(2.0, 1.0, 2.0),
            GridPosition {
                x: ((params.position.x - 0.5) + 32.0) as i32,
                z: ((params.position.z - 0.5) + 32.0) as i32,
            },
            DomainEnum::Underground,
            HeadquartersState::default(),
            SightRange(3),
        ));
        // NOTE: TunnelExpansionMarker { parent_tunnel } is NOT inserted here because
        // SpawnParams does not carry a parent_tunnel Entity. Callers must insert
        // TunnelExpansionMarker separately after spawn.
    }
}

impl StructureDefinition for HeadquartersDef {
    fn structure_stats() -> StructureStats {
        StructureStats {
            power_value: 0,
            build_radius: 0,
            symmetry: SymmetryTypeEnum::AAAA,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::entities::types::EntityRegistry;

    #[test]
    fn hq_base_stats() {
        let stats = HeadquartersDef::base_stats();
        assert_eq!(stats.max_hp, 400);
        assert_eq!(stats.point_armor, 1);
        assert_eq!(stats.full_armor, 4);
        assert_eq!(stats.faction, FactionEnum::TheSyndicate);
    }

    #[test]
    fn hq_object_type() {
        let ot = HeadquartersDef::object_type();
        assert_eq!(ot.name, "Headquarters");
        assert_eq!(ot.size, (2, 2));
        assert!(ot.destructible);
        assert_eq!(ot.sight_range, 0);
        assert!(!ot.groupable);
    }

    #[test]
    fn hq_structure_stats() {
        let ss = HeadquartersDef::structure_stats();
        assert_eq!(ss.power_value, 0);
        assert_eq!(ss.build_radius, 0);
        assert_eq!(ss.symmetry, SymmetryTypeEnum::AAAA);
    }

    #[test]
    fn hq_cost_value() {
        let cost = HeadquartersDef::cost().expect("HQ has a cost");
        assert_eq!(cost.space_crystals, 200);
        assert_eq!(cost.build_frames, 400);
    }

    #[test]
    fn hq_command_slots_count() {
        let slots = HeadquartersDef::command_slots();
        assert_eq!(slots.len(), 4);
    }

    #[test]
    fn hq_command_slots_train_agent() {
        let slots = HeadquartersDef::command_slots();
        let slot = slots.iter().find(|s| matches!(s.action, CommandButtonAction::HqTrain(ObjectEnum::SyndicateAgent)));
        assert!(slot.is_some());
        let s = slot.unwrap();
        assert_eq!(s.row, 0);
        assert_eq!(s.col, 0);
    }

    #[test]
    fn hq_command_slots_train_guard() {
        let slots = HeadquartersDef::command_slots();
        let slot = slots.iter().find(|s| matches!(s.action, CommandButtonAction::HqTrain(ObjectEnum::SyndicateGuard)));
        assert!(slot.is_some());
        let s = slot.unwrap();
        assert_eq!(s.row, 0);
        assert_eq!(s.col, 1);
    }

    #[test]
    fn hq_command_slots_set_rally_point() {
        let slots = HeadquartersDef::command_slots();
        let slot = slots.iter().find(|s| matches!(s.action, CommandButtonAction::SetRallyPoint));
        assert!(slot.is_some());
        let s = slot.unwrap();
        assert_eq!(s.row, 2);
        assert_eq!(s.col, 2);
    }

    #[test]
    fn hq_object_enum() {
        assert_eq!(HeadquartersDef::OBJECT, ObjectEnum::Headquarters);
    }

    #[test]
    fn hq_registry_lookup() {
        let mut registry = EntityRegistry::new();
        registry.register_structure::<HeadquartersDef>();
        let entry = registry.get(&ObjectEnum::Headquarters);
        assert!(entry.is_some());
        let entry = entry.unwrap();
        assert_eq!(entry.object_type.name, "Headquarters");
        assert_eq!(entry.base_stats.max_hp, 400);
        assert!(entry.cost.is_some());
        assert_eq!(entry.cost.as_ref().unwrap().space_crystals, 200);
    }
}
