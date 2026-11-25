//! DragMovement — `systems/units.md > DragMovement`, `systems/units.md > HoverVehicle` (slice M3-G, plan decision T25):
//! the model of the units that keep their velocity from tick to tick ([`Mover::momentum`]). It replaces only the
//! "desired direction → velocity and facing" part of a unit's tick (`docs/movement.md > 6.3`, `docs/movement.md >
//! 6.7`): path following, lane seeking, the rules after the sweep, yielding and stuck detection are shared with every
//! other model (`super::follow`).
//!
//! **The integration scheme**, fixed (T25), once per tick, in this order:
//! 1. **Turn** towards the step the unit wants to make (below), at most TurnRate (`systems/combat.md >
//!    LocomotionOrientationConstraints[DragMovement]`: the unit turns at TurnRate whatever it does, and never stands
//!    to turn). The forward thrust is three quarters of the Drone's, so it faces the way it pushes: along its
//!    path in the open, along a blocker's face while it goes round it.
//! 2. **Drag**: `v ← v × (1 − DragRatio)`, DragRatio doubled while the tile under the unit's centre is Rugged
//!    (`systems/units.md > HoverVehicle > RuggedTerrain`, `systems/units.md > Which tile counts`; design change 0039).
//! 3. **Thrust**: `v ← v + a`, where `a` is the thrust that brings `v` nearest to the velocity the unit wants
//!    ([`Drag::thrust`]): OmniDirectionalAcceleration any way, plus up to ForwardAcceleration along the facing
//!    only. Steps 2 and 3 together are `v ← v + a − DragRatio × v`: the drag is taken on the velocity the tick
//!    began with, so thrust and drag balance at `(Omni + Forward) / DragRatio`, the design's MaxSpeed
//!    (8 su per tick for the Drone, 4 on Rugged).
//! 4. **Position**: `pos ← pos + v`, as far as tiles and units allow. If the move runs into an obstacle, "the
//!    blocked portion of the movement is dropped … and its velocity is set to 0" (`systems/units.md >
//!    UnitCollision > Collision Response (Un-sticking)`, design change 0035): all of it, not only the part into
//!    the obstacle. "A unit pressed against an obstacle restarts from 0 every frame, so it moves at
//!    single-frame-acceleration speed … It gradually scrapes around the obstacle until it is free, then
//!    accelerates normally." A unit it passes through is no obstacle.
//!
//! **What the unit wants** is the step a unit without momentum would take ([`Drag::drive`]): the shared rules
//! turn the way to the steering waypoint into a step of the **approach speed** (lane steps, slides, steps around
//! a blocker), and the thrust steers the velocity towards that step. So a hover unit goes around what the others
//! go around, a little wider.
//!
//! **Braking and arrival** ([`approach_speed`]): the unit brakes by thrusting against its motion ("ground hover
//! units … actively thrust in the opposing direction to decelerate"), with the omni-directional thrust alone, as
//! it faces the way it goes (it doesn't turn its back to its goal to brake with its forward thrust: the design
//! doesn't say it would). The approach speed is the highest speed from which that braking still stops it
//! within the path that remains, so **braking begins when the stopping distance reaches the remaining path**, and
//! the last step lands exactly on the goal at a speed one tick of braking takes away: no overshoot, no
//! oscillation. It plans with 15/16 of that thrust ([`PLANNED_BRAKE`]). From top speed the Drone, with its numbers
//! of design change 0205, begins to brake 35 su before its goal (one more step, then 27 su and 10 ticks of
//! braking), and stops within 26 su and 9 ticks when it is stopped ([`tests`]). Before 0205 it was 51 su (43 su
//! and 16 ticks), and 42 su and 16 ticks.
//!
//! **Without a goal** ([`Drag::coast`]) a unit that still has momentum brakes to a stop the same way, moving
//! meanwhile (`Locomotion::Stopping`): a Stop order, an arrival within reach of an object, a failed move. It is
//! one of the ways a unit without a goal moves, with a sidestep, a doorway and a step off a unit it overlaps
//! (invariants I5 and I7).
//!
//! Float rules (`docs/movement.md > 7`): `+ − × ÷` and `sqrt` only, but for the facing's unit vector, which goes
//! through `sc_sim::math`. Every position is committed through the same predicates as any other move
//! (`World::clear`, `World::shuffle`), so the model adds no way for two boxes to come to overlap.

use glam::Vec2;

use super::collide::{Moved, World};
use crate::defs::MovementAttrs;
use crate::math;
use crate::mover::{Locomotion, Mover};

/// The share of its omni-directional thrust a unit plans its braking with ([`Drag::approach`]). The rest is its
/// reserve: a plan that rode the limit exactly would be pushed off it by the rounding of a tick (a position near
/// the far edge of the largest map resolves 0.002 su), and the unit would pass its goal by as much and have to
/// come back. With a sixteenth in hand it stays on its plan to the last step and lands exactly.
const PLANNED_BRAKE: f32 = 15.0 / 16.0;

/// One unit's DragMovement numbers for this tick, in space units and ticks.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(super) struct Drag {
    /// What a tick's drag leaves of the velocity: `1 − DragRatio`, the ratio doubled on Rugged terrain.
    keep: f32,
    /// ForwardAcceleration: thrust along the facing only.
    forward: f32,
    /// OmniDirectionalAcceleration: thrust any way. It is all the unit brakes with.
    omni: f32,
    /// Where thrust and drag balance on this terrain: `(omni + forward) / drag`.
    top: f32,
}

impl Drag {
    /// The model of a unit with these attributes, `None` for every other movement model. `speed_factor` is the
    /// mover's (a mode that slows the unit scales its thrust, and its top speed with it); `rugged` is whether the
    /// tile under the unit's centre is Rugged terrain.
    pub(super) fn of(movement: &MovementAttrs, speed_factor: f32, rugged: bool) -> Option<Self> {
        let MovementAttrs::Drag {
            forward_acceleration,
            omni_directional_acceleration,
            drag_ratio,
            ..
        } = *movement
        else {
            return None;
        };
        // A ratio of 1 or more would stop the unit every tick: the velocity never turns round by drag alone.
        let drag = (if rugged { drag_ratio * 2.0 } else { drag_ratio }).min(1.0);
        let (forward, omni) = (
            forward_acceleration * speed_factor,
            omni_directional_acceleration * speed_factor,
        );
        Some(Self {
            keep: 1.0 - drag,
            forward,
            omni,
            top: (forward + omni) / drag,
        })
    }

    /// The speed the unit approaches a goal `dist` space units away at ([`approach_speed`]), braking with
    /// [`PLANNED_BRAKE`] of its omni-directional thrust.
    pub(super) fn approach(&self, dist: f32) -> f32 {
        approach_speed(dist, self.keep, self.omni * PLANNED_BRAKE, self.top)
    }

    /// Step 3 of the scheme: the thrust nearest to `want`, the change of velocity the unit would like, with the
    /// unit facing along the unit vector `facing`, and whether it is `want` itself. The thrusts a unit can make
    /// are every vector of at most `omni` any way, plus between nothing and `forward` along the facing: the
    /// nearest of them takes as much of `want` along the facing as the forward thrust gives, and of the rest as
    /// much as the omni-directional thrust reaches.
    fn thrust(&self, want: Vec2, facing: Vec2) -> (Vec2, bool) {
        let along = facing * want.dot(facing).clamp(0.0, self.forward);
        let rest = want - along;
        let (length, limit) = (rest.length(), self.omni);
        if length <= limit {
            (want, true)
        } else {
            (along + rest * (limit / length), false)
        }
    }

    /// Steps 2 to 4 for a unit on its way: `probe` is the step from `from` that the shared rules gave a unit
    /// without momentum, at the approach speed, for the step `asked` of them. The unit thrusts towards that step.
    /// If its thrust reaches it, the step is the unit's move as it is (the rules already found it free, and an
    /// arrival lands exactly); otherwise it moves by the velocity it got, as far as tiles and units allow, the two
    /// axes one after the other where the whole move isn't free ([`World::shuffle`]).
    ///
    /// **Blocked** (design change 0035): if the move runs into a tile or a unit, what was blocked of it is
    /// dropped and **the momentum is zero**: the unit starts again from rest in its next tick. Where the rules'
    /// step is itself a slide along tiles (`Moved::slid`), the unit thrusts towards the step it asked for, into
    /// the tile, and meets it: "running into the wall every frame", it creeps along the wall at single-tick
    /// acceleration until its way is free. A unit it passes through (`space::blocks`) is no obstacle and takes
    /// nothing. `touched` is scratch.
    #[expect(
        clippy::too_many_arguments,
        reason = "the unit's state, split into the borrows the step holds"
    )]
    pub(super) fn drive(
        &self,
        from: Vec2,
        facing: &mut f32,
        turn_rate: f32,
        asked: Vec2,
        probe: Moved,
        mover: &mut Mover,
        world: &mut World,
        touched: &mut Vec<usize>,
    ) -> (Moved, f32) {
        // Step 1: it turns towards the step it wants, to put its forward thrust behind it.
        let step = if probe.slid { asked } else { probe.pos - from };
        let turned = if step == Vec2::ZERO {
            0.0
        } else {
            math::turn_toward(facing, math::wrap_angle(math::vector_angle(step)), turn_rate).angle
        };
        let kept = mover.momentum * self.keep;
        let (thrust, reached) = self.thrust(step - kept, math::facing_vector(*facing));
        if reached && !probe.slid {
            mover.momentum = step;
            return (probe, turned);
        }
        let velocity = if reached { step } else { kept + thrust };
        touched.clear();
        let pos = world.shuffle(from, velocity, touched);
        // The whole move, or it met something: then it has no velocity left.
        mover.momentum = if pos == from + velocity { pos - from } else { Vec2::ZERO };
        let moved = Moved {
            pos,
            // Getting on is the rules' word, as long as the unit moved at all.
            forward: probe.forward && pos != from,
            blocker: probe.blocker.or(touched.first().copied()),
            aside: probe.aside,
            slid: probe.slid,
        };
        (moved, turned)
    }

    /// (M3-D2) Steps 1 to 3 for a unit held to the line along the unit vector `dir`: one leaving a portal's energy
    /// doorway (`super::doorway::emerge`, design change 0145). It turns towards `dir`, at most `turn_rate`, and
    /// thrusts to bring its velocity, from `momentum`, nearest to `want` along the line. What drag and thrust leave
    /// across the line is dropped, so it never leaves the line. Returns its speed along `dir` for this tick (0 or
    /// more; `want` when the thrust reaches it) and the angle it turned.
    pub(super) fn along(&self, dir: Vec2, want: f32, facing: &mut f32, turn_rate: f32, momentum: Vec2) -> (f32, f32) {
        let turned = math::turn_toward(facing, math::wrap_angle(math::vector_angle(dir)), turn_rate).angle;
        let kept = momentum * self.keep;
        let (thrust, reached) = self.thrust(dir * want - kept, math::facing_vector(*facing));
        let speed = if reached {
            want
        } else {
            (kept + thrust).dot(dir).max(0.0)
        };
        (speed, turned)
    }

    /// Steps 2 to 4 for a unit that isn't steering this tick (no goal, no path yet, held by its attack) and still
    /// has momentum: it brakes, thrusting against its motion, and moves by what is left, as far as tiles and
    /// units allow. It stands once a tick's thrust takes all that the drag left, or when it meets a tile or a
    /// unit (design change 0035: no velocity is left). The facing stays as it is.
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
        let kept = mover.momentum * self.keep;
        let (thrust, stopped) = self.thrust(-kept, math::facing_vector(facing));
        let from = *pos;
        let velocity = if stopped { Vec2::ZERO } else { kept + thrust };
        if !stopped {
            touched.clear();
            *pos = world.shuffle(from, velocity, touched);
        }
        mover.velocity = *pos - from;
        mover.momentum = if *pos == from + velocity {
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

/// The highest speed, at most `top`, at which a unit may make this tick's step towards a goal `dist` space units
/// along its path and still stop on it: after a step of `s`, braking every tick (`v ← v × keep − brake`, moving
/// by the new `v`) must bring it to rest within `dist − s`. Space units per tick.
///
/// The braking steps are built from the goal backwards. The last step is made at a speed one tick of braking
/// takes away, `brake / keep` at most; each step before it is `(v + brake) / keep`, the speed braking leaves `v`
/// of. The longest run of them that fits into `dist` is shortened evenly from its last step, down to the exact
/// length. Within `brake / keep` of the goal the speed is `dist` itself: the unit lands on the goal.
///
/// It only falls as `dist` does, and never faster than braking follows: a unit at the approach speed of one tick
/// is within the next tick's by braking alone.
fn approach_speed(dist: f32, keep: f32, brake: f32, top: f32) -> f32 {
    // No drag left to speak of, or nothing to brake with: the unit stops within the tick, or by drag alone (it
    // then covers `v × keep / (1 − keep)` more).
    if keep <= 0.0 {
        return dist.min(top);
    }
    if brake <= 0.0 {
        return (dist * (1.0 - keep)).min(top);
    }
    let last = brake / keep;
    if dist <= last {
        return dist.min(top);
    }
    // `speed`: the first step of the longest run built so far, each step at its fastest; `total`: its length;
    // `weight`: how much of a change of the last step reaches the first, `keep⁻ⁿ`; `weights`: and the whole run.
    let (mut speed, mut total, mut weight, mut weights) = (last, last, 1.0_f32, 1.0_f32);
    while total < dist && speed < top {
        speed = (speed + brake) / keep;
        weight /= keep;
        total += speed;
        weights += weight;
    }
    if total <= dist {
        return speed.min(top);
    }
    (speed - (total - dist) * weight / weights).min(top)
}

#[cfg(test)]
mod tests;
