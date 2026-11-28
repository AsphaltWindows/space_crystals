//! (M6-GL) A Glider's landing approach and its hold in the air — `factions/gdo/structures/airfield.md > Returning and
//! rearming` (design change 0428 point 5, answering Q0162): "**The Dragonfly** (a Glider) lands and takes off
//! **vertically**, as the Specter at an air door: it approaches its slot at IdleSpeed, stops over it while it lands (16
//! frames), and takes off straight up, leaving at IdleSpeed. Landed, it doesn't move"; plan decision T88: "a glider
//! takes GL's straight approach at IdleSpeed". GliderMovement keeps moving (`super::glider`: "Unit must always maintain
//! movement to stay airborne"); its landing overrides that here, in the movement step, in place of its flight:
//!
//! - **The approach** ([`fly`]; `Mover::approach_to`, `Approach::Flying`): it flies at its goal's point as a Glider
//!   flies at any goal, at MaxSpeed while the point is ahead of it (within 90° of its heading), farther than
//!   `Air::approach_gu` and outside the circle of its widest turn (a point inside it a Glider slows down for,
//!   `super::glider`); else it slows to IdleSpeed, so that it turns its tightest for a point beside or behind it
//!   and flies the last of the way at IdleSpeed. It turns toward the point by at most its turn at its speed
//!   (MaxCentripetalAcceleration / v), but for a point inside the circle it flies turning its most at that speed
//!   ([`circles`]), which it would circle for ever, that the tick's turn can't bring ahead of it: it flies straight on
//!   until it can turn onto it, round again if it overflew it. **Placeholder (not in the design: how a Glider lines up
//!   with its slot)**. **Its arrival**: at IdleSpeed, with the point on the line of the tick's flight and within its
//!   reach ([`ON_LINE_SU`]), it flies onto the point and stops there, its leg over (`MoveStatus::Arrived`), Stationary,
//!   and is over it (`Approach::Over`); farther, faster or not lined up, it flies on. Its momentum stays as it flew
//!   until it holds still, so that one whose approach ends there and then (`Mover::stop`) flies on at IdleSpeed, as a
//!   Glider stopped after its flight keeps its speed. It lands from whatever heading it came in on: it faces that way.
//! - **The hold** ([`holds`], [`hold`]): a Glider holds still in the air, no move, no momentum, Stationary, keeping its
//!   goal, while it is over its approach's point and the order that sent it there runs (one whose order ended without
//!   another, `Sim::remove_objects`'s, flies on: [`stale`]), while it is on an Airfield's slot, landing or down
//!   (`crate::landing`), and while it rises off one (`Mover::rise`, slice M6-SD's). Its rise's last tick lets it move:
//!   it flies on from IdleSpeed, as a Glider that hasn't flown yet does (`super::glider`), "leaving at IdleSpeed".
//!
//! Glider's own rules (`super::glider`) are read-only to this slice: the speed step ([`next_speed`]) is `Glider`'s,
//! written again here, and the test of a point inside the turn ([`circles`]) is `Glider`'s, for the path its flight
//! takes tick by tick.

use glam::Vec2;

use super::glider::Glider;
use crate::math;
use crate::mover::{Approach, Locomotion, MoveStatus, Mover, Orientation};
use crate::ruleset::rules;
use crate::units::gu;

/// How near the line of a tick's flight the point of a Glider's approach must lie for it to fly onto it ([`fly`]),
/// space units: a technical tolerance, for the rounding of its position and its heading, so that a Glider headed at the
/// point a hair short of it doesn't overfly it.
const ON_LINE_SU: f32 = 0.1;

/// Whether a Glider whose mover is `mover` holds still this tick (module docs): over its approach's point, on an
/// Airfield's slot (`on_slot`), or rising off it (`rising`, `Mover::rises`).
pub(super) fn holds(mover: &Mover, on_slot: bool, rising: bool) -> bool {
    rising || on_slot || mover.is_over()
}

/// Whether a Glider over its approach's point (`Approach::Over`) is so no more: the order that sent it there ended
/// without another to stop it (`ordered`: it has a current order). It circles where it is from here on, as after a stop
/// (`Mover::stop`).
pub(super) fn stale(mover: &Mover, ordered: bool) -> bool {
    mover.is_over() && !ordered
}

/// One tick of a Glider's hold (module docs): it stands, its speed spent, keeping its goal.
pub(super) fn hold(mover: &mut Mover, turned: &mut f32) {
    super::stand(mover);
    mover.momentum = Vec2::ZERO;
    *turned = 0.0;
}

/// Phase 5d, one tick of `glider`'s landing approach to `spot`, its goal's point (module docs): it sets its speed,
/// turns, and flies on, or onto the point, where it stops. `turned` gets the angle it turned.
pub(super) fn fly(glider: &Glider, pos: &mut Vec2, facing: &mut f32, turned: &mut f32, mover: &mut Mover, spot: Vec2) {
    let speed = if mover.momentum == Vec2::ZERO {
        glider.idle_speed
    } else {
        mover.momentum.length()
    };
    let to = spot - *pos;
    let slows = gu(rules().tuning.air.approach_gu);
    // Ahead of it, and outside its widest turn's circle, whatever its speed now: the speed it wants doesn't swing with
    // the circle it decides by.
    let ahead = to.dot(math::facing_vector(*facing)) > 0.0;
    let wide = circles(glider, *pos, *facing, glider.max_speed, spot);
    let wanted = if ahead && !wide && to.length_squared() > slows * slows {
        glider.max_speed
    } else {
        glider.idle_speed
    };
    let speed = next_speed(glider, speed, wanted);
    *turned = 0.0;
    if to != Vec2::ZERO {
        let bearing = math::vector_angle(to);
        let budget = glider.turn_rate(speed);
        let reaches = math::wrap_angle(bearing - *facing).abs() <= budget;
        if reaches || !circles(glider, *pos, *facing, speed, spot) {
            *turned = math::turn_toward(facing, bearing, budget).angle;
        }
    }
    // Its arrival: the point on the line of this tick's flight, within its reach.
    let heading = math::facing_vector(*facing);
    let (along, across) = (to.dot(heading), heading.perp_dot(to).abs());
    if speed <= glider.idle_speed && across <= ON_LINE_SU && (-ON_LINE_SU..=speed + ON_LINE_SU).contains(&along) {
        *pos = spot;
        mover.finish(MoveStatus::Arrived);
        mover.momentum = heading * speed;
        mover.approach = Some(Approach::Over);
        return;
    }
    mover.orientation = if *turned > 0.0 {
        Orientation::Turning(spot)
    } else {
        Orientation::Maintaining
    };
    let flight = heading * speed;
    *pos += flight;
    mover.momentum = flight;
    mover.velocity = flight;
    mover.locomotion = Locomotion::Moving;
}

/// The speed `glider` flies at after `speed`, wanting `wanted`: by at most its Acceleration or Deceleration, within its
/// IdleSpeed and MaxSpeed (`Glider`'s rule, module docs).
fn next_speed(glider: &Glider, speed: f32, wanted: f32) -> f32 {
    let change = wanted - speed;
    let change = if change > 0.0 {
        glider.acceleration.map_or(change, |up| change.min(up))
    } else {
        glider.deceleration.map_or(change, |down| change.max(-down))
    };
    (speed + change).clamp(glider.idle_speed, glider.max_speed)
}

/// Whether the point `at` lies inside the circle that `glider`, at `pos` facing `facing`, flies at `speed` turning its
/// most toward that side every tick (module docs): it would circle `at` for ever at that speed, never headed at it. A
/// tick of its flight is a turn and then a step `speed` long along its new heading ([`fly`]), so that such a flight is
/// a polygon whose corners lie on a circle of radius speed / (2 sin(turn / 2)), with `pos` one of them: that circle,
/// not the turn's own of radius v² / a tangent at `pos` (`Glider`'s), which a point on its far side would lie just
/// inside of after each step, so that the Glider would circle it for ever, a straight step at a time.
fn circles(glider: &Glider, pos: Vec2, facing: f32, speed: f32, at: Vec2) -> bool {
    let side = math::facing_vector(facing).perp_dot(at - pos);
    if side == 0.0 {
        return false;
    }
    let half = glider.turn_rate(speed) / 2.0;
    let radius = speed / (2.0 * math::sin(half));
    // The centre: square to its first step, from the step's middle, on the side of `at`.
    let step = math::facing_vector(math::wrap_angle(facing + (2.0 * half).copysign(side)));
    let normal = Vec2::new(-step.y, step.x) * side.signum();
    let centre = pos + step * (speed / 2.0) + normal * (radius * math::cos(half));
    (at - centre).length_squared() < radius * radius
}

#[cfg(test)]
mod tests {
    use glam::Vec2;

    use super::*;
    use crate::defs::{MovementAttrs, ObjectKind};
    use crate::mover::MoveGoal;

    /// The Dragonfly's numbers (`defs/gdo/units/dragonfly.rs`).
    fn dragonfly() -> Glider {
        let movement = ObjectKind::Dragonfly.def().unit.as_ref().unwrap().movement;
        assert!(matches!(movement, MovementAttrs::Glider { .. }));
        Glider::of(&movement).unwrap()
    }

    /// A Dragonfly at `pos` facing `facing`, flying at `speed`, on its approach to `spot`: the ticks it takes to stop
    /// over the spot, its speed in each tick of its flight, and its facing as it arrives; `None` if it doesn't within
    /// `ticks`.
    fn fly_in(pos: Vec2, facing: f32, speed: f32, spot: Vec2, ticks: u32) -> Option<(u32, Vec<f32>, f32)> {
        let glider = dragonfly();
        let (mut pos, mut facing, mut turned) = (pos, facing, 0.0);
        let mut mover = Mover {
            momentum: math::facing_vector(facing) * speed,
            ..Mover::default()
        };
        mover.approach_to(spot, 0);
        let mut speeds = Vec::new();
        for tick in 1..=ticks {
            fly(&glider, &mut pos, &mut facing, &mut turned, &mut mover, spot);
            if mover.is_over() {
                assert_eq!(pos, spot, "over its spot");
                assert_eq!((mover.goal, mover.status), (MoveGoal::None, MoveStatus::Arrived));
                assert_eq!((mover.velocity, mover.locomotion), (Vec2::ZERO, Locomotion::Stationary));
                return Some((tick, speeds, facing));
            }
            speeds.push(mover.momentum.length());
            assert!(
                turned <= glider.turn_rate(mover.momentum.length()) + 1e-6,
                "within its turn"
            );
        }
        None
    }

    /// Straight in at MaxSpeed from 20 grid units: it flies at MaxSpeed, slows to IdleSpeed by `Air::approach_gu` and
    /// the 0.6 su/frame² of its Deceleration, and flies the rest at IdleSpeed, stopping on its spot within a tick's
    /// flight of IdleSpeed, never slower than IdleSpeed nor faster than MaxSpeed.
    #[test]
    fn straight_in_it_slows_to_idle_speed_and_stops_over_its_spot() {
        let spot = Vec2::new(2000.0, 1000.0);
        let (ticks, speeds, facing) = fly_in(spot - Vec2::new(1280.0, 0.0), 0.0, 14.0, spot, 400).unwrap();
        assert!(
            speeds.iter().all(|&v| (6.0 - 1e-4..=14.0 + 1e-4).contains(&v)),
            "{speeds:?}"
        );
        let idle = speeds.iter().rev().take_while(|&&v| v <= 6.0 + 1e-4).count();
        assert!(
            (15..=25).contains(&idle),
            "its last {idle} ticks at IdleSpeed: {speeds:?}"
        );
        assert_eq!(speeds.first(), Some(&14.0));
        assert!(facing.abs() < 1e-4, "it came in heading east: {facing}");
        assert!(ticks < 120, "{ticks}");
    }

    /// From every heading, in steps of 15°, and from either side and behind, near and far, at IdleSpeed and MaxSpeed:
    /// it stops over its spot, at IdleSpeed, within a few circles of its approach.
    #[test]
    fn it_lands_from_any_heading() {
        let spot = Vec2::new(3000.0, 3000.0);
        for (from, speed) in [
            (Vec2::new(-600.0, 0.0), 14.0),
            (Vec2::new(0.0, 90.0), 6.0),
            (Vec2::new(20.0, 10.0), 14.0),
        ] {
            for step in 0..24 {
                let facing = math::wrap_angle(step as f32 * math::PI / 12.0);
                let arrival = fly_in(spot + from, facing, speed, spot, 600);
                let (ticks, speeds, _) = arrival.unwrap_or_else(|| panic!("from {from}, facing {facing}: never"));
                assert!(
                    speeds.last().is_none_or(|&v| v <= 6.0 + 1e-4),
                    "at IdleSpeed: {speeds:?}"
                );
                assert!(ticks <= 300, "from {from}, facing {facing}: {ticks} ticks");
            }
        }
    }
}
