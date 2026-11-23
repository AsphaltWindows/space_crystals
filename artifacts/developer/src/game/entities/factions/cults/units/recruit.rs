//! Cults Recruit entity definition — Cults gatherer/builder unit.

use bevy::prelude::*;

use crate::types::{ObjectEnum, FactionEnum, UnitBaseEnum};
use crate::game::entities::types::{
    EntityDefinition, UnitDefinition, BaseStats, UnitStats, CommandSlotDef, SpawnParams,
};
use crate::game::types::objects::ObjectType;
use crate::game::types::structures::StructureCost;
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
use crate::ui::types::{ObjectInterfaceState, CommandButtonAction, CultsRecruitMenuState};

// === Constants ===

/// Recruit unit control cost (1 UC per Recruit, tracked via OriginatingCenters).
pub const RECRUIT_CONTROL_COST: u32 = 1;

/// Recruit max HP.
pub const RECRUIT_MAX_HP: u32 = 50;

/// Recruit max speed: 4 SU/frame * 16 FPS / 64 SU/GU = 1.0 GU/sec (light infantry).
const MOVE_SPEED: f32 = 4.0 * (FRAMES_PER_SECOND as f32) / (SPACE_UNITS_PER_GRID_UNIT as f32);

/// High rotation speed for instant-turn infantry.
const ROTATION_SPEED: f32 = 10.0;

/// TurnRate: 180 deg/frame * 16 FPS (effectively instant).
const TURN_RATE: f32 = 180.0_f32.to_radians() * (FRAMES_PER_SECOND as f32);

// === Command Slots ===

/// Static command slot definitions for Recruit across all interface states.
static COMMAND_SLOTS: &[CommandSlotDef] = &[
    // RecruitDefault state — recruit-specific commands
    CommandSlotDef::new(ObjectInterfaceState::CultsRecruitMenu(CultsRecruitMenuState::RecruitDefault), 0, 0, CommandButtonAction::RecruitConstruct),
    CommandSlotDef::new(ObjectInterfaceState::CultsRecruitMenu(CultsRecruitMenuState::RecruitDefault), 1, 1, CommandButtonAction::RecruitAssistConstruction),
    // RecruitConstructMenu state — building selection submenu
    CommandSlotDef::new(ObjectInterfaceState::CultsRecruitMenu(CultsRecruitMenuState::RecruitConstructMenu), 0, 0, CommandButtonAction::RecruitSelectBuilding(ObjectEnum::CultsStorage)),
    CommandSlotDef::new(ObjectInterfaceState::CultsRecruitMenu(CultsRecruitMenuState::RecruitConstructMenu), 2, 0, CommandButtonAction::Back),
    // RecruitAwaitingPlacement — no grid buttons (handled by mouse clicks + Escape)
];

// === Entity Definition ===

/// Cults Recruit entity definition marker.
pub struct CultsRecruitDef;

impl EntityDefinition for CultsRecruitDef {
    const OBJECT: ObjectEnum = ObjectEnum::CultsRecruit;

    fn object_type() -> ObjectType {
        ObjectType {
            name: "Recruit".to_string(),
            size: (1, 1),
            destructible: true,
            sight_range: 3,
            groupable: true,
        }
    }

    fn base_stats() -> BaseStats {
        BaseStats {
            max_hp: RECRUIT_MAX_HP,
            point_armor: 0,
            full_armor: 0,
            faction: FactionEnum::TheCults,
        }
    }

    fn cost() -> Option<StructureCost> {
        // RC auto-produces at no resource cost (192 frames base).
        None
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
            ObjectInstance::destructible(ObjectEnum::CultsRecruit, stats.max_hp as f32),
            params.owner,
            UnitType { name: "Recruit".to_string() },
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
            UnitControlCost(RECRUIT_CONTROL_COST),
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
            SightRange(3),
            VisibilityStateEnum::Visible,
        ));
    }
}

impl UnitDefinition for CultsRecruitDef {
    fn unit_stats() -> UnitStats {
        UnitStats {
            unit_base: UnitBaseEnum::LightInfantry,
            silhouette_width: 1,
            silhouette_height: 1,
            control_cost: RECRUIT_CONTROL_COST,
        }
    }

    fn attack_data() -> Option<AttackAttributesData> {
        // Recruit is unarmed.
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
    fn recruit_base_stats() {
        let stats = CultsRecruitDef::base_stats();
        assert_eq!(stats.max_hp, 50);
        assert_eq!(stats.point_armor, 0);
        assert_eq!(stats.full_armor, 0);
        assert_eq!(stats.faction, FactionEnum::TheCults);
    }

    #[test]
    fn recruit_object_type() {
        let ot = CultsRecruitDef::object_type();
        assert_eq!(ot.name, "Recruit");
        assert_eq!(ot.size, (1, 1));
        assert!(ot.destructible);
        assert_eq!(ot.sight_range, 3);
        assert!(ot.groupable);
    }

    #[test]
    fn recruit_unit_stats() {
        let us = CultsRecruitDef::unit_stats();
        assert_eq!(us.unit_base, UnitBaseEnum::LightInfantry);
        assert_eq!(us.silhouette_width, 1);
        assert_eq!(us.silhouette_height, 1);
        assert_eq!(us.control_cost, 1);
    }

    #[test]
    fn recruit_attack_data_is_none() {
        assert!(CultsRecruitDef::attack_data().is_none());
    }

    #[test]
    fn recruit_movement_params() {
        let mp = CultsRecruitDef::movement_params();
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
    fn recruit_cost_is_none() {
        assert!(CultsRecruitDef::cost().is_none());
    }

    #[test]
    fn recruit_command_slots_count() {
        let slots = CultsRecruitDef::command_slots();
        assert_eq!(slots.len(), 4);
    }

    #[test]
    fn recruit_command_slots_construct() {
        let slots = CultsRecruitDef::command_slots();
        let slot = slots.iter().find(|s| matches!(s.action, CommandButtonAction::RecruitConstruct));
        assert!(slot.is_some());
        let s = slot.unwrap();
        assert_eq!(s.row, 0);
        assert_eq!(s.col, 0);
        assert_eq!(s.state, ObjectInterfaceState::CultsRecruitMenu(CultsRecruitMenuState::RecruitDefault));
    }

    #[test]
    fn recruit_command_slots_assist_construction() {
        let slots = CultsRecruitDef::command_slots();
        let slot = slots.iter().find(|s| matches!(s.action, CommandButtonAction::RecruitAssistConstruction));
        assert!(slot.is_some());
        let s = slot.unwrap();
        assert_eq!(s.row, 1);
        assert_eq!(s.col, 1);
    }

    #[test]
    fn recruit_command_slots_select_building() {
        let slots = CultsRecruitDef::command_slots();
        let slot = slots.iter().find(|s| matches!(s.action, CommandButtonAction::RecruitSelectBuilding(_)));
        assert!(slot.is_some());
        let s = slot.unwrap();
        assert_eq!(s.row, 0);
        assert_eq!(s.col, 0);
        assert_eq!(s.state, ObjectInterfaceState::CultsRecruitMenu(CultsRecruitMenuState::RecruitConstructMenu));
    }

    #[test]
    fn recruit_command_slots_back() {
        let slots = CultsRecruitDef::command_slots();
        let slot = slots.iter().find(|s| matches!(s.action, CommandButtonAction::Back));
        assert!(slot.is_some());
        let s = slot.unwrap();
        assert_eq!(s.row, 2);
        assert_eq!(s.col, 0);
    }

    #[test]
    fn recruit_registry_lookup() {
        let mut registry = EntityRegistry::new();
        registry.register_unit::<CultsRecruitDef>();
        let entry = registry.get(&ObjectEnum::CultsRecruit);
        assert!(entry.is_some());
        let entry = entry.unwrap();
        assert_eq!(entry.object_type.name, "Recruit");
        assert_eq!(entry.base_stats.max_hp, 50);
        assert_eq!(entry.command_slots.len(), 4);
        assert!(entry.cost.is_none());
    }

    #[test]
    fn recruit_object_enum() {
        assert_eq!(CultsRecruitDef::OBJECT, ObjectEnum::CultsRecruit);
    }

    #[test]
    fn recruit_constants() {
        assert_eq!(RECRUIT_CONTROL_COST, 1);
        assert_eq!(RECRUIT_MAX_HP, 50);
    }
}
