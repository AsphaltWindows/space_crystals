//! The DragMovement numbers alone: the thrust, the approach speed and the braking it plans. The scenario tests
//! in `crates/sim/tests/movement_drag/` cover a Drone in a game.

use super::*;
use crate::defs::ObjectKind;

/// `factions/colonists/units/drone.md > UnitBaseAttributes[HoverVehicle]`: since design change 0205, thrust and
/// drag ×1.5 of [`BEFORE_0205`], the same top speed.
const DRONE: MovementAttrs = MovementAttrs::Drag {
    forward_acceleration: 0.9,
    omni_directional_acceleration: 0.3,
    drag_ratio: 0.15,
    turn_rate: core::f32::consts::FRAC_PI_4,
};

/// The Drone's numbers before design change 0205 (slice M3-G to M3-K2), to measure what 0205 changed against.
const BEFORE_0205: MovementAttrs = MovementAttrs::Drag {
    forward_acceleration: 0.6,
    omni_directional_acceleration: 0.2,
    drag_ratio: 0.1,
    turn_rate: core::f32::consts::FRAC_PI_4,
};

fn drone(rugged: bool) -> Drag {
    Drag::of(&DRONE, 1.0, rugged).expect("a DragMovement unit")
}

/// The distance a unit covers braking from `speed` to rest with the thrust `brake`, and the ticks it takes:
/// `v ← v × keep − brake`.
fn braking(model: &Drag, brake: f32, mut speed: f32) -> (f32, u32) {
    let (mut dist, mut ticks) = (0.0, 0);
    loop {
        speed = speed * model.keep - brake;
        if speed <= 0.0 {
            return (dist, ticks);
        }
        dist += speed;
        ticks += 1;
    }
}

/// The unit's speed after each of `ticks` ticks from rest, facing east and wanting 8 su a tick east: steps 2 and 3
/// of the scheme.
fn from_rest(model: &Drag, ticks: usize) -> Vec<f32> {
    let mut v = Vec2::ZERO;
    let mut speeds = Vec::new();
    for _ in 0..ticks {
        let (thrust, reached) = model.thrust(Vec2::new(8.0, 0.0) - v * model.keep, Vec2::X);
        v = if reached {
            Vec2::new(8.0, 0.0)
        } else {
            v * model.keep + thrust
        };
        speeds.push(v.x);
    }
    speeds
}

/// The first distance from its goal at which a unit at top speed begins to brake: the approach speed falls below
/// the top speed there.
fn brakes_at(model: &Drag) -> f32 {
    (0..2000)
        .map(|step| step as f32 * 0.05)
        .rfind(|&dist| model.approach(dist) < model.top)
        .expect("it brakes somewhere")
}

/// The numbers here are the Drone's own, as its def gives them: a change of the design's numbers fails here first,
/// and the bounds below are measured again.
#[test]
fn these_are_the_drone_s_numbers() {
    let unit = ObjectKind::Drone.def().unit.as_ref().expect("a unit");
    assert_eq!(unit.movement, DRONE);
}

/// `systems/units.md > DragMovement`: top speed is where thrust and drag balance, `(Omni + Forward) / DragRatio`;
/// a HoverVehicle's DragRatio is doubled on Rugged terrain, which halves it (design change 0039, unchanged by 0205:
/// "Rugged terrain still doubles drag"). Only DragMovement units have the model, and a mode that slows the unit
/// scales its thrust.
#[test]
fn top_speed_is_where_thrust_and_drag_balance() {
    let (open, rugged) = (drone(false), drone(true));
    assert!((open.top - 8.0).abs() < 1e-4 && (rugged.top - 4.0).abs() < 1e-4);
    // A tick's drag leaves 1 − 0.15 of the velocity, and on Rugged terrain 1 − 2 × 0.15.
    assert!((open.keep - 0.85).abs() < 1e-6 && (rugged.keep - 0.7).abs() < 1e-6);
    assert_eq!(
        (rugged.forward, rugged.omni),
        (open.forward, open.omni),
        "the thrust is the same"
    );
    let slowed = Drag::of(&DRONE, 0.5, false).unwrap();
    assert!((slowed.top - 4.0).abs() < 1e-4 && (slowed.omni - 0.15).abs() < 1e-6);
    // Design change 0205 kept the top speed: on either ground it is the one the numbers before it gave.
    for rugged in [false, true] {
        let before = Drag::of(&BEFORE_0205, 1.0, rugged).unwrap();
        assert!((drone(rugged).top - before.top).abs() < 1e-4);
    }
    let walker = MovementAttrs::TurnRate {
        turn_rate: 1.0,
        acceleration: None,
        deceleration: None,
        max_speed: 4.0,
    };
    assert_eq!(Drag::of(&walker, 1.0, false), None);
}

/// Step 3: omni-directional thrust any way, forward thrust along the facing only. Facing east, the unit gets
/// 1.2 eastwards, 0.3 westwards or sideways, and a wish within its reach exactly.
#[test]
fn the_thrust_is_omni_any_way_plus_forward_along_the_facing() {
    let model = drone(false);
    let near = |a: Vec2, b: Vec2| (a - b).length() < 1e-5;
    let (thrust, reached) = model.thrust(Vec2::new(5.0, 0.0), Vec2::X);
    assert!(near(thrust, Vec2::new(1.2, 0.0)) && !reached);
    let (thrust, reached) = model.thrust(Vec2::new(-5.0, 0.0), Vec2::X);
    assert!(
        near(thrust, Vec2::new(-0.3, 0.0)) && !reached,
        "no forward thrust backwards"
    );
    let (thrust, reached) = model.thrust(Vec2::new(0.0, 5.0), Vec2::X);
    assert!(near(thrust, Vec2::new(0.0, 0.3)) && !reached);
    // Forward and sideways at once: all the forward thrust, and the omni thrust on what is left.
    let (thrust, _) = model.thrust(Vec2::new(0.6, 3.0), Vec2::X);
    assert!(near(thrust, Vec2::new(0.6, 0.3)));
    for want in [Vec2::new(1.0, 0.2), Vec2::new(-0.1, 0.15), Vec2::ZERO] {
        assert_eq!(model.thrust(want, Vec2::X), (want, true));
    }
}

/// The scheme from rest, facing the way it goes: `v ← v + 1.2 − 0.15 v` gives 1.2, 2.22, 3.087, …, and
/// `8 × (1 − 0.85ⁿ)` after `n` ticks: within 1 % of 8 after 29 (plan decision T25).
///
/// Design change 0205: "The Drone gets to about 65% of it after ~7 frames instead of 10, and close to full speed
/// after ~20 frames instead of ~30." Measured: 67.9 % of its top speed after 7 ticks and 96.1 % after 20 (80.3 %
/// after 10), where the numbers before 0205 gave 52.2 % and 87.8 % (65.1 % after 10, 95.8 % after 30, within 1 %
/// after 44).
#[test]
fn from_rest_the_speed_follows_the_scheme_to_its_equilibrium() {
    let model = drone(false);
    let speeds = from_rest(&model, 60);
    for (got, want) in speeds.iter().zip([1.2, 2.22, 3.087, 3.82395]) {
        assert!((got - want).abs() < 1e-4, "{speeds:?}");
    }
    let mut power = 1.0;
    for (n, &speed) in speeds.iter().enumerate() {
        power *= 0.85;
        assert!((speed - 8.0 * (1.0 - power)).abs() < 1e-3, "tick {}: {speed}", n + 1);
    }
    let within_1_percent = speeds.iter().position(|&s| s >= 7.92).expect("it gets there") + 1;
    assert_eq!(within_1_percent, 29);
    assert!(speeds.iter().all(|&s| s <= 8.0));
    // 0205: about 65 % after 7 ticks, close to full speed after 20.
    let share = |speeds: &[f32], tick: usize| speeds[tick - 1] / 8.0;
    assert!((0.65..=0.70).contains(&share(&speeds, 7)), "{}", share(&speeds, 7));
    assert!(share(&speeds, 20) >= 0.96, "{}", share(&speeds, 20));
    // Before it: 65 % only after 10 ticks and 96 % after 30.
    let before = from_rest(&Drag::of(&BEFORE_0205, 1.0, false).unwrap(), 60);
    assert!(share(&before, 7) < 0.53 && share(&before, 20) < 0.88, "{before:?}");
    assert!((0.65..=0.66).contains(&share(&before, 10)), "{before:?}");
    // On Rugged terrain: 4.
    let rugged = drone(true);
    let mut v = 0.0_f32;
    for _ in 0..60 {
        v = v * rugged.keep + 1.2;
    }
    assert!((v - 4.0).abs() < 0.04);
}

/// The documented bound: from top speed the Drone stops within 26 su and 9 ticks, braking with its
/// omni-directional thrust (`v ← 0.85 v − 0.3`); on Rugged terrain, from its 4 there, within 5 su and 4 ticks.
/// On its way to a goal it plans with [`PLANNED_BRAKE`] of that thrust, 27 su and 10 ticks, and begins to brake
/// a step before: 35 su from its goal.
///
/// Design change 0205's Verification, "It stops with less drift than before": measured, 25.5 su in 9 ticks
/// stopped, where the numbers before it gave 41.5 su in 15 ticks (the bound was 42 su and 16 ticks, 51 su before
/// the goal, and 13 su and 8 ticks on Rugged terrain).
#[test]
fn the_stopping_distance_from_top_speed_is_within_the_documented_bound() {
    let model = drone(false);
    let (dist, ticks) = braking(&model, model.omni, 8.0);
    assert!(
        (25.0..=26.0).contains(&dist) && ticks <= 9,
        "{dist} su in {ticks} ticks"
    );
    let before = Drag::of(&BEFORE_0205, 1.0, false).unwrap();
    let (drifted, took) = braking(&before, before.omni, 8.0);
    assert!(
        (41.0..=42.0).contains(&drifted) && took == 15,
        "before 0205: {drifted} su in {took} ticks"
    );
    assert!(dist < drifted * 0.65 && ticks < took, "less drift than before");
    let (dist, ticks) = braking(&model, model.omni * PLANNED_BRAKE, 8.0);
    assert!(
        (26.0..=27.0).contains(&dist) && ticks <= 10,
        "{dist} su in {ticks} ticks"
    );
    // The plan itself: the first distance at which the approach speed is below top speed.
    let at = brakes_at(&model);
    assert!((33.0..=35.0).contains(&at), "a step of 8 and 27 su: {at}");
    assert!(at < brakes_at(&before), "it brakes later than before 0205");
    let rugged = drone(true);
    let (dist, ticks) = braking(&rugged, rugged.omni, 4.0);
    assert!(dist <= 5.0 && ticks <= 4, "{dist} su in {ticks} ticks");
}

/// The approach speed is the highest speed the unit still stops from: a step at it and then braking covers the
/// distance exactly (short of top speed), it never falls faster than braking follows, and within one braking
/// tick of the goal it is the distance itself, so the unit lands on the goal.
#[test]
fn the_approach_speed_is_the_highest_the_unit_still_stops_from() {
    for model in [drone(false), drone(true), Drag::of(&DRONE, 0.5, false).unwrap()] {
        let brake = model.omni * PLANNED_BRAKE;
        let last = brake / model.keep;
        let mut before = 0.0;
        for step in 0..4000 {
            let dist = step as f32 * 0.025;
            let speed = model.approach(dist);
            assert!(speed >= before && speed <= model.top, "it only grows with the distance");
            before = speed;
            if dist <= last {
                assert_eq!(speed, dist.min(model.top));
                continue;
            }
            let (stops_in, _) = braking(&model, brake, speed);
            if speed < model.top {
                assert!((speed + stops_in - dist).abs() < 2e-3, "{dist}: {speed} + {stops_in}");
            } else {
                assert!(speed + stops_in <= dist + 2e-3);
            }
            // The next tick's speed is within reach of braking.
            let next = model.approach(dist - speed);
            assert!(next >= speed * model.keep - brake - 1e-4, "{dist}: {speed} then {next}");
        }
    }
}

/// A unit following the approach speed down from top speed lands exactly on its goal, at rest, without ever
/// passing it: every tick's wish is within its thrust, with the reserve of [`PLANNED_BRAKE`] for the rounding.
#[test]
fn braking_along_the_approach_speed_lands_exactly_on_the_goal() {
    let model = drone(false);
    // A goal nearer than the stopping distance from top speed is overrun: the unit was sent too late.
    let too_late = brakes_at(&model);
    for goal in [3.0_f32, 30.0, 34.7, 41.7, 77.3, 128.0, 500.0] {
        let (mut pos, mut v, mut ticks) = (0.0_f32, model.top, 0);
        while pos != goal {
            let want = model.approach((goal - pos).abs()).copysign(goal - pos);
            let (thrust, reached) = model.thrust(Vec2::new(want - v * model.keep, 0.0), Vec2::X);
            v = if reached { want } else { v * model.keep + thrust.x };
            pos = if reached && want == goal - pos { goal } else { pos + v };
            ticks += 1;
            assert!(ticks < 200, "goal {goal}: at {pos} with {v}");
            if goal > too_late {
                assert!(pos <= goal, "goal {goal}: overshot to {pos}");
                assert!(
                    reached || want >= model.top - 1e-3,
                    "goal {goal} at {pos}: {v} to {want}"
                );
            }
        }
        assert!(
            v.abs() <= model.omni / model.keep + 1e-5,
            "one tick of braking stops it: {v}"
        );
    }
}

/// Degenerate numbers still give finite speeds: nothing to brake with (drag alone stops the unit), and a drag
/// that takes everything.
#[test]
fn degenerate_numbers_give_finite_speeds() {
    assert_eq!(approach_speed(10.0, 0.9, 0.0, 8.0), 10.0 * (1.0 - 0.9));
    assert_eq!(approach_speed(10.0, 0.0, 0.2, 0.8), 0.8);
    assert_eq!(approach_speed(0.0, 0.9, 0.2, 8.0), 0.0);
}
