//! Recruitment Center entity definition — Cults primary structure.

use bevy::prelude::*;

use crate::types::{ObjectEnum, FactionEnum, SymmetryTypeEnum};
use crate::game::entities::types::{
    EntityDefinition, StructureDefinition, BaseStats, StructureStats, CommandSlotDef, SpawnParams,
    SlotCondition,
};
use crate::game::types::objects::ObjectType;
use crate::game::types::structures::RecruitmentCenterState;
use super::types::*;
use crate::game::types::ObjectInstance;
use crate::game::types::StructureInstance;
use crate::shared::types::{Selectable, SelectionBounds, GridPosition, SightRange};
use crate::ui::types::{ObjectInterfaceState, StructureMenuState, CommandButtonAction};

/// Command slots for Recruitment Center.
static RC_COMMAND_SLOTS: &[CommandSlotDef] = &[
    // Cancel — conditional on bk_has_queue (TODO: condition system Phase 2)
    CommandSlotDef::with_condition(ObjectInterfaceState::StructureMenu(StructureMenuState::RecruitmentCenterMenu), 2, 1, CommandButtonAction::RcCancel, SlotCondition::HasQueue),
    CommandSlotDef::new(ObjectInterfaceState::StructureMenu(StructureMenuState::RecruitmentCenterMenu), 2, 2, CommandButtonAction::SetRallyPoint),
];

/// Recruitment Center entity definition marker.
pub struct RecruitmentCenterDef;

impl EntityDefinition for RecruitmentCenterDef {
    const OBJECT: ObjectEnum = ObjectEnum::RecruitmentCenter;

    fn object_type() -> ObjectType {
        ObjectType {
            name: "Recruitment Center".to_string(),
            size: (4, 4),
            destructible: true,
            sight_range: 6,
            groupable: false,
        }
    }

    fn base_stats() -> BaseStats {
        BaseStats {
            max_hp: RC_MAX_HP as u32,
            point_armor: RC_POINT_ARMOR,
            full_armor: RC_FULL_ARMOR,
            faction: FactionEnum::TheCults,
        }
    }

    fn cost() -> Option<crate::game::types::structures::StructureCost> {
        // RC is the Cults primary structure — not built by another structure
        None
    }

    fn command_slots() -> &'static [CommandSlotDef] {
        RC_COMMAND_SLOTS
    }

    fn spawn(commands: &mut EntityCommands, params: SpawnParams) {
        let stats = Self::base_stats();
        let obj = if params.under_construction {
            ObjectInstance::under_construction(ObjectEnum::RecruitmentCenter, stats.max_hp as f32)
        } else {
            ObjectInstance::destructible(ObjectEnum::RecruitmentCenter, stats.max_hp as f32)
        };

        commands.insert((
            Transform::from_translation(params.position),
            obj,
            StructureInstance::default(),
            params.owner,
            Selectable,
            SelectionBounds::from_dimensions(4.0, 1.5, 4.0),
            GridPosition {
                x: ((params.position.x - 0.5) + 32.0) as i32,
                z: ((params.position.z - 0.5) + 32.0) as i32,
            },
            SightRange(6),
            RecruitmentCenterState { build_order: 0, ..Default::default() },
        ));
    }
}

impl StructureDefinition for RecruitmentCenterDef {
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
    fn rc_base_stats() {
        let stats = RecruitmentCenterDef::base_stats();
        assert_eq!(stats.max_hp, 1000);
        assert_eq!(stats.point_armor, 1);
        assert_eq!(stats.full_armor, 16);
        assert_eq!(stats.faction, FactionEnum::TheCults);
    }

    #[test]
    fn rc_object_type() {
        let ot = RecruitmentCenterDef::object_type();
        assert_eq!(ot.name, "Recruitment Center");
        assert_eq!(ot.size, (4, 4));
        assert!(ot.destructible);
        assert_eq!(ot.sight_range, 6);
        assert!(!ot.groupable);
    }

    #[test]
    fn rc_structure_stats() {
        let ss = RecruitmentCenterDef::structure_stats();
        assert_eq!(ss.power_value, 0);
        assert_eq!(ss.build_radius, 0);
        assert_eq!(ss.symmetry, SymmetryTypeEnum::AAAA);
    }

    #[test]
    fn rc_cost_is_none() {
        assert!(RecruitmentCenterDef::cost().is_none());
    }

    #[test]
    fn rc_command_slots_count() {
        let slots = RecruitmentCenterDef::command_slots();
        assert_eq!(slots.len(), 2);
    }

    #[test]
    fn rc_command_slots_cancel() {
        let slots = RecruitmentCenterDef::command_slots();
        let slot = slots.iter().find(|s| matches!(s.action, CommandButtonAction::RcCancel));
        assert!(slot.is_some());
        let s = slot.unwrap();
        assert_eq!(s.row, 2);
        assert_eq!(s.col, 1);
    }

    #[test]
    fn rc_command_slots_rally_point() {
        let slots = RecruitmentCenterDef::command_slots();
        let slot = slots.iter().find(|s| matches!(s.action, CommandButtonAction::SetRallyPoint));
        assert!(slot.is_some());
        let s = slot.unwrap();
        assert_eq!(s.row, 2);
        assert_eq!(s.col, 2);
    }

    #[test]
    fn rc_registry_lookup() {
        let mut registry = EntityRegistry::new();
        registry.register_structure::<RecruitmentCenterDef>();
        let entry = registry.get(&ObjectEnum::RecruitmentCenter);
        assert!(entry.is_some());
        let entry = entry.unwrap();
        assert_eq!(entry.object_type.name, "Recruitment Center");
        assert_eq!(entry.base_stats.max_hp, 1000);
        assert!(entry.cost.is_none());
    }

    #[test]
    fn rc_object_enum() {
        assert_eq!(RecruitmentCenterDef::OBJECT, ObjectEnum::RecruitmentCenter);
    }
}
