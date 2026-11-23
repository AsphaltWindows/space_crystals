//! Tunnel entity definition — Syndicate primary surface structure.

use bevy::prelude::*;

use crate::types::{ObjectEnum, FactionEnum, SymmetryTypeEnum};
use crate::game::entities::types::{
    EntityDefinition, StructureDefinition, BaseStats, StructureStats, CommandSlotDef, SpawnParams,
    SlotCondition,
};
use crate::game::types::objects::ObjectType;
use crate::game::types::structures::{
    ConstructionHP, TunnelState, TunnelArea, TunnelTier,
};
use super::types::*;
use crate::game::types::ObjectInstance;
use crate::game::types::StructureInstance;
use crate::shared::types::{Selectable, SelectionBounds, GridPosition, SightRange};
use crate::ui::types::{
    ObjectInterfaceState, StructureMenuState, CommandButtonAction, EjectionQueue,
};

// Re-export dynamic cost functions from structures module
pub use crate::game::types::structures::{
    tunnel_construction_cost, tunnel_t2_upgrade_cost, tunnel_t3_upgrade_cost,
};

/// Command slots for Tunnel across all interface states.
static TUNNEL_COMMAND_SLOTS: &[CommandSlotDef] = &[
    // TunnelIdle
    CommandSlotDef::new(ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelIdle), 0, 0, CommandButtonAction::TunnelUpgrade),
    CommandSlotDef::new(ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelIdle), 0, 1, CommandButtonAction::TunnelOpenExpandMenu),
    CommandSlotDef::new(ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelIdle), 0, 2, CommandButtonAction::TunnelOpenEjectMenu),
    // Cancel upgrade — conditional on tunnel_is_upgrading (TODO: condition system Phase 2)
    CommandSlotDef::with_condition(ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelIdle), 2, 1, CommandButtonAction::TunnelCancelUpgrade, SlotCondition::TunnelIsUpgrading),
    // TunnelExpandMenu
    CommandSlotDef::new(ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelExpandMenu), 0, 0, CommandButtonAction::TunnelSelectExpansion(ObjectEnum::Headquarters)),
    CommandSlotDef::new(ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelExpandMenu), 2, 0, CommandButtonAction::Back),
    // TunnelEjectMenu
    CommandSlotDef::new(ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelEjectMenu), 2, 0, CommandButtonAction::Back),
    // TunnelAwaitingPlacement — no grid buttons (placement handled by mouse clicks + Escape)
];

/// Tunnel entity definition marker.
pub struct TunnelDef;

impl EntityDefinition for TunnelDef {
    const OBJECT: ObjectEnum = ObjectEnum::Tunnel;

    fn object_type() -> ObjectType {
        ObjectType {
            name: "Tunnel".to_string(),
            size: (4, 4),
            destructible: true,
            sight_range: 5,
            groupable: false,
        }
    }

    fn base_stats() -> BaseStats {
        BaseStats {
            max_hp: TUNNEL_T1_MAX_HP as u32,
            point_armor: TUNNEL_POINT_ARMOR,
            full_armor: TUNNEL_FULL_ARMOR,
            faction: FactionEnum::TheSyndicate,
        }
    }

    fn cost() -> Option<crate::game::types::structures::StructureCost> {
        // Tunnel costs are dynamic (depend on existing tunnel count).
        // Use tunnel_construction_cost() / tunnel_t2_upgrade_cost() / tunnel_t3_upgrade_cost() instead.
        None
    }

    fn command_slots() -> &'static [CommandSlotDef] {
        TUNNEL_COMMAND_SLOTS
    }

    fn spawn(commands: &mut EntityCommands, params: SpawnParams) {
        let tier = TunnelTier::Tier1;
        let obj = if params.under_construction {
            ObjectInstance::under_construction(ObjectEnum::Tunnel, tier.max_hp())
        } else {
            ObjectInstance::destructible(ObjectEnum::Tunnel, tier.max_hp())
        };

        let grid_pos = GridPosition {
            x: ((params.position.x - 0.5) + 32.0) as i32,
            z: ((params.position.z - 0.5) + 32.0) as i32,
        };

        commands.insert((
            Transform::from_translation(params.position),
            obj,
            StructureInstance::default(),
            params.owner,
            Selectable,
            SelectionBounds::from_dimensions(4.0, 1.0, 4.0),
            grid_pos,
            TunnelState::default_tier1(),
            TunnelArea::new(grid_pos.x, grid_pos.z, &tier),
            SightRange(TUNNEL_SIGHT_RANGE),
            EjectionQueue::default(),
        ));

        if params.under_construction {
            commands.insert(ConstructionHP::new(TUNNEL_CONSTRUCTION_FRAMES));
        }
    }
}

impl StructureDefinition for TunnelDef {
    fn structure_stats() -> StructureStats {
        StructureStats {
            power_value: 0,
            build_radius: 0,
            symmetry: SymmetryTypeEnum::ABCD,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::entities::types::EntityRegistry;

    #[test]
    fn tunnel_base_stats() {
        let stats = TunnelDef::base_stats();
        assert_eq!(stats.max_hp, 600);
        assert_eq!(stats.point_armor, 1);
        assert_eq!(stats.full_armor, 16);
        assert_eq!(stats.faction, FactionEnum::TheSyndicate);
    }

    #[test]
    fn tunnel_object_type() {
        let ot = TunnelDef::object_type();
        assert_eq!(ot.name, "Tunnel");
        assert_eq!(ot.size, (4, 4));
        assert!(ot.destructible);
        assert_eq!(ot.sight_range, 5);
        assert!(!ot.groupable);
    }

    #[test]
    fn tunnel_structure_stats() {
        let ss = TunnelDef::structure_stats();
        assert_eq!(ss.power_value, 0);
        assert_eq!(ss.build_radius, 0);
        assert_eq!(ss.symmetry, SymmetryTypeEnum::ABCD);
    }

    #[test]
    fn tunnel_cost_is_none() {
        assert!(TunnelDef::cost().is_none());
    }

    #[test]
    fn tunnel_command_slots_count() {
        let slots = TunnelDef::command_slots();
        assert_eq!(slots.len(), 7);
    }

    #[test]
    fn tunnel_idle_upgrade_slot() {
        let slots = TunnelDef::command_slots();
        let slot = slots.iter().find(|s| matches!(s.action, CommandButtonAction::TunnelUpgrade));
        assert!(slot.is_some());
        let s = slot.unwrap();
        assert_eq!(s.row, 0);
        assert_eq!(s.col, 0);
    }

    #[test]
    fn tunnel_idle_expand_menu_slot() {
        let slots = TunnelDef::command_slots();
        let slot = slots.iter().find(|s| matches!(s.action, CommandButtonAction::TunnelOpenExpandMenu));
        assert!(slot.is_some());
        let s = slot.unwrap();
        assert_eq!(s.row, 0);
        assert_eq!(s.col, 1);
    }

    #[test]
    fn tunnel_idle_eject_menu_slot() {
        let slots = TunnelDef::command_slots();
        let slot = slots.iter().find(|s| matches!(s.action, CommandButtonAction::TunnelOpenEjectMenu));
        assert!(slot.is_some());
        let s = slot.unwrap();
        assert_eq!(s.row, 0);
        assert_eq!(s.col, 2);
    }

    #[test]
    fn tunnel_expand_menu_headquarters_slot() {
        let slots = TunnelDef::command_slots();
        let slot = slots.iter().find(|s| matches!(s.action, CommandButtonAction::TunnelSelectExpansion(ObjectEnum::Headquarters)));
        assert!(slot.is_some());
        let s = slot.unwrap();
        assert_eq!(s.row, 0);
        assert_eq!(s.col, 0);
        assert!(matches!(s.state, ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelExpandMenu)));
    }

    #[test]
    fn tunnel_expand_menu_back_slot() {
        let slots = TunnelDef::command_slots();
        let slot = slots.iter().find(|s| {
            matches!(s.action, CommandButtonAction::Back)
                && matches!(s.state, ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelExpandMenu))
        });
        assert!(slot.is_some());
        let s = slot.unwrap();
        assert_eq!(s.row, 2);
        assert_eq!(s.col, 0);
    }

    #[test]
    fn tunnel_eject_menu_back_slot() {
        let slots = TunnelDef::command_slots();
        let slot = slots.iter().find(|s| {
            matches!(s.action, CommandButtonAction::Back)
                && matches!(s.state, ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelEjectMenu))
        });
        assert!(slot.is_some());
        let s = slot.unwrap();
        assert_eq!(s.row, 2);
        assert_eq!(s.col, 0);
    }

    #[test]
    fn tunnel_object_enum() {
        assert_eq!(TunnelDef::OBJECT, ObjectEnum::Tunnel);
    }

    #[test]
    fn tunnel_registry_lookup() {
        let mut registry = EntityRegistry::new();
        registry.register_structure::<TunnelDef>();
        let entry = registry.get(&ObjectEnum::Tunnel);
        assert!(entry.is_some());
        let entry = entry.unwrap();
        assert_eq!(entry.object_type.name, "Tunnel");
        assert_eq!(entry.base_stats.max_hp, 600);
        assert!(entry.cost.is_none());
    }

    #[test]
    fn tunnel_dynamic_cost_functions() {
        // tunnel_construction_cost
        assert_eq!(tunnel_construction_cost(0), 0);
        assert_eq!(tunnel_construction_cost(1), 1);
        assert_eq!(tunnel_construction_cost(2), 2);
        // tunnel_t2_upgrade_cost
        assert_eq!(tunnel_t2_upgrade_cost(0), 2);
        assert_eq!(tunnel_t2_upgrade_cost(1), 4);
        // tunnel_t3_upgrade_cost
        assert_eq!(tunnel_t3_upgrade_cost(0), 3);
        assert_eq!(tunnel_t3_upgrade_cost(1), 6);
    }
}
