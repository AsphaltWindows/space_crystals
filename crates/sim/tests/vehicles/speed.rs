//! Acceptance, **Speed** (slice M5-V2): "from a standstill, full speed after 8 frames; a stop takes 8 frames and about
//! 32–36 su". The speed comes before the move by it (`systems/units.md > UnitCollision > Collision Response`: "1.
//! Velocity: add this frame's acceleration [...] 2. Displacement"), so the moves are 1, 2 … 8 su; a Stop's braking
//! moves 7, 6 … 1 su, 28 su in all, at rest in its 8th tick (`movement/wheeled.rs`). On its way to a goal it brakes
//! where its braking distance is the path that is left, and lands on the goal.

use super::*;

/// From rest, facing its way, the Seeker is at full speed in its 8th tick, holds it, brakes once at the end, by
/// Deceleration a tick, and lands exactly on its goal, at rest, never past it.
#[test]
fn from_rest_a_seeker_is_at_full_speed_in_its_eighth_tick_and_brakes_onto_its_goal() {
    let mut t = flat(24, 5)
        .spawn(SEEKER, Some(P1), SpawnAt::Point(centre(1, 2)))
        .start();
    let goal = centre(20, 2); // 1216 su east, the way it faces
    move_to(&mut t, 1, &[1], goal);
    let mut speeds = Vec::new();
    while t.status(1) != MoveStatus::Arrived {
        let (d, turn) = step(&mut t, 1);
        assert!(
            d.y == 0.0 && d.x >= 0.0 && turn == 0.0,
            "tick {}: {d} {turn}",
            t.sim().tick()
        );
        assert!(t.pos(1).x <= goal.x, "tick {}: past its goal", t.sim().tick());
        if d.x > 0.0 || !speeds.is_empty() {
            speeds.push(d.x);
        }
        assert!(speeds.len() < 300, "{}", t.ascii(None));
    }
    assert_eq!(t.pos(1), goal, "exactly on it");
    assert_eq!(t.mover(1).momentum, Vec2::ZERO, "at rest");
    assert_eq!(&speeds[..9], [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 8.0]);
    let braking = &speeds[speeds.iter().rposition(|&s| s == TOP).expect("it cruised") + 1..];
    assert!(
        braking.windows(2).all(|pair| pair[0] - pair[1] <= 1.0 + 1e-4),
        "{braking:?}"
    );
    assert!(braking.len() <= 8 && braking.iter().sum::<f32>() <= 36.0, "{braking:?}");
    for _ in 0..10 {
        assert_eq!(step(&mut t, 1).0, Vec2::ZERO, "it stays there");
    }
}

/// A Stop at full speed: it brakes by Deceleration, 7, 6 … 1 su, 28 su in all, and is at rest in the 8th tick, still
/// facing its way: a vehicle doesn't turn while it stops (`systems/combat.md > LocomotionOrientationConstraints`:
/// "Stopping + Turning: invalid").
#[test]
fn a_stop_at_full_speed_takes_eight_ticks_and_28_su() {
    let mut t = flat(40, 5)
        .spawn(SEEKER, Some(P1), SpawnAt::Point(centre(1, 2)))
        .start();
    move_to(&mut t, 1, &[1], centre(38, 2));
    t.run_until(20);
    let (d, _) = step(&mut t, 1);
    assert_eq!(d, Vec2::new(TOP, 0.0), "at full speed");
    let tick = t.sim().tick() + 1;
    t.order_at(tick, P1, &[ObjectId(1)], OrderKind::Stop, false);
    let mut moves = Vec::new();
    for _ in 0..10 {
        let (d, turn) = step(&mut t, 1);
        assert!(d.y == 0.0 && turn == 0.0, "{d} {turn}");
        moves.push(d.x);
    }
    assert_eq!(moves, [7.0, 6.0, 5.0, 4.0, 3.0, 2.0, 1.0, 0.0, 0.0, 0.0]);
    assert_eq!(moves.iter().sum::<f32>(), 28.0);
    assert_eq!(t.mover(1).momentum, Vec2::ZERO);
}

/// The Prayer-mobile "drives as the Seeker", at ForwardMaxSpeed 6 (`factions/cults/units/prayer_mobile.md`).
#[test]
fn a_prayer_mobile_tops_at_six() {
    let mut t = flat(40, 5)
        .spawn(ObjectKind::PrayerMobile, Some(P1), SpawnAt::Point(centre(1, 2)))
        .start();
    move_to(&mut t, 1, &[1], centre(38, 2));
    let moves: Vec<f32> = (0..12).map(|_| step(&mut t, 1).0.x).filter(|&x| x > 0.0).collect();
    assert_eq!(&moves[..7], [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 6.0]);
}
