//! Turning in place (`systems/units.md > SpeedTurnRadiusMovement`: "Unit can rotate in place by spinning treads in
//! opposite directions"; design change 0417: "SpinRate = 2 × MaxSpeed ÷ TrackWidth radians per frame"), and a spin that
//! a wall blocks (`systems/units.md > UnitCollision > Collision Response (Un-sticking)`, slice M5-V1).

use sc_sim::math::PI;
use sc_sim::testing::ScenarioBuilder;
use sc_sim::{MoveStatus, SpawnAt, Vec2};

use super::{HAND, P1, SPIN, centre, done, facing, flat, move_to, step};

/// Design change 0417's Verification: "A Hand of Heaven standing still turns 90° in about 5 frames (0.3 s)". Facing
/// east and sent to a point due south, it spins in place at SpinRate, 1/3 radian (19.1°) a tick, and faces south in
/// its 5th tick of turning (90° is 4.7 ticks of it) without having moved; only then does it set off.
#[test]
fn standing_still_it_turns_a_right_angle_in_its_fifth_tick() {
    let mut t = flat(12, 12).spawn(HAND, Some(P1), SpawnAt::Point(centre(5, 2))).start();
    let start = t.pos(1);
    move_to(&mut t, 1, &[1], centre(5, 9));
    let mut turns = Vec::new();
    while facing(&t, 1) != PI / 2.0 {
        let (moved, turned) = step(&mut t, 1);
        assert_eq!(moved, Vec2::ZERO, "it spins in place");
        if turned != 0.0 {
            turns.push(turned);
        }
        assert!(t.sim().tick() < 20, "{turns:?}");
    }
    assert_eq!(turns.len(), 5, "{turns:?}");
    assert!(
        turns[..4].iter().all(|&turned| (turned - SPIN).abs() < 1e-6),
        "{turns:?}"
    );
    assert_eq!(t.pos(1), start);
    let (moved, turned) = step(&mut t, 1);
    assert_eq!(
        (moved, turned),
        (Vec2::new(0.0, 0.5), 0.0),
        "then it drives off, 0.5 su in its first tick"
    );
}

/// The brief's acceptance: "standing still facing east, ordered to a point due west, it spins in place without moving,
/// then drives off". A half turn at SpinRate takes 10 ticks (9.4 of them); it moves only once it faces west, and goes
/// straight there.
#[test]
fn facing_east_and_sent_due_west_it_spins_round_where_it_stands_then_drives_off() {
    let mut t = flat(16, 6).spawn(HAND, Some(P1), SpawnAt::Point(centre(12, 2))).start();
    let start = t.pos(1);
    let goal = centre(3, 2);
    move_to(&mut t, 1, &[1], goal);
    let mut spun = 0;
    while facing(&t, 1).abs() != PI {
        let (moved, turned) = step(&mut t, 1);
        assert_eq!(moved, Vec2::ZERO, "tick {}: it spins in place", t.sim().tick());
        spun += usize::from(turned != 0.0);
        assert!(t.sim().tick() < 30);
    }
    assert_eq!(spun, 10);
    while !done(&t, 1) {
        let (moved, turned) = step(&mut t, 1);
        assert_eq!((turned, moved.y), (0.0, 0.0), "it drives straight west");
        assert!(moved.x <= 0.0);
        assert!(t.sim().tick() < 400);
    }
    assert_eq!((t.status(1), t.pos(1)), (MoveStatus::Arrived, goal));
    assert!(start.x > goal.x);
}

/// "At a wall, a blocked spin stops at contact" (slice M5-V1; `systems/units.md > UnitCollision > Collision Response
/// (Un-sticking)`: "Overlap caused by the rotation itself (a corner sweeping into a wall) follows the same principle:
/// the blocked part is discarded and the unit never ends up overlapping"). A Hand of Heaven facing east along a wall
/// that its side almost touches, sent to a point behind it on the far side, spins to turn round: its corner sweeps into
/// the wall, and the spin stops where it meets it, short of SpinRate, its Silhouette never in the wall (invariant I2).
/// It doesn't freeze there (design change 0035: "Blocked units don't freeze; they creep and turn out of the contact"):
/// in the same tick it takes the rules' step away from the wall as it is, spins on once it is free, and arrives.
#[test]
fn a_spin_a_wall_blocks_stops_at_contact() {
    let map = ["............", "............", "............", "MMMMMMMMMMMM"].join("\n");
    // Its side 2 su from the wall's face (y = 192): its centre 20 + 2 su above it.
    let at = Vec2::new(384.0, 170.0);
    let mut t = ScenarioBuilder::new(&map)
        .spawn(HAND, Some(P1), SpawnAt::Point(at))
        .start();
    let goal = Vec2::new(100.0, 40.0);
    move_to(&mut t, 1, &[1], goal);
    let (moved, turned) = step(&mut t, 1);
    assert!(turned < 0.0 && turned > -SPIN, "the spin stopped at contact: {turned}");
    assert!(moved.y < 0.0, "out of the contact, away from the wall: {moved}");
    while !done(&t, 1) {
        t.step();
        assert!(t.sim().tick() < 400, "{}", t.ascii(Some(t.pos(1))));
    }
    assert_eq!((t.status(1), t.pos(1)), (MoveStatus::Arrived, goal));
}
