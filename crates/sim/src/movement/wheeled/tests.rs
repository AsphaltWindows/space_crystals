//! The FixedTurnRadiusMovement numbers alone: the signed speed, braking, the approach speed, backing up, and the
//! forward loop's circles. The scenario tests in `crates/sim/tests/vehicles/` cover a Seeker in a game.

use super::*;
use crate::defs::ObjectKind;

/// The Seeker's movement numbers (`factions/cults/units/seeker.md`).
fn seeker_movement() -> MovementAttrs {
    ObjectKind::Seeker.def().unit.as_ref().expect("a unit").movement
}

fn seeker(reverses: bool) -> Wheeled {
    let mut mover = Mover::default();
    mover.reverses = reverses;
    Wheeled::of(&seeker_movement(), &mover).expect("a FixedTurnRadiusMovement unit")
}

/// The signed speeds of the ticks from `speed` on, each towards `want`, until a tick changes nothing.
fn speeds(model: &Wheeled, mut speed: f32, want: f32) -> Vec<f32> {
    let mut out = Vec::new();
    loop {
        let next = model.next_speed(speed, want);
        if next == speed || out.len() > 100 {
            return out;
        }
        out.push(next);
        speed = next;
    }
}

#[test]
fn these_are_the_seeker_s_numbers() {
    assert_eq!(
        seeker(false),
        Wheeled {
            radius: 64.0,
            reverses: false,
            top: 8.0,
            accel: 1.0,
            decel: 1.0
        }
    );
    // Backing up: ReverseMaxSpeed 3, ReverseAcceleration 0.5; Deceleration either way.
    assert_eq!((seeker(true).top, seeker(true).accel, seeker(true).decel), (3.0, 0.5, 1.0));
    // The other models have none.
    let walker = MovementAttrs::TurnRate {
        turn_rate: 1.0,
        acceleration: None,
        deceleration: None,
        max_speed: 4.0,
    };
    assert_eq!(Wheeled::of(&walker, &Mover::default()), None);
}

/// Acceptance (slice M5-V2, Speed): "From a standstill, full speed after 8 frames; a stop takes 8 frames". The speed
/// comes before the move by it (module docs): 1, 2 … 8 su in the first eight ticks, 36 su in all; braking, 7, 6 … 0,
/// at rest in the 8th tick, 28 su on.
#[test]
fn from_rest_it_is_at_full_speed_in_its_eighth_tick_and_stops_in_eight() {
    let model = seeker(false);
    let up = speeds(&model, 0.0, f32::MAX);
    assert_eq!(up, [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]);
    assert_eq!(up.iter().sum::<f32>(), 36.0);
    let down = speeds(&model, 8.0, 0.0);
    assert_eq!(down, [7.0, 6.0, 5.0, 4.0, 3.0, 2.0, 1.0, 0.0]);
    assert_eq!(down.iter().sum::<f32>(), 28.0);
}

/// The approach speed is the highest from which braking by Deceleration stops the unit on the goal: from full speed
/// it brakes 36 su out (its step at full speed, then 7 + 6 + … + 1 = 28 su), and its last step, at most 1 su, lands
/// on the goal.
#[test]
fn the_approach_speed_brakes_in_time_and_lands_on_the_goal() {
    let model = seeker(false);
    assert_eq!(model.approach(1000.0), 8.0);
    assert_eq!(model.approach(36.0), 8.0);
    assert!(model.approach(35.0) < 8.0);
    assert_eq!(model.approach(28.0), 7.0);
    assert_eq!(model.approach(1.0), 1.0);
    assert_eq!(model.approach(0.25), 0.25);
    // Braking along it from 36 su out: one tick less of speed each tick, landing exactly.
    let (mut left, mut speed, mut ticks) = (36.0_f32, 8.0_f32, 0);
    while left > 0.0 {
        speed = model.next_speed(speed, model.approach(left));
        assert!(speed <= left + 1e-4, "it never passes the goal");
        left -= speed.min(left);
        ticks += 1;
        assert!(ticks < 20);
    }
    assert_eq!(ticks, 8);
    // From any distance it falls by at most Deceleration a tick.
    let mut dist = 200.0_f32;
    while dist > 0.0 {
        let next = (dist - model.approach(dist)).max(0.0);
        assert!(model.approach(dist) - model.approach(next) <= 1.0 + 1e-4, "at {dist}");
        dist = next;
    }
}

/// Acceptance (Reverse): "W drives it backwards at up to 3 su/frame": the reverse speed goes down by
/// ReverseAcceleration a tick to -3. A unit still rolling forward when a Reverse begins brakes to rest first.
#[test]
fn backing_up_tops_at_reverse_max_speed_and_first_brakes_off_a_forward_speed() {
    let model = seeker(true);
    assert_eq!(speeds(&model, 0.0, f32::MAX), [-0.5, -1.0, -1.5, -2.0, -2.5, -3.0]);
    assert_eq!(
        speeds(&model, 2.5, f32::MAX),
        [1.5, 0.5, 0.0, -0.5, -1.0, -1.5, -2.0, -2.5, -3.0]
    );
    // And forward, a unit still backing up brakes first.
    assert_eq!(speeds(&seeker(false), -1.5, 2.0), [-0.5, 0.0, 1.0, 2.0]);
}

#[test]
fn the_sign_of_the_speed_is_the_side_of_the_facing_the_momentum_points_to() {
    let ahead = Vec2::new(0.0, 1.0);
    assert_eq!(speed_of(Vec2::new(0.0, 3.0), ahead), 3.0);
    assert_eq!(speed_of(Vec2::new(0.0, -3.0), ahead), -3.0);
    assert_eq!(speed_of(Vec2::ZERO, ahead), 0.0);
}

/// The forward loop (design change 0353 point 3): a waypoint inside the circle the unit turns on towards it can't be
/// reached by turning, so it steers straight on; one outside it, straight behind included, it turns towards.
#[test]
fn a_waypoint_inside_the_turning_circle_is_steered_past_straight_on() {
    let model = seeker(false);
    // Facing east (0): the right is south (+y on screen).
    let facing = 0.0;
    // 30 su to its right: within 64 su of the right circle's centre, (0, 64).
    assert_eq!(model.steer(facing, Vec2::new(0.0, 30.0)), (Vec2::new(30.0, 0.0), true));
    assert_eq!(model.steer(facing, Vec2::new(10.0, -40.0)).1, true, "on its left");
    // Outside the circles: steered at.
    let ahead = Vec2::new(100.0, 0.0);
    assert_eq!(model.steer(facing, ahead), (ahead, false));
    let behind = Vec2::new(-20.0, 0.0);
    assert_eq!(model.steer(facing, behind), (behind, false), "straight behind is outside both");
    let wide = Vec2::new(0.0, 130.0);
    assert_eq!(model.steer(facing, wide), (wide, false), "beyond the circle's far side");
    // Backing up the circles are the same, the way it drives the other.
    assert_eq!(seeker(true).steer(facing, Vec2::new(0.0, 30.0)), (Vec2::new(-30.0, 0.0), true));
}
