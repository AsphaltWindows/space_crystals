//! The claims of one phase 5a on the patches (`> Patch Occupancy (one Drone per patch)`; the module docs of
//! `orders::gather`).

use crate::ids::{ObjectId, PlayerId};
use crate::sides::SideLetter;

/// `> Patch Occupancy (one Drone per patch)`: the patches claimed so far in this tick's phase 5a, each with the
/// worker that claimed it and its owner: one that starts mining it, or one that sets off for it, a free patch it found
/// (design change 0133: "the lowest-id Drone whose check sees the patch free claims it"). Scratch of one phase, never
/// state.
///
/// And the hatches a worker began to unload at in this tick (`factions/syndicate/tunnels.md > Tunnel > SymmetryType -
/// ABCD`: "Only one Agent may drop off resources at a side at a time"), as `(drop-off, side)`: of two workers that
/// come to a free hatch in one tick, the one with the lower id unloads.
#[derive(Debug, Default)]
pub(crate) struct Claims(Vec<(ObjectId, ObjectId, Option<PlayerId>)>, Vec<(ObjectId, SideLetter)>);

impl Claims {
    /// Whether a worker other than `me` claimed `patch`.
    pub(super) fn by_other(&self, patch: ObjectId, me: ObjectId) -> bool {
        self.0.iter().any(|&(p, worker, _)| p == patch && worker != me)
    }

    /// Whether a worker of `owner`'s other than `me` claimed `patch`: all `owner` knows of the claims on a patch out
    /// of its sight (design change 0139).
    pub(super) fn by_other_of(&self, patch: ObjectId, me: ObjectId, owner: Option<PlayerId>) -> bool {
        self.0
            .iter()
            .any(|&(p, worker, of)| p == patch && worker != me && of == owner)
    }

    pub(super) fn claim(&mut self, patch: ObjectId, worker: ObjectId, owner: Option<PlayerId>) {
        self.0.push((patch, worker, owner));
    }

    /// Whether a worker began to unload at the hatch on `side` of `node` in this tick.
    pub(super) fn hatch_taken(&self, node: ObjectId, side: SideLetter) -> bool {
        self.1.contains(&(node, side))
    }

    pub(super) fn take_hatch(&mut self, node: ObjectId, side: SideLetter) {
        self.1.push((node, side));
    }
}
