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
//! - [`scratch`]: the phase's scratch, rebuilt every tick: every unit as the others see it, the objects the units
//!   head for, and the turn budget phase 5e reads ([`Turns`]).
//! - [`collide`]: one unit's move by the rules of `docs/movement.md > 6.3`: lane seeking among the units it has to
//!   pass (oncoming ones, and idle groups that make way), then the rules after the sweep.
//! - [`stuck`]: progress, stuck detection and recovery (`docs/movement.md > 6.5`).
//! - [`yielding`]: an idle unit's sidestep for a blocked unit of its owner (`docs/movement.md > 6.6`).
//! - [`model`]: the models of the units with momentum, [`drag`] (DragMovement, slice M3-G) and (M5-V2) [`wheeled`]
//!   (FixedTurnRadiusMovement: the turn by speed on a fixed radius, braking, backing up on the Reverse order, the
//!   forward loop); [`follow`]: one unit's tick along its path, shared by every model.
//! - [`separate`]: an idle unit that overlaps another moves off it (design change 0126).
//! - (M5-H1) [`glider`]: GliderMovement, the model of the units that never stop, in place of a unit's tick along its
//!   path; [`air`]: the soft separation of air units from each other, after every unit has moved.
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
//! "desired direction → facing and velocity" part of a unit's tick, as [`drag`] and [`wheeled`] do in [`follow`].

mod air;
mod collide;
mod doorway;
mod drag;
mod follow;
mod glider;
mod model;
mod round;
mod scratch;
mod separate;
mod space;
mod stuck;
mod wheeled;
mod yielding;

use std::collections::BTreeMap;

use glam::Vec2;

use crate::augmentation::Research;
use crate::combat;
use crate::defs::MovementAttrs;
use crate::events::SimEvent;
use crate::ids::ObjectId;
use crate::map::Map;
use crate::math;
use crate::mover::{Locomotion, MoveGoal, Mover, Orientation};
use crate::nav::Nav;
use crate::object::Object;
use crate::paths;
use crate::popup;
use crate::production::ExitDoor;
use crate::shape::Shape;
use crate::units::{SU_PER_GU, Tick, TilePos};

use collide::World;
#[cfg(test)]
use follow::step_unit;
use follow::step_unit_on;
use glider::Glider;
use model::Model;
pub use scratch::MoveCounters;
pub(crate) use scratch::{Chased, Scratch, Turns};
use scratch::{body_of, layer_of, oriented_of, unit_of};
#[cfg(test)]
use space::Body;

/// The most a unit turns in one tick, radians: its TurnRate (`systems/combat.md >
/// LocomotionOrientationConstraints[TurnRateMovement]`). (M5-H1) None for a Glider: its flight alone turns it, by its
/// speed (`glider`), so nothing is left of a tick's turn for an aim of its base in phase 5e. (M5-V2) None for a
/// FixedTurnRadiusMovement unit either (`systems/combat.md > LocomotionOrientationConstraints[FixedTurnRadiusMovement]`:
/// "Stationary + Turning: invalid", and Moving its turn is `currentSpeed / MinimumTurnRadius`, made by its drive in
/// phase 5d, `wheeled`). TODO(M5-V3): SpeedTurnRadiusMovement still turns without limit.
pub(crate) fn turn_rate(movement: &MovementAttrs) -> f32 {
    match movement {
        MovementAttrs::Glider { .. } | MovementAttrs::FixedTurnRadius { .. } => 0.0,
        _ => movement.turn_rate().unwrap_or(f32::INFINITY),
    }
}

/// Phase 5d (`docs/movement.md > 4`): builds the spatial grids from **all** units, this tick's spawns included
/// (they are obstacles, though not moved), then moves every unit before `first_new`, in id order, updating the
/// grid at each commit. A unit whose attack holds its base (`combat::may_move`, `combat::may_turn`) stands and
/// keeps its goal. An idle unit asked to make way takes its sidestep in its turn, like any move
/// (`yielding::sidestep`); the sidesteps asked for in this tick are noted when every unit has moved
/// (`yielding::apply`) and begin in the next. (M4-S2) A lowered pop-up turret that wants to rise asks its owner's idle
/// units over its footprint the same way (`yielding::make_way`). The angle each unit turned goes to [`Turns`], for phase 5e.
/// (M5-N1a) Each moves by its movement numbers as its Augmentations make them, at its owner's levels in `research`.
#[expect(
    clippy::too_many_arguments,
    reason = "phase 5d's state, in the parts of the sim it reads and writes"
)]
pub(crate) fn step(
    objects: &mut BTreeMap<ObjectId, Object>,
    first_new: ObjectId,
    map: &Map,
    nav: &Nav,
    scratch: &mut Scratch,
    tick: Tick,
    research: Research,
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
        let mut augmented = None;
        let Some((unit, movement, half)) = unit_of(o, research, &mut augmented) else {
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
        let body = body_of(o, mover, movement, half);
        // (M5-V1) Its box: a turned Silhouette's bounding box.
        let half = body.half;
        scratch.bodies.push(body);
        scratch.chased.push(match mover.goal {
            // (M5-H1) Where it counts as being: a Glider past the map's edge at the nearest edge tile.
            MoveGoal::Object { id, .. } => objects.get(&id).map(|target| Chased {
                id,
                pos: target.map_pos(map),
                shape: Shape::on_map(target, map),
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
        let mut augmented = None;
        let Some((unit, movement, half)) = unit_of(o, research, &mut augmented) else {
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
        let (from, facing) = (o.pos, o.facing);
        let door = scratch.doors[me];
        if door.is_none() {
            mover.exit = None;
            mover.exit_heading = None;
        }
        let mut world = World {
            grid: paths::unit_grid(nav, base, half * 2.0),
            layer: layer.map(|l| &scratch.layers[l]),
            bodies: &scratch.bodies,
            me,
            contact: base.contact_step,
            door,
            lanes: &mut scratch.lanes,
            counters: &mut scratch.counters,
            mine: scratch.bodies[me],
        };
        let mut turned = 0.0;
        // (M3-G) A unit with momentum brakes it off before anything else (`drag`), and its drag is doubled on
        // Rugged terrain: the tile under its centre (`systems/units.md > Which tile counts`).
        let rugged = matches!(*movement, MovementAttrs::Drag { .. })
            && map
                .tile_preset(TilePos::containing(o.pos))
                .is_some_and(|preset| preset.rugged);
        let coasting = mover.momentum != Vec2::ZERO;
        let failed = if let Some(glider) = Glider::of(movement) {
            // (M5-H1) A Glider never stands, waits nor yields: it flies, whatever its attack does (`glider`).
            glider.fly(
                &mut o.pos,
                &mut o.facing,
                &mut turned,
                mover,
                scratch.chased[me].as_ref(),
                half,
                map,
            );
            false
        } else if held {
            // A unit in an attack sequence is busy: it doesn't yield (design change 0023).
            mover.sidestep = None;
            wait(mover, tick);
            if let Some(model) = Model::of(movement, mover, rugged) {
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
                movement,
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
                movement.max_speed() * mover.speed_factor,
                &mut world,
                &mut scratch.touched,
            )
        {
            // An idle unit that overlaps another moves off it (design change 0126).
            false
        } else if mover.goal == MoveGoal::None && mover.sidestep.is_some() && !coasting {
            let speed = movement.max_speed() * mover.speed_factor;
            yielding::sidestep(
                &mut o.pos,
                mover,
                speed,
                &mut world,
                &mut scratch.touched,
                &mut scratch.requests,
            );
            false
        } else if let Some(ExitDoor::Side(door)) = door.filter(|_| mover.goal == MoveGoal::None) {
            // (M5-V2) A unit with momentum walks out by its model, braking to rest where it is out.
            doorway::leave(
                &mut o.pos,
                &mut o.facing,
                &mut turned,
                mover,
                movement,
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
                movement,
                rugged,
                &mut world,
                &mut scratch.touched,
                &mut scratch.requests,
                scratch.chased[me].as_ref(),
                &scratch.engaged,
                tick,
            )
        };
        // (M5-V1) A turn its tick made outside `World::turn` (DragMovement's, a portal's doorway) is tried where the
        // unit now stands, and cut at contact as every turn of a Silhouette that turns is: it never ends up
        // overlapping, and a cut turn loses the unit its speed (`systems/units.md > UnitCollision > Collision
        // Response (Un-sticking)`, design change 0035).
        if let Some(lies) = world.body().oriented
            && lies.facing != o.facing
        {
            let turn = world.turn(o.pos, lies.facing, o.facing);
            if turn.cut {
                o.facing = turn.facing;
                turned = turned.min(math::wrap_angle(o.facing - facing).abs());
                o.mover.as_mut().expect("units have movers").momentum = Vec2::ZERO;
            }
        }
        if failed {
            events.push(SimEvent::MoveFailed { unit: o.id });
        }
        // (M5-V2) A model that turns by its speed has no TurnRate to spend (`turn_rate` 0: FixedTurnRadiusMovement,
        // GliderMovement): its turn is no part of the budget, and phase 5e's aim, which has none either, isn't left
        // a budget overdrawn by it.
        if turned > 0.0 && turn_rate(movement) > 0.0 {
            scratch.turns.0.push((o.id, turned));
        }
        // Out of the doorway: from now on the building blocks the unit like everything else, for good (design
        // change 0080). One that walked out without a goal stands there.
        let bound = oriented_of(o, half).map_or(half, |lies| lies.bound_half());
        let mover = o.mover.as_mut().expect("units have movers");
        if door.is_some_and(|door| !door.holds(o.pos, bound)) {
            mover.exit = None;
            mover.exit_heading = None;
            if mover.goal == MoveGoal::None && mover.sidestep.is_none() && mover.momentum == Vec2::ZERO {
                stand(mover);
            }
        }
        // Commit: later units see this one where it is now. (M5-V1) A unit that turned lies another way: its box,
        // a turned Silhouette's bounding box, may have changed where it didn't move.
        let mover = o.mover.as_ref().expect("units have movers");
        let before = scratch.bodies[me];
        scratch.bodies[me] = body_of(o, mover, movement, half);
        let after = &scratch.bodies[me];
        if let Some(l) = layer {
            if o.pos != from || after.half != before.half {
                scratch.layers[l].remove(me as u32, from, before.half);
                scratch.layers[l].insert(me as u32, o.pos, after.half, after.kinds());
            } else if after.kinds() != before.kinds() {
                scratch.layers[l].mark(o.pos, after.half, after.kinds());
            }
        }
    }
    // (M4-S2) After the movers' own requests, which come first for a unit asked twice.
    for &(owner, centre, half) in &scratch.rising {
        let ground = &scratch.layers[0];
        yielding::make_way(owner, centre, half, ground, &scratch.bodies, &mut scratch.requests);
    }
    yielding::apply(objects, &scratch.bodies, &mut scratch.requests, research);
    // (M5-H1) The air units keep apart, from where the moves left them.
    air::separate(objects, first_new, map, &mut scratch.air);
}

/// The unit stands this tick.
pub(crate) fn stand(mover: &mut Mover) {
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
