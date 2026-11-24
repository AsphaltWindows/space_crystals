//! Occupied targets (design changes 0023, 0026 and 0169): a wall of units on Hold doesn't budge; a unit
//! stopped within 3 grid units of its destination, or of the structure it moves to, behind units that won't
//! move has arrived; one further away gives up.

use super::*;

/// Eight Peacekeepers of player 1 shoulder to shoulder across [`PASSAGE`] at x 500, on Hold Position from tick 2:
/// units 1 to 8. Unit 9, of player 1, comes from the west.
fn hold_wall() -> TestSim {
    let mut t = ScenarioBuilder::new(PASSAGE).start();
    t.without_auto_targeting();
    for i in 0..8 {
        t.spawn_unit_at(1, PK, P1, v(500.0, 76.0 + 24.0 * i as f32));
    }
    t.spawn_unit_at(1, PK, P1, v(100.0, 160.0))
        .order_at(2, P1, &ids(1..=8), OrderKind::Hold, false);
    t
}

/// 0023, Verification: "Through units on Hold Position: they don't budge and the mover paths around or gives up."
/// 0026, Verification: "A unit ordered to a point 3 tiles behind a wall of Hold Position units keeps trying
/// (re-routes) and eventually gives up instead of counting as arrived."
#[test]
fn a_wall_of_hold_units_doesnt_budge_and_a_unit_sent_far_behind_it_gives_up() {
    let mut t = hold_wall();
    // Three tiles behind the wall. The unit meets it at x 476: 216 su short, more than 3 GU (192 su, 0169).
    t.order_at(2, P1, &[ObjectId(9)], move_to(v(692.0, 160.0)), false)
        .run(2);
    let mut repaths = Vec::new();
    while t.status(9) == MoveStatus::Moving {
        assert!(t.sim().tick() < 2 + 94 + 3 * GIVE_UP_TICKS, "{}", t.ascii(None));
        let left = t.mover(9).repaths_left;
        t.step();
        if t.mover(9).repaths_left < left {
            repaths.push(t.sim().tick());
        }
    }
    assert_eq!(t.status(9), MoveStatus::Failed, "{}", t.ascii(None));
    assert_eq!(
        repaths.len(),
        usize::from(MAX_REPATHS),
        "it tried every re-route: {repaths:?}"
    );
    assert!(
        t.events()
            .contains(&(t.sim().tick(), SimEvent::MoveFailed { unit: ObjectId(9) }))
    );
    t.step();
    assert!(t.pos(9).x <= 476.0 && is_idle(&t, 9));
    for i in 0..8 {
        assert_eq!(t.pos(i + 1), v(500.0, 76.0 + 24.0 * i as f32), "#{} budged", i + 1);
    }
}

/// 0026: the same wall, and a destination less than 2 GU behind the spot where the wall stops the unit. The wall
/// will never move, so the unit counts as arrived, without a re-route attempt and before the stuck clock runs out.
/// (Within 3 GU since design change 0169: [`a_unit_blocked_two_and_a_half_grid_units_short_by_idle_units_has_arrived`].)
#[test]
fn a_unit_stopped_within_two_grid_units_behind_units_that_wont_move_has_arrived() {
    let mut t = hold_wall();
    let goal = v(600.0, 160.0);
    t.order_at(2, P1, &[ObjectId(9)], move_to(goal), false);
    // It reaches the wall after 376 su, 94 ticks.
    let done = run_until_done(&mut t, &[ObjectId(9)], 2 + 94 + STUCK_TICKS + REPATH_WAITS[0]);
    assert_eq!(t.status(9), MoveStatus::Arrived, "{}", t.ascii(None));
    assert_eq!(t.mover(9).repaths_left, MAX_REPATHS, "no re-route attempt");
    assert!(t.pos(9).distance(goal) <= gu(OCCUPIED_ARRIVAL_GU) && t.pos(9).x <= 476.0);
    assert!(done < 2 + 94 + STUCK_TICKS, "straight away: tick {done}");
    assert_eq!(failures(&t), 0);
    for i in 0..8 {
        assert_eq!(t.pos(i + 1), v(500.0, 76.0 + 24.0 * i as f32));
    }
}

/// After whatever `builder` holds: eight idle Peacekeepers of player 2 shoulder to shoulder across [`PASSAGE`] at x
/// 500, the wall, and a Peacekeeper of player 1 west of it at (100, 160), the walker, all spawned in tick 1. The wall
/// has no order and no goal, auto-targeting is off, and an enemy never steps aside (design change 0023): it stops
/// the walker at x 476 for good. The ids of the wall and of the walker.
fn idle_enemy_wall(builder: ScenarioBuilder) -> (TestSim, Vec<ObjectId>, ObjectId) {
    let mut t = builder.start();
    t.without_auto_targeting();
    let first = t.sim().objects().count() as u32 + 1;
    for i in 0..8 {
        t.spawn_unit_at(1, PK, P2, v(500.0, 76.0 + 24.0 * i as f32));
    }
    t.spawn_unit_at(1, PK, P1, v(100.0, 160.0));
    (t, ids(first..=first + 7), ObjectId(first + 8))
}

/// Design change 0169 (`systems/controls/behaviors.md > MovingToLocation > Occupied target location`: "within 3
/// grid units", which 0026 had at 2): a walker stopped 2.5 grid units short of its destination by idle units that
/// won't move for it has arrived. It found no way forward for `JAMMED_TICKS` behind units without a goal of their
/// own, so it arrives without a re-route attempt and before the stuck clock runs out; with 0026's 2 grid units it
/// re-routed and gave up, as it still does 3.4 grid units short
/// ([`a_wall_of_hold_units_doesnt_budge_and_a_unit_sent_far_behind_it_gives_up`]).
#[test]
fn a_unit_blocked_two_and_a_half_grid_units_short_by_idle_units_has_arrived() {
    let (mut t, wall, walker) = idle_enemy_wall(ScenarioBuilder::new(PASSAGE));
    let goal = v(476.0 + gu(2.5), 160.0);
    t.order_at(2, P1, &[walker], move_to(goal), false);
    // It reaches the wall after 376 su, 94 ticks.
    let done = run_until_done(&mut t, &[walker], 2 + 94 + STUCK_TICKS + REPATH_WAITS[0]);
    assert_eq!(t.status(walker.0), MoveStatus::Arrived, "{}", t.ascii(None));
    assert_eq!(t.mover(walker.0).repaths_left, MAX_REPATHS, "no re-route attempt");
    assert!(done < 2 + 94 + STUCK_TICKS, "straight away: tick {done}");
    let short = t.pos(walker.0).distance(goal);
    assert!(
        short > gu(2.0) && short <= gu(OCCUPIED_ARRIVAL_GU),
        "{short} su short of its destination"
    );
    t.step();
    assert!(is_idle(&t, walker.0), "the Move is complete");
    assert_eq!(failures(&t), 0);
    for (i, id) in wall.iter().enumerate() {
        assert_eq!(t.pos(id.0), v(500.0, 76.0 + 24.0 * i as f32), "#{} budged", id.0);
    }
}

/// Design change 0169 for `> MovingToObject`, whose "Blocked near the target" is the same rule with the same radius,
/// measured edge to edge (design change 0033): the wall of [`idle_enemy_wall`], and player 1's Generator beyond it on
/// tiles (10, 1) to (11, 2), its west edge at x 640. A Move to the Generator stops at the wall, the walker's edge
/// 152 su (2.4 grid units) from the Generator's, and is complete there, without a re-route attempt or a failure.
#[test]
fn a_move_to_a_structure_blocked_within_three_grid_units_of_it_is_complete() {
    let builder =
        ScenarioBuilder::new(PASSAGE).spawn(ObjectKind::Generator, Some(P1), SpawnAt::Tile(TilePos::new(10, 1)));
    let (mut t, wall, walker) = idle_enemy_wall(builder);
    let generator = ObjectId(1);
    let to_generator = OrderKind::Move(Target::Object(generator));
    t.order_at(2, P1, &[walker], to_generator, false);
    let done = run_until_done(&mut t, &[walker], 2 + 94 + STUCK_TICKS + REPATH_WAITS[0]);
    assert_eq!(t.status(walker.0), MoveStatus::Arrived, "{}", t.ascii(None));
    assert_eq!(t.mover(walker.0).repaths_left, MAX_REPATHS, "no re-route attempt");
    assert!(done < 2 + 94 + STUCK_TICKS, "straight away: tick {done}");
    let walker_shape = Shape::of(t.sim().object(walker).unwrap());
    let gap = walker_shape.gap(&Shape::of(t.sim().object(generator).unwrap()));
    assert!(
        gap > gu(2.0) && gap <= gu(OCCUPIED_ARRIVAL_GU),
        "{gap} su from the Generator, edge to edge"
    );
    t.step();
    assert!(is_idle(&t, walker.0), "the Move is complete");
    assert_eq!(failures(&t), 0);
    for (i, id) in wall.iter().enumerate() {
        assert_eq!(t.pos(id.0), v(500.0, 76.0 + 24.0 * i as f32), "#{} budged", id.0);
    }
}
