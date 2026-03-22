#![allow(dead_code)]
use bevy::prelude::*;
use crate::types::{FactionEnum, UnitBaseEnum, AttackTypeEnum, FullyConnectedSubtype, TargetDomainEnum, TargetTypeEnum};

/// Component storing the unit's name (display identity)
#[derive(Component)]
pub struct UnitType {
    pub name: String,
}

/// Static data defining a specific unit type (e.g., Peacekeeper, Tank)
#[derive(Clone, Debug)]
pub struct UnitTypeData {
    pub faction: FactionEnum,
    pub silhouette_width: u32,
    pub silhouette_height: u32,
    pub max_hp: u32,
    pub point_armor: u32,
    pub full_armor: u32,
    pub unit_base: UnitBaseEnum,
}

/// Turret attributes for units with turrets.
/// Can be used as static data or as an ECS Component on turret-bearing entities.
#[derive(Component, Clone, Debug)]
pub struct TurretAttributesData {
    /// Full arc angle in degrees (max 360), centered on unit facing
    pub turn_angle: f32,
    /// Turret rotation speed in degrees per frame
    pub turn_rate: f32,
}

impl TurretAttributesData {
    /// Validate that turn_angle is within the valid range [0, 360]
    pub fn validate(&self) -> Result<(), String> {
        if self.turn_angle < 0.0 || self.turn_angle > 360.0 {
            return Err(format!(
                "turn_angle must be between 0 and 360, got {}",
                self.turn_angle
            ));
        }
        if self.turn_rate < 0.0 {
            return Err(format!(
                "turn_rate must be non-negative, got {}",
                self.turn_rate
            ));
        }
        Ok(())
    }

    /// Half-arc angle in degrees (the arc extends this far on each side of unit facing)
    pub fn half_angle(&self) -> f32 {
        self.turn_angle / 2.0
    }

    /// Whether the turret has full 360-degree rotation
    pub fn is_full_rotation(&self) -> bool {
        (self.turn_angle - 360.0).abs() < f32::EPSILON
    }
}

/// Attack attributes defining how a unit attacks
#[derive(Clone, Debug)]
pub struct AttackAttributesData {
    pub attack_type: AttackTypeEnum,
    /// FullyConnected subtype — Ranged or Melee. None for non-FullyConnected attack types.
    pub fc_subtype: Option<FullyConnectedSubtype>,
    pub target_domain: TargetDomainEnum,
    pub target_type: TargetTypeEnum,
    /// Area of effect radius in grid units (None for single-target)
    pub aoe_radius: Option<u32>,
    /// Damage per hit
    pub damage: u32,
    /// Maximum attack range in grid units
    pub range: u32,
    /// Minimum attack range in grid units
    pub min_range: u32,
    /// Projectile speed in space units per frame (None for instant-hit)
    pub projectile_speed: Option<f32>,
    /// Frames to aim before firing
    pub aim_duration: u32,
    /// Frames of firing animation
    pub firing_duration: u32,
    /// Frames of cooldown after firing
    pub cooldown_duration: u32,
    /// Frames of reload after cooldown
    pub reload_duration: u32,
}

/// Mesh type for units (visual representation)
pub enum UnitMeshType {
    Capsule,
    Cube,
}

/// Component for unit control cost (contributes to unit control cap)
#[derive(Component, Clone, Debug)]
pub struct UnitControlCost(pub u32);

/// Component for LightInfantry-specific rugged terrain defense bonus
#[derive(Component, Clone, Debug)]
pub struct RuggedTerrainDefenseBonus(pub f32);

/// Component for Syndicate unit tunnel space cost (how much tunnel capacity a unit consumes)
#[derive(Component, Clone, Debug)]
pub struct TunnelSpaceCost(pub u32);

/// Component that tracks which Recruitment Center(s) produced a Cults unit.
/// Used by the death tracking system to decrement each originating center's `local_used`.
#[derive(Component, Clone, Debug)]
pub struct OriginatingCenters {
    pub centers: Vec<Entity>,
}

// === Peacekeeper Definition ===

use crate::simulation::FRAMES_PER_SECOND;

/// Peacekeeper static type data
pub fn peacekeeper_type_data() -> UnitTypeData {
    UnitTypeData {
        faction: FactionEnum::GlobalDefenseOrdinance,
        silhouette_width: 24,
        silhouette_height: 24,
        max_hp: 50,
        point_armor: 1,
        full_armor: 1,
        unit_base: UnitBaseEnum::LightInfantry,
    }
}

/// Peacekeeper attack attributes (design-spec, frame-based)
pub fn peacekeeper_attack_data() -> AttackAttributesData {
    AttackAttributesData {
        attack_type: AttackTypeEnum::FullyConnected,
        fc_subtype: Some(FullyConnectedSubtype::Ranged),
        target_domain: TargetDomainEnum::Ground,
        target_type: TargetTypeEnum::SingleTarget,
        aoe_radius: None,
        damage: 10,
        range: 4,
        min_range: 0,
        projectile_speed: None, // FullyConnected = instant hit
        aim_duration: 2,
        firing_duration: 1,
        cooldown_duration: 2,
        reload_duration: 12,
    }
}

/// Peacekeeper unit control cost
pub const PEACEKEEPER_CONTROL_COST: u32 = 1;

/// Peacekeeper rugged terrain defense bonus
pub const PEACEKEEPER_RUGGED_BONUS: f32 = 0.5;

// === Syndicate Agent Definition ===

/// Agent static type data
pub fn agent_type_data() -> UnitTypeData {
    UnitTypeData {
        faction: FactionEnum::TheSyndicate,
        silhouette_width: 36,
        silhouette_height: 36,
        max_hp: 75,
        point_armor: 1,
        full_armor: 1,
        unit_base: UnitBaseEnum::HeavyInfantry,
    }
}

/// Agent attack attributes (design-spec, frame-based)
pub fn agent_attack_data() -> AttackAttributesData {
    AttackAttributesData {
        attack_type: AttackTypeEnum::FullyConnected,
        fc_subtype: Some(FullyConnectedSubtype::Melee),
        target_domain: TargetDomainEnum::Ground,
        target_type: TargetTypeEnum::SingleTarget,
        aoe_radius: None,
        damage: 6,
        range: 0, // Melee — actual range uses MELEE_RANGE constant at spawn time
        min_range: 0,
        projectile_speed: None, // FullyConnected = instant hit
        aim_duration: 2,
        firing_duration: 4,
        cooldown_duration: 1,
        reload_duration: 9,
    }
}

/// Agent mining duration at a Space Crystal Patch (frames)
pub const AGENT_MINING_DURATION: u32 = 48;

/// Agent pickup duration at a Supply Delivery Station (frames)
pub const AGENT_PICKUP_DURATION: u32 = 48;

/// Agent drop-off duration at a Tunnel side (frames)
pub const AGENT_DROPOFF_DURATION: u32 = 48;

/// Amount of Space Crystals an Agent picks up per gather cycle
pub const AGENT_CRYSTAL_CARRY: u32 = 50;

/// Amount of Supplies an Agent picks up per gather cycle
pub const AGENT_SUPPLY_CARRY: u32 = 1;

/// Agent unit control cost
pub const AGENT_CONTROL_COST: u32 = 1;

/// Agent tunnel space cost (occupies 2 tunnel space when inside the network)
pub const AGENT_TUNNEL_SPACE_COST: u32 = 2;

/// Agent tunnel construction duration in simulation frames (480 frames = 30 seconds at 16 FPS)
pub const AGENT_TUNNEL_BUILD_FRAMES: u32 = 480;

/// Agent rugged terrain defense bonus (HeavyInfantry has rugged_terrain: true)
pub const AGENT_RUGGED_BONUS: f32 = 0.5;

// === Syndicate Guard Definition ===

/// Guard static type data
pub fn guard_type_data() -> UnitTypeData {
    UnitTypeData {
        faction: FactionEnum::TheSyndicate,
        silhouette_width: 36,
        silhouette_height: 36,
        max_hp: 80,
        point_armor: 1,
        full_armor: 1,
        unit_base: UnitBaseEnum::HeavyInfantry,
    }
}

/// Guard attack attributes (design-spec, frame-based)
pub fn guard_attack_data() -> AttackAttributesData {
    AttackAttributesData {
        attack_type: AttackTypeEnum::FullyConnected,
        fc_subtype: Some(FullyConnectedSubtype::Ranged), // RANGED, not Melee like Agent
        target_domain: TargetDomainEnum::Ground,
        target_type: TargetTypeEnum::SingleTarget,
        aoe_radius: None,
        damage: 6,
        range: 3, // 3 grid units (unlike Agent's 0 for melee)
        min_range: 0,
        projectile_speed: None, // FullyConnected = instant hit
        aim_duration: 2,
        firing_duration: 1,
        cooldown_duration: 1,
        reload_duration: 4,
    }
}

/// Guard unit control cost
pub const GUARD_CONTROL_COST: u32 = 1;

/// Guard tunnel space cost (occupies 2 tunnel space when inside the network)
pub const GUARD_TUNNEL_SPACE_COST: u32 = 2;

/// Guard rugged terrain defense bonus (HeavyInfantry has rugged_terrain: true)
pub const GUARD_RUGGED_BONUS: f32 = 0.5;

/// Convert frame-based duration to seconds using simulation FPS
pub fn frames_to_seconds(frames: u32) -> f32 {
    frames as f32 / FRAMES_PER_SECOND as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn peacekeeper_type_data_fields() {
        let data = peacekeeper_type_data();
        assert_eq!(data.faction, FactionEnum::GlobalDefenseOrdinance);
        assert_eq!(data.silhouette_width, 24);
        assert_eq!(data.silhouette_height, 24);
        assert_eq!(data.max_hp, 50);
        assert_eq!(data.point_armor, 1);
        assert_eq!(data.full_armor, 1);
        assert_eq!(data.unit_base, UnitBaseEnum::LightInfantry);
    }

    #[test]
    fn peacekeeper_attack_data_fields() {
        let data = peacekeeper_attack_data();
        assert_eq!(data.attack_type, AttackTypeEnum::FullyConnected);
        assert_eq!(data.fc_subtype, Some(FullyConnectedSubtype::Ranged));
        assert_eq!(data.target_domain, TargetDomainEnum::Ground);
        assert_eq!(data.target_type, TargetTypeEnum::SingleTarget);
        assert!(data.aoe_radius.is_none());
        assert_eq!(data.damage, 10);
        assert_eq!(data.range, 4);
        assert_eq!(data.min_range, 0);
        assert!(data.projectile_speed.is_none());
    }

    #[test]
    fn turret_attributes_valid_full_rotation() {
        let turret = TurretAttributesData {
            turn_angle: 360.0,
            turn_rate: 10.0,
        };
        assert!(turret.validate().is_ok());
        assert!(turret.is_full_rotation());
        assert_eq!(turret.half_angle(), 180.0);
    }

    #[test]
    fn turret_attributes_valid_limited_rotation() {
        let turret = TurretAttributesData {
            turn_angle: 90.0,
            turn_rate: 5.0,
        };
        assert!(turret.validate().is_ok());
        assert!(!turret.is_full_rotation());
        assert_eq!(turret.half_angle(), 45.0);
    }

    #[test]
    fn turret_attributes_invalid_angle_over_360() {
        let turret = TurretAttributesData {
            turn_angle: 400.0,
            turn_rate: 10.0,
        };
        assert!(turret.validate().is_err());
    }

    #[test]
    fn turret_attributes_invalid_negative_angle() {
        let turret = TurretAttributesData {
            turn_angle: -10.0,
            turn_rate: 10.0,
        };
        assert!(turret.validate().is_err());
    }

    #[test]
    fn turret_attributes_invalid_negative_turn_rate() {
        let turret = TurretAttributesData {
            turn_angle: 180.0,
            turn_rate: -1.0,
        };
        assert!(turret.validate().is_err());
    }

    #[test]
    fn turret_attributes_zero_angle_valid() {
        let turret = TurretAttributesData {
            turn_angle: 0.0,
            turn_rate: 0.0,
        };
        assert!(turret.validate().is_ok());
        assert!(!turret.is_full_rotation());
    }

    #[test]
    fn frames_to_seconds_conversion() {
        // FRAMES_PER_SECOND = 16
        assert_eq!(frames_to_seconds(16), 1.0);
        assert_eq!(frames_to_seconds(0), 0.0);
        assert_eq!(frames_to_seconds(8), 0.5);
    }

    // --- Peacekeeper attack timing values ---

    #[test]
    fn peacekeeper_attack_timing_aim_duration() {
        let data = peacekeeper_attack_data();
        assert_eq!(data.aim_duration, 2);
    }

    #[test]
    fn peacekeeper_attack_timing_firing_duration() {
        let data = peacekeeper_attack_data();
        assert_eq!(data.firing_duration, 1);
    }

    #[test]
    fn peacekeeper_attack_timing_cooldown_duration() {
        let data = peacekeeper_attack_data();
        assert_eq!(data.cooldown_duration, 2);
    }

    #[test]
    fn peacekeeper_attack_timing_reload_duration() {
        let data = peacekeeper_attack_data();
        assert_eq!(data.reload_duration, 12);
    }

    #[test]
    fn peacekeeper_attack_no_projectile_speed() {
        // FullyConnected = instant hit, no projectile
        let data = peacekeeper_attack_data();
        assert!(data.projectile_speed.is_none());
    }

    #[test]
    fn peacekeeper_attack_no_aoe() {
        let data = peacekeeper_attack_data();
        assert!(data.aoe_radius.is_none());
    }

    // --- Peacekeeper constants ---

    #[test]
    fn peacekeeper_control_cost_is_one() {
        assert_eq!(PEACEKEEPER_CONTROL_COST, 1);
    }

    #[test]
    fn peacekeeper_rugged_bonus_is_fifty_percent() {
        assert!((PEACEKEEPER_RUGGED_BONUS - 0.5).abs() < f32::EPSILON);
    }

    // --- Peacekeeper ObjectType cross-check ---

    #[test]
    fn peacekeeper_object_type_matches_ticket_spec() {
        use crate::types::ObjectEnum;
        let obj = ObjectEnum::Peacekeeper.object_type();
        assert_eq!(obj.name, "Peacekeeper");
        assert_eq!(obj.size, (24, 24)); // silhouette 24x24 SU
        assert!(obj.destructible);
        assert_eq!(obj.sight_range, 5);
        assert!(obj.groupable);
    }

    #[test]
    fn peacekeeper_is_unit_not_structure() {
        use crate::types::ObjectEnum;
        assert!(ObjectEnum::Peacekeeper.is_unit());
        assert!(!ObjectEnum::Peacekeeper.is_structure());
        assert!(!ObjectEnum::Peacekeeper.is_resource());
    }

    // --- LightInfantry base matches Peacekeeper requirements ---

    #[test]
    fn peacekeeper_light_infantry_no_turret() {
        let data = peacekeeper_type_data();
        let base_data = data.unit_base.data();
        assert!(!base_data.has_turret);
    }

    #[test]
    fn peacekeeper_light_infantry_cannot_reverse() {
        let data = peacekeeper_type_data();
        let base_data = data.unit_base.data();
        assert!(!base_data.can_reverse);
    }

    #[test]
    fn peacekeeper_light_infantry_ground_domain() {
        let data = peacekeeper_type_data();
        let base_data = data.unit_base.data();
        assert_eq!(base_data.domain, crate::types::DomainEnum::Ground);
    }

    // --- Movement speed derivation ---

    #[test]
    fn peacekeeper_movement_speed_derivation() {
        // 4 SU/frame * 16 FPS / 64 SU/GU = 1.0 GU/sec
        use crate::simulation::{FRAMES_PER_SECOND, SPACE_UNITS_PER_GRID_UNIT};
        let expected = 4.0 * (FRAMES_PER_SECOND as f32) / (SPACE_UNITS_PER_GRID_UNIT as f32);
        assert!((expected - 1.0).abs() < f32::EPSILON);
    }

    // === Syndicate Agent type data tests ===

    #[test]
    fn agent_type_data_fields() {
        let data = agent_type_data();
        assert_eq!(data.faction, FactionEnum::TheSyndicate);
        assert_eq!(data.silhouette_width, 36);
        assert_eq!(data.silhouette_height, 36);
        assert_eq!(data.max_hp, 75);
        assert_eq!(data.point_armor, 1);
        assert_eq!(data.full_armor, 1);
        assert_eq!(data.unit_base, UnitBaseEnum::HeavyInfantry);
    }

    #[test]
    fn agent_attack_data_fields() {
        let data = agent_attack_data();
        assert_eq!(data.attack_type, AttackTypeEnum::FullyConnected);
        assert_eq!(data.fc_subtype, Some(FullyConnectedSubtype::Melee));
        assert_eq!(data.target_domain, TargetDomainEnum::Ground);
        assert_eq!(data.target_type, TargetTypeEnum::SingleTarget);
        assert!(data.aoe_radius.is_none());
        assert_eq!(data.damage, 6);
        assert_eq!(data.min_range, 0);
        assert!(data.projectile_speed.is_none());
    }

    #[test]
    fn agent_attack_timing_aim_duration() {
        let data = agent_attack_data();
        assert_eq!(data.aim_duration, 2);
    }

    #[test]
    fn agent_attack_timing_firing_duration() {
        let data = agent_attack_data();
        assert_eq!(data.firing_duration, 4);
    }

    #[test]
    fn agent_attack_timing_cooldown_duration() {
        let data = agent_attack_data();
        assert_eq!(data.cooldown_duration, 1);
    }

    #[test]
    fn agent_attack_timing_reload_duration() {
        let data = agent_attack_data();
        assert_eq!(data.reload_duration, 9);
    }

    #[test]
    fn agent_attack_no_projectile_speed() {
        // FullyConnected Melee = instant hit, no projectile
        let data = agent_attack_data();
        assert!(data.projectile_speed.is_none());
    }

    #[test]
    fn agent_attack_no_aoe() {
        let data = agent_attack_data();
        assert!(data.aoe_radius.is_none());
    }

    #[test]
    fn agent_attack_is_melee_subtype() {
        let data = agent_attack_data();
        assert_eq!(data.fc_subtype, Some(FullyConnectedSubtype::Melee));
    }

    // --- Agent constants ---

    #[test]
    fn agent_control_cost_is_one() {
        assert_eq!(AGENT_CONTROL_COST, 1);
    }

    #[test]
    fn agent_tunnel_space_cost_is_two() {
        assert_eq!(AGENT_TUNNEL_SPACE_COST, 2);
    }

    #[test]
    fn agent_rugged_bonus_is_fifty_percent() {
        assert!((AGENT_RUGGED_BONUS - 0.5).abs() < f32::EPSILON);
    }

    // --- Agent ObjectType cross-check ---

    #[test]
    fn agent_object_type_matches_ticket_spec() {
        use crate::types::ObjectEnum;
        let obj = ObjectEnum::SyndicateAgent.object_type();
        assert_eq!(obj.name, "Agent");
        assert_eq!(obj.size, (36, 36)); // silhouette 36x36 SU
        assert!(obj.destructible);
        assert_eq!(obj.sight_range, 5);
        assert!(!obj.groupable); // Ungroupable — each Agent is its own SelectionGroup
    }

    #[test]
    fn agent_is_unit_not_structure() {
        use crate::types::ObjectEnum;
        assert!(ObjectEnum::SyndicateAgent.is_unit());
        assert!(!ObjectEnum::SyndicateAgent.is_structure());
        assert!(!ObjectEnum::SyndicateAgent.is_resource());
    }

    // --- HeavyInfantry base matches Agent requirements ---

    #[test]
    fn agent_heavy_infantry_no_turret() {
        let data = agent_type_data();
        let base_data = data.unit_base.data();
        assert!(!base_data.has_turret);
    }

    #[test]
    fn agent_heavy_infantry_cannot_reverse() {
        let data = agent_type_data();
        let base_data = data.unit_base.data();
        assert!(!base_data.can_reverse);
    }

    #[test]
    fn agent_heavy_infantry_ground_domain() {
        let data = agent_type_data();
        let base_data = data.unit_base.data();
        assert_eq!(base_data.domain, crate::types::DomainEnum::Ground);
    }

    #[test]
    fn agent_heavy_infantry_not_crushable() {
        let data = agent_type_data();
        let base_data = data.unit_base.data();
        assert!(!base_data.crushable);
    }

    #[test]
    fn agent_heavy_infantry_rugged_terrain() {
        let data = agent_type_data();
        let base_data = data.unit_base.data();
        assert!(base_data.rugged_terrain);
    }

    // --- Agent movement speed derivation ---

    #[test]
    fn agent_movement_speed_derivation() {
        // 6 SU/frame * 16 FPS / 64 SU/GU = 1.5 GU/sec
        use crate::simulation::{FRAMES_PER_SECOND, SPACE_UNITS_PER_GRID_UNIT};
        let expected = 6.0 * (FRAMES_PER_SECOND as f32) / (SPACE_UNITS_PER_GRID_UNIT as f32);
        assert!((expected - 1.5).abs() < f32::EPSILON);
    }

    // === Syndicate Guard type data tests ===

    #[test]
    fn guard_type_data_fields() {
        let data = guard_type_data();
        assert_eq!(data.faction, FactionEnum::TheSyndicate);
        assert_eq!(data.silhouette_width, 36);
        assert_eq!(data.silhouette_height, 36);
        assert_eq!(data.max_hp, 80);
        assert_eq!(data.point_armor, 1);
        assert_eq!(data.full_armor, 1);
        assert_eq!(data.unit_base, UnitBaseEnum::HeavyInfantry);
    }

    #[test]
    fn guard_attack_data_fields() {
        let data = guard_attack_data();
        assert_eq!(data.attack_type, AttackTypeEnum::FullyConnected);
        assert_eq!(data.fc_subtype, Some(FullyConnectedSubtype::Ranged));
        assert_eq!(data.target_domain, TargetDomainEnum::Ground);
        assert_eq!(data.target_type, TargetTypeEnum::SingleTarget);
        assert!(data.aoe_radius.is_none());
        assert_eq!(data.damage, 6);
        assert_eq!(data.range, 3);
        assert_eq!(data.min_range, 0);
        assert!(data.projectile_speed.is_none());
    }

    #[test]
    fn guard_attack_timing_aim_duration() {
        let data = guard_attack_data();
        assert_eq!(data.aim_duration, 2);
    }

    #[test]
    fn guard_attack_timing_firing_duration() {
        let data = guard_attack_data();
        assert_eq!(data.firing_duration, 1);
    }

    #[test]
    fn guard_attack_timing_cooldown_duration() {
        let data = guard_attack_data();
        assert_eq!(data.cooldown_duration, 1);
    }

    #[test]
    fn guard_attack_timing_reload_duration() {
        let data = guard_attack_data();
        assert_eq!(data.reload_duration, 4);
    }

    #[test]
    fn guard_attack_is_ranged_subtype() {
        let data = guard_attack_data();
        assert_eq!(data.fc_subtype, Some(FullyConnectedSubtype::Ranged));
    }

    #[test]
    fn guard_attack_no_projectile_speed() {
        let data = guard_attack_data();
        assert!(data.projectile_speed.is_none());
    }

    #[test]
    fn guard_attack_no_aoe() {
        let data = guard_attack_data();
        assert!(data.aoe_radius.is_none());
    }

    // --- Guard constants ---

    #[test]
    fn guard_control_cost_is_one() {
        assert_eq!(GUARD_CONTROL_COST, 1);
    }

    #[test]
    fn guard_tunnel_space_cost_is_two() {
        assert_eq!(GUARD_TUNNEL_SPACE_COST, 2);
    }

    #[test]
    fn guard_rugged_bonus_is_fifty_percent() {
        assert!((GUARD_RUGGED_BONUS - 0.5).abs() < f32::EPSILON);
    }

    // --- Guard ObjectType cross-check ---

    #[test]
    fn guard_object_type_matches_spec() {
        use crate::types::ObjectEnum;
        let obj = ObjectEnum::SyndicateGuard.object_type();
        assert_eq!(obj.name, "Guard");
        assert_eq!(obj.size, (36, 36));
        assert!(obj.destructible);
        assert_eq!(obj.sight_range, 5);
        assert!(obj.groupable); // Guard IS groupable (unlike Agent)
    }

    #[test]
    fn guard_is_unit_not_structure() {
        use crate::types::ObjectEnum;
        assert!(ObjectEnum::SyndicateGuard.is_unit());
        assert!(!ObjectEnum::SyndicateGuard.is_structure());
        assert!(!ObjectEnum::SyndicateGuard.is_resource());
    }

    // --- Guard movement speed derivation ---

    #[test]
    fn guard_movement_speed_derivation() {
        // 5 SU/frame * 16 FPS / 64 SU/GU = 1.25 GU/sec
        use crate::simulation::{FRAMES_PER_SECOND, SPACE_UNITS_PER_GRID_UNIT};
        let expected = 5.0 * (FRAMES_PER_SECOND as f32) / (SPACE_UNITS_PER_GRID_UNIT as f32);
        assert!((expected - 1.25).abs() < f32::EPSILON);
    }
}
