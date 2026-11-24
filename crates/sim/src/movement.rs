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
//! - [`collide`]: one unit's move by the rules of `docs/movement.md > 6.3`: lane seeking among the units it has to
//!   pass (oncoming ones, and idle groups that make way), then the rules after the sweep.
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
    /// (G) Oncoming units looked at by lane seeking (`collide::World::lane`).
    pub lane_tests: u32,
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

/// The most a unit turns in one tick, radians: its TurnRate (`systems/combat.md >
/// LocomotionOrientationConstraints[TurnRateMovement]`), without limit for the movement models whose turn rate
/// depends on speed (TODO(movement models): not built).
pub(crate) fn turn_rate(movement: &MovementAttrs) -> f32 {
    movement.turn_rate().unwrap_or(f32::INFINITY)
}

/// (G) The one turn budget of a tick (`docs/movement.md > 6.3`): a unit turns at most its [`turn_rate`] per tick in
/// all. Phase 5d notes here the angle it turned each unit, and phase 5e's Aiming turns a unit only by what is left
/// of the budget (`combat::step`). By ascending id; a unit that didn't turn isn't listed.
///
/// Transient, not state: 5d writes it afresh and 5e reads it within the same tick, so it never outlives the tick. A
/// save between two ticks has none of it to keep, and the state hash nothing to cover: it follows from the state the
/// tick began with and the tick's inputs, which the hash and the replay already fix.
#[derive(Debug, Default)]
pub(crate) struct Turns(Vec<(ObjectId, f32)>);

impl Turns {
    /// The angle phase 5d turned unit `id` in this tick, radians.
    pub(crate) fn angle(&self, id: ObjectId) -> f32 {
        self.0
            .binary_search_by_key(&id, |&(id, _)| id)
            .map_or(0.0, |i| self.0[i].1)
    }
}

/// The scratch of phase 5d (`docs/movement.md > 3`): rebuilt from the objects every tick. Kept between ticks only
/// to reuse its allocations, except [`Turns`], which phase 5e of the same tick reads.
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
    /// The lanes of the oncoming units ahead of the unit being moved (`collide::World::lane`).
    lanes: Vec<collide::Lane>,
    /// The sidesteps asked of idle units this tick (`yielding`), applied when every unit has moved.
    requests: Vec<yielding::Request>,
    /// The angle each unit turned in this tick's phase 5d, for 5e.
    turns: Turns,
    counters: MoveCounters,
}

impl Scratch {
    pub(crate) fn counters(&self) -> MoveCounters {
        self.counters
    }

    pub(crate) fn turns(&self) -> &Turns {
        &self.turns
    }
}

/// `Body::heading`: the way to the unit's steering waypoint, if it has a path to follow.
fn heading_of(pos: Vec2, mover: &Mover) -> Vec2 {
    if mover.goal == MoveGoal::None || mover.path_state != PathState::Ready {
        return Vec2::ZERO;
    }
    let target = mover.path.get(usize::from(mover.next));
    target.map_or(Vec2::ZERO, |&target| target - pos)
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
        heading: heading_of(o.pos, mover),
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
/// (`yielding::apply`) and begin in the next. The angle each unit turned goes to [`Turns`], for phase 5e.
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
    scratch.turns.0.clear();
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
            let kinds = scratch.bodies[index as usize].kinds();
            scratch.layers[layer].insert(index, o.pos, half, kinds);
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
            lanes: &mut scratch.lanes,
            counters: &mut scratch.counters,
        };
        let mut turned = 0.0;
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
                &mut turned,
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
        if turned > 0.0 {
            scratch.turns.0.push((o.id, turned));
        }
        // Commit: later units see this one where it is now.
        let mover = o.mover.as_ref().expect("units have movers");
        let before = scratch.bodies[me].kinds();
        scratch.bodies[me] = body_of(o, mover, unit, half);
        let kinds = scratch.bodies[me].kinds();
        if let Some(l) = layer {
            if o.pos != from {
                scratch.layers[l].remove(me as u32, from, half);
                scratch.layers[l].insert(me as u32, o.pos, half, kinds);
            } else if kinds != before {
                scratch.layers[l].mark(o.pos, half, kinds);
            }
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
///   per tick, so its turn always completes. The facing follows the path, not a sidestep. The angle turned goes to
///   `turned`: the aim in phase 5e has only the rest of the tick's TurnRate ([`Turns`]).
/// - **Arrival**: when the goal is within one step and the way is free, the unit stands exactly on it and stops
///   with `status = Arrived`. The arrival radius is one tick of travel, so it never overshoots.
/// - **An object goal** (`chased`; `systems/controls/behaviors.md > MovingToObject`): the unit has `Arrived` as soon
///   as the gap between its shape and the object's is within the goal's `reach`. Its path leads to where the
///   object was when it was planned (phase 5c keeps it up to date, `paths::chase_check`): at its end the unit has
///   `Arrived` too if the object is still there (it stands on the reachable point nearest to it, design change
///   0025), and plans again if the object has moved on. An object that is gone ends the move `TargetLost`.
/// - Otherwise it moves `step = MaxSpeed × speed_factor` towards the target: a step towards a lane clear of the
///   units it has to pass, if there are some ahead ([`World::lane_step`]), else by the rules of
///   [`World::resolve`]. It asks the idle units of its owner in its way to step aside ([`yielding::ask`], and
///   [`yielding::ask_ahead`] for a group it walks through), then its progress is measured ([`stuck`]).
#[expect(
    clippy::too_many_arguments,
    reason = "the unit's state, split into the borrows the step holds"
)]
fn step_unit(
    pos: &mut Vec2,
    facing: &mut f32,
    turned: &mut f32,
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
    let mut turning = false;
    if dist > 0.0 {
        let wanted = math::wrap_angle(math::vector_angle(to));
        // Phase 5d turns a unit first in the tick: the whole budget is left.
        let turn = math::turn_toward(facing, wanted, turn_rate(movement));
        *turned = turn.angle;
        if !turn.done {
            // Turning in place for the whole tick.
            stand(mover);
            mover.orientation = Orientation::Turning(target);
            return false;
        }
        turning = turn.angle != 0.0;
    }
    let arriving = usize::from(mover.next) == last_index && dist <= step;
    let d = if dist > step { to * (step / dist) } else { to };
    touched.clear();
    // (G) First a step towards a lane clear of the units it has to pass, if there are some ahead; else the rules
    // after the sweep (`docs/movement.md > 6.3`). The idle units it is about to walk into are asked to part.
    let (shift, lane) = if arriving {
        (None, None)
    } else {
        // The end of its way, if it is steering at it.
        let to_goal = if usize::from(mover.next) == last_index {
            dist
        } else {
            f32::INFINITY
        };
        world.lane_step(*pos, d, to_goal, chased.map(|chased| chased.id), mover, touched)
    };
    if let Some(shift) = shift {
        yielding::ask_ahead(world, d, shift, mover, requests);
    }
    let moved = lane.unwrap_or_else(|| {
        touched.clear();
        world.resolve(*pos, d, arriving.then_some(target), mover, tick, touched)
    });
    let from = *pos;
    // The unit it heads for isn't in its way: it is where the unit is going.
    let blocked_by_goal = chased
        .zip(moved.blocker)
        .is_some_and(|(chased, blocker)| world.bodies[blocker].id == chased.id);
    if !blocked_by_goal {
        yielding::ask(world, from, d, &moved, shift.is_some(), mover, requests);
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
    mover.orientation = if turning {
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
