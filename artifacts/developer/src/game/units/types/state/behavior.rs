#![allow(dead_code)]
use bevy::prelude::*;
use crate::types::ObjectEnum;

/// BaseBehaviorState tracks the current behavior/action of the unit base.
/// Parameterized by UnitBase movement model — different models store different internal data.
#[derive(Component, Clone, Debug, Default)]
pub enum BaseBehaviorState {
    /// Infantry/Mech: path + progress (TurnRate movement model)
    TurnRate {
        planned_path: Vec<Vec3>,
        path_index: usize,
    },
    /// Wheeled vehicles: path + progress (FixedTurnRadius movement model)
    FixedTurnRadius {
        planned_path: Vec<Vec3>,
        path_index: usize,
    },
    /// Tracked/Drill: path + progress (SpeedTurnRadius movement model)
    SpeedTurnRadius {
        planned_path: Vec<Vec3>,
        path_index: usize,
    },
    /// Hover: path + drift state (Drag movement model)
    Drag {
        planned_path: Vec<Vec3>,
        path_index: usize,
        drift_velocity: Vec3,
    },
    /// Glider: circling state, strafing run state (Glider movement model)
    Glider {
        planned_path: Vec<Vec3>,
        path_index: usize,
        circling: bool,
        strafe_target: Option<Vec3>,
    },
    /// No active behavior
    #[default]
    None,
}

/// TurretBehaviorState tracks the turret's autonomous scanning state.
/// Only present on units whose UnitBase has_turret = true.
#[derive(Component, Clone, Debug, Default)]
pub struct TurretBehaviorState {
    /// Current autonomous scan heading (radians)
    pub scan_direction: f32,
    /// Scan sweep direction (true = clockwise)
    pub scan_clockwise: bool,
    /// Last known position of a target (for tracking lost targets)
    pub last_known_target_pos: Option<Vec3>,
}

// === Base Action Channels (3 components) ===

/// LocomotionChannel drives movement systems.
/// Behaviors write to this channel each tick; movement systems read it.
#[derive(Component, Clone, Debug, Default)]
pub enum LocomotionChannel {
    /// Moving along a path of waypoints
    Moving(Vec<Vec3>),
    /// Reversing along a path (CanReverse bases only)
    Reversing(Vec<Vec3>),
    /// Decelerating to a stop
    Stopping,
    /// At rest
    #[default]
    Stationary,
}

/// OrientationChannel drives body rotation.
/// Behaviors write to this channel each tick; rotation systems read it.
#[derive(Component, Clone, Debug, Default)]
pub enum OrientationChannel {
    /// Turning to face a target position
    Turning(Vec3),
    /// Holding current facing
    #[default]
    Maintaining,
}

/// BaseAttackChannel handles attack execution for infantry-only (no turret) units.
/// Turret-bearing units use TurretAttackChannel instead.
#[derive(Component, Clone, Debug, Default)]
pub enum BaseAttackChannel {
    /// Aiming at target entity
    Aiming(Entity),
    /// Firing at target entity
    Firing(Entity),
    /// Post-fire cooldown
    Cooldown,
    /// Reload cycle
    Reloading,
    /// Not attacking
    #[default]
    None,
}

// === Turret Action Channels (2 components, turret units only) ===

/// TurretOrientationChannel drives turret rotation (parallel to existing Turret component).
/// Only present on units whose UnitBase has_turret = true.
#[derive(Component, Clone, Debug, Default)]
pub enum TurretOrientationChannel {
    /// Turning turret to face a target position
    Turning(Vec3),
    /// Holding current turret facing
    #[default]
    Maintaining,
}

/// TurretAttackChannel handles turret attack execution.
/// Only present on units whose UnitBase has_turret = true.
#[derive(Component, Clone, Debug, Default)]
pub enum TurretAttackChannel {
    /// Aiming turret at target entity
    Aiming(Entity),
    /// Firing turret at target entity
    Firing(Entity),
    /// Post-fire cooldown
    Cooldown,
    /// Reload cycle
    Reloading,
    /// Turret not attacking
    #[default]
    Inactive,
}

/// Marker component for the EnteringTunnel behavior.
/// When present, a dedicated system moves the unit toward the target Tunnel's Side A position.
/// On arrival, the unit is despawned and added to the Tunnel Network's unit pool.
/// Uses ECS composition (marker component) rather than extending BaseBehaviorState,
/// keeping BaseBehaviorState focused on movement models.
#[derive(Component, Clone, Debug)]
pub struct EnteringTunnelBehavior {
    /// The tunnel entity this unit is entering
    pub target_tunnel: Entity,
    /// Precomputed path to Side A (set once when behavior starts, recomputed if needed)
    pub path: Vec<Vec3>,
    /// Current index in the path
    pub path_index: usize,
}

impl EnteringTunnelBehavior {
    /// Create a new EnteringTunnelBehavior targeting a specific tunnel
    pub fn new(target_tunnel: Entity) -> Self {
        Self {
            target_tunnel,
            path: Vec::new(),
            path_index: 0,
        }
    }

    /// Create with an initial path
    pub fn with_path(target_tunnel: Entity, path: Vec<Vec3>) -> Self {
        Self {
            target_tunnel,
            path,
            path_index: 0,
        }
    }
}

/// Marker component for the BuildingStructure behavior.
/// When present, a dedicated system moves the unit toward the target build location.
/// On arrival, tile validation is performed:
/// - If valid: the structure is spawned and the worker transitions to idle.
/// - If invalid: the build command is cancelled and the worker idles.
/// Uses ECS composition (marker component) rather than extending BaseBehaviorState.
#[derive(Component, Clone, Debug)]
pub struct BuildingStructureBehavior {
    /// Target world-space location to build at
    pub target_location: Vec3,
    /// Object type to build
    pub object_to_build: ObjectEnum,
    /// Whether the unit has arrived at the build location
    pub arrived: bool,
    /// Precomputed path to target (set once when behavior starts)
    pub path: Vec<Vec3>,
    /// Current index in the path
    pub path_index: usize,
}

impl BuildingStructureBehavior {
    /// Create a new BuildingStructureBehavior targeting a location
    pub fn new(target_location: Vec3, object_to_build: ObjectEnum) -> Self {
        Self {
            target_location,
            object_to_build,
            arrived: false,
            path: Vec::new(),
            path_index: 0,
        }
    }

    /// Create with an initial path
    pub fn with_path(target_location: Vec3, object_to_build: ObjectEnum, path: Vec<Vec3>) -> Self {
        Self {
            target_location,
            object_to_build,
            arrived: false,
            path,
            path_index: 0,
        }
    }
}

/// Phase of the gathering resource behavior cycle.
#[derive(Clone, Debug, PartialEq)]
pub enum GatherPhase {
    /// Moving to the resource source
    MovingToResource,
    /// Extracting resources (frame counter)
    Extracting { frames_remaining: u32 },
    /// Moving to tunnel for drop-off
    MovingToTunnel { tunnel_entity: Entity, side_position: Vec3 },
    /// Dropping off resources at tunnel (frame counter)
    DroppingOff { tunnel_entity: Entity, frames_remaining: u32 },
}

/// Marker component for the GatheringResource behavior.
/// Encompasses the full gather-deliver cycle: approach resource -> extract ->
/// travel to nearest own Tunnel -> drop off.
#[derive(Component, Clone, Debug)]
pub struct GatheringResourceBehavior {
    /// Target resource entity (SpaceCrystalPatch or SupplyDeliveryStation)
    pub target_resource: Entity,
    /// Current phase of the gathering cycle
    pub phase: GatherPhase,
    /// Path to current target
    pub path: Vec<Vec3>,
    pub path_index: usize,
}

impl GatheringResourceBehavior {
    pub fn new(target_resource: Entity) -> Self {
        Self {
            target_resource,
            phase: GatherPhase::MovingToResource,
            path: Vec::new(),
            path_index: 0,
        }
    }
}

/// Phase of the drop-off resources behavior.
#[derive(Clone, Debug, PartialEq)]
pub enum DropOffPhase {
    /// Moving to the tunnel's appropriate side
    MovingToTunnel,
    /// Dropping off resources (frame counter)
    DroppingOff { frames_remaining: u32 },
}

/// Marker component for the DroppingOffResources behavior.
/// Moves to the target Tunnel's appropriate side based on carried resource type,
/// then performs drop-off.
#[derive(Component, Clone, Debug)]
pub struct DroppingOffResourcesBehavior {
    /// Target tunnel entity
    pub target_tunnel: Entity,
    /// Current phase
    pub phase: DropOffPhase,
    /// Path to tunnel side
    pub path: Vec<Vec3>,
    pub path_index: usize,
}

impl DroppingOffResourcesBehavior {
    pub fn new(target_tunnel: Entity) -> Self {
        Self {
            target_tunnel,
            phase: DropOffPhase::MovingToTunnel,
            path: Vec::new(),
            path_index: 0,
        }
    }
}

/// Phase of the tunnel building behavior.
#[derive(Clone, Debug, PartialEq)]
pub enum BuildTunnelPhase {
    /// Moving to the build location
    MovingToSite,
    /// Constructing — Agent is embedded, tunnel entity exists with ConstructionHP
    Constructing {
        /// The partially-built tunnel entity
        tunnel_entity: Entity,
        /// Frames of construction elapsed
        frames_elapsed: u32,
    },
}

/// Marker component for the BuildingTunnel behavior.
/// When present, a dedicated system moves the Agent to the build site,
/// spawns a partially-built Tunnel, embeds the Agent, and ticks construction.
#[derive(Component, Clone, Debug)]
pub struct BuildingTunnelBehavior {
    /// Target world-space location to build at
    pub target_location: Vec3,
    /// Current phase
    pub phase: BuildTunnelPhase,
    /// Precomputed path to target
    pub path: Vec<Vec3>,
    /// Current index in the path
    pub path_index: usize,
}

impl BuildingTunnelBehavior {
    pub fn new(target_location: Vec3) -> Self {
        Self {
            target_location,
            phase: BuildTunnelPhase::MovingToSite,
            path: Vec::new(),
            path_index: 0,
        }
    }
}

// === Supply Chopper Behaviors ===

/// Phase of the PickUpSupplies behavior.
#[derive(Clone, Debug, PartialEq)]
pub enum PickUpPhase {
    /// Moving to the Supply Delivery Station
    MovingToSDS,
    /// Transferring supplies from SDS (frame counter)
    Transferring { frames_remaining: u32 },
}

/// Marker component for the PickUpSupplies behavior.
/// Moves chopper to the target SDS, transfers supplies, then returns to attached tower or idles.
#[derive(Component, Clone, Debug)]
pub struct PickingUpSuppliesBehavior {
    /// Target Supply Delivery Station entity
    pub target_sds: Entity,
    /// Current phase
    pub phase: PickUpPhase,
}

impl PickingUpSuppliesBehavior {
    pub fn new(target_sds: Entity) -> Self {
        Self {
            target_sds,
            phase: PickUpPhase::MovingToSDS,
        }
    }
}

/// Marker component for the AttachToTower behavior.
/// Moves chopper to the target Supply Tower and establishes attachment on arrival.
#[derive(Component, Clone, Debug)]
pub struct AttachingToTowerBehavior {
    /// Target Supply Tower entity
    pub target_tower: Entity,
}

impl AttachingToTowerBehavior {
    pub fn new(target_tower: Entity) -> Self {
        Self { target_tower }
    }
}

/// Marker component for the DropOffSupplies behavior.
/// Moves chopper to the target Supply Tower and delivers carried supplies on arrival.
#[derive(Component, Clone, Debug)]
pub struct DroppingOffSuppliesBehavior {
    /// Target Supply Tower entity
    pub target_tower: Entity,
}

impl DroppingOffSuppliesBehavior {
    pub fn new(target_tower: Entity) -> Self {
        Self { target_tower }
    }
}

/// Marker component for units that are inside the Tunnel Network.
/// Placed on unit entities when they complete the EnteringTunnel behavior.
/// The entity is despawned from the map but logically exists in the network.
#[derive(Component, Clone, Debug)]
pub struct InTunnelNetwork {
    /// Player ID of the network owner
    pub owner_player: u8,
}

#[cfg(test)]
mod tests {
    use super::*;

    // === BaseBehaviorState tests ===

    #[test]
    fn base_behavior_state_default_is_none() {
        let state = BaseBehaviorState::default();
        assert!(matches!(state, BaseBehaviorState::None));
    }

    #[test]
    fn base_behavior_state_turn_rate_variant() {
        let state = BaseBehaviorState::TurnRate {
            planned_path: vec![Vec3::ZERO, Vec3::new(5.0, 0.0, 5.0)],
            path_index: 0,
        };
        if let BaseBehaviorState::TurnRate { planned_path, path_index } = &state {
            assert_eq!(planned_path.len(), 2);
            assert_eq!(*path_index, 0);
        } else {
            panic!("Expected TurnRate variant");
        }
    }

    #[test]
    fn base_behavior_state_fixed_turn_radius_variant() {
        let state = BaseBehaviorState::FixedTurnRadius {
            planned_path: vec![Vec3::ONE],
            path_index: 0,
        };
        assert!(matches!(state, BaseBehaviorState::FixedTurnRadius { .. }));
    }

    #[test]
    fn base_behavior_state_speed_turn_radius_variant() {
        let state = BaseBehaviorState::SpeedTurnRadius {
            planned_path: vec![],
            path_index: 0,
        };
        assert!(matches!(state, BaseBehaviorState::SpeedTurnRadius { .. }));
    }

    #[test]
    fn base_behavior_state_drag_variant_with_drift() {
        let state = BaseBehaviorState::Drag {
            planned_path: vec![Vec3::ZERO],
            path_index: 0,
            drift_velocity: Vec3::new(1.0, 0.0, -0.5),
        };
        if let BaseBehaviorState::Drag { drift_velocity, .. } = &state {
            assert_eq!(*drift_velocity, Vec3::new(1.0, 0.0, -0.5));
        } else {
            panic!("Expected Drag variant");
        }
    }

    #[test]
    fn base_behavior_state_glider_variant_with_circling() {
        let state = BaseBehaviorState::Glider {
            planned_path: vec![Vec3::new(10.0, 5.0, 10.0)],
            path_index: 0,
            circling: true,
            strafe_target: Some(Vec3::new(5.0, 0.0, 5.0)),
        };
        if let BaseBehaviorState::Glider { circling, strafe_target, .. } = &state {
            assert!(*circling);
            assert!(strafe_target.is_some());
        } else {
            panic!("Expected Glider variant");
        }
    }

    #[test]
    fn base_behavior_state_has_variant_per_movement_model() {
        // 5 movement model variants + None = 6 total
        let variants: Vec<BaseBehaviorState> = vec![
            BaseBehaviorState::TurnRate { planned_path: vec![], path_index: 0 },
            BaseBehaviorState::FixedTurnRadius { planned_path: vec![], path_index: 0 },
            BaseBehaviorState::SpeedTurnRadius { planned_path: vec![], path_index: 0 },
            BaseBehaviorState::Drag { planned_path: vec![], path_index: 0, drift_velocity: Vec3::ZERO },
            BaseBehaviorState::Glider { planned_path: vec![], path_index: 0, circling: false, strafe_target: None },
            BaseBehaviorState::None,
        ];
        assert_eq!(variants.len(), 6);
    }

    // === TurretBehaviorState tests ===

    #[test]
    fn turret_behavior_state_default() {
        let state = TurretBehaviorState::default();
        assert_eq!(state.scan_direction, 0.0);
        assert!(!state.scan_clockwise);
        assert!(state.last_known_target_pos.is_none());
    }

    #[test]
    fn turret_behavior_state_with_scan_data() {
        let state = TurretBehaviorState {
            scan_direction: std::f32::consts::PI,
            scan_clockwise: true,
            last_known_target_pos: Some(Vec3::new(10.0, 0.0, 10.0)),
        };
        assert_eq!(state.scan_direction, std::f32::consts::PI);
        assert!(state.scan_clockwise);
        assert_eq!(state.last_known_target_pos, Some(Vec3::new(10.0, 0.0, 10.0)));
    }

    // === LocomotionChannel tests ===

    #[test]
    fn locomotion_channel_default_is_stationary() {
        let channel = LocomotionChannel::default();
        assert!(matches!(channel, LocomotionChannel::Stationary));
    }

    #[test]
    fn locomotion_channel_moving_with_path() {
        let path = vec![Vec3::new(1.0, 0.0, 0.0), Vec3::new(5.0, 0.0, 5.0)];
        let channel = LocomotionChannel::Moving(path.clone());
        if let LocomotionChannel::Moving(waypoints) = &channel {
            assert_eq!(waypoints.len(), 2);
        } else {
            panic!("Expected Moving variant");
        }
    }

    #[test]
    fn locomotion_channel_reversing() {
        let channel = LocomotionChannel::Reversing(vec![Vec3::ZERO]);
        assert!(matches!(channel, LocomotionChannel::Reversing(_)));
    }

    #[test]
    fn locomotion_channel_stopping() {
        let channel = LocomotionChannel::Stopping;
        assert!(matches!(channel, LocomotionChannel::Stopping));
    }

    // === OrientationChannel tests ===

    #[test]
    fn orientation_channel_default_is_maintaining() {
        let channel = OrientationChannel::default();
        assert!(matches!(channel, OrientationChannel::Maintaining));
    }

    #[test]
    fn orientation_channel_turning_to_target() {
        let target = Vec3::new(10.0, 0.0, 5.0);
        let channel = OrientationChannel::Turning(target);
        if let OrientationChannel::Turning(pos) = channel {
            assert_eq!(pos, target);
        } else {
            panic!("Expected Turning variant");
        }
    }

    // === BaseAttackChannel tests ===

    #[test]
    fn base_attack_channel_default_is_none() {
        let channel = BaseAttackChannel::default();
        assert!(matches!(channel, BaseAttackChannel::None));
    }

    #[test]
    fn base_attack_channel_aiming_at_target() {
        let target = Entity::from_raw_u32(42).unwrap();
        let channel = BaseAttackChannel::Aiming(target);
        if let BaseAttackChannel::Aiming(e) = channel {
            assert_eq!(e, Entity::from_raw_u32(42).unwrap());
        } else {
            panic!("Expected Aiming variant");
        }
    }

    #[test]
    fn base_attack_channel_firing_at_target() {
        let channel = BaseAttackChannel::Firing(Entity::from_raw_u32(10).unwrap());
        assert!(matches!(channel, BaseAttackChannel::Firing(_)));
    }

    #[test]
    fn base_attack_channel_cooldown_and_reloading() {
        let cooldown = BaseAttackChannel::Cooldown;
        let reloading = BaseAttackChannel::Reloading;
        assert!(matches!(cooldown, BaseAttackChannel::Cooldown));
        assert!(matches!(reloading, BaseAttackChannel::Reloading));
    }

    // === TurretOrientationChannel tests ===

    #[test]
    fn turret_orientation_channel_default_is_maintaining() {
        let channel = TurretOrientationChannel::default();
        assert!(matches!(channel, TurretOrientationChannel::Maintaining));
    }

    #[test]
    fn turret_orientation_channel_turning() {
        let target = Vec3::new(3.0, 0.0, 7.0);
        let channel = TurretOrientationChannel::Turning(target);
        if let TurretOrientationChannel::Turning(pos) = channel {
            assert_eq!(pos, target);
        } else {
            panic!("Expected Turning variant");
        }
    }

    // === TurretAttackChannel tests ===

    #[test]
    fn turret_attack_channel_default_is_inactive() {
        let channel = TurretAttackChannel::default();
        assert!(matches!(channel, TurretAttackChannel::Inactive));
    }

    #[test]
    fn turret_attack_channel_aiming() {
        let channel = TurretAttackChannel::Aiming(Entity::from_raw_u32(99).unwrap());
        assert!(matches!(channel, TurretAttackChannel::Aiming(_)));
    }

    #[test]
    fn turret_attack_channel_firing() {
        let channel = TurretAttackChannel::Firing(Entity::from_raw_u32(5).unwrap());
        assert!(matches!(channel, TurretAttackChannel::Firing(_)));
    }

    #[test]
    fn turret_attack_channel_cooldown_and_reloading() {
        let cooldown = TurretAttackChannel::Cooldown;
        let reloading = TurretAttackChannel::Reloading;
        assert!(matches!(cooldown, TurretAttackChannel::Cooldown));
        assert!(matches!(reloading, TurretAttackChannel::Reloading));
    }

    // === Channel independence tests ===

    #[test]
    fn locomotion_and_orientation_can_hold_independent_states() {
        let locomotion = LocomotionChannel::Moving(vec![Vec3::new(10.0, 0.0, 10.0)]);
        let orientation = OrientationChannel::Turning(Vec3::new(5.0, 0.0, 5.0));

        // Both can hold different states simultaneously
        assert!(matches!(locomotion, LocomotionChannel::Moving(_)));
        assert!(matches!(orientation, OrientationChannel::Turning(_)));
    }

    #[test]
    fn turret_channels_independent_of_base_channels() {
        let base_orientation = OrientationChannel::Maintaining;
        let turret_orientation = TurretOrientationChannel::Turning(Vec3::ONE);
        let turret_attack = TurretAttackChannel::Aiming(Entity::from_raw_u32(1).unwrap());

        // Turret can be active while base is maintaining
        assert!(matches!(base_orientation, OrientationChannel::Maintaining));
        assert!(matches!(turret_orientation, TurretOrientationChannel::Turning(_)));
        assert!(matches!(turret_attack, TurretAttackChannel::Aiming(_)));
    }

    // === EnteringTunnelBehavior tests ===

    #[test]
    fn entering_tunnel_behavior_new() {
        let tunnel = Entity::from_raw_u32(42).unwrap();
        let behavior = EnteringTunnelBehavior::new(tunnel);
        assert_eq!(behavior.target_tunnel, Entity::from_raw_u32(42).unwrap());
        assert!(behavior.path.is_empty());
        assert_eq!(behavior.path_index, 0);
    }

    #[test]
    fn entering_tunnel_behavior_with_path() {
        let tunnel = Entity::from_raw_u32(10).unwrap();
        let path = vec![Vec3::new(1.0, 0.0, 1.0), Vec3::new(5.0, 0.0, 5.0)];
        let behavior = EnteringTunnelBehavior::with_path(tunnel, path.clone());
        assert_eq!(behavior.target_tunnel, Entity::from_raw_u32(10).unwrap());
        assert_eq!(behavior.path.len(), 2);
        assert_eq!(behavior.path_index, 0);
    }

    #[test]
    fn entering_tunnel_behavior_stores_tunnel_entity() {
        let tunnel = Entity::from_raw_u32(99).unwrap();
        let behavior = EnteringTunnelBehavior::new(tunnel);
        assert_eq!(behavior.target_tunnel, tunnel);
    }

    #[test]
    fn entering_tunnel_behavior_is_component() {
        // Verify it can be used as a Bevy component
        let mut world = World::new();
        let entity = world.spawn(EnteringTunnelBehavior::new(Entity::from_raw_u32(1).unwrap())).id();
        let behavior = world.entity(entity).get::<EnteringTunnelBehavior>().unwrap();
        assert_eq!(behavior.target_tunnel, Entity::from_raw_u32(1).unwrap());
    }

    // === InTunnelNetwork tests ===

    #[test]
    fn in_tunnel_network_stores_owner() {
        let marker = InTunnelNetwork { owner_player: 1 };
        assert_eq!(marker.owner_player, 1);
    }

    #[test]
    fn in_tunnel_network_is_component() {
        let mut world = World::new();
        let entity = world.spawn(InTunnelNetwork { owner_player: 2 }).id();
        let marker = world.entity(entity).get::<InTunnelNetwork>().unwrap();
        assert_eq!(marker.owner_player, 2);
    }

    #[test]
    fn in_tunnel_network_different_players() {
        let p1 = InTunnelNetwork { owner_player: 0 };
        let p2 = InTunnelNetwork { owner_player: 1 };
        assert_ne!(p1.owner_player, p2.owner_player);
    }

    // === BuildingStructureBehavior tests ===

    #[test]
    fn building_structure_behavior_new() {
        let behavior = BuildingStructureBehavior::new(
            Vec3::new(10.0, 0.0, 10.0),
            ObjectEnum::Tunnel,
        );
        assert_eq!(behavior.target_location, Vec3::new(10.0, 0.0, 10.0));
        assert_eq!(behavior.object_to_build, ObjectEnum::Tunnel);
        assert!(!behavior.arrived);
        assert!(behavior.path.is_empty());
        assert_eq!(behavior.path_index, 0);
    }

    #[test]
    fn building_structure_behavior_with_path() {
        let path = vec![Vec3::new(1.0, 0.0, 1.0), Vec3::new(10.0, 0.0, 10.0)];
        let behavior = BuildingStructureBehavior::with_path(
            Vec3::new(10.0, 0.0, 10.0),
            ObjectEnum::Tunnel,
            path.clone(),
        );
        assert_eq!(behavior.path.len(), 2);
        assert!(!behavior.arrived);
        assert_eq!(behavior.path_index, 0);
    }

    #[test]
    fn building_structure_behavior_is_component() {
        let mut world = World::new();
        let entity = world.spawn(BuildingStructureBehavior::new(
            Vec3::ZERO, ObjectEnum::Tunnel,
        )).id();
        let b = world.entity(entity).get::<BuildingStructureBehavior>().unwrap();
        assert_eq!(b.object_to_build, ObjectEnum::Tunnel);
    }

    #[test]
    fn building_structure_behavior_arrived_defaults_false() {
        let behavior = BuildingStructureBehavior::new(Vec3::ZERO, ObjectEnum::Tunnel);
        assert!(!behavior.arrived);
    }

    #[test]
    fn building_structure_behavior_stores_object_type() {
        let behavior = BuildingStructureBehavior::new(Vec3::ZERO, ObjectEnum::Tunnel);
        assert_eq!(behavior.object_to_build, ObjectEnum::Tunnel);
    }

    // === GatheringResourceBehavior tests ===

    #[test]
    fn gathering_resource_behavior_new() {
        let target = Entity::from_raw_u32(5).unwrap();
        let behavior = GatheringResourceBehavior::new(target);
        assert_eq!(behavior.target_resource, target);
        assert_eq!(behavior.phase, GatherPhase::MovingToResource);
        assert!(behavior.path.is_empty());
        assert_eq!(behavior.path_index, 0);
    }

    #[test]
    fn gathering_resource_behavior_is_component() {
        let mut world = World::new();
        let target = Entity::from_raw_u32(10).unwrap();
        let entity = world.spawn(GatheringResourceBehavior::new(target)).id();
        let b = world.entity(entity).get::<GatheringResourceBehavior>().unwrap();
        assert_eq!(b.target_resource, Entity::from_raw_u32(10).unwrap());
    }

    #[test]
    fn gather_phase_extracting_tracks_frames() {
        let phase = GatherPhase::Extracting { frames_remaining: 48 };
        assert_eq!(phase, GatherPhase::Extracting { frames_remaining: 48 });
    }

    #[test]
    fn gather_phase_moving_to_tunnel_stores_data() {
        let phase = GatherPhase::MovingToTunnel {
            tunnel_entity: Entity::from_raw_u32(1).unwrap(),
            side_position: Vec3::new(5.0, 0.0, 5.0),
        };
        if let GatherPhase::MovingToTunnel { tunnel_entity, side_position } = phase {
            assert_eq!(tunnel_entity, Entity::from_raw_u32(1).unwrap());
            assert_eq!(side_position, Vec3::new(5.0, 0.0, 5.0));
        } else {
            panic!("Expected MovingToTunnel");
        }
    }

    #[test]
    fn gather_phase_dropping_off_tracks_frames() {
        let tunnel = Entity::from_raw_u32(99).unwrap();
        let phase = GatherPhase::DroppingOff { tunnel_entity: tunnel, frames_remaining: 24 };
        assert_eq!(phase, GatherPhase::DroppingOff { tunnel_entity: tunnel, frames_remaining: 24 });
    }

    // === DroppingOffResourcesBehavior tests ===

    #[test]
    fn dropping_off_resources_behavior_new() {
        let tunnel = Entity::from_raw_u32(42).unwrap();
        let behavior = DroppingOffResourcesBehavior::new(tunnel);
        assert_eq!(behavior.target_tunnel, tunnel);
        assert_eq!(behavior.phase, DropOffPhase::MovingToTunnel);
        assert!(behavior.path.is_empty());
        assert_eq!(behavior.path_index, 0);
    }

    #[test]
    fn dropping_off_resources_behavior_is_component() {
        let mut world = World::new();
        let tunnel = Entity::from_raw_u32(7).unwrap();
        let entity = world.spawn(DroppingOffResourcesBehavior::new(tunnel)).id();
        let b = world.entity(entity).get::<DroppingOffResourcesBehavior>().unwrap();
        assert_eq!(b.target_tunnel, Entity::from_raw_u32(7).unwrap());
    }

    #[test]
    fn drop_off_phase_moving_is_default() {
        let phase = DropOffPhase::MovingToTunnel;
        assert_eq!(phase, DropOffPhase::MovingToTunnel);
    }

    #[test]
    fn drop_off_phase_dropping_off_tracks_frames() {
        let phase = DropOffPhase::DroppingOff { frames_remaining: 48 };
        assert_eq!(phase, DropOffPhase::DroppingOff { frames_remaining: 48 });
    }

    // === BuildingTunnelBehavior tests ===

    #[test]
    fn building_tunnel_behavior_new() {
        let target = Vec3::new(10.0, 0.0, 10.0);
        let behavior = BuildingTunnelBehavior::new(target);
        assert_eq!(behavior.target_location, target);
        assert_eq!(behavior.phase, BuildTunnelPhase::MovingToSite);
        assert!(behavior.path.is_empty());
        assert_eq!(behavior.path_index, 0);
    }

    #[test]
    fn building_tunnel_behavior_is_component() {
        let mut world = World::new();
        let target = Vec3::new(5.0, 0.0, 5.0);
        let entity = world.spawn(BuildingTunnelBehavior::new(target)).id();
        let b = world.entity(entity).get::<BuildingTunnelBehavior>().unwrap();
        assert_eq!(b.target_location, target);
    }

    #[test]
    fn build_tunnel_phase_constructing_equality() {
        let phase_a = BuildTunnelPhase::Constructing {
            tunnel_entity: Entity::from_raw_u32(1).unwrap(),
            frames_elapsed: 50,
        };
        let phase_b = BuildTunnelPhase::Constructing {
            tunnel_entity: Entity::from_raw_u32(1).unwrap(),
            frames_elapsed: 50,
        };
        assert_eq!(phase_a, phase_b);
    }

    #[test]
    fn build_tunnel_phase_constructing_not_equal_different_frames() {
        let phase_a = BuildTunnelPhase::Constructing {
            tunnel_entity: Entity::from_raw_u32(1).unwrap(),
            frames_elapsed: 50,
        };
        let phase_b = BuildTunnelPhase::Constructing {
            tunnel_entity: Entity::from_raw_u32(1).unwrap(),
            frames_elapsed: 100,
        };
        assert_ne!(phase_a, phase_b);
    }

    #[test]
    fn build_tunnel_phase_moving_not_equal_constructing() {
        let phase_a = BuildTunnelPhase::MovingToSite;
        let phase_b = BuildTunnelPhase::Constructing {
            tunnel_entity: Entity::from_raw_u32(1).unwrap(),
            frames_elapsed: 0,
        };
        assert_ne!(phase_a, phase_b);
    }

    // === PickingUpSuppliesBehavior tests ===

    #[test]
    fn picking_up_supplies_behavior_new() {
        let target = Entity::from_raw_u32(42).unwrap();
        let behavior = PickingUpSuppliesBehavior::new(target);
        assert_eq!(behavior.target_sds, target);
        assert_eq!(behavior.phase, PickUpPhase::MovingToSDS);
    }

    #[test]
    fn pick_up_phase_transferring_tracks_frames() {
        let phase = PickUpPhase::Transferring { frames_remaining: 30 };
        assert_eq!(phase, PickUpPhase::Transferring { frames_remaining: 30 });
    }

    #[test]
    fn pick_up_phase_moving_not_equal_transferring() {
        assert_ne!(
            PickUpPhase::MovingToSDS,
            PickUpPhase::Transferring { frames_remaining: 0 },
        );
    }

    // === AttachingToTowerBehavior tests ===

    #[test]
    fn attaching_to_tower_behavior_new() {
        let target = Entity::from_raw_u32(7).unwrap();
        let behavior = AttachingToTowerBehavior::new(target);
        assert_eq!(behavior.target_tower, target);
    }

    // === DroppingOffSuppliesBehavior tests ===

    #[test]
    fn dropping_off_supplies_behavior_new() {
        let target = Entity::from_raw_u32(99).unwrap();
        let behavior = DroppingOffSuppliesBehavior::new(target);
        assert_eq!(behavior.target_tower, target);
    }
}
