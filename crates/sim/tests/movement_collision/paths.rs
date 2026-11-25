//! Paths under collision: goals that can't be reached (question Q0008), the one repath an object spawned on
//! the path forces (`docs/movement.md > 5.6`), Shift-queued waypoints visited in order, and the deterministic
//! counters of a group move (`> 5.8`).

use super::*;

/// Question Q0008, G5 (default): a goal that can't be reached (on a cliff) ends at the reachable point nearest to
/// it, and a unit enclosed by water goes as near to an outside goal as its pocket allows. Both arrive; neither
/// fails or searches the map.
#[test]
fn a_goal_on_a_cliff_and_an_enclosed_unit_end_at_the_nearest_point() {
    let mut t = ScenarioBuilder::new(
        "
        ................
        ..~~~~......ccc.
        ..~..~......ccc.
        ..~..~......ccc.
        ..~~~~..........
        ................
        ",
    )
    .start();
    let cliff = TilePos::new(13, 2).center();
    let outside = v(480.0, 352.0);
    t.spawn_unit_at(1, PK, P1, v(480.0, 160.0))
        .spawn_unit_at(1, PK, P1, v(230.0, 160.0)) // in the pocket, tiles (3..=4, 2..=3)
        .order_at(2, P1, &[ObjectId(1)], move_to(cliff), false)
        .order_at(2, P1, &[ObjectId(2)], move_to(outside), false);
    t.run(2);
    assert!(t.sim().nav_counters().expansions < 40, "{:?}", t.sim().nav_counters());
    run_until_done(&mut t, &ids(1..=2), 200);
    assert_eq!((t.status(1), t.status(2)), (MoveStatus::Arrived, MoveStatus::Arrived));
    // The cliff's nearest open tile is two tiles from its middle.
    assert_eq!(t.pos(1).distance(cliff), 128.0);
    // The pocket's tile nearest the goal is (4, 3).
    assert_eq!(t.pos(2), TilePos::new(4, 3).center());
    assert!(
        t.events()
            .iter()
            .all(|(_, e)| !matches!(e, SimEvent::MoveFailed { .. }))
    );
}

/// The path searches (A\* and fields) of every tick so far.
fn searches(t: &mut TestSim, ticks: u32) -> u32 {
    let mut searches = 0;
    for _ in 0..ticks {
        t.step();
        let counters = t.sim().nav_counters();
        searches += counters.searches + counters.fields;
    }
    searches
}

/// `docs/movement.md > 5.6`: when what its owner knows of the ground changes, a unit checks line of sight along its
/// remaining path on its staggered tick. An object off the path costs no search; one on it costs exactly one, once
/// the owner has seen it (`systems/controls/behaviors.md > PathKnowledge`).
#[test]
fn an_object_spawned_on_the_path_forces_exactly_one_repath() {
    let station = |tile: TilePos| DebugCommand::Spawn {
        kind: ObjectKind::SupplyDeliveryStation,
        owner: None,
        at: SpawnAt::Tile(tile),
        orientation: Default::default(),
    };
    let mut t = ScenarioBuilder::new(OPEN).start();
    t.without_auto_targeting();
    let goal = v(928.0, 224.0);
    t.spawn_unit_at(1, PK, P1, v(96.0, 224.0))
        .order_at(2, P1, &[ObjectId(1)], move_to(goal), false)
        .debug_at(10, station(TilePos::new(8, 8))) // far south of the route
        .debug_at(30, station(TilePos::new(8, 3))); // across it: x 512..640, y 192..320
    assert_eq!(searches(&mut t, 9), 1, "the order's path");
    assert_eq!(searches(&mut t, 20), 0, "the first station is off the path");
    // The true grids follow at once; the unit's owner knows of a station only once the unit sees it, 5 GU away.
    let known = |t: &TestSim| t.sim().path_knowledge().version(P1).unwrap();
    assert_eq!((t.sim().nav_version(), known(&t), t.mover(1).nav_version), (1, 0, 0));
    assert_eq!(t.mover(1).path.len(), 2);
    assert_eq!(searches(&mut t, 1), 0);
    assert_eq!(
        (t.sim().nav_version(), known(&t)),
        (2, 0),
        "the second one, across the path"
    );
    // The one across the path comes into sight first, straight ahead.
    while known(&t) == 0 {
        assert!(t.sim().tick() < 200);
        assert_eq!(searches(&mut t, 1), 0);
    }
    assert_eq!(t.mover(1).path.len(), 2, "not replanned before its next check");
    assert_eq!(searches(&mut t, 4), 1, "it cuts the path");
    assert_eq!(t.mover(1).nav_version, 1);
    assert!(t.mover(1).path.len() > 2);
    assert_eq!(t.mover(1).repaths_left, MAX_REPATHS, "not counted against the cap");
    let mut rest = 0;
    while t.status(1) == MoveStatus::Moving {
        assert!(t.sim().tick() < 400);
        rest += searches(&mut t, 1);
    }
    assert_eq!(rest, 0);
    assert_eq!(known(&t), 1, "the station to the south never came into sight");
    assert_eq!((t.pos(1), t.status(1)), (goal, MoveStatus::Arrived));
}

/// `systems/controls/commands.md > Unit Command`: Shift-queued waypoints are visited in order, also when other
/// units cross the route and stand on it.
#[test]
fn queued_waypoints_are_visited_in_order_under_collision() {
    let mut t = ScenarioBuilder::new(OPEN).start();
    t.without_auto_targeting();
    let stops = [v(500.0, 100.0), v(500.0, 500.0), v(120.0, 500.0)];
    t.spawn_unit_at(1, PK, P1, v(100.0, 100.0))
        .spawn_unit_at(1, GUARD, P2, v(300.0, 104.0)) // idle on the first leg
        .spawn_unit_at(1, PK, P2, v(500.0, 300.0)) // idle on the second
        .spawn_unit_at(1, PK, P2, v(300.0, 700.0)); // crosses the third
    for (i, stop) in stops.into_iter().enumerate() {
        t.order_at(2, P1, &[ObjectId(1)], move_to(stop), i > 0);
    }
    t.order_at(150, P2, &[ObjectId(4)], move_to(v(300.0, 300.0)), false);
    let mut reached = Vec::new();
    for _ in 0..400 {
        t.step();
        if let Some(i) = stops.iter().position(|s| *s == t.pos(1))
            && reached.last() != Some(&i)
        {
            reached.push(i);
        }
    }
    assert_eq!(reached, [0, 1, 2], "{}", t.ascii(None));
    assert_eq!((t.pos(1), t.status(1)), (stops[2], MoveStatus::Arrived));
    assert_eq!((t.pos(2), t.pos(3)), (v(300.0, 104.0), v(500.0, 300.0)));
}

/// `docs/movement.md > 5.8`: the deterministic counters of a group move. One field paths the whole group in the
/// tick of the order, and a unit's tick costs a bounded number of sweeps.
#[test]
fn a_group_move_costs_one_field_and_bounded_sweeps() {
    let mut t = ScenarioBuilder::new(OPEN).start();
    t.without_auto_targeting();
    let mut n = 0;
    for col in 0..6 {
        for row in 0..5 {
            t.spawn_unit_at(1, PK, P1, v(60.0 + 36.0 * col as f32, 60.0 + 36.0 * row as f32));
            n += 1;
        }
    }
    t.order_at(2, P1, &ids(1..=n), move_to(v(800.0, 600.0)), false);
    t.run(2);
    let nav = t.sim().nav_counters();
    assert_eq!((nav.fields, nav.searches), (1, 0));
    assert!(nav.expansions <= 16 * 12);
    while (1..=n).any(|id| t.status(id) == MoveStatus::Moving) {
        assert!(t.sim().tick() < 500, "{}", t.ascii(None));
        let moves = t.sim().move_counters();
        assert!(moves.steered <= n);
        // The move itself, a glance, two sidesteps on each side, and the closing-up: at most 8 sweeps of each
        // kind per unit, each against the few units of the cells around it.
        assert!(moves.unit_sweeps <= 8 * moves.steered, "{moves:?}");
        assert!(moves.tile_sweeps <= 8 * moves.steered, "{moves:?}");
        assert!(moves.pair_tests <= 24 * moves.unit_sweeps, "{moves:?}");
        t.step();
    }
    t.step();
    assert_eq!(t.sim().move_counters().steered, 0);
}
