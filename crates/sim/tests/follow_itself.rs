//! Design change 0168 (`systems/controls/commands.md > Move`: "Following itself = Stop", answering question Q0031): a
//! unit whose Move or Follow target is itself gets the equivalent of Stop: "the order ends immediately, as if
//! arrived". The others follow it normally; Shift-queued, "the step completes instantly when reached". It ends the
//! never-ending self-follow of slice M1-G3. (Through the client's right-click: `crates/game/tests/commands/
//! remembered.rs`.)

use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{MoveStatus, ObjectId, ObjectKind, OrderKind, PlayerId, SpawnAt, Target, Vec2};

const P1: PlayerId = PlayerId(1);

fn id(n: u32) -> ObjectId {
    ObjectId(n)
}

/// Player 1's six Peacekeepers on 30 × 20 open tiles, in two rows of three; no unit fights by itself.
fn six() -> TestSim {
    let mut builder = ScenarioBuilder::new(&vec![".".repeat(30); 20].join("\n"));
    for i in 0..6 {
        let at = Vec2::new(600.0 + 80.0 * (i % 3) as f32, 400.0 + 80.0 * (i / 3) as f32);
        builder = builder.spawn(ObjectKind::Peacekeeper, Some(P1), SpawnAt::Point(at));
    }
    let mut t = builder.start();
    t.without_auto_targeting();
    t
}

fn current(t: &TestSim, n: u32) -> Option<OrderKind> {
    let orders = t.sim().object(id(n)).unwrap().orders.as_ref().unwrap();
    orders.current.as_ref().map(|active| active.order.kind)
}

/// 0168's Verification: "Select 6 Peacekeepers (one walking), right-click the walking one: it stops; the other 5
/// follow it." The right-click's order for each of them is a Move to the walking one (`Sim::default_order`: its own
/// unit); the walking one's ends in the tick it starts, its move with it, and it stays; the others follow it.
#[test]
fn a_unit_ordered_after_itself_stops_and_the_others_follow_it() {
    let mut t = six();
    let walking = 1;
    t.order_at(
        1,
        P1,
        &[id(walking)],
        OrderKind::Move(Target::Point(Vec2::new(1700.0, 400.0))),
        false,
    );
    t.run_until(20);
    assert_eq!(t.status(walking), MoveStatus::Moving);
    let follow = OrderKind::Move(Target::Object(id(walking)));
    for n in 1..=6 {
        assert_eq!(
            t.sim().default_order(id(n), Target::Object(id(walking))),
            Some(follow),
            "unit {n}"
        );
    }
    let all: Vec<ObjectId> = (1..=6).map(id).collect();
    t.order_at(21, P1, &all, follow, false).run_until(21);
    assert_eq!(current(&t, walking), None, "it stopped, as if it had arrived");
    assert_ne!(t.status(walking), MoveStatus::Moving);
    let stopped = t.pos(walking);
    t.run(80);
    assert_eq!(current(&t, walking), None);
    assert!(t.pos(walking).distance(stopped) < 4.0, "it stays: {}", t.pos(walking));
    for n in 2..=6 {
        assert_eq!(current(&t, n), Some(follow), "unit {n} follows");
        assert!(t.pos(n).distance(stopped) < 200.0, "unit {n}: {}", t.pos(n));
    }
}

/// 0168: "Shift-queued: the step completes instantly when reached." A unit's route: a Move to a point, a Move to
/// itself, then a Move to another point. When it gets to the first point, the step after itself starts and ends in
/// the same tick, and the unit sets off for the second point; no tick ends with it current.
#[test]
fn a_queued_move_after_itself_completes_at_once() {
    let mut t = six();
    let (first, second) = (Vec2::new(900.0, 700.0), Vec2::new(1300.0, 700.0));
    t.order_at(1, P1, &[id(1)], OrderKind::Move(Target::Point(first)), false)
        .order_at(1, P1, &[id(1)], OrderKind::Move(Target::Object(id(1))), true)
        .order_at(1, P1, &[id(1)], OrderKind::Move(Target::Point(second)), true)
        .run_until(1);
    let mut seen = Vec::new();
    while current(&t, 1).is_some() {
        assert!(t.sim().tick() < 600, "the route should be done by now");
        let kind = current(&t, 1);
        if seen.last() != Some(&kind) {
            seen.push(kind);
        }
        t.step();
    }
    assert_eq!(
        seen,
        [
            Some(OrderKind::Move(Target::Point(first))),
            Some(OrderKind::Move(Target::Point(second)))
        ],
        "never current at a tick's end"
    );
    assert_eq!(t.pos(1), second);
}
