//! Syndicate object types — `factions/syndicate/`.

use super::units::{
    AttackAttrs, AttackType, FullyConnectedSubtype, MovementAttrs, TargetDomain, TargetType, UnitBase, UnitDef,
};
use super::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::math::PI;
use crate::player::Faction;

/// `factions/syndicate/units.md > Guard`.
pub(super) static GUARD: ObjectDef = ObjectDef {
    kind: ObjectKind::Guard,
    name: "Guard",
    faction: Some(Faction::Syndicate),                     // Faction - TheSyndicate
    category: Category::Unit,                              // Entity Type - Unit
    footprint: Footprint::Silhouette { w: 36.0, h: 36.0 }, // Silhouette - 36x36
    selectable: true,                                      // `core/entities.md > Object Type > Selectable - true`
    // Not listed for the Guard; a type with MaxHP is destructible (`core/entities.md > Object Instance > HP`:
    // destructible types only).
    destructible: true,
    groupable: true,    // Groupable - true
    sight_range: 5.0,   // SightRange - 5 (grid units, `core/scale.md > GridUnit`)
    max_hp: Some(80.0), // MaxHP - 80
    max_shields: None,
    armor: Armor {
        point: 1.0, // PointArmor - 1
        full: 1.0,  // FullArmor - 1
    },
    needs_flat_ground: false,
    unit: Some(UnitDef {
        base: UnitBase::HeavyInfantry, // UnitBase - HeavyInfantry
        // UnitBaseAttributes[HeavyInfantry]
        movement: MovementAttrs::TurnRate {
            turn_rate: PI,      // TurnRate: 180 degrees/frame
            acceleration: None, // Acceleration: infinite
            deceleration: None, // Deceleration: infinite
            max_speed: 5.0,     // MaxSpeed: 5 space units/frame
        },
        rugged_terrain_defense_bonus: None, // only LightInfantry has it
        turret: None,                       // TurretAttributes - None
        // The laser rifle (design change 0186): one shot every 24 frames, instant hit; the numbers are placeholders.
        attack: Some(AttackAttrs {
            // FullyConnected (laser; instant hit, cannot miss); its Range of 3 grid units makes it the Ranged subtype
            // (`systems/combat.md > FullyConnected > Melee` is adjacent contact).
            attack_type: AttackType::FullyConnected(FullyConnectedSubtype::Ranged),
            target_domain: TargetDomain::Universal, // TargetDomain: Universal (ground and air)
            target_type: TargetType::SingleTarget,  // TargetType: SingleTarget
            damage: 18.0,                           // Damage: 18 (placeholder)
            range: 3.0,                             // Range: 3 grid units
            min_range: 0.0,                         // MinRange: 0
            projectile_speed: None,                 // FullyConnected: no projectile
            aim_ticks: 4,                           // AimDuration: 4 frames (placeholder)
            firing_ticks: 1,                        // FiringDuration: 1 frame (placeholder)
            cooldown_ticks: 3,                      // CooldownDuration: 3 frames (placeholder)
            reload_ticks: 16,                       // ReloadDuration: 16 frames (placeholder)
        }),
        unit_control_cost: None,
        tunnel_space_cost: Some(1), // TunnelSpaceCost - 1
        beacon_capacity_cost: None,
        riot_shield: None,
        worker: None,
    }),
    structure: None,
};
