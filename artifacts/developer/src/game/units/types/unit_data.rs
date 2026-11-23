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

/// Convert frame-based duration to seconds using simulation FPS
pub fn frames_to_seconds(frames: u32) -> f32 {
    use crate::simulation::FRAMES_PER_SECOND;
    frames as f32 / FRAMES_PER_SECOND as f32
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
