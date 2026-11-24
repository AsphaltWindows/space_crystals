//! A worker's job — `factions/colonists/units.md > Drone > Gathering`, `> Patch Occupancy (one Drone per patch)`,
//! `> Last load of a patch`, `> Interrupted mining`, `> Other orders keep the load`, `> Gather while carrying a
//! load`, `> Drop Off order`, `> When a Gather ends` and `> Assigned Drop-off` (slices M3-H and M3-H3, plan decision
//! T17; design changes 0123 and 0127 to 0141 answered question Q0037). `systems/controls/commands.md` and
//! `behaviors.md` define no Gather or Drop Off, so the Drone's own sections are the spec.
//!
//! One behaviour runs both orders: `OrderKind::Gather(patch)` and `OrderKind::DropOff(node)`. Its phases
//! (`state::GatherPhase`):
//! - **ToPatch**: a leg to the order's patch. On arrival the worker **claims** the patch if it is free and starts
//!   Mining. At a patch it can't mine, occupied or out of reach, it looks round (below) and goes to another, or
//!   waits. A worker that carries a load goes to the patch all the same, and on arrival "skips mining (it doesn't
//!   occupy the patch) and returns to deliver as if it had just mined the load there" (design change 0128).
//! - **Waiting**: "right at the patch it tried to mine, within MiningReach", or as close as it got to a patch it
//!   can't reach (design changes 0132 and 0141). It mines the patch as soon as it frees, and looks round again every
//!   [`RECHECK_TICKS`]: it goes to a patch that has freed up meanwhile, and one that couldn't get to its patch tries
//!   again ("carrying on as soon as the way clears"). It never gives up for good.
//! - **Mining** and **DroppingOff**: the timed work, which the economy phase counts (`economy::gathering::step`).
//!   Mining that stops before its 32 frames are done, whatever stops it, collects nothing, and the patch frees at
//!   once (design change 0130): the frames are the order's state, and go with it.
//! - **ToDropOff**: a leg to its AssignedDropOff, read again every tick, so that Set Drop-off Location redirects
//!   "a trip already in progress" (`> Assigned Drop-off`); for a Drop Off order, to the order's own drop-off.
//!   Without a drop-off as the return starts, the worker assigns the nearest of its owner's, and with none it idles
//!   holding its load (design change 0135).
//! - **Delivered**: the load is in the bank. A Gather goes back to its patch, the one where it last mined (design
//!   change 0132, item 4: the order follows the worker from patch to patch). A Drop Off order is complete: "an
//!   explicit Drop Off is only a delivery" (0127); the gate never gives one to a worker without a load (0140).
//!
//! **Occupancy** (`> Patch Occupancy`; design change 0133 and the Director's choice of rule): a patch is taken
//! while a worker of any player mines it (as phase 5a began, `World::patch_mined`), once a worker with a lower id
//! has claimed it in this tick ([`Claims`]), and, for every other worker, while a worker waits at it within
//! MiningReach (`World::waited_at`), which mines it as soon as it frees. Of several workers waiting at a patch the
//! lowest id gets it: the behaviours run in id order, and the first to see the patch free claims it. A worker on
//! its way doesn't occupy anything, but one that sets off for a free patch it found claims it for that tick, so
//! that no other worker's look in the same tick sends it there too.
//!
//! **Looking round** (`search::search`, design changes 0123, 0131 and 0141): within the worker's search radius
//! (`WorkerDef::search_radius`) around itself, among the patches its owner knows that no plate covers, the nearest
//! free one it can reach.
//!
//! **A patch that is gone** — depleted (design change 0134), or under a plate (0137: "the patch then counts as
//! gone for the Drones, exactly like a patch that runs out") — is treated like an occupied one. A worker that was
//! there, waiting or mining it (which collects nothing) or carrying away the load that emptied it, looks round at
//! once. One on its way to it keeps going to where it was (`state::Gather::spot`), and one carrying a load back
//! goes there after its drop-off, and looks round there: "the search radius is centred on the field". With no free
//! patch it goes to the nearest patch left and waits there (0123: "if only occupied patches remain, the Drone keeps
//! gathering"); with none at all, the Gather ends.
//!
//! **When a Gather ends** (design change 0123): only then. The worker "first delivers any load it carries"
//! (`state::Gather::ending`), then the order is complete: the next queued order starts, or the worker idles. Orders
//! Shift-queued behind a Gather wait for that, a Deploy as much as any.
//!
//! **Arriving.** MiningReach and DropOffReach are gaps between the worker's Silhouette and the target's edge. The
//! behaviour measures them itself, each tick, on the snapshot's positions, and stops the worker when it is within
//! reach.
//!
//! **Legs are moves to a point, not to the object** (a deviation from the plan's sketch, which named
//! `MoveGoal::Object`): the point nearest to the worker where it stands against a face of the target
//! (`legs::approach_point`). A move to an object is planned to the point nearest the object's **centre** that the
//! unit can stand on, ties going to the lowest tile: always the north side of a patch in a row and of an Egress
//! Node, wherever the worker comes from. A Drone south of a row of patches would walk round the row's end to mine
//! from the north. The point nearest to the worker keeps trips straight and spreads workers along a face.
//!
//! **A leg to a patch that ends short of it** (the mover arrived as near as it gets behind units that won't move,
//! `systems/controls/behaviors.md > MovingToLocation > Occupied target location`, or gave up) counts as arriving
//! there, out of reach: "a patch the Drone can't get within MiningReach of (path blocked by units, structures or
//! terrain) is treated like an occupied patch" (design change 0141). A leg back to the drop-off that ends short is
//! begun again after [`LEG_RETRY_TICKS`], a back-off: what stopped it is a unit that won't move, and that is rarely
//! gone a tick later. A worker that set off again at once would ask for a path every dozen ticks and jitter against
//! its blocker; standing, it is a unit at rest, which the units behind it route around at once.
//!
//! **Collision** (design change 0106, `factions/colonists/units.md > Drone > Gathering Drones pass through each
//! other`): Drones on a Gather job don't collide with each other, whoever owns them ([`on_gather_job`]; the
//! exception itself is `movement::space::blocks`). They collide with everything else, and having an order they
//! never yield (`systems/units.md > UnitCollision > Ground Collision > Who yields`). So Drones wait, mine and
//! deliver inside each other, and the patch's occupancy is the only limit. A Drone whose job ends while it is
//! inside another follows `> Units that already overlap`: the two ignore each other until they are apart.

mod legs;
mod search;

use glam::Vec2;

use crate::defs::WorkerDef;
use crate::economy::gathering::is_drop_off;
use crate::ids::{ObjectId, PlayerId};
use crate::mover::MoveStatus;
use crate::object::Object;
use crate::shape::Shape;
use crate::units::Tick;

use super::OrderKind;
use super::state::{Behaviour, Gather, GatherPhase};
use super::world::{Unit, View, World};
use legs::{leg, return_leg, spot_shape};
use search::{search, taken};

/// `> Patch Occupancy > Waiting and re-checking`, item 2 (design change 0132): a waiting worker looks round again
/// this many ticks after it began to wait, and every as many after that. The design's placeholder: 16 frames, 1
/// second.
pub const RECHECK_TICKS: Tick = 16;

/// A worker whose leg back to its drop-off ended short of it stands for this many ticks before it tries again
/// (module docs). A placeholder, unspecified (the design has no rule for a worker that can't get to its drop-off;
/// a patch it can't get to is design change 0141's): 2 seconds.
pub const LEG_RETRY_TICKS: Tick = 32;

/// `factions/colonists/units.md > Drone > Gathering Drones pass through each other` (design change 0106): whether
/// `o` is a Drone **on a Gather job**: a worker of a type that passes through its like (`WorkerDef::pass_through`,
/// "Drones only") whose current order is a Gather or a Drop Off. That is "every step of the jobs in the table":
/// travelling to the pick-up point, mining, waiting at an occupied patch, carrying the load and delivering it.
/// An order still queued doesn't count, and neither does a load carried without an order.
pub(crate) fn on_gather_job(o: &Object) -> bool {
    let passes = || {
        let worker = o.kind.def().unit.as_ref().and_then(|unit| unit.worker);
        worker.is_some_and(|worker| worker.pass_through)
    };
    let current = o.orders.as_ref().and_then(|orders| orders.current.as_ref());
    o.worker.is_some() && current.is_some_and(|active| matches!(active.state, Behaviour::Gather(_))) && passes()
}

/// `> Patch Occupancy (one Drone per patch)`: the patches claimed so far in this tick's phase 5a, each with the
/// worker that claimed it: one that starts mining it, or one that sets off for it, a free patch it found (design
/// change 0133: "the lowest-id Drone whose check sees the patch free claims it"). Scratch of one phase, never
/// state.
#[derive(Debug, Default)]
pub(crate) struct Claims(Vec<(ObjectId, ObjectId)>);

impl Claims {
    /// Whether a worker other than `me` claimed `patch`.
    fn by_other(&self, patch: ObjectId, me: ObjectId) -> bool {
        self.0.iter().any(|&(p, worker)| p == patch && worker != me)
    }

    fn claim(&mut self, patch: ObjectId, worker: ObjectId) {
        self.0.push((patch, worker));
    }
}

/// The state a Gather or a Drop Off order starts with, for `unit`, a worker (`orders::start`).
/// - `> Assigned Drop-off > Auto-assignment`: "When a Drone with no AssignedDropOff is ordered to Gather, it
///   automatically assigns the nearest Egress Node".
/// - `> Gather while carrying a load` (design change 0128): a worker that carries a load "goes straight to the new
///   pick-up point" all the same.
/// - `> Drop Off order` (design change 0127): straight to the order's drop-off.
pub(super) fn begin(kind: OrderKind, unit: &mut Unit, world: &World) -> Gather {
    let (pos, owner) = (unit.pos, unit.owner);
    let Some(worker) = unit.worker.as_deref_mut() else {
        return Gather::new(GatherPhase::ToPatch);
    };
    match kind {
        OrderKind::Gather(_) => {
            if worker.drop_off.is_none() {
                worker.drop_off = nearest_drop_off(world, pos, owner).map(|node| node.id);
            }
            Gather::new(GatherPhase::ToPatch)
        }
        _ => Gather::new(GatherPhase::ToDropOff),
    }
}

/// The drop-off of `owner` nearest to `pos`, centre to centre; the lowest id among equals.
fn nearest_drop_off<'a>(world: &'a World, pos: Vec2, owner: Option<PlayerId>) -> Option<&'a View> {
    owner?;
    world
        .drop_offs()
        .filter(|node| node.owner == owner)
        .min_by(|a, b| a.pos.distance_squared(pos).total_cmp(&b.pos.distance_squared(pos)))
}

/// What a phase's tick leaves.
enum Flow {
    /// The order goes on in the next tick.
    Stay,
    /// The phase changed: the new one runs in this tick too.
    Next,
    /// The order is complete.
    Done,
}

/// One tick of a Gather or a Drop Off order (module docs). Returns whether it is complete.
pub(super) fn run(
    kind: &mut OrderKind,
    state: &mut Gather,
    unit: &mut Unit,
    world: &World,
    claims: &mut Claims,
) -> bool {
    let Some(def) = unit
        .unit_def()
        .and_then(|unit| unit.worker)
        .filter(|_| unit.worker.is_some())
    else {
        return true;
    };
    // A phase that ends starts the next one in the same tick. A worker goes through a few at most: it looks round,
    // finds a patch beside it and claims it, say. None of the paths comes back to where it began.
    for _ in 0..6 {
        let flow = match state.phase {
            GatherPhase::ToPatch => to_patch(kind, state, unit, &def, world, claims),
            GatherPhase::Waiting { since } => wait(kind, state, unit, &def, since, world, claims),
            GatherPhase::Mining { .. } => mine(kind, state, unit, &def, world, claims),
            GatherPhase::ToDropOff => to_drop_off(kind, state, unit, &def, world, claims),
            GatherPhase::Delivered => delivered(*kind, state),
            // The economy phase counts the frames and moves the job on.
            GatherPhase::DroppingOff { .. } => Flow::Stay,
        };
        match flow {
            Flow::Stay => return false,
            Flow::Next => {}
            Flow::Done => {
                unit.halt();
                return true;
            }
        }
    }
    false
}

/// The order's patch, as the snapshot has it.
enum Site<'w> {
    /// A patch workers can gather.
    Patch(&'w View),
    /// The shape of a patch that is gone for the workers: one that left the game, where it was (design change
    /// 0134, `state::Gather::spot`), or one under a plate (0137).
    Gone(Shape),
    /// Nothing to head for: the order names no patch (a Drop Off, or a save edited by hand).
    Nowhere,
}

fn site<'w>(kind: OrderKind, state: &Gather, world: &'w World) -> Site<'w> {
    let OrderKind::Gather(patch) = kind else {
        return Site::Nowhere;
    };
    match world.get(patch) {
        Some(view) if view.is_patch && !world.patch_covered(patch) => Site::Patch(view),
        Some(view) if view.is_patch => Site::Gone(view.shape),
        Some(_) => Site::Nowhere,
        None => state.spot.map_or(Site::Nowhere, |spot| Site::Gone(spot_shape(spot))),
    }
}

/// Whether the worker carries anything.
fn loaded(unit: &Unit) -> bool {
    unit.worker.as_ref().is_some_and(|worker| worker.load > 0)
}

/// Whether the job is on its way back: from the patch with its load, at the drop-off, or just done there.
fn returning(state: &Gather) -> bool {
    matches!(
        state.phase,
        GatherPhase::ToDropOff | GatherPhase::DroppingOff { .. } | GatherPhase::Delivered
    )
}

fn to_patch(
    kind: &mut OrderKind,
    state: &mut Gather,
    unit: &mut Unit,
    def: &WorkerDef,
    world: &World,
    claims: &mut Claims,
) -> Flow {
    let site = site(*kind, state, world);
    let target = match site {
        Site::Patch(patch) => patch.shape,
        Site::Gone(shape) => shape,
        Site::Nowhere => return end(state, unit),
    };
    // `> Gathering`: MiningReach. A leg that ended short of the patch counts as arriving, out of reach (0141).
    let in_reach = unit.shape().gap(&target) <= def.mining_reach;
    let ended = state.moving && unit.mover.status != MoveStatus::Moving;
    if !in_reach && !ended {
        leg(state, unit, &target, world);
        return Flow::Stay;
    }
    let Site::Patch(patch) = site else {
        // Design change 0134: where the patch was, it looks round.
        return find_another(kind, state, unit, def, world, claims);
    };
    // Design change 0128: loaded, it turns round without mining, and claims nothing. (A leg that ended short of
    // the patch is an arrival too: the patch it can't reach is like an occupied one, 0141, and an occupied one
    // doesn't keep it.)
    if loaded(unit) {
        state.enter(GatherPhase::ToDropOff);
        return Flow::Next;
    }
    if in_reach && !taken(world, claims, patch.id, unit.id, false) {
        claims.claim(patch.id, unit.id);
        unit.halt();
        state.enter(GatherPhase::Mining { done: 0 });
        return Flow::Stay;
    }
    // `> Patch Occupancy`: "When a Drone arrives at an occupied patch, it goes to the nearest unoccupied patch to
    // itself instead" (0132, item 1; and item 3: one sent on that finds that patch taken too "starts over from
    // there"), and so it does at a patch it can't reach (0141). With none, it waits.
    if let Some(other) = search(world, claims, unit, def, Some(patch.id)).free {
        go_on(kind, state, unit.id, other, true, claims);
        return Flow::Next;
    }
    unit.halt();
    state.enter(GatherPhase::Waiting { since: world.tick });
    Flow::Stay
}

fn wait(
    kind: &mut OrderKind,
    state: &mut Gather,
    unit: &mut Unit,
    def: &WorkerDef,
    since: Tick,
    world: &World,
    claims: &mut Claims,
) -> Flow {
    let patch = match site(*kind, state, world) {
        Site::Patch(patch) => patch,
        // Design change 0134: "Drones already waiting at it look for another patch right away"; 0137, a plate.
        Site::Gone(_) => return find_another(kind, state, unit, def, world, claims),
        Site::Nowhere => return end(state, unit),
    };
    // 0132, item 1: it "mines it as soon as it frees up"; 0133: the lowest id of those waiting there.
    let in_reach = unit.shape().gap(&patch.shape) <= def.mining_reach;
    if in_reach && !taken(world, claims, patch.id, unit.id, true) {
        claims.claim(patch.id, unit.id);
        state.enter(GatherPhase::Mining { done: 0 });
        return Flow::Stay;
    }
    // 0132, item 2: "it re-checks at intervals for a nearby patch that has freed up. If one has, it goes there."
    let waited = world.tick.saturating_sub(since);
    if waited == 0 || !waited.is_multiple_of(RECHECK_TICKS) {
        return Flow::Stay;
    }
    if let Some(other) = search(world, claims, unit, def, Some(patch.id)).free {
        go_on(kind, state, unit.id, other, true, claims);
        return Flow::Next;
    }
    // 0141: short of a patch it couldn't get to, it tries again, "carrying on as soon as the way clears". Not while
    // its owner's grids say there is no way: nothing but a change to them can open one.
    if !in_reach && world.reachable(unit, &patch.shape) {
        state.enter(GatherPhase::ToPatch);
        return Flow::Next;
    }
    Flow::Stay
}

/// Mining, whose frames the economy phase counts. A patch that is gone for the workers stops it, and the worker
/// looks round at once (design change 0137: "a Drone in the middle of mining it is interrupted and collects
/// nothing").
fn mine(
    kind: &mut OrderKind,
    state: &mut Gather,
    unit: &mut Unit,
    def: &WorkerDef,
    world: &World,
    claims: &mut Claims,
) -> Flow {
    match site(*kind, state, world) {
        Site::Patch(_) => Flow::Stay,
        Site::Gone(_) => find_another(kind, state, unit, def, world, claims),
        Site::Nowhere => end(state, unit),
    }
}

fn to_drop_off(
    kind: &mut OrderKind,
    state: &mut Gather,
    unit: &mut Unit,
    def: &WorkerDef,
    world: &World,
    claims: &mut Claims,
) -> Flow {
    // Design change 0134: a worker that is where its patch was when the patch goes (the one that carries off its
    // last load) looks round at once. One farther off goes on, and looks round there after its drop-off.
    if !state.ending
        && let Site::Gone(spot) = site(*kind, state, world)
        && unit.shape().gap(&spot) <= def.mining_reach
    {
        return find_another(kind, state, unit, def, world, claims);
    }
    let (pos, owner) = (unit.pos, unit.owner);
    let Some(worker) = unit.worker.as_deref_mut() else {
        return Flow::Done;
    };
    if worker.load == 0 {
        state.enter(GatherPhase::Delivered);
        return Flow::Next;
    }
    let own_drop_off = |id: ObjectId| {
        world
            .get(id)
            .filter(|node| is_drop_off(node.kind) && owner.is_some() && node.owner == owner)
    };
    let node = match *kind {
        OrderKind::DropOff(node) => own_drop_off(node),
        // `> Assigned Drop-off`: "the return trip's destination is read from AssignedDropOff". Design change 0135
        // (`> Drop-off gone`): without one as the trip starts, the nearest own Egress Node becomes it; with none, the
        // worker idles holding its load. (It is read every tick, so one lost on the way counts as a trip starting
        // there.)
        _ => worker.drop_off.and_then(own_drop_off).or_else(|| {
            let nearest = nearest_drop_off(world, pos, owner);
            worker.drop_off = nearest.map(|node| node.id);
            nearest
        }),
    };
    let Some(node) = node else {
        return Flow::Done;
    };
    if state.node != Some(node.id) {
        state.node = Some(node.id);
        state.moving = false;
        state.retry_at = 0;
    }
    // `> Gathering`: DropOffReach.
    if unit.shape().gap(&node.shape) <= def.drop_off_reach {
        unit.halt();
        state.enter(GatherPhase::DroppingOff { done: 0 });
        return Flow::Stay;
    }
    return_leg(state, unit, &node.shape, world);
    Flow::Stay
}

/// The load is in the bank.
fn delivered(kind: OrderKind, state: &mut Gather) -> Flow {
    state.node = None;
    match kind {
        // 0132, item 4: back to "the patch where it last successfully mined", which the order names; or to where
        // it was, if it is gone (0134).
        OrderKind::Gather(_) if !state.ending => {
            state.enter(GatherPhase::ToPatch);
            Flow::Next
        }
        // 0123: the Gather ran out of patches. 0127: "the Drone delivers its load and the order is complete".
        _ => Flow::Done,
    }
}

/// The worker is where its patch was, and the patch is gone for the workers (module docs: "a depleted patch is
/// treated like an occupied one, so the redirect rule applies", design change 0134). It goes to the nearest free
/// patch it can reach; with none free, to the nearest patch left, where it will wait (0123: "if only occupied
/// patches remain, the Drone keeps gathering (waits at one)"); with none at all, the Gather ends ([`end`]).
fn find_another(
    kind: &mut OrderKind,
    state: &mut Gather,
    unit: &mut Unit,
    def: &WorkerDef,
    world: &World,
    claims: &mut Claims,
) -> Flow {
    let gone = match *kind {
        OrderKind::Gather(patch) => Some(patch),
        _ => None,
    };
    let found = search(world, claims, unit, def, gone);
    match found.free.or(found.any) {
        Some(other) => {
            go_on(kind, state, unit.id, other, found.free.is_some(), claims);
            Flow::Next
        }
        None => end(state, unit),
    }
}

/// The order goes on at `patch` (`OrderKind::Gather`: it names the patch the worker heads for): at once, or after
/// the drop-off if the worker is on its way back. One that sets off for a `free` patch claims it for this tick
/// ([`Claims`]).
fn go_on(kind: &mut OrderKind, state: &mut Gather, me: ObjectId, patch: ObjectId, free: bool, claims: &mut Claims) {
    *kind = OrderKind::Gather(patch);
    state.spot = None;
    if !returning(state) {
        state.enter(GatherPhase::ToPatch);
        if free {
            claims.claim(patch, me);
        }
    }
}

/// Design change 0123 (`> When a Gather ends`): no patch is left within the worker's search radius. "It first
/// delivers any load it carries, then becomes idle (or starts its next queued order)."
fn end(state: &mut Gather, unit: &Unit) -> Flow {
    if !loaded(unit) {
        return Flow::Done;
    }
    state.ending = true;
    if !matches!(state.phase, GatherPhase::ToDropOff | GatherPhase::DroppingOff { .. }) {
        state.enter(GatherPhase::ToDropOff);
    }
    Flow::Next
}

#[cfg(test)]
mod tests;
