//! Acceptance, **Turning** and **Behind it** (slice M5-V2): "a 90° turn at full speed follows a 64 su arc, within 2
//! su"; "ordered to a point behind it, it loops forward and never turns in place" (`systems/units.md >
//! FixedTurnRadiusMovement`: "Unit cannot turn in place. Turns at a fixed minimum radius regardless of speed"), and
//! design change 0353's Verification: "a Seeker ordered just behind itself loops forward" (`systems/units.md >
//! UnitCollision > Ground Collision > Who yields > Vehicles`: "a point inside its turning circle (e.g. just behind
//! it) is reached by a **forward loop**").

use super::*;

/// Runs unit 1 until its move ends, at most `ticks`, checking each tick that it drives forward and turns by no more
/// than its speed allows (|v| / r): never in place, never backwards. Returns its positions, the first its own.
fn drive_forward(t: &mut TestSim, ticks: u32) -> Vec<Vec2> {
    let mut path = vec![t.pos(1)];
    for _ in 0..ticks {
        let was = ahead(t, 1);
        let (d, turn) = step(t, 1);
        let tick = t.sim().tick();
        assert!(
            turn.abs() <= d.length() / RADIUS + 1e-5,
            "tick {tick}: turned {turn} on a move of {d}"
        );
        assert!(d.dot(was) >= 0.0, "tick {tick}: backwards, {d}");
        path.push(t.pos(1));
        if done(t, 1) {
            return path;
        }
    }
    panic!("not there after {ticks} ticks: {:?}\n{}", t.mover(1), t.ascii(None));
}

/// The Seeker runs east to full speed, and is then sent far to the north of the point 64 su east and 64 su north of
/// where it is: it turns left on its circle round that point, 90°, and drives north. Every point of the turn is
/// within 2 su of the circle, at full speed but for the tick the new path takes.
#[test]
fn a_90_degree_turn_at_full_speed_follows_its_64_su_circle() {
    let mut t = flat(40, 40)
        .spawn(SEEKER, Some(P1), SpawnAt::Point(centre(2, 36)))
        .start();
    move_to(&mut t, 1, &[1], centre(38, 36));
    t.run_until(16);
    assert_eq!(step(&mut t, 1).0, Vec2::new(TOP, 0.0), "at full speed");
    let at = t.pos(1);
    let centre_of_turn = at + Vec2::new(0.0, -RADIUS);
    let tick = t.sim().tick() + 1;
    move_to(&mut t, tick, &[1], at + Vec2::new(RADIUS, -2000.0));
    let mut turned = 0.0;
    let mut speeds = Vec::new();
    while turned > -core::f32::consts::FRAC_PI_2 + 1e-4 {
        let (d, turn) = step(&mut t, 1);
        turned += turn;
        speeds.push(d.length());
        let off = (t.pos(1).distance(centre_of_turn) - RADIUS).abs();
        assert!(off <= 2.0, "tick {}: {off} su off the circle", t.sim().tick());
        assert!(turn <= 0.0, "left only");
        assert!(speeds.len() < 40, "{turned}");
    }
    assert!(speeds.iter().filter(|&&s| s < TOP - 0.01).count() <= 2, "{speeds:?}");
    let ahead = ahead(&t, 1);
    assert!(ahead.y < -0.99, "heading north: {ahead}");
}

/// Ordered to a point straight behind it, 100 su, the Seeker drives a forward loop round to it: it never turns in
/// place nor backs up ([`drive_forward`]), turns all the way round, and arrives.
#[test]
fn ordered_to_a_point_behind_it_a_seeker_loops_forward() {
    let start = centre(10, 10);
    let mut t = flat(20, 20).spawn(SEEKER, Some(P1), SpawnAt::Point(start)).start();
    let goal = start - Vec2::new(100.0, 0.0);
    move_to(&mut t, 1, &[1], goal);
    let path = drive_forward(&mut t, 400);
    assert_eq!(t.status(1), MoveStatus::Arrived);
    assert!(t.pos(1).distance(goal) < 1.0, "{}", t.pos(1));
    let farthest = path.iter().map(|p| p.distance(start)).fold(0.0, f32::max);
    assert!(farthest > RADIUS, "a loop, {farthest} su out");
}

/// Design change 0353's Verification: "a Seeker ordered just behind itself loops forward". A point 60 su behind its
/// centre, a little to its right, and one inside its turning circle on its right (it can't turn onto it: it drives on
/// until it can, `movement/wheeled.rs`): it reaches both by a forward loop.
#[test]
fn a_seeker_ordered_just_behind_itself_or_inside_its_circle_loops_forward() {
    for goal_from_start in [Vec2::new(-60.0, 10.0), Vec2::new(10.0, 40.0)] {
        let start = centre(10, 10);
        let mut t = flat(20, 20).spawn(SEEKER, Some(P1), SpawnAt::Point(start)).start();
        let goal = start + goal_from_start;
        move_to(&mut t, 1, &[1], goal);
        let path = drive_forward(&mut t, 400);
        assert_eq!(t.status(1), MoveStatus::Arrived, "{goal_from_start}");
        assert!(t.pos(1).distance(goal) < 1.0, "{goal_from_start}: {}", t.pos(1));
        let farthest = path.iter().map(|p| p.distance(goal)).fold(0.0, f32::max);
        assert!(farthest > RADIUS, "{goal_from_start}: a loop, {farthest} su out");
    }
}
