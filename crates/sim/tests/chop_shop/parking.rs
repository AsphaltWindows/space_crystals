//! (M6-GF) Design change 0451's Verification at the Chop Shop (answering Q0177; `core/entities.md > Entrance and exit >
//! Exit Doorway` point 7: "a newly released vehicle whose rally point is already taken by parked vehicles **parks at
//! the nearest free spot around the point** (the free spot nearest the point where it fits), not in a line behind the
//! last one back toward the door. Applies to the rally move of a newly released vehicle, at any rally distance";
//! `sim/producers/park.rs`): five Seekers, five Hands of Heaven (tracked), and five Seekers rallied farther away, each
//! time clustering round the point and leaving the doorway clear. "With the point cleared, the stall stays the
//! player's" (design change 0447) is `exit.rs`'s `a_second_seeker_with_no_rally_point_stands_in_the_doorway_behind_the_
//! first`; the Factory's cases are `tests/factory/parking.rs`'.

use sc_sim::production::Doorway;

use super::*;

const HAND: ObjectKind = ObjectKind::HandOfHeaven;
/// The parking radius (`Production::parking_radius_gu`, 3 grid units), space units.
const RADIUS: f32 = 3.0 * 64.0;
/// [`base`]'s Chop Shop's default rally point (design change 0446): 3 grid units west of [`EXIT`], out of side C.
const DEFAULT: Vec2 = Vec2::new(EXIT.x - RADIUS, EXIT.y);

/// Runs `t` until the Chop Shop has released `count` vehicles, at most `limit` ticks, checking every tick that it never
/// waits for its doorway (`ProductionStatus::waiting`) and calling `each` after every tick; then 200 ticks more for the
/// last to park. Returns the vehicles, in the order they came out.
fn release_all(t: &mut TestSim, count: usize, limit: Tick, mut each: impl FnMut(&mut TestSim)) -> Vec<ObjectId> {
    let end = t.sim().tick() + limit;
    let out = |t: &TestSim| produced(t).into_iter().map(|(_, _, unit)| unit).collect::<Vec<_>>();
    while out(t).len() < count {
        assert!(t.sim().tick() < end, "{} out\n{}", out(t).len(), t.ascii(None));
        step_checked(t);
        let stalled = t.sim().production_status(CS).is_some_and(|status| status.waiting);
        assert!(!stalled, "tick {}: stalled\n{}", t.sim().tick(), t.ascii(None));
        each(t);
    }
    run_for(t, 200);
    out(t)
}

/// The cluster's checks: every vehicle out of the doorway, stopped and idle, within the parking radius and its own
/// reach (the half diagonal of its Silhouette) of `point`, no two boxes overlapping, the first on the point; and a
/// cluster rather than a line toward the door: some of them stand off the line from the door through the point, on
/// each side of it.
fn assert_cluster(t: &TestSim, vehicles: &[ObjectId], point: Vec2) {
    let door = Doorway::of(t.sim().object(CS).expect("the Chop Shop"))
        .unwrap()
        .doors()
        .next()
        .unwrap();
    let across = Vec2::new(-door.outward().y, door.outward().x);
    let (mut left, mut right) = (0, 0);
    for (i, &unit) in vehicles.iter().enumerate() {
        let o = t.sim().object(unit).expect("on the map");
        assert_eq!(t.mover(unit.0).exit, None, "{i}: out of the doorway");
        assert_eq!(t.mover(unit.0).momentum, Vec2::ZERO, "{i}: stopped");
        assert!(o.orders.as_ref().is_some_and(|orders| orders.is_idle()), "{i}: idle");
        let reach = o.def().half_extents().unwrap().length();
        assert!(
            o.pos.distance(point) <= RADIUS + reach,
            "{i} round the point: {}, {} su off\n{}",
            o.pos,
            o.pos.distance(point),
            t.ascii(None)
        );
        let side = (o.pos - point).dot(across);
        left += usize::from(side < -16.0);
        right += usize::from(side > 16.0);
        for &other in &vehicles[i + 1..] {
            assert!(!shape(t, unit).overlaps(&shape(t, other)), "{unit:?} and {other:?}");
        }
    }
    let first = pos(t, vehicles[0]);
    assert!(first.distance(point) < 1.0, "the first on the point: {first}");
    assert!(
        left >= 1 && right >= 1,
        "a cluster, not a line: {left} and {right}\n{}",
        t.ascii(None)
    );
}

/// 0451's Verification 1: "A Chop Shop keeps its default rally point, with five Seekers queued. They cluster around the
/// point, and all five leave without the door stalling." Ten Recruits inside queue five Seekers at once: each comes out
/// 320 frames after the one before, the doorway never holding one back, and they park round the point.
#[test]
fn five_seekers_cluster_round_the_default_rally_point_and_never_stall_the_doorway() {
    let (mut t, _) = filled(10);
    let start = t.sim().tick() + 1;
    for _ in 0..5 {
        t.at(start, P1, train(&[CS], RecruitKind::Plain));
    }
    let seekers = release_all(&mut t, 5, 6 * FRAMES, |_| {});
    let ticks: Vec<Tick> = produced(&t).iter().map(|&(tick, _, _)| tick).collect();
    let expected: Vec<Tick> = (0..5).map(|n| ticks[0] + n * FRAMES).collect();
    assert_eq!(ticks, expected, "one after the other");
    assert_cluster(&t, &seekers, DEFAULT);
}

/// 0451's Verification 2: "The same with Hands of Heaven" (tracked, three Recruits each, 480 frames;
/// `factions/cults/units/hand_of_heaven.md`). The Chop Shop holds ten Recruits, so the player keeps it going: ten
/// inside queue three Hands at once, and as each comes out, three more of player 1's Recruits, waiting outside, go in,
/// and a fourth and a fifth are queued as soon as they can be. All five park round the default rally point, the
/// doorway never stalling.
#[test]
fn five_hands_of_heaven_cluster_round_the_default_rally_point_and_never_stall_the_doorway() {
    let (mut t, _) = filled(10);
    let next = t.sim().tick() + 1;
    let waiting = block(&mut t, next, 6, 3, Vec2::new(1150.0, 300.0));
    let hand = |t: &mut TestSim| {
        let tick = t.sim().tick() + 1;
        let command = production(
            &[CS],
            ProductionCommand::TrainFrom {
                kind: HAND,
                from: RecruitKind::Plain,
            },
        );
        t.at(tick, P1, command);
    };
    for _ in 0..3 {
        hand(&mut t);
    }
    let (mut queued, mut sent, mut seen) = (3, 0, 0);
    let hands = release_all(&mut t, 5, 5 * 480 + 600, |t| {
        let out = produced(t).len();
        if out > seen && sent < waiting.len() {
            let tick = t.sim().tick() + 1;
            t.at(tick, P1, enter(&waiting[sent..sent + 3], CS));
            sent += 3;
        }
        seen = out;
        let command = production(
            &[CS],
            ProductionCommand::TrainFrom {
                kind: HAND,
                from: RecruitKind::Plain,
            },
        );
        if queued < 5 && validate(t, command).is_ok() {
            hand(t);
            queued += 1;
        }
    });
    for &unit in &hands {
        assert_eq!(t.sim().object(unit).map(|o| o.kind), Some(HAND));
    }
    assert_cluster(&t, &hands, DEFAULT);
}

/// 0451's Verification 3: "With a rally point set farther away, they cluster there": the rally point set 368 su west of
/// side C ([`RALLY`]), ten Recruits inside queue five Seekers, which park round it.
#[test]
fn with_a_rally_point_farther_away_they_cluster_there() {
    let (mut t, _) = filled(10);
    let start = t.sim().tick() + 1;
    t.at(start, P1, rally(&[CS], RALLY));
    for _ in 0..5 {
        t.at(start, P1, train(&[CS], RecruitKind::Plain));
    }
    let seekers = release_all(&mut t, 5, 6 * FRAMES, |_| {});
    assert_cluster(&t, &seekers, RALLY);
}
