//! GDO object types — `factions/gdo/`: its units (`units.md`) and its structures (`structures.md`, `economy.md`).

use super::structures::{MiningDef, Recipe, StructureDef, SymmetryType};
use super::units::{
    AttackAttrs, AttackType, FullyConnectedSubtype, MovementAttrs, RiotShieldDef, TargetDomain, TargetType, UnitBase,
    UnitDef,
};
use super::{Armor, Category, Footprint, ObjectDef, ObjectKind};
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
    armor: Armor {
        point: 1.0, // PointArmor - 1
        full: 1.0,  // FullArmor - 1
    },
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
    structure: None,
};

/// `factions/gdo/structures.md > DeploymentCenter`.
pub(super) static DEPLOYMENT_CENTER: ObjectDef = ObjectDef {
    kind: ObjectKind::DeploymentCenter,
    name: "Deployment Center",
    faction: Some(Faction::Gdo), // Faction - GlobalDefenseOrganization
    category: Category::Structure,
    footprint: Footprint::Tiles { w: 4, h: 4 }, // Size - 4x4 grid units
    selectable: true,                           // `core/entities.md > Object Type > Selectable - true`
    destructible: true,                         // Destructible - true
    groupable: false,                           // Groupable - false
    sight_range: 6.0,                           // SightRange - 6 grid units
    max_hp: Some(1000.0),                       // MaxHP - 1000
    armor: Armor {
        point: 1.0, // PointArmor - 1
        full: 16.0, // FullArmor - 16
    },
    needs_flat_ground: true, // `core/map.md > Elevation > Structures only on flat terrain`
    unit: None,
    structure: Some(StructureDef {
        symmetry: SymmetryType::AAAA, // SymmetryType - AAAA
        build_radius_extension: 12,   // BuildRadiusExtension - 12 grid units
        power: 30,                    // Power - +30
        mining: None,
        // Constructs: the three buildings with a cost and a time. The design's others (Supply Tower, Arsenal, …) have
        // none yet (TBD); their Requires clauses (`core/tech_and_research.md > Tech Prerequisites`) come with them.
        constructs: &[
            Recipe {
                kind: ObjectKind::Generator,
                crystals: 100, // Generator: 100 Space Crystals,
                ticks: 240,    // 240 frames (15 seconds)
            },
            Recipe {
                kind: ObjectKind::Barracks,
                crystals: 200, // Barracks: 200 Space Crystals,
                ticks: 160,    // 160 frames (10 seconds)
            },
            Recipe {
                kind: ObjectKind::ExtractionFacility,
                crystals: 200, // Extraction Facility: 200 Space Crystals,
                ticks: 320,    // 320 frames (20 seconds)
            },
        ],
    }),
};

/// `factions/gdo/economy.md > Generator`.
pub(super) static GENERATOR: ObjectDef = ObjectDef {
    kind: ObjectKind::Generator,
    name: "Generator",
    faction: Some(Faction::Gdo), // Faction - GlobalDefenseOrganization
    category: Category::Structure,
    footprint: Footprint::Tiles { w: 2, h: 2 }, // Size - 2x2 grid units
    selectable: true,                           // `core/entities.md > Object Type > Selectable - true`
    destructible: true,                         // Destructible - true
    groupable: true,                            // Groupable - true
    sight_range: 3.0,                           // SightRange - 3 grid units
    max_hp: Some(350.0),                        // MaxHP - 350
    armor: Armor {
        point: 1.0, // PointArmor - 1
        full: 4.0,  // FullArmor - 4
    },
    needs_flat_ground: true, // `core/map.md > Elevation > Structures only on flat terrain`
    unit: None,
    structure: Some(StructureDef {
        symmetry: SymmetryType::AAAA, // SymmetryType - AAAA
        build_radius_extension: 1,    // BuildRadiusExtension - 1 grid unit
        power: 20,                    // Power - +20
        mining: None,
        constructs: &[],
    }),
};

/// `factions/gdo/structures.md > Barracks`.
pub(super) static BARRACKS: ObjectDef = ObjectDef {
    kind: ObjectKind::Barracks,
    name: "Barracks",
    faction: Some(Faction::Gdo), // Faction - GlobalDefenseOrganization
    category: Category::Structure,
    // Size - 2x3 grid units (2 tall, 3 wide in the default orientation; design change 0079).
    footprint: Footprint::Tiles { w: 3, h: 2 },
    selectable: true,    // `core/entities.md > Object Type > Selectable - true`
    destructible: true,  // Destructible - true
    groupable: true,     // Groupable - true
    sight_range: 4.0,    // SightRange - 4 grid units
    max_hp: Some(300.0), // MaxHP - 300
    armor: Armor {
        point: 1.0, // PointArmor - 1
        full: 6.0,  // FullArmor - 6
    },
    needs_flat_ground: true, // `core/map.md > Elevation > Structures only on flat terrain`
    unit: None,
    structure: Some(StructureDef {
        symmetry: SymmetryType::ABAC, // SymmetryType - ABAC (top A, right B, bottom A, left C)
        build_radius_extension: 2,    // BuildRadiusExtension - 2 grid units
        power: -30,                   // Power - -30
        mining: None,
        constructs: &[],
    }),
};

/// `factions/gdo/economy.md > ExtractionFacility`.
pub(super) static EXTRACTION_FACILITY: ObjectDef = ObjectDef {
    kind: ObjectKind::ExtractionFacility,
    name: "Extraction Facility",
    faction: Some(Faction::Gdo), // Faction - GlobalDefenseOrganization
    category: Category::Structure,
    footprint: Footprint::Tiles { w: 3, h: 3 }, // Size - 3x3 grid units
    selectable: true,                           // `core/entities.md > Object Type > Selectable - true`
    destructible: true,                         // Destructible - true
    groupable: false,                           // Groupable - false
    sight_range: 3.0,                           // SightRange - 3 grid units
    max_hp: Some(500.0),                        // MaxHP - 500
    armor: Armor {
        point: 1.0, // PointArmor - 1
        full: 9.0,  // FullArmor - 9
    },
    needs_flat_ground: true, // `core/map.md > Elevation > Structures only on flat terrain`
    unit: None,
    structure: Some(StructureDef {
        symmetry: SymmetryType::AAAA, // SymmetryType - AAAA
        build_radius_extension: 2,    // BuildRadiusExtension - 2 grid units
        power: -15,                   // Power - -15
        mining: None,
        // Constructs
        constructs: &[Recipe {
            kind: ObjectKind::ExtractionPlate,
            crystals: 75, // Extraction Plate: 75 Space Crystals,
            ticks: 400,   // 400 frames (25 seconds)
        }],
    }),
};

/// `factions/gdo/economy.md > ExtractionPlate`: placed onto a Space Crystals Patch, which it shares its tile with
/// (`ObjectKind::may_overlap`).
pub(super) static EXTRACTION_PLATE: ObjectDef = ObjectDef {
    kind: ObjectKind::ExtractionPlate,
    name: "Extraction Plate",
    faction: Some(Faction::Gdo), // Faction - GlobalDefenseOrganization
    category: Category::Structure,
    footprint: Footprint::Tiles { w: 1, h: 1 }, // Size - 1x1 grid unit
    selectable: true,                           // `core/entities.md > Object Type > Selectable - true`
    destructible: true,                         // Destructible - true
    groupable: true,                            // Groupable - true
    sight_range: 0.0,                           // SightRange - 0
    max_hp: Some(85.0),                         // MaxHP - 85
    armor: Armor {
        point: 2.0, // PointArmor - 2
        full: 2.0,  // FullArmor - 2
    },
    needs_flat_ground: true, // `core/map.md > Elevation > Structures only on flat terrain`
    unit: None,
    structure: Some(StructureDef {
        symmetry: SymmetryType::AAAA, // SymmetryType - AAAA
        build_radius_extension: 0,    // BuildRadiusExtension - 0
        power: -3,                    // Power - -3
        mining: Some(MiningDef {
            load: 10,        // MiningRate - 10 Space Crystals per 48 frames (3 seconds)
            residual: 1,     // ResidualMiningRate - 1 Space Crystal per 48 frames (3 seconds)
            cycle_ticks: 48, // "per 48 frames"
        }),
        constructs: &[],
    }),
};
