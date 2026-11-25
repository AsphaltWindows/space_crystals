//! The tiles a footprint covers, and the per-tile sets of problems the check keeps for them.

use super::problems::TileProblem;
use crate::nav::TileRect;
use crate::units::TilePos;

/// The tiles a footprint covers, which may lie anywhere: off the map, even past the last tile coordinate.
#[derive(Copy, Clone)]
pub(super) struct Footprint {
    pub(super) origin: TilePos,
    pub(super) w: u32,
    pub(super) h: u32,
}

impl Footprint {
    /// How many tiles it covers (at most 16 × 16: `defs::tests::defs_are_consistent`).
    pub(super) fn len(self) -> usize {
        self.w as usize * self.h as usize
    }

    pub(super) fn rect(self) -> TileRect {
        TileRect::new(self.origin, self.w, self.h)
    }

    /// Its tiles row by row from the north-west, each with its index. Those whose coordinates would overflow are
    /// left out: the origin tile is then far off the map, so the check fails on it anyway.
    pub(super) fn tiles(self) -> impl Iterator<Item = (usize, TilePos)> {
        (0..self.h)
            .flat_map(move |dy| (0..self.w).map(move |dx| (dx, dy)))
            .enumerate()
            .filter_map(move |(i, (dx, dy))| {
                let x = self.origin.x.checked_add_unsigned(dx)?;
                let y = self.origin.y.checked_add_unsigned(dy)?;
                Some((i, TilePos::new(x, y)))
            })
    }

    /// Adds `problem` to the tiles that `covered` picks.
    pub(super) fn mark(self, found: &mut [u8], problem: TileProblem, covered: impl Fn(TilePos) -> bool) {
        for (i, tile) in self.tiles() {
            if covered(tile) {
                found[i] |= problem.bit();
            }
        }
    }

    /// Takes every problem but those of the bits `kept` off the tiles that `exempt` picks.
    pub(super) fn keep_only(self, found: &mut [u8], kept: u8, exempt: impl Fn(TilePos) -> bool) {
        for (i, tile) in self.tiles() {
            if exempt(tile) {
                found[i] &= kept;
            }
        }
    }
}
