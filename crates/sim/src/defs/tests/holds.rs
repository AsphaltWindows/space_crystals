//! The types that hold units (plan decision T60, `crate::holding`): their entrance and their exit (slice M5-F3).

use crate::defs::{ExitRule, ObjectKind};

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
            // An exit by no side has no side to be opposite.
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
