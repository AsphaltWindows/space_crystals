//! A worker's job — `factions/colonists/units.md > Drone > Gathering`, `> Patch Occupancy (one Drone per patch)`,
//! `> Assigned Drop-off` (slice M3-H, plan decision T17). `systems/controls/commands.md` and `behaviors.md` define
//! no Gather or Drop Off, so the Drone's own sections are the spec, and question Q0037 covers what they leave open:
//! each of its proposals built here is marked with its point.
//!
//! One behaviour runs both orders: `OrderKind::Gather(patch)` and `OrderKind::DropOff(node)`. Its phases
//! (`state::GatherPhase`):
//! - **ToPatch**: a leg to the order's patch. On arrival the worker **claims** the patch if it is free and starts
//!   Mining. `> Patch Occupancy`: a patch is taken while a worker of any player mines it (as phase 5a began,
//!   `World::patch_mined`), when a worker with a lower id claimed it earlier in this tick ([`Claims`]), or while
//!   another worker is next in line for it (Q0037 point 6). At a taken patch the worker "goes to the nearest
//!   unoccupied patch to itself instead" ([`free_patch`], Q0037 point 5), or waits.
//! - **Waiting**: where it stands. It mines "as soon as it frees up": the first to wait gets the patch, ties by id
//!   (Q0037 point 6).
//! - **Mining** and **DroppingOff**: the timed work, which the economy phase counts (`economy::gathering::step`).
//! - **ToDropOff**: a leg to its AssignedDropOff, read again every tick, so that Set Drop-off Location redirects
//!   "a trip already in progress" (`> Assigned Drop-off`); for a Drop Off order, to the order's own drop-off.
//!   Without a drop-off the worker assigns the nearest of its owner's as it starts its return, and with none it
//!   idles holding its load (Q0037 point 8).
//! - **Delivered**: the load is in the bank. A Gather goes back to its patch and never ends on its own, unless
//!   orders are queued behind it, which start now (Q0037 point 12). A Drop Off order goes back to the patch the
//!   worker was mining, as a Gather, or completes if it has none (Q0037 point 1).
//!
//! **Arriving.** MiningReach and DropOffReach are gaps between the worker's Silhouette and the target's edge. The
//! behaviour measures them itself, each tick, on the snapshot's positions, and stops the worker when it is within
//! reach.
//!
//! **Legs are moves to a point, not to the object** (a deviation from the plan's sketch, which named
//! `MoveGoal::Object`): the point nearest to the worker where it stands against a face of the target
//! ([`approach_point`]). A move to an object is planned to the point nearest the object's **centre** that the unit
//! can stand on, ties going to the lowest tile: always the north side of a patch in a row and of an Egress Node,
//! wherever the worker comes from. A Drone south of a row of patches would walk round the row's end to mine from
//! the north. The point nearest to the worker keeps trips straight and spreads workers along a face.
//!
//! **A leg that ends short of its target** (the mover arrived as near as it gets behind units that won't move,
//! `systems/controls/behaviors.md > MovingToLocation > Occupied target location`, or gave up): at a taken patch
//! that counts as arriving there, and the occupancy rule applies. Otherwise the worker stands for
//! [`LEG_RETRY_TICKS`] and tries again from where it is. A Gather never fails: it has no end of its own to fail to
//! (plan decision T17). **It stands in between** because a worker that set off again at once would be "on its
//! way" for good, and the units behind a unit that is on its way wait for it instead of looking for a way round
//! (`movement::stuck`: they take over its progress clock, which every new leg starts afresh). Standing, it is a
//! unit without a goal, which they route around.
//!
//! **Collision** is as the design reads today (question Q0038 point 1, open and blocking): workers collide with
//! each other like all ground units, and a worker on a job has an active command, so it never yields
//! (`systems/units.md > UnitCollision > Ground Collision > Who yields`), whether it mines, waits or travels.
//! **Crowds gridlock under these rules**: from about one and a half Drones a patch on a row of patches, a Drone
//! that has mined can't leave for the Drones waiting behind and beside it, and none of them gives way
//! (`tests/gathering_soak.rs` has the measurements and the smallest jam). Nothing here works around it: that
//! waits for Q0038's answer.

use glam::Vec2;

use crate::defs::WorkerDef;
use crate::economy::Worker;
use crate::economy::gathering::is_drop_off;
use crate::ids::{ObjectId, PlayerId};
use crate::mover::{MoveGoal, MoveStatus};
use crate::units::{Tick, gu};

use super::OrderKind;
use super::state::{Gather, GatherPhase};
use super::world::{Unit, View, World};

/// Placeholder: Q0037 point 5. A worker at a taken patch goes to the nearest free patch its owner knows within
/// this many grid units of the patch it arrived at, centre to centre; farther ones it leaves alone and waits.
pub const REDIRECT_RADIUS_GU: f32 = 6.0;

/// A worker whose leg ended short of its target stands for this many ticks before it tries again (module docs).
/// A placeholder, unspecified (the design has no rule for a worker that can't get to its patch or its drop-off): 2
/// seconds, longer than the 24 ticks after which the units behind it count it as in their way for good.
pub const LEG_RETRY_TICKS: Tick = 32;

/// `> Patch Occupancy (one Drone per patch)`: the patches claimed so far in this tick's phase 5a, by the workers
/// with lower ids. Scratch of one phase, never state.
#[derive(Debug, Default)]
pub(crate) struct Claims(Vec<ObjectId>);

impl Claims {
    fn has(&self, patch: ObjectId) -> bool {
        self.0.contains(&patch)
    }
}

/// The state a Gather or a Drop Off order starts with, for `unit`, a worker (`orders::start`).
/// - `> Assigned Drop-off > Auto-assignment`: "When a Drone with no AssignedDropOff is ordered to Gather, it
///   automatically assigns the nearest Egress Node".
/// - Q0037 point 2: a worker ordered to Gather while it carries a load delivers that first.
pub(super) fn begin(kind: OrderKind, unit: &mut Unit, world: &World) -> Gather {
    let (pos, owner) = (unit.pos, unit.owner);
    let Some(worker) = unit.worker.as_deref_mut() else {
        return Gather::new(GatherPhase::ToPatch);
    };
    match kind {
        OrderKind::Gather(patch) => {
            take_up(worker, patch, pos, owner, world);
            Gather::new(if worker.load > 0 {
                GatherPhase::ToDropOff
            } else {
                GatherPhase::ToPatch
            })
        }
        _ => Gather::new(GatherPhase::ToDropOff),
    }
}

/// `worker` takes up gathering at `patch`, with a drop-off if its owner has one (`> Auto-assignment`).
fn take_up(worker: &mut Worker, patch: ObjectId, pos: Vec2, owner: Option<PlayerId>, world: &World) {
    worker.patch = Some(patch);
    if worker.drop_off.is_none() {
        worker.drop_off = nearest_drop_off(world, pos, owner).map(|node| node.id);
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

/// `> Patch Occupancy (one Drone per patch)`: "the nearest unoccupied patch to itself", for a worker of `owner` at
/// `pos` that can't use the patch at `around`. Placeholder: Q0037 point 5 — among the patches its owner knows
/// (seen or remembered), not under a structure (point 11), within [`REDIRECT_RADIUS_GU`] of `around`. Unoccupied:
/// nobody mines it, nobody claimed it this tick and nobody waits for it. Nearest centre to centre, the lowest id
/// among equals; never one of `except`.
pub(crate) fn free_patch(
    world: &World,
    claims: &Claims,
    pos: Vec2,
    owner: Option<PlayerId>,
    around: Vec2,
    except: &[ObjectId],
) -> Option<ObjectId> {
    let radius = gu(REDIRECT_RADIUS_GU);
    world
        .patches()
        .filter(|patch| {
            patch.pos.distance_squared(around) <= radius * radius
                && !except.contains(&patch.id)
                && !taken(world, claims, patch.id, None)
                && !world.patch_covered(patch.id)
                && world.knows(owner, patch)
        })
        .min_by(|a, b| a.pos.distance_squared(pos).total_cmp(&b.pos.distance_squared(pos)))
        .map(|patch| patch.id)
}

/// `> Patch Occupancy (one Drone per patch)`: whether `patch` is taken for the worker `me` (module docs).
fn taken(world: &World, claims: &Claims, patch: ObjectId, me: Option<ObjectId>) -> bool {
    world.patch_mined(patch) || claims.has(patch) || world.first_waiter(patch).is_some_and(|first| Some(first) != me)
}

/// The order's patch changes to `to`, or is lost for good (`None`): depleted, or covered by a plate, with no other
/// in reach (Q0037 point 7: "treated as occupied, so the redirect rule applies. With no patch left, the Drone
/// delivers any load and idles"). Returns whether the order goes on: as a Gather at the new patch, or as a Drop
/// Off of the load the worker carries.
pub(crate) fn retarget(kind: &mut OrderKind, state: &mut Gather, worker: &mut Worker, to: Option<ObjectId>) -> bool {
    worker.patch = to;
    let returning = matches!(
        state.phase,
        GatherPhase::ToDropOff | GatherPhase::DroppingOff { .. } | GatherPhase::Delivered
    );
    match to {
        Some(patch) => {
            *kind = OrderKind::Gather(patch);
            if !returning {
                state.enter(GatherPhase::ToPatch);
            }
            true
        }
        None => match state.node.or(worker.drop_off) {
            Some(node) if returning && worker.load > 0 => {
                *kind = OrderKind::DropOff(node);
                true
            }
            _ => false,
        },
    }
}

/// The point a leg to `target` heads for (module docs): the point nearest to the unit where its Silhouette stands
/// against one of the target's four faces, wholly within that face's span, so that a worker stands in front of its
/// own patch and not half in front of the next one in a row. A face is left out if the worker couldn't stand
/// there for a structure or a map resource (the faces between the patches of a row). With no face left, the point
/// nearest to the unit where it touches the target at all; the mover takes it from there (the nearest point it can
/// reach, design change 0025).
fn approach_point(unit: &Unit, target: &View, world: &World) -> Vec2 {
    let half = unit.kind.def().half_extents().unwrap_or(Vec2::ZERO);
    let (lo, hi) = target.shape.bounds();
    // Where the unit's centre may be along a face: the face's span less the unit's half width, or its middle.
    let along = |from: f32, to: f32, half: f32, at: f32| {
        if to - from >= 2.0 * half {
            at.clamp(from + half, to - half)
        } else {
            (from + to) / 2.0
        }
    };
    let (x, y) = (
        along(lo.x, hi.x, half.x, unit.pos.x),
        along(lo.y, hi.y, half.y, unit.pos.y),
    );
    // North, south, west, east.
    let faces = [
        Vec2::new(x, lo.y - half.y),
        Vec2::new(x, hi.y + half.y),
        Vec2::new(lo.x - half.x, y),
        Vec2::new(hi.x + half.x, y),
    ];
    let overlaps = |p: Vec2, other: &View| {
        let (other_lo, other_hi) = other.shape.bounds();
        p.x - half.x < other_hi.x && other_lo.x < p.x + half.x && p.y - half.y < other_hi.y && other_lo.y < p.y + half.y
    };
    faces
        .into_iter()
        .filter(|&p| !world.fixed().any(|other| overlaps(p, other)))
        .min_by(|a, b| a.distance_squared(unit.pos).total_cmp(&b.distance_squared(unit.pos)))
        .unwrap_or_else(|| unit.pos.clamp(lo - half, hi + half))
}

/// The phase's leg to `target`: it begins, or begins again after ending short (module docs).
fn leg(state: &mut Gather, unit: &mut Unit, target: &View, world: &World) {
    let tick = world.tick;
    if state.moving && unit.mover.status != MoveStatus::Moving {
        state.moving = false;
        state.retry_at = tick.saturating_add(LEG_RETRY_TICKS);
    }
    if !state.moving && tick >= state.retry_at {
        let to = approach_point(unit, target, world);
        unit.mover.go_to(MoveGoal::Point(to), None, tick);
        state.moving = true;
    }
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

/// One tick of a Gather or a Drop Off order (module docs). `queued`: orders wait behind it. Returns whether it is
/// complete.
pub(super) fn run(
    kind: &mut OrderKind,
    state: &mut Gather,
    unit: &mut Unit,
    queued: bool,
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
    // A phase that ends starts the next one in the same tick; none of them comes round twice in a tick but
    // ToPatch, once, after a redirect.
    for _ in 0..4 {
        let flow = match state.phase {
            GatherPhase::ToPatch => to_patch(kind, state, unit, &def, world, claims),
            GatherPhase::Waiting { .. } => wait(kind, state, unit, &def, world, claims),
            GatherPhase::ToDropOff => to_drop_off(kind, state, unit, &def, world),
            GatherPhase::Delivered => delivered(kind, state, unit, queued, world),
            // The economy phase counts the frames and moves the job on.
            GatherPhase::Mining { .. } | GatherPhase::DroppingOff { .. } => Flow::Stay,
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

/// The order's patch can't be gathered any more: the worker goes to the nearest free one, or the order ends
/// ([`retarget`]).
fn lose_patch(kind: &mut OrderKind, state: &mut Gather, unit: &mut Unit, world: &World, claims: &Claims) -> Flow {
    let OrderKind::Gather(patch) = *kind else {
        return Flow::Done;
    };
    let around = world.get(patch).map_or(unit.pos, |view| view.pos);
    let to = free_patch(world, claims, unit.pos, unit.owner, around, &[patch]);
    let goes_on = unit
        .worker
        .as_deref_mut()
        .is_some_and(|worker| retarget(kind, state, worker, to));
    if goes_on { Flow::Next } else { Flow::Done }
}

/// The order's patch as the snapshot has it, if workers can gather it.
fn patch_of<'a>(kind: OrderKind, world: &'a World) -> Option<&'a View> {
    let OrderKind::Gather(patch) = kind else {
        return None;
    };
    world
        .get(patch)
        .filter(|view| view.is_patch && !world.patch_covered(patch))
}

fn to_patch(
    kind: &mut OrderKind,
    state: &mut Gather,
    unit: &mut Unit,
    def: &WorkerDef,
    world: &World,
    claims: &mut Claims,
) -> Flow {
    let Some(patch) = patch_of(*kind, world) else {
        return lose_patch(kind, state, unit, world, claims);
    };
    // `> Gathering`: MiningReach.
    let in_reach = unit.shape().gap(&patch.shape) <= def.mining_reach;
    // As near as the mover gets it, behind units that won't move: at a taken patch, that is arriving at it.
    let stopped_short = state.moving && unit.mover.status == MoveStatus::Arrived;
    if in_reach || stopped_short {
        let is_taken = taken(world, claims, patch.id, None);
        if !is_taken && in_reach {
            claims.0.push(patch.id);
            unit.halt();
            state.enter(GatherPhase::Mining { done: 0 });
            return Flow::Stay;
        }
        if is_taken {
            // `> Patch Occupancy`: "it goes to the nearest unoccupied patch to itself instead. If no patch is
            // unoccupied, it waits at the current patch".
            let to = free_patch(world, claims, unit.pos, unit.owner, patch.pos, &[patch.id]);
            let Some(worker) = unit.worker.as_deref_mut() else {
                return Flow::Done;
            };
            return match to {
                Some(other) => {
                    retarget(kind, state, worker, Some(other));
                    Flow::Next
                }
                None => {
                    unit.halt();
                    state.enter(GatherPhase::Waiting { since: world.tick });
                    Flow::Stay
                }
            };
        }
        // Stopped short of a free patch (a worker that has finished there is still in the way, say): no rule
        // sends it elsewhere, so it tries again ([`leg`]).
    }
    leg(state, unit, patch, world);
    Flow::Stay
}

fn wait(
    kind: &mut OrderKind,
    state: &mut Gather,
    unit: &mut Unit,
    def: &WorkerDef,
    world: &World,
    claims: &mut Claims,
) -> Flow {
    let Some(patch) = patch_of(*kind, world) else {
        return lose_patch(kind, state, unit, world, claims);
    };
    // Q0037 point 6: the first to wait gets the patch next.
    if taken(world, claims, patch.id, Some(unit.id)) || world.first_waiter(patch.id) != Some(unit.id) {
        return Flow::Stay;
    }
    if unit.shape().gap(&patch.shape) <= def.mining_reach {
        claims.0.push(patch.id);
        state.enter(GatherPhase::Mining { done: 0 });
        return Flow::Stay;
    }
    // It waited out of reach, behind the worker that mined: it closes in, and arrives like any other.
    state.enter(GatherPhase::ToPatch);
    Flow::Next
}

fn to_drop_off(kind: &mut OrderKind, state: &mut Gather, unit: &mut Unit, def: &WorkerDef, world: &World) -> Flow {
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
        // `> Assigned Drop-off`: "the return trip's destination is read from AssignedDropOff". Q0037 point 8:
        // without one, the nearest of the owner's as the return starts; with none, the worker idles with its load.
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
    leg(state, unit, node, world);
    Flow::Stay
}

fn delivered(kind: &mut OrderKind, state: &mut Gather, unit: &mut Unit, queued: bool, world: &World) -> Flow {
    state.node = None;
    // Q0037 point 12: "a queued order starts after the Drone's next drop-off".
    if queued {
        return Flow::Done;
    }
    let (pos, owner) = (unit.pos, unit.owner);
    let Some(worker) = unit.worker.as_deref_mut() else {
        return Flow::Done;
    };
    match *kind {
        OrderKind::Gather(_) => {
            state.enter(GatherPhase::ToPatch);
            Flow::Next
        }
        // Q0037 point 1: after a Drop Off order "it goes back to the patch it was mining, if any; otherwise it
        // idles".
        _ => match worker
            .patch
            .filter(|&patch| patch_of(OrderKind::Gather(patch), world).is_some())
        {
            Some(patch) => {
                *kind = OrderKind::Gather(patch);
                take_up(worker, patch, pos, owner, world);
                state.enter(GatherPhase::ToPatch);
                Flow::Next
            }
            None => Flow::Done,
        },
    }
}

#[cfg(test)]
mod tests;
