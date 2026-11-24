//! Group destinations by the magic box (`systems/controls/commands.md > Group Destinations (Magic Box)`, design
//! change 0021; `docs/movement.md > 6.4`), the checks of 0021's Verification: a group clicked far outside its box
//! arrives in its own shape, centred on the click; clicked inside, it converges and bunches up, and nobody fails; a
//! mixed group strings out, fast units first; an offset destination off the unit's ground or off the map is
//! retargeted (question Q0008); a single unit goes to the click as before; a unit walled out of its spot by its own
//! group has arrived within 3 grid units of it and gives up beyond (design changes 0169 and 0183). Every `TestSim`
//! checks the invariants I1–I7 after each tick (`docs/movement.md > 8`), and the runs replay to the same state.

use sc_sim::mover::OCCUPIED_ARRIVAL_GU;
use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{MoveStatus, ObjectId, ObjectKind, OrderKind, PlayerId, SimEvent, Target, Vec2, gu};

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

/// Units `1..=n`.
fn ids(n: usize) -> Vec<ObjectId> {
    (1..=n as u32).map(ObjectId).collect()
}

/// Player 1's `units` spawn in tick 1 and get one order, `kind`, in tick 2. The sim is at tick 2: the order has
/// started.
fn ordered(map: &str, units: &[(ObjectKind, Vec2)], kind: OrderKind) -> TestSim {
    let mut t = ScenarioBuilder::new(map).start();
    for &(unit, at) in units {
        t.spawn_unit_at(1, unit, P1, at);
    }
    t.order_at(2, P1, &ids(units.len()), kind, false);
    t.run(2);
    t
}

/// Unit `id`'s own destination for its current order (`Order::slot`).
fn slot(t: &TestSim, id: u32) -> Option<Vec2> {
    let orders = t.sim().object(ObjectId(id)).unwrap().orders.as_ref().unwrap();
    orders.current.as_ref().expect("an order").order.slot
}

/// Steps until no unit of `1..=n` is moving; the tick each one stopped moving. Panics, with the map, after
/// `max_tick`.
fn run_until_done(t: &mut TestSim, n: usize, max_tick: u32) -> Vec<u32> {
    let mut done = vec![0; n];
    loop {
        for (i, tick) in done.iter_mut().enumerate() {
            if *tick == 0 && t.status(i as u32 + 1) != MoveStatus::Moving {
                *tick = t.sim().tick();
            }
        }
        if done.iter().all(|&tick| tick > 0) {
            return done;
        }
        assert!(t.sim().tick() < max_tick, "still moving\n{}", t.ascii(None));
        t.step();
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

/// 0021, Verification: a line ordered far outside its box arrives as the same line, centred on the click.
#[test]
fn a_line_ordered_far_away_arrives_as_the_same_line_centred_on_the_click() {
    // Five Peacekeepers 40 su apart: the box is x 100..260 on y 100, its centre (180, 100).
    let line: Vec<(ObjectKind, Vec2)> = (0..5).map(|i| (PK, v(100.0 + 40.0 * i as f32, 100.0))).collect();
    let click = v(700.0, 500.0);
    let mut t = ordered(OPEN, &line, move_to(click));
    let shift = click - v(180.0, 100.0);
    for (i, &(_, start)) in line.iter().enumerate() {
        assert_eq!(slot(&t, i as u32 + 1), Some(start + shift));
    }
    // 656 su at 4 su per tick, side by side.
    run_until_done(&mut t, line.len(), 2 + 164 + 20);
    let ends: Vec<Vec2> = (1..=line.len() as u32).map(|id| t.pos(id)).collect();
    for (i, end) in ends.iter().enumerate() {
        assert_eq!(t.status(i as u32 + 1), MoveStatus::Arrived);
        assert_eq!(*end, line[i].1 + shift, "#{}", i + 1);
    }
    assert_eq!((ends[0] + ends[4]) / 2.0, click, "centred on the click");
    assert!(
        ends.windows(2).all(|pair| pair[1] - pair[0] == v(40.0, 0.0)),
        "{ends:?}"
    );
    assert_eq!(t.replay().run().state_hash(), t.sim().state_hash());
}

/// 0021, Verification: a click inside the box makes the group converge on it and bunch up; the crowd's arrival
/// rules stop every unit (`docs/movement.md > 6.5`, rule 1), and nobody fails.
#[test]
fn a_click_inside_the_box_makes_the_group_converge() {
    // Sixteen Peacekeepers 48 su apart in a square: the box is x 300..444, y 200..344.
    let square: Vec<(ObjectKind, Vec2)> = (0..16)
        .map(|i| (PK, v(300.0 + 48.0 * (i % 4) as f32, 200.0 + 48.0 * (i / 4) as f32)))
        .collect();
    let click = v(380.0, 260.0);
    let mut t = ordered(OPEN, &square, move_to(click));
    for id in 1..=16 {
        assert_eq!(slot(&t, id), None, "#{id} goes to the click itself");
    }
    run_until_done(&mut t, square.len(), 400);
    assert_nobody_failed(&t);
    for id in 1..=16 {
        assert_eq!(t.status(id), MoveStatus::Arrived, "#{id}\n{}", t.ascii(None));
    }
    // Bunched up: sixteen boxes of 24 su need a square of 96 su, so the farthest unit can't be much nearer than
    // 68 su; it started 116 su away. Everyone ends near the click, within `OCCUPIED_ARRIVAL_GU` of it (0026).
    let farthest = |points: Vec<Vec2>| points.iter().map(|p| p.distance(click)).fold(0.0_f32, f32::max);
    let before = farthest(square.iter().map(|u| u.1).collect());
    let after = farthest((1..=16).map(|id| t.pos(id)).collect());
    assert!(
        before > 115.0 && after <= 96.0,
        "the farthest unit is {after} su from the click, {before} before"
    );
    assert_eq!(t.replay().run().state_hash(), t.sim().state_hash());
}

/// A group standing in a line has a box without area: a click on the line itself is inside it, and the line closes
/// up on the click.
#[test]
fn a_line_clicked_on_itself_closes_up() {
    let line: Vec<(ObjectKind, Vec2)> = (0..5).map(|i| (PK, v(100.0 + 40.0 * i as f32, 300.0))).collect();
    // On the middle unit, which is there already.
    let click = v(180.0, 300.0);
    let mut t = ordered(OPEN, &line, move_to(click));
    assert!((1..=5).all(|id| slot(&t, id).is_none()));
    run_until_done(&mut t, line.len(), 200);
    assert_nobody_failed(&t);
    assert_eq!(t.pos(3), click);
    // Its neighbours close up to it, and the outer two stop at them, a step round them: each by the occupied
    // target rule, within `OCCUPIED_ARRIVAL_GU` of the click (`docs/movement.md > 6.5`, rule 1).
    for (id, &(_, start)) in (1..=5).zip(&line) {
        assert_eq!(t.status(id), MoveStatus::Arrived);
        let (before, after) = (start.distance(click), t.pos(id).distance(click));
        assert!(
            after <= before && after <= 96.0,
            "#{id} {after} su from the click\n{}",
            t.ascii(None)
        );
    }
    assert_eq!(
        (t.pos(2), t.pos(4)),
        (v(156.0, 300.0), v(204.0, 300.0)),
        "shoulder to shoulder"
    );
}

/// 0021, Verification, and "no formation movement": every unit goes at its own speed, so a mixed group strings out
/// on the way and its fast units arrive first; then it stands in its old shape again.
#[test]
fn a_mixed_group_strings_out_and_its_fast_units_arrive_first() {
    // Guards (5 su per tick) and Peacekeepers (4 su per tick) by turns, in a column 60 su apart; 800 su east.
    let column: Vec<(ObjectKind, Vec2)> = (0..6)
        .map(|i| (if i % 2 == 0 { GUARD } else { PK }, v(100.0, 100.0 + 60.0 * i as f32)))
        .collect();
    let mut t = ordered(OPEN, &column, move_to(v(900.0, 250.0)));
    let (guards, pks): (Vec<u32>, Vec<u32>) = (1..=6).partition(|id| id % 2 == 1);
    t.run_until(101);
    let x = |ids: &[u32]| ids.iter().map(|&id| t.pos(id).x).collect::<Vec<f32>>();
    let slowest_guard = x(&guards).into_iter().fold(f32::MAX, f32::min);
    let fastest_pk = x(&pks).into_iter().fold(f32::MIN, f32::max);
    assert!(
        slowest_guard - fastest_pk >= 90.0,
        "strung out by 100 su after 100 ticks\n{}",
        t.ascii(None)
    );
    let done = run_until_done(&mut t, column.len(), 2 + 200 + 20);
    let last_guard = guards.iter().map(|&id| done[id as usize - 1]).max().unwrap();
    let first_pk = pks.iter().map(|&id| done[id as usize - 1]).min().unwrap();
    assert!(last_guard < first_pk, "arrival ticks {done:?}");
    for (i, &(_, start)) in column.iter().enumerate() {
        assert_eq!(t.status(i as u32 + 1), MoveStatus::Arrived);
        // 0028: the box bounds the silhouettes, y 82 (the first Guard's top) to 412: its centre is (100, 247).
        assert_eq!(t.pos(i as u32 + 1), start + v(800.0, 3.0));
    }
}

/// 0021, Verification, question Q0008 (default): an offset destination on a cliff moves to the nearest point the
/// unit can stand on, and one beyond the map's edge is clamped inside the map. The units arrive there.
#[test]
fn an_offset_destination_on_a_cliff_or_off_the_map_is_retargeted() {
    // A cliff on tiles (14, 4..=5): x 896..960, y 256..384.
    let map = "
        ................
        ................
        ................
        ................
        ..............c.
        ..............c.
        ................
        ................
    ";
    // The box is x 100..300 on y 300, its centre (200, 300); the click moves the line by (750, 0).
    let line = [(PK, v(100.0, 300.0)), (PK, v(200.0, 300.0)), (PK, v(300.0, 300.0))];
    let mut t = ordered(map, &line, move_to(v(950.0, 300.0)));
    // (850, 300) is open; (950, 300) is on the cliff, 10 su from its east edge; (1050, 300) is off the map.
    let slots = [v(850.0, 300.0), v(972.0, 300.0), v(1012.0, 300.0)];
    for (i, &expected) in slots.iter().enumerate() {
        assert_eq!(slot(&t, i as u32 + 1), Some(expected));
    }
    run_until_done(&mut t, line.len(), 400);
    for (i, &expected) in slots.iter().enumerate() {
        assert_eq!(
            (t.status(i as u32 + 1), t.pos(i as u32 + 1)),
            (MoveStatus::Arrived, expected)
        );
    }
}

/// 0028, Verification: "Line up units horizontally and right-click between two of them: the group converges." and
/// "Right-click a few tiles above the line: the line moves there, keeping its shape." The box bounds the units'
/// silhouettes, so the line's box is as high as its units.
#[test]
fn a_line_clicked_between_two_units_converges_and_clicked_above_keeps_its_shape() {
    // Five Peacekeepers 40 su apart on y 300: the box is x 88..272, y 288..312, its centre (180, 300).
    let line: Vec<(ObjectKind, Vec2)> = (0..5).map(|i| (PK, v(100.0 + 40.0 * i as f32, 300.0))).collect();
    // Between the second and the third unit, a little below their centres.
    let between = v(160.0, 305.0);
    let mut t = ordered(OPEN, &line, move_to(between));
    assert!((1..=5).all(|id| slot(&t, id).is_none()), "everyone goes to the click");
    run_until_done(&mut t, line.len(), 200);
    assert_nobody_failed(&t);
    for (id, &(_, start)) in (1..=5).zip(&line) {
        assert_eq!(t.status(id), MoveStatus::Arrived);
        let (before, after) = (start.distance(between), t.pos(id).distance(between));
        assert!(after <= before && after <= 64.0, "#{id} {after} su from the click");
    }
    let width = |t: &TestSim| t.pos(5).x - t.pos(1).x;
    assert!(width(&t) < 110.0, "the line closed up from 160 su to {}", width(&t));

    // Three tiles above the line: outside the box, so the line moves by one vector.
    let above = v(180.0, 108.0);
    let mut t = ordered(OPEN, &line, move_to(above));
    for (id, &(_, start)) in (1..=5).zip(&line) {
        assert_eq!(slot(&t, id), Some(start + v(0.0, -192.0)));
    }
    run_until_done(&mut t, line.len(), 100);
    for (id, &(_, start)) in (1..=5).zip(&line) {
        assert_eq!(
            (t.status(id), t.pos(id)),
            (MoveStatus::Arrived, start + v(0.0, -192.0)),
            "#{id}"
        );
    }
}

/// 0025, Verification: "Move a line of units so that one unit's offset spot falls in water: that unit stops at the
/// nearest shore; the others take their offset spots."
#[test]
fn a_unit_whose_offset_spot_is_in_water_stops_at_the_nearest_shore() {
    // A pond on tiles (10..=12, 3..=5): x 640..832, y 192..384.
    let map = "
        ................
        ................
        ................
        ..........~~~...
        ..........~~~...
        ..........~~~...
        ................
        ................
    ";
    // The box's centre is (200, 300); the click moves the line by (500, 0).
    let line = [(PK, v(100.0, 300.0)), (PK, v(200.0, 300.0)), (PK, v(300.0, 300.0))];
    let mut t = ordered(map, &line, move_to(v(700.0, 300.0)));
    // (700, 300) is in the pond, 60 su from its west shore and 84 from its south shore: the unit stands against
    // the west shore. (600, 300) is open ground; (800, 300) is in the pond, 32 su from its east shore.
    let slots = [v(600.0, 300.0), v(628.0, 300.0), v(844.0, 300.0)];
    for (i, &expected) in slots.iter().enumerate() {
        assert_eq!(slot(&t, i as u32 + 1), Some(expected));
    }
    run_until_done(&mut t, line.len(), 400);
    assert_nobody_failed(&t);
    for (i, &expected) in slots.iter().enumerate() {
        assert_eq!(
            (t.status(i as u32 + 1), t.pos(i as u32 + 1)),
            (MoveStatus::Arrived, expected)
        );
    }
}

/// 0021, Verification: a single unit goes to the click itself, exactly as before the magic box, for Move,
/// AttackMove and Patrol alike; ordered together, units get their own destinations for each of them.
#[test]
fn a_single_unit_goes_to_the_click_as_before() {
    let click = v(500.0, 100.0);
    let mut t = ordered(OPEN, &[(PK, v(100.0, 100.0))], move_to(click));
    assert_eq!(slot(&t, 1), None);
    // 400 su at 4 su per tick, from tick 2 (as `unit_moves.rs` has it).
    let done = run_until_done(&mut t, 1, 200);
    assert_eq!((done[0], t.pos(1), t.status(1)), (1 + 100, click, MoveStatus::Arrived));
    // AttackMove and Patrol have no behaviour yet, so the orders are looked at in the queue, behind a move.
    let queued = |units: &[(ObjectKind, Vec2)]| {
        let mut t = ScenarioBuilder::new(OPEN).start();
        for &(unit, at) in units {
            t.spawn_unit_at(1, unit, P1, at);
        }
        let all = ids(units.len());
        t.order_at(2, P1, &all, move_to(v(900.0, 700.0)), false)
            .order_at(2, P1, &all, OrderKind::AttackMove(v(500.0, 400.0)), true)
            .order_at(2, P1, &all, OrderKind::Patrol(v(300.0, 600.0)), true);
        t.run(2);
        (1..=units.len() as u32)
            .map(|id| {
                let orders = t.sim().object(ObjectId(id)).unwrap().orders.as_ref().unwrap();
                orders.queue.iter().map(|order| order.slot).collect::<Vec<_>>()
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(queued(&[(PK, v(100.0, 100.0))]), [[None, None]]);
    // Two units: the box's centre is (120, 100).
    assert_eq!(
        queued(&[(PK, v(100.0, 100.0)), (PK, v(140.0, 100.0))]),
        [
            [Some(v(480.0, 400.0)), Some(v(280.0, 600.0))],
            [Some(v(520.0, 400.0)), Some(v(320.0, 600.0))]
        ]
    );
}

/// Design change 0183 (answering question Q0018 point 6), with design change 0169's radius: a unit walled out of its
/// magic-box spot by its own group, whose units arrived before it and don't make way for it (design change 0027),
/// counts as arrived within `OCCUPIED_ARRIVAL_GU` (3 grid units) of its spot, and gives up beyond it. Seven Guards
/// in a column along a dead-end corridor one Guard wide, clicked beyond its end: every offset spot lies in the water
/// and becomes the corridor's end (design change 0025), so the whole group crowds one spot. The Guards stack up
/// behind the first to get there, each against the one before, 36 su apart: the fifth and the sixth stop 144 and 180
/// su short of the spot (2.25 and 2.8 grid units) and have arrived, the seventh 216 su short (3.4) re-routes and
/// gives up.
#[test]
fn a_unit_walled_out_of_its_spot_by_its_own_group_arrives_within_three_grid_units() {
    // The corridor: y 64..128, from the map's west edge to x 768, water beyond.
    let map = "
        ~~~~~~~~~~~~~~~~
        ............~~~~
        ~~~~~~~~~~~~~~~~
    ";
    let column: Vec<(ObjectKind, Vec2)> = (0..7).map(|i| (GUARD, v(100.0 + 40.0 * i as f32, 96.0))).collect();
    let mut t = ordered(map, &column, move_to(v(900.0, 96.0)));
    let end = v(768.0 - 18.0, 96.0);
    for id in 1..=7 {
        assert_eq!(slot(&t, id), Some(end), "#{id}");
    }
    run_until_done(&mut t, column.len(), 800);
    let short_of_it: Vec<f32> = (1..=7).map(|id| t.pos(id).distance(end)).collect();
    for (id, &short) in (1..=7).zip(&short_of_it) {
        let expected = if short <= gu(OCCUPIED_ARRIVAL_GU) {
            MoveStatus::Arrived
        } else {
            MoveStatus::Failed
        };
        assert_eq!(t.status(id), expected, "#{id}, {short} su short\n{}", t.ascii(None));
    }
    // 0169's own cases, walled out more than 2 grid units short and arrived all the same; and one beyond 3.
    let arrived_beyond_two = (1..=7)
        .zip(&short_of_it)
        .filter(|&(id, &short)| short > gu(2.0) && t.status(id) == MoveStatus::Arrived);
    assert_eq!(arrived_beyond_two.count(), 2, "{short_of_it:?}");
    let failed = (1..=7).filter(|&id| t.status(id) == MoveStatus::Failed);
    assert_eq!(failed.count(), 1, "{short_of_it:?}");
    assert_eq!(t.replay().run().state_hash(), t.sim().state_hash());
}
