//! `systems/controls/behaviors.md > Patrolling`: cycling between the two ends and engaging on the way, the origin of
//! a queued Patrol (design change 0036) and failed legs (design change 0037).

use sc_sim::orders::Behaviour;
use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{MoveStatus, ObjectId, ObjectKind, OrderKind, PlayerId, SimEvent, Target, Tick, Vec2};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const PK: ObjectKind = ObjectKind::Peacekeeper; // 24 × 24 su, 4 su per tick, Range 4, Sight 5
const GUARD: ObjectKind = ObjectKind::Guard; // 36 × 36 su, 5 su per tick, Range 3, Sight 5

/// 32 × 24 open tiles: 2048 × 1536 su.
fn open() -> String {
    "................................\n".repeat(24)
}

/// A corridor one tile (64 su) wide and 26 long, y from 64 to 128.
const CORRIDOR: &str = "
    ~~~~~~~~~~~~~~~~~~~~~~~~~~
    ..........................
    ~~~~~~~~~~~~~~~~~~~~~~~~~~
";

fn v(x: f32, y: f32) -> Vec2 {
    Vec2::new(x, y)
}

fn id(n: u32) -> ObjectId {
    ObjectId(n)
}

fn ids(range: std::ops::RangeInclusive<u32>) -> Vec<ObjectId> {
    range.map(ObjectId).collect()
}

fn move_to(p: Vec2) -> OrderKind {
    OrderKind::Move(Target::Point(p))
}

fn exists(t: &TestSim, n: u32) -> bool {
    t.sim().object(id(n)).is_some()
}

fn pos(t: &TestSim, n: u32) -> Vec2 {
    t.sim().object(id(n)).expect("exists").pos
}

fn is_idle(t: &TestSim, n: u32) -> bool {
    t.sim().object(id(n)).unwrap().orders.as_ref().unwrap().is_idle()
}

fn current(t: &TestSim, n: u32) -> Option<OrderKind> {
    let orders = t.sim().object(id(n)).unwrap().orders.as_ref().unwrap();
    orders.current.as_ref().map(|active| active.order.kind)
}

fn behaviour(t: &TestSim, n: u32) -> Option<Behaviour> {
    let orders = t.sim().object(id(n)).unwrap().orders.as_ref().unwrap();
    orders.current.as_ref().map(|active| active.state.clone())
}

fn attack_ticks(t: &TestSim, attacker: u32) -> Vec<Tick> {
    t.events()
        .iter()
        .filter(|(_, e)| matches!(e, SimEvent::Attacked { attacker: a, .. } if *a == id(attacker)))
        .map(|&(tick, _)| tick)
        .collect()
}

/// The ticks in which a move of `unit` failed.
fn failed_moves(t: &TestSim, unit: u32) -> Vec<Tick> {
    t.events()
        .iter()
        .filter(|(_, e)| matches!(e, SimEvent::MoveFailed { unit: u } if *u == id(unit)))
        .map(|&(tick, _)| tick)
        .collect()
}

/// The brief's "Patrol cycling and engaging" (`> Patrolling`): the unit walks to and fro between where it stood and
/// the target location, and never completes. An enemy that comes into sight is engaged, as on an AttackMove, and
/// the patrol goes on when it is destroyed.
#[test]
fn a_patrol_cycles_and_engages() {
    let (a, b) = (v(300.0, 300.0), v(700.0, 300.0));
    let mut t = ScenarioBuilder::new(&open()).start();
    t.spawn_unit_at(1, PK, P1, a)
        .order_at(2, P1, &[id(1)], OrderKind::Patrol(b), false);
    let mut ends = Vec::new();
    t.run_until(2);
    while t.sim().tick() < 2 + 6 * 100 {
        t.step();
        let at = pos(&t, 1);
        if at == a || at == b {
            ends.push((t.sim().tick(), at));
        }
        assert_eq!(current(&t, 1), Some(OrderKind::Patrol(b)));
    }
    // 100 steps a leg, and the unit turns around in the tick after it arrives.
    let expected: Vec<(Tick, Vec2)> = (1..=6)
        .map(|leg| (1 + 100 * leg, if leg % 2 == 1 { b } else { a }))
        .collect();
    assert_eq!(ends, expected);
    assert!(matches!(behaviour(&t, 1), Some(Behaviour::Patrol(p)) if p.origin == a && p.destination == b));

    // A Guard appears beside the route and holds there, out of its own range of the route: the patrol turns on it.
    let now = t.sim().tick();
    t.spawn_unit_at(now + 1, GUARD, P2, v(500.0, 560.0))
        .order_at(now + 2, P2, &[id(2)], OrderKind::Hold, false);
    t.run_until(now + 2);
    while exists(&t, 2) {
        assert!(t.sim().tick() < now + 600, "the Guard outlasts it\n{}", t.ascii(None));
        t.step();
    }
    assert!(attack_ticks(&t, 1).len() >= 9, "{:?}", attack_ticks(&t, 1));
    assert_eq!(attack_ticks(&t, 2), [] as [Tick; 0]);
    assert_eq!(current(&t, 1), Some(OrderKind::Patrol(b)));
    // And goes on: both ends again.
    let killed = t.sim().tick();
    let (mut at_a, mut at_b) = (false, false);
    while !(at_a && at_b) {
        assert!(
            t.sim().tick() < killed + 400,
            "the patrol didn't go on\n{}",
            t.ascii(None)
        );
        t.step();
        at_a |= pos(&t, 1) == a;
        at_b |= pos(&t, 1) == b;
    }
}

/// Design change 0036's Verification: "Shift-queue Move to X, then Patrol to Y: after reaching X the unit patrols
/// between X and Y, never returning to its original spot."
#[test]
fn a_queued_patrol_starts_from_where_the_order_before_it_ended() {
    let (start, x, y) = (v(200.0, 200.0), v(600.0, 200.0), v(600.0, 600.0));
    let mut t = ScenarioBuilder::new(&open()).start();
    t.spawn_unit_at(1, PK, P1, start)
        .order_at(2, P1, &[id(1)], move_to(x), false)
        .order_at(2, P1, &[id(1)], OrderKind::Patrol(y), true);
    // 100 steps: the move ordered in tick 2 arrives in tick 101, and the Patrol starts in 102.
    t.run_until(101);
    assert_eq!((current(&t, 1), pos(&t, 1)), (Some(move_to(x)), x));
    let mut ends = Vec::new();
    while t.sim().tick() < 101 + 500 {
        t.step();
        let at = pos(&t, 1);
        if at == x || at == y {
            ends.push(at);
        }
        assert!(
            at.distance(start) >= 400.0,
            "it came back towards its original spot: {at}"
        );
    }
    assert_eq!(ends, [y, x, y, x, y]);
    assert!(matches!(behaviour(&t, 1), Some(Behaviour::Patrol(p)) if p.origin == x && p.destination == y));
}

/// Design change 0037's Verification: "Block one end of a patrol with Hold Position units: the unit gives up that
/// leg, turns around, and keeps patrolling the reachable part."
#[test]
fn a_failed_patrol_leg_counts_as_arrived_and_the_unit_turns_around() {
    let (origin, far) = (v(100.0, 96.0), v(1400.0, 96.0));
    let mut t = ScenarioBuilder::new(CORRIDOR).start();
    t.spawn_unit_at(1, PK, P1, origin)
        .spawn_unit_at(1, GUARD, P1, v(700.0, 96.0))
        .order_at(2, P1, &[id(2)], OrderKind::Hold, false)
        .order_at(2, P1, &[id(1)], OrderKind::Patrol(far), false);
    let mut back_at_origin = Vec::new();
    t.run_until(2);
    while t.sim().tick() < 1200 {
        t.step();
        if pos(&t, 1) == origin {
            back_at_origin.push(t.sim().tick());
        }
        assert_eq!(current(&t, 1), Some(OrderKind::Patrol(far)), "tick {}", t.sim().tick());
    }
    let failed = failed_moves(&t, 1);
    assert!(failed.len() >= 2, "it gave up the blocked leg each time: {failed:?}");
    // After each failed leg it walked all the way back: an arrival, which starts the count afresh.
    assert!(back_at_origin.len() >= 2, "{back_at_origin:?}");
    assert!(back_at_origin[0] > failed[0] && failed[1] > back_at_origin[0]);
    assert!(matches!(behaviour(&t, 1), Some(Behaviour::Patrol(p)) if p.failed_legs <= 1));
    assert_eq!(pos(&t, 2), v(700.0, 96.0), "the Guard on Hold never moved");
}

/// Design change 0037's Verification: "Box the unit in on both sides: after two failed legs it stops patrolling and
/// is idle."
#[test]
fn two_failed_patrol_legs_in_a_row_end_the_patrol() {
    let (origin, far) = (v(100.0, 96.0), v(1500.0, 96.0));
    let mut t = ScenarioBuilder::new(CORRIDOR).start();
    t.spawn_unit_at(1, PK, P1, origin)
        .order_at(2, P1, &[id(1)], OrderKind::Patrol(far), false);
    // When it is in the middle, 700 su on, Guards on Hold plug the corridor on both sides of it.
    t.spawn_unit_at(176, GUARD, P1, v(600.0, 96.0))
        .spawn_unit_at(176, GUARD, P1, v(1000.0, 96.0))
        .order_at(177, P1, &ids(2..=3), OrderKind::Hold, false);
    t.run_until(176);
    assert_eq!(pos(&t, 1), v(100.0 + 4.0 * 175.0, 96.0));
    while !is_idle(&t, 1) {
        assert!(t.sim().tick() < 800, "the patrol never ends\n{}", t.ascii(None));
        t.step();
    }
    let failed = failed_moves(&t, 1);
    assert_eq!(failed.len(), 2, "one leg each way: {failed:?}");
    assert_eq!(t.sim().tick(), failed[1] + 1, "the Patrol ends with the second");
    let stopped = pos(&t, 1);
    assert_eq!(stopped.x, 600.0 + 18.0 + 12.0, "against the western Guard");
    assert_eq!(t.status(1), MoveStatus::Failed);
    t.run(100);
    assert!(is_idle(&t, 1));
    assert_eq!(pos(&t, 1), stopped);
}
