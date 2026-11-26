//! The movement invariants I1–I7 of `docs/movement.md > 8` (I1 with its two exceptions of design change 0106,
//! [`may_overlap`]; I2 with its one exception, the exit doorway of design change 0080, and of a portal design change
//! 0145's energy doorway), checked from outside through the sim's public state,
//! the vision invariant V1 (the vision kept up tick by tick is the one a fresh build gives, and every Visible tile
//! is Explored: `vision`), the path knowledge invariant K1 (each player's planning grids are those a fresh build
//! from the state gives: `paths`), the projectile invariant P1 (every projectile lands by its arrival tick:
//! `projectiles`), the operation invariant O1 (a Tunnel's operation and its site name each other: `operations`), and
//! an ASCII picture of the map with its units for readable failures.
//!
//! [`super::TestSim`] checks them after every tick (on by default). A test that steps a bare [`Sim`] keeps its own
//! [`Invariants`] and calls [`Invariants::check`] after each step.

mod operations;
mod picture;
mod projectiles;

use std::collections::BTreeMap;

use glam::Vec2;

use crate::command::{Command, DebugCommand, TickInputs};
use crate::defs::{Domain, MovementModel};
use crate::ids::ObjectId;
use crate::mover::{Locomotion, MoveGoal, MoveStatus, Mover, PathState};
use crate::nav;
use crate::object::Object;
use crate::orders::{OrderKind, Target, gather};
use crate::paths::boxes_overlap;
use crate::production::ExitDoor;
use crate::sim::Sim;
use crate::tuning::movement::MAX_REPATHS;
use crate::units::Tick;

/// I6: a path request is served within this many ticks. Requests are served oldest first and at least one per
/// tick (`docs/movement.md > 5.5`), so this bounds how far a test's scenario may fall behind, not the sim.
pub const MAX_REQUEST_AGE: Tick = 32;

/// (M3-D2) I2: how far off the open arc of a portal's energy doorway a unit in it may be, by
/// `EnergyDoorway::margin` (in the footprint's half extents): the rounding of a walk along an end of the arc.
const ARC_TOLERANCE: f32 = 1e-3;

pub use picture::ascii;

/// A broken invariant.
#[derive(Clone, Debug, PartialEq)]
pub struct Violation {
    /// "I1" … "I7", "V1", "K1", "P1", "O1".
    pub invariant: &'static str,
    pub message: String,
    /// Where on the map to look.
    pub at: Vec2,
}

/// What the checker remembers of a unit from the tick before.
#[derive(Clone, Debug)]
struct Seen {
    pos: Vec2,
    had_goal: bool,
    /// It was taking a sidestep for a blocked unit of its owner (design change 0022).
    sidestepping: bool,
    repaths_left: u8,
    /// The building whose exit doorway it was still in (`Mover::exit`, design change 0080).
    exit: Option<ObjectId>,
    /// It was a Drone on a Gather job (`orders::gather::on_gather_job`, design change 0106).
    gathering: bool,
    /// The velocity it carried into the next tick (`Mover::momentum`, slice M3-G).
    momentum: Vec2,
}

/// Checks I1–I7 between ticks. It remembers each unit's state of the tick before, so check after **every** step.
#[derive(Clone, Debug)]
pub struct Invariants {
    seen: BTreeMap<ObjectId, Seen>,
}

fn leg_of(o: &Object) -> Option<Tick> {
    o.orders.as_ref()?.current.as_ref().map(|active| active.since)
}

fn seen(o: &Object, mover: &Mover) -> Seen {
    Seen {
        pos: o.pos,
        had_goal: mover.goal != MoveGoal::None,
        sidestepping: mover.sidestep.is_some(),
        repaths_left: mover.repaths_left,
        exit: mover.exit,
        gathering: gather::on_gather_job(o),
        momentum: mover.momentum,
    }
}

impl Invariants {
    /// Starts from the sim's current state.
    pub fn new(sim: &Sim) -> Self {
        let seen = sim
            .objects()
            .filter_map(|o| Some((o.id, seen(o, o.mover.as_ref()?))))
            .collect();
        Self { seen }
    }

    /// Checks the state after a step that applied `inputs` (teleports are read from them: a teleported unit may
    /// "move" without a goal). Returns the first violation.
    pub fn check(&mut self, sim: &Sim, inputs: &TickInputs) -> Result<(), Violation> {
        let teleported: Vec<ObjectId> = inputs
            .commands
            .iter()
            .filter_map(|c| match c.command {
                Command::Debug(DebugCommand::SetPosition { id, .. }) => Some(id),
                _ => None,
            })
            .collect();
        let result = self
            .check_units(sim, &teleported)
            .and_then(|()| no_overlaps(sim, &self.seen))
            .and_then(|()| vision(sim))
            .and_then(|()| path_knowledge(sim))
            .and_then(|()| projectiles::projectiles(sim))
            .and_then(|()| operations::operations(sim));
        self.seen = Self::new(sim).seen;
        result
    }

    fn check_units(&self, sim: &Sim, teleported: &[ObjectId]) -> Result<(), Violation> {
        let tick = sim.tick();
        let overlapped = overlapped_before(sim, &self.seen);
        for o in sim.objects() {
            let fail = |invariant, message: String| {
                Err(Violation {
                    invariant,
                    message: format!("#{} ({:?}) {message}", o.id.0, o.kind),
                    at: o.pos,
                })
            };
            // I4: every float of the state is finite.
            if !floats_finite(o) {
                return fail("I4", format!("has a float that isn't finite: {o:?}"));
            }
            let def = o.def();
            let (Some(unit), Some(half), Some(mover)) = (&def.unit, def.half_extents(), &o.mover) else {
                continue;
            };
            let before = self.seen.get(&o.id);
            // I2: a ground box overlaps no blocked tile and stays on the map. The one exception (design change
            // 0080, `core/entities.md > Structure Type > Exit Doorway`): a unit that a producer released, while
            // its box still overlaps that building's footprint (`Mover::exit`), overlaps that footprint's tiles
            // and no other blocked tile; and from tick to tick it only comes out (`ExitDoor::allows`). (M3-D2) Out
            // of a portal's energy doorway (design change 0145) it never stands off the open arc: its centre lies
            // in an open direction from the portal's, within a rounding tolerance (`EnergyDoorway::margin`).
            let grid = unit
                .base_def()
                .nav_class()
                .and_then(|class| sim.nav().grid(class, nav::size_class(half * 2.0)));
            match mover.exit {
                None => {
                    if grid.is_some_and(|grid| !nav::box_clear(grid, half, o.pos)) {
                        return fail("I2", format!("at {} overlaps a blocked tile or the map's edge", o.pos));
                    }
                }
                Some(building) => {
                    let Some(door) = sim.object(building).and_then(ExitDoor::of) else {
                        return fail("I2", format!("is in the doorway of #{}, which has none", building.0));
                    };
                    if let ExitDoor::Centre(portal) = door
                        && portal.margin(o.pos - portal.centre()) < -ARC_TOLERANCE
                    {
                        return fail(
                            "I2",
                            format!(
                                "at {} left the energy doorway of #{} by a closed side",
                                o.pos, building.0
                            ),
                        );
                    }
                    if !door.holds(o.pos, half) {
                        return fail(
                            "I2",
                            format!("at {} is out of the doorway of #{} but still exempt", o.pos, building.0),
                        );
                    }
                    if grid.is_some_and(|grid| !door.box_clear(grid, half, o.pos)) {
                        return fail(
                            "I2",
                            format!("at {} in a doorway overlaps a blocked tile of another object", o.pos),
                        );
                    }
                    let came_from = before.filter(|b| b.exit == Some(building) && !teleported.contains(&o.id));
                    if let Some(b) = came_from
                        && !door.allows(b.pos, o.pos, half)
                    {
                        return fail("I2", format!("went from {} back into its doorway, to {}", b.pos, o.pos));
                    }
                    // Only the units a producer just released come into being inside a footprint.
                    if before.is_none_or(|b| b.exit != Some(building)) && o.prev_pos != o.pos {
                        return fail("I2", format!("entered the doorway of #{} from outside", building.0));
                    }
                }
            }
            // I3: a unit moves at most its speed per tick. (M3-G) For a unit with momentum that is its model's
            // maximum, where thrust and drag balance on open ground (`MovementAttrs::max_speed`): a mode that
            // slows the unit only weakens its thrust, and the speed it has decays.
            let drag = unit.movement.model() == MovementModel::Drag;
            let step = unit.movement.max_speed() * if drag { 1.0 } else { mover.speed_factor };
            let moved = o.pos.distance(o.prev_pos);
            if moved > step * 1.0001 + 1e-3 {
                return fail("I3", format!("moved {moved} su in one tick, more than its {step}"));
            }
            if let Some(before) = before
                && before.pos != o.prev_pos
                && !teleported.contains(&o.id)
            {
                return fail(
                    "I5",
                    format!("was moved outside the movement step: {} to {}", before.pos, o.prev_pos),
                );
            }
            // I5, I7: a unit without a goal doesn't move, so an Arrived unit never moves again without an order,
            // except by the one short sidestep it takes for a blocked unit of its owner (design change 0022).
            // It may move in a tick it ends without a goal only if it had one before, an order started, or it was
            // taking a sidestep.
            let had_goal = before.is_some_and(|b| b.had_goal);
            let sidestepping = before.is_some_and(|b| b.sidestepping);
            // A unit in an exit doorway walks out of it by itself (design changes 0080 and 0086).
            let leaving = before.is_some_and(|b| b.exit.is_some());
            let order_started = leg_of(o) == Some(tick);
            // A leg is one `Mover::go_to`: an order's move, or one a behaviour starts (a chase, a Patrol's next
            // leg, the way back to an IdleOrigin). `go_to` in this tick left a full count and its tick behind,
            // also when the move ended in the same tick.
            let new_leg = mover.repaths_left == MAX_REPATHS && mover.last_repath == tick;
            let began = order_started || new_leg;
            // (M3-G) A DragMovement unit that still had momentum brakes to a stop, moving meanwhile
            // (`movement::drag`, `docs/movement.md > 6.7`). No other model has momentum, ever.
            let had_momentum = before.is_some_and(|b| b.momentum != Vec2::ZERO);
            if mover.momentum != Vec2::ZERO && !drag {
                return fail(
                    "I5/I7",
                    format!("has momentum, which its movement model has none of: {mover:?}"),
                );
            }
            let coasting = drag && had_momentum;
            // An idle unit that overlapped another moves off it (design change 0126, `movement::separate`).
            let separating = overlapped.contains(&o.id) && o.orders.as_ref().is_some_and(|orders| orders.is_idle());
            if o.pos != o.prev_pos
                && mover.goal == MoveGoal::None
                && !had_goal
                && !began
                && !sidestepping
                && !leaving
                && !coasting
                && !separating
            {
                return fail(
                    "I5/I7",
                    format!(
                        "moved from {} to {} without a goal, a new order or move, a sidestep, a doorway to leave, \
                         momentum to brake off, or a unit it overlapped to move off",
                        o.prev_pos, o.pos
                    ),
                );
            }
            // A finished move leaves the unit standing (`docs/movement.md > 6.5`), and so does a finished sidestep.
            // (M3-G) But a DragMovement unit braking off its momentum: it is Stopping, its velocity is that
            // momentum, and it is slower than a tick before, every tick, so it does come to rest.
            let at_rest = mover.velocity == Vec2::ZERO && mover.locomotion == Locomotion::Stationary;
            let braking = drag
                && mover.locomotion == Locomotion::Stopping
                // Or it met an obstacle in this tick, which left it no velocity (design change 0035).
                && (mover.velocity == mover.momentum || mover.momentum == Vec2::ZERO)
                && before.is_some_and(|b| mover.velocity.length_squared() < b.momentum.length_squared());
            if mover.goal == MoveGoal::None
                && ((mover.sidestep.is_none() && mover.exit.is_none() && !at_rest && !braking && !separating)
                    || mover.path_state != PathState::None)
            {
                return fail("I7", format!("has no goal but isn't at rest: {mover:?}"));
            }
            // Only a unit without a goal and without an order takes a sidestep (design change 0023).
            let idle = o.orders.as_ref().is_some_and(|orders| orders.is_idle());
            if mover.sidestep.is_some() && (mover.goal != MoveGoal::None || !idle) {
                return fail("I7", format!("sidesteps though it has a goal or an order: {mover:?}"));
            }
            if mover.goal != MoveGoal::None && mover.status != MoveStatus::Moving {
                return fail("I7", format!("has a goal with status {:?}", mover.status));
            }
            // I6: repaths per leg are capped, and a request doesn't wait for ever.
            if mover.repaths_left > MAX_REPATHS {
                return fail("I6", format!("has {} repaths left", mover.repaths_left));
            }
            // Only a new leg gives repaths back.
            if let Some(before) = before
                && mover.repaths_left > before.repaths_left
                && !new_leg
            {
                return fail(
                    "I6",
                    format!(
                        "got repaths back within one leg: {} to {}",
                        before.repaths_left, mover.repaths_left
                    ),
                );
            }
            if let PathState::Wanted { since, .. } = mover.path_state
                && tick.saturating_sub(since) > MAX_REQUEST_AGE
            {
                return fail("I6", format!("has waited for a path since tick {since}"));
            }
        }
        Ok(())
    }
}

/// V1: the vision phase 5f keeps up incrementally is the one a fresh build from the objects gives, every Visible
/// tile is Explored, and each player's records are those it should keep (`vision::memory`).
fn vision(sim: &Sim) -> Result<(), Violation> {
    sim.vision()
        .check(sim.map(), sim.objects(), sim.tick())
        .map_err(|(at, message)| Violation {
            invariant: "V1",
            message,
            at,
        })
}

/// K1: the grids each player plans on are those of what it knows at the end of the tick
/// (`systems/controls/behaviors.md > PathKnowledge`), exactly as a loaded save rebuilds them.
fn path_knowledge(sim: &Sim) -> Result<(), Violation> {
    sim.check_path_knowledge().map_err(|message| Violation {
        invariant: "K1",
        message,
        at: Vec2::ZERO,
    })
}

/// I1: no two units of a layer with hard collision overlap. Touching is free.
/// I1's two exceptions (design change 0106): whether the units `a` and `b` of one layer, whose boxes overlap as
/// the tick ends, may. `before` is what the checker saw of the units a tick earlier.
///
/// 1. **Gathering Drones** (`factions/colonists/units/drone.md > Gathering Drones pass through each other`):
///    both are Drones on a Gather job. The movement step saw a unit so if it is on one now, or was a tick earlier
///    and has no order now: an order ends after the movement only by leaving the unit idle (a Drop Off whose
///    drop-off was destroyed in phase 5e), while an order given in this tick began before the movement and is the
///    unit's current one. (A job that ended in the behaviours of this tick, by a Stop, for want of a drop-off or
///    with no patch left, design change 0123, reads the same for that one tick: the check can't see the phases.)
/// 2. **Units that already overlap** (`systems/units.md > UnitCollision > Units that already overlap`): the two
///    overlapped a tick earlier too. They ignore each other until they have separated, so the pair may move in
///    any way meanwhile; once apart it is held to I1 again, like every pair that never overlapped.
///
/// Everything else fails: two units that come to overlap, unless both are gathering Drones. (M3-D2) A portal's
/// energy doorway (design change 0145) takes no exception here: its unit appears at the centre only where its box is
/// clear of every unit and walks out only up to contact, so it overlaps no unit (`tests/production/portal_stalls.rs`
/// has the negative test).
fn may_overlap(a: &Object, b: &Object, before: &BTreeMap<ObjectId, Seen>) -> bool {
    let gathered = |o: &Object| {
        let idle = o.orders.as_ref().is_some_and(|orders| orders.current.is_none());
        gather::on_gather_job(o) || (idle && before.get(&o.id).is_some_and(|seen| seen.gathering))
    };
    let overlapped = || {
        let (Some(was_a), Some(was_b)) = (before.get(&a.id), before.get(&b.id)) else {
            return false;
        };
        let (Some(a_half), Some(b_half)) = (a.def().half_extents(), b.def().half_extents()) else {
            return false;
        };
        boxes_overlap(was_a.pos, a_half, was_b.pos, b_half)
    };
    (gathered(a) && gathered(b)) || overlapped()
}

/// The units whose box overlapped another unit's of their layer a tick earlier, by what the checker saw then: those
/// that may be moving off it now, if they are idle (design change 0126).
fn overlapped_before(sim: &Sim, before: &BTreeMap<ObjectId, Seen>) -> Vec<ObjectId> {
    let mut found = Vec::new();
    for domain in [Domain::Ground, Domain::Underground] {
        let mut boxes: Vec<(f32, Vec2, Vec2, ObjectId)> = sim
            .objects()
            .filter_map(|o| {
                let def = o.def();
                let half = def.half_extents()?;
                let pos = before.get(&o.id)?.pos;
                (def.unit.as_ref()?.base_def().domain == domain).then_some((pos.x - half.x, pos, half, o.id))
            })
            .collect();
        boxes.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.3.cmp(&b.3)));
        for (i, &(_, a, a_half, a_id)) in boxes.iter().enumerate() {
            for &(low, b, b_half, b_id) in &boxes[i + 1..] {
                if low >= a.x + a_half.x {
                    break;
                }
                if boxes_overlap(a, a_half, b, b_half) {
                    found.extend([a_id, b_id]);
                }
            }
        }
    }
    found.sort();
    found.dedup();
    found
}

/// I1: no two units of a layer overlap, but for the two exceptions of [`may_overlap`].
fn no_overlaps(sim: &Sim, before: &BTreeMap<ObjectId, Seen>) -> Result<(), Violation> {
    for domain in [Domain::Ground, Domain::Underground] {
        // (low x, position, half extents, the unit), by low x: a unit only meets those starting before its high x.
        let mut boxes: Vec<(f32, Vec2, Vec2, &Object)> = sim
            .objects()
            .filter_map(|o| {
                let def = o.def();
                let half = def.half_extents()?;
                (def.unit.as_ref()?.base_def().domain == domain).then_some((o.pos.x - half.x, o.pos, half, o))
            })
            .collect();
        boxes.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.3.id.cmp(&b.3.id)));
        for (i, &(_, a, a_half, a_unit)) in boxes.iter().enumerate() {
            for &(low, b, b_half, b_unit) in &boxes[i + 1..] {
                if low >= a.x + a_half.x {
                    break;
                }
                if boxes_overlap(a, a_half, b, b_half) && !may_overlap(a_unit, b_unit, before) {
                    return Err(Violation {
                        invariant: "I1",
                        message: format!("#{} at {a} and #{} at {b} overlap", a_unit.id.0, b_unit.id.0),
                        at: (a + b) / 2.0,
                    });
                }
            }
        }
    }
    Ok(())
}

fn floats_finite(o: &Object) -> bool {
    let core = o.pos.is_finite()
        && o.prev_pos.is_finite()
        && o.facing.is_finite()
        && o.prev_facing.is_finite()
        && o.hp.is_none_or(f32::is_finite);
    let mover = o.mover.as_ref().is_none_or(|m| {
        let goal = match m.goal {
            MoveGoal::None => true,
            MoveGoal::Point(p) => p.is_finite(),
            MoveGoal::Object { reach, .. } => reach.is_finite(),
        };
        goal && m.velocity.is_finite()
            && m.momentum.is_finite()
            && m.exit_heading.is_none_or(Vec2::is_finite)
            && m.speed_factor.is_finite()
            && m.planned_target.is_finite()
            && m.best_remaining.is_finite()
            && m.sidestep.is_none_or(|s| s.dir.is_finite() && s.left.is_finite())
            && m.path.iter().all(|p| p.is_finite())
    });
    let orders = o.orders.as_ref().is_none_or(|orders| {
        let current = orders.current.iter().map(|active| &active.order);
        current.chain(&orders.queue).all(|order| {
            order.slot.is_none_or(Vec2::is_finite)
                && match order.kind {
                    OrderKind::Move(Target::Point(p))
                    | OrderKind::AttackMove(p)
                    | OrderKind::Patrol(p)
                    | OrderKind::AttackGround(p) => p.is_finite(),
                    OrderKind::Move(Target::Object(_)) | OrderKind::Attack(_) | OrderKind::Hold | OrderKind::Stop => {
                        true
                    }
                    OrderKind::Gather(_) | OrderKind::DropOff(_) | OrderKind::Deploy { .. } => true,
                    OrderKind::Enter(_) | OrderKind::Build { .. } | OrderKind::Assist(_) => true,
                }
        })
    });
    let rally = o
        .production
        .as_ref()
        .and_then(|production| production.rally)
        .is_none_or(|rally| rally.point().is_finite());
    core && mover && orders && rally
}
