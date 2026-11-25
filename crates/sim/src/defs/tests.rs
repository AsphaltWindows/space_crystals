use super::*;
use serde::Deserialize;
use serde::de::IntoDeserializer;
use serde::de::value::{Error as ValueError, U32Deserializer};

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
                assert_eq!(def.category, Category::Unit, "{name}: only units have silhouettes");
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
    assert!(def.selectable, "{name}: selectable");
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
        }
    }
    // `Produces`: units of the producer's own faction, each listed once, each taking time, out of sides its
    // SymmetryType has: a `Side` rule's letter names exactly one side (the Barracks' B), each open letter of a
    // `Centre` rule at least one (plan decision T19; the portals' energy doorway, design change 0145).
    if let Some(production) = &structure.production {
        let letters = structure.symmetry.letters();
        let count = |letter| letters.iter().filter(|&&l| l == letter).count();
        match production.exits {
            ExitRule::Side(letter) => assert_eq!(count(letter), 1, "{name}: exit side {letter:?}"),
            ExitRule::Centre(exits) => {
                assert!(!exits.is_empty(), "{name}: no open side");
                for &letter in exits {
                    assert!(count(letter) >= 1, "{name}: exit side {letter:?}");
                }
            }
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
        }
    }
}

/// `defs_are_consistent` for a structure's tier table (plan decision T39): only the Tunnel has one, a surface
/// structure (`factions/syndicate/tunnels.md > Tunnel > Tiers`). Its tiers are 1 to 3, each with more MaxHP, a wider
/// Tunnel Area and at least the transit of the tier below, infantry from Tier 1; its type's MaxHP is that of Tier 1, the
/// tier a Tunnel starts at.
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

/// `defs_are_consistent` for a unit type: its movement attributes are its base's model, and it has a collision box.
fn unit_is_consistent(def: &ObjectDef, unit: &UnitDef) {
    let name = def.name;
    let base = unit.base_def();
    assert_eq!(unit.movement.model(), base.movement_model, "{name}: movement model");
    let speed = unit.movement.max_speed();
    assert!(speed.is_finite() && speed > 0.0, "{name}: max speed {speed}");
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
    // Each faction's army cost belongs to its own units: `factions/gdo/units.md > UnitControlCost`, the Syndicate's
    // TunnelSpaceCost (`factions/syndicate/tunnels.md`), the Colonists' BeaconCapacityCost
    // (`factions/colonists/overview.md > Beacon Capacity`).
    let of = |faction| def.faction == Some(faction);
    assert_eq!(
        unit.unit_control_cost.is_some(),
        of(Faction::Gdo),
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
    assert!(def.faction.is_some(), "{name}: units belong to a faction");
    assert!(def.half_extents().is_some(), "{name}: units have a collision box");
}

/// The stat blocks of the GDO structures: `factions/gdo/structures.md > DeploymentCenter`, `> Barracks`, and
/// `factions/gdo/economy.md > Generator`, `> ExtractionFacility`, `> ExtractionPlate`.
#[test]
fn the_gdo_structures_follow_the_design() {
    use ObjectKind as K;
    use SymmetryType as S;
    // Kind, size (w × h in the default orientation), symmetry, MaxHP, PointArmor, FullArmor, SightRange,
    // Groupable, BuildRadiusExtension, Power.
    let expected = [
        (
            K::DeploymentCenter,
            (4, 4),
            S::AAAA,
            1000.0,
            1.0,
            16.0,
            6.0,
            false,
            12,
            30,
        ),
        (K::Generator, (2, 2), S::AAAA, 350.0, 1.0, 4.0, 3.0, true, 1, 20),
        // `> Barracks > Size - 2x3 grid units (2 tall, 3 wide in the default orientation)` (design change 0079).
        (K::Barracks, (3, 2), S::ABAC, 300.0, 1.0, 6.0, 4.0, true, 2, -30),
        (
            K::ExtractionFacility,
            (3, 3),
            S::AAAA,
            500.0,
            1.0,
            9.0,
            3.0,
            false,
            2,
            -15,
        ),
        (K::ExtractionPlate, (1, 1), S::AAAA, 85.0, 2.0, 2.0, 0.0, true, 0, -3),
    ];
    for (kind, size, symmetry, hp, point, full, sight, groupable, extension, power) in expected {
        let def = kind.def();
        let structure = def.structure.as_ref().unwrap();
        assert_eq!(
            (def.category, def.faction, def.tile_size(), structure.symmetry),
            (Category::Structure, Some(Faction::Gdo), Some(size), symmetry),
            "{kind:?}"
        );
        assert_eq!(
            (def.max_hp, def.armor, def.sight_range),
            (Some(hp), Armor { point, full }, sight),
            "{kind:?}"
        );
        assert_eq!(
            (def.destructible, def.groupable, def.selectable),
            (true, groupable, true),
            "{kind:?}"
        );
        assert_eq!(
            (structure.build_radius_extension, structure.power),
            (extension, power),
            "{kind:?}"
        );
        assert!(def.unit.is_none() && def.needs_flat_ground, "{kind:?}");
    }
    // `> ExtractionPlate > MiningRate - 10 Space Crystals per 48 frames`, `> ResidualMiningRate - 1 Space
    // Crystal per 48 frames`: the plate is the one structure that mines.
    for &kind in ObjectKind::ALL {
        let mining = kind.def().structure.as_ref().and_then(|s| s.mining);
        let expected = (kind == K::ExtractionPlate).then_some(MiningDef {
            load: 10,
            residual: 1,
            cycle_ticks: 48,
        });
        assert_eq!(mining, expected, "{kind:?}");
    }
    // `factions/gdo/structures.md > DeploymentCenter > Constructs` and `factions/gdo/economy.md >
    // ExtractionFacility > Constructs`: the two constructors, with each building's cost and time.
    let recipe = |kind, cost, frames| Recipe { kind, cost, frames };
    for &kind in ObjectKind::ALL {
        let constructs = kind.def().structure.as_ref().map_or(&[][..], |s| s.constructs);
        let expected = match kind {
            K::DeploymentCenter => vec![
                recipe(K::Generator, 100, 240),
                recipe(K::Barracks, 200, 160),
                recipe(K::ExtractionFacility, 200, 320),
            ],
            K::ExtractionFacility => vec![recipe(K::ExtractionPlate, 75, 400)],
            _ => vec![],
        };
        assert_eq!(constructs, expected.as_slice(), "{kind:?}");
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

/// `factions/gdo/units.md > Peacekeeper`, `factions/syndicate/units.md > Guard`: the M1 units.
#[test]
fn the_m1_units_follow_the_design() {
    let peacekeeper = ObjectKind::Peacekeeper.def();
    let guard = ObjectKind::Guard.def();
    assert_eq!(peacekeeper.silhouette(), Some(Vec2::new(24.0, 24.0)));
    assert_eq!(guard.half_extents(), Some(Vec2::new(18.0, 18.0)));
    assert_eq!((peacekeeper.max_hp, guard.max_hp), (Some(50.0), Some(80.0)));
    let one = Armor { point: 1.0, full: 1.0 };
    assert_eq!(
        (peacekeeper.armor, guard.armor),
        (one, one),
        "PointArmor - 1, FullArmor - 1"
    );
    assert_eq!(
        (peacekeeper.faction, guard.faction),
        (Some(Faction::Gdo), Some(Faction::Syndicate))
    );
    let (pk, gd) = (peacekeeper.unit.as_ref().unwrap(), guard.unit.as_ref().unwrap());
    assert_eq!((pk.base, gd.base), (UnitBase::LightInfantry, UnitBase::HeavyInfantry));
    // Only TurnRateMovement exists in M1: 180°/tick, so every turn completes within a tick.
    for unit in [pk, gd] {
        let MovementAttrs::TurnRate {
            turn_rate,
            acceleration,
            deceleration,
            ..
        } = unit.movement
        else {
            panic!("{:?} is not TurnRateMovement", unit.base);
        };
        assert_eq!(turn_rate, crate::math::PI);
        assert_eq!((acceleration, deceleration), (None, None), "infinite");
    }
    assert_eq!((pk.movement.max_speed(), gd.movement.max_speed()), (4.0, 5.0));
    let (pk_attack, gd_attack) = (pk.attack.unwrap(), gd.attack.unwrap());
    assert_eq!(
        (pk_attack.damage, pk_attack.range, pk_attack.reload_ticks),
        (10.0, 4.0, 12)
    );
    assert_eq!(
        (gd_attack.damage, gd_attack.range, gd_attack.reload_ticks),
        (18.0, 3.0, 16)
    );
    let shield = pk.riot_shield.unwrap();
    assert_eq!(shield.damage_factor, 0.5);
    assert!(shield.toggle_ticks < 8, "under half a second (8 ticks)");
    assert_eq!((pk.unit_control_cost, gd.tunnel_space_cost), (Some(1), Some(1)));
}

/// `factions/gdo/units.md > Peacekeeper > Riot Shield > Two unit types` (design change 0173): Peacekeeper (Shielded)
/// is the Peacekeeper but for its name and "what the shield changes": "MaxSpeed is halved (4 → 2 space
/// units/frame)". Its weapon is the same, which its shield keeps it from using (`riot_shield::allows_attack`); its
/// Unit Control is the same ("Cost and Unit Control are the same for both types"), and so is its shield, which names
/// both types. The Barracks produces the Peacekeeper only.
#[test]
fn the_shielded_peacekeeper_is_the_peacekeeper_at_half_speed() {
    use ObjectKind as K;
    let (peacekeeper, shielded) = (K::Peacekeeper.def(), K::PeacekeeperShielded.def());
    assert_eq!(
        (shielded.kind, shielded.name),
        (K::PeacekeeperShielded, "Peacekeeper (Shielded)")
    );
    let (pk, sh) = (peacekeeper.unit.as_ref().unwrap(), shielded.unit.as_ref().unwrap());
    assert_eq!((pk.movement.max_speed(), sh.movement.max_speed()), (4.0, 2.0));
    let unit = UnitDef {
        movement: pk.movement,
        ..sh.clone()
    };
    assert_eq!(&unit, pk, "the same unit but for its MaxSpeed");
    let object = ObjectDef {
        kind: K::Peacekeeper,
        name: peacekeeper.name,
        unit: Some(unit),
        ..shielded.clone()
    };
    // `ObjectDef` has no `PartialEq`: its fields, all of them, as `Debug` shows them.
    assert_eq!(
        format!("{object:?}"),
        format!("{peacekeeper:?}"),
        "the same object but for its kind and name"
    );
    let shield = sh.riot_shield.unwrap();
    assert_eq!(
        (shield.lowered, shield.raised),
        (K::Peacekeeper, K::PeacekeeperShielded)
    );
    let barracks = K::Barracks.def().structure.as_ref().unwrap().production.unwrap();
    let trained: Vec<ObjectKind> = barracks.recipes.iter().map(|recipe| recipe.kind).collect();
    assert_eq!(trained, [K::Peacekeeper]);
}
