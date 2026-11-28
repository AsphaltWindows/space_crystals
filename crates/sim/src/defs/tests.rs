use super::*;
use ::serde::Deserialize;
use ::serde::de::IntoDeserializer;
use ::serde::de::value::{Error as ValueError, U32Deserializer};

/// (M5-F3) The types that hold units: every exit opposite its entrance (design change 0347 point 1), and what
/// `defs_are_consistent` checks of their holds and their occupants' fire (M5-F1, M5-N2).
mod holds;
/// (M5-N3) The types that research: what `defs_are_consistent` checks of them.
mod research;
/// The defs as plain data: every def through JSON and back (plan decision T54).
mod serde;

/// Every variant, found through serde's derived `Deserialize`, which accepts a unit
/// variant by its declaration index and rejects the first index past the last variant.
fn declared_kinds() -> Vec<ObjectKind> {
    (0u32..)
        .map_while(|i| {
            let index: U32Deserializer<ValueError> = i.into_deserializer();
            ObjectKind::deserialize(index).ok()
        })
        .collect()
}

#[test]
fn all_lists_every_kind_in_declaration_order() {
    assert!(!ObjectKind::ALL.is_empty());
    assert_eq!(ObjectKind::ALL, declared_kinds().as_slice());
}

/// The registry's order and names are stored: the order in the state hash (`hash.rs`), the names in saves, replays and
/// maps. New kinds go last.
#[test]
fn every_kind_keeps_its_place_and_name() {
    let names: Vec<String> = ObjectKind::ALL
        .iter()
        .map(|kind| serde_json::to_string(kind).unwrap())
        .collect();
    let expected = [
        "CrystalPatch",
        "SupplyDeliveryStation",
        "Peacekeeper",
        "Guard",
        "DeploymentCenter",
        "Generator",
        "Barracks",
        "ExtractionFacility",
        "ExtractionPlate",
        "Drone",
        "Beacon",
        "EgressNode",
        "RoboPortal",
        "BioPortal",
        "SabreLegionnaire",
        "RifleLegionnaire",
        "PeacekeeperShielded",
        "Agent",
        "Tunnel",
        "Hideout",
        "OpticsLab",
        "ChemicalPlant",
        "Grenadier",
        "LaserTurret",
        "Recruit",
        "Militant",
        "Gunner",
        "RecruitmentHall",
        "Storage",
        "Armory",
        // Slice M5-G2 (plan decision T69): the Cults' other kinds.
        "Gunworks",
        "Garrison",
        "ChopShop",
        "MachineShop",
        "Airstrip",
        "ScrapHangar",
        "Altar",
        "Temple",
        "Cathedral",
        "Shrine",
        "Disciple",
        "Seeker",
        "Sparrow",
        "HandOfHeaven",
        "PrayerMobile",
        // Slice M5-N9 (design change 0328).
        "CareerCampus",
        "FieldEngineer",
        // Slice M5-K (design change 0356).
        "Arsenal",
        "Rocketman",
        // Slice M6-GK (design change 0366, plan decision T84): the GDO's other kinds.
        "Factory",
        "Airfield",
        "AdvancedMechanics",
        "PropulsionLabs",
        "IndustrialPowerplant",
        "RadarStation",
        "DefenseSystem",
        "MissileTurret",
        "Outpost",
        "Bunker",
        "FirstAidTent",
        "SupplyTower",
        "Raptor",
        "Twister",
        "Prospector",
        "ArtilleryRig",
        "ArtilleryRigDeployed",
        "Hornet",
        "Dragonfly",
        "DroneOperator",
        "OperatorDrone",
        "SupplyChopper",
        "Mine",
        // Slice M6-SK (plan decision T84): the Syndicate's kinds of design change 0367.
        "Vault",
        "VehicleAssembly",
        "Forge",
        "NullChamber",
        "AerospaceAssembly",
        "OrbitalCommand",
        "AssaultBike",
        "Shadebringer",
        "LaserBattery",
        "LaserBatteryDeployed",
        "SubterraneanTransport",
        "SubterraneanTransportBurrowed",
        "Phantom",
        "Specter",
        "Prism",
    ];
    assert_eq!(names[..expected.len()], expected.map(|name| format!("\"{name}\"")));
    for (index, &kind) in ObjectKind::ALL.iter().enumerate() {
        assert_eq!(kind as usize, index, "{kind:?}");
    }
}

#[test]
fn defs_are_consistent() {
    for &kind in ObjectKind::ALL {
        let def = kind.def();
        let name = def.name;
        assert_eq!(def.kind, kind, "{name}: def of the wrong kind");
        assert!(!name.is_empty(), "{kind:?}: empty name");
        match def.footprint {
            Footprint::Tiles { w, h } => {
                assert!((1..=16).contains(&w) && (1..=16).contains(&h), "{name}: {w}×{h} tiles");
                assert_ne!(def.category, Category::Unit, "{name}: units have silhouettes");
            }
            Footprint::Silhouette { w, h } => {
                assert!(
                    w.is_finite() && w > 0.0 && h.is_finite() && h > 0.0,
                    "{name}: {w}×{h} su"
                );
                // (M6-GK) And the mine, "closer to a permanently deployed unit" (design change 0426).
                assert!(
                    matches!(def.category, Category::Unit | Category::Mine),
                    "{name}: only units and mines have silhouettes"
                );
            }
        }
        assert!(
            def.sight_range.is_finite() && def.sight_range >= 0.0,
            "{name}: sight range"
        );
        match def.max_hp {
            Some(hp) => assert!(def.destructible && hp.is_finite() && hp > 0.0, "{name}: max HP {hp}"),
            None => assert!(!def.destructible, "{name}: destructible without max HP"),
        }
        // `systems/combat.md > Shields > MaxShields`: "Colonists units/buildings only; defined per-entity alongside
        // MaxHP".
        let colonists = def.faction == Some(Faction::Colonists);
        assert_eq!(def.max_shields.is_some(), colonists, "{name}: MaxShields");
        if let Some(shields) = def.max_shields {
            assert!(
                def.max_hp.is_some() && shields.is_finite() && shields > 0.0,
                "{name}: MaxShields {shields}"
            );
        }
        let Armor { point, full } = def.armor;
        assert!(
            point.is_finite() && point >= 0.0 && full.is_finite() && full >= 0.0,
            "{name}: armour {point} / {full}"
        );
        if def.category == Category::Resource {
            // `core/map.md > Resource`: indestructible, unowned (no faction), no vision.
            assert!(
                !def.destructible && def.max_hp.is_none(),
                "{name}: resources are indestructible"
            );
            assert_eq!(def.faction, None, "{name}: resources belong to no faction");
            assert_eq!(def.sight_range, 0.0, "{name}: resources provide no vision");
            assert_eq!(def.armor, Armor::NONE, "{name}: resources have no armour");
        }
        if def.category == Category::Mine {
            // (M6-GK) `factions/gdo/units/twister.md > Mines`: a Twister's, which "enemies can attack and kill", and
            // its owner "can select", "like a unit" (design change 0426).
            assert_eq!(def.faction, Some(Faction::Gdo), "{name}: the GDO's mine");
            assert!(def.destructible && def.selectable, "{name}: a mine");
        }
        // `core/map_editor.md > Per-object Resource Values`: the map resources have them, each its own kind's, and
        // valid; no other type has any.
        assert_eq!(
            def.default_settings.is_some(),
            def.category == Category::Resource,
            "{name}: per-object values"
        );
        if let Some(settings) = def.default_settings {
            assert_eq!(
                (settings.kind(), settings.check()),
                (kind, Ok(())),
                "{name}: {settings:?}"
            );
        }
        // Exactly the structures have structure attributes. Every structure so far is a destructible building
        // of a faction, on whole tiles of flat ground but for the underground ones (`core/map.md > Elevation >
        // Structures only on flat terrain`), whose footprint its symmetry type allows (`core/entities.md >
        // Structure Type`: only ABAB, ABAC and ABCD may be oblong).
        assert_eq!(
            def.structure.is_some(),
            def.category == Category::Structure,
            "{name}: structure attributes"
        );
        if let Some(structure) = &def.structure {
            structure_is_consistent(def, structure);
        }
        // Exactly the units have unit attributes, and their movement attributes are their base's model.
        assert_eq!(
            def.unit.is_some(),
            def.category == Category::Unit,
            "{name}: unit attributes"
        );
        if let Some(unit) = &def.unit {
            unit_is_consistent(def, unit);
        }
    }
}

/// `defs_are_consistent` for a structure type. Every structure so far is a destructible building of a faction, on
/// whole tiles of flat ground (`core/map.md > Elevation > Structures only on flat terrain`) but for the underground
/// ones (design change 0005), whose footprint its symmetry type allows (`core/entities.md > Structure Type`: only
/// ABAB, ABAC and ABCD may be oblong; AAAA "requires for Height and Width to be equal").
fn structure_is_consistent(def: &ObjectDef, structure: &StructureDef) {
    let name = def.name;
    let (w, h) = def.tile_size().expect("structures are tile-based");
    assert!(
        w == h || structure.symmetry.allows_oblong(),
        "{name}: {w}×{h} with {:?}",
        structure.symmetry
    );
    if structure.symmetry == SymmetryType::AAAA {
        assert_eq!(w, h, "{name}: AAAA is square");
    }
    assert!(def.destructible && def.max_hp.is_some(), "{name}: destructible");
    assert!(def.faction.is_some(), "{name}: structures belong to a faction");
    // The underground structures (plan decision T32) are the Syndicate's expansions, which "may be placed on sloped
    // tiles" (design change 0005); every other structure stands on flat ground.
    assert_eq!(def.needs_flat_ground, !structure.underground, "{name}: flat ground");
    assert!(
        !structure.underground || def.faction == Some(Faction::Syndicate),
        "{name}: underground"
    );
    tunnel_is_consistent(def, structure);
    requires_are_consistent(def, def.kind, structure.requires);
    if let Some(attack) = &structure.attack {
        attack_is_consistent(name, attack);
    }
    assert!(def.selectable, "{name}: selectable");
    // What a structure goes onto is a map resource (the plate's patch), and seeding the build area is the GDO's
    // (`factions/gdo/structures/README.md > GDOBuildArea`).
    if let Some(onto) = structure.onto {
        assert_eq!(onto.def().category, Category::Resource, "{name} goes onto {onto:?}");
    }
    assert!(
        !structure.seeds_build_area || def.faction == Some(Faction::Gdo),
        "{name}: seeds a build area"
    );
    // The Colonists have neither Power nor a build area (plan decision T21, `defs/colonists.rs`).
    if def.faction == Some(Faction::Colonists) {
        assert_eq!(
            (structure.power, structure.build_radius_extension),
            (0, 0),
            "{name}: Power, BuildRadiusExtension"
        );
    }
    // `Constructs`, and the Beacon's Warp In list (`factions/colonists/economy.md > Warp In > Constructs List`):
    // buildings of the type's own faction, each listed once, each taking time, and short enough for their progress in
    // fixed point (`crate::construction`). A type has one list or the other (plan decision T14): the Beacon warps
    // in, it is no GDO-style constructor. Nothing warps in a Beacon ("every structure other than the Beacon itself").
    assert!(
        structure.constructs.is_empty() || structure.warp_in.is_empty(),
        "{name}: constructs and warps in"
    );
    let longest = u32::MAX / crate::economy::RATIO_ONE;
    for recipes in [structure.constructs, structure.warp_in] {
        for (i, recipe) in recipes.iter().enumerate() {
            let built = recipe.kind.def();
            assert!(
                built.structure.is_some() && built.faction == def.faction && recipe.kind != ObjectKind::Beacon,
                "{name} builds {:?}",
                recipe.kind
            );
            assert!((1..=longest).contains(&recipe.frames), "{name}: {recipe:?}");
            assert!(
                recipes[..i].iter().all(|r| r.kind != recipe.kind),
                "{name} lists {:?} twice",
                recipe.kind
            );
            requires_are_consistent(def, recipe.kind, recipe.requires);
        }
    }
    // `Produces`: units of the producer's own faction, each listed once, each taking time, out of sides its
    // SymmetryType has: a `Side` rule's letter names exactly one side (the Barracks' B), a `Sides` rule's several (the
    // Recruitment Hall's four A sides, slice M5-C), each open letter of a `Centre` rule at least one (plan decision
    // T19; the portals' energy doorway, design change 0145), a `Runway`'s two ends one each (slice M5-N7). A producer that recruits by itself has one recipe, what
    // it recruits, and an area at least as large as its footprint (plan decision T57).
    if let Some(production) = &structure.production {
        let letters = structure.symmetry.letters();
        let count = |letter| letters.iter().filter(|&&l| l == letter).count();
        match production.exits {
            ExitRule::Side(letter) => assert_eq!(count(letter), 1, "{name}: exit side {letter:?}"),
            ExitRule::Sides(letter) => assert!(count(letter) >= 2, "{name}: exit sides {letter:?}"),
            ExitRule::Centre(exits) => {
                assert!(!exits.is_empty(), "{name}: no open side");
                for &letter in exits {
                    assert!(count(letter) >= 1, "{name}: exit side {letter:?}");
                }
            }
            // Into the Tunnel Network: through no side, from an underground expansion.
            ExitRule::Network => assert!(structure.underground, "{name}: produces into the network"),
            // (M5-N2) Everyone out at once: the Garrison's door, which produces nothing.
            ExitRule::AtOnce(_) => panic!("{name}: a producer releases its units one at a time"),
            // (M5-N7) Off a runway, from one end to the other: two sides its SymmetryType letters once each, opposite
            // each other (`production::Runway`), at a building that trains its aircraft of the units it holds
            // (`holds_are_consistent`).
            ExitRule::Runway { from, toward } => {
                assert_eq!(
                    (count(from), count(toward)),
                    (1, 1),
                    "{name}: a runway {from:?} to {toward:?}"
                );
                let side = |letter| letters.iter().position(|&l| l == letter);
                assert_eq!(
                    side(from).map(|i| (i + 2) % 4),
                    side(toward),
                    "{name}: a runway from end to end"
                );
                assert!(
                    structure.holds.is_some(),
                    "{name}: a runway's aircraft of no one inside"
                );
            }
            // (M6-GH1) Over its pads: an Airfield's (`StructureDef::airfield`), whose every unit is an aircraft that
            // rearms at it (`UnitDef::ammo`); or over its centre, a Supply Tower's
            // (`StructureDef::supply_tower`), whose every unit is an air unit that lands on it
            // (`UnitDef::supply_chopper`).
            ExitRule::Pad => {
                let tower = structure.supply_tower.is_some();
                assert!(
                    structure.airfield.is_some() != tower,
                    "{name}: pads, and no Airfield or Supply Tower"
                );
                for recipe in production.recipes {
                    let unit = recipe.kind.def().unit.as_ref();
                    let lands = |unit: &UnitDef| match tower {
                        true => unit.supply_chopper.is_some(),
                        false => unit.ammo.is_some(),
                    };
                    assert!(
                        unit.is_some_and(|unit| lands(unit) && unit.base_def().domain == Domain::Air),
                        "{name}: {:?} is no aircraft that lands on it",
                        recipe.kind
                    );
                }
            }
        }
        // (M5-S8) A default rally point lies out of the middle of the exit side (`production::default_rally`,
        // `core/entities.md > Entrance and exit > Exit Doorway` point 7): a type with one has one exit side, and makes
        // the units it is for, "wheeled or tracked vehicles".
        if let Some(out) = production.default_rally {
            assert!(
                matches!(production.exits, ExitRule::Side(_)) && out > 0,
                "{name}: a default rally point {out} GU out of {:?}",
                production.exits
            );
            let vehicle = |kind: ObjectKind| {
                let base = kind.def().unit.as_ref().map(|unit| unit.base);
                matches!(base, Some(UnitBase::WheeledVehicle | UnitBase::TrackedVehicle))
            };
            assert!(
                production.recipes.iter().any(|recipe| vehicle(recipe.kind)),
                "{name}: a default rally point for vehicles"
            );
        }
        if let Some(recruitment) = production.recruitment {
            let (w, h) = def.tile_size().expect("a structure has tiles");
            assert_eq!(production.recipes.len(), 1, "{name} recruits one kind");
            assert!(
                recruitment.area >= w.max(h) && recruitment.capacity > 0,
                "{name}: {recruitment:?}"
            );
        }
        for (i, recipe) in production.recipes.iter().enumerate() {
            let unit = recipe.kind.def();
            assert!(
                unit.unit.is_some() && unit.faction == def.faction,
                "{name} produces {:?}",
                recipe.kind
            );
            assert!((1..=longest).contains(&recipe.frames), "{name}: {recipe:?}");
            assert!(
                production.recipes[..i].iter().all(|r| r.kind != recipe.kind),
                "{name} lists {:?} twice",
                recipe.kind
            );
            requires_are_consistent(def, recipe.kind, recipe.requires);
        }
    }
    holds::holds_are_consistent(def, structure);
    research::research_is_consistent(def, structure);
}

/// `defs_are_consistent` for the attack of a unit type or of a structure type named `name` (`StructureDef::attack`,
/// slice M4-S1: the Laser Turret's). `systems/combat.md > AttackType`: "ProjectileSpeed - number (HeadDisjointed /
/// DoublyDisjointed only)", which both require (`systems/combat.md > HeadDisjointed`, `systems/combat.md >
/// DoublyDisjointed`: "RequiresProjectileSpeed - true"); the Grenadier's grenade is the first (slice M4-U). An area
/// attack's circle has a radius.
fn attack_is_consistent(name: &str, attack: &AttackAttrs) {
    let flies = matches!(
        attack.attack_type,
        AttackType::HeadDisjointed | AttackType::DoublyDisjointed
    );
    let speed = attack.projectile_speed;
    assert_eq!(
        speed.is_some_and(|speed| speed.is_finite() && speed > 0.0),
        flies,
        "{name}: ProjectileSpeed {speed:?}"
    );
    if let TargetType::AoE { radius, .. } = attack.target_type {
        assert!(radius.is_finite() && radius > 0.0, "{name}: AoE radius {radius}");
    }
}

/// `defs_are_consistent` for a TechPrerequisite, a structure type's, or the "Requires" of a recipe of `def`'s, gating
/// `gated` (`core/tech_and_research.md > Tech Prerequisites`, slice M4-K): structure types of `def`'s own faction (one
/// rule for every faction, plan decision T37), each listed once, none the type it gates.
fn requires_are_consistent(def: &ObjectDef, gated: ObjectKind, requires: &[ObjectKind]) {
    let name = def.name;
    for (i, &required) in requires.iter().enumerate() {
        let its = required.def();
        assert!(
            its.structure.is_some() && its.faction == def.faction && required != gated,
            "{name}: {gated:?} requires {required:?}"
        );
        assert!(
            !requires[..i].contains(&required),
            "{name}: {gated:?} requires {required:?} twice"
        );
    }
}

/// `defs_are_consistent` for a structure's tier table (plan decision T39): only the Tunnel has one, a surface structure
/// (`factions/syndicate/tunnels.md > Tunnel > Tiers`). Its tiers are 1 to 3, each with more MaxHP, a wider Tunnel Area
/// and at least the transit of the tier below, infantry from Tier 1; its type's MaxHP is that of Tier 1, the tier a
/// Tunnel starts at.
fn tunnel_is_consistent(def: &ObjectDef, structure: &StructureDef) {
    let name = def.name;
    assert_eq!(
        structure.tunnel.is_some(),
        def.kind == ObjectKind::Tunnel,
        "{name}: tier table"
    );
    let Some(tunnel) = structure.tunnel else {
        return;
    };
    assert!(!structure.underground, "{name}: on the surface");
    assert_eq!(tunnel.tiers.len(), 3, "{name}: Tiers 1 to 3");
    assert_eq!((tunnel.tier(0), tunnel.tier(4)), (None, None), "{name}: Tiers 1 to 3");
    let base = tunnel.tier(crate::tunnel::BASE_TIER).map(|tier| tier.max_hp);
    assert_eq!(def.max_hp, base, "{name}: the MaxHP of Tier 1");
    let regen = tunnel.regen_per_second;
    assert!(regen.is_finite() && regen >= 0.0, "{name}: regeneration {regen}");
    for tier in tunnel.tiers {
        assert!(
            tier.max_hp.is_finite() && tier.max_hp > 0.0 && tier.transit.contains(&Transit::Infantry),
            "{name}: {tier:?}"
        );
    }
    for pair in tunnel.tiers.windows(2) {
        let (lower, higher) = (pair[0], pair[1]);
        assert!(
            lower.max_hp < higher.max_hp
                && lower.area_radius < higher.area_radius
                && lower.transit.iter().all(|t| higher.transit.contains(t)),
            "{name}: {pair:?}"
        );
    }
}

/// The unit types that can't move, immobile by their defs (plan decision T87: "The deployed kind is immobile by its
/// def, so "can't move" and "never yields" follow from the existing rules"): a MaxSpeed of 0, and a ReverseMaxSpeed of
/// 0 for a model that has one, so that `Mover::can_move` answers false. Slice M6-GK's deployed Artillery Rig
/// (`factions/gdo/units/artillery_rig.md > Modes`: "**Deployed:** cannot move") and slice M6-SK's deployed Laser
/// Battery (`factions/syndicate/units/laser_battery.md > Deploy (ability)`: "**Deployed**: **cannot move**"), in the
/// registry's order. A named list until the pairs' deploying exists: slices M6-GA and M6-SL may replace it by a rule
/// read from `UnitDef::vehicle_deploy` (the kind a deploy leads into).
const IMMOBILE: &[ObjectKind] = &[ObjectKind::ArtilleryRigDeployed, ObjectKind::LaserBatteryDeployed];

/// `defs_are_consistent` for a unit type: its movement attributes are its base's model, and it has a collision box. It
/// moves, at a top speed above 0, unless it is one of the [`IMMOBILE`] types, whose top speeds are 0.
fn unit_is_consistent(def: &ObjectDef, unit: &UnitDef) {
    let name = def.name;
    let base = unit.base_def();
    assert_eq!(unit.movement.model(), base.movement_model, "{name}: movement model");
    let speed = unit.movement.max_speed();
    if IMMOBILE.contains(&def.kind) {
        let reverse = match unit.movement {
            MovementAttrs::FixedTurnRadius { reverse_max_speed, .. }
            | MovementAttrs::SpeedTurnRadius { reverse_max_speed, .. } => reverse_max_speed,
            _ => 0.0,
        };
        assert_eq!((speed, reverse), (0.0, 0.0), "{name}: immobile, its max speeds 0");
    } else {
        assert!(speed.is_finite() && speed > 0.0, "{name}: max speed {speed}");
    }
    // `systems/units.md > TurretAttributes`: "Only present on units whose UnitBase has CanHaveTurret = true; a unit
    // of such a base may still have TurretAttributes = None (no turret)" (design change 0154): the HoverVehicle
    // can carry one, and the Drone has none.
    assert!(
        unit.turret.is_none() || base.can_have_turret,
        "{name}: a turret on a base that can't carry one"
    );
    assert_eq!(unit.has_turret(), unit.turret.is_some(), "{name}");
    assert_eq!(
        unit.rugged_terrain_defense_bonus.is_some(),
        unit.base == UnitBase::LightInfantry,
        "{name}: RuggedTerrainDefenseBonus belongs to LightInfantry"
    );
    // Each faction's army cost belongs to its own units: the GDO's UnitControlCost (`factions/gdo/overview.md > Unit
    // Control`), the Syndicate's TunnelSpaceCost (`factions/syndicate/tunnels.md`), the Colonists' BeaconCapacityCost
    // (`factions/colonists/overview.md > Beacon Capacity`). The Cults' Unit Control cost, the Recruits a unit takes,
    // is kept in the GDO's field (plan decision T55; `factions/cults/overview.md > CultsResources > Unit Control`).
    let of = |faction| def.faction == Some(faction);
    assert_eq!(
        unit.unit_control_cost.is_some(),
        of(Faction::Gdo) || of(Faction::Cults),
        "{name}: UnitControlCost"
    );
    assert_eq!(
        unit.tunnel_space_cost.is_some(),
        of(Faction::Syndicate),
        "{name}: TunnelSpaceCost"
    );
    assert_eq!(
        unit.beacon_capacity_cost.is_some(),
        of(Faction::Colonists),
        "{name}: BeaconCapacityCost"
    );
    // `systems/combat.md > FullyConnected > Melee`: "Short reach: 8 space units for all melee attacks", the one
    // placeholder Range of every melee attack (design change 0149), and nothing nearer to keep out.
    if let Some(attack) = &unit.attack
        && attack.attack_type == AttackType::FullyConnected(FullyConnectedSubtype::Melee)
    {
        assert_eq!(
            (attack.range, attack.min_range),
            (MELEE_CONTACT_GU, 0.0),
            "{name}: melee range"
        );
    }
    if let Some(attack) = &unit.attack {
        attack_is_consistent(name, attack);
    }
    assert!(def.faction.is_some(), "{name}: units belong to a faction");
    assert!(def.half_extents().is_some(), "{name}: units have a collision box");
    // A unit with two forms is one of them, and its other form's def has the same conversion
    // (`factions/colonists/units/weapon_conversion.md`); a conversion takes time.
    if let Some(conversion) = unit.conversion {
        let other =
            crate::toggle::other(conversion.forms, def.kind).unwrap_or_else(|| panic!("{name}: not one of its forms"));
        let back = other.def().unit.as_ref().and_then(|u| u.conversion);
        assert_eq!(back, Some(conversion), "{name}: its other form's conversion");
        assert!(conversion.ticks >= 1, "{name}: {conversion:?}");
    }
    // So does a unit with a Riot Shield, whose two forms are its lowered and its raised type (design change 0173,
    // `crate::toggle`); raising or lowering it takes time.
    if let Some(shield) = unit.riot_shield {
        let other = crate::toggle::other([shield.lowered, shield.raised], def.kind)
            .unwrap_or_else(|| panic!("{name}: not one of its shield's types"));
        let back = other.def().unit.as_ref().and_then(|u| u.riot_shield);
        assert_eq!(back, Some(shield), "{name}: its other type's shield");
        assert!(shield.toggle_ticks >= 1, "{name}: {shield:?}");
    }
    // A unit deploys into a building of its own faction, which takes time (`factions/colonists/economy.md > Beacon
    // Deployment`).
    if let Some(deploy) = unit.deploy {
        let into = deploy.kind.def();
        assert!(
            into.structure.is_some() && into.faction == def.faction && deploy.frames >= 1,
            "{name} deploys into {deploy:?}"
        );
    }
    // (M6-GD) A stock holds units of its own faction, a tethered one (`crate::stock`, `crate::tether`: the Drone
    // Operator's drones), starting with at most its max; a purchase takes time, short enough for its progress in fixed
    // point (`crate::progress`), and costs crystals. A tethered unit has a leash and a battery, and an attack, its end.
    if let Some(stock) = unit.stock {
        let item = stock.item.def();
        let tethered = item.unit.as_ref().is_some_and(|unit| unit.tether.is_some());
        assert!(
            tethered && item.faction == def.faction && stock.start <= stock.max && stock.max >= 1,
            "{name}: its stock {stock:?}"
        );
        assert!(
            (1..=u32::MAX / crate::economy::RATIO_ONE).contains(&stock.frames) && stock.price >= 1,
            "{name}: its stock's purchase {stock:?}"
        );
    }
    if let Some(tether) = unit.tether {
        assert!(
            tether.leash.is_finite() && tether.leash > 0.0 && tether.battery >= 1 && unit.attack.is_some(),
            "{name}: its tether {tether:?}"
        );
    }
    // (M6-GA) A vehicle deploy (`crate::vehicle_deploy`) leads into a unit type of its faction whose own leads back, a
    // pair of types (slice M6-GP's site arm will lead a unit into a structure); it takes time, short enough for its
    // progress in fixed point (`crate::progress`).
    if let Some(deploy) = unit.vehicle_deploy {
        let into = deploy.into.def();
        let back = into.unit.as_ref().and_then(|unit| unit.vehicle_deploy);
        assert!(
            back.is_some_and(|back| back.into == def.kind) && into.faction == def.faction,
            "{name} deploys into {deploy:?}"
        );
        assert!(
            (1..=u32::MAX / crate::economy::RATIO_ONE).contains(&deploy.frames),
            "{name}: its deploy's frames {deploy:?}"
        );
    }
}

/// Only a plate and a patch share tiles, in either order.
#[test]
fn only_a_plate_and_a_patch_may_overlap() {
    for &a in ObjectKind::ALL {
        for &b in ObjectKind::ALL {
            let pair = [a, b];
            let plate_on_patch =
                pair.contains(&ObjectKind::ExtractionPlate) && pair.contains(&ObjectKind::CrystalPatch);
            assert_eq!(a.may_overlap(b), plate_on_patch, "{a:?} and {b:?}");
        }
    }
}

/// (M5-H1) `systems/units.md > Air Collision > SeparationRadius`: "per unit type, circular, the distance at which
/// air-to-air repulsion activates. Must be larger than the unit's Silhouette": every air unit has one, and no other
/// unit has one. (M5-H2) It is larger than the Silhouette's longer side, as design change 0405 reads the rule ("it must
/// exceed the 40 su Silhouette", the Sparrow's raised from 32 to 48 su); slice M5-H1 had read it as a circle that holds
/// the Silhouette, longer than its half-diagonal, which 32 su was.
#[test]
fn air_units_keep_apart_by_a_circle_round_their_silhouette() {
    for &kind in ObjectKind::ALL {
        let def = kind.def();
        let Some(unit) = &def.unit else {
            continue;
        };
        let air = unit.base_def().domain == Domain::Air;
        let silhouette = def.silhouette().expect("a unit has a Silhouette");
        match unit.separation_radius {
            Some(radius) => assert!(
                air && radius.is_finite() && radius > silhouette.max_element(),
                "{kind:?}: a SeparationRadius of {radius} su"
            ),
            None => assert!(!air, "{kind:?}: an air unit without a SeparationRadius"),
        }
    }
}
