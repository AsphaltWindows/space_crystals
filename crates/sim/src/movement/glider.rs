//! (M5-H1) GliderMovement — `systems/units.md > GliderMovement` (plan decision T81): "Unit must always maintain
//! movement to stay airborne. When idle, the unit circles at a low speed in tight loops. When given orders, the unit
//! accelerates to max speed. Turn radius is governed by centripetal acceleration (r = v²/a), so higher speeds produce
//! wider turns and lower speeds allow tighter circles."
//!
//! A Glider's tick ([`Glider::fly`]) takes the place of a unit's tick along its path. It collides with nothing
//! (`systems/units.md > Air Collision`: "Air units ignore terrain", they "do not collide with ground units or
//! structures"; its soft separation from other air units comes after every unit has moved, `super::air`), it flies
//! straight at what it heads for (its path is the straight line, `paths::serve`), and it never stands, waits, yields or
//! is held by its attack: its base is free while its turret fires (`combat::base_is_free`), and one without a turret
//! couldn't stop either.
//!
//! - **Its speed** is the length of its momentum (`Mover::momentum`), which it carries from tick to tick as a
//!   DragMovement unit does, along its facing. It stays between IdleSpeed and MaxSpeed and changes by at most its
//!   Acceleration or Deceleration in a tick (`None`: at once, as for the other models). It wants MaxSpeed while it has
//!   a goal ("When given orders, the unit accelerates to max speed") and IdleSpeed while it circles. One that hasn't
//!   flown yet starts at IdleSpeed: **Placeholder (not in the design: a Glider that comes into the game in the air, as
//!   a debug spawn does, is flying at IdleSpeed)**; the Airstrip's takeoff is slice N7's.
//! - **Its turn** in a tick is at most MaxCentripetalAcceleration / v radians at the speed v it flies in that tick
//!   (`systems/combat.md > LocomotionOrientationConstraints[GliderMovement]`: "Moving + Turning: maxTurnRate =
//!   f(currentSpeed, MaxCentripetalAcceleration)", "Moving + Maintaining: valid"), so its tightest circle at v has a
//!   radius of v² / a. It turns, then flies its speed along its new facing: its Locomotion is always Moving.
//! - **Circling** ([`Aim::Circle`]): without a goal it circles its orbit (`Mover::orbit`) at IdleSpeed, on the idle
//!   circle's radius of [`IDLE_CIRCLE_GU`] (design change 0354: a fixed radius of 1 grid unit), clockwise:
//!   **Placeholder (not in the design: which way a Glider circles)**. Its heading blends the circle's tangent with the
//!   way to the circle, by how far off the circle it is ([`CIRCLE_PULL`]), plus half a tick's turn along the circle,
//!   so that from the circle it flies the chord to the circle's next point. A Glider whose orbit isn't set yet (its
//!   goal ended, or it was stopped) circles the point of the map nearest to where it is, which becomes its orbit.
//! - **A goal** (a point, or an object as this phase finds it, `super::Chased`): it flies at it at MaxSpeed. It has
//!   arrived (`MoveStatus::Arrived`) once within the idle circle's radius of a point, or within the goal's reach of an
//!   object (its gap, edge to edge, as the other models measure it), and circles the goal's point from then on
//!   (`systems/controls/command_behaviors.md > MovingToLocation`, GliderMovement exception: "Final step is Locomotion
//!   = Moving(idle circle over TargetLocation), Orientation = Turning(circle path)"). An object that is gone ends the
//!   goal as `TargetLost`. A point inside its turn at the speed it has, which it would circle without ever reaching, it
//!   slows down for, to IdleSpeed, until it can turn onto it: **Placeholder (not in the design: how a Glider reaches a
//!   point just beside it)**.
//! - **A strafing run** (`Mover::strafe`, [`Pass`]): it flies at its goal at MaxSpeed and never arrives. Once the
//!   goal is behind it (more than 90° off its heading) it has passed it, and holds its course until the gap to the goal
//!   is more than the run's range; then it turns back toward the goal at MaxSpeed, its tightest turn at that speed,
//!   and the next pass begins once the goal is ahead of it again (`factions/cults/units/sparrow.md > Flight rules`:
//!   "Strafing: after a pass it flies on until the target is out of range, then loops back at full speed"). A run
//!   begins as a turn toward the goal, wherever the goal is ("1. Fly toward TargetObject").
//! - **The map's edge**: nothing keeps it on the map (`systems/units.md > Air Collision`: "Gliders may unavoidably
//!   travel a little outside the map edge while turning or circling"; `factions/cults/units/sparrow.md > Flight
//!   rules`: "Map edge: it may overshoot the edge as far as its turn needs (no extra cap)"). Its goals and orbits are
//!   on the map (the gate clamps orders' points onto it, `Sim::check_order`; an orbit is a point of the map), so its
//!   turns always bring it back (design change 0045: "Their turning logic should still steer back in"). Off the map
//!   it counts as at the nearest edge tile for everything else (`Object::map_pos`).

use glam::Vec2;

use super::Chased;
use crate::defs::MovementAttrs;
use crate::map::Map;
use crate::math;
use crate::mover::{Locomotion, MoveGoal, MoveStatus, Mover, Orientation, Pass};
use crate::shape::Shape;
use crate::tuning::air::{CIRCLE_PULL, IDLE_CIRCLE_GU};
use crate::units::gu;

/// A Glider's numbers (`MovementAttrs::Glider`, `systems/units.md > GliderMovement`), as its Augmentations make them.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct Glider {
    /// IdleSpeed, space units per tick: its speed while it circles, and its least.
    pub idle_speed: f32,
    /// MaxSpeed, space units per tick.
    pub max_speed: f32,
    /// Acceleration and Deceleration, space units per tick²; `None`: infinite.
    pub acceleration: Option<f32>,
    pub deceleration: Option<f32>,
    /// MaxCentripetalAcceleration, space units per tick².
    pub max_centripetal_acceleration: f32,
}

/// What a Glider flies at in a tick (module docs).
#[derive(Copy, Clone, Debug, PartialEq)]
enum Aim {
    /// Its idle circle round this point, at IdleSpeed.
    Circle(Vec2),
    /// This point, at MaxSpeed; `slows`: unless it lies inside the Glider's turn (a goal it arrives at, not a run).
    At { at: Vec2, slows: bool },
    /// Straight on, at MaxSpeed: past the goal of its strafing run.
    Straight,
}

impl Glider {
    /// The numbers of a unit that moves by `movement`, if it is a Glider.
    pub(crate) fn of(movement: &MovementAttrs) -> Option<Self> {
        match *movement {
            MovementAttrs::Glider {
                idle_speed,
                max_speed,
                acceleration,
                deceleration,
                max_centripetal_acceleration,
            } => Some(Self {
                idle_speed,
                max_speed: max_speed.max(idle_speed),
                acceleration,
                deceleration,
                max_centripetal_acceleration,
            }),
            _ => None,
        }
    }

    /// The radius of its idle circle, space units ([`IDLE_CIRCLE_GU`]).
    pub(crate) fn idle_radius() -> f32 {
        gu(IDLE_CIRCLE_GU)
    }

    /// The speed it flies at after `speed`, wanting `wanted`: by at most its Acceleration or Deceleration, within
    /// IdleSpeed and MaxSpeed.
    fn next_speed(&self, speed: f32, wanted: f32) -> f32 {
        let change = wanted - speed;
        let change = if change > 0.0 {
            self.acceleration.map_or(change, |up| change.min(up))
        } else {
            self.deceleration.map_or(change, |down| change.max(-down))
        };
        (speed + change).clamp(self.idle_speed, self.max_speed)
    }

    /// The most it turns in a tick at `speed`, radians: MaxCentripetalAcceleration / v (module docs).
    pub(crate) fn turn_rate(&self, speed: f32) -> f32 {
        self.max_centripetal_acceleration / speed.max(f32::MIN_POSITIVE)
    }

    /// Phase 5d, one tick of a Glider of half extents `half` (module docs): it settles its goal (arrived, lost, a
    /// strafing run's pass), sets its speed, turns, and flies on. `chased` is the object its goal names, if that
    /// exists, as this phase finds it; `turned` gets the angle it turned.
    #[expect(
        clippy::too_many_arguments,
        reason = "the parts of a unit and of the phase that a Glider's tick reads"
    )]
    pub(crate) fn fly(
        &self,
        pos: &mut Vec2,
        facing: &mut f32,
        turned: &mut f32,
        mover: &mut Mover,
        chased: Option<&Chased>,
        half: Vec2,
        map: &Map,
    ) {
        let heading = math::facing_vector(*facing);
        let aim = self.aim(*pos, heading, half, mover, chased, map);
        let speed = if mover.momentum == Vec2::ZERO {
            self.idle_speed
        } else {
            mover.momentum.length()
        };
        let wanted = match aim {
            Aim::Circle(_) => self.idle_speed,
            Aim::At { at, slows: true } if self.inside_turn(*pos, heading, speed, at) => self.idle_speed,
            Aim::At { .. } | Aim::Straight => self.max_speed,
        };
        let speed = self.next_speed(speed, wanted);
        let toward = match aim {
            Aim::Circle(centre) => circling(*pos, heading, centre, speed),
            Aim::At { at, .. } => at - *pos,
            Aim::Straight => heading,
        };
        *turned = 0.0;
        if toward != Vec2::ZERO {
            let turn = math::turn_toward(facing, math::vector_angle(toward), self.turn_rate(speed));
            *turned = turn.angle;
        }
        mover.orientation = if *turned > 0.0 {
            Orientation::Turning(*pos + toward)
        } else {
            Orientation::Maintaining
        };
        let flight = math::facing_vector(*facing) * speed;
        *pos += flight;
        mover.momentum = flight;
        mover.velocity = flight;
        mover.locomotion = Locomotion::Moving;
    }

    /// What it flies at this tick (module docs), settling its goal: arrived, or lost, it circles; on a strafing run, it
    /// moves on to the run's next part.
    fn aim(&self, pos: Vec2, heading: Vec2, half: Vec2, mover: &mut Mover, chased: Option<&Chased>, map: &Map) -> Aim {
        let shape = Shape::rect(pos, half);
        let goal = match (mover.goal, chased) {
            (MoveGoal::None, _) => None,
            (MoveGoal::Point(at), _) => Some((at, shape.distance_to(at), pos.distance(at) <= Self::idle_radius())),
            (MoveGoal::Object { reach, .. }, Some(target)) => {
                let gap = shape.gap(&target.shape);
                Some((target.pos, gap, gap <= reach))
            }
            (MoveGoal::Object { .. }, None) => {
                mover.finish(MoveStatus::TargetLost);
                None
            }
        };
        let Some((at, gap, arrived)) = goal else {
            return Aim::Circle(*mover.orbit.get_or_insert_with(|| map.clamp_point(pos)));
        };
        let Some(strafe) = &mut mover.strafe else {
            if !arrived {
                return Aim::At { at, slows: true };
            }
            mover.finish(MoveStatus::Arrived);
            mover.orbit = Some(at);
            return Aim::Circle(at);
        };
        let ahead = (at - pos).dot(heading) > 0.0;
        strafe.pass = match strafe.pass {
            Pass::Toward if ahead => Pass::Run,
            Pass::Run if !ahead => Pass::On,
            Pass::On if gap > strafe.range => Pass::Toward,
            pass => pass,
        };
        match strafe.pass {
            Pass::Toward | Pass::Run => Aim::At { at, slows: false },
            Pass::On => Aim::Straight,
        }
    }

    /// Whether the point `at` lies inside the tightest circle it can turn at `speed` toward that side, flying along
    /// `heading` from `pos`: it would circle `at` for ever at that speed, never reaching it.
    fn inside_turn(&self, pos: Vec2, heading: Vec2, speed: f32, at: Vec2) -> bool {
        let to = at - pos;
        let side = heading.perp_dot(to);
        if side == 0.0 {
            return false;
        }
        let radius = speed * speed / self.max_centripetal_acceleration;
        // The centre of that circle: square to the heading, on the side of `at`.
        let normal = Vec2::new(-heading.y, heading.x) * side.signum();
        (at - (pos + normal * radius)).length_squared() < radius * radius
    }
}

/// The way a Glider at `pos`, flying along `heading` at `speed`, wants to go to circle `centre` clockwise on its idle
/// circle (module docs): the circle's tangent where it is, turned toward the circle by [`CIRCLE_PULL`] times its
/// distance off the circle over the radius, and by half the turn of a tick on the circle. Its heading as it is right
/// on the centre, from which every way leads out.
fn circling(pos: Vec2, heading: Vec2, centre: Vec2, speed: f32) -> Vec2 {
    let radius = Glider::idle_radius();
    let off = pos - centre;
    let distance = off.length();
    if distance <= f32::EPSILON * radius {
        return heading;
    }
    let out = off / distance;
    // Clockwise on screen (+y south): the tangent is the outward direction turned a quarter clockwise.
    let tangent = Vec2::new(-out.y, out.x);
    let pull = CIRCLE_PULL * (distance - radius) / radius;
    let toward = tangent - out * pull;
    // Half of the tick's turn along the circle: the chord to the circle's next point, not the tangent.
    let chord = math::vector_angle(toward) + speed / (2.0 * radius);
    math::facing_vector(math::wrap_angle(chord))
}

#[cfg(test)]
mod tests;
