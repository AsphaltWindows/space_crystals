//! What the workers' scans read of the snapshot (`orders::gather`; `factions/colonists/units/drone.md > Patch
//! Occupancy (one Drone per patch)`): the patches and the drop-offs, who mines and who waits where, whether a
//! worker can reach a patch, and the patches and the fixed objects as a worker's owner knows them (design change
//! 0139: [`World::known_patch`], [`World::known_fixed`]). And the Supply Delivery Stations (slice M4-I): the Supplies
//! no pick-up has claimed ([`World::unclaimed_supplies`]), and a station as a worker's owner knows it
//! ([`World::known_station`]).

use std::collections::BTreeMap;

use glam::Vec2;

use crate::defs::Resource;
use crate::economy::drop_off::{DropPoint, drop_point_of};
use crate::economy::gathering::worker_def;
use crate::economy::stations::{is_station, supplies};
use crate::ids::{ObjectId, PlayerId};
use crate::object::{Object, ResourceState};
use crate::orders::OrderKind;
use crate::orders::state::{Behaviour, GatherPhase};
use crate::paths;
use crate::shape::Shape;
use crate::sides::SideLetter;
use crate::targets;
use crate::units::{SU_PER_GU, Tick, TilePos};
use crate::vision::LastKnown;

use super::{Known, Unit, View, World};

/// A Space Crystals patch as a worker's owner knows it (design changes 0131, 0137 and 0139, `crate::targets`):
/// as it is while the owner sees it, and as the owner remembers it while it doesn't, standing or not.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct KnownPatch {
    pub id: ObjectId,
    /// Its centre and its shape.
    pub pos: Vec2,
    pub shape: Shape,
    /// The owner sees it now. Otherwise it remembers it out of its sight (`core/vision.md > Explored`), or knows it
    /// from the start of the game (design change 0159).
    pub seen: bool,
    /// A mining structure stands on it as far as the owner knows: the patch counts as gone for workers (design
    /// change 0137).
    pub covered: bool,
}

/// A Supply Delivery Station as a worker's owner knows it (design change 0139, `crate::targets`; slice M4-I): where
/// it stands while the owner sees it, and where the owner remembers it while it doesn't. What it holds counts only
/// while the owner sees it ([`World::unclaimed_supplies`]): a worker picks up within reach of it, where it sees it.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct KnownStation {
    pub id: ObjectId,
    pub shape: Shape,
    /// The owner sees it now.
    pub seen: bool,
}

/// Where a drop-off takes each resource (`economy::drop_off::drop_point`).
#[derive(Copy, Clone, Debug)]
struct Points {
    crystals: DropPoint,
    supplies: DropPoint,
}

impl Points {
    fn of(&self, resource: Resource) -> &DropPoint {
        match resource {
            Resource::Crystals => &self.crystals,
            Resource::Supplies => &self.supplies,
        }
    }
}

/// The gathering part of the snapshot, taken with the rest of it (`World::new`).
#[derive(Debug, Default)]
pub(super) struct Gathering {
    /// Indices into the snapshot's views of the drop-offs, in id order, each with where it takes each resource
    /// (`economy::drop_off`): what a worker's scan for one looks at, on a return without a drop-off. (The patches it
    /// looks round among are its owner's records, [`World::known_patches_near`].)
    drop_offs: Vec<(u32, Points)>,
    /// The Supply Delivery Stations, in id order, each with the Supplies it holds that no pick-up under way has
    /// claimed as the phase began (`factions/syndicate/units/agent.md > Gathering > Supplies`, design change 0256
    /// point 2: "A pick-up starts only if a Supply remains that no running pick-up has claimed"). Derived from the
    /// workers' orders, as a patch's occupancy is.
    stations: Vec<(ObjectId, u32)>,
    /// The hatches a worker was unloading at as the phase began, as `(drop-off, side)`, sorted: "Only one Agent may
    /// drop off resources at a side at a time" (`factions/syndicate/tunnels.md > Tunnel > SymmetryType - ABCD`).
    /// Derived from the workers' orders, as a patch's occupancy is, so that it can't outlive the order that holds it
    /// (plan decision T34).
    unloading: Vec<(ObjectId, SideLetter)>,
    /// The workers waiting at a hatch within their DropOffReach of it, as `(drop-off, side, since, worker)`, sorted:
    /// the line, in the order they arrived, ties by id (design change 0255: "the first to arrive delivers next").
    in_line: Vec<(ObjectId, SideLetter, Tick, ObjectId)>,
    /// `> Patch Occupancy`: the patches a worker was mining as the phase began, sorted.
    mined: Vec<ObjectId>,
    /// The patches under a mining structure (a GDO plate, design change 0137), each with the structure's owner,
    /// sorted.
    covered: Vec<(ObjectId, Option<PlayerId>)>,
    /// The workers waiting at a patch within their MiningReach of it, as `(patch, worker)`, sorted: the patch is
    /// theirs as soon as it frees (design change 0133).
    waiting: Vec<(ObjectId, ObjectId)>,
}

impl Gathering {
    pub(super) fn new(objects: &BTreeMap<ObjectId, Object>) -> Self {
        let mut gathering = Self::default();
        // The station of each pick-up under way.
        let mut picking: Vec<ObjectId> = Vec::new();
        for (index, o) in objects.values().enumerate() {
            // An Egress Node being warped in takes nothing until it is complete (design change 0116,
            // `crate::transform`), nor does a Tunnel under construction (design change 0234: "no drop-offs",
            // `Object::is_operational`).
            if o.is_operational()
                && let Some(crystals) = drop_point_of(o, Resource::Crystals)
                && let Some(supplies) = drop_point_of(o, Resource::Supplies)
            {
                gathering.drop_offs.push((index as u32, Points { crystals, supplies }));
            }
            if is_station(o.kind)
                && let Some(held) = supplies(o)
            {
                gathering.stations.push((o.id, held));
            }
            picking.extend(pick_up(objects, o));
            gathering
                .covered
                .extend(o.extractor.map(|extractor| (extractor.patch, o.owner)));
            match job(o) {
                Some((patch, GatherPhase::Mining { .. })) => gathering.mined.push(patch),
                Some((patch, GatherPhase::Waiting { .. })) if within_reach(objects, o, patch) => {
                    gathering.waiting.push((patch, o.id));
                }
                _ => {}
            }
            gathering.note_hatch(objects, o);
        }
        for station in picking {
            if let Ok(i) = (gathering.stations).binary_search_by_key(&station, |&(id, _)| id) {
                gathering.stations[i].1 = gathering.stations[i].1.saturating_sub(1);
            }
        }
        gathering.mined.sort_unstable();
        gathering.covered.sort_unstable();
        gathering.waiting.sort_unstable();
        gathering.unloading.sort_unstable();
        gathering.in_line.sort_unstable();
        gathering
    }

    /// Worker `o` unloads or waits at a hatch (`unloading`, `in_line`), on a Gather or a Drop Off order.
    fn note_hatch(&mut self, objects: &BTreeMap<ObjectId, Object>, o: &Object) {
        let state = o
            .worker
            .as_ref()
            .and(o.orders.as_ref())
            .and_then(|orders| orders.current.as_ref());
        let Some(Behaviour::Gather(gather)) = state.map(|active| &active.state) else {
            return;
        };
        let since = match gather.phase {
            GatherPhase::AtHatch { since } => Some(since),
            GatherPhase::DroppingOff { .. } => None,
            _ => return,
        };
        // The hatch of what it carries (slice M4-I).
        let resource = o.worker.map_or(Resource::Crystals, |worker| worker.resource);
        let point = gather
            .node
            .and_then(|node| Some((node, drop_point_of(objects.get(&node)?, resource)?)));
        let Some((node, point, side)) = point.and_then(|(node, point)| Some((node, point, point.side?))) else {
            return;
        };
        match since {
            None => self.unloading.push((node, side)),
            Some(since) => {
                let load = worker_def(o.kind).and_then(|def| def.load(resource));
                let reach = load.map_or(0.0, |load| load.drop_off_reach);
                if Shape::of(o).gap(&point.shape) <= reach {
                    self.in_line.push((node, side, since, o.id));
                }
            }
        }
    }
}

/// The patch and the phase of the Gather that `o` carries out, if it is a worker with one whose type takes part in
/// the patches' occupancy: not one that shares patches (`WorkerDef::shares_patches`, the Agent: "no occupancy
/// rule", design change 0255), which neither occupies a patch nor holds one by waiting at it.
fn job(o: &Object) -> Option<(ObjectId, GatherPhase)> {
    o.worker.as_ref()?;
    let active = o.orders.as_ref()?.current.as_ref()?;
    if worker_def(o.kind).is_some_and(|def| def.shares_patches) {
        return None;
    }
    match (active.order.kind, &active.state) {
        (OrderKind::Gather(patch), Behaviour::Gather(gather)) => Some((patch, gather.phase)),
        _ => None,
    }
}

/// The station at which `o` picks up a Supply, if it is a worker of a type that gathers them, picking one up there
/// now (`GatherPhase::Mining` on a Gather at a station): its pick-up claims one of the station's Supplies until it
/// ends (design change 0256 point 2). Only the types that gather Supplies are looked at.
fn pick_up(objects: &BTreeMap<ObjectId, Object>, o: &Object) -> Option<ObjectId> {
    o.worker.as_ref()?;
    worker_def(o.kind)?.supplies?;
    let active = o.orders.as_ref()?.current.as_ref()?;
    match (active.order.kind, &active.state) {
        (OrderKind::Gather(station), Behaviour::Gather(gather))
            if matches!(gather.phase, GatherPhase::Mining { .. }) && is_station(objects.get(&station)?.kind) =>
        {
            Some(station)
        }
        _ => None,
    }
}

/// Whether worker `o` is within its MiningReach of `patch` (`factions/colonists/units/drone.md > Gathering`).
fn within_reach(objects: &BTreeMap<ObjectId, Object>, o: &Object, patch: ObjectId) -> bool {
    let (Some(def), Some(patch)) = (worker_def(o.kind), objects.get(&patch)) else {
        return false;
    };
    Shape::of(o).gap(&Shape::of(patch)) <= def.crystals.pick_up_reach
}

impl World<'_> {
    /// The structures that take workers' loads, in id order, each with where it takes Space Crystals: what the nearest
    /// is measured to, whatever the load (`economy::drop_off::nearest`).
    pub(crate) fn drop_offs(&self) -> impl Iterator<Item = (&View, &DropPoint)> + '_ {
        (self.gathering.drop_offs.iter()).map(|(index, points)| (&self.views[*index as usize], &points.crystals))
    }

    /// Where drop-off `node` takes `resource`, if it takes loads now.
    pub(crate) fn drop_point(&self, node: ObjectId, resource: Resource) -> Option<&DropPoint> {
        let drop_offs = &self.gathering.drop_offs;
        let found = drop_offs.binary_search_by_key(&node, |(index, _)| self.views[*index as usize].id);
        found.ok().map(|i| drop_offs[i].1.of(resource))
    }

    /// The Supplies station `station` holds that no pick-up under way claimed as the phase began (design change 0256
    /// point 2): a pick-up may start there while this is more than the claims of the tick (`orders::gather`). 0 for
    /// anything but a station.
    pub(crate) fn unclaimed_supplies(&self, station: ObjectId) -> u32 {
        let stations = &self.gathering.stations;
        stations
            .binary_search_by_key(&station, |&(id, _)| id)
            .map_or(0, |i| stations[i].1)
    }

    /// Station `id` as `owner` knows it ([`KnownStation`], [`World::known`]): the station itself while the owner sees
    /// it, the owner's record of it while it doesn't. `None` if the owner knows of no station by that id.
    pub(crate) fn known_station(&self, owner: Option<PlayerId>, id: ObjectId) -> Option<KnownStation> {
        match self.known(owner, id) {
            Known::Seen(view) => is_station(view.kind).then_some(KnownStation {
                id,
                shape: view.shape,
                seen: true,
            }),
            Known::Remembered { .. } => {
                let record = self.vision.player(owner?)?.record(id)?;
                is_station(record.kind).then(|| KnownStation {
                    id,
                    shape: Shape::footprint(&record.placement),
                    seen: false,
                })
            }
            Known::Gone => None,
        }
    }

    /// `factions/syndicate/tunnels.md > Tunnel > SymmetryType - ABCD`: whether a worker was unloading at the hatch on
    /// `side` of `node` as the phase began.
    pub(crate) fn hatch_busy(&self, node: ObjectId, side: SideLetter) -> bool {
        self.gathering.unloading.binary_search(&(node, side)).is_ok()
    }

    /// Design change 0255 (`factions/syndicate/units/agent.md > Gathering rules > Waiting at the hatch`): the
    /// worker that unloads next at the hatch on `side` of `node`, of those that waited within reach of it as the
    /// phase began: the first to arrive, the lowest id of those that arrived in one tick.
    pub(crate) fn first_in_line(&self, node: ObjectId, side: SideLetter) -> Option<ObjectId> {
        let line = &self.gathering.in_line;
        let first = line.partition_point(|&(n, s, ..)| (n, s) < (node, side));
        let &(n, s, _, worker) = line.get(first)?;
        ((n, s) == (node, side)).then_some(worker)
    }

    /// `> Patch Occupancy (one Drone per patch)`: whether a worker, of any player, was mining `patch` as the
    /// phase began.
    pub(crate) fn patch_mined(&self, patch: ObjectId) -> bool {
        self.gathering.mined.binary_search(&patch).is_ok()
    }

    /// Whether a mining structure stands on `patch` (design change 0137: the patch counts as gone for workers).
    pub(crate) fn patch_covered(&self, patch: ObjectId) -> bool {
        self.covers(patch).next().is_some()
    }

    /// The owners of the mining structures on `patch`.
    fn covers(&self, patch: ObjectId) -> impl Iterator<Item = Option<PlayerId>> + '_ {
        let first = self.gathering.covered.partition_point(|&(p, _)| p < patch);
        self.gathering.covered[first..]
            .iter()
            .take_while(move |&&(p, _)| p == patch)
            .map(|&(_, owner)| owner)
    }

    /// Patch `id` as `owner` knows it (design change 0139, [`World::known`]): the patch itself while the owner sees
    /// it; while it doesn't, the patch as the owner's record has it, covered if the owner knows of a mining structure
    /// on it (`targets::remembered_gatherable`). `None` if the owner knows of no patch by that id: none it sees or
    /// remembers, or no patch.
    pub(crate) fn known_patch(&self, owner: Option<PlayerId>, id: ObjectId) -> Option<KnownPatch> {
        match self.known(owner, id) {
            Known::Seen(view) => view.is_patch.then(|| KnownPatch {
                id,
                pos: view.pos,
                shape: view.shape,
                seen: true,
                covered: self.patch_covered(id),
            }),
            Known::Remembered { .. } => {
                let record = self.vision.player(owner?)?.record(id)?;
                self.remembered_patch(owner, record)
            }
            Known::Gone => None,
        }
    }

    /// `record`, one of `owner`'s out of its sight, as a patch the owner knows ([`World::known_patch`]).
    fn remembered_patch(&self, owner: Option<PlayerId>, record: &LastKnown) -> Option<KnownPatch> {
        let sight = self.vision.player(owner?)?;
        let is_patch = matches!(record.resource, Some(ResourceState::CrystalPatch { .. }));
        // Its own plates the owner always knows of.
        let own_cover = self.covers(record.id).any(|by| by == owner);
        is_patch.then(|| KnownPatch {
            id: record.id,
            pos: record.pos(),
            shape: Shape::footprint(&record.placement),
            seen: false,
            covered: !targets::remembered_gatherable(sight, record, own_cover),
        })
    }

    /// `> Patch Occupancy > Search radius` (design changes 0131, 0139 and 0159): the patches `owner` knows whose
    /// centres lie within `radius` of `centre` (space units), in id order, as it knows them ([`World::known_patch`]):
    /// its records of patches, which take in those it sees, those it remembers whether or not they still stand, and
    /// the map's from the start.
    pub(crate) fn known_patches_near(
        &self,
        owner: Option<PlayerId>,
        centre: Vec2,
        radius: f32,
    ) -> impl Iterator<Item = KnownPatch> + '_ {
        let sight = owner.and_then(|owner| self.vision.player(owner));
        sight
            .into_iter()
            .flat_map(|sight| sight.records())
            .filter(move |r| {
                matches!(r.resource, Some(ResourceState::CrystalPatch { .. }))
                    && r.pos().distance_squared(centre) <= radius * radius
            })
            .filter_map(move |r| self.known_patch(owner, r.id))
    }

    /// The shapes of the structures and map resources `owner` knows to stand where they are (design change 0139):
    /// its own, those it sees, and those it remembers out of its sight, where it remembers them, standing or not.
    pub(crate) fn known_fixed(&self, owner: Option<PlayerId>) -> Vec<Shape> {
        let sight = owner.and_then(|owner| self.vision.player(owner));
        let mut known: Vec<Shape> = self
            .fixed()
            .filter(|view| (owner.is_some() && view.owner == owner) || self.sees(owner, view))
            .map(|view| view.shape)
            .collect();
        known.extend(
            (sight.into_iter())
                .flat_map(|sight| sight.remembered())
                .map(|record| Shape::footprint(&record.placement)),
        );
        known
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
        let grid = (def.unit.as_ref().zip(def.silhouette()))
            .and_then(|(walker, silhouette)| paths::unit_grid(self.nav.planning(unit.owner), walker, silhouette));
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
