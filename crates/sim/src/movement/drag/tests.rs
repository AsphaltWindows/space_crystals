//! The DragMovement numbers alone: the thrust, the approach speed and the braking it plans. The scenario tests
//! in `crates/sim/tests/movement_drag.rs` cover a Drone in a game.

use super::*;

/// `factions/colonists/units.md > Drone > UnitBaseAttributes[HoverVehicle]`.
const DRONE: MovementAttrs = MovementAttrs::Drag {
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

/// `systems/units.md > DragMovement`: top speed is where thrust and drag balance, `(Omni + Forward) / DragRatio`;
/// a HoverVehicle's DragRatio is doubled on Rugged terrain, which halves it (design change 0039). Only
/// DragMovement units have the model, and a mode that slows the unit scales its thrust.
#[test]
fn top_speed_is_where_thrust_and_drag_balance() {
    let (open, rugged) = (drone(false), drone(true));
    assert!((open.top - 8.0).abs() < 1e-4 && (rugged.top - 4.0).abs() < 1e-4);
    assert!((open.keep - 0.9).abs() < 1e-6 && (rugged.keep - 0.8).abs() < 1e-6);
    let slowed = Drag::of(&DRONE, 0.5, false).unwrap();
    assert!((slowed.top - 4.0).abs() < 1e-4 && (slowed.omni - 0.1).abs() < 1e-6);
    let walker = MovementAttrs::TurnRate {
        turn_rate: 1.0,
        acceleration: None,
        deceleration: None,
        max_speed: 4.0,
    };
    assert_eq!(Drag::of(&walker, 1.0, false), None);
}

/// Step 3: omni-directional thrust any way, forward thrust along the facing only. Facing east, the unit gets
/// 0.8 eastwards, 0.2 westwards or sideways, and a wish within its reach exactly.
#[test]
fn the_thrust_is_omni_any_way_plus_forward_along_the_facing() {
    let model = drone(false);
    let near = |a: Vec2, b: Vec2| (a - b).length() < 1e-5;
    let (thrust, reached) = model.thrust(Vec2::new(5.0, 0.0), Vec2::X);
    assert!(near(thrust, Vec2::new(0.8, 0.0)) && !reached);
    let (thrust, reached) = model.thrust(Vec2::new(-5.0, 0.0), Vec2::X);
    assert!(
        near(thrust, Vec2::new(-0.2, 0.0)) && !reached,
        "no forward thrust backwards"
    );
    let (thrust, reached) = model.thrust(Vec2::new(0.0, 5.0), Vec2::X);
    assert!(near(thrust, Vec2::new(0.0, 0.2)) && !reached);
    // Forward and sideways at once: all the forward thrust, and the omni thrust on what is left.
    let (thrust, _) = model.thrust(Vec2::new(0.6, 3.0), Vec2::X);
    assert!(near(thrust, Vec2::new(0.6, 0.2)));
    for want in [Vec2::new(0.7, 0.1), Vec2::new(-0.1, 0.15), Vec2::ZERO] {
        assert_eq!(model.thrust(want, Vec2::X), (want, true));
    }
}

/// The scheme from rest, facing the way it goes: `v ← v + 0.8 − 0.1 v` gives 0.8, 1.52, 2.168, …, and
/// `8 × (1 − 0.9ⁿ)` after `n` ticks: within 1 % of 8 after 44 (plan decision T25).
#[test]
fn from_rest_the_speed_follows_the_scheme_to_its_equilibrium() {
    let model = drone(false);
    let mut v = Vec2::ZERO;
    let mut speeds = Vec::new();
    for _ in 0..60 {
        let (thrust, reached) = model.thrust(Vec2::new(8.0, 0.0) - v * model.keep, Vec2::X);
        v = if reached {
            Vec2::new(8.0, 0.0)
        } else {
            v * model.keep + thrust
        };
        speeds.push(v.x);
    }
    for (got, want) in speeds.iter().zip([0.8, 1.52, 2.168, 2.7512]) {
        assert!((got - want).abs() < 1e-4, "{speeds:?}");
    }
    let mut power = 1.0;
    for (n, &speed) in speeds.iter().enumerate() {
        power *= 0.9;
        assert!((speed - 8.0 * (1.0 - power)).abs() < 1e-3, "tick {}: {speed}", n + 1);
    }
    assert!(speeds[43] > 7.92 && speeds.iter().all(|&s| s <= 8.0));
    // On Rugged terrain: 4.
    let rugged = drone(true);
    let mut v = 0.0_f32;
    for _ in 0..60 {
        v = v * rugged.keep + 0.8;
    }
    assert!((v - 4.0).abs() < 0.04);
}

/// The documented bound: from top speed the Drone stops within 42 su and 16 ticks, braking with its
/// omni-directional thrust (`v ← 0.9 v − 0.2`); on Rugged terrain, from its 4 there, within 13 su and 8 ticks.
/// On its way to a goal it plans with [`PLANNED_BRAKE`] of that thrust, 43 su and 16 ticks, and begins to brake
/// a step before: 51 su from its goal.
#[test]
fn the_stopping_distance_from_top_speed_is_within_the_documented_bound() {
    let model = drone(false);
    let (dist, ticks) = braking(&model, model.omni, 8.0);
    assert!(
        (41.0..=42.0).contains(&dist) && ticks <= 16,
        "{dist} su in {ticks} ticks"
    );
    let (dist, ticks) = braking(&model, model.omni * PLANNED_BRAKE, 8.0);
    assert!(
        (42.0..=43.0).contains(&dist) && ticks <= 16,
        "{dist} su in {ticks} ticks"
    );
    // The plan itself: the first distance at which the approach speed is below top speed.
    let brakes_at = (0..2000)
        .map(|step| step as f32 * 0.05)
        .rfind(|&dist| model.approach(dist) < model.top)
        .expect("it brakes somewhere");
    assert!((49.0..=51.0).contains(&brakes_at), "a step of 8 and 43 su: {brakes_at}");
    let rugged = drone(true);
    let (dist, ticks) = braking(&rugged, rugged.omni, 4.0);
    assert!(dist <= 13.0 && ticks <= 8, "{dist} su in {ticks} ticks");
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
            assert!(
                speed >= before && speed <= model.top,
                "it only grows with the distance"
            );
            before = speed;
            if dist <= last {
                assert_eq!(speed, dist.min(model.top));
                continue;
            }
            let (stops_in, _) = braking(&model, brake, speed);
            if speed < model.top {
                assert!(
                    (speed + stops_in - dist).abs() < 2e-3,
                    "{dist}: {speed} + {stops_in}"
                );
            } else {
                assert!(speed + stops_in <= dist + 2e-3);
            }
            // The next tick's speed is within reach of braking.
            let next = model.approach(dist - speed);
            assert!(
                next >= speed * model.keep - brake - 1e-4,
                "{dist}: {speed} then {next}"
            );
        }
    }
}

/// A unit following the approach speed down from top speed lands exactly on its goal, at rest, without ever
/// passing it: every tick's wish is within its thrust, with the reserve of [`PLANNED_BRAKE`] for the rounding.
#[test]
fn braking_along_the_approach_speed_lands_exactly_on_the_goal() {
    let model = drone(false);
    for goal in [3.0_f32, 40.0, 41.7, 77.3, 128.0, 500.0] {
        let (mut pos, mut v, mut ticks) = (0.0_f32, model.top, 0);
        while pos != goal {
            let want = model.approach((goal - pos).abs()).copysign(goal - pos);
            let (thrust, reached) = model.thrust(Vec2::new(want - v * model.keep, 0.0), Vec2::X);
            v = if reached { want } else { v * model.keep + thrust.x };
            pos = if reached && want == goal - pos { goal } else { pos + v };
            ticks += 1;
            assert!(ticks < 200, "goal {goal}: at {pos} with {v}");
            // A goal nearer than the stopping distance from top speed is overrun: the unit was sent too late.
            if goal > 51.0 {
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
