//! FixedTurnRadiusMovement — `systems/units.md > FixedTurnRadiusMovement` ("Unit cannot turn in place. Turns at a
//! fixed minimum radius regardless of speed. Can stop and reverse. Models a simple wheeled vehicle with no drift or
//! slippage."), `systems/units.md > WheeledVehicle` (slice M5-V2, plan decision T79). Like DragMovement
//! (`super::drag`) the model keeps the unit's velocity from tick to tick ([`Mover::momentum`]) and replaces only the
//! "desired direction → velocity and facing" part of a unit's tick: path following, lane seeking, the rules after the
//! sweep, yielding and stuck detection are shared with every model (`super::follow`).
//!
//! **A signed speed** (T79): forward along the facing, or backwards against it on a leg of the Reverse order
//! (`Mover::reverses`, `systems/controls/command_behaviors.md > ReversingToLocation`). `Mover::momentum` carries it
//! from tick to tick as the unit's latest displacement: its length is the speed, the side of the facing it points to
//! the sign ([`speed_of`]). Each tick, in the order of `systems/units.md > UnitCollision > Collision Response
//! (Un-sticking)` ("1. Velocity … 2. Displacement … 3. Blocked"):
//! 1. **Speed**: towards the speed the unit wants (below), by at most ForwardAcceleration (ReverseAcceleration
//!    backing up) a tick up and Deceleration down, and never above ForwardMaxSpeed (ReverseMaxSpeed), each top speed
//!    times the mover's `speed_factor`. A unit moving the other way from the one its leg drives brakes to rest first,
//!    by Deceleration, and only then sets off.
//! 2. **Turn** (`systems/combat.md > LocomotionOrientationConstraints[FixedTurnRadiusMovement]`): "Moving + Turning:
//!    maxTurnRate = currentSpeed / MinimumTurnRadius", and the same Reversing; "Stationary + Turning: invalid",
//!    "Stopping + Turning: invalid". The speed is this tick's, after step 1, as the design orders the steps ("add this
//!    frame's acceleration ... and compute this frame's turn"), so a unit setting off from rest turns as far as it
//!    moves. A Silhouette turns as far as it stays free (`World::turn`, slice M5-V1), and a turn cut short is a blocked
//!    move (step 4).
//! 3. **Displacement**: the speed along the heading halfway through the tick's turn, so that the positions of a turn
//!    at full lock lie on a circle of MinimumTurnRadius: each tick is the chord of its arc, as long as the arc (on the
//!    Seeker's 64 su at 8 su a tick, a circle 0.04 su wider). As far as tiles and units allow, the two axes one after
//!    the other where the whole move isn't free (`World::shuffle`).
//! 4. **Blocked** (design change 0035): if the move or the turn met a tile or a unit, "the blocked portion of the
//!    movement is dropped ... and its velocity is set to 0": the unit starts again from rest in its next tick. Pressed
//!    against an obstacle it moves one tick's acceleration a tick (the Seeker 1 su) and turns as far as that takes it,
//!    so it creeps and turns out of the contact ("It creeps, but it keeps making its turn"). A turn cut where a corner
//!    meets what it is pressed against is made all the same, its blocked part dropped: the unit backs off the way it
//!    came by the least that frees it, at most what its corners sweep in (`World::back_off`), so that nose first
//!    against a wall it pivots about the corner that touches it. Only a turn that backing off doesn't free stays cut
//!    (a unit beside it).
//!
//! The integration is the design's order, the speed before the move by it (semi-implicit, as DragMovement's): from
//! rest the Seeker covers 1, 2, 3 … su in its first ticks and is at full speed, 8 su, in its 8th; braking from it, it
//! covers 7 + 6 + … + 1 = 28 su and is at rest in its 8th tick ([`tests`]).
//!
//! **What the unit wants** is the step the shared rules gave a unit without momentum (the probe, as for DragMovement):
//! the way to its steering waypoint, a lane past oncoming units, the way round a blocker, along a wall. It turns to
//! head along that step (its back along it, backing up), and wants the step's length as its speed, which is at most
//! the **approach speed** ([`Wheeled::approach`]). Where it can make that step exactly this tick, it is its move as
//! the rules gave it (they found it free, and an arrival lands exactly on the goal). Where the rules' step is a slide
//! along tiles or nothing, the unit heads along the step it asked of them instead, into what blocks it: "running into
//! the wall every frame", as DragMovement's creep (`docs/movement.md > 6.7`), so that it keeps turning towards its
//! way as it scrapes, where a slide's step would turn it into the wall.
//!
//! **Braking and arrival**: the approach speed is the highest from which braking by Deceleration still stops the unit
//! within the path that remains (`drag::approach_speed`, without drag): the steps back from the goal are 1, 2, 3 …
//! su, so from full speed the Seeker begins to brake 36 su before its goal (8 + 7 + … + 1: the step at full speed in
//! which the braking distance v² / (2 × Deceleration) of T79 is reached, then 28 su of braking), and its last step,
//! of at most 1 su, lands exactly on the goal. The radius doesn't depend on the speed, so it doesn't slow for a
//! corner (T79).
//!
//! **The forward loop** (design change 0353, answering Q0119 point 3: "a point inside its turning circle (e.g. just
//! behind it) is reached by a **forward loop**"): a waypoint inside the circle the unit turns on towards it can't be
//! reached by turning, so the unit drives straight on until it lies outside it, then turns towards it at full lock,
//! round and onto it ([`Wheeled::steer`]). A point straight behind it is outside both its circles: it turns round on
//! one. Backing up, the same on the circles it backs along. Only the goal is reached so: a waypoint on the way to it
//! that lies inside the circle is passed, the path going on from there (`super::follow`). A loop that the way ahead
//! blocks presses on towards the waypoint, its turn's blocked parts dropped, rather than into what blocks it. "Predictable, not optimal" (T79): no reversing by itself
//! (0353 point 2: "never by itself, e.g. no automatic three-point turns"), no planned manoeuvres.
//!
//! **Without a goal** ([`Wheeled::coast`]) a unit that still has momentum brakes to rest along its facing, moving
//! meanwhile (`Locomotion::Stopping`, which doesn't turn): a Stop order, an arrival within reach of an object, a failed
//! move, an attack that holds it.
//!
//! Float rules (`docs/movement.md > 7`): `+ − × ÷` and `sqrt` only, but for the angles and the facing's unit vector,
//! which go through `sc_sim::math`. Every position is committed through the same predicates as any other move
//! (`World::clear`, `World::shuffle`, `World::turn`), so the model adds no way for two shapes to come to overlap.

use glam::Vec2;

use super::collide::{Moved, World};
use super::drag::approach_speed;
use crate::defs::MovementAttrs;
use crate::math;
use crate::mover::{Locomotion, Mover};

/// One unit's FixedTurnRadiusMovement numbers for this tick, the way its leg drives, in space units and ticks.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(super) struct Wheeled {
    /// MinimumTurnRadius.
    radius: f32,
    /// Whether the leg drives backwards (`Mover::reverses`).
    reverses: bool,
    /// ForwardMaxSpeed, or ReverseMaxSpeed backing up, times the mover's `speed_factor`.
    top: f32,
    /// ForwardAcceleration, or ReverseAcceleration backing up.
    accel: f32,
    /// Deceleration, either way.
    decel: f32,
}

/// What became of a unit's tick by the model ([`Wheeled::drive`]).
#[derive(Copy, Clone, Debug, PartialEq)]
pub(super) struct Driven {
    /// Its move, with the rules' word on it.
    pub moved: Moved,
    /// The move it meant to make, before tiles and units cut it.
    pub velocity: Vec2,
    /// The angle it turned, radians.
    pub turned: f32,
    /// How it moved: Moving, Reversing, or Stopping while it brakes to set off the other way.
    pub locomotion: Locomotion,
    /// Its turn was cut short (`World::turn`): by the unit with this index, or by the tiles (`Some(None)`).
    pub cut: Option<Option<usize>>,
}

/// The signed speed of a unit with momentum `momentum` and the facing unit vector `ahead`: the momentum's length,
/// negative where it points behind the facing.
fn speed_of(momentum: Vec2, ahead: Vec2) -> f32 {
    let speed = momentum.length();
    if momentum.dot(ahead) < 0.0 { -speed } else { speed }
}

impl Wheeled {
    /// The model of a unit with these attributes and this mover, `None` for every other movement model. A leg of the
    /// Reverse order backs up by the model's reverse numbers (`MovementAttrs::reverse_speed`). An acceleration or a
    /// deceleration the design leaves out is reached within the tick, as by the models that stop within the tick.
    pub(super) fn of(movement: &MovementAttrs, mover: &Mover) -> Option<Self> {
        let MovementAttrs::FixedTurnRadius {
            minimum_turn_radius,
            forward_acceleration,
            forward_max_speed,
            deceleration,
            ..
        } = *movement
        else {
            return None;
        };
        let back = movement.reverse_speed().filter(|_| mover.reverses);
        let (top, accel) = back.map_or((forward_max_speed, forward_acceleration), |back| {
            (back.max_speed, back.acceleration)
        });
        Some(Self {
            radius: minimum_turn_radius,
            reverses: back.is_some(),
            top: top * mover.speed_factor,
            accel: accel.unwrap_or(f32::INFINITY),
            decel: deceleration.unwrap_or(f32::INFINITY),
        })
    }

    /// The speed the unit approaches a goal `dist` space units along its path at: the highest from which braking by
    /// Deceleration stops it on the goal (module docs).
    pub(super) fn approach(&self, dist: f32) -> f32 {
        approach_speed(dist, 1.0, self.decel, self.top)
    }

    /// The unit vector the leg drives along, for a unit facing `facing`: ahead, or behind it backing up.
    fn travel(&self, facing: f32) -> Vec2 {
        let ahead = math::facing_vector(facing);
        if self.reverses { -ahead } else { ahead }
    }

    /// Whether the point `to` away from a unit facing `facing` lies inside the circle of MinimumTurnRadius it would
    /// turn on towards it (the one on its side of the way the unit drives): no turn reaches it from here.
    pub(super) fn inside(&self, facing: f32, to: Vec2) -> bool {
        let travel = self.travel(facing);
        // Its right, clockwise on screen.
        let right = Vec2::new(-travel.y, travel.x);
        let side = to.dot(right);
        let centre = right * self.radius.copysign(side);
        side != 0.0 && (to - centre).length_squared() < self.radius * self.radius
    }

    /// The forward loop (module docs; design change 0353 point 3): where a unit facing `facing` steers for its
    /// steering waypoint `to` away. At it, unless it lies inside its turning circle ([`Wheeled::inside`]): then
    /// straight on, as far as it is, and `true`.
    pub(super) fn steer(&self, facing: f32, to: Vec2) -> (Vec2, bool) {
        if self.inside(facing, to) {
            (self.travel(facing) * to.length(), true)
        } else {
            (to, false)
        }
    }

    /// Step 1 (module docs): the signed speed after this tick's acceleration or braking, from the signed speed
    /// `speed`, towards `want` (0 or more) the way the leg drives. Moving the other way, it brakes towards rest.
    fn next_speed(&self, speed: f32, want: f32) -> f32 {
        let way = if self.reverses { -1.0 } else { 1.0 };
        let along = speed * way;
        let next = if along < 0.0 {
            (along + self.decel).min(0.0)
        } else {
            let want = want.min(self.top);
            if along < want {
                (along + self.accel).min(want)
            } else {
                (along - self.decel).max(want)
            }
        };
        next * way
    }

    /// Steps 1 to 4 (module docs) for a unit on its way: `probe` is the step from `from` that the shared rules gave a
    /// unit without momentum, at the approach speed, for the step `asked` of them. `lands` is set when that step lands
    /// exactly on the goal. The unit turns to head along the step and wants its length as its speed. If it can make
    /// that step exactly in this tick (it heads along it, at its length), the step is its move as the rules gave it;
    /// one that lands it on its goal is made also from a little faster, the rounding of the braking
    /// ([`Wheeled::approach`]), and leaves it at rest. Otherwise it moves by its speed along its heading, as far as
    /// tiles and units allow ([`World::shuffle`]). A move that met something, or a turn cut short, leaves it no
    /// momentum (design change 0035). Where the rules' step is a slide along tiles, or none (blocked), the unit heads
    /// for the step it asked for, as `super::drag` does: "running into the wall every frame", it creeps and turns
    /// towards its way at single-tick acceleration, its turns' blocked parts dropped, until it is free. `touched` is
    /// scratch.
    #[expect(
        clippy::too_many_arguments,
        reason = "the unit's state, split into the borrows the step holds"
    )]
    pub(super) fn drive(
        &self,
        from: Vec2,
        facing: &mut f32,
        asked: Vec2,
        probe: Moved,
        lands: bool,
        mover: &mut Mover,
        world: &mut World,
        touched: &mut Vec<usize>,
    ) -> Driven {
        // The rules' step, unless it is a slide or nothing: then the step asked of them.
        let ruled = !probe.slid && probe.pos != from;
        let step = if ruled { probe.pos - from } else { asked };
        let want = step.length();
        let start = *facing;
        let speed = self.next_speed(speed_of(mover.momentum, math::facing_vector(start)), want);
        // Braking to set off the other way: Stopping, which doesn't turn.
        let stopping = speed != 0.0 && (speed < 0.0) != self.reverses;
        let turn = if speed == 0.0 || stopping || want == 0.0 {
            None
        } else {
            let along = if self.reverses { -step } else { step };
            let wanted = math::wrap_angle(math::vector_angle(along));
            Some(math::turn_toward(facing, wanted, speed.abs() / self.radius))
        };
        let turned = world.turn(from, start, *facing);
        // Design change 0035: a turn cut by what the unit is pressed against is made all the same, its blocked part
        // dropped: the unit backs off the way it came by what its corners would sweep in (`World::back_off`). Only a
        // turn that can't be freed so stays cut.
        let mut set_off = from;
        if turned.cut {
            let back = math::facing_vector(start) * -speed.signum();
            let most = math::wrap_angle(*facing - start).abs() * world.body().oriented.map_or(0.0, |o| o.half.length());
            match world.back_off(from, *facing, back, most) {
                Some(at) => set_off = at,
                None => *facing = turned.facing,
            }
        }
        let angle = math::wrap_angle(*facing - start);
        let done = turn.is_none_or(|turn| turn.done) && !turned.cut;
        // The rules' step, if the unit makes it as they gave it: free in the shape it now has, and driven the way the
        // leg goes (not while it brakes to set off the other way).
        let exact = done
            && ruled
            && !stopping
            && (speed.abs() == want || (lands && speed.abs() >= want))
            && (angle == 0.0 || world.clear(from, step, touched));
        let locomotion = if stopping {
            Locomotion::Stopping
        } else if speed < 0.0 {
            Locomotion::Reversing
        } else {
            Locomotion::Moving
        };
        let cut = turned.cut.then_some(turned.blocker);
        if exact {
            mover.momentum = if lands { Vec2::ZERO } else { step };
            return Driven {
                moved: probe,
                velocity: step,
                turned: angle.abs(),
                locomotion,
                cut,
            };
        }
        let velocity = math::facing_vector(start + angle * 0.5) * speed;
        touched.clear();
        let pos = world.shuffle(set_off, velocity, touched);
        // The whole move, or it met something: then it has no velocity left.
        mover.momentum = if pos == from + velocity && !turned.cut {
            pos - from
        } else {
            Vec2::ZERO
        };
        let moved = Moved {
            pos,
            // Getting on is the rules' word, as long as the unit moved at all.
            forward: probe.forward && pos != from,
            blocker: probe.blocker.or(touched.first().copied()),
            aside: probe.aside,
            slid: probe.slid,
        };
        Driven {
            moved,
            velocity,
            turned: angle.abs(),
            locomotion,
            cut,
        }
    }

    /// Steps 1 and 2 for a unit held to the line along the unit vector `dir`, forward: one walking out of a doorway
    /// (`super::doorway`). It turns towards `dir` by what its speed allows, and its speed along the line goes from that
    /// of `momentum` towards `want`. Returns its speed along `dir` for this tick (0 or more) and the angle it turned.
    pub(super) fn along(&self, dir: Vec2, want: f32, facing: &mut f32, momentum: Vec2) -> (f32, f32) {
        let speed = self.next_speed(momentum.dot(dir), want).max(0.0);
        if speed == 0.0 {
            return (0.0, 0.0);
        }
        let wanted = math::wrap_angle(math::vector_angle(dir));
        let turn = math::turn_toward(facing, wanted, speed / self.radius);
        (speed, turn.angle)
    }

    /// Steps 1, 3 and 4 for a unit that isn't steering this tick (no goal, no path yet, held by its attack) and still
    /// has momentum: it brakes by Deceleration along its facing, which stays as it is ("Stopping + Turning: invalid"),
    /// and moves by what is left, as far as tiles and units allow. It stands once at rest, or when it meets a tile or a
    /// unit (design change 0035: no velocity is left).
    pub(super) fn coast(
        &self,
        pos: &mut Vec2,
        facing: f32,
        mover: &mut Mover,
        world: &mut World,
        touched: &mut Vec<usize>,
    ) {
        if mover.momentum == Vec2::ZERO {
            return;
        }
        let ahead = math::facing_vector(facing);
        let speed = speed_of(mover.momentum, ahead);
        let next = if speed > 0.0 {
            (speed - self.decel).max(0.0)
        } else {
            (speed + self.decel).min(0.0)
        };
        let from = *pos;
        let velocity = ahead * next;
        if next != 0.0 {
            touched.clear();
            *pos = world.shuffle(from, velocity, touched);
        }
        mover.velocity = *pos - from;
        mover.momentum = if next != 0.0 && *pos == from + velocity {
            *pos - from
        } else {
            Vec2::ZERO
        };
        mover.locomotion = if mover.velocity == Vec2::ZERO {
            Locomotion::Stationary
        } else {
            Locomotion::Stopping
        };
    }
}

#[cfg(test)]
mod tests;
