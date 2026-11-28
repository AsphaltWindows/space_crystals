//! Phase 5d, the one movement pipeline — `docs/movement.md > 4`, `docs/movement.md > 6`. The only code that writes
//! a unit's `pos` (besides a debug teleport).
//!
//! Units move one after another in id order. Each steers at its path's steering waypoint, sweeps its box against
//! the tiles and the other units of its layer, and commits a position that overlaps nothing that blocks it
//! ([`space::blocks`]: every unit but the two exceptions of design change 0106 and, (M5-V4), a crusher's crushable
//! enemies, which it kills); the spatial grid is updated at each commit, so later units see earlier ones where they
//! now are. "No overlap" therefore holds by construction, and the id order makes it deterministic
//! (`docs/movement.md > 4`).
//!
//! - [`space`]: the spatial grids (one per collision layer), the box sweep, and [`space::blocks`], the single
//!   filter for collision exceptions: gathering Drones, units that already overlap (design change 0106), and (M5-V4)
//!   a crusher's crushable enemies.
//! - (M5-V4) [`crush`]: crushing (`systems/units.md > UnitCollision > Crushing`): whose move crushes this tick, and the
//!   crushable enemies it kills, which leave the grid at once and the game at the end of the phase.
//! - [`scratch`]: the phase's scratch, rebuilt every tick: every unit as the others see it, the objects the units
//!   head for, and the turn budget phase 5e reads ([`Turns`]).
//! - [`collide`]: one unit's move by the rules of `docs/movement.md > 6.3`: lane seeking among the units it has to
//!   pass (oncoming ones, and idle groups that make way), then the rules after the sweep.
//! - [`stuck`]: progress, stuck detection and recovery (`docs/movement.md > 6.5`).
//! - [`yielding`]: an idle unit's sidestep for a blocked unit of its owner (`docs/movement.md > 6.6`).
//! - [`model`]: the models of the units with momentum, [`drag`] (DragMovement, slice M3-G), (M5-V2) [`wheeled`]
//!   (FixedTurnRadiusMovement: the turn by speed on a fixed radius, braking, backing up on the Reverse order, the
//!   forward loop) and (M5-V3) [`tracked`] (SpeedTurnRadiusMovement: the tread law, the spin, the speed each turn is
//!   taken at); [`follow`]: one unit's tick along its path, shared by every model.
//! - [`separate`]: an idle unit that overlaps another moves off it (design change 0126).
//! - (M5-H1) [`glider`]: GliderMovement, the model of the units that never stop, in place of a unit's tick along its
//!   path; [`air`]: the soft separation of air units from each other, after every unit has moved. (M6-GL)
//!   [`approach`]: a Glider's landing approach, which ends stopped over its point, in place of its flight, and its hold
//!   in the air while it is over that point, on an Airfield's slot or rising off one.
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
//! Hooks for later slices: other movement models replace the "desired direction → facing and velocity" part of a
//! unit's tick, as [`drag`], [`wheeled`] and [`tracked`] do in [`follow`]; (M5-V4) crushing is an exception in
//! [`space::blocks`] and a kill at the crusher's commit ([`crush`]).

mod air;
mod approach;
mod collide;
mod crush;
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
mod tracked;
mod wheeled;
mod yielding;

use std::collections::BTreeMap;

use glam::Vec2;

use crate::augmentation::Research;
use crate::combat;
use crate::defs::{Domain, MovementAttrs};
use crate::events::SimEvent;
use crate::ids::ObjectId;
use crate::map::Map;
use crate::math;
use crate::mover::{Approach, Locomotion, MoveGoal, Mover, Orientation};
use crate::nav::Nav;
use crate::object::Object;
use crate::paths;
use crate::popup;
use crate::production::ExitDoor;
use crate::shape::Shape;
use crate::units::{SU_PER_GU, Tick, TilePos};

use collide::World;
pub(crate) use drag::flight_ticks;
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
/// phase 5d, `wheeled`). (M5-V3) A SpeedTurnRadiusMovement unit's SpinRate, the most its treads turn it in a tick,
/// stopped (`systems/units.md > SpeedTurnRadiusMovement > The turning law`, design change 0417): a stopped one aims by
/// spinning its hull. Moving it turns less (`tracked`), and phase 5d notes what its speed took of the budget with
/// what it turned ([`tracked_turn_noted`]).
pub(crate) fn turn_rate(movement: &MovementAttrs) -> f32 {
    match movement {
        MovementAttrs::Glider { .. } | MovementAttrs::FixedTurnRadius { .. } => 0.0,
        MovementAttrs::SpeedTurnRadius { .. } => movement.spin_rate().unwrap_or(0.0),
        _ => movement.turn_rate().unwrap_or(f32::INFINITY),
    }
}

/// (M5-V3) What phase 5d notes in [`Turns`] for a SpeedTurnRadiusMovement unit that turned `turned` and carries
/// `momentum` into the next tick: the angle turned, and the part of SpinRate its speed leaves no tread for
/// (`SpinRate × |v| ÷ MaxSpeed`, the law's `maxTurnRate = SpinRate × (1 − |v| ÷ MaxSpeed)` the other way round). So
/// phase 5e's aim of its hull has what the law leaves it at that speed, never less than nothing, and the full SpinRate
/// once it stands.
fn tracked_turn_noted(movement: &MovementAttrs, turned: f32, momentum: Vec2) -> f32 {
    let spin = movement.spin_rate().unwrap_or(0.0);
    let max = movement.max_speed();
    let taken = if max > 0.0 {
        spin * (momentum.length() / max).min(1.0)
    } else {
        0.0
    };
    (turned + taken).min(spin)
}

/// Phase 5d (`docs/movement.md > 4`): builds the spatial grids from **all** units, this tick's spawns included
/// (they are obstacles, though not moved), then moves every unit before `first_new`, in id order, updating the
/// grid at each commit. A unit whose attack holds its base (`combat::may_move`, `combat::may_turn`) stands and
/// keeps its goal, and so (M6-SD) does an aircraft rising out of an air door (`Mover::rises`), and (M6-GL) a Glider
/// held in the air by its landing: over its slot, on it, or rising off it (`approach`), a Glider otherwise flying
/// whatever its attack does. An idle unit asked to
/// make way takes its sidestep in its turn, like any move (`yielding::sidestep`); the sidesteps asked for in this tick
/// are noted when every unit has moved (`yielding::apply`) and begin in the next. (M4-S2) A lowered pop-up turret that
/// wants to rise asks its owner's idle units over its footprint the same way (`yielding::make_way`). The angle each
/// unit turned goes to [`Turns`], for phase 5e. (M5-N1a) Each moves by its movement numbers as its Augmentations make
/// them, at its owner's levels in `research`. (M5-V4) A crusher whose move crushes passes through the crushable enemies
/// in its way and kills those it overlaps or sweeps (`crush`): they make no move of their own after it, and their
/// `SimEvent::Crushed` come once every unit has moved. Returns them, for the removal path.
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
) -> Vec<ObjectId> {
    scratch.counters = MoveCounters::default();
    scratch.bodies.clear();
    scratch.chased.clear();
    scratch.engaged.clear();
    scratch.doors.clear();
    scratch.requests.clear();
    scratch.rising.clear();
    scratch.turns.0.clear();
    scratch.crushed.clear();
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
        if let Some(layer) = layer_of(unit.domain()) {
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
        // (M5-N10) The turn its behaviour asked of it in phase 5a, for this tick alone (`Mover::turn_to`).
        let turn_to = o.mover.as_mut().and_then(|mover| mover.turn_to.take());
        // (M5-V4) A unit crushed earlier in the tick is dead: it makes no move.
        if o.id >= first_new || scratch.crushed.iter().any(|crush| crush.victim == me) {
            continue;
        }
        // (M6-SD) An aircraft rising out of an air door holds still, keeping its goal, until its rise is over
        // (`Mover::rises`).
        let rising = o.mover.as_mut().is_some_and(Mover::rises);
        // `systems/combat.md > UnitBaseSource`: while its attack aims, fires or cools down, a unit's base neither
        // moves nor turns (the attack turns it while aiming, in phase 5e).
        let held = rising || !combat::may_move(o) || !combat::may_turn(o);
        // (M6-GL) A Glider on an Airfield's slot, landing or down, holds still there (`approach::holds`), and so does
        // one over the point of its landing approach while the order that sent it there runs (`approach::stale`).
        let on_slot = crate::landing::of(o).is_some();
        let ordered = o.orders.as_ref().is_some_and(|orders| orders.current.is_some());
        let mover = o.mover.as_mut().expect("units have movers");
        let base = unit.base_def();
        // (M6-ST1) Its type's Domain, its own over its base's (`UnitDef::domain`).
        let domain = unit.domain();
        let layer = layer_of(domain);
        let (from, facing) = (o.pos, o.facing);
        let door = scratch.doors[me];
        if door.is_none() {
            mover.exit = None;
            mover.exit_heading = None;
        }
        // (M5-V4) Whether its move this tick crushes (`crush::drives`): its own body says so, for its turn alone.
        // (M6-ST1) By its type's flag, its own over its base's (`UnitDef::crushes`).
        let crushing = crush::drives(unit.crushes(), mover, held);
        scratch.bodies[me].crushes = crushing;
        let mut world = World {
            grid: paths::unit_grid(nav, unit, half * 2.0),
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
        // (M6-GH1) An air HoverCraft brakes by its drag alone (`UnitBaseDef::brakes_by_drag`, `drag`).
        let terms = drag::Terms {
            rugged: matches!(*movement, MovementAttrs::Drag { .. })
                && map
                    .tile_preset(TilePos::containing(o.pos))
                    .is_some_and(|preset| preset.rugged),
            drag_only: base.brakes_by_drag,
        };
        let coasting = mover.momentum != Vec2::ZERO;
        let failed = if let Some(glider) = Glider::of(movement) {
            if approach::stale(mover, ordered) {
                mover.stop();
            }
            if approach::holds(mover, on_slot, rising) {
                // (M6-GL) Its landing holds it still in the air: over its slot, on it, and rising off it (`approach`).
                approach::hold(mover, &mut turned);
            } else if let (Some(Approach::Flying), MoveGoal::Point(spot)) = (mover.approach, mover.goal) {
                // (M6-GL) Its landing approach, in place of its flight (`approach`).
                approach::fly(&glider, &mut o.pos, &mut o.facing, &mut turned, mover, spot);
            } else {
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
            }
            false
        } else if held {
            // A unit in an attack sequence is busy: it doesn't yield (design change 0023).
            mover.sidestep = None;
            wait(mover, tick);
            if let Some(model) = Model::of(movement, mover, terms) {
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
                terms,
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
        } else if let Some(toward) = turn_to.filter(|_| mover.goal == MoveGoal::None && !coasting) {
            // (M5-N10) At rest, stopped by its behaviour to aim with its hull (`Mover::turn_to`).
            turn_in_place(
                &mut o.pos,
                &mut o.facing,
                &mut turned,
                toward,
                mover,
                movement,
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
                terms,
                &mut world,
                &mut scratch.touched,
                &mut scratch.requests,
                scratch.chased[me].as_ref(),
                &scratch.engaged,
                tick,
            )
        };
        // (M6-GK) A HoverCraft stays on the map (`systems/units.md > Air Collision > Map edge`: "HoverCraft stay inside
        // the map"; design change 0045: "Movement bounds: clamp HoverCraft to the map"): its own move is held where its
        // box lies on the map (`Map::clamp_box`), as the push of the soft separation is ([`air`]). The edge stops it as
        // an obstacle does (design change 0035): the blocked part of the move is dropped, and no momentum is left. Its
        // path ends where its box lies on the map (`paths::serve`), so it lands there and arrives. A Glider may fly past
        // the edge (`glider`).
        if domain == Domain::Air && !unit.glides() {
            let on_map = map.clamp_box(o.pos, half);
            if on_map != o.pos {
                o.pos = on_map;
                let mover = o.mover.as_mut().expect("units have movers");
                mover.velocity = on_map - from;
                mover.momentum = Vec2::ZERO;
                if mover.velocity == Vec2::ZERO {
                    mover.locomotion = Locomotion::Stationary;
                }
            }
        }
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
        // a budget overdrawn by it. (M5-V3) A SpeedTurnRadiusMovement unit's budget is its SpinRate, of which its
        // speed takes a part (`tracked_turn_noted`).
        let noted = match movement {
            MovementAttrs::SpeedTurnRadius { .. } => {
                tracked_turn_noted(movement, turned, o.mover.as_ref().map_or(Vec2::ZERO, |m| m.momentum))
            }
            _ => turned,
        };
        if noted > 0.0 && turn_rate(movement) > 0.0 {
            scratch.turns.0.push((o.id, noted));
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
        // (M5-V4) The crushable enemies its move crushed leave the grid at once (`crush::victims`): the units after it
        // no longer meet them, and those that haven't moved yet don't move.
        if crushing && let Some(l) = layer {
            let first = scratch.crushed.len();
            crush::victims(
                &scratch.bodies,
                &scratch.layers[l],
                me,
                from,
                facing,
                &mut scratch.crushed,
            );
            for crush in &scratch.crushed[first..] {
                let victim = &scratch.bodies[crush.victim];
                scratch.layers[l].remove(crush.victim as u32, victim.pos, victim.half);
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
    crush::tell(objects, &scratch.bodies, &scratch.crushed, events)
}

/// (M5-N10) A unit at rest without a goal that its behaviour asked to turn toward `toward` (`Mover::turn_to`: the hull
/// aim of `combat::turret::aims_by_hull`, "the vehicle must **stop and turn its hull** to face them",
/// `factions/cults/units/hand_of_heaven.md > TurretAttributes`) turns toward it in place and does nothing else this
/// tick: as its model spins at rest (`Tracked::spin`, the SpinRate of a TrackedVehicle, backing off or creeping out
/// of a contact as its spin toward its way does), or by its TurnRate for another model ([`turn_rate`]), as far as its
/// Silhouette stays free (`World::turn`, invariants I1 and I2). The unit that stopped the turn, if one did, is asked to
/// make way (`yielding::ask`), as one that stops a turn on a path is (`follow::creep_on`). It stands, Turning
/// (`systems/controls/action_channels.md > OrientationChannel`), and what it turned goes to `turned`, for [`Turns`].
/// `touched` is scratch.
#[expect(
    clippy::too_many_arguments,
    reason = "the unit's state, split into the borrows the step holds"
)]
fn turn_in_place(
    pos: &mut Vec2,
    facing: &mut f32,
    turned: &mut f32,
    toward: Vec2,
    mover: &mut Mover,
    movement: &MovementAttrs,
    world: &mut World,
    touched: &mut Vec<usize>,
    requests: &mut Vec<yielding::Request>,
) {
    let (start, from) = (*facing, *pos);
    let dir = toward - from;
    if dir != Vec2::ZERO {
        let blocker = match Model::of(movement, mover, drag::Terms::default()) {
            Some(Model::Tracked(tracked)) => {
                let (at, blocker) = tracked.spin(from, facing, dir, world, touched);
                *pos = at;
                blocker
            }
            _ => {
                math::turn_toward(facing, math::wrap_angle(math::vector_angle(dir)), turn_rate(movement));
                let turn = world.turn(from, start, *facing);
                *facing = turn.facing;
                turn.blocker.filter(|_| turn.cut)
            }
        };
        let stopped = collide::Moved {
            pos: from,
            forward: false,
            blocker,
            aside: false,
            slid: false,
        };
        yielding::ask(world, from, dir, &stopped, false, mover, requests);
    }
    *turned = math::wrap_angle(*facing - start).abs();
    stand(mover);
    mover.orientation = Orientation::Turning(toward);
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
