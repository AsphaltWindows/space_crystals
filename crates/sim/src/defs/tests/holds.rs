//! The types that hold units (plan decision T60, `crate::holding`): their holds (slice M5-F1), their entrance and their
//! exit (slice M5-F3), the fire of the units inside (slice M5-N2), the augmentation orders of the units inside (slice
//! M5-N1b).

use crate::augmentation;
use crate::combat;
use crate::defs::{ExitRule, ObjectDef, ObjectKind, StructureDef};

/// `defs_are_consistent` for the holds of a structure type (`StructureDef::holds`) and its occupants' fire
/// (`StructureDef::occupants_fire`).
///
/// (M5-F1) A building that holds units (plan decision T60, `crate::holding`): an entrance side and an exit side its
/// SymmetryType letters once each, apart; a pool that holds some; units of its own faction, each listed once. One that
/// also produces releases through the same doorway, on its exit side
/// (`factions/cults/structures/armory.md > ExitSide - opposite short side (C)`: "Trained units and ejected Recruits
/// exit through the exit side, one at a time through the exit doorway"), but (M5-N7) for a runway's aircraft, which
/// take off along it and use no doorway (`ExitRule::Runway`): the units it holds come out at the runway's far end, its
/// `toward` side (the Airstrip's C).
/// (M5-N2) A building with one door lets everyone out at once through its entrance (`ExitRule::AtOnce`, the
/// Garrison's: "one door, entrance and exit"), and produces nothing.
///
/// (M5-N2) The units inside fire only from a building that holds them, has no attack of its own and takes them in:
/// every kind it takes in fires, its own attack or its stand-in's, and the kinds its rules name are kinds it takes in
/// (`OccupantFire`: the Garrison's "Without units inside, the Garrison does not attack").
///
/// (M5-N1b) Augmentation orders are of the units inside (`ProductionDef::augment`, `factions/cults/augmentation.md >
/// Altar > Augmentation Orders`): a producer that takes them holds units, of kinds it can augment, and lists each order
/// once, each taking time.
pub(super) fn holds_are_consistent(def: &ObjectDef, structure: &StructureDef) {
    let name = def.name;
    if let Some(production) = structure.production.as_ref().filter(|p| !p.augment.is_empty()) {
        let holds = (structure.holds.as_ref()).unwrap_or_else(|| panic!("{name}: augments, holds none"));
        for &kind in holds.accepts {
            assert!(
                augmentation::is_augmentable(kind),
                "{name}: takes in {kind:?}, which it can't augment"
            );
        }
        for (i, recipe) in production.augment.iter().enumerate() {
            assert!(recipe.frames > 0, "{name}: {recipe:?}");
            let twice = production.augment[..i].iter().any(|r| r.order == recipe.order);
            assert!(!twice, "{name} lists {:?} twice", recipe.order);
        }
    }
    if let Some(fire) = &structure.occupants_fire {
        let holds = structure
            .holds
            .as_ref()
            .unwrap_or_else(|| panic!("{name}: its occupants fire, none inside"));
        assert!(structure.attack.is_none(), "{name}: fires itself and its occupants");
        // (M6-CM) A kind inside fires its stand-in's attack, or its own, or nothing (design change 0450: "**Disciples
        // and illusions don't fire** from inside (user, 2026-11-27): they are only sheltered"): every stand-in has an
        // attack, and some kind inside fires.
        for &(kind, stand_in) in fire.stand_in {
            let weapon = combat::weapon_of(stand_in);
            assert!(
                weapon.is_some(),
                "{name}: {kind:?} inside fires {stand_in:?}'s attack, which has none"
            );
        }
        let fires = |kind: ObjectKind| combat::weapon_of(fire.weapon_kind(kind)).is_some();
        assert!(
            holds.accepts.iter().any(|&kind| fires(kind)),
            "{name}: no one inside fires"
        );
        let named = (fire.stand_in.iter())
            .map(|&(unit, _)| unit)
            .chain(fire.bonus_for.iter().copied());
        for kind in named {
            assert!(
                holds.accepts.contains(&kind),
                "{name}: its fire names {kind:?}, which it doesn't take in"
            );
        }
        assert!(
            fire.range_bonus_gu >= 0.0,
            "{name}: a range bonus of {}",
            fire.range_bonus_gu
        );
    }
    let Some(holds) = &structure.holds else {
        return;
    };
    let letters = structure.symmetry.letters();
    let count = |letter| letters.iter().filter(|&&l| l == letter).count();
    let (exit, one_door) = match holds.exit {
        ExitRule::Side(exit) => (exit, false),
        ExitRule::AtOnce(door) => (door, true),
        _ => panic!("{name}: only an exit side or one door is built for a building that holds units"),
    };
    for letter in [holds.entrance, exit] {
        assert_eq!(count(letter), 1, "{name}: side {letter:?} of its holds");
    }
    if one_door {
        assert_eq!(holds.entrance, exit, "{name}: one door, entrance and exit");
        assert!(structure.production.is_none(), "{name}: produces out of its one door");
    } else {
        assert_ne!(holds.entrance, exit, "{name}: in and out at one side");
    }
    assert!(holds.capacity > 0, "{name}: holds none");
    assert!(!holds.accepts.is_empty(), "{name}: takes no unit in");
    for (i, &kind) in holds.accepts.iter().enumerate() {
        let unit = kind.def();
        assert!(
            unit.unit.is_some() && unit.faction == def.faction,
            "{name} takes {kind:?}"
        );
        assert!(!holds.accepts[..i].contains(&kind), "{name} lists {kind:?} twice");
    }
    if let Some(production) = &structure.production {
        match production.exits {
            ExitRule::Runway { toward, .. } => {
                assert_eq!(
                    holds.exit,
                    ExitRule::Side(toward),
                    "{name}: out at the runway's far end"
                );
            }
            exits => assert_eq!(exits, holds.exit, "{name}: one exit doorway"),
        }
    }
}

/// Design change 0347 point 1 (`core/entities.md > Entrance and exit (production and research buildings)`): "For
/// training, augmentation and unit-producing buildings the exit is the side opposite the entrance". Every type that
/// takes units in by an entrance side and lets them out by an exit side, as it holds them (`HoldsDef::exit`) and as it
/// produces (`ProductionDef::exits`), has the two sides opposite each other in its own frame
/// (`SymmetryType::letters`: top, right, bottom, left), and so in every orientation, which turns the four sides
/// together (`crate::sides::sides`). The Armory's B and C, ABAC, are its two short sides. The design's exceptions come
/// with their types: the Airstrip, which "places a finished aircraft directly on its runway", and the Garrison, whose
/// "one door" is both (`factions/cults/structures/garrison.md`).
#[test]
fn every_exit_is_opposite_its_entrance() {
    let mut checked = Vec::new();
    for &kind in ObjectKind::ALL {
        let Some(structure) = kind.def().structure.as_ref() else {
            continue;
        };
        let Some(holds) = &structure.holds else {
            continue;
        };
        let letters = structure.symmetry.letters();
        let side = |letter| letters.iter().position(|&l| l == letter);
        let entrance = side(holds.entrance).unwrap_or_else(|| panic!("{kind:?}: no side {:?}", holds.entrance));
        let exits = std::iter::once(holds.exit).chain(structure.production.map(|production| production.exits));
        for exit in exits {
            // An exit by no side has no side to be opposite, and one door (`ExitRule::AtOnce`, the Garrison's) is the
            // entrance itself (`defs_are_consistent`).
            let ExitRule::Side(exit) = exit else {
                continue;
            };
            let exit = side(exit).unwrap_or_else(|| panic!("{kind:?}: no side {exit:?}"));
            assert_eq!(
                (entrance + 2) % 4,
                exit,
                "{kind:?}: in at {:?}, out at {:?} of {:?}",
                letters[entrance],
                letters[exit],
                structure.symmetry
            );
        }
        checked.push(kind);
    }
    assert!(checked.contains(&ObjectKind::Armory), "{checked:?}");
}
