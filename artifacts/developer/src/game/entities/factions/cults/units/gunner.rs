//! Cults Gunner entity definition — Cults ranged combat unit (placeholder).

use bevy::prelude::*;

use crate::types::{ObjectEnum, FactionEnum, UnitBaseEnum};
use crate::game::entities::types::{
    EntityDefinition, UnitDefinition, BaseStats, UnitStats, CommandSlotDef, SpawnParams,
};
use crate::game::types::objects::ObjectType;
use crate::game::types::structures::StructureCost;
use crate::game::entities::factions::cults::buildings::types::{
    GUNNER_TRAINING_COST, GUNNER_TRAINING_FRAMES,
};
use crate::game::units::types::unit_data::{
    AttackAttributesData, UnitType, UnitControlCost, OriginatingCenters,
};
use crate::game::units::types::movement::{
    MovementParams, TurnRateMovementParams, MovementSpeed, RotationSpeed, Velocity,
};
use crate::game::units::types::state::behavior::{
    BaseBehaviorState, LocomotionChannel, OrientationChannel, BaseAttackChannel,
};
use crate::game::units::types::state::commands::{BaseCommandState, CommandQueue};
use crate::game::combat::types::{Armor, Silhouette};
use crate::shared::types::{
    Unit, Selectable, SelectionBounds, GridPosition, SightRange,
    VisibilityStateEnum,
};
use crate::game::types::ObjectInstance;
use crate::game::units::types::UnitCommand;
use crate::simulation::{FRAMES_PER_SECOND, SPACE_UNITS_PER_GRID_UNIT};
use crate::ui::types::{ObjectInterfaceState, CommandButtonAction};

// === Constants ===

/// Gunner unit control cost.
pub const GUNNER_CONTROL_COST: u32 = 1;

/// Gunner max HP.
pub const GUNNER_MAX_HP: u32 = 80;

/// Gunner max speed: 4 SU/frame * 16 FPS / 64 SU/GU = 1.0 GU/sec.
const MOVE_SPEED: f32 = 4.0 * (FRAMES_PER_SECOND as f32) / (SPACE_UNITS_PER_GRID_UNIT as f32);

/// High rotation speed for instant-turn infantry.
const ROTATION_SPEED: f32 = 10.0;

/// TurnRate: 180 deg/frame * 16 FPS (effectively instant).
const TURN_RATE: f32 = 180.0_f32.to_radians() * (FRAMES_PER_SECOND as f32);

// === Command Slots ===

/// Static command slot definitions for Gunner in Default state.
/// Standard combat unit layout — same as Soldier (placeholder, no AttackGround yet).
static COMMAND_SLOTS: &[CommandSlotDef] = &[
    CommandSlotDef::new(ObjectInterfaceState::Default, 0, 0, CommandButtonAction::UnitMove),
    CommandSlotDef::new(ObjectInterfaceState::Default, 0, 2, CommandButtonAction::UnitHoldPosition),
    CommandSlotDef::new(ObjectInterfaceState::Default, 1, 0, CommandButtonAction::UnitAttack),
    CommandSlotDef::new(ObjectInterfaceState::Default, 1, 1, CommandButtonAction::UnitPatrol),
    CommandSlotDef::new(ObjectInterfaceState::Default, 2, 1, CommandButtonAction::UnitStop),
];

// === Entity Definition ===

/// Cults Gunner entity definition marker.
pub struct CultsGunnerDef;

impl EntityDefinition for CultsGunnerDef {
    const OBJECT: ObjectEnum = ObjectEnum::CultsGunner;

    fn object_type() -> ObjectType {
        ObjectType {
            name: "Gunner".to_string(),
            size: (1, 1),
            destructible: true,
            sight_range: 6,
            groupable: true,
        }
    }

    fn base_stats() -> BaseStats {
        BaseStats {
            max_hp: GUNNER_MAX_HP,
            point_armor: 1,
            full_armor: 1,
            faction: FactionEnum::TheCults,
        }
    }

    fn cost() -> Option<StructureCost> {
        Some(StructureCost {
            space_crystals: GUNNER_TRAINING_COST,
            build_frames: GUNNER_TRAINING_FRAMES,
        })
    }

    fn command_slots() -> &'static [CommandSlotDef] {
        COMMAND_SLOTS
    }

    fn spawn(commands: &mut EntityCommands, params: SpawnParams) {
        let stats = Self::base_stats();
        let unit_base = UnitBaseEnum::LightInfantry;

        let turn_rate_params = TurnRateMovementParams {
            turn_rate: TURN_RATE,
            acceleration: f32::MAX,
            deceleration: f32::MAX,
            max_speed: MOVE_SPEED,
        };

        commands.insert((
            Transform::from_translation(params.position + Vec3::new(0.0, 0.5, 0.0)),
            Unit,
            ObjectInstance::destructible(ObjectEnum::CultsGunner, stats.max_hp as f32),
            params.owner,
            UnitType { name: "Gunner".to_string() },
            Selectable,
            SelectionBounds::unit(),
            GridPosition {
                x: ((params.position.x - 0.5) + 32.0) as i32,
                z: ((params.position.z - 0.5) + 32.0) as i32,
            },
            unit_base,
            MovementSpeed(MOVE_SPEED),
            RotationSpeed(ROTATION_SPEED),
            Velocity(Vec3::ZERO),
        ));
        commands.insert((
            UnitCommand::Idle,
            turn_rate_params,
            unit_base.data().domain,
            UnitControlCost(GUNNER_CONTROL_COST),
            OriginatingCenters { centers: Vec::new() },
            CommandQueue::new(),
            BaseCommandState::default(),
            BaseBehaviorState::default(),
            LocomotionChannel::default(),
            OrientationChannel::default(),
            BaseAttackChannel::default(),
        ));
        commands.insert((
            Armor {
                point_armor: stats.point_armor as f32,
                full_armor: stats.full_armor as f32,
                directional_armor: unit_base.data().directional_armor,
            },
            Silhouette {
                width: 1.0 / SPACE_UNITS_PER_GRID_UNIT as f32,
                height: 1.0 / SPACE_UNITS_PER_GRID_UNIT as f32,
            },
            SightRange(6),
            VisibilityStateEnum::Visible,
        ));
    }
}

impl UnitDefinition for CultsGunnerDef {
    fn unit_stats() -> UnitStats {
        UnitStats {
            unit_base: UnitBaseEnum::LightInfantry,
            silhouette_width: 1,
            silhouette_height: 1,
            control_cost: GUNNER_CONTROL_COST,
        }
    }

    fn attack_data() -> Option<AttackAttributesData> {
        // Placeholder unit — no attack data yet.
        None
    }

    fn movement_params() -> MovementParams {
        MovementParams::TurnRate(TurnRateMovementParams {
            turn_rate: TURN_RATE,
            acceleration: f32::MAX,
            deceleration: f32::MAX,
            max_speed: MOVE_SPEED,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::entities::types::EntityRegistry;

    #[test]
    fn gunner_base_stats() {
        let stats = CultsGunnerDef::base_stats();
        assert_eq!(stats.max_hp, 80);
        assert_eq!(stats.point_armor, 1);
        assert_eq!(stats.full_armor, 1);
        assert_eq!(stats.faction, FactionEnum::TheCults);
    }

    #[test]
    fn gunner_object_type() {
        let ot = CultsGunnerDef::object_type();
        assert_eq!(ot.name, "Gunner");
        assert_eq!(ot.size, (1, 1));
        assert!(ot.destructible);
        assert_eq!(ot.sight_range, 6);
        assert!(ot.groupable);
    }

    #[test]
    fn gunner_unit_stats() {
        let us = CultsGunnerDef::unit_stats();
        assert_eq!(us.unit_base, UnitBaseEnum::LightInfantry);
        assert_eq!(us.silhouette_width, 1);
        assert_eq!(us.silhouette_height, 1);
        assert_eq!(us.control_cost, 1);
    }

    #[test]
    fn gunner_attack_data_is_none() {
        assert!(CultsGunnerDef::attack_data().is_none());
    }

    #[test]
    fn gunner_movement_params() {
        let mp = CultsGunnerDef::movement_params();
        match mp {
            MovementParams::TurnRate(tr) => {
                assert!((tr.max_speed - 1.0).abs() < f32::EPSILON);
                assert_eq!(tr.acceleration, f32::MAX);
                assert_eq!(tr.deceleration, f32::MAX);
                let expected_turn_rate = 180.0_f32.to_radians() * 16.0;
                assert!((tr.turn_rate - expected_turn_rate).abs() < 0.001);
            }
            _ => panic!("Expected MovementParams::TurnRate"),
        }
    }

    #[test]
    fn gunner_cost() {
        let cost = CultsGunnerDef::cost().expect("Gunner has a training cost");
        assert_eq!(cost.space_crystals, 100);
        assert_eq!(cost.build_frames, 200);
    }

    #[test]
    fn gunner_command_slots_count() {
        let slots = CultsGunnerDef::command_slots();
        assert_eq!(slots.len(), 5);
    }

    #[test]
    fn gunner_command_slots_move() {
        let slots = CultsGunnerDef::command_slots();
        let slot = slots.iter().find(|s| matches!(s.action, CommandButtonAction::UnitMove));
        assert!(slot.is_some());
        let s = slot.unwrap();
        assert_eq!(s.row, 0);
        assert_eq!(s.col, 0);
        assert_eq!(s.state, ObjectInterfaceState::Default);
    }

    #[test]
    fn gunner_command_slots_attack() {
        let slots = CultsGunnerDef::command_slots();
        let slot = slots.iter().find(|s| matches!(s.action, CommandButtonAction::UnitAttack));
        assert!(slot.is_some());
        let s = slot.unwrap();
        assert_eq!(s.row, 1);
        assert_eq!(s.col, 0);
    }

    #[test]
    fn gunner_command_slots_hold_position() {
        let slots = CultsGunnerDef::command_slots();
        let slot = slots.iter().find(|s| matches!(s.action, CommandButtonAction::UnitHoldPosition));
        assert!(slot.is_some());
        let s = slot.unwrap();
        assert_eq!(s.row, 0);
        assert_eq!(s.col, 2);
    }

    #[test]
    fn gunner_command_slots_stop() {
        let slots = CultsGunnerDef::command_slots();
        let slot = slots.iter().find(|s| matches!(s.action, CommandButtonAction::UnitStop));
        assert!(slot.is_some());
        let s = slot.unwrap();
        assert_eq!(s.row, 2);
        assert_eq!(s.col, 1);
    }

    #[test]
    fn gunner_registry_lookup() {
        let mut registry = EntityRegistry::new();
        registry.register_unit::<CultsGunnerDef>();
        let entry = registry.get(&ObjectEnum::CultsGunner);
        assert!(entry.is_some());
        let entry = entry.unwrap();
        assert_eq!(entry.object_type.name, "Gunner");
        assert_eq!(entry.base_stats.max_hp, 80);
        assert_eq!(entry.command_slots.len(), 5);
        assert!(entry.cost.is_some());
        assert_eq!(entry.cost.as_ref().unwrap().space_crystals, 100);
    }

    #[test]
    fn gunner_object_enum() {
        assert_eq!(CultsGunnerDef::OBJECT, ObjectEnum::CultsGunner);
    }

    #[test]
    fn gunner_constants() {
        assert_eq!(GUNNER_CONTROL_COST, 1);
        assert_eq!(GUNNER_MAX_HP, 80);
    }
}
