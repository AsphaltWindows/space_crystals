//! What the workers' scans read of the snapshot (`orders::gather`; `factions/colonists/units.md > Drone > Patch
//! Occupancy (one Drone per patch)`): the patches and the drop-offs, who mines and who waits where, and whether a
//! worker can reach a patch.

use std::collections::BTreeMap;

use crate::economy::gathering::{is_drop_off, worker_def};
use crate::ids::ObjectId;
use crate::nav;
use crate::object::{Object, ResourceState};
use crate::orders::OrderKind;
use crate::orders::state::{Behaviour, GatherPhase};
use crate::shape::Shape;
use crate::units::{SU_PER_GU, TilePos};

use super::{Unit, View, World};

/// The gathering part of the snapshot, taken with the rest of it (`World::new`).
#[derive(Debug, Default)]
pub(super) struct Gathering {
    /// Indices into the snapshot's views of the Space Crystals patches and of the drop-offs, in id order: what a
    /// worker's scans look at. They run on events only: an arrival at a patch it can't mine, a waiting worker's
    /// look round every `orders::gather::RECHECK_TICKS`, a return without a drop-off.
    patches: Vec<u32>,
    drop_offs: Vec<u32>,
    /// `> Patch Occupancy`: the patches a worker was mining as the phase began, sorted.
    mined: Vec<ObjectId>,
    /// The patches under a mining structure (a GDO plate, design change 0137), sorted.
    covered: Vec<ObjectId>,
    /// The workers waiting at a patch within their MiningReach of it, as `(patch, worker)`, sorted: the patch is
    /// theirs as soon as it frees (design change 0133).
    waiting: Vec<(ObjectId, ObjectId)>,
}

impl Gathering {
    pub(super) fn new(objects: &BTreeMap<ObjectId, Object>) -> Self {
        let mut gathering = Self::default();
        for (index, o) in objects.values().enumerate() {
            if matches!(o.resource, Some(ResourceState::CrystalPatch { .. })) {
                gathering.patches.push(index as u32);
            }
            // An Egress Node being warped in takes nothing until it is complete (design change 0116,
            // `crate::transform`).
            if is_drop_off(o.kind) && !o.is_transforming() {
                gathering.drop_offs.push(index as u32);
            }
            gathering.covered.extend(o.extractor.map(|extractor| extractor.patch));
            match job(o) {
                Some((patch, GatherPhase::Mining { .. })) => gathering.mined.push(patch),
                Some((patch, GatherPhase::Waiting { .. })) if within_reach(objects, o, patch) => {
                    gathering.waiting.push((patch, o.id));
                }
                _ => {}
            }
        }
        gathering.mined.sort_unstable();
        gathering.covered.sort_unstable();
        gathering.waiting.sort_unstable();
        gathering
    }
}

/// The patch and the phase of the Gather that `o` carries out, if it is a worker with one.
fn job(o: &Object) -> Option<(ObjectId, GatherPhase)> {
    o.worker.as_ref()?;
    let active = o.orders.as_ref()?.current.as_ref()?;
    match (active.order.kind, &active.state) {
        (OrderKind::Gather(patch), Behaviour::Gather(gather)) => Some((patch, gather.phase)),
        _ => None,
    }
}

/// Whether worker `o` is within its MiningReach of `patch` (`factions/colonists/units.md > Drone > Gathering`).
fn within_reach(objects: &BTreeMap<ObjectId, Object>, o: &Object, patch: ObjectId) -> bool {
    let (Some(def), Some(patch)) = (worker_def(o.kind), objects.get(&patch)) else {
        return false;
    };
    Shape::of(o).gap(&Shape::of(patch)) <= def.mining_reach
}

impl World<'_> {
    /// The Space Crystals patches, in id order.
    pub(crate) fn patches(&self) -> impl Iterator<Item = &View> + '_ {
        (self.gathering.patches.iter()).map(|&index| &self.views[index as usize])
    }

    /// The structures that take workers' loads, in id order.
    pub(crate) fn drop_offs(&self) -> impl Iterator<Item = &View> + '_ {
        (self.gathering.drop_offs.iter()).map(|&index| &self.views[index as usize])
    }

    /// `> Patch Occupancy (one Drone per patch)`: whether a worker, of any player, was mining `patch` as the
    /// phase began.
    pub(crate) fn patch_mined(&self, patch: ObjectId) -> bool {
        self.gathering.mined.binary_search(&patch).is_ok()
    }

    /// Whether a mining structure stands on `patch` (design change 0137: the patch counts as gone for workers).
    pub(crate) fn patch_covered(&self, patch: ObjectId) -> bool {
        self.gathering.covered.binary_search(&patch).is_ok()
    }

    /// Design change 0133 (`> Patch Occupancy > Waiting and re-checking`, item 4, and the Director's choice):
    /// whether a worker other than `me` waits at `patch` within its MiningReach. Such a worker mines the patch as
    /// soon as it frees, so the patch is occupied for every other worker.
    pub(crate) fn waited_at(&self, patch: ObjectId, me: ObjectId) -> bool {
        let first = self.gathering.waiting.partition_point(|&(p, _)| p < patch);
        self.gathering.waiting[first..]
            .iter()
            .take_while(|&&(p, _)| p == patch)
            .any(|&(_, worker)| worker != me)
    }

    /// Design change 0141 (`> Patch Occupancy > A patch it can't reach`): whether `unit` can get to the edge of
    /// `target`, a footprint, as far as its owner knows the map: a tile next to the footprint, where the unit can
    /// stand against it, is in the unit's component of the grid it plans on (`paths::NavState::planning`; the
    /// tiles diagonal to it count, a slight overestimate). Cheap: a ring of tiles, no search.
    ///
    /// The grids leave units out, so a patch walled off by units counts as reachable here: the worker finds out
    /// when its leg ends short of it (`orders::gather`). A unit without a grid (an air unit), or standing on a tile
    /// its grid blocks, reaches everything.
    pub(crate) fn reachable(&self, unit: &Unit, target: &Shape) -> bool {
        let def = unit.kind.def();
        let grid = (def.unit.as_ref())
            .and_then(|unit| unit.base_def().nav_class())
            .zip(def.silhouette())
            .and_then(|(class, silhouette)| {
                let planning = self.nav.planning(unit.owner);
                planning.grid(class, nav::size_class(silhouette))
            });
        let Some(grid) = grid else {
            return true;
        };
        let Some(component) = grid.component(grid.tile_of(unit.pos)) else {
            return true;
        };
        // The footprint's tiles, `x0..=x1` and `y0..=y1`, and the anchor tiles of the unit's blocks of `size` tiles
        // that touch it.
        let (lo, hi) = target.bounds();
        let (x0, y0) = ((lo.x / SU_PER_GU).floor() as i32, (lo.y / SU_PER_GU).floor() as i32);
        let (x1, y1) = (
            (hi.x / SU_PER_GU).ceil() as i32 - 1,
            (hi.y / SU_PER_GU).ceil() as i32 - 1,
        );
        let size = i32::from(grid.size());
        // Whether the block anchored at `a` overlaps `from..=to`: its last tile, `a + size - 1`, isn't before `from`.
        let inside = |a: i32, from: i32, to: i32| a + size > from && a <= to;
        (y0 - size..=y1 + 1).any(|y| {
            (x0 - size..=x1 + 1).any(|x| {
                !(inside(x, x0, x1) && inside(y, y0, y1)) && grid.component(TilePos::new(x, y)) == Some(component)
            })
        })
    }
}
