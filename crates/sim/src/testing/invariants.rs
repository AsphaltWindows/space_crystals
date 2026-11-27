//! The movement invariants I1–I7 of `docs/movement.md > 8` (I1 with its two exceptions of design change 0106, the
//! evacuation's of design change 0349 and the Garrison's door's, slice M5-N2: `overlaps`; I2 with its one exception,
//! the exit doorway of design change 0080, and of a portal design change 0145's energy doorway), checked from outside
//! through the sim's public state,
//! the vision invariant V1 (the vision kept up tick by tick is the one a fresh build gives, and every Visible tile
//! is Explored: `vision`), the path knowledge invariant K1 (each player's planning grids are those a fresh build
//! from the state gives: `paths`), the projectile invariant P1 (every projectile lands by its arrival tick:
//! `projectiles`), the operation invariant O1 (a Tunnel's operation and its site name each other: `operations`), the
//! reservation invariant R1 (a building that trains the units inside it reserves them as its queue has them:
//! `reservations`, slice M5-F2), (M5-M1) the hit-point invariant H1 (every object's HP is above 0 and at most its MaxHP
//! with its Augmentations, its Shields within 0 and its MaxShields: `hit_points`), the bank invariant B1 (no bank goes
//! below 0 or past its top: `banks`) and the augmentation invariant A1 (only a kind that can be augmented has an
//! Augmentation: `augmentations`), H1 and A1 over the held units too, the energy invariant E1 (an object has energy
//! exactly when its type has a pool, and at most its maximum, held units too: `energy`, slice M5-N4a), (M5-N3) the
//! research invariant T1 (techs come only from completed research, each queued at one structure that offers it:
//! `research`), (M5-N4b) the cloud invariant C1 (Shroud's clouds are the channels under way, each connected:
//! `clouds`), (M5-N5) the buff invariant F1 (no unit holds two buffs of an ability, each ends within its ability's
//! duration and only a kind the ability affects holds it, held units too: `buffs`), (M5-H1) the Glider invariant G1 (a
//! Glider in flight never stops, keeps within its IdleSpeed and MaxSpeed, and counts as being on the map: `gliders`),
//! (M5-V2) the wheeled invariant W1 (a FixedTurnRadiusMovement unit turns by its speed, never in place: `momentum`),
//! and an ASCII picture of the map with its units for readable failures.
//!
//! [`super::TestSim`] checks them after every tick (on by default). A test that steps a bare [`Sim`] keeps its own
//! [`Invariants`] and calls [`Invariants::check`] after each step, as the golden games do (slice M5-M1); the fuzz's
//! games call [`Invariants::check_rules`], the invariants of the game's rules alone.

mod augmentations;
mod banks;
mod buffs;
mod clouds;
mod energy;
mod gliders;
mod hit_points;
mod momentum;
mod operations;
mod overlaps;
mod picture;
mod projectiles;
mod research;
mod reservations;

use std::collections::BTreeMap;

use glam::Vec2;

use crate::augmentation::Research;
use crate::command::{Command, DebugCommand, TickInputs};
use crate::events::SimEvent;
use crate::ids::ObjectId;
use crate::mover::{Locomotion, MoveGoal, MoveStatus, Mover, PathState};
use crate::nav;
use crate::object::Object;
use crate::orders::{OrderKind, Target, gather};
use crate::production::ExitDoor;
use crate::shape::Shape;
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
    /// "I1" … "I7", "V1", "K1", "P1", "O1", "R1", "H1", "B1", "A1", "E1", "T1", "C1", "F1", "G1", "W1".
    pub invariant: &'static str,
    pub message: String,
    /// Where on the map to look.
    pub at: Vec2,
}

/// What the checker remembers of a unit from the tick before.
#[derive(Clone, Debug)]
struct Seen {
    pos: Vec2,
    /// (M5-V1) Its facing, which a Silhouette that turns lies along (`shape::Shape::silhouette`).
    facing: f32,
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
    /// (M5-N3) Its MaxSpeed at its owner's research levels (`Object::max_speed`), for I3 in a tick that changes them.
    max_speed: f32,
}

/// Checks the invariants between ticks (module docs). It remembers each unit's state, and each player's banks, of the
/// tick before, so check after **every** step.
#[derive(Clone, Debug)]
pub struct Invariants {
    seen: BTreeMap<ObjectId, Seen>,
    /// (M5-M1) Each player's banks of the tick before, for B1.
    books: banks::Books,
}

fn leg_of(o: &Object) -> Option<Tick> {
    o.orders.as_ref()?.current.as_ref().map(|active| active.since)
}

fn seen(o: &Object, mover: &Mover, research: Research) -> Seen {
    Seen {
        pos: o.pos,
        facing: o.facing,
        had_goal: mover.goal != MoveGoal::None,
        sidestepping: mover.sidestep.is_some(),
        repaths_left: mover.repaths_left,
        exit: mover.exit,
        gathering: gather::on_gather_job(o),
        momentum: mover.momentum,
        max_speed: o.max_speed(research),
    }
}

impl Invariants {
    /// Starts from the sim's current state.
    pub fn new(sim: &Sim) -> Self {
        let research = sim.research();
        let seen = sim
            .objects()
            .filter_map(|o| Some((o.id, seen(o, o.mover.as_ref()?, research))))
            .collect();
        Self {
            seen,
            books: banks::Books::of(sim),
        }
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
        // (M5-F3) The units evacuated in this tick (design change 0349), which came onto the map overlapping.
        let evacuated: Vec<ObjectId> = (sim.events().iter())
            .filter_map(|e| match *e {
                SimEvent::Evacuated { unit, .. } => Some(unit),
                _ => None,
            })
            .collect();
        let arrivals = overlaps::Arrivals::of(sim, &evacuated);
        let result = self
            .check_units(sim, &teleported, &evacuated)
            .and_then(|()| overlaps::no_overlaps(sim, &self.seen, &arrivals))
            .and_then(|()| vision(sim))
            .and_then(|()| path_knowledge(sim))
            .and_then(|()| self.rules(sim, inputs));
        *self = Self::new(sim);
        result
    }

    /// (M5-M1) Of [`Invariants::check`]'s invariants, those of the game's rules alone: P1, O1, R1, H1, B1, A1, E1
    /// (slice M5-N4a), T1 (slice M5-N3), C1 (slice M5-N4b), F1 (slice M5-N5) and G1 (slice M5-H1), which hold whatever
    /// the input and cost little. Not the movement invariants, whose I6 bounds how far a scenario may fall behind, nor
    /// V1 and K1, which rebuild every player's vision and planning grids and make most of a check's cost. For a harness
    /// of many games of random input:
    /// `tests/robustness.rs`, whose seed sweep V1 and K1 would take from 5 s to 26 s.
    pub fn check_rules(&mut self, sim: &Sim, inputs: &TickInputs) -> Result<(), Violation> {
        let result = self.rules(sim, inputs);
        *self = Self::new(sim);
        result
    }

    fn rules(&self, sim: &Sim, inputs: &TickInputs) -> Result<(), Violation> {
        projectiles::projectiles(sim)
            .and_then(|()| operations::operations(sim))
            .and_then(|()| reservations::reservations(sim))
            .and_then(|()| hit_points::hit_points(sim))
            .and_then(|()| self.books.check(sim, inputs))
            .and_then(|()| augmentations::augmentations(sim))
            .and_then(|()| energy::energy(sim))
            .and_then(|()| research::research(sim))
            .and_then(|()| clouds::clouds(sim))
            .and_then(|()| buffs::buffs(sim))
            .and_then(|()| gliders::gliders(sim, &self.seen))
    }

    fn check_units(&self, sim: &Sim, teleported: &[ObjectId], evacuated: &[ObjectId]) -> Result<(), Violation> {
        let tick = sim.tick();
        let overlapped = overlaps::overlapped_before(sim, &self.seen);
        // (M5-N3) The research levels changed after the movement step of this tick: a level completed in the economy
        // phase (`SimEvent::ResearchCompleted`). The units moved at the MaxSpeeds of the levels before (I3). A defeat
        // changes none: the units turned neutral keep their research (design change 0379, `Object::kept_research`).
        let levels_changed = (sim.events().iter()).any(|e| matches!(e, SimEvent::ResearchCompleted { .. }));
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
            let (Some(unit), Some(silhouette), Some(mover)) = (&def.unit, def.silhouette(), &o.mover) else {
                continue;
            };
            // (M5-V1) Its shape: a Silhouette that turns with its facing is tested against the tiles as it lies
            // (`Shape::clear_on`), in a doorway by its bounding box (`Shape::bound_half`), as the movement step tests
            // it.
            let shape = Shape::of(o);
            let half = shape.bound_half();
            let before = self.seen.get(&o.id);
            // I2: a ground box overlaps no blocked tile and stays on the map. The one exception (design change
            // 0080, `core/entities.md > Structure Type > Exit Doorway`): a unit that a producer released, while
            // its box still overlaps that building's footprint (`Mover::exit`), overlaps that footprint's tiles
            // and no other blocked tile, nor ground under the footprint that it can't stand on (slice M5-Z1:
            // `ExitDoor::box_clear`, so that it stands on open ground once the building is gone); and from tick to
            // tick it only comes out (`ExitDoor::allows`). (M3-D2) Out of a portal's energy doorway (design change
            // 0145) it never stands off the open arc: its centre lies in an open direction from the portal's,
            // within a rounding tolerance (`EnergyDoorway::margin`).
            // Every unit in the game has the grid it walks on (slice M5-Z2, `paths`): a missing one would let it pass.
            let class = unit.base_def().nav_class();
            let grid = class.and_then(|class| sim.nav().grid(class, nav::size_class(silhouette)));
            if let (Some(class), None) = (class, grid) {
                return fail("I2", format!("walks on {class:?} grids, but its grid isn't built"));
            }
            match mover.exit {
                None => {
                    if grid.is_some_and(|grid| !shape.clear_on(grid)) {
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
            // I3: a unit moves at most its speed per tick, (M5-N1a) its Augmentations' included. (M3-G) For a unit
            // with momentum that is its model's maximum, where thrust and drag balance on open ground, or (M5-V2) its
            // ForwardMaxSpeed: a mode that slows the unit only weakens its thrust or lowers its top speed, and the
            // speed it has decays. (M5-N3) In a tick that changed the
            // research levels after the movement step, the larger of its MaxSpeeds before and after: it moved at the
            // one before, and moves at the one after from the next tick (`Sim::complete_research`). (M5-H1) An air unit
            // that keeps apart from others may be pushed by at most `SEPARATION_PUSH_SU` on top (`gliders`), (M5-H2)
            // its base's share of it (a Glider's a third, design change 0412; a quarter from design change 0405).
            let drag = momentum::carries(&unit.movement);
            let glides = gliders::glides(o);
            let max_speed = o.max_speed(sim.research());
            let max_speed = match before {
                Some(b) if levels_changed => b.max_speed.max(max_speed),
                _ => max_speed,
            };
            let step = max_speed * if drag { 1.0 } else { mover.speed_factor } + gliders::push_allowance(o);
            let moved = o.pos.distance(o.prev_pos);
            if moved > step * 1.0001 + 1e-3 {
                return fail("I3", format!("moved {moved} su in one tick, more than its {step}"));
            }
            // (M5-V2) W1: a wheeled unit turns by its speed, never in place (`momentum::turns_by_its_speed`).
            let wheeled = o.movement(sim.research()).zip(before);
            if let Some(message) =
                wheeled.and_then(|(movement, b)| momentum::turns_by_its_speed(&movement, o.facing, b))
            {
                return fail("W1", message);
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
            // (M3-G) A unit with momentum that still had some brakes to a stop, moving meanwhile (`momentum`): a
            // DragMovement unit and (M5-V2) a FixedTurnRadiusMovement one. (M5-H1) A Glider's is its speed, and it
            // never stops (G1). No other model has momentum, ever.
            let had_momentum = before.is_some_and(|b| b.momentum != Vec2::ZERO);
            if mover.momentum != Vec2::ZERO && !drag && !glides {
                return fail(
                    "I5/I7",
                    format!("has momentum, which its movement model has none of: {mover:?}"),
                );
            }
            let coasting = drag && had_momentum;
            // An idle unit that overlapped another moves off it (design change 0126, `movement::separate`), and so does
            // one evacuated in this tick, which may have come out in its commands phase, before the movement
            // (`may_overlap`'s exception 3).
            let separating = (overlapped.contains(&o.id) || evacuated.contains(&o.id))
                && o.orders.as_ref().is_some_and(|orders| orders.is_idle());
            // (M5-H1) A Glider flies whether or not it has a goal (G1), and an air unit near another may be pushed off
            // it (`movement::air`).
            let flying = glides || gliders::pushed(sim, o);
            if o.pos != o.prev_pos
                && mover.goal == MoveGoal::None
                && !had_goal
                && !began
                && !sidestepping
                && !leaving
                && !coasting
                && !separating
                && !flying
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
            // (M3-G) But a unit braking off its momentum (`momentum::braking`).
            let at_rest = mover.velocity == Vec2::ZERO && mover.locomotion == Locomotion::Stationary;
            let braking = drag && momentum::braking(mover, before);
            if mover.goal == MoveGoal::None
                && ((mover.sidestep.is_none()
                    && mover.exit.is_none()
                    && !at_rest
                    && !braking
                    && !separating
                    && !glides)
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

/// (M5-M1) Every object in the game, each with whether it is held: those on the map in id order, then the units held off
/// it (`Sim::held`: in the Tunnel Network or a building), which H1 and A1 check as well.
fn every_object(sim: &Sim) -> impl Iterator<Item = (&Object, bool)> + '_ {
    let held = sim.held().map(|held| (&held.object, true));
    sim.objects().map(|o| (o, false)).chain(held)
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
            && m.orbit.is_none_or(Vec2::is_finite)
            && m.strafe.is_none_or(|s| s.range.is_finite())
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
                    | OrderKind::AttackGround(p)
                    | OrderKind::Reverse(p) => p.is_finite(),
                    OrderKind::Move(Target::Object(_)) | OrderKind::Attack(_) | OrderKind::Hold | OrderKind::Stop => {
                        true
                    }
                    OrderKind::Gather(_) | OrderKind::DropOff(_) | OrderKind::Deploy { .. } => true,
                    OrderKind::Enter(_) | OrderKind::Build { .. } | OrderKind::Assist(_) | OrderKind::Repair(_) => true,
                    OrderKind::Shroud => true,
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
