//! The corner choice of SpeedTurnRadiusMovement (`super`; slice M5-V3, T85 with design change 0417, the slice's rule:
//! "choose the speed to take it at, from full speed down to a stop and spin. Take the fastest whose arc fits the free
//! space on the nav grid, counting the time to brake to it"): analytic, from the law, with no tables
//! ([`Tracked::corner_speed`]). For a turn of θ it compares `CORNER_SPEEDS` + 1 speeds `v` from a stop to its top speed
//! by the time each takes, counted together: changing speed to `v` from the speed it has, turning θ at the law's rate
//! at `v`, getting back to its top speed, less the way it makes meanwhile towards where the turn leads (its braking
//! along its old heading, `cos θ` of it; the arc's `r × sin θ`; the acceleration after). It takes the soonest of them
//! whose arc, the tightest at `v` from where it starts until it heads the new way, tested every `ARC_TEST_RAD` in its
//! own Silhouette, is clear of the nav grid's blocked tiles ([`Tracked::arc_fits`]); a stop and spin when none is. So
//! it depends only on the angle, the speed it has and the room the arc needs, never on an estimate of a path:
//! predictable (design change 0223). For the Hand of Heaven from full speed: a gentle 15° at 4.5 su a tick, 45° at 3.9,
//! a right angle at 2.8 (on the law's 19 su radius, nearly pivoting about its inner tread), a turn about at 0.9
//! (`super::tests`). From rest, going forward, every turn is a spin first: from rest turning in place is the quickest
//! turn there is. Backing up, its top speed is ReverseMaxSpeed, the Hand of Heaven's half its MaxSpeed, at which the
//! law still turns it at half SpinRate: from rest it backs at that speed as it turns, up to about 60°, slower up to
//! about 105°, and spins first beyond.
//! - **The turn now**: each tick, for the turn from its heading onto the step it takes ([`Tracked::drive`]): its speed
//!   is at most the choice's. A heading that misses the steering waypoint by less than `TRACKED_MISS_SU` is held as it
//!   is: at full speed the law turns it by nothing.
//! - **The corner ahead**: the turn at its steering waypoint, from the way there onto the path's next leg
//!   ([`Tracked::ahead`]): the unit brakes in time to be at that corner's speed once it is within a step of the
//!   waypoint, where `movement::follow` passes it and the turn now takes over. Only within braking distance of it, and
//!   not within `TRACKED_MISS_SU` of where it passes it.

use glam::Vec2;

use super::Tracked;
use crate::math;
use crate::movement::collide::World;
use crate::shape::Shape;
use crate::tuning::movement::{ARC_TEST_RAD, CORNER_SPEEDS, TRACKED_MISS_SU};

impl Tracked {
    /// The time a turn of `angle` (radians, `0..=π`; `sin` and `cos` its sine and cosine) takes at the speed `v`, from
    /// the speed `from` (module docs), in ticks: changing to `v`, the arc at the law's rate, getting
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
    pub(super) fn soonest(&self, from: f32, angle: f32) -> u32 {
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
    /// one it wouldn't slow for; and within `TRACKED_MISS_SU` of where it passes it, which it counts as there (else,
    /// braking to a spin, it would close in by ever shorter moves, the last lost to rounding, and never get there).
    pub(in crate::movement) fn ahead(&self, world: &World, pos: Vec2, corner: Vec2, beyond: Vec2, step: f32) -> f32 {
        let to = corner - pos;
        let dist = to.length();
        let room = dist - step;
        let out = beyond - corner;
        if room < TRACKED_MISS_SU || self.approach(room) >= self.top || out == Vec2::ZERO {
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
    pub(super) fn brake_to(&self, dist: f32, v: f32) -> f32 {
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
}
