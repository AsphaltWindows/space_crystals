//! Unit moves (`systems/controls/behaviors.md > MovingToLocation`, `docs/movement.md > 4–6`): a unit follows its
//! smoothed path at its top speed, arrives exactly, goes around walls and cliffs, plays queued orders back to back,
//! spreads a group over its slots, replans when the ground changes, and stays deterministic across a save and
//! between runs. Units in each other's way are in `movement_collision.rs`. Every `TestSim` checks the invariants
//! I1–I7 after each tick (`docs/movement.md > 8`).

use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{
    DebugCommand, MoveStatus, Mover, ObjectId, ObjectKind, OrderKind, PlayerId, Sim, SpawnAt, Target, TickInputs,
    TilePos, Vec2, nav,
};

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

fn pos(t: &TestSim, id: u32) -> Vec2 {
    t.sim().object(ObjectId(id)).expect("exists").pos
}

fn mover(t: &TestSim, id: u32) -> Mover {
    t.sim()
        .object(ObjectId(id))
        .expect("exists")
        .mover
        .clone()
        .expect("a unit")
}

/// `docs/movement.md > 3`, I2: every unit's box is clear of blocked tiles on its grid.
#[track_caller]
fn assert_on_open_ground(sim: &Sim) {
    for o in sim.objects() {
        let def = o.kind.def();
        let (Some(unit), Some(silhouette)) = (&def.unit, def.silhouette()) else {
            continue;
        };
        let class = unit.base_def().nav_class().expect("a ground unit");
        let grid = sim.nav().grid(class, nav::size_class(silhouette)).expect("built");
        assert!(
            nav::box_clear(grid, silhouette / 2.0, o.pos),
            "tick {}: #{} at {} is on blocked ground",
            sim.tick(),
            o.id.0,
            o.pos
        );
    }
}

/// Steps until unit `id` arrives, checking I2 every tick; the tick it arrives in.
fn run_to_arrival(t: &mut TestSim, id: u32, max_ticks: u32) -> u32 {
    for _ in 0..max_ticks {
        t.step();
        assert_on_open_ground(t.sim());
        if mover(t, id).status == MoveStatus::Arrived {
            return t.sim().tick();
        }
    }
    panic!("#{id} didn't arrive within {max_ticks} ticks: {:?}", mover(t, id));
}

/// `docs/movement.md > 6.1`: a unit spawned in tick 1 and ordered in tick 2 moves in ticks 2, 3, …, at its top
/// speed in a straight line, and stops exactly on the goal in tick 1 + ⌈d / speed⌉, facing the way it went.
#[test]
fn a_straight_move_takes_ceil_distance_over_speed_ticks() {
    let mut t = ScenarioBuilder::new(OPEN).start();
    let (pk_goal, guard_goal) = (v(500.0, 100.0), v(400.0, 533.0));
    t.spawn_unit_at(1, PK, P1, v(100.0, 100.0))
        .spawn_unit_at(1, GUARD, P1, v(100.0, 300.0))
        .order_at(2, P1, &[ObjectId(1)], move_to(pk_goal), false)
        .order_at(2, P1, &[ObjectId(2)], move_to(guard_goal), false);
    t.run(2);
    assert_eq!(pos(&t, 1), v(104.0, 100.0), "the order acts in its own tick");
    assert_eq!(mover(&t, 1).velocity, v(4.0, 0.0));
    // 400 su at 4 su per tick: 100 steps, the last in tick 101.
    t.run_until(100);
    assert_eq!((pos(&t, 1), mover(&t, 1).status), (v(496.0, 100.0), MoveStatus::Moving));
    t.step();
    assert_eq!((pos(&t, 1), mover(&t, 1).status), (pk_goal, MoveStatus::Arrived));
    // Its order completes in the next tick's phase 5a.
    t.step();
    assert!(t.sim().object(ObjectId(1)).unwrap().orders.as_ref().unwrap().is_idle());

    // The Guard: 379.85 su at 5 su per tick, 76 steps, the last in tick 77.
    let mut t = ScenarioBuilder::new(OPEN).start();
    t.spawn_unit_at(1, GUARD, P1, v(100.0, 300.0))
        .order_at(2, P1, &[ObjectId(1)], move_to(guard_goal), false);
    t.run_until(76);
    assert_eq!(mover(&t, 1).status, MoveStatus::Moving);
    t.step();
    let guard = t.sim().object(ObjectId(1)).unwrap();
    assert_eq!(
        (guard.pos, guard.mover.as_ref().unwrap().status),
        (guard_goal, MoveStatus::Arrived)
    );
    let expected = libm::atan2f(233.0, 300.0);
    assert!((guard.facing - expected).abs() < 1e-5, "{} vs {expected}", guard.facing);
    t.run(20);
    assert_eq!(pos(&t, 1), guard_goal, "it stays");
}

/// A water wall from the top edge down to the last row: the unit goes around its south end.
#[test]
fn moves_go_around_walls() {
    let mut t = ScenarioBuilder::new(
        "
        .......~........
        .......~........
        .......~........
        .......~........
        .......~........
        .......~........
        .......~........
        ................
        ",
    )
    .start();
    let goal = v(736.0, 224.0);
    t.spawn_unit_at(1, PK, P1, v(224.0, 224.0))
        .order_at(2, P1, &[ObjectId(1)], move_to(goal), false);
    t.run(2);
    assert!(mover(&t, 1).path.len() > 2, "{:?}", mover(&t, 1).path);
    let mut passed_below = false;
    while mover(&t, 1).status != MoveStatus::Arrived {
        assert!(t.sim().tick() < 400, "no arrival: {:?}", mover(&t, 1));
        t.step();
        assert_on_open_ground(t.sim());
        let p = pos(&t, 1);
        passed_below |= (448.0..=512.0).contains(&p.x) && p.y >= 448.0 + 12.0;
    }
    assert!(passed_below);
    assert_eq!(pos(&t, 1), goal);
}

/// A cliff plateau one level up with a one-tile ramp on its west side, tile (4, 5). A unit east of it climbs to
/// the top through the ramp.
#[test]
fn moves_climb_the_plateau_through_its_entrance() {
    let mut t = ScenarioBuilder::new(
        "
        ................
        ................
        ....cccccccc....
        ....c......c....
        ....c......c....
        ...........c....
        ....c......c....
        ....cccccccc....
        ................
        ................

        00000000000000000
        00000000000000000
        00000000000000000
        00000111111100000
        00000111111100000
        00000111111100000
        00000111111100000
        00000111111100000
        00000000000000000
        00000000000000000
        00000000000000000
        ",
    )
    .start();
    let goal = v(480.0, 288.0); // tile (7, 4), on top
    t.spawn_unit_at(1, PK, P1, v(864.0, 352.0)) // tile (13, 5), east of the plateau
        .order_at(2, P1, &[ObjectId(1)], move_to(goal), false)
        .run(1);
    let mut through_ramp = false;
    while mover(&t, 1).status != MoveStatus::Arrived {
        assert!(t.sim().tick() < 600, "no arrival: {:?}", mover(&t, 1));
        t.step();
        assert_on_open_ground(t.sim());
        through_ramp |= TilePos::containing(pos(&t, 1)) == TilePos::new(4, 5);
    }
    assert!(through_ramp);
    assert_eq!(pos(&t, 1), goal);
    assert_eq!(t.sim().map().height_at(goal), 1.0);
}

/// `systems/controls/commands.md > Unit Command`: Shift-queued moves are visited in order, each leg starting in the
/// tick after the last one ended.
#[test]
fn queued_moves_are_visited_in_order() {
    let stops = [v(400.0, 100.0), v(400.0, 500.0), v(100.0, 500.0)];
    let mut t = ScenarioBuilder::new(OPEN).start();
    t.spawn_unit_at(1, PK, P1, v(100.0, 100.0))
        .order_at(2, P1, &[ObjectId(1)], move_to(stops[0]), false)
        .order_at(2, P1, &[ObjectId(1)], move_to(stops[1]), true)
        .order_at(3, P1, &[ObjectId(1)], move_to(stops[2]), true);
    t.run(1);
    let mut reached = Vec::new();
    let mut previous = pos(&t, 1);
    for _ in 0..400 {
        t.step();
        let p = pos(&t, 1);
        if let Some(i) = stops.iter().position(|s| *s == p)
            && reached.last() != Some(&i)
        {
            reached.push(i);
        }
        if reached.len() == 3 {
            break;
        }
        assert_ne!(p, previous, "tick {}: no idle tick between legs", t.sim().tick());
        previous = p;
    }
    assert_eq!(reached, [0, 1, 2]);
    // 300 + 400 + 300 su at 4 su per tick, from tick 2.
    assert_eq!(t.sim().tick(), 1 + 250);
}

/// `docs/movement.md > 6.4`: units ordered together get distinct slots around the click, apart.
#[test]
fn a_group_spreads_over_distinct_slots() {
    let mut t = ScenarioBuilder::new(OPEN).start();
    let starts = [v(100.0, 100.0), v(140.0, 100.0), v(100.0, 140.0), v(140.0, 140.0)];
    for p in starts {
        t.spawn_unit_at(1, PK, P1, p);
    }
    let click = v(600.0, 400.0);
    let ids: Vec<ObjectId> = (1..=4).map(ObjectId).collect();
    t.order_at(2, P1, &ids, move_to(click), false);
    t.run(2);
    let slots: Vec<Option<Vec2>> = ids
        .iter()
        .map(|id| {
            let orders = t.sim().object(*id).unwrap().orders.as_ref().unwrap();
            orders.current.as_ref().unwrap().order.slot
        })
        .collect();
    assert!(slots.iter().all(Option::is_some), "{slots:?}");
    t.run(200);
    let ends: Vec<Vec2> = (1..=4).map(|id| pos(&t, id)).collect();
    for (i, a) in ends.iter().enumerate() {
        assert_eq!(mover(&t, i as u32 + 1).status, MoveStatus::Arrived);
        assert_eq!(Some(*a), slots[i]);
        // Slot pitch: 24 su of box plus an 8 su gap; the four nearest lattice points.
        assert!(a.distance(click) <= 32.0 + 1e-3, "{a}");
        for b in &ends[i + 1..] {
            assert!(
                (a.x - b.x).abs() >= 24.0 || (a.y - b.y).abs() >= 24.0,
                "{a} and {b} overlap"
            );
        }
    }
}

/// `docs/movement.md > 5.4`: a goal on blocked ground is moved to the nearest reachable tile; the unit arrives there.
#[test]
fn an_unreachable_goal_is_retargeted() {
    let mut t = ScenarioBuilder::new(
        "
        ................
        ................
        ................
        ..........~~~...
        ..........~~~...
        ..........~~~...
        ................
        ................
        ",
    )
    .start();
    let click = v(736.0, 288.0); // the middle of the pond, tile (11, 4)
    t.spawn_unit_at(1, PK, P1, v(100.0, 288.0))
        .order_at(2, P1, &[ObjectId(1)], move_to(click), false);
    run_to_arrival(&mut t, 1, 400);
    // The center of a nearest open tile, two tiles from the pond's middle.
    let end = pos(&t, 1);
    assert_eq!(end.distance(click), 2.0 * 64.0, "{end}");
    assert_eq!(end, TilePos::containing(end).center());
}

/// `docs/movement.md > 5.7`, `> 5.6`: a station spawned across the route rebuilds the grids (a new `nav_version`);
/// on its next staggered check the unit finds its path cut, replans and goes around it.
#[test]
fn a_new_footprint_on_the_route_makes_the_unit_replan() {
    let mut t = ScenarioBuilder::new(OPEN).start();
    let goal = v(928.0, 224.0);
    t.spawn_unit_at(1, PK, P1, v(96.0, 224.0))
        .order_at(2, P1, &[ObjectId(1)], move_to(goal), false)
        .debug_at(
            10,
            DebugCommand::Spawn {
                kind: ObjectKind::SupplyDeliveryStation,
                owner: None,
                at: SpawnAt::Tile(TilePos::new(8, 3)), // x 512..640, y 192..320
            },
        );
    t.run(9);
    assert_eq!((t.sim().nav_version(), mover(&t, 1).path.len()), (0, 2));
    t.step();
    assert_eq!(t.sim().objects().count(), 2, "the station");
    assert_eq!(t.sim().nav_version(), 1);
    // Unit 1 checks in the ticks where `(tick + 1) % 4 == 0`: tick 11.
    assert_eq!((mover(&t, 1).nav_version, mover(&t, 1).path.len()), (0, 2));
    t.step();
    let replanned = mover(&t, 1);
    assert_eq!(replanned.nav_version, 1);
    assert!(replanned.path.len() > 2, "{:?}", replanned.path);
    run_to_arrival(&mut t, 1, 400);
    assert_eq!(pos(&t, 1), goal);
}

/// Units on the move, in a group and with queued orders: the schedule of the save and determinism tests.
fn busy() -> TestSim {
    let mut t = ScenarioBuilder::new(OPEN).seed(7).start();
    for (i, p) in [v(100.0, 100.0), v(140.0, 100.0), v(100.0, 140.0)]
        .into_iter()
        .enumerate()
    {
        t.spawn_unit_at(1, if i == 0 { GUARD } else { PK }, P1, p);
    }
    t.spawn_unit_at(1, PK, PlayerId(2), v(900.0, 700.0));
    let group = [ObjectId(1), ObjectId(2), ObjectId(3)];
    t.order_at(2, P1, &group, move_to(v(700.0, 600.0)), false)
        .order_at(3, P1, &group[..1], move_to(v(200.0, 650.0)), true)
        .order_at(5, PlayerId(2), &[ObjectId(4)], move_to(v(150.0, 150.0)), false)
        .order_at(40, PlayerId(2), &[ObjectId(4)], OrderKind::Stop, false)
        .order_at(45, PlayerId(2), &[ObjectId(4)], move_to(v(500.0, 50.0)), true);
    t
}

/// A save mid-move (JSON round trip) continues exactly like the game it came from.
#[test]
fn a_save_mid_move_continues_identically() {
    let mut t = busy();
    t.run(30);
    assert!((1..=4).all(|id| mover(&t, id).status == MoveStatus::Moving));
    let json = serde_json::to_string(t.sim()).unwrap();
    let mut loaded: Sim = serde_json::from_str(&json).unwrap();
    assert_eq!(&loaded, t.sim());
    assert_eq!(loaded.state_hash(), t.sim().state_hash());
    for _ in 0..300 {
        let tick = t.sim().tick() + 1;
        t.step();
        let inputs: &TickInputs = &t.replay().inputs[tick as usize - 1];
        loaded.step(inputs);
        assert_eq!(loaded.state_hash(), t.sim().state_hash(), "tick {tick}");
    }
    assert_eq!(&loaded, t.sim());
    assert!((1..=4).all(|id| mover(&t, id).status == MoveStatus::Arrived));
}

/// The same schedule twice gives the same state hash every tick, and the replay ends in the same state.
#[test]
fn two_runs_give_equal_hashes_every_tick() {
    let (mut a, mut b) = (busy(), busy());
    for _ in 0..300 {
        a.step();
        b.step();
        assert_eq!(a.sim().state_hash(), b.sim().state_hash(), "tick {}", a.sim().tick());
    }
    assert_eq!(a.replay().run().state_hash(), a.sim().state_hash());
}
