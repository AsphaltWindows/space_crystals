//! Phase 5d, the one movement pipeline — `docs/movement.md > 4`, `> 6`. The only code that writes a unit's `pos`
//! (besides a debug teleport).
//!
//! Units move one after another in id order. Each steers at its path's steering waypoint, sweeps its box against
//! the tiles and the other units of its layer, and commits a position that overlaps nothing that blocks it
//! ([`space::blocks`]: every unit but the two exceptions of design change 0106); the spatial grid is updated at
//! each commit, so later units see earlier ones where they now are. "No overlap" therefore holds by construction,
//! and the id order makes it deterministic (`docs/movement.md > 4`).
//!
//! - [`space`]: the spatial grids (one per collision layer), the box sweep, and [`space::blocks`], the single
//!   filter for collision exceptions: gathering Drones, and units that already overlap (design change 0106).
//! - [`collide`]: one unit's move by the rules of `docs/movement.md > 6.3`: lane seeking among the units it has to
//!   pass (oncoming ones, and idle groups that make way), then the rules after the sweep.
//! - [`stuck`]: progress, stuck detection and recovery (`docs/movement.md > 6.5`).
//! - [`yielding`]: an idle unit's sidestep for a blocked unit of its owner (`docs/movement.md > 6.6`).
//! - [`drag`]: DragMovement, the model of the units with momentum (slice M3-G); [`follow`]: one unit's tick along
//!   its path, shared by every model.
//! - [`separate`]: an idle unit that overlaps another moves off it (design change 0126).
//! - [`doorway`]: a released unit walking out of its producer's exit doorway (design change 0080), the one case in
//!   which a unit's box overlaps a structure: `collide::World::door` is where the tiles of that building stop
//!   blocking it. (M3-D2) Out of a portal's energy doorway (design change 0145) the unit walks straight from the
//!   centre along its heading, in its own movement model, and makes no other move until it is clear
//!   (`doorway::emerge`, `collide::World::straight`).
//!
//! **Object goals** (`MoveGoal::Object`): the step looks up the object each such unit heads for
//! ([`Chased`]). The unit has arrived once the gap between the two shapes is within the goal's `reach`, or at the
//! end of its path when the object hasn't moved since it was planned (the nearest point it can reach); the
//! chase's progress, the occupied-target rule and queueing behind attackers are in [`stuck`].
//!
//! Hooks for later slices: crushing is an exception in [`space::blocks`]; other movement models replace the
//! "desired direction → facing and velocity" part of a unit's tick, as [`drag`] does in [`follow`].

mod collide;
mod doorway;
mod drag;
mod follow;
mod round;
mod separate;
mod space;
mod stuck;
mod yielding;

use std::collections::BTreeMap;

use glam::Vec2;

use crate::combat;
use crate::defs::{Domain, MovementAttrs, UnitDef};
use crate::events::SimEvent;
use crate::ids::{ObjectId, PlayerId};
use crate::map::Map;
#[cfg(test)]
use crate::math;
use crate::mover::{Locomotion, MoveGoal, MoveStatus, Mover, Orientation, PathState};
use crate::nav::{self, Nav};
use crate::object::Object;
use crate::orders::gather;
use crate::popup;
use crate::production::ExitDoor;
use crate::shape::Shape;
use crate::units::{SU_PER_GU, Tick};

use crate::units::TilePos;
use collide::World;
use drag::Drag;
#[cfg(test)]
use follow::step_unit;
use follow::step_unit_on;
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
    /// For each of `bodies`: the exit doorway it is still in (`Mover::exit`, design changes 0080 and 0145).
    doors: Vec<Option<ExitDoor>>,
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
    /// (M4-S2) The footprints of the lowered pop-up turrets that want to rise and have a unit over them
    /// (`popup::is_blocked`), each with its owner: their owners' idle units there are asked to step off
    /// (`yielding::make_way`).
    rising: Vec<(Option<PlayerId>, Vec2, Vec2)>,
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
        gathers: gather::on_gather_job(o),
        leads: stuck::leads(mover),
        moved: mover.last_progress,
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
/// (`yielding::apply`) and begin in the next. (M4-S2) A lowered pop-up turret that wants to rise asks its owner's idle
/// units over its footprint the same way (`yielding::make_way`). The angle each unit turned goes to [`Turns`], for phase 5e.
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
    scratch.doors.clear();
    scratch.requests.clear();
    scratch.rising.clear();
    scratch.turns.0.clear();
    for layer in &mut scratch.layers {
        layer.reset(map.width(), map.height());
    }
    for o in objects.values() {
        let Some((unit, half)) = unit_of(o) else {
            // (M4-S2) A pop-up turret that a unit keeps from rising (`popup`).
            if popup::is_blocked(o)
                && let Some(placement) = &o.placement
            {
                let half = Vec2::new(placement.w as f32, placement.h as f32) * (SU_PER_GU / 2.0);
                scratch.rising.push((o.owner, o.pos, half));
            }
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
        // Only while its box still overlaps the footprint (a teleport may have taken it away).
        let door = mover.exit.and_then(|building| ExitDoor::of(objects.get(&building)?));
        scratch.doors.push(door.filter(|door| door.holds(o.pos, half)));
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
        let door = scratch.doors[me];
        if door.is_none() {
            mover.exit = None;
            mover.exit_heading = None;
        }
        let mut world = World {
            grid: base
                .nav_class()
                .and_then(|class| nav.grid(class, nav::size_class(half * 2.0))),
            layer: layer.map(|l| &scratch.layers[l]),
            bodies: &scratch.bodies,
            me,
            contact: base.contact_step,
            door,
            lanes: &mut scratch.lanes,
            counters: &mut scratch.counters,
        };
        let mut turned = 0.0;
        // (M3-G) A unit with momentum brakes it off before anything else (`drag`), and its drag is doubled on
        // Rugged terrain: the tile under its centre (`systems/units.md > Which tile counts`).
        let rugged = matches!(unit.movement, MovementAttrs::Drag { .. })
            && map
                .tile_preset(TilePos::containing(o.pos))
                .is_some_and(|preset| preset.rugged);
        let coasting = mover.momentum != Vec2::ZERO;
        let failed = if held {
            // A unit in an attack sequence is busy: it doesn't yield (design change 0023).
            mover.sidestep = None;
            wait(mover, tick);
            if let Some(model) = Drag::of(&unit.movement, mover.speed_factor, rugged) {
                model.coast(&mut o.pos, o.facing, mover, &mut world, &mut scratch.touched);
            }
            false
        } else if let Some(ExitDoor::Centre(portal)) = door {
            // (M3-D2) Out of a portal's energy doorway the unit walks straight along its heading, with or without a
            // goal, and makes no other move until it is clear (design change 0145).
            doorway::emerge(
                &mut o.pos,
                &mut o.facing,
                &mut turned,
                mover,
                &unit.movement,
                rugged,
                &portal,
                &mut world,
                &mut scratch.touched,
                &mut scratch.requests,
                scratch.chased[me].as_ref(),
                &scratch.engaged,
                tick,
            )
        } else if scratch.bodies[me].yields
            && !coasting
            && door.is_none()
            && separate::step(
                &mut o.pos,
                mover,
                unit.movement.max_speed() * mover.speed_factor,
                &mut world,
                &mut scratch.touched,
            )
        {
            // An idle unit that overlaps another moves off it (design change 0126).
            false
        } else if mover.goal == MoveGoal::None && mover.sidestep.is_some() && !coasting {
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
        } else if let Some(ExitDoor::Side(door)) = door.filter(|_| mover.goal == MoveGoal::None && !coasting) {
            doorway::leave(
                &mut o.pos,
                &mut o.facing,
                &mut turned,
                mover,
                &unit.movement,
                &door,
                &mut world,
                &mut scratch.touched,
                &mut scratch.requests,
            );
            false
        } else {
            step_unit_on(
                &mut o.pos,
                &mut o.facing,
                &mut turned,
                mover,
                &unit.movement,
                rugged,
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
        // Out of the doorway: from now on the building blocks the unit like everything else, for good (design
        // change 0080). One that walked out without a goal stands there.
        let mover = o.mover.as_mut().expect("units have movers");
        if door.is_some_and(|door| !door.holds(o.pos, half)) {
            mover.exit = None;
            mover.exit_heading = None;
            if mover.goal == MoveGoal::None && mover.sidestep.is_none() && mover.momentum == Vec2::ZERO {
                stand(mover);
            }
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
    // (M4-S2) After the movers' own requests, which come first for a unit asked twice.
    for &(owner, centre, half) in &scratch.rising {
        let ground = &scratch.layers[0];
        yielding::make_way(owner, centre, half, ground, &scratch.bodies, &mut scratch.requests);
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

#[cfg(test)]
mod tests;
