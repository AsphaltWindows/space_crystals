//! Units meeting in a corridor (`docs/movement.md > 6.3`): two Peacekeepers pass each other, two Guards can't
//! and give up (question Q0007); and an idle unit of the mover's own in its way steps aside (design change
//! 0022).

use super::*;

/// `factions/cults/units/recruit.md` (throughput note), `docs/movement.md > 6.3` rule 4: a 64 su gap fits two
/// 24 su units side by side. Two Peacekeepers meeting head-on in it each step to their right and pass.
#[test]
fn two_peacekeepers_pass_each_other_in_a_corridor() {
    let mut t = ScenarioBuilder::new(CORRIDOR).start();
    t.without_auto_targeting();
    let (west, east) = (v(96.0, 96.0), v(928.0, 96.0));
    t.spawn_unit_at(1, PK, P1, west)
        .spawn_unit_at(1, PK, P2, east)
        .order_at(2, P1, &[ObjectId(1)], move_to(east), false)
        .order_at(2, P2, &[ObjectId(2)], move_to(west), false);
    let mut passed_at = None;
    // 832 su at 4 su per tick is 208 ticks; passing costs a few.
    for _ in 0..208 + 24 {
        t.step();
        let (a, b) = (t.pos(1), t.pos(2));
        if passed_at.is_none() && a.x > b.x {
            passed_at = Some(t.sim().tick());
            // Like traffic: each keeps to its right, the eastbound unit south (+y), the westbound north.
            assert!(a.y - b.y >= 24.0, "{a} and {b}");
        }
    }
    assert!(passed_at.is_some(), "they never passed:\n{}", t.ascii(None));
    assert_eq!((t.pos(1), t.status(1)), (east, MoveStatus::Arrived));
    assert_eq!((t.pos(2), t.status(2)), (west, MoveStatus::Arrived));
    assert!(
        t.events()
            .iter()
            .all(|(_, e)| !matches!(e, SimEvent::MoveFailed { .. }))
    );
}

/// Question Q0007, gap G3 (default): two Guards (36 su) can't pass in a 64 su corridor. Both give up by the
/// Failure rule (`systems/controls/behaviors.md > MovingToLocation > Failure`), stop, and their orders complete.
#[test]
fn two_guards_head_on_in_a_corridor_both_fail() {
    let mut t = ScenarioBuilder::new(CORRIDOR).start();
    t.without_auto_targeting();
    let (west, east) = (v(96.0, 96.0), v(928.0, 96.0));
    t.spawn_unit_at(1, GUARD, P1, west)
        .spawn_unit_at(1, GUARD, P2, east)
        .order_at(2, P1, &[ObjectId(1)], move_to(east), false)
        .order_at(2, P2, &[ObjectId(2)], move_to(west), false);
    // They meet after (832 − 36) / (2 × 5) = 80 ticks. (M4-W3) Their contact steps (design change 0323) take them
    // from wall to wall across the corridor's middle, which with their closing up is once more half a step nearer:
    // their clocks stand from 16 ticks after they met (202 in all).
    let done = run_until_done(&mut t, &ids(1..=2), 2 + 80 + GIVE_UP_TICKS + 8 + 16);
    assert_eq!((t.status(1), t.status(2)), (MoveStatus::Failed, MoveStatus::Failed));
    let failed: Vec<&SimEvent> = t.events().iter().map(|(_, e)| e).collect();
    for unit in ids(1..=2) {
        assert!(failed.contains(&&SimEvent::MoveFailed { unit }));
    }
    // They stand, face to face, and stay.
    let (a, b) = (t.pos(1), t.pos(2));
    assert!(a.x < b.x && b.x - a.x < 64.0, "{a} and {b}");
    t.run(40);
    assert_eq!((t.pos(1), t.pos(2)), (a, b));
    for id in 1..=2 {
        assert_eq!(t.mover(id).locomotion, Locomotion::Stationary);
        assert_eq!(t.mover(id).goal, MoveGoal::None);
        assert!(t.sim().object(ObjectId(id)).unwrap().orders.as_ref().unwrap().is_idle());
    }
    assert!(done > 82, "they met first");
}

/// `systems/units.md > UnitCollision > Ground Collision > Friendly yielding` (design change 0022): an idle unit of
/// the mover's owner standing exactly in its way takes one short sidestep, without turning, and stays there (no
/// way back); an enemy's idle unit never moves, and the mover walks around it.
#[test]
fn an_idle_unit_in_the_way_steps_aside_if_it_is_the_movers_own() {
    let mut t = ScenarioBuilder::new(OPEN).start();
    t.without_auto_targeting();
    let (start, idle, goal) = (v(100.0, 300.0), v(400.0, 300.0), v(800.0, 300.0));
    t.spawn_unit_at(1, PK, P1, start)
        .spawn_unit_at(1, GUARD, P1, idle)
        .spawn_unit_at(1, PK, P2, v(600.0, 304.0))
        .order_at(2, P1, &[ObjectId(1)], move_to(goal), false);
    // 700 su at 4 su per tick is 175 ticks; two detours.
    let done = run_until_done(&mut t, &ids(1..=1), 2 + 175 + 40);
    assert_eq!((t.pos(1), t.status(1)), (goal, MoveStatus::Arrived));
    // The Guard stepped ahead and aside, to the side the mover didn't go round on: one sidestep of its own width,
    // 36 su (`YIELD_SIDESTEP_WIDTHS`), the most it takes.
    let stepped = t.pos(2) - idle;
    assert!(stepped.x > 0.0 && stepped.y < 0.0, "{stepped}");
    assert!((stepped.length() - 36.0).abs() < 1e-3, "{stepped}");
    assert_eq!(t.pos(3), v(600.0, 304.0));
    assert_eq!(t.sim().object(ObjectId(2)).unwrap().facing, 0.0);
    assert_eq!(t.mover(2).sidestep, None);
    assert!(done >= 177);
}

/// `systems/units.md > UnitCollision > Ground Collision`, "The contact step" (design change 0323): "the step is at
/// **90°** to its heading (straight sideways), and its length is **the unit's own width** [...] so it clears the lane
/// it was in". An Agent (36 su wide, 6 su a tick) walks east at an enemy's Agent, which stands and never yields:
/// in contact it steps straight south, its Right, for six ticks, 36 su in all and not a step forward, then walks on
/// past it and arrives.
#[test]
fn a_blocked_agent_steps_fully_sideways_by_its_own_width() {
    let mut t = ScenarioBuilder::new(OPEN).start();
    t.without_auto_targeting();
    let (start, goal) = (v(100.0, 300.0), v(700.0, 300.0));
    t.spawn_unit_at(1, AGENT, P1, start)
        .spawn_unit_at(1, AGENT, P2, v(400.0, 300.0))
        .order_at(2, P1, &[ObjectId(1)], move_to(goal), false);
    let mut aside = Vec::new();
    t.run_until(3);
    while t.status(1) == MoveStatus::Moving {
        assert!(t.sim().tick() < 2 + 100 + 30, "still moving:\n{}", t.ascii(None));
        let from = t.pos(1);
        t.step();
        let step = t.pos(1) - from;
        if step.x == 0.0 && step.y != 0.0 {
            aside.push(step.y);
        }
    }
    assert_eq!(aside, [6.0; 6], "straight south, its own width in all");
    assert_eq!((t.pos(1), t.status(1)), (goal, MoveStatus::Arrived));
    assert_eq!(t.pos(2), v(400.0, 300.0), "the enemy's Agent never moved");
}

/// Design change 0323's Verification: "Two Agents meeting head-on in a lane with room to the side [...] both pass";
/// `systems/units.md > UnitCollision > Ground Collision`: "Two units meeting head-on in the open pass each other
/// almost without delay". In a lane two tiles wide and in the open, each keeps to its right and both arrive, within
/// a second of the time the walk alone takes.
#[test]
fn two_agents_meeting_head_on_pass_each_other() {
    let lane = "
        ~~~~~~~~~~~~~~~~
        ................
        ................
        ~~~~~~~~~~~~~~~~
    ";
    for (map, y) in [(lane, 128.0), (OPEN, 300.0)] {
        let mut t = ScenarioBuilder::new(map).start();
        t.without_auto_targeting();
        let (west, east) = (v(96.0, y), v(928.0, y));
        t.spawn_unit_at(1, AGENT, P1, west)
            .spawn_unit_at(1, AGENT, P2, east)
            .order_at(2, P1, &[ObjectId(1)], move_to(east), false)
            .order_at(2, P2, &[ObjectId(2)], move_to(west), false);
        // 832 su at 6 su per tick is 139 ticks.
        let done = run_until_done(&mut t, &ids(1..=2), 2 + 139 + 16);
        assert_eq!((t.pos(1), t.status(1)), (east, MoveStatus::Arrived));
        assert_eq!((t.pos(2), t.status(2)), (west, MoveStatus::Arrived));
        println!("two Agents head-on: both arrived by tick {done}");
    }
}
