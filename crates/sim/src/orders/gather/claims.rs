//! The claims of one phase 5a on the patches (`> Patch Occupancy (one Drone per patch)`; the module docs of
//! `orders::gather`), the hatches and the Supplies of the stations.

use crate::ids::{ObjectId, PlayerId};
use crate::shape::Shape;
use crate::sides::SideLetter;

/// `> Patch Occupancy (one Drone per patch)`: the patches claimed so far in this tick's phase 5a, each with the
/// worker that claimed it and its owner: one that starts mining it, or one that sets off for it, a free patch it found
/// (design change 0133: "the lowest-id Drone whose check sees the patch free claims it"). Scratch of one phase, never
/// state.
///
/// And the hatches a worker began to unload at in this tick (`factions/syndicate/tunnels.md > Tunnel > SymmetryType -
/// ABCD`: "Only one Agent may drop off resources at a side at a time"), as `(drop-off, side)`: of two workers that
/// come to a free hatch in one tick, the one with the lower id unloads.
///
/// And the station of each pick-up that began in this tick (`factions/syndicate/units/agent.md > Gathering >
/// Supplies`, design change 0256 point 2: "A pick-up starts only if a Supply remains that no running pick-up has
/// claimed"): each claims one of its Supplies, so the lowest ids get the last ones.
///
/// And the spots at the stations that workers set off for in this tick, as `(station, the worker's Silhouette there)`
/// (`station::spot`): of workers sent together, each heads for a spot the ones before it left free.
#[derive(Debug, Default)]
pub(crate) struct Claims(
    Vec<(ObjectId, ObjectId, Option<PlayerId>)>,
    Vec<(ObjectId, SideLetter)>,
    Vec<ObjectId>,
    Vec<(ObjectId, Shape)>,
);

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

    /// How many of `station`'s Supplies the pick-ups that began in this tick claimed.
    pub(super) fn supplies_taken(&self, station: ObjectId) -> u32 {
        self.2.iter().filter(|&&s| s == station).count() as u32
    }

    /// A pick-up begins at `station`, and claims one of its Supplies.
    pub(super) fn take_supply(&mut self, station: ObjectId) {
        self.2.push(station);
    }

    /// The spots at `station` that workers set off for in this tick.
    pub(super) fn spots(&self, station: ObjectId) -> impl Iterator<Item = Shape> + '_ {
        (self.3.iter())
            .filter(move |&&(s, _)| s == station)
            .map(|&(_, spot)| spot)
    }

    /// A worker sets off for the spot `spot` at `station`.
    pub(super) fn take_spot(&mut self, station: ObjectId, spot: Shape) {
        self.3.push((station, spot));
    }
}
