//! The job at a Supply Delivery Station (slice M4-I; `factions/syndicate/units/agent.md > Gathering > Supplies`,
//! design change 0256, which answers question Q0067): the way there, the wait and the start of the pick-up. The Drone's
//! job rules hold here as at a patch (design change 0255; the module docs of `orders::gather`), with the station's own:
//! - **"Any number pick up at once, as at patches"** (0256 point 1): there is no occupancy, and each worker heads for
//!   a spot of its own round the station ([`spot`]). **"A pick-up starts only if a Supply remains that no running
//!   pick-up has claimed"** (point 2): a worker within its PickUpReach of the station
//!   begins a pick-up if the station's Supplies that no pick-up under way claimed as the phase began
//!   (`World::unclaimed_supplies`) outnumber those claimed in this tick ([`Claims`]), and claims one. The behaviours
//!   run in id order, so the lowest ids get the last Supplies.
//! - **An empty station** (point 3: "every Agent picking up there stands next to it indefinitely, keeping its job,
//!   waiting for the next supply delivery"): with every Supply claimed or none left, the worker waits beside the
//!   station (`GatherPhase::Waiting`), and begins its pick-up as soon as a Supply is free for it. It never looks round
//!   for another station. One whose leg ended short of the station (design change 0141's rule for a patch it can't
//!   reach) waits as close as it got, and sets off again, for a spot free by then, every `Gathering::recheck_ticks`
//!   while its owner's grids show a way.
//! - **Gather while carrying a load** (design change 0128, one of the Drone's rules): a worker that carries a Supply
//!   goes to the station all the same, and on arrival turns round to deliver it, without picking up. One that carries
//!   Space Crystals picks up as any other, and its crystals are lost when its pick-up ends (0256 point 4; design change
//!   0292 point 3, `economy::gathering`).
//! - **The pick-up** (`GatherPhase::Mining` at a station) is timed work that the economy phase counts, PickUpDuration;
//!   the Supply leaves the station as it ends (point 2, `economy::stations::take`). One that stops before then takes
//!   nothing (design change 0130), and the Supply it claimed is free again in the next tick's snapshot.

use crate::defs::{Resource, WorkerDef};
use crate::mover::MoveStatus;
use crate::units::Tick;

use super::super::state::{Gather, GatherPhase};
use super::super::world::{KnownStation, Unit, World};
use super::patch::end;
use super::{Claims, Flow, carrying};
use crate::ruleset::rules;

mod spot;

/// Whether a pick-up may start at `station` now (0256 point 2, module docs): a Supply remains that no pick-up claimed.
fn supply_free(world: &World, claims: &Claims, station: &KnownStation) -> bool {
    world.unclaimed_supplies(station.id) > claims.supplies_taken(station.id)
}

/// The worker is within its PickUpReach of `station`, which its owner sees, and a Supply is free there: its pick-up
/// begins, and claims the Supply.
fn begin_pick_up(state: &mut Gather, unit: &mut Unit, station: &KnownStation, claims: &mut Claims) -> Flow {
    claims.take_supply(station.id);
    unit.halt();
    state.enter(GatherPhase::Mining { done: 0 });
    Flow::Stay
}

/// `GatherPhase::ToPatch` for a Gather on `station`: the leg there, and what the worker does on arrival (module
/// docs). A worker of a type that gathers no Supplies has nothing to do there (a save edited by hand: the gate gives
/// it no such order): its Gather ends.
pub(super) fn to_station(
    state: &mut Gather,
    unit: &mut Unit,
    def: &WorkerDef,
    station: KnownStation,
    world: &World,
    claims: &mut Claims,
) -> Flow {
    let Some(load) = def.supplies else {
        return end(state, unit);
    };
    // A leg that ended short of the station counts as arriving, out of reach (0141).
    let in_reach = unit.shape().gap(&station.shape) <= load.pick_up_reach;
    let ended = state.moving && unit.mover.status != MoveStatus::Moving;
    if !in_reach && !ended {
        spot::station_leg(state, unit, &station, load.pick_up_reach, world, claims);
        return Flow::Stay;
    }
    // Design change 0128: with a Supply, it turns round without picking up, and claims nothing.
    if carrying(unit, Resource::Supplies) {
        state.enter(GatherPhase::ToDropOff);
        return Flow::Next;
    }
    // Design change 0139: within reach the worker sees the station, at the latest in the next tick's snapshot.
    if in_reach && !station.seen {
        return Flow::Stay;
    }
    if in_reach && supply_free(world, claims, &station) {
        return begin_pick_up(state, unit, &station, claims);
    }
    // 0256 point 3: an empty station, or every Supply claimed; or a station it can't get to (0141).
    unit.halt();
    state.enter(GatherPhase::Waiting { since: world.tick });
    Flow::Stay
}

/// `GatherPhase::Waiting` beside `station`, since tick `since` (module docs): the pick-up begins as soon as a Supply is
/// free; out of reach, the worker sets off again in the ticks that are multiples of `Gathering::recheck_ticks`, but in
/// the one it began to wait in, if its owner's grids show a way.
pub(super) fn wait_at_station(
    state: &mut Gather,
    unit: &mut Unit,
    def: &WorkerDef,
    since: Tick,
    station: KnownStation,
    world: &World,
    claims: &mut Claims,
) -> Flow {
    let Some(load) = def.supplies else {
        return end(state, unit);
    };
    let in_reach = unit.shape().gap(&station.shape) <= load.pick_up_reach;
    if in_reach && station.seen && supply_free(world, claims, &station) {
        return begin_pick_up(state, unit, &station, claims);
    }
    if in_reach || since == world.tick || !world.tick.is_multiple_of(rules().tuning.gathering.recheck_ticks) {
        return Flow::Stay;
    }
    if world.reachable(unit, &station.shape) {
        state.enter(GatherPhase::ToPatch);
        return Flow::Next;
    }
    Flow::Stay
}
