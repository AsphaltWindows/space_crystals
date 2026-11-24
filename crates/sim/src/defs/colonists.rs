//! Colonists object types — `factions/colonists/`: its units (`units.md`) and its structures (`economy.md`,
//! `structures.md`), with their full stat blocks (slice M3-A).
//!
//! What the stat blocks name and nothing reads yet: MaxShields (no Shields until slice M3-B), BeaconCapacityCost
//! (M3-C), the Beacon's Warp In list (M3-E). The HoverVehicle's DragMovement attributes are the Drone's movement
//! (`movement::drag`, slice M3-G). "BeaconCapacity provided - 5" is no field: every Colonists
//! building provides it (`economy.md > Warp In > Beacon Capacity Retention`), a faction rule M3-C derives (plan
//! decision T14).
//!
//! The Colonists have neither Power nor a build area: their stat blocks list no Power nor BuildRadiusExtension, so
//! both are 0 (plan decision T21), and a Beacon may be deployed with "no range limit from other Colonists buildings"
//! (`economy.md > Beacon Deployment > Placement`).

use super::structures::{ExitRule, ProductionDef, Recipe, StructureDef, SymmetryType};
use super::units::{
    AttackAttrs, AttackType, FullyConnectedSubtype, MELEE_CONTACT_GU, MovementAttrs, TargetDomain, TargetType,
    UnitBase, UnitDef, WorkerDef,
};
use super::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::math::PI;
use crate::player::Faction;
use crate::sides::SideLetter;

/// `factions/colonists/units.md > Drone`: the worker, unarmed. It moves by DragMovement (`movement::drag`, slice
/// M3-G, plan decision T25): it thrusts up to the equilibrium of 8 space units/frame (`MovementAttrs::max_speed`),
/// 4 on Rugged terrain, and brakes with its omni-directional thrust.
pub(super) static DRONE: ObjectDef = ObjectDef {
    kind: ObjectKind::Drone,
    name: "Drone",
    faction: Some(Faction::Colonists),                     // Faction - Colonists
    category: Category::Unit,                              // Entity Type - Unit
    footprint: Footprint::Silhouette { w: 36.0, h: 36.0 }, // Silhouette - 36x36 space units (square)
    selectable: true,                                      // `core/entities.md > Object Type > Selectable - true`
    destructible: true,                                    // Destructible - true
    groupable: false,                                      // Groupable - false
    sight_range: 5.0,                                      // SightRange - 5 grid units
    max_hp: Some(20.0),                                    // MaxHP - 20
    max_shields: Some(10.0),                               // MaxShields - 10
    armor: Armor {
        point: 0.0, // PointArmor - 0
        full: 0.0,  // FullArmor - 0
    },
    needs_flat_ground: false,
    unit: Some(UnitDef {
        base: UnitBase::HoverVehicle, // UnitBase - HoverVehicle
        // UnitBaseAttributes[HoverVehicle]
        movement: MovementAttrs::Drag {
            forward_acceleration: 0.6,          // ForwardAcceleration: 0.6 space units/frame squared
            omni_directional_acceleration: 0.2, // OmniDirectionalAcceleration: 0.2 space units/frame squared
            drag_ratio: 0.1,                    // DragRatio: 0.1 per frame
            turn_rate: PI / 4.0,                // TurnRate: 45 degrees/frame
        },
        rugged_terrain_defense_bonus: None, // only LightInfantry has it
        // TurretAttributes - None: the HoverVehicle base can carry a turret, and the Drone has none (question Q0042
        // point 10).
        turret: None,
        attack: None, // AttackAttributes - None (unarmed)
        unit_control_cost: None,
        tunnel_space_cost: None,
        beacon_capacity_cost: Some(1), // BeaconCapacityCost - 1
        riot_shield: None,
        // `> Gathering`
        worker: Some(WorkerDef {
            carry_capacity: 8,   // CarryCapacity: 8 per load
            mining_ticks: 32,    // MiningDuration: 32 frames
            drop_off_ticks: 2,   // DropOffDuration: 2 frames
            mining_reach: 8.0,   // MiningReach: 8 space units (the design's placeholder)
            drop_off_reach: 8.0, // DropOffReach: 8 space units (the design's placeholder)
            pass_through: true,  // > Gathering Drones pass through each other (design change 0106)
        }),
    }),
    structure: None,
};

/// `factions/colonists/units.md > Sabre Legionnaire`: the melee form of the Legionnaire. `> Weapon Conversion`: it
/// shares the Rifle Legionnaire's physical profile and differs only in its AttackAttributes (a test keeps the two
/// stat blocks so).
pub(super) static SABRE_LEGIONNAIRE: ObjectDef = ObjectDef {
    kind: ObjectKind::SabreLegionnaire,
    name: "Sabre Legionnaire",
    faction: Some(Faction::Colonists),                     // Faction - Colonists
    category: Category::Unit,                              // Entity Type - Unit
    footprint: Footprint::Silhouette { w: 36.0, h: 36.0 }, // Silhouette - 36x36 space units (square)
    selectable: true,                                      // `core/entities.md > Object Type > Selectable - true`
    destructible: true,                                    // Destructible - true
    groupable: true,                                       // Groupable - true
    sight_range: 5.0,                                      // SightRange - 5 grid units
    max_hp: Some(70.0),                                    // MaxHP - 70
    max_shields: Some(10.0),                               // MaxShields - 10
    armor: Armor {
        point: 0.0, // PointArmor - 0
        full: 0.0,  // FullArmor - 0
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
            // AttackType: FullyConnected (Melee)
            attack_type: AttackType::FullyConnected(FullyConnectedSubtype::Melee),
            target_domain: TargetDomain::Ground,   // TargetDomain: Ground
            target_type: TargetType::SingleTarget, // TargetType: SingleTarget
            damage: 15.0,                          // Damage: 15
            // Range: melee (adjacent contact). A placeholder (question Q0042 point 5).
            range: MELEE_CONTACT_GU,
            min_range: 0.0,         // none listed: contact is as near as a target gets
            projectile_speed: None, // FullyConnected: no projectile
            aim_ticks: 3,           // AimDuration: 3 frames
            firing_ticks: 3,        // FiringDuration: 3 frames
            cooldown_ticks: 1,      // CooldownDuration: 1 frame
            reload_ticks: 5,        // ReloadDuration: 5 frames
        }),
        unit_control_cost: None,
        tunnel_space_cost: None,
        beacon_capacity_cost: Some(2), // BeaconCapacityCost - 2
        riot_shield: None,
        worker: None,
    }),
    structure: None,
};

/// `factions/colonists/units.md > Rifle Legionnaire`: the ranged form of the Legionnaire, "Not directly produced —
/// reached only by converting a Sabre Legionnaire" (slice M3-B).
pub(super) static RIFLE_LEGIONNAIRE: ObjectDef = ObjectDef {
    kind: ObjectKind::RifleLegionnaire,
    name: "Rifle Legionnaire",
    faction: Some(Faction::Colonists),                     // Faction - Colonists
    category: Category::Unit,                              // Entity Type - Unit
    footprint: Footprint::Silhouette { w: 36.0, h: 36.0 }, // Silhouette - 36x36 space units (square)
    selectable: true,                                      // `core/entities.md > Object Type > Selectable - true`
    destructible: true,                                    // Destructible - true
    groupable: true,                                       // Groupable - true
    sight_range: 5.0,                                      // SightRange - 5 grid units
    max_hp: Some(70.0),                                    // MaxHP - 70
    max_shields: Some(10.0),                               // MaxShields - 10
    armor: Armor {
        point: 0.0, // PointArmor - 0
        full: 0.0,  // FullArmor - 0
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
            // AttackType: FullyConnected (Ranged)
            attack_type: AttackType::FullyConnected(FullyConnectedSubtype::Ranged),
            target_domain: TargetDomain::Ground,   // TargetDomain: Ground
            target_type: TargetType::SingleTarget, // TargetType: SingleTarget
            damage: 10.0,                          // Damage: 10
            range: 4.0,                            // Range: 4 grid units
            min_range: 0.0,                        // MinRange: 0
            projectile_speed: None,                // FullyConnected: no projectile
            aim_ticks: 3,                          // AimDuration: 3 frames
            firing_ticks: 2,                       // FiringDuration: 2 frames
            cooldown_ticks: 7,                     // CooldownDuration: 7 frames
            reload_ticks: 4,                       // ReloadDuration: 4 frames
        }),
        unit_control_cost: None,
        tunnel_space_cost: None,
        beacon_capacity_cost: Some(2), // BeaconCapacityCost - 2
        riot_shield: None,
        worker: None,
    }),
    structure: None,
};

/// `factions/colonists/economy.md > Beacon`: a deployed Drone (`> Beacon Deployment`, slice M3-F), onto which every
/// other Colonists building is warped in (`> Warp In`, slice M3-E). "Full ObjectInterfaceState still TBD".
pub(super) static BEACON: ObjectDef = ObjectDef {
    kind: ObjectKind::Beacon,
    name: "Beacon",
    faction: Some(Faction::Colonists),          // Faction - Colonists
    category: Category::Structure,              // Entity Type - Structure Type
    footprint: Footprint::Tiles { w: 2, h: 2 }, // Size - 2x2 grid units
    selectable: true,                           // `core/entities.md > Object Type > Selectable - true`
    destructible: true,                         // Destructible - true
    groupable: true,                            // Groupable - true
    sight_range: 4.0,                           // SightRange - 4 grid units
    max_hp: Some(120.0),                        // MaxHP - 120
    max_shields: Some(10.0),                    // MaxShields - 10
    armor: Armor {
        point: 0.0, // PointArmor - 0
        full: 0.0,  // FullArmor - 0
    },
    needs_flat_ground: true, // `core/map.md > Elevation > Structures only on flat terrain`
    unit: None,
    structure: Some(StructureDef {
        symmetry: SymmetryType::AAAA, // SymmetryType - AAAA
        build_radius_extension: 0,    // none (see the module docs)
        power: 0,                     // none (plan decision T21)
        production: None,
        mining: None,
        constructs: &[],
        // `> Warp In > Constructs List`: the three buildings with a cost and a WarpInTime. The others (Alloy
        // Furnace, …, Structural Analyzer) are TBD, and deferred past M3 (question Q0043 point 1).
        warp_in: &[
            Recipe {
                kind: ObjectKind::EgressNode,
                cost: 75,    // Egress Node | 75 Space Crystals |
                frames: 320, // 320 frames (20 seconds)
            },
            Recipe {
                kind: ObjectKind::RoboPortal,
                cost: 400,   // Robo Portal | 400 Space Crystals |
                frames: 640, // 640 frames (40 seconds)
            },
            Recipe {
                kind: ObjectKind::BioPortal,
                cost: 150,   // Bio Portal | 150 Space Crystals |
                frames: 480, // 480 frames (30 seconds)
            },
        ],
        drop_off: false,
    }),
};

/// `factions/colonists/economy.md > Egress Node`: where Drones drop off what they gather (`> Drop Off`, slice M3-H).
pub(super) static EGRESS_NODE: ObjectDef = ObjectDef {
    kind: ObjectKind::EgressNode,
    name: "Egress Node",
    faction: Some(Faction::Colonists),          // Faction - Colonists
    category: Category::Structure,              // Entity Type - Structure Type
    footprint: Footprint::Tiles { w: 2, h: 2 }, // Size - 2x2 grid units
    selectable: true,                           // `core/entities.md > Object Type > Selectable - true`
    destructible: true,                         // Destructible - true
    groupable: false,                           // Groupable - false
    sight_range: 3.0,                           // SightRange - 3 grid units
    max_hp: Some(200.0),                        // MaxHP - 200
    max_shields: Some(10.0),                    // MaxShields - 10
    armor: Armor {
        point: 0.0, // PointArmor - 0
        full: 0.0,  // FullArmor - 0
    },
    needs_flat_ground: true, // `core/map.md > Elevation > Structures only on flat terrain`
    unit: None,
    structure: Some(StructureDef {
        symmetry: SymmetryType::AAAA, // SymmetryType - AAAA
        build_radius_extension: 0,    // none (see the module docs)
        power: 0,                     // none (plan decision T21)
        production: None,
        mining: None,
        constructs: &[],
        warp_in: &[],
        drop_off: true, // `economy.md > Egress Node > Drop Off`
    }),
};

/// `factions/colonists/structures.md > Robo Portal`: the Drones' producer.
pub(super) static ROBO_PORTAL: ObjectDef = ObjectDef {
    kind: ObjectKind::RoboPortal,
    name: "Robo Portal",
    faction: Some(Faction::Colonists),          // Faction - Colonists
    category: Category::Structure,              // Entity Type - Structure Type
    footprint: Footprint::Tiles { w: 4, h: 4 }, // Size - 4x4 grid units
    selectable: true,                           // `core/entities.md > Object Type > Selectable - true`
    destructible: true,                         // Destructible - true
    groupable: true,                            // Groupable - true
    sight_range: 4.0,                           // SightRange - 4 grid units
    max_hp: Some(600.0),                        // MaxHP - 600
    max_shields: Some(20.0),                    // MaxShields - 20
    armor: Armor {
        point: 1.0, // PointArmor - 1
        full: 16.0, // FullArmor - 16
    },
    needs_flat_ground: true, // `core/map.md > Elevation > Structures only on flat terrain`
    unit: None,
    structure: Some(StructureDef {
        symmetry: SymmetryType::AAAA, // SymmetryType - AAAA
        build_radius_extension: 0,    // none (see the module docs)
        power: 0,                     // none (plan decision T21)
        production: Some(ProductionDef {
            // "Since the Robo Portal is fully symmetric (AAAA), it has no fixed exit side — a warped-in Drone may
            // emerge from any of the 4 sides": all four are exits. **Placeholder** (question Q0041 point 3): which one
            // follows the Bio Portal's Exit Behavior over all four sides (plan decision T19).
            exits: ExitRule::Nearest(&[SideLetter::A]),
            recipes: &[Recipe {
                kind: ObjectKind::Drone, // Produces: "Drone: 50 Space Crystals, 160 frames (10 seconds)"
                cost: 50,
                frames: 160,
            }],
        }),
        mining: None,
        constructs: &[],
        warp_in: &[],
        drop_off: false,
    }),
};

/// `factions/colonists/structures.md > Bio Portal`: the infantry's producer.
pub(super) static BIO_PORTAL: ObjectDef = ObjectDef {
    kind: ObjectKind::BioPortal,
    name: "Bio Portal",
    faction: Some(Faction::Colonists),          // Faction - Colonists
    category: Category::Structure,              // Entity Type - Structure Type
    footprint: Footprint::Tiles { w: 3, h: 3 }, // Size - 3x3 grid units
    selectable: true,                           // `core/entities.md > Object Type > Selectable - true`
    destructible: true,                         // Destructible - true
    groupable: true,                            // Groupable - true
    sight_range: 3.0,                           // SightRange - 3 grid units
    max_hp: Some(250.0),                        // MaxHP - 250
    max_shields: Some(15.0),                    // MaxShields - 15
    armor: Armor {
        point: 1.0, // PointArmor - 1
        full: 9.0,  // FullArmor - 9
    },
    needs_flat_ground: true, // `core/map.md > Elevation > Structures only on flat terrain`
    unit: None,
    structure: Some(StructureDef {
        // SymmetryType - AABB: A on top and on the right, B at the bottom and on the left, unturned (design change
        // 0079), so the two A sides are adjacent; built as written (question Q0041 point 5).
        symmetry: SymmetryType::AABB,
        build_radius_extension: 0, // none (see the module docs)
        power: 0,                  // none (plan decision T21)
        production: Some(ProductionDef {
            // `> Exit Behavior`: "two exit-eligible sides (the "A" sides)": "A produced unit exits from whichever
            // eligible A side is closest to the RallyPoint" (plan decision T19).
            exits: ExitRule::Nearest(&[SideLetter::A]),
            recipes: &[Recipe {
                kind: ObjectKind::SabreLegionnaire, // Produces: "Sabre Legionnaire: 100 Space Crystals, 160 frames"
                cost: 100,
                frames: 160,
            }],
        }),
        mining: None,
        constructs: &[],
        warp_in: &[],
        drop_off: false,
    }),
};

#[cfg(test)]
mod tests;
