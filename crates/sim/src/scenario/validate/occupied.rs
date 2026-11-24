//! Footprints that overlap where they may not (`core/entities.md > Placement Validation`): only a plate and the
//! patch it is placed onto share a tile ([`ObjectKind::may_overlap`]; design change 0096: no buildings on
//! resources).

use std::collections::BTreeMap;

use crate::defs::ObjectKind;
use crate::object::Placement;
use crate::units::TilePos;

/// The tiles taken so far, each with the objects on it, in the order they came.
pub(super) struct Occupied<T>(BTreeMap<TilePos, Vec<(T, ObjectKind)>>);

impl<T> Default for Occupied<T> {
    fn default() -> Self {
        Self(BTreeMap::new())
    }
}

impl<T: Copy> Occupied<T> {
    /// Takes the tiles of `placement` for `owner`, an object of `kind`. The first of them that an earlier object
    /// covers and `kind` may not overlap, with the first such object on it: of two objects in conflict, the later
    /// one is reported.
    pub(super) fn take(&mut self, kind: ObjectKind, placement: &Placement, owner: T) -> Option<(T, TilePos)> {
        let mut overlap = None;
        for tile in placement.tiles() {
            let on_tile = self.0.entry(tile).or_default();
            if overlap.is_none()
                && let Some(&(first, _)) = on_tile.iter().find(|(_, other)| !kind.may_overlap(*other))
            {
                overlap = Some((first, tile));
            }
            on_tile.push((owner, kind));
        }
        overlap
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(kind: ObjectKind, x: i32, y: i32) -> (ObjectKind, Placement) {
        (kind, Placement::new(TilePos::new(x, y), kind.def().footprint))
    }

    /// A plate goes onto an uncovered patch, in either order, and nothing else shares its tiles.
    #[test]
    fn only_a_plate_on_an_uncovered_patch_overlaps() {
        use ObjectKind::{CrystalPatch, ExtractionPlate, Generator};
        let mut occupied = Occupied::default();
        let mut take = |n: usize, (kind, placement): (ObjectKind, Placement)| occupied.take(kind, &placement, n);
        let tile = TilePos::new;
        assert_eq!(take(0, at(CrystalPatch, 2, 2)), None);
        assert_eq!(take(1, at(ExtractionPlate, 2, 2)), None, "a plate on the patch");
        assert_eq!(
            take(2, at(ExtractionPlate, 2, 2)),
            Some((1, tile(2, 2))),
            "the patch is covered"
        );
        assert_eq!(
            take(3, at(CrystalPatch, 2, 2)),
            Some((0, tile(2, 2))),
            "a patch on a patch"
        );
        // A plate first, then its patch.
        assert_eq!(take(4, at(ExtractionPlate, 5, 5)), None);
        assert_eq!(take(5, at(CrystalPatch, 5, 5)), None);
        // A Generator (2×2) over the patch at (5, 5) and its plate: the first tile in conflict, and the first object
        // there.
        assert_eq!(take(6, at(Generator, 4, 4)), Some((4, tile(5, 5))));
        assert_eq!(take(7, at(Generator, 7, 7)), None);
    }
}
