//! Phase 5d, the one movement pipeline — `docs/movement.md > 4`, `> 6`. The only code that writes a unit's `pos`
//! (besides a debug teleport).
//!
//! Units move one after another in id order. Each steers at its path's steering waypoint, sweeps its box against
//! the tiles and the other units of its layer, and commits a position that overlaps nothing; the spatial grid is
//! updated at each commit, so later units see earlier ones where they now are. "No overlap" therefore holds by
//! construction, and the id order makes it deterministic (`docs/movement.md > 4`).
//!
//! - [`space`]: the spatial grids (one per collision layer), the box sweep, and [`space::blocks`], the single
//!   filter for collision exceptions.
//! - [`collide`]: one unit's move by the rules of `docs/movement.md > 6.3`.
//! - [`stuck`]: progress, stuck detection and recovery (`docs/movement.md > 6.5`).
//! - [`yielding`]: an idle unit's sidestep for a blocked unit of its owner (`docs/movement.md > 6.6`).
//!
//! **Object goals** (`MoveGoal::Object`): the step looks up the object each such unit heads for
//! ([`Chased`]). The unit has arrived once the gap between the two shapes is within the goal's `reach`, or at the
//! end of its path when the object hasn't moved since it was planned (the nearest point it can reach); the
//! chase's progress, the occupied-target rule and queueing behind attackers are in [`stuck`].
//!
//! Hooks for later slices: crushing is an exception in [`space::blocks`]; other movement models replace the
//! "desired direction → facing and velocity" part of [`step_unit`].

mod collide;
mod space;
mod stuck;
mod yielding;

use std::collections::BTreeMap;

use glam::Vec2;

use crate::combat;
use crate::defs::{Domain, MovementAttrs, UnitDef};
use crate::events::SimEvent;
use crate::ids::ObjectId;
use crate::map::Map;
use crate::math;
use crate::mover::{Locomotion, MoveGoal, MoveStatus, Mover, Orientation, PathState};
use crate::nav::{self, Nav};
use crate::object::Object;
use crate::shape::Shape;
use crate::units::Tick;

use collide::World;
use space::{Body, Layer};

/// The work of one movement phase: output for tests and the perf overlay, never part of the state or its hash
/// (`docs/movement.md > 4`).
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct MoveCounters {
    /// Units with a goal and a path that the step moved, or tried to.
    pub steered: u32,
    /// Box sweeps against the units of a layer.
    pub unit_sweeps: u32,
    /// Box-against-box tests inside those sweeps.
    pub pair_tests: u32,
    /// Box sweeps against the tiles.
    pub tile_sweeps: u32,
}

/// The collision layers with hard collision (`docs/movement.md > 6.1`): ground and underground. Air units have
/// none: they collide with nothing (their soft separation isn't M1).
const LAYERS: usize = 2;

fn layer_of(domain: Domain) -> Option<usize> {
    match domain {
        Domain::Ground => Some(0),
        Domain::Underground => Some(1),
        Domain::Air => None,
    }
}

/// The object a unit heads for (`MoveGoal::Object`), as phase 5d finds it this tick.
#[derive(Copy, Clone, Debug)]
pub(crate) struct Chased {
    pub id: ObjectId,
    /// Its centre: what paths to it are planned to (`Mover::planned_target`).
    pub pos: Vec2,
    /// Reach and the occupied-target rule are measured to its edge (`shape`, design changes 0031–0033).
    pub shape: Shape,
}

/// The scratch of phase 5d (`docs/movement.md > 3`): rebuilt from the objects every tick. Kept between ticks only
/// to reuse its allocations.
#[derive(Debug, Default)]
pub(crate) struct Scratch {
    /// Every unit, in id order.
    bodies: Vec<Body>,
    /// For each of `bodies`: the object it heads for, if its goal is one that exists.
    chased: Vec<Option<Chased>>,
    /// For each of `bodies`: the target its attack engages (`AttackState::target`), for queueing behind attackers
    /// (`stuck`).
    engaged: Vec<Option<ObjectId>>,
    layers: [Layer; LAYERS],
    /// Units in the way of the unit being moved.
    touched: Vec<usize>,
    /// The sidesteps asked of idle units this tick (`yielding`), applied when every unit has moved.
    requests: Vec<yielding::Request>,
    counters: MoveCounters,
}

impl Scratch {
    pub(crate) fn counters(&self) -> MoveCounters {
        self.counters
    }
}

/// A unit as the others see it this tick.
fn body_of(o: &Object, mover: &Mover, unit: &UnitDef, half: Vec2) -> Body {
    // Also while it waits for a path: a unit that just planned again is still on its way.
    let on_its_way = mover.goal != MoveGoal::None;
    Body {
        id: o.id,
        owner: o.owner,
        pos: o.pos,
        half,
        velocity: mover.velocity,
        stationary: mover.is_stationary(),
        progress: on_its_way.then_some(mover.last_progress),
        arrived_with: mover.group.filter(|_| mover.status == MoveStatus::Arrived),
        yields: yielding::can_yield(o, mover, unit),
    }
}

/// A unit that takes part in movement: its definition and half extents.
fn unit_of(o: &Object) -> Option<(&'static UnitDef, Vec2)> {
    let def = o.kind.def();
    o.mover.as_ref()?;
    Some((def.unit.as_ref()?, def.half_extents()?))
}

/// Phase 5d (`docs/movement.md > 4`): builds the spatial grids from **all** units, this tick's spawns included
/// (they are obstacles, though not moved), then moves every unit before `first_new`, in id order, updating the
/// grid at each commit. A unit whose attack holds its base (`combat::may_move`, `combat::may_turn`) stands and
/// keeps its goal. An idle unit asked to make way takes its sidestep in its turn, like any move
/// (`yielding::sidestep`); the sidesteps asked for in this tick are noted when every unit has moved
/// (`yielding::apply`) and begin in the next.
pub(crate) fn step(
    objects: &mut BTreeMap<ObjectId, Object>,
    first_new: ObjectId,
    map: &Map,
    nav: &Nav,
    scratch: &mut Scratch,
    tick: Tick,
    events: &mut Vec<SimEvent>,
) {
    scratch.counters = MoveCounters::default();
    scratch.bodies.clear();
    scratch.chased.clear();
    scratch.engaged.clear();
    scratch.requests.clear();
    for layer in &mut scratch.layers {
        layer.reset(map.width(), map.height());
    }
    for o in objects.values() {
        let Some((unit, half)) = unit_of(o) else {
            continue;
        };
        let mover = o.mover.as_ref().expect("units have movers");
        let index = scratch.bodies.len() as u32;
        scratch.bodies.push(body_of(o, mover, unit, half));
        scratch.chased.push(match mover.goal {
            MoveGoal::Object { id, .. } => objects.get(&id).map(|target| Chased {
                id,
                pos: target.pos,
                shape: Shape::of(target),
            }),
            MoveGoal::None | MoveGoal::Point(_) => None,
        });
        scratch.engaged.push(o.attack.as_ref().and_then(|attack| attack.target));
        if let Some(layer) = layer_of(unit.base_def().domain) {
            scratch.layers[layer].insert(index, o.pos, half);
        }
    }

    // The bodies were pushed in this same order.
    let mut index = 0;
    for o in objects.values_mut() {
        let Some((unit, half)) = unit_of(o) else {
            continue;
        };
        let me = index;
        index += 1;
        if o.id >= first_new {
            continue;
        }
        // `systems/combat.md > UnitBaseSource`: while its attack aims, fires or cools down, a unit's base neither
        // moves nor turns (the attack turns it while aiming, in phase 5e).
        let held = !combat::may_move(o) || !combat::may_turn(o);
        let mover = o.mover.as_mut().expect("units have movers");
        let base = unit.base_def();
        let layer = layer_of(base.domain);
        let from = o.pos;
        let mut world = World {
            grid: base
                .nav_class()
                .and_then(|class| nav.grid(class, nav::size_class(half * 2.0))),
            layer: layer.map(|l| &scratch.layers[l]),
            bodies: &scratch.bodies,
            me,
            counters: &mut scratch.counters,
        };
        let failed = if held {
            // A unit in an attack sequence is busy: it doesn't yield (design change 0023).
            mover.sidestep = None;
            wait(mover, tick);
            false
        } else if mover.goal == MoveGoal::None && mover.sidestep.is_some() {
            let speed = unit.movement.max_speed() * mover.speed_factor;
            yielding::sidestep(
                &mut o.pos,
                mover,
                speed,
                &mut world,
                &mut scratch.touched,
                &mut scratch.requests,
            );
            false
        } else {
            step_unit(
                &mut o.pos,
                &mut o.facing,
                mover,
                &unit.movement,
                &mut world,
                &mut scratch.touched,
                &mut scratch.requests,
                scratch.chased[me].as_ref(),
                &scratch.engaged,
                tick,
            )
        };
        if failed {
            events.push(SimEvent::MoveFailed { unit: o.id });
        }
        // Commit: later units see this one where it is now.
        let mover = o.mover.as_ref().expect("units have movers");
        scratch.bodies[me] = body_of(o, mover, unit, half);
        if o.pos != from
            && let Some(l) = layer
        {
            scratch.layers[l].remove(me as u32, from, half);
            scratch.layers[l].insert(me as u32, o.pos, half);
        }
    }
    yielding::apply(objects, &scratch.bodies, &mut scratch.requests);
}

/// The unit stands this tick.
fn stand(mover: &mut Mover) {
    mover.velocity = Vec2::ZERO;
    mover.locomotion = Locomotion::Stationary;
    mover.orientation = Orientation::Maintaining;
}

/// The unit stands this tick but keeps its goal and path: it waits for its path, or for its attack to let it go.
/// The stuck clock doesn't run meanwhile.
fn wait(mover: &mut Mover, tick: Tick) {
    stand(mover);
    mover.blocked_ticks = 0;
    if mover.goal != MoveGoal::None {
        mover.last_progress = mover.last_progress.saturating_add(1).min(tick);
    }
}

/// One unit's tick (`docs/movement.md > 6.2`, `> 6.3`). Returns whether its move failed this tick.
///
/// - Without a goal, or while waiting for its path, the unit stands still (`> 5.5`). Only its own sidestep moves
///   an idle unit (`yielding`); nothing pushes it.
/// - **Steering target**: the path's waypoint `Mover::next`. A waypoint within one step is passed without being
///   touched; the staggered check in phase 5c moves on to the farthest waypoint in sight.
/// - **Facing** (`systems/combat.md > LocomotionOrientationConstraints[TurnRateMovement]`): the unit turns towards
///   the steering target, at most TurnRate per tick, and moves only once it faces it. Every M1 unit turns 180°
///   per tick, so its turn always completes. The facing follows the path, not a sidestep.
/// - **Arrival**: when the goal is within one step and the way is free, the unit stands exactly on it and stops
///   with `status = Arrived`. The arrival radius is one tick of travel, so it never overshoots.
/// - **An object goal** (`chased`; `systems/controls/behaviors.md > MovingToObject`): the unit has `Arrived` as soon
///   as the gap between its shape and the object's is within the goal's `reach`. Its path leads to where the
///   object was when it was planned (phase 5c keeps it up to date, `paths::chase_check`): at its end the unit has
///   `Arrived` too if the object is still there (it stands on the reachable point nearest to it, design change
///   0025), and plans again if the object has moved on. An object that is gone ends the move `TargetLost`.
/// - Otherwise it moves `step = MaxSpeed × speed_factor` towards the target by the rules of
///   [`World::resolve`], asks an idle unit of its owner in its way to step aside ([`yielding::ask`]), then its
///   progress is measured ([`stuck`]).
#[expect(
    clippy::too_many_arguments,
    reason = "the unit's state, split into the borrows the step holds"
)]
fn step_unit(
    pos: &mut Vec2,
    facing: &mut f32,
    mover: &mut Mover,
    movement: &MovementAttrs,
    world: &mut World,
    touched: &mut Vec<usize>,
    requests: &mut Vec<yielding::Request>,
    chased: Option<&Chased>,
    engaged: &[Option<ObjectId>],
    tick: Tick,
) -> bool {
    if let MoveGoal::Object { reach, .. } = mover.goal {
        let Some(chased) = chased else {
            mover.finish(MoveStatus::TargetLost);
            return false;
        };
        if Shape::rect(*pos, world.body().half).gap(&chased.shape) <= reach {
            mover.finish(MoveStatus::Arrived);
            return false;
        }
    }
    if mover.goal == MoveGoal::None || mover.path_state != PathState::Ready {
        wait(mover, tick);
        return false;
    }
    world.counters.steered += 1;
    let step = movement.max_speed() * mover.speed_factor;
    let last_index = mover.path.len().saturating_sub(1);
    while usize::from(mover.next) < last_index && pos.distance(mover.path[usize::from(mover.next)]) <= step {
        mover.next += 1;
    }
    let Some(&target) = mover.path.get(usize::from(mover.next)) else {
        // An empty path: the unit is where it ends.
        mover.finish(MoveStatus::Arrived);
        return false;
    };
    let to = target - *pos;
    let dist = to.length();
    // TODO(movement models): only TurnRateMovement (every M1 unit) is modelled; the others turn without limit here.
    let mut turned = false;
    if dist > 0.0 {
        let turn_rate = movement.turn_rate().unwrap_or(math::PI);
        let wanted = math::wrap_angle(math::vector_angle(to));
        let turn = math::wrap_angle(wanted - *facing);
        if turn.abs() > turn_rate {
            // Turning in place for the whole tick.
            *facing = math::wrap_angle(*facing + turn_rate.copysign(turn));
            stand(mover);
            mover.orientation = Orientation::Turning(target);
            return false;
        }
        if turn != 0.0 {
            *facing = wanted;
            turned = true;
        }
    }
    let arriving = usize::from(mover.next) == last_index && dist <= step;
    let d = if dist > step { to * (step / dist) } else { to };
    touched.clear();
    let moved = world.resolve(*pos, d, arriving.then_some(target), mover, tick, touched);
    let from = *pos;
    // The unit it heads for isn't in its way: it is where the unit is going.
    let blocked_by_goal = chased
        .zip(moved.blocker)
        .is_some_and(|(chased, blocker)| world.bodies[blocker].id == chased.id);
    if !blocked_by_goal {
        yielding::ask(world, from, d, &moved, mover, requests);
    }
    *pos = moved.pos;
    if arriving && moved.pos == target {
        match chased {
            // The object moved on while the unit walked to where it was: plan again (not a counted repath).
            Some(chased) if chased.pos != mover.planned_target => {
                stand(mover);
                mover.repath(tick);
            }
            _ => mover.finish(MoveStatus::Arrived),
        }
        return false;
    }
    mover.velocity = *pos - from;
    mover.locomotion = if mover.velocity == Vec2::ZERO {
        Locomotion::Stationary
    } else {
        Locomotion::Moving
    };
    mover.orientation = if turned {
        Orientation::Turning(target)
    } else {
        Orientation::Maintaining
    };
    mover.blocked_ticks = if moved.forward {
        0
    } else {
        mover.blocked_ticks.saturating_add(1)
    };
    stuck::after_move(*pos, mover, step, &moved, world, chased, engaged, tick)
}

#[cfg(test)]
mod tests;
