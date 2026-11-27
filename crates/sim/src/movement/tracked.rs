//! SpeedTurnRadiusMovement — `systems/units.md > SpeedTurnRadiusMovement` ("Unit can rotate in place by spinning
//! treads in opposite directions. Turn radius increases with speed ... Small directional adjustments at speed cost
//! little momentum, but sharp turns require significant slowing or a complete stop"), the model of the TrackedVehicle
//! (slice M5-V3, plan decision T85 as design change 0417 replaced its law). Like FixedTurnRadiusMovement
//! (`super::wheeled`), whose structure it takes, it keeps the unit's speed from tick to tick (`Mover::momentum`) and
//! replaces only the "desired direction → velocity and facing" part of a unit's tick: path following, lane seeking,
//! the rules after the sweep, yielding and stuck detection are shared with every model (`super::follow`).
//!
//! **The tread law** (design change 0417, `systems/units.md > SpeedTurnRadiusMovement > The turning law`): "forward
//! speed is their average, turning comes from
//! their difference, and each tread's top speed is MaxSpeed". Stopped, it spins at **SpinRate = 2 × MaxSpeed ÷
//! TrackWidth** a tick; at speed `v` it turns at most **SpinRate × (1 − |v| ÷ MaxSpeed)** ([`Tracked::rate`]), on a
//! tightest radius of `(TrackWidth ÷ 2) × |v| ÷ (MaxSpeed − |v|)`, which is `|v|` over that rate; at MaxSpeed it can't
//! turn at all. Backing up it turns by the same law with `|v|`. Turning costs no speed by itself.
//! `systems/combat.md > LocomotionOrientationConstraints[SpeedTurnRadiusMovement]` calls Stationary and Stopping +
//! Turning "unconstrained": the law at their speed is what limits them here, as 0417's Verification asks ("standing
//! still turns 90° in about 5 frames").
//!
//! **A signed speed**, as `super::wheeled`'s: forward along the facing, or backwards on a leg of the Reverse order
//! (`Mover::reverses`). Each tick, in the order of `systems/units.md > UnitCollision > Collision Response
//! (Un-sticking)`:
//! 1. **Speed**: towards the speed the unit wants (below), by at most Acceleration a tick up (backing up too) and
//!    Deceleration down, never above MaxSpeed (ReverseMaxSpeed backing up), each times the mover's `speed_factor`.
//!    Moving the other way from its leg, it brakes to rest first.
//! 2. **Turn**: towards the step it heads along, by at most the law's rate at this tick's speed, so "it turns as soon
//!    as it is below MaxSpeed, while still braking". At rest that is a spin in place. A Silhouette turns as far as it
//!    stays free (`World::turn`, slice M5-V1): a spin cut short **stops at contact**; a turn cut while it moves is
//!    made as V2's is, backing off by what its corners sweep (`World::back_off`).
//! 3. **Displacement**: the speed along the heading halfway through the tick's turn, as far as tiles and units allow
//!    (`World::shuffle`).
//! 4. **Blocked** (design change 0035): a move that met something, or a turn cut short, leaves no momentum.
//!
//! **The corner choice** (T85 with 0417, the brief's rule: "choose the speed to take it at, from full speed down to a
//! stop and spin. Take the fastest whose arc fits the free space on the nav grid, counting the time to brake to it"):
//! analytic, from the law, with no tables ([`Tracked::corner_speed`]). For a turn of θ it compares
//! `CORNER_SPEEDS` + 1 speeds `v` from a stop to its top speed by the time each takes, counted together: changing
//! speed to `v` from the speed it has, turning θ at the law's rate at `v`, getting back to its top speed, less the
//! way it makes meanwhile towards where the turn leads (its braking along its old heading, `cos θ` of it; the arc's
//! `r × sin θ`; the acceleration after). It takes the soonest of them whose arc, the tightest at `v` from where it
//! starts until it heads the new way, tested every `ARC_TEST_RAD` in its own Silhouette, is clear of the nav grid's
//! blocked tiles ([`Tracked::arc_fits`]); a stop and spin when none is. So it depends only on the angle, the speed it
//! has and the room the arc needs, never on an estimate of a path: predictable (design change 0223). For the Hand of
//! Heaven from full speed: a gentle 15° at 4.5 su a tick, 45° at 3.9, a right angle at 2.8 (on the law's 19 su
//! radius, nearly pivoting about its inner tread), a turn about at 0.9 ([`tests`]). From rest every turn is a spin
//! first: turning in place is the quickest turn there is, and it gives up no speed it has.
//! - **The turn now**: each tick, for the turn from its heading onto the step it takes ([`Tracked::drive`]): its
//!   speed is at most the choice's. A heading that misses the steering waypoint by less than `TRACKED_MISS_SU` is
//!   held as it is: at full speed the law turns it by nothing.
//! - **The corner ahead**: the turn at its steering waypoint, from the way there onto the path's next leg
//!   ([`Tracked::ahead`]): the unit brakes in time to be at that corner's speed once it is within a step of the
//!   waypoint, where `super::follow` passes it and the turn now takes over. Only within braking distance of it.
//!
//! **Without a goal** ([`Tracked::coast`]) a unit that still has momentum brakes to rest along its facing.
//!
//! Float rules (`docs/movement.md > 7`): `+ − × ÷` and `sqrt` only, but for the angles, their sines and cosines and
//! the facing's unit vector, which go through `sc_sim::math`. Every position is committed through the same predicates
//! as any other move (`World::clear`, `World::shuffle`, `World::turn`), so the model adds no way for two shapes to come
//! to overlap.

use glam::Vec2;

use super::collide::{Moved, World};
use super::drag::approach_speed;
use super::wheeled::Driven;
use crate::defs::MovementAttrs;
use crate::math;
use crate::mover::{Locomotion, Mover};
use crate::shape::Shape;
use crate::tuning::movement::{ARC_TEST_RAD, CORNER_SPEEDS, TRACKED_MISS_SU};

/// One unit's SpeedTurnRadiusMovement numbers for this tick, the way its leg drives, in space units and ticks.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(super) struct Tracked {
    /// SpinRate, radians a tick (`MovementAttrs::spin_rate`).
    spin: f32,
    /// MaxSpeed, the law's: each tread's top speed.
    max: f32,
    /// Whether the leg drives backwards (`Mover::reverses`).
    reverses: bool,
    /// MaxSpeed, or ReverseMaxSpeed backing up, times the mover's `speed_factor`.
    top: f32,
    /// Acceleration, either way.
    accel: f32,
    /// Deceleration, either way.
    decel: f32,
}

/// The signed speed of a unit with momentum `momentum` and the facing unit vector `ahead`: the momentum's length,
/// negative where it points behind the facing (as `super::wheeled`'s).
fn speed_of(momentum: Vec2, ahead: Vec2) -> f32 {
    let speed = momentum.length();
    if momentum.dot(ahead) < 0.0 { -speed } else { speed }
}

impl Tracked {
    /// The model of a unit with these attributes and this mover, `None` for every other movement model. A leg of the
    /// Reverse order backs up at ReverseMaxSpeed (`MovementAttrs::reverse_speed`). An acceleration or a deceleration
    /// the design leaves out is reached within the tick.
    pub(super) fn of(movement: &MovementAttrs, mover: &Mover) -> Option<Self> {
        let MovementAttrs::SpeedTurnRadius {
            acceleration,
            deceleration,
            max_speed,
            ..
        } = *movement
        else {
            return None;
        };
        let back = movement.reverse_speed().filter(|_| mover.reverses);
        let top = back.map_or(max_speed, |back| back.max_speed);
        Some(Self {
            spin: movement.spin_rate()?,
            max: max_speed,
            reverses: back.is_some(),
            top: top * mover.speed_factor,
            accel: acceleration.unwrap_or(f32::INFINITY),
            decel: deceleration.unwrap_or(f32::INFINITY),
        })
    }

    /// The speed the unit approaches a goal `dist` space units along its path at: the highest from which braking by
    /// Deceleration stops it on the goal (`drag::approach_speed`, as `super::wheeled`'s).
    pub(super) fn approach(&self, dist: f32) -> f32 {
        approach_speed(dist, 1.0, self.decel, self.top)
    }

    /// The tread law (design change 0417): the most the unit turns in a tick at the speed `speed`, radians,
    /// `SpinRate × (1 − |speed| ÷ MaxSpeed)`, nothing at MaxSpeed and beyond.
    pub(super) fn rate(&self, speed: f32) -> f32 {
        (self.spin * (1.0 - speed.abs() / self.max)).max(0.0)
    }

    /// The unit vector the leg drives along, for a unit facing `facing`: ahead, or behind it backing up.
    fn travel(&self, facing: f32) -> Vec2 {
        let ahead = math::facing_vector(facing);
        if self.reverses { -ahead } else { ahead }
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

    /// The time a turn of `angle` (radians, `0..=π`; `sin` and `cos` its sine and cosine) takes at the speed `v`, from
    /// the speed `from` (module docs, the corner choice), in ticks: changing to `v`, the arc at the law's rate, getting
    /// back to the top speed, less the way it makes meanwhile towards where the turn leads, at the top speed. Infinite
    /// where the law turns nothing.
    fn turn_time(&self, from: f32, angle: f32, sin: f32, cos: f32, v: f32) -> f32 {
        let rate = self.rate(v);
        if rate <= 0.0 || self.top <= 0.0 {
            return f32::INFINITY;
        }
        let change = if v <= from {
            (from - v) / self.decel
        } else {
            (v - from) / self.accel
        };
        let changing = (from + v) * 0.5 * change;
        let after = (self.top - v).max(0.0) / self.accel;
        let rejoining = (self.top + v) * 0.5 * after;
        change + angle / rate + after - (cos * changing + v * sin / rate + rejoining) / self.top
    }

    /// The corner choice (module docs): the speed at which a unit that drives along the unit vector `travel` from `at`,
    /// facing `facing`, takes a turn of `turn` (radians, signed: clockwise on screen if positive) onto its new way,
    /// from the speed `from`. Of the speeds from a stop to the top speed, the one whose turn takes least time
    /// ([`Tracked::turn_time`]), else the next slower one whose arc fits ([`Tracked::arc_fits`]); a stop and spin if
    /// none does.
    pub(super) fn corner_speed(&self, world: &World, at: Vec2, facing: f32, travel: Vec2, turn: f32, from: f32) -> f32 {
        let angle = turn.abs();
        if angle == 0.0 || self.top <= 0.0 {
            return self.top;
        }
        let speed = |k: u32| self.top * k as f32 / CORNER_SPEEDS as f32;
        (1..=self.soonest(from, angle))
            .rev()
            .map(speed)
            .find(|&v| self.arc_fits(world, at, facing, travel, turn, v))
            .unwrap_or(0.0)
    }

    /// Which of the corner choice's speeds, `top × k ÷ CORNER_SPEEDS`, makes a turn of `angle` (radians, `0..=π`) from
    /// the speed `from` soonest ([`Tracked::turn_time`]): its `k`, the slowest of equals.
    fn soonest(&self, from: f32, angle: f32) -> u32 {
        let (sin, cos) = (math::sin(angle), math::cos(angle));
        let mut best = (0, self.turn_time(from, angle, sin, cos, 0.0));
        for k in 1..=CORNER_SPEEDS {
            let v = self.top * k as f32 / CORNER_SPEEDS as f32;
            let time = self.turn_time(from, angle, sin, cos, v);
            if time < best.1 {
                best = (k, time);
            }
        }
        best.0
    }

    /// Whether the tightest arc at the speed `v` (> 0) of a unit at `at`, facing `facing` and driving along `travel`,
    /// that turns it by `turn` (signed, radians), is clear of the blocked tiles of its nav grid: its Silhouette, turned
    /// as far as the arc has turned it, at every `ARC_TEST_RAD` of the turn and at its end (`Shape::clear_on`).
    /// Units and the map's edge aside from tiles are the collision's to meet (`World::shuffle`, `World::turn`).
    fn arc_fits(&self, world: &World, at: Vec2, facing: f32, travel: Vec2, turn: f32, v: f32) -> bool {
        let Some(grid) = world.grid else {
            return true;
        };
        let radius = v / self.rate(v);
        let angle = turn.abs();
        // Towards the side it turns to: clockwise on screen (+y is south) for a positive turn.
        let side = Vec2::new(-travel.y, travel.x) * turn.signum();
        let parts = (angle / ARC_TEST_RAD).ceil().max(1.0) as u32;
        let body = world.body();
        (1..=parts).all(|part| {
            let phi = angle * part as f32 / parts as f32;
            let pos = at + travel * (radius * math::sin(phi)) + side * (radius * (1.0 - math::cos(phi)));
            let shape = match body.oriented {
                Some(oriented) => Shape::oriented(pos, oriented.half, math::facing_vector(facing + phi.copysign(turn))),
                None => Shape::rect(pos, body.half),
            };
            shape.clear_on(grid)
        })
    }

    /// The corner ahead (module docs): the most a unit at `pos` with the speed `speed` (0 or more) may go this tick to
    /// be at the corner speed of its steering waypoint `corner` (the turn from the way there onto the leg to `beyond`)
    /// once it is within `step` of it, where it passes it. Braking by Deceleration from it each tick brings it to that
    /// speed in time. Infinite where it needs no braking yet: it could still stop in the room it has, or the turn is
    /// one it wouldn't slow for.
    pub(super) fn ahead(&self, world: &World, pos: Vec2, corner: Vec2, beyond: Vec2, step: f32) -> f32 {
        let to = corner - pos;
        let dist = to.length();
        let room = dist - step;
        let out = beyond - corner;
        if room <= 0.0 || self.approach(room) >= self.top || out == Vec2::ZERO {
            return f32::INFINITY;
        }
        let travel = to / dist;
        let turn = math::wrap_angle(math::vector_angle(out) - math::vector_angle(travel));
        if out.length() * math::sin(turn.abs().min(core::f32::consts::FRAC_PI_2)) < TRACKED_MISS_SU {
            return f32::INFINITY;
        }
        let facing = math::vector_angle(if self.reverses { -travel } else { travel });
        let corner_speed = self.corner_speed(world, corner, facing, travel, turn, self.top);
        self.brake_to(room, corner_speed)
    }

    /// The highest speed for this tick's move from which braking by Deceleration every tick brings the unit to the
    /// speed `v` within `dist` space units: its moves `v + n × Deceleration`, … , `v + Deceleration` fit in it, the
    /// first a little longer by what is left over. At most the top speed.
    fn brake_to(&self, dist: f32, v: f32) -> f32 {
        if !self.decel.is_finite() {
            return self.top;
        }
        let (mut n, mut sum) = (0.0_f32, 0.0_f32);
        loop {
            let next = v + (n + 1.0) * self.decel;
            if next > self.top || sum + next > dist {
                break;
            }
            n += 1.0;
            sum += next;
        }
        let longer = ((dist - self.decel * n * (n + 1.0) * 0.5) / (n + 1.0) - v).max(0.0);
        (v + n * self.decel + longer).min(self.top)
    }

    /// Steps 1 to 4 (module docs) for a unit on its way: `probe` is the step from `from` that the shared rules gave a
    /// unit without momentum, at the approach speed, for the step `asked` of them, and `lands` is set when it lands
    /// exactly on the goal; `ahead` is the most the corner ahead lets it go ([`Tracked::ahead`]) and `aim` how far its
    /// steering waypoint is. It turns to head along the step and wants its length as its speed, at most the corner
    /// choice's for the turn onto it ([`Tracked::corner_speed`]). If it makes that step exactly this tick, the step is
    /// its move as the rules gave it; otherwise it moves by its speed along its heading, as far as tiles and units
    /// allow. Where the rules' step is a slide along tiles, or none, it heads for the step it asked for, as
    /// `super::wheeled` does. A move that met something, or a turn cut short, leaves it no momentum (design change
    /// 0035). `touched` is scratch.
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
        ahead: f32,
        aim: f32,
        mover: &mut Mover,
        world: &mut World,
        touched: &mut Vec<usize>,
    ) -> Driven {
        // The rules' step, unless it is a slide or nothing: then the step asked of them.
        let ruled = !probe.slid && probe.pos != from;
        let step = if ruled { probe.pos - from } else { asked };
        let length = step.length();
        let start = *facing;
        let current = speed_of(mover.momentum, math::facing_vector(start));
        let travel = self.travel(start);
        let turn = if step == Vec2::ZERO {
            0.0
        } else {
            math::wrap_angle(math::vector_angle(step) - math::vector_angle(travel))
        };
        // The turn now: none for a miss it wouldn't slow down for.
        let misses = aim * math::sin(turn.abs().min(core::f32::consts::FRAC_PI_2));
        let turning = if turn == 0.0 || misses < TRACKED_MISS_SU {
            f32::INFINITY
        } else {
            self.corner_speed(world, from, start, travel, turn, current.abs())
        };
        let want = length.min(ahead).min(turning);
        let speed = self.next_speed(current, want);
        // Braking to set off the other way: Stopping, which turns by the law all the same.
        let stopping = speed != 0.0 && (speed < 0.0) != self.reverses;
        let turned_to = (step != Vec2::ZERO).then(|| {
            let along = if self.reverses { -step } else { step };
            math::turn_toward(facing, math::wrap_angle(math::vector_angle(along)), self.rate(speed))
        });
        let turned = world.turn(from, start, *facing);
        let mut set_off = from;
        if turned.cut {
            // A spin cut short stops at contact (slice M5-V1). A turn cut while it moves is made all the same, as a
            // wheeled unit's is, backing off the way it came by what its corners sweep (`World::back_off`).
            let most = math::wrap_angle(*facing - start).abs() * world.body().oriented.map_or(0.0, |o| o.half.length());
            let back = math::facing_vector(start) * -speed.signum();
            match (speed != 0.0)
                .then(|| world.back_off(from, *facing, back, most))
                .flatten()
            {
                Some(at) => set_off = at,
                None => *facing = turned.facing,
            }
        }
        let angle = math::wrap_angle(*facing - start);
        let done = turned_to.is_none_or(|turn| turn.done) && !turned.cut;
        let exact = done
            && ruled
            && !stopping
            && (speed.abs() == length || (lands && speed.abs() >= length))
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
    /// (`super::doorway`). It turns towards `dir` by what the law allows at its speed, in place at rest, and its speed
    /// along the line goes from that of `momentum` towards `want`. Returns its speed along `dir` for this tick (0 or
    /// more) and the angle it turned.
    pub(super) fn along(&self, dir: Vec2, want: f32, facing: &mut f32, momentum: Vec2) -> (f32, f32) {
        let speed = self.next_speed(momentum.dot(dir), want).max(0.0);
        let wanted = math::wrap_angle(math::vector_angle(dir));
        let turn = math::turn_toward(facing, wanted, self.rate(speed));
        (speed, turn.angle)
    }

    /// Steps 1, 3 and 4 for a unit that isn't steering this tick (no goal, no path yet, held by its attack) and still
    /// has momentum: it brakes by Deceleration along its facing, which stays as it is, and moves by what is left, as
    /// far as tiles and units allow. It stands once at rest, or when it meets a tile or a unit (design change 0035).
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
