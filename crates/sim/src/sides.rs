//! Which side of a placed structure is which: `core/entities.md > Structure Type > Side lettering convention`
//! (design change 0079). A structure type's SymmetryType letters its four sides clockwise from the top edge in the
//! default orientation (rotation 0, unflipped): top, right, bottom, left. Rotating or flipping the building carries
//! its sides with it. [`sides`] is the one place that says where a placed structure's lettered sides face, for every
//! rule that needs a side (the Barracks' exit on side B, the placement ghost's labels).

use serde::{Deserialize, Serialize};

use crate::defs::ObjectKind;
use crate::scenario::BuildingOrientation;
use crate::symmetry::Orientation;

/// A side's letter in its type's SymmetryType (`core/entities.md > Structure Type > SymmetryType`).
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum SideLetter {
    A,
    B,
    C,
    D,
}

/// The map directions the entries of a [`sides`] array face, in its order: north (the top edge as the map is shown),
/// east, south, west, as offsets in the sim's axes (+x east, +y south).
pub const DIRECTIONS: [(i64, i64); 4] = [(0, -1), (1, 0), (0, 1), (-1, 0)];

/// The letters of a `kind` structure's sides when it is placed turned by `orientation`, facing north, east, south
/// and west ([`DIRECTIONS`]); `None` for a kind without a SymmetryType (units and map resources). In the default
/// orientation they are the SymmetryType's letters themselves: a Barracks (3 wide, 2 tall, ABAC) has A on top, B on
/// the right, A at the bottom and C on the left; turned 90° clockwise its B faces south.
pub fn sides(kind: ObjectKind, orientation: BuildingOrientation) -> Option<[SideLetter; 4]> {
    let structure = kind.def().structure.as_ref()?;
    Some(turned(structure.symmetry.letters(), orientation))
}

/// `letters`, the sides of a building in its own frame (top, right, bottom, left), where they face once it is
/// turned by `orientation` (`Orientation::of_building`: flipped in its own frame, then rotated clockwise): each
/// side's outward direction is turned, and the letter goes to the direction it turns to.
fn turned(letters: [SideLetter; 4], orientation: BuildingOrientation) -> [SideLetter; 4] {
    let turn = Orientation::of_building(orientation.rotation, orientation.flip_h, orientation.flip_v);
    let mut placed = letters;
    for (letter, own) in letters.into_iter().zip(DIRECTIONS) {
        let facing = turn.apply(own);
        if let Some(i) = DIRECTIONS.iter().position(|&d| d == facing) {
            placed[i] = letter;
        }
    }
    placed
}

#[cfg(test)]
mod tests {
    use super::SideLetter::{A, B, C, D};
    use super::*;
    use crate::defs::SymmetryType;
    use crate::object::Rotation;

    /// Every orientation a building can be placed in: the 4 rotations, each unflipped and flipped left and right.
    /// (A top-and-bottom flip is the same as a left-and-right one turned 180°: tested below.)
    const ROTATIONS: [Rotation; 4] = [Rotation::R0, Rotation::R90, Rotation::R180, Rotation::R270];

    fn oriented(rotation: Rotation, flip_h: bool, flip_v: bool) -> BuildingOrientation {
        BuildingOrientation {
            rotation,
            flip_h,
            flip_v,
        }
    }

    /// Design change 0079: the Barracks' B, its exit, is its right short side unturned, and its bottom end turned
    /// 90° clockwise; every orientation carries the letters with it.
    #[test]
    fn the_barracks_sides_in_all_eight_orientations() {
        let barracks = |rotation, flip_h| sides(ObjectKind::Barracks, oriented(rotation, flip_h, false)).unwrap();
        // North, east, south, west.
        assert_eq!(barracks(Rotation::R0, false), [A, B, A, C]);
        assert_eq!(barracks(Rotation::R90, false), [C, A, B, A]);
        assert_eq!(barracks(Rotation::R180, false), [A, C, A, B]);
        assert_eq!(barracks(Rotation::R270, false), [B, A, C, A]);
        // Flipped left and right first: B on the left, C on the right.
        assert_eq!(barracks(Rotation::R0, true), [A, C, A, B]);
        assert_eq!(barracks(Rotation::R90, true), [B, A, C, A]);
        assert_eq!(barracks(Rotation::R180, true), [A, B, A, C]);
        assert_eq!(barracks(Rotation::R270, true), [C, A, B, A]);
    }

    /// A square kind (AAAA: the Generator, the Robo Portal, …) shows A on every side, however it is placed.
    #[test]
    fn a_square_kinds_sides_in_all_eight_orientations() {
        let aaaa: Vec<ObjectKind> = ObjectKind::ALL
            .iter()
            .copied()
            .filter(|kind| {
                kind.def()
                    .structure
                    .as_ref()
                    .is_some_and(|s| s.symmetry == SymmetryType::AAAA)
            })
            .collect();
        assert!(aaaa.contains(&ObjectKind::Generator) && aaaa.contains(&ObjectKind::RoboPortal));
        for kind in aaaa {
            for rotation in ROTATIONS {
                for flip_h in [false, true] {
                    let at = oriented(rotation, flip_h, false);
                    assert_eq!(sides(kind, at), Some([A; 4]), "{kind:?} {at:?}");
                }
            }
        }
    }

    /// Design change 0079 for the Bio Portal (AABB, `factions/colonists/structures/bio_portal.md`): unturned, A on
    /// top and on the right, B at the bottom and on the left. Its two A sides, its exits (`> Exit Behavior`), stay
    /// side by side in every orientation, and a flip gives what a turn would.
    #[test]
    fn the_bio_portals_sides_in_all_eight_orientations() {
        let portal = |rotation, flip_h| sides(ObjectKind::BioPortal, oriented(rotation, flip_h, false)).unwrap();
        // North, east, south, west.
        assert_eq!(portal(Rotation::R0, false), [A, A, B, B]);
        assert_eq!(portal(Rotation::R90, false), [B, A, A, B]);
        assert_eq!(portal(Rotation::R180, false), [B, B, A, A]);
        assert_eq!(portal(Rotation::R270, false), [A, B, B, A]);
        // Flipped left and right first: A on top and on the left.
        assert_eq!(portal(Rotation::R0, true), [A, B, B, A]);
        assert_eq!(portal(Rotation::R90, true), [A, A, B, B]);
        assert_eq!(portal(Rotation::R180, true), [B, A, A, B]);
        assert_eq!(portal(Rotation::R270, true), [B, B, A, A]);
        for rotation in ROTATIONS {
            for flip_h in [false, true] {
                let placed = portal(rotation, flip_h);
                let a: Vec<usize> = (0..4).filter(|&i| placed[i] == A).collect();
                assert_eq!(a.len(), 2, "{rotation:?}, flipped: {flip_h}");
                assert!((a[0] + 1) % 4 == a[1] || (a[1] + 1) % 4 == a[0], "adjacent: {placed:?}");
            }
        }
    }

    /// With four different letters (ABCD) the eight orientations are eight different arrangements: the rotations
    /// keep the clockwise order, the flips reverse it. A top-and-bottom flip is the left-and-right one turned 180°.
    #[test]
    fn eight_orientations_give_eight_arrangements() {
        let mut seen = Vec::new();
        for rotation in ROTATIONS {
            for flip_h in [false, true] {
                let placed = turned([A, B, C, D], oriented(rotation, flip_h, false));
                // Clockwise from north: a rotation of A, B, C, D, or of D, C, B, A after a flip.
                let start = placed.iter().position(|&l| l == A).unwrap();
                let clockwise: Vec<SideLetter> = (0..4).map(|i| placed[(start + i) % 4]).collect();
                let order = if flip_h { [A, D, C, B] } else { [A, B, C, D] };
                assert_eq!(clockwise, order, "{rotation:?}, flipped: {flip_h}");
                assert!(!seen.contains(&placed), "{rotation:?}, flipped: {flip_h}");
                seen.push(placed);
                let quarter_turns = ROTATIONS.iter().position(|&r| r == rotation).unwrap();
                let flipped_v = oriented(ROTATIONS[(quarter_turns + 2) % 4], !flip_h, true);
                assert_eq!(turned([A, B, C, D], flipped_v), placed, "{flipped_v:?}");
            }
        }
        // Unturned: the letters themselves, top first.
        assert_eq!(turned([A, B, C, D], BuildingOrientation::default()), [A, B, C, D]);
    }

    #[test]
    fn units_and_resources_have_no_sides() {
        for kind in [ObjectKind::Peacekeeper, ObjectKind::CrystalPatch] {
            assert_eq!(sides(kind, BuildingOrientation::default()), None);
        }
    }
}
