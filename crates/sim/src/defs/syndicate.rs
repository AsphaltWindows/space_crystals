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
        attack: Some(AttackAttrs {
            // FullyConnected; its Range of 3 grid units makes it the Ranged subtype (`systems/combat.md >
            // FullyConnected > Melee` is adjacent contact).
            attack_type: AttackType::FullyConnected(FullyConnectedSubtype::Ranged),
            target_domain: TargetDomain::Ground,   // TargetDomain: Ground
            target_type: TargetType::SingleTarget, // TargetType: SingleTarget
            damage: 6.0,                           // Damage: 6
            range: 3.0,                            // Range: 3 grid units
            min_range: 0.0,                        // MinRange: 0
            projectile_speed: None,                // FullyConnected: no projectile
            aim_ticks: 2,                          // AimDuration: 2 frames
            firing_ticks: 1,                       // FiringDuration: 1 frame
            cooldown_ticks: 1,                     // CooldownDuration: 1 frame
            reload_ticks: 4,                       // ReloadDuration: 4 frames
        }),
        unit_control_cost: None,
        tunnel_space_cost: Some(1), // TunnelSpaceCost - 1
        beacon_capacity_cost: None,
        riot_shield: None,
        worker: None,
    }),
    structure: None,
};
