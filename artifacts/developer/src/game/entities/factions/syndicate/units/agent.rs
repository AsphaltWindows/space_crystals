//! Syndicate Agent entity definition — Syndicate gatherer/builder unit.

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
    AttackAttributesData, UnitType, UnitControlCost, RuggedTerrainDefenseBonus, TunnelSpaceCost,
    frames_to_seconds,
};
use crate::game::units::types::movement::{
    MovementParams, TurnRateMovementParams, MovementSpeed, RotationSpeed, Velocity,
};
use crate::game::units::types::state::behavior::{
    BaseBehaviorState, LocomotionChannel, OrientationChannel, BaseAttackChannel,
};
use crate::game::units::types::state::commands::{BaseCommandState, CommandQueue};
use crate::game::units::types::state::AgentCarryState;
use crate::game::combat::types::{AttackCapability, AttackState, AttackType, Armor, Silhouette, MELEE_RANGE};
use crate::shared::types::{
    Unit, Selectable, SelectionBounds, GridPosition, SightRange,
    VisibilityStateEnum,
};
use crate::game::types::ObjectInstance;
use crate::game::units::types::UnitCommand;
use crate::simulation::{FRAMES_PER_SECOND, SPACE_UNITS_PER_GRID_UNIT};
use crate::ui::types::{ObjectInterfaceState, CommandButtonAction, AgentMenuState};

// === Constants ===

/// Agent unit control cost.
pub const AGENT_CONTROL_COST: u32 = 1;

/// Agent tunnel space cost (occupies 2 tunnel space when inside the network).
pub const AGENT_TUNNEL_SPACE_COST: u32 = 2;

/// Agent rugged terrain defense bonus (HeavyInfantry has rugged_terrain: true).
pub const AGENT_RUGGED_BONUS: f32 = 0.5;

/// Agent mining duration at a Space Crystal Patch (frames).
pub const AGENT_MINING_DURATION: u32 = 48;

/// Agent pickup duration at a Supply Delivery Station (frames).
pub const AGENT_PICKUP_DURATION: u32 = 48;

/// Agent drop-off duration at a Tunnel side (frames).
pub const AGENT_DROPOFF_DURATION: u32 = 48;

/// Amount of Space Crystals an Agent picks up per gather cycle.
pub const AGENT_CRYSTAL_CARRY: u32 = 50;

/// Amount of Supplies an Agent picks up per gather cycle.
pub const AGENT_SUPPLY_CARRY: u32 = 1;

/// Agent tunnel construction duration in simulation frames (480 frames = 30 seconds at 16 FPS).
pub const AGENT_TUNNEL_BUILD_FRAMES: u32 = 480;

/// Agent max speed: 6 SU/frame * 16 FPS / 64 SU/GU = 1.5 GU/sec.
const MOVE_SPEED: f32 = 6.0 * (FRAMES_PER_SECOND as f32) / (SPACE_UNITS_PER_GRID_UNIT as f32);

/// High rotation speed for instant-turn infantry.
const ROTATION_SPEED: f32 = 10.0;

/// TurnRate: 180 deg/frame * 16 FPS (effectively instant).
const TURN_RATE: f32 = 180.0_f32.to_radians() * (FRAMES_PER_SECOND as f32);

// === Command Slots ===

/// Static command slot definitions for Agent across all interface states.
static COMMAND_SLOTS: &[CommandSlotDef] = &[
    // Default state — standard combat unit layout (no Reverse for HeavyInfantry)
    CommandSlotDef::new(ObjectInterfaceState::Default, 0, 0, CommandButtonAction::UnitMove),
    CommandSlotDef::new(ObjectInterfaceState::Default, 0, 2, CommandButtonAction::UnitHoldPosition),
    CommandSlotDef::new(ObjectInterfaceState::Default, 1, 0, CommandButtonAction::UnitAttack),
    CommandSlotDef::new(ObjectInterfaceState::Default, 1, 1, CommandButtonAction::UnitPatrol),
    // Note: Agent is Melee/Ground — no AttackGround slot (can_target_ground is false for melee)
    CommandSlotDef::new(ObjectInterfaceState::Default, 2, 1, CommandButtonAction::UnitStop),
    // AgentDefault state — agent-specific commands
    CommandSlotDef::new(ObjectInterfaceState::AgentMenu(AgentMenuState::AgentDefault), 0, 0, CommandButtonAction::AgentBuildTunnel),
    CommandSlotDef::new(ObjectInterfaceState::AgentMenu(AgentMenuState::AgentDefault), 0, 1, CommandButtonAction::AgentDropOff),
    // AgentAwaitingPlacement — no grid buttons (handled by mouse clicks + Escape)
];

// === Entity Definition ===

/// Syndicate Agent entity definition marker.
pub struct SyndicateAgentDef;

impl EntityDefinition for SyndicateAgentDef {
    const OBJECT: ObjectEnum = ObjectEnum::SyndicateAgent;

    fn object_type() -> ObjectType {
        ObjectType {
            name: "Agent".to_string(),
            size: (36, 36),
            destructible: true,
            sight_range: 5,
            groupable: false, // Ungroupable — each Agent is its own SelectionGroup
        }
    }

    fn base_stats() -> BaseStats {
        BaseStats {
            max_hp: 75,
            point_armor: 1,
            full_armor: 1,
            faction: FactionEnum::TheSyndicate,
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
        let attack = Self::attack_data().expect("Agent has attack data");
        let unit_base = UnitBaseEnum::HeavyInfantry;

        let attack_capability = AttackCapability {
            damage: attack.damage as f32,
            range: MELEE_RANGE, // Melee subtype uses fixed melee range constant
            min_range: attack.min_range as f32,
            aim_time: frames_to_seconds(attack.aim_duration),
            fire_time: frames_to_seconds(attack.firing_duration),
            cooldown_time: frames_to_seconds(attack.cooldown_duration),
            reload_time: frames_to_seconds(attack.reload_duration),
            attack_type: AttackType::FullyConnected {
                subtype: attack.fc_subtype.unwrap_or(FullyConnectedSubtype::Melee),
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
            ObjectInstance::destructible(ObjectEnum::SyndicateAgent, stats.max_hp as f32),
            params.owner,
            UnitType { name: "Agent".to_string() },
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
            UnitControlCost(AGENT_CONTROL_COST),
            RuggedTerrainDefenseBonus(AGENT_RUGGED_BONUS),
            TunnelSpaceCost(AGENT_TUNNEL_SPACE_COST),
            CommandQueue::new(),
            BaseCommandState::default(),
            BaseBehaviorState::default(),
            LocomotionChannel::default(),
            OrientationChannel::default(),
            BaseAttackChannel::default(),
            AgentCarryState::default(),
        ));
        commands.insert((
            Armor {
                point_armor: stats.point_armor as f32,
                full_armor: stats.full_armor as f32,
                directional_armor: unit_base.data().directional_armor,
            },
            Silhouette {
                width: 36.0 / SPACE_UNITS_PER_GRID_UNIT as f32,
                height: 36.0 / SPACE_UNITS_PER_GRID_UNIT as f32,
            },
            SightRange(5),
            VisibilityStateEnum::Visible,
        ));
    }
}

impl UnitDefinition for SyndicateAgentDef {
    fn unit_stats() -> UnitStats {
        UnitStats {
            unit_base: UnitBaseEnum::HeavyInfantry,
            silhouette_width: 36,
            silhouette_height: 36,
            control_cost: AGENT_CONTROL_COST,
        }
    }

    fn attack_data() -> Option<AttackAttributesData> {
        Some(AttackAttributesData {
            attack_type: AttackTypeEnum::FullyConnected,
            fc_subtype: Some(FullyConnectedSubtype::Melee),
            target_domain: TargetDomainEnum::Ground,
            target_type: TargetTypeEnum::SingleTarget,
            aoe_radius: None,
            damage: 6,
            range: 0, // Melee — actual range uses MELEE_RANGE constant at spawn time
            min_range: 0,
            projectile_speed: None,
            aim_duration: 2,
            firing_duration: 4,
            cooldown_duration: 1,
            reload_duration: 9,
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
    fn agent_base_stats() {
        let stats = SyndicateAgentDef::base_stats();
        assert_eq!(stats.max_hp, 75);
        assert_eq!(stats.point_armor, 1);
        assert_eq!(stats.full_armor, 1);
        assert_eq!(stats.faction, FactionEnum::TheSyndicate);
    }

    #[test]
    fn agent_object_type() {
        let ot = SyndicateAgentDef::object_type();
        assert_eq!(ot.name, "Agent");
        assert_eq!(ot.size, (36, 36));
        assert!(ot.destructible);
        assert_eq!(ot.sight_range, 5);
        assert!(!ot.groupable);
    }

    #[test]
    fn agent_unit_stats() {
        let us = SyndicateAgentDef::unit_stats();
        assert_eq!(us.unit_base, UnitBaseEnum::HeavyInfantry);
        assert_eq!(us.silhouette_width, 36);
        assert_eq!(us.silhouette_height, 36);
        assert_eq!(us.control_cost, 1);
    }

    #[test]
    fn agent_attack_data_values() {
        let ad = SyndicateAgentDef::attack_data().expect("Agent has attack data");
        assert_eq!(ad.attack_type, AttackTypeEnum::FullyConnected);
        assert_eq!(ad.fc_subtype, Some(FullyConnectedSubtype::Melee));
        assert_eq!(ad.target_domain, TargetDomainEnum::Ground);
        assert_eq!(ad.target_type, TargetTypeEnum::SingleTarget);
        assert!(ad.aoe_radius.is_none());
        assert_eq!(ad.damage, 6);
        assert_eq!(ad.range, 0);
        assert_eq!(ad.min_range, 0);
        assert!(ad.projectile_speed.is_none());
        assert_eq!(ad.aim_duration, 2);
        assert_eq!(ad.firing_duration, 4);
        assert_eq!(ad.cooldown_duration, 1);
        assert_eq!(ad.reload_duration, 9);
    }

    #[test]
    fn agent_movement_params() {
        let mp = SyndicateAgentDef::movement_params();
        match mp {
            MovementParams::TurnRate(tr) => {
                assert!((tr.max_speed - 1.5).abs() < f32::EPSILON);
                assert_eq!(tr.acceleration, f32::MAX);
                assert_eq!(tr.deceleration, f32::MAX);
                let expected_turn_rate = 180.0_f32.to_radians() * 16.0;
                assert!((tr.turn_rate - expected_turn_rate).abs() < 0.001);
            }
            _ => panic!("Expected MovementParams::TurnRate"),
        }
    }

    #[test]
    fn agent_cost() {
        let cost = SyndicateAgentDef::cost().expect("Agent has a cost");
        assert_eq!(cost.space_crystals, 100);
        assert_eq!(cost.build_frames, 160);
    }

    #[test]
    fn agent_command_slots_count() {
        let slots = SyndicateAgentDef::command_slots();
        assert_eq!(slots.len(), 7);
    }

    #[test]
    fn agent_command_slots_move() {
        let slots = SyndicateAgentDef::command_slots();
        let slot = slots.iter().find(|s| matches!(s.action, CommandButtonAction::UnitMove));
        assert!(slot.is_some());
        let s = slot.unwrap();
        assert_eq!(s.row, 0);
        assert_eq!(s.col, 0);
        assert_eq!(s.state, ObjectInterfaceState::Default);
    }

    #[test]
    fn agent_command_slots_attack() {
        let slots = SyndicateAgentDef::command_slots();
        let slot = slots.iter().find(|s| matches!(s.action, CommandButtonAction::UnitAttack));
        assert!(slot.is_some());
        let s = slot.unwrap();
        assert_eq!(s.row, 1);
        assert_eq!(s.col, 0);
    }

    #[test]
    fn agent_command_slots_hold_position() {
        let slots = SyndicateAgentDef::command_slots();
        let slot = slots.iter().find(|s| matches!(s.action, CommandButtonAction::UnitHoldPosition));
        assert!(slot.is_some());
        let s = slot.unwrap();
        assert_eq!(s.row, 0);
        assert_eq!(s.col, 2);
    }

    #[test]
    fn agent_command_slots_stop() {
        let slots = SyndicateAgentDef::command_slots();
        let slot = slots.iter().find(|s| matches!(s.action, CommandButtonAction::UnitStop));
        assert!(slot.is_some());
        let s = slot.unwrap();
        assert_eq!(s.row, 2);
        assert_eq!(s.col, 1);
    }

    #[test]
    fn agent_command_slots_build_tunnel() {
        let slots = SyndicateAgentDef::command_slots();
        let slot = slots.iter().find(|s| matches!(s.action, CommandButtonAction::AgentBuildTunnel));
        assert!(slot.is_some());
        let s = slot.unwrap();
        assert_eq!(s.row, 0);
        assert_eq!(s.col, 0);
        assert_eq!(s.state, ObjectInterfaceState::AgentMenu(AgentMenuState::AgentDefault));
    }

    #[test]
    fn agent_command_slots_dropoff() {
        let slots = SyndicateAgentDef::command_slots();
        let slot = slots.iter().find(|s| matches!(s.action, CommandButtonAction::AgentDropOff));
        assert!(slot.is_some());
        let s = slot.unwrap();
        assert_eq!(s.row, 0);
        assert_eq!(s.col, 1);
        assert_eq!(s.state, ObjectInterfaceState::AgentMenu(AgentMenuState::AgentDefault));
    }

    #[test]
    fn agent_registry_lookup() {
        let mut registry = EntityRegistry::new();
        registry.register_unit::<SyndicateAgentDef>();
        let entry = registry.get(&ObjectEnum::SyndicateAgent);
        assert!(entry.is_some());
        let entry = entry.unwrap();
        assert_eq!(entry.object_type.name, "Agent");
        assert_eq!(entry.base_stats.max_hp, 75);
        assert_eq!(entry.command_slots.len(), 7);
        assert!(entry.cost.is_some());
        assert_eq!(entry.cost.as_ref().unwrap().space_crystals, 100);
    }

    #[test]
    fn agent_object_enum() {
        assert_eq!(SyndicateAgentDef::OBJECT, ObjectEnum::SyndicateAgent);
    }

    #[test]
    fn agent_constants() {
        assert_eq!(AGENT_CONTROL_COST, 1);
        assert_eq!(AGENT_TUNNEL_SPACE_COST, 2);
        assert!((AGENT_RUGGED_BONUS - 0.5).abs() < f32::EPSILON);
        assert_eq!(AGENT_MINING_DURATION, 48);
        assert_eq!(AGENT_PICKUP_DURATION, 48);
        assert_eq!(AGENT_DROPOFF_DURATION, 48);
        assert_eq!(AGENT_CRYSTAL_CARRY, 50);
        assert_eq!(AGENT_SUPPLY_CARRY, 1);
        assert_eq!(AGENT_TUNNEL_BUILD_FRAMES, 480);
    }
}
