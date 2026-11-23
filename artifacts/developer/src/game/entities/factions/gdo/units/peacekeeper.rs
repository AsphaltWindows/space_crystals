//! Peacekeeper entity definition — GDO light infantry unit.

use bevy::prelude::*;

use crate::types::{
    ObjectEnum, FactionEnum, UnitBaseEnum,
    FullyConnectedSubtype, TargetDomainEnum, TargetTypeEnum, AttackTypeEnum,
};
use crate::game::entities::types::{
    EntityDefinition, UnitDefinition, BaseStats, UnitStats, CommandSlotDef, SpawnParams,
};
use crate::game::types::objects::ObjectType;
use crate::game::types::structures::StructureCost;
use crate::game::units::types::unit_data::{
    AttackAttributesData, UnitType, UnitControlCost, RuggedTerrainDefenseBonus,
    frames_to_seconds,
};
use crate::game::units::types::movement::{
    MovementParams, TurnRateMovementParams, MovementSpeed, RotationSpeed, Velocity,
};
use crate::game::units::types::state::behavior::{
    BaseBehaviorState, LocomotionChannel, OrientationChannel, BaseAttackChannel,
};
use crate::game::units::types::state::commands::{BaseCommandState, CommandQueue};
use crate::game::combat::types::{AttackCapability, AttackState, AttackType, Armor, Silhouette};
use crate::shared::types::{
    Unit, Selectable, SelectionBounds, GridPosition, SightRange,
    VisibilityStateEnum,
};
use crate::game::types::ObjectInstance;
use crate::game::units::types::UnitCommand;
use crate::simulation::{FRAMES_PER_SECOND, SPACE_UNITS_PER_GRID_UNIT};
use crate::ui::types::{ObjectInterfaceState, CommandButtonAction};

// === Constants ===

/// Peacekeeper unit control cost.
pub const PEACEKEEPER_CONTROL_COST: u32 = 1;

/// Peacekeeper rugged terrain defense bonus.
pub const PEACEKEEPER_RUGGED_BONUS: f32 = 0.5;

/// LightInfantry max speed: 4 SU/frame * 16 FPS / 64 SU/GU = 1.0 GU/sec.
const MOVE_SPEED: f32 = 4.0 * (FRAMES_PER_SECOND as f32) / (SPACE_UNITS_PER_GRID_UNIT as f32);

/// High rotation speed for instant-turn infantry.
const ROTATION_SPEED: f32 = 10.0;

/// TurnRate: 180 deg/frame * 16 FPS (effectively instant).
const TURN_RATE: f32 = std::f32::consts::PI * (FRAMES_PER_SECOND as f32);

// === Command Slots ===

/// Static command slot definitions for Peacekeeper in Default state.
static COMMAND_SLOTS: &[CommandSlotDef] = &[
    CommandSlotDef::new(ObjectInterfaceState::Default, 0, 0, CommandButtonAction::UnitMove),
    CommandSlotDef::new(ObjectInterfaceState::Default, 0, 2, CommandButtonAction::UnitHoldPosition),
    CommandSlotDef::new(ObjectInterfaceState::Default, 1, 0, CommandButtonAction::UnitAttack),
    CommandSlotDef::new(ObjectInterfaceState::Default, 1, 1, CommandButtonAction::UnitPatrol),
    CommandSlotDef::new(ObjectInterfaceState::Default, 1, 2, CommandButtonAction::UnitAttackGround),
    CommandSlotDef::new(ObjectInterfaceState::Default, 2, 1, CommandButtonAction::UnitStop),
];

// === Entity Definition ===

/// Peacekeeper entity definition marker.
pub struct Peacekeeper;

impl EntityDefinition for Peacekeeper {
    const OBJECT: ObjectEnum = ObjectEnum::Peacekeeper;

    fn object_type() -> ObjectType {
        ObjectType {
            name: "Peacekeeper".to_string(),
            size: (24, 24),
            destructible: true,
            sight_range: 5,
            groupable: true,
        }
    }

    fn base_stats() -> BaseStats {
        BaseStats {
            max_hp: 50,
            point_armor: 1,
            full_armor: 1,
            faction: FactionEnum::GlobalDefenseOrdinance,
        }
    }

    fn cost() -> Option<StructureCost> {
        Some(StructureCost {
            space_crystals: 50,
            build_frames: 80,
        })
    }

    fn command_slots() -> &'static [CommandSlotDef] {
        COMMAND_SLOTS
    }

    fn spawn(commands: &mut EntityCommands, params: SpawnParams) {
        let stats = Self::base_stats();
        let attack = Self::attack_data().expect("Peacekeeper has attack data");
        let unit_base = UnitBaseEnum::LightInfantry;

        let attack_capability = AttackCapability {
            damage: attack.damage as f32,
            range: attack.range as f32,
            min_range: attack.min_range as f32,
            aim_time: frames_to_seconds(attack.aim_duration),
            fire_time: frames_to_seconds(attack.firing_duration),
            cooldown_time: frames_to_seconds(attack.cooldown_duration),
            reload_time: frames_to_seconds(attack.reload_duration),
            attack_type: AttackType::FullyConnected {
                subtype: attack.fc_subtype.unwrap_or(FullyConnectedSubtype::Ranged),
            },
            target_domain: attack.target_domain,
            target_type: attack.target_type,
            aoe_radius: attack.aoe_radius.map(|r| r as f32),
        };

        let turn_rate_params = TurnRateMovementParams {
            turn_rate: TURN_RATE,
            acceleration: f32::MAX,
            deceleration: f32::MAX,
            max_speed: MOVE_SPEED,
        };

        commands.insert((
            Transform::from_translation(params.position + Vec3::new(0.0, 0.5, 0.0)),
            Unit,
            ObjectInstance::destructible(ObjectEnum::Peacekeeper, stats.max_hp as f32),
            params.owner,
            UnitType { name: "Peacekeeper".to_string() },
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
            attack_capability,
            AttackState::default(),
            UnitCommand::Idle,
            turn_rate_params,
            unit_base.data().domain,
            UnitControlCost(PEACEKEEPER_CONTROL_COST),
            RuggedTerrainDefenseBonus(PEACEKEEPER_RUGGED_BONUS),
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
                width: 24.0 / SPACE_UNITS_PER_GRID_UNIT as f32,
                height: 24.0 / SPACE_UNITS_PER_GRID_UNIT as f32,
            },
            SightRange(5),
            VisibilityStateEnum::Visible,
        ));
    }
}

impl UnitDefinition for Peacekeeper {
    fn unit_stats() -> UnitStats {
        UnitStats {
            unit_base: UnitBaseEnum::LightInfantry,
            silhouette_width: 24,
            silhouette_height: 24,
            control_cost: PEACEKEEPER_CONTROL_COST,
        }
    }

    fn attack_data() -> Option<AttackAttributesData> {
        Some(AttackAttributesData {
            attack_type: AttackTypeEnum::FullyConnected,
            fc_subtype: Some(FullyConnectedSubtype::Ranged),
            target_domain: TargetDomainEnum::Ground,
            target_type: TargetTypeEnum::SingleTarget,
            aoe_radius: None,
            damage: 10,
            range: 4,
            min_range: 0,
            projectile_speed: None,
            aim_duration: 2,
            firing_duration: 1,
            cooldown_duration: 2,
            reload_duration: 12,
        })
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
    fn peacekeeper_base_stats() {
        let stats = Peacekeeper::base_stats();
        assert_eq!(stats.max_hp, 50);
        assert_eq!(stats.point_armor, 1);
        assert_eq!(stats.full_armor, 1);
        assert_eq!(stats.faction, FactionEnum::GlobalDefenseOrdinance);
    }

    #[test]
    fn peacekeeper_object_type() {
        let ot = Peacekeeper::object_type();
        assert_eq!(ot.name, "Peacekeeper");
        assert_eq!(ot.size, (24, 24));
        assert!(ot.destructible);
        assert_eq!(ot.sight_range, 5);
        assert!(ot.groupable);
    }

    #[test]
    fn peacekeeper_unit_stats() {
        let us = Peacekeeper::unit_stats();
        assert_eq!(us.unit_base, UnitBaseEnum::LightInfantry);
        assert_eq!(us.silhouette_width, 24);
        assert_eq!(us.silhouette_height, 24);
        assert_eq!(us.control_cost, 1);
    }

    #[test]
    fn peacekeeper_attack_data_values() {
        let ad = Peacekeeper::attack_data().expect("Peacekeeper has attack data");
        assert_eq!(ad.attack_type, AttackTypeEnum::FullyConnected);
        assert_eq!(ad.fc_subtype, Some(FullyConnectedSubtype::Ranged));
        assert_eq!(ad.target_domain, TargetDomainEnum::Ground);
        assert_eq!(ad.target_type, TargetTypeEnum::SingleTarget);
        assert!(ad.aoe_radius.is_none());
        assert_eq!(ad.damage, 10);
        assert_eq!(ad.range, 4);
        assert_eq!(ad.min_range, 0);
        assert!(ad.projectile_speed.is_none());
        assert_eq!(ad.aim_duration, 2);
        assert_eq!(ad.firing_duration, 1);
        assert_eq!(ad.cooldown_duration, 2);
        assert_eq!(ad.reload_duration, 12);
    }

    #[test]
    fn peacekeeper_movement_params() {
        let mp = Peacekeeper::movement_params();
        match mp {
            MovementParams::TurnRate(tr) => {
                assert!((tr.max_speed - 1.0).abs() < f32::EPSILON);
                assert_eq!(tr.acceleration, f32::MAX);
                assert_eq!(tr.deceleration, f32::MAX);
                // 180 deg/frame * 16 FPS = pi * 16 rad/s
                let expected_turn_rate = std::f32::consts::PI * 16.0;
                assert!((tr.turn_rate - expected_turn_rate).abs() < 0.001);
            }
            _ => panic!("Expected MovementParams::TurnRate"),
        }
    }

    #[test]
    fn peacekeeper_cost() {
        let cost = Peacekeeper::cost().expect("Peacekeeper has a cost");
        assert_eq!(cost.space_crystals, 50);
        assert_eq!(cost.build_frames, 80);
    }

    #[test]
    fn peacekeeper_command_slots_count() {
        let slots = Peacekeeper::command_slots();
        assert_eq!(slots.len(), 6);
    }

    #[test]
    fn peacekeeper_command_slots_move() {
        let slots = Peacekeeper::command_slots();
        let move_slot = slots.iter().find(|s| matches!(s.action, CommandButtonAction::UnitMove));
        assert!(move_slot.is_some());
        let s = move_slot.unwrap();
        assert_eq!(s.row, 0);
        assert_eq!(s.col, 0);
    }

    #[test]
    fn peacekeeper_command_slots_attack() {
        let slots = Peacekeeper::command_slots();
        let attack_slot = slots.iter().find(|s| matches!(s.action, CommandButtonAction::UnitAttack));
        assert!(attack_slot.is_some());
        let s = attack_slot.unwrap();
        assert_eq!(s.row, 1);
        assert_eq!(s.col, 0);
    }

    #[test]
    fn peacekeeper_command_slots_hold_position() {
        let slots = Peacekeeper::command_slots();
        let slot = slots.iter().find(|s| matches!(s.action, CommandButtonAction::UnitHoldPosition));
        assert!(slot.is_some());
        let s = slot.unwrap();
        assert_eq!(s.row, 0);
        assert_eq!(s.col, 2);
    }

    #[test]
    fn peacekeeper_command_slots_stop() {
        let slots = Peacekeeper::command_slots();
        let slot = slots.iter().find(|s| matches!(s.action, CommandButtonAction::UnitStop));
        assert!(slot.is_some());
        let s = slot.unwrap();
        assert_eq!(s.row, 2);
        assert_eq!(s.col, 1);
    }

    #[test]
    fn peacekeeper_registry_lookup() {
        let mut registry = EntityRegistry::new();
        registry.register_unit::<Peacekeeper>();
        let entry = registry.get(&ObjectEnum::Peacekeeper);
        assert!(entry.is_some());
        let entry = entry.unwrap();
        assert_eq!(entry.object_type.name, "Peacekeeper");
        assert_eq!(entry.base_stats.max_hp, 50);
        assert_eq!(entry.command_slots.len(), 6);
        assert!(entry.cost.is_some());
        assert_eq!(entry.cost.as_ref().unwrap().space_crystals, 50);
    }

    #[test]
    fn peacekeeper_object_enum() {
        assert_eq!(Peacekeeper::OBJECT, ObjectEnum::Peacekeeper);
    }

    #[test]
    fn peacekeeper_constants() {
        assert_eq!(PEACEKEEPER_CONTROL_COST, 1);
        assert!((PEACEKEEPER_RUGGED_BONUS - 0.5).abs() < f32::EPSILON);
    }
}
