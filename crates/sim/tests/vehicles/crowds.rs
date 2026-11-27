//! Acceptance, **Crowds** (slice M5-V2): "eight Seekers cross a field of idle own infantry, with no overlap and within
//! a time bound", and design change 0353's Verification: "infantry idle in its path step aside; an idle Seeker bumped
//! by infantry stays put" (`systems/units.md > UnitCollision > Ground Collision > Who yields > Vehicles`: "an **idle
//! vehicle doesn't yield** (it can't take a sidestep; it stands like a unit on Hold Position). **Own idle infantry
//! yield to a moving vehicle** as to any unit"). The invariants (I1: no overlap) hold every tick.

use super::*;

const MILITANT: ObjectKind = ObjectKind::Militant;

/// Where Militant `i` (0 to 23) of [`eight_seekers_cross_a_field_of_idle_own_infantry`] stands: a grid of 6 × 4,
/// 60 su apart, so 36 su between two, too little for a Seeker, 40 su wide, to pass.
fn in_the_field(i: u32) -> Vec2 {
    Vec2::new(1200.0 + 60.0 * (i % 6) as f32, 600.0 + 60.0 * (i / 6) as f32)
}

/// Eight Seekers in two columns on the west of a field, and 24 idle Militants of their owner standing close together
/// in the middle of it, across their way ([`in_the_field`]). The Seekers are sent east across it as a group: the
/// Militants in their way step aside, and all eight arrive (each on its own spot of the group's, the magic box)
/// within 600 ticks.
#[test]
fn eight_seekers_cross_a_field_of_idle_own_infantry() {
    let mut builder = flat(48, 24);
    for i in 0..8 {
        let at = Vec2::new(150.0 + 120.0 * (i % 2) as f32, 520.0 + 100.0 * (i / 2) as f32);
        builder = builder.spawn(SEEKER, Some(P1), SpawnAt::Point(at));
    }
    for i in 0..24 {
        builder = builder.spawn(MILITANT, Some(P1), SpawnAt::Point(in_the_field(i)));
    }
    let mut t = builder.start();
    let seekers: Vec<u32> = (1..=8).collect();
    move_to(&mut t, 1, &seekers, Vec2::new(2600.0, 700.0));
    while seekers.iter().any(|&id| !done(&t, id)) {
        t.step();
        assert!(t.sim().tick() < 600, "{}", t.ascii(None));
    }
    for &id in &seekers {
        assert_eq!(t.status(id), MoveStatus::Arrived, "Seeker {id}");
        assert!(t.pos(id).x > 2300.0, "Seeker {id} at {}", t.pos(id));
    }
    let stepped_aside = (0..24).filter(|&i| t.pos(9 + i) != in_the_field(i)).count();
    assert!(stepped_aside > 0, "the Militants in the way stepped aside");
    println!(
        "across in {} ticks; {stepped_aside} of the 24 Militants stepped aside",
        t.sim().tick()
    );
}

/// "Infantry idle in its path step aside": an idle Militant stands on the Seeker's line east, 740 su ahead of it. One
/// idle unit alone in its way is no group to go round (`movement/collide/lanes.rs`): the Seeker drives on at it and
/// asks it to make way, it steps aside, and the Seeker drives by, never more than 20 su off its line (round it would
/// be 32 su), and arrives on its goal.
#[test]
fn idle_infantry_in_a_seeker_s_path_step_aside() {
    let line = centre(0, 5).y;
    let militant_at = Vec2::new(900.0, line);
    let mut t = flat(30, 10)
        .spawn(SEEKER, Some(P1), SpawnAt::Point(centre(2, 5)))
        .spawn(MILITANT, Some(P1), SpawnAt::Point(militant_at))
        .start();
    let goal = centre(26, 5);
    move_to(&mut t, 1, &[1], goal);
    let mut widest: f32 = 0.0;
    while !done(&t, 1) {
        t.step();
        widest = widest.max((t.pos(1).y - line).abs());
        assert!(t.sim().tick() < 300, "{:?}\n{}", t.mover(1), t.ascii(None));
    }
    assert_eq!(t.status(1), MoveStatus::Arrived, "{}", t.ascii(None));
    assert_eq!(t.pos(1), goal);
    assert_ne!(t.pos(2), militant_at, "the Militant stepped aside");
    assert!(widest < 20.0, "by it, not round it: {widest} su off its line");
    println!(
        "by in {} ticks, {widest} su off its line at most; the Militant to {}",
        t.sim().tick(),
        t.pos(2)
    );
}

/// "An idle Seeker bumped by infantry stays put": an idle Seeker stands in the way of two Militants of its owner sent
/// east past it, and one of the other player. Whoever walks into it, it never moves; they go round it.
#[test]
fn an_idle_seeker_bumped_by_infantry_stays_put() {
    let seeker_at = centre(10, 5);
    let mut t = flat(24, 10)
        .spawn(SEEKER, Some(P1), SpawnAt::Point(seeker_at))
        .spawn(MILITANT, Some(P1), SpawnAt::Point(centre(4, 5)))
        .spawn(MILITANT, Some(P1), SpawnAt::Point(centre(4, 5) + Vec2::new(0.0, 30.0)))
        .spawn(MILITANT, Some(P2), SpawnAt::Point(centre(4, 5) - Vec2::new(0.0, 30.0)))
        .start();
    t.without_auto_targeting();
    let goal = centre(18, 5);
    move_to(&mut t, 1, &[2, 3], goal);
    t.order_at(1, P2, &[ObjectId(4)], OrderKind::Move(Target::Point(goal)), false);
    let facing_was = facing(&t, 1);
    while [2, 3, 4].iter().any(|&id| !done(&t, id)) {
        t.step();
        assert_eq!(t.pos(1), seeker_at, "tick {}: the Seeker moved", t.sim().tick());
        assert!(t.sim().tick() < 400, "{}", t.ascii(None));
    }
    assert_eq!(facing(&t, 1), facing_was);
    for id in 2..=4 {
        assert!(t.pos(id).x > seeker_at.x, "Militant {id} got past it: {}", t.pos(id));
    }
}
