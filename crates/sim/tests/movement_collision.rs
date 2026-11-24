//! Steering and hard collision (`docs/movement.md > 6`, `> 8`; `systems/units.md > UnitCollision > Ground
//! Collision`): units pass, follow, go around and give up, and never overlap. Every `TestSim` checks the invariants
//! I1–I7 after each tick and fails with a picture of the map.
//!
//! The give-up numbers are the defaults of question Q0008, and what two units blocking each other for good do is
//! the default of question Q0007.

use sc_sim::mover::{GIVE_UP_TICKS, Locomotion, MAX_REPATHS};
use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{
    DebugCommand, MoveGoal, MoveStatus, ObjectId, ObjectKind, OrderKind, PlayerId, SimEvent, SpawnAt, Target, TilePos,
    Vec2,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const PK: ObjectKind = ObjectKind::Peacekeeper; // 24 × 24 su, 4 su per tick
const GUARD: ObjectKind = ObjectKind::Guard; // 36 × 36 su, 5 su per tick

/// A corridor one tile (64 su) wide and 16 long, y from 64 to 128.
const CORRIDOR: &str = "
    ~~~~~~~~~~~~~~~~
    ................
    ~~~~~~~~~~~~~~~~
";

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

fn ids(range: std::ops::RangeInclusive<u32>) -> Vec<ObjectId> {
    range.map(ObjectId).collect()
}

/// Steps until every unit of `units` has a status other than `Moving`; the tick. Panics, with the map, after
/// `max_tick`.
#[track_caller]
fn run_until_done(t: &mut TestSim, units: &[ObjectId], max_tick: u32) -> u32 {
    // The units are spawned in tick 1 and ordered in tick 2.
    t.run_until(t.sim().tick().max(2));
    while units.iter().any(|id| t.status(id.0) == MoveStatus::Moving) {
        assert!(
            t.sim().tick() < max_tick,
            "still moving at tick {max_tick}:\n{}",
            t.ascii(None)
        );
        t.step();
    }
    t.sim().tick()
}

/// `factions/cults/units.md > Recruit` (throughput note), `docs/movement.md > 6.3` rule 4: a 64 su gap fits two
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
    // They meet after (832 − 36) / (2 × 5) = 80 ticks.
    let done = run_until_done(&mut t, &ids(1..=2), 2 + 80 + GIVE_UP_TICKS + 8);
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

/// `docs/movement.md > 8`: 20 units through a one-tile choke all arrive within `2 × path / speed + c`.
#[test]
fn twenty_units_go_through_a_choke() {
    let mut t = ScenarioBuilder::new(
        "
        ..........~.........
        ..........~.........
        ..........~.........
        ..........~.........
        ....................
        ..........~.........
        ..........~.........
        ..........~.........
        ..........~.........
        ",
    )
    .start();
    // Five columns of four, west of the wall, 56 su apart. The group keeps this shape at the click (the magic box),
    // and its units fit through the gaps between those already standing there. In a tighter shape the first
    // arrivals wall off places behind them, and a unit shut out farther than 2 GU from its place gives up
    // (design changes 0026 and 0027).
    let mut n = 0;
    for col in 0..5 {
        for row in 0..4 {
            t.spawn_unit_at(1, PK, P1, v(60.0 + 56.0 * col as f32, 204.0 + 56.0 * row as f32));
            n += 1;
        }
    }
    let click = v(1056.0, 288.0);
    t.order_at(2, P1, &ids(1..=n), move_to(click), false);
    // The farthest unit walks about 1000 su: 250 ticks alone.
    let done = run_until_done(&mut t, &ids(1..=n), 2 + 2 * 250 + 60);
    for id in 1..=n {
        assert_eq!(t.status(id), MoveStatus::Arrived, "#{id}:\n{}", t.ascii(None));
        assert!(t.pos(id).x > 704.0, "#{id} is through the choke");
        assert!(t.pos(id).distance(click) < 160.0, "#{id} at {}", t.pos(id));
    }
    assert!(
        t.events()
            .iter()
            .all(|(_, e)| !matches!(e, SimEvent::MoveFailed { .. }))
    );
    println!("20 units through a choke: all arrived, done in tick {done}");
    assert!(done > 250);
}

/// `docs/movement.md > 8`: a mixed group arrives in its own shape (the magic box, `> 6.4`), every unit is
/// Stationary, and stays so for 100 ticks (nothing oscillates once it has arrived, `> 6.5`).
#[test]
fn a_group_arrives_and_stands_still() {
    let mut t = ScenarioBuilder::new(OPEN).start();
    t.without_auto_targeting();
    let mut n = 0;
    for col in 0..4 {
        for row in 0..3 {
            let kind = if (col + row) % 3 == 0 { GUARD } else { PK };
            t.spawn_unit_at(1, kind, P1, v(80.0 + 48.0 * col as f32, 80.0 + 48.0 * row as f32));
            n += 1;
        }
    }
    t.order_at(2, P1, &ids(1..=n), move_to(v(760.0, 560.0)), false);
    t.run(2);
    let goals: Vec<MoveGoal> = (1..=n).map(|id| t.mover(id).goal).collect();
    run_until_done(&mut t, &ids(1..=n), 400);
    let at: Vec<Vec2> = (1..=n).map(|id| t.pos(id)).collect();
    // The click is outside the group's box, so the units keep their places, 48 su apart: nothing walls in another's
    // destination, and each stands exactly on its own, its start moved by the box's centre (152, 128) to the click.
    for id in 1..=n {
        assert_eq!(t.status(id), MoveStatus::Arrived);
        let (col, row) = ((id - 1) / 3, (id - 1) % 3);
        let start = v(80.0 + 48.0 * col as f32, 80.0 + 48.0 * row as f32);
        let destination = start + v(760.0, 560.0) - v(152.0, 128.0);
        assert_eq!(goals[id as usize - 1], MoveGoal::Point(destination));
        assert_eq!(t.pos(id), destination, "#{id}");
    }
    for _ in 0..100 {
        t.step();
        for id in 1..=n {
            assert_eq!(t.pos(id), at[id as usize - 1]);
            assert_eq!(t.mover(id).locomotion, Locomotion::Stationary);
            assert_eq!(t.mover(id).velocity, Vec2::ZERO);
        }
    }
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

/// `docs/movement.md > 5.6`: when the ground changes, a unit checks line of sight along its remaining path on its
/// staggered tick. An object off the path costs no search; one on it costs exactly one.
#[test]
fn an_object_spawned_on_the_path_forces_exactly_one_repath() {
    let station = |tile: TilePos| DebugCommand::Spawn {
        kind: ObjectKind::SupplyDeliveryStation,
        owner: None,
        at: SpawnAt::Tile(tile),
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
    assert_eq!((t.sim().nav_version(), t.mover(1).nav_version), (1, 1));
    assert_eq!(t.mover(1).path.len(), 2);
    assert_eq!(searches(&mut t, 8), 1, "the second one cuts it");
    assert_eq!((t.sim().nav_version(), t.mover(1).nav_version), (2, 2));
    assert!(t.mover(1).path.len() > 2);
    assert_eq!(t.mover(1).repaths_left, MAX_REPATHS, "not counted against the cap");
    let mut rest = 0;
    while t.status(1) == MoveStatus::Moving {
        assert!(t.sim().tick() < 400);
        rest += searches(&mut t, 1);
    }
    assert_eq!(rest, 0);
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

/// `docs/movement.md > 8`, oscillation: on the way through a crowd, a unit travels at most `1.5 × path + c`, and
/// its velocity turns against its path only a few times.
#[test]
fn units_dont_oscillate() {
    let mut t = ScenarioBuilder::new(OPEN).start();
    t.without_auto_targeting();
    // Two groups of eight cross each other's way, around a few idle units in the middle.
    let mut n = 0;
    for i in 0..8 {
        let (col, row) = ((i % 2) as f32, (i / 2) as f32);
        t.spawn_unit_at(1, PK, P1, v(80.0 + 40.0 * col, 300.0 + 40.0 * row));
        n += 1;
    }
    for i in 0..8 {
        let (col, row) = ((i % 2) as f32, (i / 2) as f32);
        t.spawn_unit_at(1, PK, P2, v(940.0 - 40.0 * col, 300.0 + 40.0 * row));
        n += 1;
    }
    for p in [v(500.0, 270.0), v(520.0, 390.0), v(480.0, 510.0)] {
        t.spawn_unit_at(1, GUARD, P1, p);
    }
    t.order_at(2, P1, &ids(1..=8), move_to(v(900.0, 360.0)), false)
        .order_at(2, P2, &ids(9..=16), move_to(v(120.0, 360.0)), false);
    t.run(2);
    let start: Vec<Vec2> = (1..=n)
        .map(|id| t.sim().object(ObjectId(id)).unwrap().prev_pos)
        .collect();
    let goals: Vec<Vec2> = (1..=n)
        .map(|id| match t.mover(id).goal {
            MoveGoal::Point(p) => p,
            other => panic!("#{id} has goal {other:?}"),
        })
        .collect();
    let mut travelled = vec![0.0_f32; n as usize];
    let mut reversals = vec![0_u32; n as usize];
    let mut backwards = vec![false; n as usize];
    loop {
        for id in 1..=n {
            let i = id as usize - 1;
            let o = t.sim().object(ObjectId(id)).unwrap();
            let moved = o.pos - o.prev_pos;
            travelled[i] += moved.length();
            // Clearly backwards: more than 60° off the straight line from start to goal, the other way.
            let line = goals[i] - start[i];
            let against = moved.dot(line) < -0.5 * moved.length() * line.length();
            if against && !backwards[i] {
                reversals[i] += 1;
            }
            backwards[i] = against;
        }
        if (1..=n).all(|id| t.status(id) != MoveStatus::Moving) {
            break;
        }
        assert!(t.sim().tick() < 600, "{}", t.ascii(None));
        t.step();
    }
    for id in 1..=n {
        let i = id as usize - 1;
        assert_eq!(t.status(id), MoveStatus::Arrived, "#{id}\n{}", t.ascii(None));
        let straight = start[i].distance(goals[i]);
        assert!(
            travelled[i] <= 1.5 * straight + 64.0,
            "#{id} travelled {} for a path of {straight}",
            travelled[i]
        );
        assert!(reversals[i] <= 6, "#{id} turned back {} times", reversals[i]);
    }
}

/// `docs/movement.md > 8`: a save in the middle of a crowd (JSON round trip) continues exactly like the game it came
/// from: the spatial grids and the search scratch carry nothing across ticks.
#[test]
fn a_save_in_a_crowd_continues_identically() {
    let crowd = || {
        let mut t = ScenarioBuilder::new(CORRIDOR).seed(3).start();
        t.without_auto_targeting();
        for i in 0..4 {
            t.spawn_unit_at(1, PK, P1, v(96.0 + 60.0 * i as f32, 84.0 + 8.0 * i as f32));
            t.spawn_unit_at(
                1,
                if i == 0 { GUARD } else { PK },
                P2,
                v(928.0 - 60.0 * i as f32, 100.0),
            );
        }
        let (east, west): (Vec<ObjectId>, Vec<ObjectId>) = ids(1..=8).into_iter().partition(|id| id.0 % 2 == 1);
        t.order_at(2, P1, &east, move_to(v(900.0, 96.0)), false)
            .order_at(3, P2, &west, move_to(v(120.0, 96.0)), false);
        t
    };
    let mut t = crowd();
    for k in [60, 100, 130] {
        t.run_until(k);
        let json = serde_json::to_string(t.sim()).unwrap();
        let mut loaded: sc_sim::Sim = serde_json::from_str(&json).unwrap();
        assert_eq!(&loaded, t.sim());
        let mut twin = crowd();
        twin.run_until(k);
        for _ in 0..150 {
            twin.step();
            loaded.step(&twin.replay().inputs[twin.sim().tick() as usize - 1]);
            assert_eq!(
                loaded.state_hash(),
                twin.sim().state_hash(),
                "saved at {k}, tick {}",
                twin.sim().tick()
            );
        }
        assert_eq!(&loaded, twin.sim());
    }
    // The crowd did meet: somebody was blocked.
    t.run_until(400);
    assert!(
        (1..=8).all(|id| t.status(id) != MoveStatus::Moving),
        "{}",
        t.ascii(None)
    );
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
