//! A worker's job — `factions/colonists/units/drone.md > Gathering`, `factions/colonists/units/drone.md > Patch
//! Occupancy (one Drone per patch)`, `factions/colonists/units/drone.md > Last load of a patch`,
//! `factions/colonists/units/drone.md > Interrupted mining`, `factions/colonists/units/drone.md > Other orders keep the
//! load`, `factions/colonists/units/drone.md > Gather while carrying a load`, `factions/colonists/units/drone.md > Drop
//! Off order`, `factions/colonists/units/drone.md > When a Gather ends` and `factions/colonists/units/drone.md >
//! Assigned Drop-off` (slices M3-H and M3-H3, plan decision T17; design changes 0123 and 0127 to 0141 answered question
//! Q0037). `systems/controls/commands.md` and `systems/controls/behaviors.md` define no Gather or Drop Off, so the
//! Drone's own sections are the spec.
//!
//! **One job for every worker type** (slice M4-D, plan decision T34; `factions/syndicate/units/agent.md >
//! Gathering > Gathering rules`, design change 0255: "The Drone's job rules apply to Agents [...] with two exceptions:
//! Agents share patches (no occupancy rule), and there is no Set Drop-off command"). The differences are data: the
//! worker's `WorkerDef` (its loads, `shares_patches`, `drop_off`, `pass_through`) and its drop-off's rule
//! (`defs::DropOff`: anywhere at an Egress Node, at the crystal hatch of a Tunnel, `economy::drop_off`). The text
//! below says "Drone" where the Drone's sections are quoted; it holds for the Agent but for:
//! - **patches are shared**: an Agent neither occupies a patch nor takes one for occupied (`search::taken`), so it
//!   mines on arrival, and waits only short of a patch it can't get to (0141);
//! - **which Tunnel** (design changes 0375 and 0384, answering Q0136): "the own operational Tunnel whose crystal hatch
//!   is **nearest to the Agent itself** (straight line) **when it finishes mining** [...]. It is chosen **at every
//!   return**, like the Recruit's Storage [...], so a new Tunnel is used at once; a Tunnel lost during a return means
//!   picking the nearest again": a Gather begins without an AssignedDropOff ([`begin`]) and each return takes the one
//!   nearest to the Agent as it starts (`DropOffChoice::EachReturn(Nearest::StraightLine)`,
//!   `deliver::nearest_drop_off`), by its crystal hatch whatever the load (design change 0292 point 2);
//! - **the hatch** (`deliver::at_hatch`, phase **AtHatch**): DropOffReach is measured to the 1-square hatch, which
//!   takes one Agent at a time; those within reach wait in the order they came. One that isn't within reach doesn't
//!   wait: its leg presses on, exempt from the occupied-target rule, and begins again at once when the mover gives
//!   it up, for ever (design change 0315, `legs::return_leg`: "Until it is within reach it is never 'arrived' [...]
//!   It waits as long as it takes; it never gives up").
//!
//! **Recruits** (slice M5-D, plan decision T58; `factions/cults/units/recruit.md > Gathering`,
//! `factions/cults/structures/storage.md`) are data on the same job: they share patches as Agents do ("any number of
//! Recruits may mine the same Space Crystals patch at the same time"), deliver anywhere at a Storage, any number at
//! once ("Multiple Recruits can drop off Space Crystals simultaneously"), and keep normal collision (design change
//! 0106). **Which Storage** (design change 0345, answering Q0102, and design change 0375, answering Q0136;
//! `deliver::nearest_to_worker`): "at **every return** the Recruit takes its load to the own completed Storage
//! **nearest to the Recruit itself** (edge to edge) **when it finishes mining**", chosen as each return starts, the
//! earlier id among equals; with none, the Recruit idles holding its load (the rule of design change 0135).
//!
//! One behaviour runs both orders: `OrderKind::Gather(patch)` and `OrderKind::DropOff(node)`. Its phases
//! (`state::GatherPhase`):
//! - **ToPatch**: a leg to the order's patch. On arrival the worker **claims** the patch if it is free and starts
//!   Mining. At a patch it can't mine, occupied or out of reach, it looks round (below) and goes to another, or
//!   waits. A worker that carries a load goes to the patch all the same, and on arrival "skips mining (it doesn't
//!   occupy the patch) and returns to deliver as if it had just mined the load there" (design change 0128).
//! - **Waiting**: "right at the patch it tried to mine, within MiningReach", or as close as it got to a patch it can't
//!   reach (design changes 0132 and 0141). It mines the patch as soon as it frees, and looks round again every
//!   `Gathering::recheck_ticks`, all waiting workers in the same ticks: it goes to a patch that has freed up meanwhile,
//!   and one that couldn't get to its patch tries again ("carrying on as soon as the way clears"). It never gives up
//!   for good.
//! - **Mining** and **DroppingOff**: the timed work, which the economy phase counts (`economy::gathering::step`).
//!   Mining that stops before its 32 frames are done, whatever stops it, collects nothing, and the patch frees at
//!   once (design change 0130): the frames are the order's state, and go with it.
//! - **ToDropOff**: a leg to its AssignedDropOff, read again every tick, so that Set Drop-off Location redirects
//!   "a trip already in progress" (`factions/colonists/units/drone.md > Assigned Drop-off`); for a Drop Off order, to
//!   the order's own drop-off. Without a drop-off as the return starts, the worker assigns the nearest of its owner's,
//!   and with none it idles holding its load (design change 0135). The removal path replaces one lost on the way
//!   (0170).
//! - **Delivered**: the load is in the bank. A Gather goes back to its patch, the one where it last mined (design
//!   change 0132, item 4: the order follows the worker from patch to patch). A Drop Off order is complete: "an
//!   explicit Drop Off is only a delivery" (0127); the gate never gives one to a worker without a load (0140).
//!
//! **Occupancy** (`factions/colonists/units/drone.md > Patch Occupancy`; design change 0133 and the Director's choice
//! of rule): a patch is taken while a worker of any player mines it (as phase 5a began, `World::patch_mined`), once a
//! worker with a lower id has claimed it in this tick ([`Claims`]), and, for every other worker, while a worker waits
//! at it within MiningReach (`World::waited_at`), which mines it as soon as it frees. Of several workers waiting at a
//! patch the lowest id gets it: the behaviours run in id order, and the first to see the patch free claims it. A worker
//! on its way doesn't occupy anything, but one that sets off for a free patch it found claims it for that tick, so
//! that no other worker's look in the same tick sends it there too.
//!
//! **Looking round** (`search::search`, design changes 0123, 0131 and 0141): within the worker's search radius
//! (`WorkerDef::search_radius`) around itself, among the patches its owner knows that no plate covers, the nearest
//! free one it can reach.
//!
//! **What the worker goes by** (design change 0139): its patch as its owner knows it, as every order on an object
//! does (`site`).
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
//! **Legs** (`legs`) are moves to a point, not to the object. One that ends short of its patch counts as arriving
//! there, out of reach (design change 0141); one back to the drop-off that ends short is begun again after
//! `tuning::gathering::Gathering::leg_retry_ticks`; an Agent's leg back to a hatch doesn't end short behind standing
//! units (design change 0315, `legs::return_leg`).
//!
//! **Collision** (design change 0106, `factions/colonists/units/drone.md > Gathering Drones pass through each
//! other`): Drones on a Gather job don't collide with each other, whoever owns them ([`on_gather_job`]; the
//! exception itself is `movement::space::blocks`). They collide with everything else, and having an order they
//! never yield (`systems/units.md > UnitCollision > Ground Collision > Who yields`). So Drones wait, mine and
//! deliver inside each other, and the patch's occupancy is the only limit. A Drone whose job ends while it is
//! inside another follows `systems/units.md > Units that already overlap`: the two ignore each other until they are
//! apart.

mod claims;
mod deliver;
mod legs;
mod patch;
mod search;
mod site;
mod station;

use crate::defs::{DropOffChoice, Resource};
use crate::object::Object;

use super::OrderKind;
use super::state::{Behaviour, Gather, GatherPhase};
use super::world::{Unit, World};
pub(crate) use claims::Claims;
use deliver::{delivered, nearest_drop_off, to_drop_off};
use patch::{mine, to_patch, wait};
use site::note_spot;

/// `factions/colonists/units/drone.md > Gathering Drones pass through each other` (design change 0106): whether
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

/// The state a Gather or a Drop Off order starts with, for `unit`, a worker (`orders::start`).
/// - `factions/colonists/units/drone.md > Assigned Drop-off > Auto-assignment`: "When a Drone with no AssignedDropOff
///   is ordered to Gather, it automatically assigns the nearest Egress Node".
/// - `factions/colonists/units/drone.md > Gather while carrying a load` (design change 0128): a worker that carries a
///   load "goes straight to the new pick-up point" all the same.
/// - `factions/colonists/units/drone.md > Drop Off order` (design change 0127): straight to the order's drop-off.
/// - For a worker without a lasting AssignedDropOff, that chooses at every return (`DropOffChoice::EachReturn`): the
///   Gather begins without one, and each return takes the one nearest to the worker as it starts
///   (`deliver::to_drop_off`): the Agent's Tunnel (design changes 0375 and 0384; `factions/syndicate/units/agent.md >
///   Gathering rules`: "It is chosen **at every return**, like the Recruit's Storage") and the Recruit's Storage
///   (design changes 0345 and 0375).
pub(super) fn begin(kind: OrderKind, unit: &mut Unit, world: &World) -> Gather {
    let (pos, owner) = (unit.pos, unit.owner);
    let keeps =
        (unit.unit_def().and_then(|unit| unit.worker)).is_none_or(|def| def.drop_off == DropOffChoice::Assigned);
    let Some(worker) = unit.worker.as_deref_mut() else {
        return Gather::new(GatherPhase::ToPatch);
    };
    match kind {
        OrderKind::Gather(_) => {
            if !keeps {
                worker.drop_off = None;
            } else if worker.drop_off.is_none() {
                worker.drop_off = nearest_drop_off(world, pos, owner).map(|node| node.id);
            }
            Gather::new(GatherPhase::ToPatch)
        }
        _ => Gather::new(GatherPhase::ToDropOff),
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
    note_spot(*kind, state, unit, world);
    // A phase that ends starts the next one in the same tick. A worker goes through a few at most: it looks round,
    // finds a patch beside it and claims it, say. None of the paths comes back to where it began.
    for _ in 0..6 {
        let flow = match state.phase {
            GatherPhase::ToPatch => to_patch(kind, state, unit, &def, world, claims),
            GatherPhase::Waiting { since } => wait(kind, state, unit, &def, since, world, claims),
            GatherPhase::Mining { .. } => mine(kind, state, unit, &def, world, claims),
            GatherPhase::ToDropOff => to_drop_off(kind, state, unit, &def, None, world, claims),
            GatherPhase::AtHatch { since } => to_drop_off(kind, state, unit, &def, Some(since), world, claims),
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

/// Whether the worker carries anything.
fn loaded(unit: &Unit) -> bool {
    unit.worker.as_ref().is_some_and(|worker| worker.load > 0)
}

/// Whether the worker carries a load of `resource` (`economy::gathering::Worker::resource`).
fn carrying(unit: &Unit, resource: Resource) -> bool {
    unit.worker
        .as_ref()
        .is_some_and(|worker| worker.load > 0 && worker.resource == resource)
}

/// Whether the job is on its way back: from the patch with its load, at the drop-off, or just done there.
fn returning(state: &Gather) -> bool {
    matches!(
        state.phase,
        GatherPhase::ToDropOff | GatherPhase::AtHatch { .. } | GatherPhase::DroppingOff { .. } | GatherPhase::Delivered
    )
}

#[cfg(test)]
mod tests;
