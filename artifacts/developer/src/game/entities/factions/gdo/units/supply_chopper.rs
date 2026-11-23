//! Supply Chopper entity definition — GDO unarmed utility hover unit.

use bevy::prelude::*;

use crate::types::{ObjectEnum, FactionEnum, UnitBaseEnum};
use crate::game::entities::types::{
    EntityDefinition, UnitDefinition, BaseStats, UnitStats, CommandSlotDef, SpawnParams,
};
use crate::game::types::objects::ObjectType;
use crate::game::types::structures::{StructureCost, SupplyChopperState};
use crate::game::units::types::unit_data::UnitType;
use crate::game::units::types::movement::{
    MovementParams, DragMovementParams, MovementSpeed, RotationSpeed, Velocity,
};
use crate::game::units::types::state::behavior::{
    BaseBehaviorState, LocomotionChannel, OrientationChannel,
};
use crate::game::units::types::state::commands::{BaseCommandState, CommandQueue};
use crate::game::combat::types::{Armor, Silhouette, SeparationRadius};
use crate::shared::types::{
    Unit, Selectable, SelectionBounds, GridPosition, SightRange,
    VisibilityStateEnum, DomainEnum,
};
use crate::game::types::ObjectInstance;
use crate::game::units::types::UnitCommand;
use crate::simulation::{FRAMES_PER_SECOND, SPACE_UNITS_PER_GRID_UNIT};
use crate::ui::types::{ObjectInterfaceState, CommandButtonAction};

// === Constants ===

/// Supply Chopper max HP.
pub const SC_MAX_HP: f32 = 150.0;
/// Supply Chopper point armor.
pub const SC_POINT_ARMOR: u32 = 1;
/// Supply Chopper full armor.
pub const SC_FULL_ARMOR: u32 = 1;
/// Supply Chopper unit control cost (does not consume unit control).
pub const SC_CONTROL_COST: u32 = 0;
/// Supply Chopper rotation speed.
const ROTATION_SPEED: f32 = 10.0;
/// Supply Chopper silhouette size (raw pixels).
const SILHOUETTE_SIZE: u32 = 60;
/// Supply Chopper hover height.
const HOVER_HEIGHT: f32 = 1.5;
/// Supply Chopper separation radius.
const SEPARATION_RADIUS: f32 = 1.25;

// === Drag Movement Constants ===

/// Forward acceleration: 0.9 su/f^2, scaled to world units.
fn forward_acceleration() -> f32 {
    0.9 * (FRAMES_PER_SECOND as f32) * (FRAMES_PER_SECOND as f32)
        / (SPACE_UNITS_PER_GRID_UNIT as f32)
}

/// Non-forward (omni) acceleration: 0.1 su/f^2, scaled to world units.
fn non_forward_acceleration() -> f32 {
    0.1 * (FRAMES_PER_SECOND as f32) * (FRAMES_PER_SECOND as f32)
        / (SPACE_UNITS_PER_GRID_UNIT as f32)
}

/// Drag ratio: 0.1/f, scaled to world units.
fn drag_ratio() -> f32 {
    0.1 * (FRAMES_PER_SECOND as f32)
}

/// Turn rate: 10 deg/f, scaled to rad/s.
fn turn_rate() -> f32 {
    (10.0_f32).to_radians() * (FRAMES_PER_SECOND as f32)
}

fn drag_params() -> DragMovementParams {
    DragMovementParams {
        forward_acceleration: forward_acceleration(),
        non_forward_acceleration: non_forward_acceleration(),
        drag_ratio: drag_ratio(),
        turn_rate: turn_rate(),
    }
}

// === Command Slots ===

/// Static command slot definitions for Supply Chopper in Default state.
static COMMAND_SLOTS: &[CommandSlotDef] = &[
    CommandSlotDef::new(ObjectInterfaceState::Default, 0, 0, CommandButtonAction::UnitMove),
    CommandSlotDef::new(ObjectInterfaceState::Default, 0, 1, CommandButtonAction::ChopperPickUpSupplies),
    CommandSlotDef::new(ObjectInterfaceState::Default, 0, 2, CommandButtonAction::ChopperAttachToTower),
    CommandSlotDef::new(ObjectInterfaceState::Default, 1, 0, CommandButtonAction::ChopperDropOffSupplies),
    CommandSlotDef::new(ObjectInterfaceState::Default, 1, 2, CommandButtonAction::UnitHoldPosition),
    CommandSlotDef::new(ObjectInterfaceState::Default, 2, 1, CommandButtonAction::UnitStop),
];

// === Entity Definition ===

/// Supply Chopper entity definition marker.
pub struct SupplyChopper;

impl EntityDefinition for SupplyChopper {
    const OBJECT: ObjectEnum = ObjectEnum::SupplyChopper;

    fn object_type() -> ObjectType {
        ObjectType {
            name: "Supply Chopper".to_string(),
            size: (1, 1),
            destructible: true,
            sight_range: 5,
            groupable: true,
        }
    }

    fn base_stats() -> BaseStats {
        BaseStats {
            max_hp: SC_MAX_HP as u32,
            point_armor: SC_POINT_ARMOR,
            full_armor: SC_FULL_ARMOR,
            faction: FactionEnum::GlobalDefenseOrdinance,
        }
    }

    fn cost() -> Option<StructureCost> {
        Some(StructureCost {
            space_crystals: 100,
            build_frames: 160,
        })
    }

    fn command_slots() -> &'static [CommandSlotDef] {
        COMMAND_SLOTS
    }

    fn spawn(commands: &mut EntityCommands, params: SpawnParams) {
        let stats = Self::base_stats();
        let unit_base = UnitBaseEnum::HoverCraft;
        let dp = drag_params();
        let max_speed = dp.max_speed();

        commands.insert((
            Transform::from_xyz(params.position.x, HOVER_HEIGHT, params.position.z),
            Unit,
            ObjectInstance::destructible(ObjectEnum::SupplyChopper, SC_MAX_HP),
            params.owner,
            UnitType { name: "Supply Chopper".to_string() },
            Selectable,
            SelectionBounds::unit(),
            GridPosition {
                x: ((params.position.x - 0.5) + 32.0) as i32,
                z: ((params.position.z - 0.5) + 32.0) as i32,
            },
            unit_base,
            MovementSpeed(max_speed),
            RotationSpeed(ROTATION_SPEED),
            Velocity(Vec3::ZERO),
            UnitCommand::Idle,
        ));
        commands.insert((
            dp,
            DomainEnum::Air,
            SupplyChopperState::default(),
            CommandQueue::new(),
            BaseCommandState::default(),
            BaseBehaviorState::default(),
            LocomotionChannel::default(),
            OrientationChannel::default(),
        ));
        commands.insert((
            Armor {
                point_armor: stats.point_armor as f32,
                full_armor: stats.full_armor as f32,
                directional_armor: false,
            },
            Silhouette {
                width: SILHOUETTE_SIZE as f32 / SPACE_UNITS_PER_GRID_UNIT as f32,
                height: SILHOUETTE_SIZE as f32 / SPACE_UNITS_PER_GRID_UNIT as f32,
            },
            SeparationRadius(SEPARATION_RADIUS),
            SightRange(5),
            VisibilityStateEnum::Visible,
        ));
    }
}

impl UnitDefinition for SupplyChopper {
    fn unit_stats() -> UnitStats {
        UnitStats {
            unit_base: UnitBaseEnum::HoverCraft,
            silhouette_width: SILHOUETTE_SIZE,
            silhouette_height: SILHOUETTE_SIZE,
            control_cost: SC_CONTROL_COST,
        }
    }

    fn attack_data() -> Option<crate::game::units::types::unit_data::AttackAttributesData> {
        None
    }

    fn movement_params() -> MovementParams {
        MovementParams::Drag(drag_params())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::entities::types::EntityRegistry;

    #[test]
    fn supply_chopper_base_stats() {
        let stats = SupplyChopper::base_stats();
        assert_eq!(stats.max_hp, 150);
        assert_eq!(stats.point_armor, 1);
        assert_eq!(stats.full_armor, 1);
        assert_eq!(stats.faction, FactionEnum::GlobalDefenseOrdinance);
    }

    #[test]
    fn supply_chopper_object_type() {
        let ot = SupplyChopper::object_type();
        assert_eq!(ot.name, "Supply Chopper");
        assert_eq!(ot.size, (1, 1));
        assert!(ot.destructible);
        assert_eq!(ot.sight_range, 5);
        assert!(ot.groupable);
    }

    #[test]
    fn supply_chopper_unit_stats() {
        let us = SupplyChopper::unit_stats();
        assert_eq!(us.unit_base, UnitBaseEnum::HoverCraft);
        assert_eq!(us.silhouette_width, 60);
        assert_eq!(us.silhouette_height, 60);
        assert_eq!(us.control_cost, 0);
    }

    #[test]
    fn supply_chopper_attack_data_none() {
        assert!(SupplyChopper::attack_data().is_none());
    }

    #[test]
    fn supply_chopper_movement_params() {
        let mp = SupplyChopper::movement_params();
        match mp {
            MovementParams::Drag(dp) => {
                // Verify the drag params match the spec values
                let expected_fwd = 0.9 * (FRAMES_PER_SECOND as f32).powi(2)
                    / (SPACE_UNITS_PER_GRID_UNIT as f32);
                let expected_omni = 0.1 * (FRAMES_PER_SECOND as f32).powi(2)
                    / (SPACE_UNITS_PER_GRID_UNIT as f32);
                let expected_drag = 0.1 * (FRAMES_PER_SECOND as f32);
                let expected_turn = (10.0_f32).to_radians() * (FRAMES_PER_SECOND as f32);
                assert!(f32::abs(dp.forward_acceleration - expected_fwd) < 0.001);
                assert!(f32::abs(dp.non_forward_acceleration - expected_omni) < 0.001);
                assert!(f32::abs(dp.drag_ratio - expected_drag) < 0.001);
                assert!(f32::abs(dp.turn_rate - expected_turn) < 0.001);
            }
            _ => panic!("Expected MovementParams::Drag"),
        }
    }

    #[test]
    fn supply_chopper_cost() {
        let cost = SupplyChopper::cost().expect("Supply Chopper has a production cost");
        assert_eq!(cost.space_crystals, 100);
        assert_eq!(cost.build_frames, 160);
    }

    #[test]
    fn supply_chopper_command_slots_count() {
        let slots = SupplyChopper::command_slots();
        assert_eq!(slots.len(), 6);
    }

    #[test]
    fn supply_chopper_command_slots_move() {
        let slots = SupplyChopper::command_slots();
        let slot = slots.iter().find(|s| matches!(s.action, CommandButtonAction::UnitMove));
        assert!(slot.is_some());
        let s = slot.unwrap();
        assert_eq!(s.row, 0);
        assert_eq!(s.col, 0);
    }

    #[test]
    fn supply_chopper_command_slots_pickup() {
        let slots = SupplyChopper::command_slots();
        let slot = slots.iter().find(|s| matches!(s.action, CommandButtonAction::ChopperPickUpSupplies));
        assert!(slot.is_some());
        let s = slot.unwrap();
        assert_eq!(s.row, 0);
        assert_eq!(s.col, 1);
    }

    #[test]
    fn supply_chopper_command_slots_attach() {
        let slots = SupplyChopper::command_slots();
        let slot = slots.iter().find(|s| matches!(s.action, CommandButtonAction::ChopperAttachToTower));
        assert!(slot.is_some());
        let s = slot.unwrap();
        assert_eq!(s.row, 0);
        assert_eq!(s.col, 2);
    }

    #[test]
    fn supply_chopper_command_slots_dropoff() {
        let slots = SupplyChopper::command_slots();
        let slot = slots.iter().find(|s| matches!(s.action, CommandButtonAction::ChopperDropOffSupplies));
        assert!(slot.is_some());
        let s = slot.unwrap();
        assert_eq!(s.row, 1);
        assert_eq!(s.col, 0);
    }

    #[test]
    fn supply_chopper_command_slots_hold_position() {
        let slots = SupplyChopper::command_slots();
        let slot = slots.iter().find(|s| matches!(s.action, CommandButtonAction::UnitHoldPosition));
        assert!(slot.is_some());
        let s = slot.unwrap();
        assert_eq!(s.row, 1);
        assert_eq!(s.col, 2);
    }

    #[test]
    fn supply_chopper_command_slots_stop() {
        let slots = SupplyChopper::command_slots();
        let slot = slots.iter().find(|s| matches!(s.action, CommandButtonAction::UnitStop));
        assert!(slot.is_some());
        let s = slot.unwrap();
        assert_eq!(s.row, 2);
        assert_eq!(s.col, 1);
    }

    #[test]
    fn supply_chopper_registry_lookup() {
        let mut registry = EntityRegistry::new();
        registry.register_unit::<SupplyChopper>();
        let entry = registry.get(&ObjectEnum::SupplyChopper);
        assert!(entry.is_some());
        let entry = entry.unwrap();
        assert_eq!(entry.object_type.name, "Supply Chopper");
        assert_eq!(entry.base_stats.max_hp, 150);
        assert_eq!(entry.command_slots.len(), 6);
        assert!(entry.cost.is_some());
        assert_eq!(entry.cost.as_ref().unwrap().space_crystals, 100);
    }

    #[test]
    fn supply_chopper_object_enum() {
        assert_eq!(SupplyChopper::OBJECT, ObjectEnum::SupplyChopper);
    }

    #[test]
    fn supply_chopper_constants() {
        assert!(f32::abs(SC_MAX_HP - 150.0) < f32::EPSILON);
        assert_eq!(SC_POINT_ARMOR, 1);
        assert_eq!(SC_FULL_ARMOR, 1);
        assert_eq!(SC_CONTROL_COST, 0);
    }
}
