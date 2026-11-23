//! Deployment Center and Power Plant entity definitions — GDO base infrastructure.

use bevy::prelude::*;

use crate::types::{ObjectEnum, FactionEnum, SymmetryTypeEnum};
use crate::game::entities::types::{
    EntityDefinition, StructureDefinition, BaseStats, StructureStats, CommandSlotDef, SpawnParams,
    SlotCondition,
};
use crate::game::types::objects::ObjectType;
use crate::game::types::structures::{
    StructureCost, PowerValue, BuildRadiusExtension,
    DeploymentCenterState,
};
use super::types::*;
use crate::game::types::ObjectInstance;
use crate::shared::types::{
    Selectable, SelectionBounds, GridPosition, SightRange,
};
use crate::game::types::StructureInstance;
use crate::ui::types::{ObjectInterfaceState, StructureMenuState, CommandButtonAction};

// === Deployment Center ===

/// Command slots for Deployment Center across all 4 interface states.
static DC_COMMAND_SLOTS: &[CommandSlotDef] = &[
    // DcIdle
    CommandSlotDef::new(ObjectInterfaceState::StructureMenu(StructureMenuState::DcIdle), 0, 0, CommandButtonAction::DcOpenBuildMenu),
    // DcIdle cancel — conditional on has_active_construction
    CommandSlotDef::with_condition(ObjectInterfaceState::StructureMenu(StructureMenuState::DcIdle), 2, 1, CommandButtonAction::DcCancel, SlotCondition::HasActiveConstruction),
    // DcBuildMenu
    CommandSlotDef::new(ObjectInterfaceState::StructureMenu(StructureMenuState::DcBuildMenu), 0, 0, CommandButtonAction::DcBuild(ObjectEnum::PowerPlant)),
    CommandSlotDef::new(ObjectInterfaceState::StructureMenu(StructureMenuState::DcBuildMenu), 0, 1, CommandButtonAction::DcBuild(ObjectEnum::Barracks)),
    CommandSlotDef::new(ObjectInterfaceState::StructureMenu(StructureMenuState::DcBuildMenu), 1, 0, CommandButtonAction::DcBuild(ObjectEnum::SupplyTower)),
    CommandSlotDef::new(ObjectInterfaceState::StructureMenu(StructureMenuState::DcBuildMenu), 1, 1, CommandButtonAction::DcBuild(ObjectEnum::ExtractionFacility)),
    CommandSlotDef::new(ObjectInterfaceState::StructureMenu(StructureMenuState::DcBuildMenu), 2, 0, CommandButtonAction::Back),
    // DcConstructing
    CommandSlotDef::new(ObjectInterfaceState::StructureMenu(StructureMenuState::DcConstructing), 2, 0, CommandButtonAction::Back),
    CommandSlotDef::new(ObjectInterfaceState::StructureMenu(StructureMenuState::DcConstructing), 2, 1, CommandButtonAction::DcCancel),
    // DcReadyToPlace
    CommandSlotDef::new(ObjectInterfaceState::StructureMenu(StructureMenuState::DcReadyToPlace), 0, 0, CommandButtonAction::EnterPlacement),
    CommandSlotDef::new(ObjectInterfaceState::StructureMenu(StructureMenuState::DcReadyToPlace), 2, 0, CommandButtonAction::Back),
    CommandSlotDef::new(ObjectInterfaceState::StructureMenu(StructureMenuState::DcReadyToPlace), 2, 1, CommandButtonAction::DcCancel),
];

/// Deployment Center entity definition marker.
pub struct DeploymentCenter;

impl EntityDefinition for DeploymentCenter {
    const OBJECT: ObjectEnum = ObjectEnum::DeploymentCenter;

    fn object_type() -> ObjectType {
        ObjectType {
            name: "Deployment Center".to_string(),
            size: (4, 4),
            destructible: true,
            sight_range: 6,
            groupable: false,
        }
    }

    fn base_stats() -> BaseStats {
        BaseStats {
            max_hp: DC_MAX_HP as u32,
            point_armor: DC_POINT_ARMOR,
            full_armor: DC_FULL_ARMOR,
            faction: FactionEnum::GlobalDefenseOrdinance,
        }
    }

    fn cost() -> Option<StructureCost> {
        // DC is the primary builder — it is not itself built by DC
        None
    }

    fn command_slots() -> &'static [CommandSlotDef] {
        DC_COMMAND_SLOTS
    }

    fn spawn(commands: &mut EntityCommands, params: SpawnParams) {
        let stats = Self::base_stats();
        let obj = if params.under_construction {
            ObjectInstance::under_construction(ObjectEnum::DeploymentCenter, stats.max_hp as f32)
        } else {
            ObjectInstance::destructible(ObjectEnum::DeploymentCenter, stats.max_hp as f32)
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
            PowerValue(DC_POWER),
            BuildRadiusExtension(DC_BUILD_RADIUS),
            DeploymentCenterState::default(),
            SightRange(6),
        ));
    }
}

impl StructureDefinition for DeploymentCenter {
    fn structure_stats() -> StructureStats {
        StructureStats {
            power_value: DC_POWER,
            build_radius: DC_BUILD_RADIUS,
            symmetry: SymmetryTypeEnum::AAAA,
        }
    }
}

// === Power Plant ===

/// Power Plant entity definition marker.
pub struct PowerPlant;

impl EntityDefinition for PowerPlant {
    const OBJECT: ObjectEnum = ObjectEnum::PowerPlant;

    fn object_type() -> ObjectType {
        ObjectType {
            name: "Power Plant".to_string(),
            size: (2, 2),
            destructible: true,
            sight_range: 3,
            groupable: true,
        }
    }

    fn base_stats() -> BaseStats {
        BaseStats {
            max_hp: PP_MAX_HP as u32,
            point_armor: PP_POINT_ARMOR,
            full_armor: PP_FULL_ARMOR,
            faction: FactionEnum::GlobalDefenseOrdinance,
        }
    }

    fn cost() -> Option<StructureCost> {
        Some(StructureCost {
            space_crystals: 150,
            build_frames: 160,
        })
    }

    fn command_slots() -> &'static [CommandSlotDef] {
        // Power Plant is inert — no command slots
        &[]
    }

    fn spawn(commands: &mut EntityCommands, params: SpawnParams) {
        let stats = Self::base_stats();
        let rotation = params.rotation.unwrap_or_default();
        let obj = if params.under_construction {
            ObjectInstance::under_construction(ObjectEnum::PowerPlant, stats.max_hp as f32)
        } else {
            ObjectInstance::destructible(ObjectEnum::PowerPlant, stats.max_hp as f32)
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
            SelectionBounds::from_dimensions(2.0, 1.0, 2.0),
            GridPosition {
                x: ((params.position.x - 0.5) + 32.0) as i32,
                z: ((params.position.z - 0.5) + 32.0) as i32,
            },
            PowerValue(PP_POWER),
            BuildRadiusExtension(PP_BUILD_RADIUS),
            SightRange(3),
        ));
    }
}

impl StructureDefinition for PowerPlant {
    fn structure_stats() -> StructureStats {
        StructureStats {
            power_value: PP_POWER,
            build_radius: PP_BUILD_RADIUS,
            symmetry: SymmetryTypeEnum::AAAA,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::entities::types::EntityRegistry;

    // === Deployment Center Tests ===

    #[test]
    fn dc_base_stats() {
        let stats = DeploymentCenter::base_stats();
        assert_eq!(stats.max_hp, 1000);
        assert_eq!(stats.point_armor, 1);
        assert_eq!(stats.full_armor, 16);
        assert_eq!(stats.faction, FactionEnum::GlobalDefenseOrdinance);
    }

    #[test]
    fn dc_object_type() {
        let ot = DeploymentCenter::object_type();
        assert_eq!(ot.name, "Deployment Center");
        assert_eq!(ot.size, (4, 4));
        assert!(ot.destructible);
        assert_eq!(ot.sight_range, 6);
        assert!(!ot.groupable);
    }

    #[test]
    fn dc_structure_stats() {
        let ss = DeploymentCenter::structure_stats();
        assert_eq!(ss.power_value, 20);
        assert_eq!(ss.build_radius, 12);
        assert_eq!(ss.symmetry, SymmetryTypeEnum::AAAA);
    }

    #[test]
    fn dc_cost_is_none() {
        assert!(DeploymentCenter::cost().is_none());
    }

    #[test]
    fn dc_command_slots_count() {
        let slots = DeploymentCenter::command_slots();
        assert_eq!(slots.len(), 12);
    }

    #[test]
    fn dc_command_slots_open_build_menu() {
        let slots = DeploymentCenter::command_slots();
        let slot = slots.iter().find(|s| matches!(s.action, CommandButtonAction::DcOpenBuildMenu));
        assert!(slot.is_some());
        let s = slot.unwrap();
        assert_eq!(s.row, 0);
        assert_eq!(s.col, 0);
    }

    #[test]
    fn dc_command_slots_build_power_plant() {
        let slots = DeploymentCenter::command_slots();
        let slot = slots.iter().find(|s| matches!(s.action, CommandButtonAction::DcBuild(ObjectEnum::PowerPlant)));
        assert!(slot.is_some());
        let s = slot.unwrap();
        assert_eq!(s.row, 0);
        assert_eq!(s.col, 0);
        assert!(matches!(s.state, ObjectInterfaceState::StructureMenu(StructureMenuState::DcBuildMenu)));
    }

    #[test]
    fn dc_command_slots_build_barracks() {
        let slots = DeploymentCenter::command_slots();
        let slot = slots.iter().find(|s| matches!(s.action, CommandButtonAction::DcBuild(ObjectEnum::Barracks)));
        assert!(slot.is_some());
        let s = slot.unwrap();
        assert_eq!(s.row, 0);
        assert_eq!(s.col, 1);
    }

    #[test]
    fn dc_command_slots_enter_placement() {
        let slots = DeploymentCenter::command_slots();
        let slot = slots.iter().find(|s| {
            matches!(s.action, CommandButtonAction::EnterPlacement)
                && matches!(s.state, ObjectInterfaceState::StructureMenu(StructureMenuState::DcReadyToPlace))
        });
        assert!(slot.is_some());
        let s = slot.unwrap();
        assert_eq!(s.row, 0);
        assert_eq!(s.col, 0);
    }

    #[test]
    fn dc_registry_lookup() {
        let mut registry = EntityRegistry::new();
        registry.register_structure::<DeploymentCenter>();
        let entry = registry.get(&ObjectEnum::DeploymentCenter);
        assert!(entry.is_some());
        let entry = entry.unwrap();
        assert_eq!(entry.object_type.name, "Deployment Center");
        assert_eq!(entry.base_stats.max_hp, 1000);
        assert!(entry.cost.is_none());
    }

    #[test]
    fn dc_object_enum() {
        assert_eq!(DeploymentCenter::OBJECT, ObjectEnum::DeploymentCenter);
    }

    // === Power Plant Tests ===

    #[test]
    fn pp_base_stats() {
        let stats = PowerPlant::base_stats();
        assert_eq!(stats.max_hp, 350);
        assert_eq!(stats.point_armor, 1);
        assert_eq!(stats.full_armor, 4);
        assert_eq!(stats.faction, FactionEnum::GlobalDefenseOrdinance);
    }

    #[test]
    fn pp_object_type() {
        let ot = PowerPlant::object_type();
        assert_eq!(ot.name, "Power Plant");
        assert_eq!(ot.size, (2, 2));
        assert!(ot.destructible);
        assert_eq!(ot.sight_range, 3);
        assert!(ot.groupable);
    }

    #[test]
    fn pp_structure_stats() {
        let ss = PowerPlant::structure_stats();
        assert_eq!(ss.power_value, 20);
        assert_eq!(ss.build_radius, 1);
        assert_eq!(ss.symmetry, SymmetryTypeEnum::AAAA);
    }

    #[test]
    fn pp_cost() {
        let cost = PowerPlant::cost().expect("PowerPlant has a cost");
        assert_eq!(cost.space_crystals, 150);
        assert_eq!(cost.build_frames, 160);
    }

    #[test]
    fn pp_command_slots_empty() {
        assert!(PowerPlant::command_slots().is_empty());
    }

    #[test]
    fn pp_registry_lookup() {
        let mut registry = EntityRegistry::new();
        registry.register_structure::<PowerPlant>();
        let entry = registry.get(&ObjectEnum::PowerPlant);
        assert!(entry.is_some());
        let entry = entry.unwrap();
        assert_eq!(entry.object_type.name, "Power Plant");
        assert_eq!(entry.base_stats.max_hp, 350);
        assert!(entry.cost.is_some());
        assert_eq!(entry.cost.as_ref().unwrap().space_crystals, 150);
    }

    #[test]
    fn pp_object_enum() {
        assert_eq!(PowerPlant::OBJECT, ObjectEnum::PowerPlant);
    }
}
