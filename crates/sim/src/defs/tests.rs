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
        // of a faction, on whole tiles of flat ground (`core/map.md > Elevation > Structures only on flat
        // terrain`), whose footprint its symmetry type allows (`core/entities.md > Structure Type`: only ABAB,
        // ABAC and ABCD may be oblong).
        assert_eq!(
            def.structure.is_some(),
            def.category == Category::Structure,
            "{name}: structure attributes"
        );
        if let Some(structure) = &def.structure {
            let (w, h) = def.tile_size().expect("structures are tile-based");
            assert!(
                w == h || structure.symmetry.allows_oblong(),
                "{name}: {w}×{h} with {:?}",
                structure.symmetry
            );
            assert!(def.destructible && def.max_hp.is_some(), "{name}: destructible");
            assert!(def.faction.is_some(), "{name}: structures belong to a faction");
            assert!(def.needs_flat_ground, "{name}: structures stand on flat ground");
            assert!(def.selectable, "{name}: selectable");
            // `Constructs`: buildings of the constructor's own faction, each listed once, each taking time, and
            // short enough for their progress in fixed point (`crate::construction`).
            for (i, recipe) in structure.constructs.iter().enumerate() {
                let built = recipe.kind.def();
                assert!(
                    built.structure.is_some() && built.faction == def.faction,
                    "{name} constructs {:?}",
                    recipe.kind
                );
                let longest = u32::MAX / crate::economy::RATIO_ONE;
                assert!((1..=longest).contains(&recipe.frames), "{name}: {recipe:?}");
                assert!(
                    structure.constructs[..i].iter().all(|r| r.kind != recipe.kind),
                    "{name} lists {:?} twice",
                    recipe.kind
                );
            }
        }
        // Exactly the units have unit attributes, and their movement attributes are their base's model.
        assert_eq!(
            def.unit.is_some(),
            def.category == Category::Unit,
            "{name}: unit attributes"
        );
        if let Some(unit) = &def.unit {
            let base = unit.base_def();
            assert_eq!(unit.movement.model(), base.movement_model, "{name}: movement model");
            let speed = unit.movement.max_speed();
            assert!(speed.is_finite() && speed > 0.0, "{name}: max speed {speed}");
            assert_eq!(unit.turret.is_some(), base.has_turret, "{name}: turret");
            assert_eq!(
                unit.rugged_terrain_defense_bonus.is_some(),
                unit.base == UnitBase::LightInfantry,
                "{name}: RuggedTerrainDefenseBonus belongs to LightInfantry"
            );
            assert!(def.faction.is_some(), "{name}: units belong to a faction");
            assert!(def.half_extents().is_some(), "{name}: units have a collision box");
        }
    }
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
        (6.0, 3.0, 4)
    );
    let shield = pk.riot_shield.unwrap();
    assert_eq!((shield.speed_factor, shield.damage_factor), (0.5, 0.5));
    assert!(shield.toggle_ticks < 8, "under half a second (8 ticks)");
    assert_eq!((pk.unit_control_cost, gd.tunnel_space_cost), (Some(1), Some(1)));
}
