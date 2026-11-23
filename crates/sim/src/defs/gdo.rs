//! GDO object types — `factions/gdo/`.

use super::units::{
    AttackAttrs, AttackType, FullyConnectedSubtype, MovementAttrs, RiotShieldDef, TargetDomain, TargetType, UnitBase,
    UnitDef,
};
use super::{Category, Footprint, ObjectDef, ObjectKind};
use crate::math::PI;
use crate::player::Faction;

/// `factions/gdo/units.md > Peacekeeper > Riot Shield`: "raising and lowering the shield takes a short time …
/// Exact value TBD (should be under half a second)". Placeholder, unspecified: 6 ticks (0.375 s).
pub const RIOT_SHIELD_TOGGLE_TICKS: u32 = 6;

/// `factions/gdo/units.md > Peacekeeper`.
pub(super) static PEACEKEEPER: ObjectDef = ObjectDef {
    kind: ObjectKind::Peacekeeper,
    name: "Peacekeeper",
    faction: Some(Faction::Gdo), // Faction - GlobalDefenseOrganization
    category: Category::Unit,
    footprint: Footprint::Silhouette { w: 24.0, h: 24.0 }, // Silhouette - 24x24 space units (square)
    selectable: true,                                      // `core/entities.md > Object Type > Selectable - true`
    destructible: true,                                    // Destructible - true
    groupable: true,                                       // Groupable - true
    sight_range: 5.0,                                      // SightRange - 5 grid units
    max_hp: Some(50.0),                                    // MaxHP - 50
    needs_flat_ground: false,
    unit: Some(UnitDef {
        base: UnitBase::LightInfantry, // UnitBase - LightInfantry
        // UnitBaseAttributes[LightInfantry]
        movement: MovementAttrs::TurnRate {
            turn_rate: PI,      // TurnRate: 180 degrees/frame
            acceleration: None, // Acceleration: infinite
            deceleration: None, // Deceleration: infinite
            max_speed: 4.0,     // MaxSpeed: 4 space units/frame
        },
        rugged_terrain_defense_bonus: Some(0.5), // RuggedTerrainDefenseBonus: 50%
        point_armor: 1.0,                        // PointArmor - 1
        full_armor: 1.0,                         // FullArmor - 1
        turret: None,                            // TurretAttributes - None
        attack: Some(AttackAttrs {
            // FullyConnected; its Range of 4 grid units makes it the Ranged subtype (`systems/combat.md >
            // FullyConnected > Melee` is adjacent contact).
            attack_type: AttackType::FullyConnected(FullyConnectedSubtype::Ranged),
            target_domain: TargetDomain::Ground,   // TargetDomain: Ground
            target_type: TargetType::SingleTarget, // TargetType: SingleTarget
            damage: 10.0,                          // Damage: 10
            range: 4.0,                            // Range: 4 grid units
            min_range: 0.0,                        // MinRange: 0
            projectile_speed: None,                // FullyConnected: no projectile
            aim_ticks: 2,                          // AimDuration: 2 frames
            firing_ticks: 1,                       // FiringDuration: 1 frame
            cooldown_ticks: 2,                     // CooldownDuration: 2 frames
            reload_ticks: 12,                      // ReloadDuration: 12 frames
        }),
        unit_control_cost: Some(1), // UnitControlCost - 1
        tunnel_space_cost: None,
        riot_shield: Some(RiotShieldDef {
            speed_factor: 0.5,  // MaxSpeed is halved (4 → 2 space units/frame)
            damage_factor: 0.5, // Incoming damage is halved, a 50% pre-armor reduction
            toggle_ticks: RIOT_SHIELD_TOGGLE_TICKS,
        }),
    }),
};
