//! The job's way back (the module docs of `orders::gather`): which drop-off, the leg to it, the line at a hatch, and
//! what follows the delivery.

use glam::Vec2;

use crate::defs::{DropOffChoice, Nearest, WorkerDef};
use crate::economy::drop_off::{self, DropPoint};
use crate::economy::gathering::is_drop_off;
use crate::ids::{ObjectId, PlayerId};
use crate::shape::Shape;
use crate::sides::SideLetter;
use crate::units::Tick;

use super::super::OrderKind;
use super::super::state::{Gather, GatherPhase};
use super::super::world::{Unit, View, World};
use super::legs::return_leg;
use super::patch::find_another;
use super::site::{Site, site};
use super::{Claims, Flow};

/// The drop-off of `owner` nearest to `pos`, for a load of either resource: by its centre, or by the centre of its
/// crystal hatch for one with hatches (`factions/syndicate/units/agent.md > Gathering rules`, design change 0375,
/// answering Q0136: "the own operational Tunnel whose crystal hatch is **nearest to the Agent itself** (straight line)
/// **when it finishes mining** [...]; a tie goes to the Tunnel with the **earlier id**. This is chosen **regardless of
/// how many other Agents already use it**"; design change 0292 point 2: "'nearest Tunnel' for Supplies is measured to
/// the crystal hatch, the same measure as for crystals"); the lowest id among equals (`economy::drop_off::nearest`, as
/// `Sim::nearest_drop_off`). The Agent's `pos` is its own centre as each of its returns starts, where it finished
/// mining (design change 0384: "It is chosen **at every return**, like the Recruit's Storage", [`to_drop_off`]); a
/// Drone's, where it is as it gets its Gather (`super::begin`) or as a return starts without one.
pub(super) fn nearest_drop_off<'a>(world: &'a World, pos: Vec2, owner: Option<PlayerId>) -> Option<&'a View> {
    owner?;
    let own = (world.drop_offs()).filter(|(node, _)| node.owner == owner);
    drop_off::nearest(pos, own.map(|(node, crystals)| (node, *crystals)))
}

/// Design change 0375 (answering Q0136; `factions/cults/units/recruit.md > Gathering`, Which Storage), for a worker
/// that chooses at each return edge to edge (`Nearest::EdgeToEdge`: the Recruit): "the own completed Storage
/// **nearest to the Recruit itself** (edge to edge) **when it finishes mining**", `worker` being its shape as its
/// return starts ([`to_drop_off`]), the gap measured to where the drop-off takes Space Crystals; "**Tie**: the Storage
/// with the **earlier id** (built first)", the first of equals in id order. It replaces design change 0345's "edge to
/// edge from its patch". Only those that take loads now are looked at (`World::drop_offs`: a completed Storage).
///
/// **At the end of a face**, 0375 has "A Recruit mining an end patch from its own face's side is nearer that face's
/// Storage, so it stays with its face": so it is while its Silhouette stands wholly over its own face's Storage. Of two
/// Storages side by side under a row, a Recruit whose Silhouette reaches over both (one that came along the lane from
/// the neighbouring face and stopped as it came within MiningReach) is as near both, edge to edge, and the earlier id
/// takes its load. That is the literal reading, built as it reads and reported by slice M5-S2a
/// (`tests/recruit_gathering/storages.rs`).
fn nearest_to_worker<'a>(world: &'a World, worker: &Shape, owner: Option<PlayerId>) -> Option<&'a View> {
    owner?;
    (world.drop_offs())
        .filter(|(node, _)| node.owner == owner)
        .map(|(node, crystals)| (node, crystals.shape.gap(worker)))
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(node, _)| node)
}

/// The way back, and the wait at a hatch: `since` is the tick the worker came within reach of the hatch it waits
/// at (`GatherPhase::AtHatch`), `None` while it is on its way (`GatherPhase::ToDropOff`).
pub(super) fn to_drop_off(
    kind: &mut OrderKind,
    state: &mut Gather,
    unit: &mut Unit,
    def: &WorkerDef,
    since: Option<Tick>,
    world: &World,
    claims: &mut Claims,
) -> Flow {
    // Design change 0134: a worker that is where its patch was when the patch goes (the one that carries off its
    // last load) looks round at once. One farther off goes on, and looks round there after its drop-off.
    if !state.ending
        && let Site::Gone(spot) = site(*kind, state, unit, world)
        && unit.shape().gap(&spot) <= def.crystals.pick_up_reach
    {
        return find_another(kind, state, unit, def, world, claims);
    }
    // Design changes 0375 and 0384: the return of a worker that chooses at every return starts (it has read no
    // drop-off yet), and it takes the one nearest to itself where it finished mining (or turned round with the load it
    // brought, design change 0128): the Recruit the Storage nearest edge to edge, the Agent the Tunnel whose crystal
    // hatch is nearest in a straight line. One lost on the way was replaced by the removal path, as any worker's is
    // (design change 0170).
    let starts = state.node.is_none() && matches!(*kind, OrderKind::Gather(_));
    let chosen = match def.drop_off {
        DropOffChoice::EachReturn(Nearest::EdgeToEdge) if starts => {
            Some(nearest_to_worker(world, &unit.shape(), unit.owner))
        }
        DropOffChoice::EachReturn(Nearest::StraightLine) if starts => {
            Some(nearest_drop_off(world, unit.pos, unit.owner))
        }
        _ => None,
    }
    .map(|node| node.map(|node| node.id));
    let (pos, owner) = (unit.pos, unit.owner);
    let Some(worker) = unit.worker.as_deref_mut() else {
        return Flow::Done;
    };
    if worker.load == 0 {
        state.enter(GatherPhase::Delivered);
        return Flow::Next;
    }
    // The hatch and the reach are those of what it carries (slice M4-I); the nearest drop-off is by its crystal hatch,
    // whatever it carries (design change 0292 point 2, [`nearest_drop_off`]).
    let resource = worker.resource;
    let own_drop_off = |id: ObjectId| {
        world
            .get(id)
            .filter(|node| is_drop_off(node.kind) && owner.is_some() && node.owner == owner)
    };
    let node = match *kind {
        OrderKind::DropOff(node) => own_drop_off(node),
        // `factions/colonists/units/drone.md > Assigned Drop-off`: "the return trip's destination is read from
        // AssignedDropOff". Design change 0135 (`factions/colonists/units/drone.md > Drop-off gone`): without one as
        // the trip starts, the nearest own Egress Node becomes it; with none, the worker idles holding its load. One
        // destroyed on the way was replaced as it went, or left none (design change 0170, `Sim::reassign_drop_offs`):
        // the leg takes the one it reads. An Agent's or a Recruit's Gather begins without one (`super::begin`), and
        // each of its returns takes the nearest as it starts (above; design change 0384 replaces the Agent's Tunnel
        // "chosen at the first return and kept for that Gather", design change 0255).
        _ => {
            let nearest = || nearest_drop_off(world, pos, owner).map(|node| node.id);
            if let Some(chosen) = chosen {
                worker.drop_off = chosen;
            }
            worker.drop_off = worker.drop_off.or_else(nearest);
            worker.drop_off.and_then(own_drop_off)
        }
    };
    let Some(node) = node else {
        return Flow::Done;
    };
    if state.node != Some(node.id) {
        state.node = Some(node.id);
        state.moving = false;
        state.retry_at = 0;
    }
    // Where it takes the load: the structure, or the hatch of what it carries, Space Crystals at Side B and Supplies at
    // Side D (design change 0248). (One that takes none now, being warped in, is walked to as a whole.)
    let anywhere = DropPoint {
        shape: node.shape,
        centre: node.pos,
        side: None,
    };
    let point = world.drop_point(node.id, resource).unwrap_or(&anywhere);
    // `factions/syndicate/units/agent.md > Gathering`: DropOffReach; `factions/syndicate/units/agent.md > Supplies`:
    // the same 8 su, "from the Supplies hatch on Side D" (design change 0292 point 1).
    let reach = def.load(resource).unwrap_or(def.crystals).drop_off_reach;
    let in_reach = unit.shape().gap(&point.shape) <= reach;
    let Some(side) = point.side else {
        if in_reach {
            unit.halt();
            state.enter(GatherPhase::DroppingOff { done: 0 });
        } else {
            return_leg(state, unit, &point.shape, None, world);
        }
        return Flow::Stay;
    };
    if !in_reach {
        // On its way; or pushed out of reach while it waited, which costs it its place in the line.
        if since.is_some() {
            state.enter(GatherPhase::ToDropOff);
        }
        return_leg(state, unit, &point.shape, Some(reach), world);
        return Flow::Stay;
    }
    at_hatch(state, unit, (node.id, side), since, world, claims)
}

/// `factions/syndicate/tunnels.md > Tunnel > SymmetryType - ABCD`: "Only one Agent may drop off resources at a side
/// at a time"; `factions/syndicate/units/agent.md > Gathering > Gathering rules`, "Waiting at the hatch" (design
/// changes 0255 and 0315): "an Agent waits only once it is within reach of the hatch, while another drops off; the
/// first to arrive within reach delivers next. [...] It waits as long as it takes; it never gives up". (Out of reach
/// it is on its leg, which presses on: `legs::return_leg`.)
///
/// The worker is within reach of `hatch`, waiting there since `since`, or just arrived. It unloads if nobody does
/// there (as phase 5a began, `World::hatch_busy`, or since, [`Claims`]) and it is the first in the line
/// (`World::first_in_line`: of those within reach as the phase began, the one that arrived first, the lowest id among
/// equals; one that arrives in this tick goes behind them all). The line is derived from the workers' orders each
/// tick (plan decision T34), so a worker whose order ends leaves it at once.
fn at_hatch(
    state: &mut Gather,
    unit: &mut Unit,
    hatch: (ObjectId, SideLetter),
    since: Option<Tick>,
    world: &World,
    claims: &mut Claims,
) -> Flow {
    let (node, side) = hatch;
    if since.is_none() {
        unit.halt();
        state.enter(GatherPhase::AtHatch { since: world.tick });
    }
    let free = !world.hatch_busy(node, side) && !claims.hatch_taken(node, side);
    if free && world.first_in_line(node, side).is_none_or(|first| first == unit.id) {
        claims.take_hatch(node, side);
        state.enter(GatherPhase::DroppingOff { done: 0 });
    }
    Flow::Stay
}

/// The load is in the bank.
pub(super) fn delivered(kind: OrderKind, state: &mut Gather) -> Flow {
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
