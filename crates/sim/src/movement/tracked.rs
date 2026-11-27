//! SpeedTurnRadiusMovement — `systems/units.md > SpeedTurnRadiusMovement` ("Unit can rotate in place by spinning
//! treads in opposite directions. Turn radius increases with speed ... Small directional adjustments at speed cost
//! little momentum, but sharp turns require significant slowing or a complete stop"), the model of the TrackedVehicle
//! (slice M5-V3, plan decision T85 as design change 0417 replaced its law). Like FixedTurnRadiusMovement
//! (`super::wheeled`), whose structure it takes, it keeps the unit's speed from tick to tick (`Mover::momentum`) and
//! replaces only the "desired direction → velocity and facing" part of a unit's tick: path following, lane seeking,
//! the rules after the sweep, yielding and stuck detection are shared with every model (`super::follow`).
//!
//! **The tread law** (design change 0417, `systems/units.md > SpeedTurnRadiusMovement > The turning law`): "forward
//! speed is their average, turning comes from their difference, and each tread's top speed is MaxSpeed". Stopped, it
//! spins at **SpinRate = 2 × MaxSpeed ÷ TrackWidth** a tick; at speed `v` it turns at most **SpinRate × (1 − |v| ÷
//! MaxSpeed)** ([`Tracked::rate`]), on a tightest radius of `(TrackWidth ÷ 2) × |v| ÷ (MaxSpeed − |v|)`, which is `|v|`
//! over that rate; at MaxSpeed it can't turn at all. Backing up it turns by the same law with `|v|`. Turning costs no
//! speed by itself.
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
//!    stays free (`World::turn`, slice M5-V1). A turn cut short is made all the same, as V2's is, backing off by what
//!    its corners sweep (`World::back_off`), but by no more than the unit moves in the tick: its speed, or at rest
//!    one tick's acceleration, away from its way. Else it **stops at contact**.
//! 3. **Displacement**: the speed along the heading halfway through the tick's turn, as far as tiles and units allow
//!    (`World::shuffle`).
//! 4. **Blocked** (design change 0035): a move that met something, or a turn cut short, leaves no momentum.
//!
//! **At rest** (`docs/movement.md > 6.12`; design change 0035: "Blocked units don't freeze; they creep and turn out of
//! the contact"):
//! - **A glance**, the rules' step along the face of a unit it met that doesn't move away ([`glances`]), is taken as
//!   it is where the unit would spin to head along it: its facing kept and no speed carried, as it takes its contact
//!   step (design change 0323). A glance depends on how the Silhouette lies, and a spin would turn the Silhouette into
//!   the contact or out of it, and the glance with it, so that the unit turned to and fro where it stood. A slide
//!   along tiles is no glance: there the unit heads along the step it asked for, which its Silhouette doesn't change.
//! - **A spin cut short** that backing off doesn't free takes the rules' step as far as its Silhouette lies free where
//!   the spin stopped, so it creeps along what it is pressed against; with no such step it stays.
//!
//! **The corner choice** ([`corner`]; T85 with 0417): the speed it takes a turn at, from full speed down to a stop and
//! spin, the soonest whose arc fits the free space on the nav grid; for the turn now, onto the step it takes, and for
//! the corner ahead, at its steering waypoint.
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
use crate::tuning::movement::TRACKED_MISS_SU;

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

/// Whether a unit whose steering waypoint is `aim` away makes the turn `turn` (signed, radians) onto its step now (the
/// corner choice's turn now, module docs): a heading that misses the waypoint by less than `TRACKED_MISS_SU` is held as
/// it is.
fn turns_now(turn: f32, aim: f32) -> bool {
    turn != 0.0 && aim * math::sin(turn.abs().min(core::f32::consts::FRAC_PI_2)) >= TRACKED_MISS_SU
}

/// Whether the shared rules' step `probe` from `from`, for the step `asked` of them, is a glance (module docs, at
/// rest): a slide along the face of a unit it met that doesn't move away along `asked` (behind one that does, the step
/// closes up along `asked`), which moves the unit. A slide along tiles is no glance: the unit heads along `asked`
/// there, which doesn't depend on how its Silhouette lies, so it spins to its way and creeps along them.
fn glances(world: &World, probe: &Moved, asked: Vec2, from: Vec2) -> bool {
    probe.pos != from
        && probe.forward
        && !probe.aside
        && !probe.slid
        && probe
            .blocker
            .is_some_and(|blocker| world.bodies[blocker].velocity.dot(asked) <= 0.0)
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
        let turning = if turns_now(turn, aim) {
            self.corner_speed(world, from, start, travel, turn, current.abs())
        } else {
            f32::INFINITY
        };
        let want = length.min(ahead).min(turning);
        let speed = self.next_speed(current, want);
        // At rest, a glance it would spin to head along is taken as it is (module docs).
        if current == 0.0 && speed == 0.0 && turns_now(turn, aim) && glances(world, &probe, asked, from) {
            mover.momentum = Vec2::ZERO;
            return Driven {
                moved: probe,
                velocity: probe.pos - from,
                turned: 0.0,
                locomotion: Locomotion::Moving,
                cut: None,
            };
        }
        // Braking to set off the other way: Stopping, which turns by the law all the same.
        let stopping = speed != 0.0 && (speed < 0.0) != self.reverses;
        let turned_to = (step != Vec2::ZERO).then(|| {
            let along = if self.reverses { -step } else { step };
            math::turn_toward(facing, math::wrap_angle(math::vector_angle(along)), self.rate(speed))
        });
        let turned = world.turn(from, start, *facing);
        let mut set_off = from;
        if turned.cut {
            // A turn cut short is made all the same, as a wheeled unit's is, backing off the way it came by what its
            // corners sweep (`World::back_off`), but by no more than it moves in the tick: its speed, or from rest one
            // tick's acceleration, backing away from its way (it creeps out of the contact, design change 0035). Slow,
            // the law turns it far more than a wheeled unit's speed does, and its corners sweep farther than it moves.
            // Else it stops at contact (slice M5-V1).
            let swept =
                math::wrap_angle(*facing - start).abs() * world.body().oriented.map_or(0.0, |o| o.half.length());
            let (most, back) = if speed != 0.0 {
                (speed.abs(), math::facing_vector(start) * -speed.signum())
            } else {
                (self.accel.min(self.top), -travel)
            };
            match (swept > 0.0)
                .then(|| world.back_off(from, *facing, back, swept.min(most)))
                .flatten()
            {
                Some(at) => set_off = at,
                None => *facing = turned.facing,
            }
            if speed == 0.0 && set_off == from && probe.pos != from {
                // At rest, its spin cut short, it takes the rules' step as it is, as far as its Silhouette lies free
                // where the spin stopped (module docs).
                let step = probe.pos - from;
                touched.clear();
                let pos = if world.clear(from, step, touched) {
                    from + step
                } else {
                    world.shuffle(from, step, touched)
                };
                mover.momentum = Vec2::ZERO;
                return Driven {
                    moved: Moved {
                        pos,
                        forward: probe.forward && pos != from,
                        blocker: probe.blocker.or(touched.first().copied()),
                        aside: probe.aside,
                        slid: probe.slid,
                    },
                    velocity: pos - from,
                    turned: math::wrap_angle(*facing - start).abs(),
                    locomotion: Locomotion::Moving,
                    cut: Some(turned.blocker),
                };
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

mod corner;
#[cfg(test)]
mod tests;
