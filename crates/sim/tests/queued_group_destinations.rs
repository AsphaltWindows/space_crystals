//! Shift-queued group orders and the magic box (`systems/controls/commands.md > Group Destinations (Magic Box)`,
//! design changes 0029 and 0030; `docs/movement.md > 6.4`), the checks of both entries' Verification:
//! - 0030: every unit's destination is resolved once, at the click, and kept in its own order. Units that start a
//!   queued leg at different times, or that lose a member on the way, still arrive in the group's shape.
//! - 0029: a queued order builds the box where the units' earlier orders leave them. "Far north, then back"
//!   brings a line back as the line, and a queued converge followed by a far click moves the blob on as a blob.
//!
//! Every `TestSim` checks the invariants I1–I7 after each tick (`docs/movement.md > 8`), and every run replays to
//! the same state.

use sc_sim::orders::Order;
use sc_sim::testing::{ScenarioBuilder, TestSim, dangling_references};
use sc_sim::{DebugCommand, MoveGoal, MoveStatus, ObjectId, ObjectKind, OrderKind, PlayerId, SimEvent, Target, Vec2};

const P1: PlayerId = PlayerId(1);
const PK: ObjectKind = ObjectKind::Peacekeeper; // 24 × 24 su, 4 su per tick
const GUARD: ObjectKind = ObjectKind::Guard; // 36 × 36 su, 5 su per tick

/// 16 × 12 open tiles: 1024 × 768 su.
const OPEN: &str = "
    ................
    ................
    ................
    ................
    ................
    ................
    ................
    ................
    ................
    ................
    ................
    ................
";

fn v(x: f32, y: f32) -> Vec2 {
    Vec2::new(x, y)
}

fn move_to(p: Vec2) -> OrderKind {
    OrderKind::Move(Target::Point(p))
}

/// Units `1..=4`.
fn four() -> Vec<ObjectId> {
    (1..=4).map(ObjectId).collect()
}

/// Four units 80 su apart on y 650, from x 100: Peacekeepers, or Peacekeepers and Guards by turns.
fn line(mixed: bool) -> Vec<(ObjectKind, Vec2)> {
    (0..4)
        .map(|i| {
            let kind = if mixed && i % 2 == 1 { GUARD } else { PK };
            (kind, v(100.0 + 80.0 * i as f32, 650.0))
        })
        .collect()
}

/// Player 1's `units` spawn in tick 1; the sim is at tick 1.
fn spawned(units: &[(ObjectKind, Vec2)]) -> TestSim {
    let mut t = ScenarioBuilder::new(OPEN).start();
    for &(unit, at) in units {
        t.spawn_unit_at(1, unit, P1, at);
    }
    t.run(1);
    t
}

/// Unit `id`'s orders, the running one first.
fn orders(t: &TestSim, id: u32) -> Vec<Order> {
    let orders = t.sim().object(ObjectId(id)).unwrap().orders.as_ref().unwrap();
    orders
        .current
        .iter()
        .map(|active| &active.order)
        .chain(&orders.queue)
        .cloned()
        .collect()
}

/// Unit `id`'s own destination (`Order::slot`) for its order to `click`, running or waiting.
fn slot(t: &TestSim, id: u32, click: Vec2) -> Option<Vec2> {
    let order = orders(t, id).into_iter().find(|order| order.kind == move_to(click));
    order.unwrap_or_else(|| panic!("#{id} has no order to {click}")).slot
}

/// The order unit `id` is carrying out.
fn running(t: &TestSim, id: u32) -> Option<Order> {
    let orders = t.sim().object(ObjectId(id)).unwrap().orders.as_ref().unwrap();
    orders.current.as_ref().map(|active| active.order.clone())
}

/// Whether unit `id` has done all it was ordered.
fn idle(t: &TestSim, id: u32) -> bool {
    orders(t, id).is_empty() && t.status(id) != MoveStatus::Moving
}

/// Steps until units `ids` are idle, calling `each_tick` after every step. Panics, with the map, after `max_tick`.
fn run_until_idle(t: &mut TestSim, ids: &[u32], max_tick: u32, mut each_tick: impl FnMut(&TestSim)) {
    while !ids.iter().all(|&id| idle(t, id)) {
        assert!(t.sim().tick() < max_tick, "still busy\n{}", t.ascii(None));
        t.step();
        each_tick(t);
    }
}

fn assert_nobody_failed(t: &TestSim) {
    let failed: Vec<_> = t
        .events()
        .iter()
        .filter(|(_, e)| matches!(e, SimEvent::MoveFailed { .. }))
        .collect();
    assert!(failed.is_empty(), "{failed:?}\n{}", t.ascii(None));
}

/// 0029, Verification: "Line up 4 units horizontally. Shift-right-click far north, then Shift-right-click back at
/// the starting spot: the line goes north and returns as the same line, not a blob."
#[test]
fn a_line_queued_far_north_and_back_returns_as_the_same_line() {
    // The box is x 88..352, y 638..662, its centre (220, 650).
    let line = line(false);
    let (north, back) = (v(220.0, 150.0), v(220.0, 650.0));
    let mut t = spawned(&line);
    // The click back comes a tick later, while the first leg runs. The line has gone 4 su north: the click is
    // inside the box around where the units stand, and outside the box around where they will stand.
    t.order_at(2, P1, &four(), move_to(north), true)
        .order_at(3, P1, &four(), move_to(back), true)
        .run_until(3);
    for (id, &(_, start)) in (1..=4).zip(&line) {
        assert_eq!(slot(&t, id, north), Some(start - v(0.0, 500.0)));
        assert_eq!(
            slot(&t, id, back),
            Some(start),
            "#{id} goes back to its own spot, not to the click"
        );
    }
    let mut went_north = [false; 4];
    run_until_idle(&mut t, &[1, 2, 3, 4], 2 + 2 * 125 + 20, |t| {
        for (i, &(_, start)) in line.iter().enumerate() {
            went_north[i] |= t.pos(i as u32 + 1) == start - v(0.0, 500.0);
        }
    });
    assert_eq!(went_north, [true; 4]);
    assert_nobody_failed(&t);
    for (id, &(_, start)) in (1..=4).zip(&line) {
        assert_eq!(
            (t.status(id), t.pos(id)),
            (MoveStatus::Arrived, start),
            "#{id}\n{}",
            t.ascii(None)
        );
    }
    assert_eq!(t.replay().run().state_hash(), t.sim().state_hash());
}

/// 0029, Verification: "Shift-right-click inside the box (converge), then Shift-right-click far away: the blob
/// moves there as a blob."
#[test]
fn a_queued_converge_then_a_far_queued_click_moves_the_blob_as_a_blob() {
    let line = line(false);
    // Between the second and the third unit, inside the box: they converge. Both clicks come in one tick.
    let (between, far) = (v(220.0, 650.0), v(220.0, 150.0));
    let mut t = spawned(&line);
    t.order_at(2, P1, &four(), move_to(between), true)
        .order_at(2, P1, &four(), move_to(far), true)
        .run_until(2);
    for id in 1..=4 {
        assert_eq!(slot(&t, id, between), None, "#{id} converges on the click");
        // The planned ends are all the converge point, so the box is one silhouette around it: every unit goes
        // to the far click itself, the blob it will be, and not to its place in the line.
        assert_eq!(slot(&t, id, far), Some(far), "#{id}");
    }
    run_until_idle(&mut t, &[1, 2, 3, 4], 400, |_| {});
    assert_nobody_failed(&t);
    // They bunch up round the far click, as after a converge (the occupied target rule, `docs/movement.md > 6.5`):
    // each unit within 1.5 GU of it. Kept as the line, the end units would stand 120 su from it.
    for id in 1..=4 {
        let off = t.pos(id).distance(far);
        assert_eq!(t.status(id), MoveStatus::Arrived, "#{id}");
        assert!(off <= 96.0, "#{id} ended {off} su from the click\n{}", t.ascii(None));
    }
    assert_eq!(t.replay().run().state_hash(), t.sim().state_hash());
}

/// 0030, Verification: "Line of 4 mixed-speed units. Shift-queue far north, then back to the start: units turn
/// around at different times but end in the original line at the start." Each unit starts its leg back alone,
/// when its own leg north ends, and heads for the spot resolved at the click: nothing recomputes it then (as a
/// group of one, it would converge on the click).
#[test]
fn a_mixed_line_queued_north_and_back_turns_at_different_times_and_ends_in_its_line() {
    // Peacekeepers (4 su per tick) and Guards (5): the box is x 88..358 (the second Guard reaches 18 su east of
    // its centre), y 632..668, its centre (223, 650). Both clicks come in one tick.
    let line = line(true);
    let (north, back) = (v(223.0, 150.0), v(223.0, 650.0));
    let mut t = spawned(&line);
    t.order_at(2, P1, &four(), move_to(north), true)
        .order_at(2, P1, &four(), move_to(back), true)
        .run_until(2);
    let resolved: Vec<Option<Vec2>> = (1..=4).map(|id| slot(&t, id, back)).collect();
    assert_eq!(resolved, line.iter().map(|&(_, start)| Some(start)).collect::<Vec<_>>());
    // The tick each unit turned back: its leg back started, from its spot in the north, for its resolved spot.
    let mut turned = [None; 4];
    run_until_idle(&mut t, &[1, 2, 3, 4], 2 + 2 * 125 + 20, |t| {
        for (i, &(_, start)) in line.iter().enumerate() {
            let id = i as u32 + 1;
            let Some(order) = running(t, id).filter(|order| order.kind == move_to(back)) else {
                continue;
            };
            if turned[i].is_none() {
                turned[i] = Some(t.sim().tick());
                assert_eq!(order.slot, resolved[i], "#{id}");
                assert_eq!(t.mover(id).goal, MoveGoal::Point(start), "#{id}");
                let came_from = t.sim().object(ObjectId(id)).unwrap().prev_pos;
                assert_eq!(came_from, start - v(0.0, 500.0), "#{id} turned at its spot");
            }
        }
    });
    // 500 su from tick 2: the Guards arrive in tick 101 and the Peacekeepers in tick 126 (`t − 1 + ⌈d / speed⌉`),
    // and each turns in the next tick.
    assert_eq!(turned, [Some(127), Some(102), Some(127), Some(102)]);
    assert_nobody_failed(&t);
    for (id, &(_, start)) in (1..=4).zip(&line) {
        assert_eq!(
            (t.status(id), t.pos(id)),
            (MoveStatus::Arrived, start),
            "#{id}\n{}",
            t.ascii(None)
        );
    }
    assert_eq!(t.sim().tick(), 252, "the Peacekeepers arrived last, in tick 251");
    assert_eq!(t.replay().run().state_hash(), t.sim().state_hash());
}

/// 0030, Verification: "Kill one unit mid-route: the others still end in their spots." The unit at the line's west
/// end is removed on the way north, as a death removes it (`Sim::remove_objects`): the box of the three left would
/// be 80 su narrower, but their spots were resolved at the click and stay where they were.
#[test]
fn a_unit_killed_mid_route_leaves_the_others_spots_unchanged() {
    let line = line(false);
    let (north, back) = (v(220.0, 150.0), v(220.0, 650.0));
    let mut t = spawned(&line);
    t.order_at(2, P1, &four(), move_to(north), true)
        .order_at(6, P1, &four(), move_to(back), true)
        .debug_at(60, DebugCommand::Remove { id: ObjectId(1) })
        .run_until(6);
    let resolved: Vec<Vec<Order>> = (2..=4).map(|id| orders(&t, id)).collect();
    for (id, &(_, start)) in (2..=4).zip(&line[1..]) {
        assert_eq!(slot(&t, id, back), Some(start), "#{id}");
    }
    t.run_until(60);
    assert!(t.sim().object(ObjectId(1)).is_none());
    assert!(dangling_references(t.sim()).is_empty());
    assert_eq!(
        (2..=4).map(|id| orders(&t, id)).collect::<Vec<_>>(),
        resolved,
        "the others' orders are as given"
    );
    run_until_idle(&mut t, &[2, 3, 4], 2 + 2 * 125 + 20, |_| {});
    assert_nobody_failed(&t);
    for (id, &(_, start)) in (2..=4).zip(&line[1..]) {
        assert_eq!(
            (t.status(id), t.pos(id)),
            (MoveStatus::Arrived, start),
            "#{id}\n{}",
            t.ascii(None)
        );
    }
    assert_eq!(t.replay().run().state_hash(), t.sim().state_hash());
}
