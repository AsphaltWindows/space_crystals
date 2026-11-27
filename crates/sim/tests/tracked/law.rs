//! The tread law at speed (design change 0417, `systems/units.md > SpeedTurnRadiusMovement > The turning law`):
//! "maxTurnRate = SpinRate × (1 − |v| ÷ MaxSpeed)", on a tightest radius of "(TrackWidth ÷ 2) × |v| ÷ (MaxSpeed −
//! |v|)"; the Reverse order, "the same law with the speed's size |v|; reverse is capped at ReverseMaxSpeed".

use sc_sim::{MoveStatus, SpawnAt, Vec2};

use super::{BACK, HAND, P1, TOP, about, centre, done, facing, flat, move_to, rate, reverse_to, step, turns};

/// Design change 0417's Verification: "At full speed, ordered to a point 90° to its side, it first slows, then turns;
/// it never turns while at MaxSpeed. At half speed its tightest circle has a radius of about 15 su; at 4 su/frame
/// about 60 su." Driving east at 5 su a tick, it is sent to a point 640 su due south of where it is. It brakes, and
/// turns from its first tick below MaxSpeed, by the law at each tick's speed: at full lock its moves lie on the law's
/// circle for their speed, `15 × v ÷ (5 − v)` su, 60 su at 4 su a tick (and 15 at half speed, 2.5 su a tick, which a
/// turn from full speed doesn't slow to: `movement/tracked/tests.rs`). It never turns at MaxSpeed, and gets there.
#[test]
fn at_full_speed_sent_to_its_side_it_slows_then_turns_on_the_law_s_circles() {
    let mut t = flat(40, 20).spawn(HAND, Some(P1), SpawnAt::Point(centre(1, 4))).start();
    move_to(&mut t, 1, &[1], centre(39, 4));
    while t.mover(1).momentum.length() < TOP {
        t.step();
        assert!(t.sim().tick() < 40);
    }
    t.run(4);
    let side = t.pos(1) + Vec2::new(0.0, 640.0);
    let tick = t.sim().tick() + 1;
    move_to(&mut t, tick, &[1], side);
    let mut ticks = Vec::new();
    while !done(&t, 1) {
        let (moved, turned) = step(&mut t, 1);
        ticks.push((moved.length(), turned));
        assert!(t.sim().tick() < 600);
    }
    assert_eq!((t.status(1), t.pos(1)), (MoveStatus::Arrived, side));
    let first_turn = ticks.iter().position(|&(_, turned)| turned != 0.0).expect("it turned");
    assert!(
        ticks[first_turn].0 < TOP - 1e-3,
        "it turned only once below MaxSpeed: {ticks:?}"
    );
    assert!(
        ticks[..first_turn].iter().all(|&(speed, _)| about(speed, TOP)),
        "{ticks:?}"
    );
    for &(speed, turned) in &ticks {
        assert!(
            turned.abs() <= rate(speed) * 1.0001 + 1e-5,
            "{speed} su, {turned} rad: {ticks:?}"
        );
        assert!(
            !about(speed, TOP) || !turns(turned),
            "it never turns at MaxSpeed: {ticks:?}"
        );
    }
    // At full lock (the law's rate) the move is the chord of the law's circle for the speed: `|v|` long, turning the
    // heading by the rate, so the circle's radius is `|v| ÷ (2 sin(rate ÷ 2))`, nearly `|v| ÷ rate`.
    let circles: Vec<(f32, f32)> = ticks
        .iter()
        .filter(|&&(speed, turned)| speed > 0.5 && turns(turned) && (turned.abs() - rate(speed)).abs() < 1e-4)
        .map(|&(speed, turned)| (speed, speed / (2.0 * sc_sim::math::sin(turned.abs() / 2.0))))
        .collect();
    for &(speed, radius) in &circles {
        let law = 15.0 * speed / (TOP - speed);
        assert!(
            (radius - law).abs() < 0.01 * law + 0.1,
            "at {speed} su a tick on {radius} su, not {law}"
        );
    }
    let at_four = circles
        .iter()
        .find(|&&(speed, _)| about(speed, 4.0))
        .expect("a tick at full lock at 4 su a tick");
    assert!((at_four.1 - 60.0).abs() < 0.1, "{at_four:?}");
    let slowest = ticks[first_turn..]
        .iter()
        .map(|&(speed, _)| speed)
        .fold(f32::INFINITY, f32::min);
    println!(
        "sent 90° to its side at full speed: slowest {slowest} su a tick, {} ticks",
        ticks.len()
    );
}

/// Design change 0417's Verification: "Reverse order: it backs up at no more than 2.5 su/frame and turns by the same
/// law" (W, `systems/controls/commands.md > Reverse`). Facing east, reversed to a point behind it and to its side, it
/// backs up, its back leading, speeding up by its Acceleration to ReverseMaxSpeed and no faster, turning by at most the
/// law's rate at each tick's speed, and gets there facing roughly away from it.
#[test]
fn reversed_it_backs_up_at_two_and_a_half_at_most_and_turns_by_the_same_law() {
    let mut t = flat(30, 14)
        .spawn(HAND, Some(P1), SpawnAt::Point(centre(20, 7)))
        .start();
    let goal = centre(6, 3);
    reverse_to(&mut t, 1, &[1], goal);
    let mut ticks = Vec::new();
    while !done(&t, 1) {
        let ahead = sc_sim::math::facing_vector(facing(&t, 1));
        let (moved, turned) = step(&mut t, 1);
        assert!(moved.dot(ahead) <= 1e-4, "it backs up: {moved} facing {ahead}");
        ticks.push((moved.length(), turned));
        assert!(t.sim().tick() < 800);
    }
    assert_eq!((t.status(1), t.pos(1)), (MoveStatus::Arrived, goal));
    let fastest = ticks.iter().map(|&(speed, _)| speed).fold(0.0, f32::max);
    assert!(about(fastest, BACK), "{fastest}: {ticks:?}");
    for &(speed, turned) in &ticks {
        assert!(turned.abs() <= rate(speed) * 1.0001 + 1e-5, "{speed} su, {turned} rad");
    }
    assert!(
        ticks.iter().any(|&(_, turned)| turned != 0.0),
        "it turned to back towards the point"
    );
    let away = sc_sim::math::facing_vector(facing(&t, 1)).dot((goal - centre(20, 7)).normalize());
    assert!(away < -0.5, "its back leads: {away}");
}
