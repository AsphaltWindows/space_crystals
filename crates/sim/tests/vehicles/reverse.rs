//! Acceptance, **Reverse** (slice M5-V2): "W drives it backwards at up to 3 su/frame, turning at |v| / r"
//! (`systems/controls/commands.md > Reverse`, `systems/controls/command_behaviors.md > ReversingToLocation`: "Same as
//! MovingToLocation, except: Plan uses Reversing(path) instead of Moving(path)"; the Seeker's ReverseMaxSpeed 3 su a
//! frame and ReverseAcceleration 0.5). Only units that can back up take the order (`systems/controls/commands.md >
//! Reverse`: "Only available to bases with CanReverse = true"): the gate drops the others, and refuses it if none can.

use sc_sim::mover::Locomotion;
use sc_sim::{Command, PlayerCommand, Reject};

use super::*;

fn reverse_to(t: &mut TestSim, tick: u32, ids: &[u32], to: Vec2) {
    let units: Vec<ObjectId> = ids.iter().map(|&id| ObjectId(id)).collect();
    t.order_at(tick, P1, &units, OrderKind::Reverse(to), false);
}

/// Runs unit 1 until its move ends, at most `ticks`, checking each tick that it backs up (it moves against its
/// facing, or not at all) at 3 su a tick at most, turning by no more than its speed allows. Returns its moves.
fn back_up(t: &mut TestSim, ticks: u32) -> Vec<Vec2> {
    let mut moves = Vec::new();
    for _ in 0..ticks {
        let was = ahead(t, 1);
        let (d, turn) = step(t, 1);
        let tick = t.sim().tick();
        assert!(d.dot(was) <= 1e-4, "tick {tick}: forward, {d}");
        assert!(d.length() <= BACK + 1e-4, "tick {tick}: {d}");
        assert!(
            turn.abs() <= d.length() / RADIUS + 1e-5,
            "tick {tick}: turned {turn} on a move of {d}"
        );
        if d != Vec2::ZERO && t.status(1) == MoveStatus::Moving {
            assert_eq!(t.mover(1).locomotion, Locomotion::Reversing, "tick {tick}");
        }
        moves.push(d);
        if done(t, 1) {
            return moves;
        }
    }
    panic!("not there after {ticks} ticks: {:?}\n{}", t.mover(1), t.ascii(None));
}

/// Reverse to a point 300 su straight behind it: it backs up, 0.5 su a tick faster each tick up to 3, facing as it
/// did, brakes and stops on the point. Then a Move drives it forward again.
#[test]
fn a_seeker_backs_up_to_a_point_behind_it_at_3_su_a_tick() {
    let start = centre(15, 3);
    let mut t = flat(24, 6).spawn(SEEKER, Some(P1), SpawnAt::Point(start)).start();
    let goal = start - Vec2::new(300.0, 0.0);
    reverse_to(&mut t, 1, &[1], goal);
    let moves = back_up(&mut t, 300);
    assert_eq!(t.status(1), MoveStatus::Arrived);
    assert_eq!(t.pos(1), goal);
    assert_eq!(facing(&t, 1), 0.0, "facing east as it did");
    let speeds: Vec<f32> = moves.iter().map(|d| -d.x).filter(|&s| s > 0.0).collect();
    assert_eq!(&speeds[..7], [0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 3.0]);
    // Forward again.
    let tick = t.sim().tick() + 1;
    move_to(&mut t, tick, &[1], start);
    t.run(3);
    assert!(t.mover(1).velocity.x > 0.0 && t.mover(1).locomotion == Locomotion::Moving);
}

/// Reverse to a point behind it and off to its left: it backs along an arc, turning at |v| / r at most, its tail
/// swinging towards the point, and stops on it.
#[test]
fn backing_up_to_one_side_it_turns_at_its_speed_over_its_radius() {
    let start = centre(15, 10);
    let mut t = flat(24, 20).spawn(SEEKER, Some(P1), SpawnAt::Point(start)).start();
    let goal = start + Vec2::new(-300.0, -200.0);
    reverse_to(&mut t, 1, &[1], goal);
    let moves = back_up(&mut t, 600);
    assert_eq!(t.status(1), MoveStatus::Arrived);
    assert!(t.pos(1).distance(goal) < 1.0, "{}", t.pos(1));
    let tail = -ahead(&t, 1);
    assert!(tail.y < -0.2, "its tail turned north, towards the point: {tail}");
    assert!(moves.iter().all(|d| d.length() <= BACK + 1e-4));
}

/// A Reverse given at full speed forward: it brakes first, without turning (Stopping), then backs up.
#[test]
fn a_reverse_at_full_speed_brakes_first() {
    let start = centre(2, 3);
    let mut t = flat(40, 6).spawn(SEEKER, Some(P1), SpawnAt::Point(start)).start();
    move_to(&mut t, 1, &[1], centre(38, 3));
    t.run_until(20);
    assert_eq!(t.mover(1).velocity, Vec2::new(TOP, 0.0));
    reverse_to(&mut t, 21, &[1], start);
    let mut moves = Vec::new();
    for _ in 0..12 {
        let (d, turn) = step(&mut t, 1);
        assert_eq!(turn, 0.0);
        moves.push(d.x);
    }
    assert_eq!(&moves[..8], [7.0, 6.0, 5.0, 4.0, 3.0, 2.0, 1.0, 0.0]);
    assert_eq!(&moves[8..], [-0.5, -1.0, -1.5, -2.0]);
}

/// Only units that can back up take a Reverse: the Guard among the units keeps the order it had; given to Guards
/// alone it is refused, `Reject::CannotReverse`.
#[test]
fn only_units_that_can_reverse_take_the_order() {
    let mut t = flat(24, 10)
        .spawn(SEEKER, Some(P1), SpawnAt::Point(centre(12, 5)))
        .spawn(ObjectKind::Guard, Some(P1), SpawnAt::Point(centre(5, 5)))
        .start();
    let alone = PlayerCommand {
        player: P1,
        command: Command::Order {
            units: vec![ObjectId(2)],
            kind: OrderKind::Reverse(centre(3, 5)),
            queue: false,
        },
    };
    t.step();
    assert_eq!(t.sim().validate(&alone), Err(Reject::CannotReverse));
    reverse_to(&mut t, 2, &[1, 2], centre(8, 5));
    t.run(2);
    assert_eq!(t.status(1), MoveStatus::Moving, "the Seeker backs up");
    assert_eq!(t.status(2), MoveStatus::Idle, "the Guard ignores it");
    let current = |t: &TestSim, id: u32| {
        let o = t.sim().object(ObjectId(id)).unwrap();
        o.orders
            .as_ref()
            .and_then(|orders| orders.current.as_ref())
            .map(|a| a.order.kind)
    };
    assert!(matches!(current(&t, 1), Some(OrderKind::Reverse(_))));
    assert_eq!(current(&t, 2), None);
}
