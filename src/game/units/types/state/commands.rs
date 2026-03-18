#![allow(dead_code)]
use bevy::prelude::*;
use std::collections::VecDeque;
use crate::types::ObjectEnum;

/// Component representing a unit's current command
#[derive(Component, Clone, Debug)]
pub enum UnitCommand {
    Idle,
    Move(Vec3),
    AttackTarget(Entity),
    AttackLocation(Vec3),
    /// Move toward a location while auto-attacking enemies en route
    AttackMove(Vec3),
    Patrol { start: Vec3, end: Vec3, going_to_end: bool },
    HoldPosition,
    Stop,
    /// Reverse toward a location (only for units with can_reverse)
    Reverse(Vec3),
    /// Pick up supplies from a Supply Delivery Station (chopper only)
    PickUpSupplies(Entity),
    /// Attach to a Supply Tower (chopper only)
    AttachToTower(Entity),
    /// Enter a Tunnel structure (Syndicate units only)
    Enter(Entity),
    /// Build a structure at a target location (worker units only)
    Build { target: Vec3, object: ObjectEnum },
    /// Gather resources from a crystal patch or supply source (Agent only)
    Gather(Entity),
    /// Drop off carried resources at an own Tunnel (Agent only)
    DropOffResources(Entity),
    /// Build a Tunnel at a target location (Agent only — walk there first)
    BuildTunnel(Vec3),
}

impl UnitCommand {
    /// Check if this command is available for a unit with the given capabilities.
    /// - `has_attack`: unit has an AttackCapability component
    /// - `can_target_ground`: unit can target ground locations (for AttackGround)
    /// - `can_reverse`: unit base supports reversing (WheeledVehicle, TrackedVehicle, DrillUnit)
    /// - `is_syndicate`: unit belongs to the Syndicate faction (for Enter command)
    pub fn is_available(&self, has_attack: bool, can_target_ground: bool, can_reverse: bool, is_syndicate: bool) -> bool {
        match self {
            // All units can move, patrol, hold position, and stop
            UnitCommand::Idle => true,
            UnitCommand::Move(_) => true,
            UnitCommand::Patrol { .. } => true,
            UnitCommand::HoldPosition => true,
            UnitCommand::Stop => true,
            // Attack commands require attack capability
            UnitCommand::AttackTarget(_) => has_attack,
            UnitCommand::AttackMove(_) => has_attack,
            // AttackGround requires ability to target ground
            UnitCommand::AttackLocation(_) => has_attack && can_target_ground,
            // Reverse requires can_reverse on unit base
            UnitCommand::Reverse(_) => can_reverse,
            // Supply chopper commands — always available (chopper-specific UI handles visibility)
            UnitCommand::PickUpSupplies(_) => true,
            UnitCommand::AttachToTower(_) => true,
            // Enter requires Syndicate faction (tunnel tier validation happens at command-issuing time)
            UnitCommand::Enter(_) => is_syndicate,
            // Build requires Syndicate faction (Agent is the only builder)
            UnitCommand::Build { .. } => is_syndicate,
            // Gather requires Syndicate faction (Agent is the only gatherer)
            UnitCommand::Gather(_) => is_syndicate,
            // Drop off resources at own Tunnel (Agent only)
            UnitCommand::DropOffResources(_) => is_syndicate,
            // Build Tunnel (Agent only)
            UnitCommand::BuildTunnel(_) => is_syndicate,
        }
    }
}

/// Types of command modes for input
#[derive(Default, Debug, PartialEq, Clone, Copy)]
pub enum CommandType {
    #[default]
    Default,
    Move,
    Attack,
    AttackGround,
    AttackMove,
    Patrol,
    Reverse,
    Enter,
    Build,
    Gather,
    DropOff,
    BuildTunnel,
    SetRallyPoint,
    ScheduleDeliveries,
}

impl CommandType {
    pub fn name(&self) -> &str {
        match self {
            CommandType::Default => "Default",
            CommandType::Move => "Move",
            CommandType::Attack => "Attack",
            CommandType::AttackGround => "Attack Ground",
            CommandType::AttackMove => "Attack Move",
            CommandType::Patrol => "Patrol",
            CommandType::Reverse => "Reverse",
            CommandType::Enter => "Enter",
            CommandType::Build => "Build",
            CommandType::Gather => "Gather",
            CommandType::DropOff => "Drop Off",
            CommandType::BuildTunnel => "Build Tunnel",
            CommandType::SetRallyPoint => "Set Rally Point",
            CommandType::ScheduleDeliveries => "Schedule Deliveries",
        }
    }

    pub fn hotkey(&self) -> &str {
        match self {
            CommandType::Default => "",
            CommandType::Move => "M",
            CommandType::Attack => "A",
            CommandType::AttackGround => "G",
            CommandType::AttackMove => "T",
            CommandType::Patrol => "P",
            CommandType::Reverse => "R",
            CommandType::Enter => "N",
            CommandType::Build => "B",
            CommandType::Gather => "G",
            CommandType::DropOff => "D",
            CommandType::BuildTunnel => "A",
            CommandType::SetRallyPoint => "C",
            CommandType::ScheduleDeliveries => "S",
        }
    }
}

/// Unit state for behavior management
#[derive(Component, Default, Debug)]
pub enum UnitState {
    #[default]
    Idle,
    Busy,
    HoldingPosition,
}

/// Component marking a unit as holding position
#[derive(Component)]
pub struct HoldingPosition;

/// Component storing a queue of pending commands for shift-click queuing.
/// Uses VecDeque for efficient FIFO operations.
#[derive(Component, Clone, Debug, Default)]
pub struct CommandQueue {
    pub commands: VecDeque<UnitCommand>,
}

impl CommandQueue {
    /// Create a new empty command queue
    pub fn new() -> Self {
        Self { commands: VecDeque::new() }
    }

    /// Push a command to the back of the queue
    pub fn push(&mut self, command: UnitCommand) {
        self.commands.push_back(command);
    }

    /// Pop the next command from the front of the queue
    pub fn pop_front(&mut self) -> Option<UnitCommand> {
        self.commands.pop_front()
    }

    /// Check if the queue is empty
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    /// Get the number of queued commands
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    /// Clear all queued commands
    pub fn clear(&mut self) {
        self.commands.clear();
    }
}

/// Placeholder component for the base command state.
/// Tracks what command is currently executing on the unit base.
/// Will be fleshed out by the behavior_states_and_action_channels task.
#[derive(Component, Clone, Debug, Default)]
pub struct BaseCommandState {
    /// The type of command currently executing
    pub command_type: CommandType,
    /// Optional target location for location-based commands
    pub target_location: Option<Vec3>,
    /// Optional target entity for entity-targeted commands
    pub target_entity: Option<Entity>,
}

/// Component for the turret command state.
/// Only present on units whose UnitBase has_turret = true.
/// When `locked_target` is None, the turret falls back to autonomous scanning.
#[derive(Component, Clone, Debug, Default)]
pub struct TurretCommandState {
    /// Target entity locked by base behaviors (e.g., Attack command).
    /// When None, turret uses autonomous scanning to find targets.
    pub locked_target: Option<Entity>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // === CommandQueue tests ===

    #[test]
    fn command_queue_new_is_empty() {
        let queue = CommandQueue::new();
        assert!(queue.is_empty());
        assert_eq!(queue.len(), 0);
    }

    #[test]
    fn command_queue_push_and_len() {
        let mut queue = CommandQueue::new();
        queue.push(UnitCommand::Move(Vec3::new(1.0, 0.0, 0.0)));
        queue.push(UnitCommand::Stop);
        assert_eq!(queue.len(), 2);
        assert!(!queue.is_empty());
    }

    #[test]
    fn command_queue_pop_front_fifo_order() {
        let mut queue = CommandQueue::new();
        queue.push(UnitCommand::Move(Vec3::new(1.0, 0.0, 0.0)));
        queue.push(UnitCommand::HoldPosition);
        queue.push(UnitCommand::Stop);

        let first = queue.pop_front().unwrap();
        assert!(matches!(first, UnitCommand::Move(_)));

        let second = queue.pop_front().unwrap();
        assert!(matches!(second, UnitCommand::HoldPosition));

        let third = queue.pop_front().unwrap();
        assert!(matches!(third, UnitCommand::Stop));

        assert!(queue.pop_front().is_none());
        assert!(queue.is_empty());
    }

    #[test]
    fn command_queue_clear() {
        let mut queue = CommandQueue::new();
        queue.push(UnitCommand::Idle);
        queue.push(UnitCommand::Stop);
        assert_eq!(queue.len(), 2);

        queue.clear();
        assert!(queue.is_empty());
        assert_eq!(queue.len(), 0);
    }

    #[test]
    fn command_queue_pop_front_empty_returns_none() {
        let mut queue = CommandQueue::new();
        assert!(queue.pop_front().is_none());
    }

    #[test]
    fn command_queue_default_is_empty() {
        let queue = CommandQueue::default();
        assert!(queue.is_empty());
    }

    #[test]
    fn command_queue_uses_vecdeque() {
        // Verify VecDeque-backed queue handles many push/pop efficiently
        let mut queue = CommandQueue::new();
        for i in 0..100 {
            queue.push(UnitCommand::Move(Vec3::new(i as f32, 0.0, 0.0)));
        }
        assert_eq!(queue.len(), 100);
        for _ in 0..50 {
            queue.pop_front();
        }
        assert_eq!(queue.len(), 50);
    }

    // === BaseCommandState tests ===

    #[test]
    fn base_command_state_default() {
        let state = BaseCommandState::default();
        assert_eq!(state.command_type, CommandType::Default);
        assert!(state.target_location.is_none());
        assert!(state.target_entity.is_none());
    }

    #[test]
    fn base_command_state_with_move() {
        let state = BaseCommandState {
            command_type: CommandType::Move,
            target_location: Some(Vec3::new(10.0, 0.0, 5.0)),
            target_entity: None,
        };
        assert_eq!(state.command_type, CommandType::Move);
        assert!(state.target_location.is_some());
        assert!(state.target_entity.is_none());
    }

    // === TurretCommandState tests ===

    #[test]
    fn turret_command_state_default() {
        let state = TurretCommandState::default();
        assert!(state.locked_target.is_none());
    }

    #[test]
    fn turret_command_state_with_locked_target() {
        let state = TurretCommandState {
            locked_target: Some(Entity::from_raw_u32(42).unwrap()),
        };
        assert_eq!(state.locked_target, Some(Entity::from_raw_u32(42).unwrap()));
    }

    // === CommandType tests ===

    #[test]
    fn command_type_default_is_default() {
        let ct = CommandType::default();
        assert_eq!(ct, CommandType::Default);
    }

    #[test]
    fn command_type_attack_move_name_and_hotkey() {
        assert_eq!(CommandType::AttackMove.name(), "Attack Move");
        assert_eq!(CommandType::AttackMove.hotkey(), "T");
    }

    #[test]
    fn command_type_all_have_names() {
        let types = [
            CommandType::Default, CommandType::Move, CommandType::Attack,
            CommandType::AttackGround, CommandType::AttackMove, CommandType::Patrol,
            CommandType::Reverse,
        ];
        for ct in types {
            assert!(!ct.name().is_empty());
        }
    }

    #[test]
    fn command_type_reverse_name_and_hotkey() {
        assert_eq!(CommandType::Reverse.name(), "Reverse");
        assert_eq!(CommandType::Reverse.hotkey(), "R");
    }

    // === UnitCommand variants tests ===

    #[test]
    fn unit_command_attack_move_variant() {
        let cmd = UnitCommand::AttackMove(Vec3::new(5.0, 0.0, 5.0));
        assert!(matches!(cmd, UnitCommand::AttackMove(_)));
    }

    #[test]
    fn unit_command_reverse_variant() {
        let cmd = UnitCommand::Reverse(Vec3::new(3.0, 0.0, 2.0));
        assert!(matches!(cmd, UnitCommand::Reverse(_)));
    }

    // === is_available tests ===

    #[test]
    fn is_available_move_always() {
        let cmd = UnitCommand::Move(Vec3::ZERO);
        assert!(cmd.is_available(false, false, false, false));
        assert!(cmd.is_available(true, true, true, true));
    }

    #[test]
    fn is_available_stop_always() {
        assert!(UnitCommand::Stop.is_available(false, false, false, false));
    }

    #[test]
    fn is_available_hold_position_always() {
        assert!(UnitCommand::HoldPosition.is_available(false, false, false, false));
    }

    #[test]
    fn is_available_patrol_always() {
        let cmd = UnitCommand::Patrol { start: Vec3::ZERO, end: Vec3::ONE, going_to_end: true };
        assert!(cmd.is_available(false, false, false, false));
    }

    #[test]
    fn is_available_attack_target_requires_attack() {
        let cmd = UnitCommand::AttackTarget(Entity::from_raw_u32(1).unwrap());
        assert!(!cmd.is_available(false, false, false, false));
        assert!(cmd.is_available(true, false, false, false));
    }

    #[test]
    fn is_available_attack_move_requires_attack() {
        let cmd = UnitCommand::AttackMove(Vec3::ZERO);
        assert!(!cmd.is_available(false, false, false, false));
        assert!(cmd.is_available(true, false, false, false));
    }

    #[test]
    fn is_available_attack_location_requires_attack_and_ground() {
        let cmd = UnitCommand::AttackLocation(Vec3::ZERO);
        assert!(!cmd.is_available(false, false, false, false));
        assert!(!cmd.is_available(true, false, false, false));
        assert!(cmd.is_available(true, true, false, false));
    }

    #[test]
    fn is_available_reverse_requires_can_reverse() {
        let cmd = UnitCommand::Reverse(Vec3::ZERO);
        assert!(!cmd.is_available(false, false, false, false));
        assert!(!cmd.is_available(true, true, false, false));
        assert!(cmd.is_available(false, false, true, false));
    }

    // === Supply Chopper command tests ===

    #[test]
    fn unit_command_pick_up_supplies_variant() {
        let cmd = UnitCommand::PickUpSupplies(Entity::from_raw_u32(1).unwrap());
        assert!(matches!(cmd, UnitCommand::PickUpSupplies(_)));
    }

    #[test]
    fn unit_command_attach_to_tower_variant() {
        let cmd = UnitCommand::AttachToTower(Entity::from_raw_u32(2).unwrap());
        assert!(matches!(cmd, UnitCommand::AttachToTower(_)));
    }

    #[test]
    fn is_available_pick_up_supplies_always() {
        let cmd = UnitCommand::PickUpSupplies(Entity::from_raw_u32(1).unwrap());
        assert!(cmd.is_available(false, false, false, false));
        assert!(cmd.is_available(true, true, true, true));
    }

    #[test]
    fn is_available_attach_to_tower_always() {
        let cmd = UnitCommand::AttachToTower(Entity::from_raw_u32(2).unwrap());
        assert!(cmd.is_available(false, false, false, false));
        assert!(cmd.is_available(true, true, true, true));
    }

    // === Enter command tests ===

    #[test]
    fn unit_command_enter_variant() {
        let cmd = UnitCommand::Enter(Entity::from_raw_u32(10).unwrap());
        assert!(matches!(cmd, UnitCommand::Enter(_)));
    }

    #[test]
    fn unit_command_enter_stores_entity() {
        let tunnel = Entity::from_raw_u32(42).unwrap();
        let cmd = UnitCommand::Enter(tunnel);
        if let UnitCommand::Enter(e) = cmd {
            assert_eq!(e, Entity::from_raw_u32(42).unwrap());
        } else {
            panic!("Expected Enter variant");
        }
    }

    #[test]
    fn is_available_enter_requires_syndicate() {
        let cmd = UnitCommand::Enter(Entity::from_raw_u32(1).unwrap());
        // Not syndicate — should be unavailable
        assert!(!cmd.is_available(false, false, false, false));
        assert!(!cmd.is_available(true, true, true, false));
        // Is syndicate — should be available
        assert!(cmd.is_available(false, false, false, true));
        assert!(cmd.is_available(true, true, true, true));
    }

    #[test]
    fn command_type_enter_name() {
        assert_eq!(CommandType::Enter.name(), "Enter");
    }

    #[test]
    fn command_type_enter_hotkey() {
        assert_eq!(CommandType::Enter.hotkey(), "N");
    }

    #[test]
    fn command_type_enter_is_not_default() {
        assert_ne!(CommandType::Enter, CommandType::Default);
    }

    #[test]
    fn command_type_all_have_names_including_enter() {
        let types = [
            CommandType::Default, CommandType::Move, CommandType::Attack,
            CommandType::AttackGround, CommandType::AttackMove, CommandType::Patrol,
            CommandType::Reverse, CommandType::Enter, CommandType::Build,
            CommandType::Gather, CommandType::DropOff, CommandType::BuildTunnel,
            CommandType::SetRallyPoint, CommandType::ScheduleDeliveries,
        ];
        for ct in types {
            assert!(!ct.name().is_empty());
        }
    }

    // === Build command tests ===

    #[test]
    fn unit_command_build_variant() {
        let cmd = UnitCommand::Build {
            target: Vec3::new(10.0, 0.0, 10.0),
            object: ObjectEnum::Tunnel,
        };
        assert!(matches!(cmd, UnitCommand::Build { .. }));
    }

    #[test]
    fn unit_command_build_stores_target_and_object() {
        let cmd = UnitCommand::Build {
            target: Vec3::new(5.0, 0.0, 3.0),
            object: ObjectEnum::Tunnel,
        };
        if let UnitCommand::Build { target, object } = cmd {
            assert_eq!(target, Vec3::new(5.0, 0.0, 3.0));
            assert_eq!(object, ObjectEnum::Tunnel);
        } else {
            panic!("Expected Build variant");
        }
    }

    #[test]
    fn is_available_build_requires_syndicate() {
        let cmd = UnitCommand::Build {
            target: Vec3::ZERO,
            object: ObjectEnum::Tunnel,
        };
        // Not syndicate — should be unavailable
        assert!(!cmd.is_available(false, false, false, false));
        assert!(!cmd.is_available(true, true, true, false));
        // Is syndicate — should be available
        assert!(cmd.is_available(false, false, false, true));
        assert!(cmd.is_available(true, true, true, true));
    }

    #[test]
    fn command_type_build_name() {
        assert_eq!(CommandType::Build.name(), "Build");
    }

    #[test]
    fn command_type_build_hotkey() {
        assert_eq!(CommandType::Build.hotkey(), "B");
    }

    #[test]
    fn command_type_build_is_not_default() {
        assert_ne!(CommandType::Build, CommandType::Default);
    }

    // === Gather command tests ===

    #[test]
    fn unit_command_gather_variant() {
        let cmd = UnitCommand::Gather(Entity::from_raw_u32(5).unwrap());
        assert!(matches!(cmd, UnitCommand::Gather(_)));
    }

    #[test]
    fn is_available_gather_requires_syndicate() {
        let cmd = UnitCommand::Gather(Entity::from_raw_u32(1).unwrap());
        assert!(!cmd.is_available(false, false, false, false));
        assert!(cmd.is_available(false, false, false, true));
    }

    // === DropOffResources command tests ===

    #[test]
    fn unit_command_drop_off_resources_variant() {
        let cmd = UnitCommand::DropOffResources(Entity::from_raw_u32(7).unwrap());
        assert!(matches!(cmd, UnitCommand::DropOffResources(_)));
    }

    #[test]
    fn is_available_drop_off_requires_syndicate() {
        let cmd = UnitCommand::DropOffResources(Entity::from_raw_u32(1).unwrap());
        assert!(!cmd.is_available(false, false, false, false));
        assert!(cmd.is_available(false, false, false, true));
    }

    // === BuildTunnel command tests ===

    #[test]
    fn unit_command_build_tunnel_variant() {
        let cmd = UnitCommand::BuildTunnel(Vec3::new(10.0, 0.0, 10.0));
        assert!(matches!(cmd, UnitCommand::BuildTunnel(_)));
    }

    #[test]
    fn is_available_build_tunnel_requires_syndicate() {
        let cmd = UnitCommand::BuildTunnel(Vec3::ZERO);
        assert!(!cmd.is_available(false, false, false, false));
        assert!(cmd.is_available(false, false, false, true));
    }

    #[test]
    fn command_type_gather_name_and_hotkey() {
        assert_eq!(CommandType::Gather.name(), "Gather");
        assert_eq!(CommandType::Gather.hotkey(), "G");
    }

    #[test]
    fn command_type_drop_off_name_and_hotkey() {
        assert_eq!(CommandType::DropOff.name(), "Drop Off");
        assert_eq!(CommandType::DropOff.hotkey(), "D");
    }

    #[test]
    fn command_type_build_tunnel_name_and_hotkey() {
        assert_eq!(CommandType::BuildTunnel.name(), "Build Tunnel");
        assert_eq!(CommandType::BuildTunnel.hotkey(), "A");
    }

    #[test]
    fn command_type_set_rally_point_name() {
        assert_eq!(CommandType::SetRallyPoint.name(), "Set Rally Point");
    }

    #[test]
    fn command_type_set_rally_point_hotkey() {
        assert_eq!(CommandType::SetRallyPoint.hotkey(), "C");
    }

    #[test]
    fn command_type_schedule_deliveries_name() {
        assert_eq!(CommandType::ScheduleDeliveries.name(), "Schedule Deliveries");
    }

    #[test]
    fn command_type_schedule_deliveries_hotkey() {
        assert_eq!(CommandType::ScheduleDeliveries.hotkey(), "S");
    }

    #[test]
    fn command_type_schedule_deliveries_is_not_default() {
        assert_ne!(CommandType::ScheduleDeliveries, CommandType::Default);
    }
}

