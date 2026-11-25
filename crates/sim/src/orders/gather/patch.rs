//! The job at its patch (the module docs of `orders::gather`): the way there, waiting, mining, looking round for
//! another patch, and the Gather's end.

use crate::defs::WorkerDef;
use crate::ids::{ObjectId, PlayerId};
use crate::mover::MoveStatus;
use crate::units::Tick;

use super::super::OrderKind;
use super::super::state::{Gather, GatherPhase};
use super::super::world::{Unit, World};
use super::legs::leg;
use super::search::{claim, search, taken};
use super::site::{Site, site};
use super::{Claims, Flow, RECHECK_TICKS, loaded, returning};

pub(super) fn to_patch(
    kind: &mut OrderKind,
    state: &mut Gather,
    unit: &mut Unit,
    def: &WorkerDef,
    world: &World,
    claims: &mut Claims,
) -> Flow {
    let site = site(*kind, state, unit, world);
    let target = match site {
        Site::Patch(patch) => patch.shape,
        Site::Gone(shape) => shape,
        Site::Nowhere => return end(state, unit),
    };
    // `> Gathering`: MiningReach. A leg that ended short of the patch counts as arriving, out of reach (0141).
    let in_reach = unit.shape().gap(&target) <= def.crystals.pick_up_reach;
    let ended = state.moving && unit.mover.status != MoveStatus::Moving;
    if !in_reach && !ended {
        leg(state, unit, &target, None, world);
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
    // Design change 0139: a patch out of its owner's sight is neither mined nor taken for occupied, whatever stands
    // there. Within MiningReach the worker sees it, at the latest in the next tick's snapshot.
    if in_reach && !patch.seen {
        return Flow::Stay;
    }
    if in_reach && !taken(world, claims, def, patch.id, unit.id, false) {
        claim(claims, def, patch.id, unit);
        unit.halt();
        state.enter(GatherPhase::Mining { done: 0 });
        return Flow::Stay;
    }
    // `> Patch Occupancy`: "When a Drone arrives at an occupied patch, it goes to the nearest unoccupied patch to
    // itself instead" (0132, item 1; and item 3: one sent on that finds that patch taken too "starts over from
    // there"), and so it does at a patch it can't reach (0141). With none, it waits.
    if let Some(other) = search(world, claims, unit, def, Some(patch.id)).free {
        go_on(kind, state, (unit.id, unit.owner), other, !def.shares_patches, claims);
        return Flow::Next;
    }
    unit.halt();
    state.enter(GatherPhase::Waiting { since: world.tick });
    Flow::Stay
}

pub(super) fn wait(
    kind: &mut OrderKind,
    state: &mut Gather,
    unit: &mut Unit,
    def: &WorkerDef,
    since: Tick,
    world: &World,
    claims: &mut Claims,
) -> Flow {
    let patch = match site(*kind, state, unit, world) {
        Site::Patch(patch) => patch,
        // Design change 0134: "Drones already waiting at it look for another patch right away"; 0137, a plate.
        Site::Gone(_) => return find_another(kind, state, unit, def, world, claims),
        Site::Nowhere => return end(state, unit),
    };
    // 0132, item 1: it "mines it as soon as it frees up"; 0133: the lowest id of those waiting there. Not one out of
    // its owner's sight (design change 0139).
    let in_reach = unit.shape().gap(&patch.shape) <= def.crystals.pick_up_reach;
    if in_reach && patch.seen && !taken(world, claims, def, patch.id, unit.id, true) {
        claim(claims, def, patch.id, unit);
        state.enter(GatherPhase::Mining { done: 0 });
        return Flow::Stay;
    }
    // 0132, item 2: "it re-checks at intervals for a nearby patch that has freed up. If one has, it goes there."
    // All of them in the same ticks ([`RECHECK_TICKS`]); not in the tick it began to wait, when it just looked.
    if since == world.tick || !world.tick.is_multiple_of(RECHECK_TICKS) {
        return Flow::Stay;
    }
    if let Some(other) = search(world, claims, unit, def, Some(patch.id)).free {
        go_on(kind, state, (unit.id, unit.owner), other, !def.shares_patches, claims);
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
pub(super) fn mine(
    kind: &mut OrderKind,
    state: &mut Gather,
    unit: &mut Unit,
    def: &WorkerDef,
    world: &World,
    claims: &mut Claims,
) -> Flow {
    match site(*kind, state, unit, world) {
        Site::Patch(_) => Flow::Stay,
        Site::Gone(_) => find_another(kind, state, unit, def, world, claims),
        Site::Nowhere => end(state, unit),
    }
}

/// The worker is where its patch was, and the patch is gone for the workers (module docs: "a depleted patch is
/// treated like an occupied one, so the redirect rule applies", design change 0134). It goes to the nearest free
/// patch it can reach; with none free, to the nearest patch left, where it will wait (0123: "if only occupied
/// patches remain, the Drone keeps gathering (waits at one)"); with none at all, the Gather ends ([`end`]).
pub(super) fn find_another(
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
            let claim = found.free.is_some() && !def.shares_patches;
            go_on(kind, state, (unit.id, unit.owner), other, claim, claims);
            Flow::Next
        }
        None => end(state, unit),
    }
}

/// The order goes on at `patch` (`OrderKind::Gather`: it names the patch the worker heads for): at once, or after
/// the drop-off if the worker is on its way back. One that sets off for a `free` patch claims it for this tick
/// ([`Claims`]), as `me`, the worker and its owner; one of a type that shares patches claims none (`search::taken`).
pub(super) fn go_on(
    kind: &mut OrderKind,
    state: &mut Gather,
    me: (ObjectId, Option<PlayerId>),
    patch: ObjectId,
    free: bool,
    claims: &mut Claims,
) {
    *kind = OrderKind::Gather(patch);
    state.spot = None;
    if !returning(state) {
        state.enter(GatherPhase::ToPatch);
        if free {
            claims.claim(patch, me.0, me.1);
        }
    }
}

/// Design change 0123 (`> When a Gather ends`): no patch is left within the worker's search radius. "It first
/// delivers any load it carries, then becomes idle (or starts its next queued order)."
pub(super) fn end(state: &mut Gather, unit: &Unit) -> Flow {
    if !loaded(unit) {
        return Flow::Done;
    }
    state.ending = true;
    if !matches!(
        state.phase,
        GatherPhase::ToDropOff | GatherPhase::AtHatch { .. } | GatherPhase::DroppingOff { .. }
    ) {
        state.enter(GatherPhase::ToDropOff);
    }
    Flow::Next
}
