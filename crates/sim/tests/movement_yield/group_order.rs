//! A group order's units among themselves (design change 0027): a unit that arrived doesn't yield to the rest
//! of its own group order, but does to another unit of its owner.

use super::*;

/// The point the group of [`converging`] is ordered to.
const CONVERGE_ON: Vec2 = Vec2::new(390.0, 260.0);

/// Twelve Peacekeepers of player 1, 60 su apart in four columns of three (units 1 to 12), ordered in tick 2 to a
/// point inside their box; unit 13 of player 1 stands apart.
fn converging() -> TestSim {
    let mut t = ScenarioBuilder::new(OPEN).start();
    t.without_auto_targeting();
    for i in 0..12 {
        t.spawn_unit_at(
            1,
            PK,
            P1,
            v(300.0 + 60.0 * (i % 4) as f32, 200.0 + 60.0 * (i / 4) as f32),
        );
    }
    t.spawn_unit_at(1, PK, P1, v(100.0, 260.0))
        .order_at(2, P1, &ids(1..=12), move_to(CONVERGE_ON), false);
    t
}

/// 0027, Verification: "Converge 12 units on a point: they settle into a calm blob with no shuffling at the
/// centre." 0026, Verification: "the outer units stop within about 2 tiles of the point straight away, with no
/// re-route attempts." A unit that has arrived never moves again while the others come in, and the whole group
/// has arrived within a bounded time.
#[test]
fn a_converging_group_settles_without_shuffling_or_re_routing() {
    let mut t = converging();
    t.run(2);
    let mut arrived_at: Vec<Option<Vec2>> = vec![None; 12];
    // The farthest unit starts 108 su from the point: 27 ticks. (M4-W3) A unit stopped by the blob takes design
    // change 0323's contact step round the unit in its way, a whole width in six ticks, and walks at the point
    // again: it never stands `JAMMED_TICKS` in a row, so it has arrived when its stuck clock says so, `STUCK_TICKS`
    // after it last got nearer, where a unit shuffling a step along its blocker had arrived after 12. The group has
    // settled in tick 83 (53 until 0323): still without a re-route, nobody moving once arrived, all within the
    // radius.
    while arrived_at.iter().any(Option::is_none) {
        assert!(
            t.sim().tick() < 2 + 27 + 2 * STUCK_TICKS + 8,
            "not settled\n{}",
            t.ascii(None)
        );
        t.step();
        for (i, at) in arrived_at.iter_mut().enumerate() {
            let id = i as u32 + 1;
            match at {
                Some(at) => assert_eq!(t.pos(id), *at, "#{id} moved after it arrived\n{}", t.ascii(None)),
                None if t.status(id) == MoveStatus::Arrived => *at = Some(t.pos(id)),
                None => assert_eq!(t.status(id), MoveStatus::Moving),
            }
        }
    }
    for (i, at) in arrived_at.iter().enumerate() {
        let id = i as u32 + 1;
        assert_eq!(t.mover(id).repaths_left, MAX_REPATHS, "#{id} tried to re-route");
        assert!(at.unwrap().distance(CONVERGE_ON) <= gu(OCCUPIED_ARRIVAL_GU), "#{id}");
    }
    assert_eq!(failures(&t), 0);
    // Calm: nobody moves once every unit has arrived.
    for _ in 0..100 {
        t.step();
        for (i, at) in arrived_at.iter().enumerate() {
            assert_eq!(Some(t.pos(i as u32 + 1)), *at);
        }
    }
}

/// 0027, Verification: "Then walk a different own unit through the blob: the blob's units step aside for it." The
/// exception holds only within the group order.
#[test]
fn a_settled_group_steps_aside_for_another_unit_of_its_owner() {
    let mut t = converging();
    run_until_done(&mut t, &ids(1..=12), 100);
    let settled: Vec<Vec2> = (1..=12).map(|id| t.pos(id)).collect();
    let goal = v(700.0, 260.0);
    let from = t.sim().tick() + 1;
    t.order_at(from, P1, &[ObjectId(13)], move_to(goal), false).run(1);
    // 600 su: 150 ticks.
    run_until_done(&mut t, &[ObjectId(13)], from + 150 + 60);
    assert_eq!(
        (t.pos(13), t.status(13)),
        (goal, MoveStatus::Arrived),
        "{}",
        t.ascii(None)
    );
    assert_eq!(t.mover(13).repaths_left, MAX_REPATHS);
    let stepped_aside = (1..=12).filter(|&id| t.pos(id) != settled[id as usize - 1]).count();
    assert!(stepped_aside >= 1, "{}", t.ascii(None));
}
