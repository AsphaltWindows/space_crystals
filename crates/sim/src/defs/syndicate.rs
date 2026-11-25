//! Syndicate object types — `factions/syndicate/`.

use super::structures::{StructureDef, SymmetryType, TierDef, Transit, TunnelDef};
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

/// `factions/syndicate/units.md > Agent`: the Syndicate's worker, with its pistol (slice M4-A). Its gathering
/// (`> Gathering`: its `worker`) and its construction come with slices M4-D, M4-E and M4-I.
pub(super) static AGENT: ObjectDef = ObjectDef {
    kind: ObjectKind::Agent,
    name: "Agent",
    faction: Some(Faction::Syndicate),                     // Faction - TheSyndicate
    category: Category::Unit,                              // Entity Type - Unit
    footprint: Footprint::Silhouette { w: 36.0, h: 36.0 }, // Silhouette - 36x36
    selectable: true,                                      // `core/entities.md > Object Type > Selectable - true`
    // Not listed for the Agent; a type with MaxHP is destructible (`core/entities.md > Object Instance > HP`).
    destructible: true,
    groupable: false,   // Groupable - false
    sight_range: 5.0,   // SightRange - 5
    max_hp: Some(75.0), // MaxHP - 75
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
            max_speed: 6.0,     // MaxSpeed: 6 space units/frame
        },
        rugged_terrain_defense_bonus: None, // only LightInfantry has it
        turret: None,                       // TurretAttributes - None
        // The pistol (user, 2026-11-24, replacing the melee attack; "Numbers are placeholders"): "One shot every 20
        // frames (1.25 s) = 4 damage per second, a third of a Guard's".
        attack: Some(AttackAttrs {
            attack_type: AttackType::FullyConnected(FullyConnectedSubtype::Ranged), // FullyConnected (Ranged; instant hit)
            target_domain: TargetDomain::Ground,                                    // TargetDomain: Ground
            target_type: TargetType::SingleTarget,                                  // TargetType: SingleTarget
            damage: 5.0,                                                            // Damage: 5
            range: 2.0,                                                             // Range: 2 grid units
            min_range: 0.0,                                                         // MinRange: 0
            projectile_speed: None,                                                 // FullyConnected: no projectile
            aim_ticks: 3,                                                           // AimDuration: 3 frames
            firing_ticks: 1,                                                        // FiringDuration: 1 frame
            cooldown_ticks: 2,                                                      // CooldownDuration: 2 frames
            reload_ticks: 14,                                                       // ReloadDuration: 14 frames
        }),
        unit_control_cost: None,
        tunnel_space_cost: Some(1), // TunnelSpaceCost - 1
        beacon_capacity_cost: None,
        riot_shield: None,
        worker: None,
    }),
    structure: None,
};

/// `factions/syndicate/tunnels.md > Tunnel > Tiers`, its table column by column.
static TUNNEL_TIERS: TunnelDef = TunnelDef {
    tiers: &[
        // Tier 1 (base): HP 600, Tunnel Area Radius 2 (8x8), Transit Infantry.
        TierDef {
            max_hp: 600.0,
            area_radius: 2,
            transit: &[Transit::Infantry],
        },
        // Tier 2: HP 800, Tunnel Area Radius 3 (10x10), Transit Infantry + Vehicles.
        TierDef {
            max_hp: 800.0,
            area_radius: 3,
            transit: &[Transit::Infantry, Transit::Vehicles],
        },
        // Tier 3: HP 1000, Tunnel Area Radius 4 (12x12), Transit Infantry + Vehicles + Air.
        TierDef {
            max_hp: 1000.0,
            area_radius: 4,
            transit: &[Transit::Infantry, Transit::Vehicles, Transit::Air],
        },
    ],
};

/// `factions/syndicate/tunnels.md > Tunnel`: one type with a tier (plan decision T39; `crate::tunnel`). Inert so far
/// (slice M4-A): it blocks, sees and can be destroyed. The network, the drop-offs at Sides B and D, its construction
/// and its upgrades come with slices M4-C to M4-I.
pub(super) static TUNNEL: ObjectDef = ObjectDef {
    kind: ObjectKind::Tunnel,
    name: "Tunnel",
    faction: Some(Faction::Syndicate),          // `factions/syndicate/`
    category: Category::Structure,              // Entity Type - Structure Type
    footprint: Footprint::Tiles { w: 4, h: 4 }, // Size - 4x4
    selectable: true,                           // `core/entities.md > Object Type > Selectable - true`
    destructible: true,                         // Destructible - true
    groupable: false,                           // Groupable - false
    sight_range: 5.0,                           // SightRange - 5
    // Tiers: HP 600 at Tier 1, the tier a Tunnel starts at; `Object::max_hp` reads its tier's (`crate::tunnel`).
    max_hp: Some(600.0),
    max_shields: None,
    armor: Armor {
        point: 1.0, // Tiers: PointArmor 1 (every tier)
        full: 16.0, // Tiers: FullArmor 16 (every tier)
    },
    // A surface structure (`core/map.md > Elevation > Structures only on flat terrain`): only the expansions may stand
    // on slopes (design change 0005).
    needs_flat_ground: true,
    unit: None,
    structure: Some(StructureDef {
        // SymmetryType - ABCD, clockwise from the top: A the entrance and exit, B the crystal hatch, C the back wall
        // (opposite A, no function), D the Supplies hatch (design change 0248, correcting 0224). Of Side A's two
        // paths, the exit is the half nearer B and the entry the half nearer D (0248, for slice M4-C).
        symmetry: SymmetryType::ABCD,
        build_radius_extension: 0,
        power: 0,
        production: None,
        mining: None,
        constructs: &[],
        warp_in: &[],
        // Agents deliver to its hatches, crystals at Side B and Supplies at Side D (design change 0248), from slices
        // M4-D and M4-I on (plan decision T34, with the side rule in its def).
        drop_off: false,
        underground: false,
        tunnel: Some(&TUNNEL_TIERS),
    }),
};

/// `factions/syndicate/structures.md > Hideout`: an underground expansion (plan decision T32). Inert so far (slice
/// M4-A): hidden from the other players while owned, it is walked over, and can be destroyed. Its BuildQueue
/// (`> Produces`) comes with slice M4-C, its construction with M4-F.
pub(super) static HIDEOUT: ObjectDef = ObjectDef {
    kind: ObjectKind::Hideout,
    name: "Hideout",
    faction: Some(Faction::Syndicate),          // `factions/syndicate/`
    category: Category::Structure,              // Entity Type - Structure Type (Underground)
    footprint: Footprint::Tiles { w: 2, h: 2 }, // Size - 2x2
    selectable: true,                           // `core/entities.md > Object Type > Selectable - true`
    // Not listed for the Hideout; a type with MaxHP is destructible (`core/entities.md > Object Instance > HP`).
    destructible: true,
    groupable: true,     // Placeholder: Q0065 point 6, not given ("true, like the Barracks")
    sight_range: 0.0,    // Placeholder: Q0065 point 5, not given ("none, since they are underground")
    max_hp: Some(400.0), // HP - 400
    max_shields: None,
    armor: Armor {
        point: 1.0, // PointArmor - 1
        full: 4.0,  // FullArmor - 4
    },
    // `core/map.md > Elevation` (design change 0005): "Syndicate underground expansions (inside a Tunnel Area) may be
    // placed on sloped tiles; the flatness rule doesn't apply to them".
    needs_flat_ground: false,
    unit: None,
    structure: Some(StructureDef {
        // Not given (plan decision T29): AAAA, since none of its sides has a function.
        symmetry: SymmetryType::AAAA,
        build_radius_extension: 0,
        power: 0,
        production: None,
        mining: None,
        constructs: &[],
        warp_in: &[],
        drop_off: false,
        underground: true, // Entity Type - Structure Type (Underground)
        tunnel: None,
    }),
};

#[cfg(test)]
mod tests;
