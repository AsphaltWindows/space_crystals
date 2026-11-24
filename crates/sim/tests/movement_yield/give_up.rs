//! The give-up schedule (design change 0024): stuck after 24 ticks, three re-route attempts after waits of 8,
//! 16 and 32 ticks, then give up; a unit blocked for less than that carries on.

use super::*;

/// 0024, Verification: "Box a unit in completely: it stops trying and its order completes after about 7 s." Eight
/// enemy units stand around it. It is stuck 24 ticks after the order, looks for another route after waits of 8, 16
/// and 32 ticks, and gives up 24 ticks after the third attempt: 104 ticks, 6.5 s.
#[test]
fn a_boxed_in_unit_gives_up_after_about_seven_seconds() {
    let mut t = ScenarioBuilder::new(OPEN).start();
    t.without_auto_targeting();
    let at = v(500.0, 300.0);
    t.spawn_unit_at(1, PK, P1, at);
    for (dx, dy) in [(-1, -1), (0, -1), (1, -1), (-1, 0), (1, 0), (-1, 1), (0, 1), (1, 1)] {
        t.spawn_unit_at(1, PK, P2, at + v(24.0 * dx as f32, 24.0 * dy as f32));
    }
    t.order_at(2, P1, &[ObjectId(1)], move_to(v(900.0, 300.0)), false)
        .run(2);
    let mut repaths = Vec::new();
    while t.status(1) == MoveStatus::Moving {
        assert!(t.sim().tick() < 200, "{}", t.ascii(None));
        let left = t.mover(1).repaths_left;
        t.step();
        if t.mover(1).repaths_left < left {
            repaths.push(t.sim().tick() - 2);
        }
    }
    let waits = REPATH_WAITS;
    let first = STUCK_TICKS + waits[0];
    assert_eq!(repaths, [first, first + waits[1], first + waits[1] + waits[2]]);
    assert_eq!(repaths, [32, 48, 80]);
    let gave_up = t.sim().tick() - 2;
    assert_eq!(gave_up, GIVE_UP_TICKS, "104 ticks: 6.5 s");
    assert_eq!((t.status(1), t.pos(1)), (MoveStatus::Failed, at));
    assert!(
        t.events()
            .contains(&(t.sim().tick(), SimEvent::MoveFailed { unit: ObjectId(1) }))
    );
    t.step();
    assert!(is_idle(&t, 1), "its order completed");
}

/// 0024, Verification: "Block a unit briefly (under 1.5 s): it continues without re-routing." An enemy unit plugs
/// a corridor and is taken away 20 ticks after the walker ran into it.
#[test]
fn a_unit_blocked_for_less_than_the_stuck_time_carries_on_without_re_routing() {
    let mut t = ScenarioBuilder::new(CORRIDOR).start();
    t.without_auto_targeting();
    let goal = v(900.0, 96.0);
    t.spawn_unit_at(1, PK, P1, v(100.0, 96.0))
        .spawn_unit_at(1, GUARD, P2, v(500.0, 96.0))
        .order_at(2, P1, &[ObjectId(1)], move_to(goal), false)
        .run(3);
    // 370 su to the Guard, whose box begins at x 482: the walker finds no way forward from about tick 94 on.
    while t.mover(1).blocked_ticks == 0 {
        assert!(t.sim().tick() < 110);
        t.step();
    }
    let blocked = t.sim().tick();
    assert!(t.pos(1).x > 460.0, "against the Guard: {}", t.pos(1));
    let wait = STUCK_TICKS - 4;
    t.debug_at(blocked + wait, DebugCommand::Remove { id: ObjectId(2) });
    // 430 su more: 108 ticks.
    let done = run_until_done(&mut t, &[ObjectId(1)], blocked + wait + 108 + 4);
    assert_eq!((t.pos(1), t.status(1)), (goal, MoveStatus::Arrived));
    assert_eq!(t.mover(1).repaths_left, MAX_REPATHS, "it never re-routed");
    assert!(done >= blocked + wait + 100, "it waited: tick {done}");
    assert_eq!(failures(&t), 0);
}
