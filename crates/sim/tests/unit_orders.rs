//! Unit orders (`systems/controls/commands.md > Unit Command`, `docs/movement.md > 3`): the order gate, the queue
//! rules, Stop and Hold, and the spawn and teleport checks that keep units apart and off blocked ground.

use sc_sim::orders::MAX_ORDER_UNITS;
use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{
    Command, DebugCommand, MoveStatus, ObjectId, ObjectKind, OrderKind, PlayerCommand, PlayerId, Reject, SimEvent,
    SpawnAt, Target, TickInputs, TilePos, Vec2,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

/// 12 × 8 tiles (768 × 512 su), with a pond on tiles (6..=7, 2..=3).
const MAP: &str = "
    ............
    ............
    ......~~....
    ......~~....
    ............
    ............
    ............
    ............
";

fn v(x: f32, y: f32) -> Vec2 {
    Vec2::new(x, y)
}

fn move_to(p: Vec2) -> OrderKind {
    OrderKind::Move(Target::Point(p))
}

fn order(player: PlayerId, units: &[u32], kind: OrderKind, queue: bool) -> PlayerCommand {
    PlayerCommand {
        player,
        command: Command::Order {
            units: units.iter().map(|&i| ObjectId(i)).collect(),
            kind,
            queue,
        },
    }
}

fn debug(command: DebugCommand) -> PlayerCommand {
    PlayerCommand {
        player: P1,
        command: Command::Debug(command),
    }
}

fn spawn(kind: ObjectKind, owner: Option<PlayerId>, p: Vec2) -> PlayerCommand {
    debug(DebugCommand::Spawn {
        kind,
        owner,
        at: SpawnAt::Point(p),
    })
}

fn pos(t: &TestSim, id: u32) -> Vec2 {
    t.sim().object(ObjectId(id)).expect("exists").pos
}

/// Units 1 (player 1, at (100, 100)) and 2 (player 2, at (100, 300)), Peacekeepers, spawned in tick 1, plus a
/// crystal patch (id 3) on tile (10, 6).
fn two_units() -> TestSim {
    let mut t = ScenarioBuilder::new(MAP).start();
    t.spawn_unit_at(1, ObjectKind::Peacekeeper, P1, v(100.0, 100.0))
        .spawn_unit_at(1, ObjectKind::Peacekeeper, P2, v(100.0, 300.0))
        .debug_at(
            1,
            DebugCommand::Spawn {
                kind: ObjectKind::CrystalPatch,
                owner: None,
                at: SpawnAt::Tile(TilePos::new(10, 6)),
            },
        )
        .run(1);
    assert_eq!(t.sim().objects().count(), 3);
    t
}

/// Steps `t` with `cmd` and checks that the gate's verdict was `expected` and, for a rejection, that the command
/// changed nothing (compared with a copy that ran the tick without it).
fn apply(t: &mut TestSim, cmd: PlayerCommand, expected: Result<(), Reject>) {
    assert_eq!(t.sim().validate(&cmd), expected, "{cmd:?}");
    let mut without = t.sim().clone();
    without.step(&TickInputs::default());
    let tick = t.sim().tick() + 1;
    t.at(tick, cmd.player, cmd.command.clone()).step();
    if expected.is_err() {
        assert_eq!(t.sim().state_hash(), without.state_hash(), "{cmd:?}");
        assert_eq!(t.sim(), &without, "{cmd:?}");
    }
}

#[test]
fn orders_drop_units_that_are_gone_and_repeated_ids() {
    let mut t = two_units();
    t.spawn_unit_at(2, ObjectKind::Peacekeeper, P1, v(200.0, 100.0)) // id 4
        .debug_at(3, DebugCommand::Remove { id: ObjectId(4) })
        .run(2);
    let goal = v(300.0, 420.0);
    // 4 was removed, 99 never existed: dropped. 1 is named twice: one order.
    apply(&mut t, order(P1, &[1, 4, 1, 99, 1], move_to(goal), false), Ok(()));
    let orders = t.sim().object(ObjectId(1)).unwrap().orders.as_ref().unwrap();
    let current = orders.current.as_ref().expect("started in its own tick");
    assert_eq!((current.order.kind, current.order.slot), (move_to(goal), None));
    assert!(orders.queue.is_empty());
    // With no unit left, the order is rejected.
    apply(
        &mut t,
        order(P1, &[4, 99], OrderKind::Stop, false),
        Err(Reject::NoUnits),
    );
    apply(&mut t, order(P1, &[], OrderKind::Stop, false), Err(Reject::NoUnits));
    // A crystal patch takes no orders, whoever names it.
    apply(
        &mut t,
        order(P1, &[3], OrderKind::Stop, false),
        Err(Reject::NotOwned(ObjectId(3))),
    );
}

#[test]
fn players_order_only_their_own_units() {
    let mut t = two_units();
    apply(
        &mut t,
        order(P1, &[1, 2], move_to(v(300.0, 300.0)), false),
        Err(Reject::NotOwned(ObjectId(2))),
    );
    apply(
        &mut t,
        order(P1, &[2], OrderKind::Hold, false),
        Err(Reject::NotOwned(ObjectId(2))),
    );
    apply(
        &mut t,
        order(PlayerId(7), &[1], OrderKind::Hold, false),
        Err(Reject::UnknownPlayer(PlayerId(7))),
    );
    apply(&mut t, order(P2, &[2], OrderKind::Hold, false), Ok(()));
    // Orders aren't debug commands: they work in a real match.
    let mut real = TestSim::with_config(ScenarioBuilder::new(MAP).build(), Default::default());
    real.run(1);
    assert_eq!(
        real.sim().validate(&order(P1, &[1], OrderKind::Stop, false)),
        Err(Reject::NoUnits)
    );
}

#[test]
fn an_order_names_at_most_max_order_units_ids() {
    let t = two_units();
    let ids = |n: usize| vec![1; n];
    assert_eq!(MAX_ORDER_UNITS, 1024);
    let at_most = order(P1, &ids(MAX_ORDER_UNITS), OrderKind::Stop, false);
    assert_eq!(t.sim().validate(&at_most), Ok(()));
    let too_many = order(P1, &ids(MAX_ORDER_UNITS + 1), OrderKind::Stop, false);
    assert_eq!(t.sim().validate(&too_many), Err(Reject::TooManyUnits));
}

#[test]
fn order_targets_must_exist_and_lie_on_the_map() {
    let mut t = two_units();
    for p in [
        v(f32::NAN, 10.0),
        v(10.0, f32::INFINITY),
        v(-1.0, 10.0),
        v(768.0, 10.0),
        v(10.0, 512.0),
    ] {
        for kind in [move_to(p), OrderKind::AttackMove(p), OrderKind::Patrol(p)] {
            apply(&mut t, order(P1, &[1], kind, false), Err(Reject::OffMap));
        }
    }
    let missing = ObjectId(99);
    apply(
        &mut t,
        order(P1, &[1], OrderKind::Move(Target::Object(missing)), false),
        Err(Reject::UnknownObject(missing)),
    );
    apply(
        &mut t,
        order(P1, &[1], OrderKind::Attack(missing), false),
        Err(Reject::UnknownObject(missing)),
    );
    // `systems/controls/commands.md > Attack`: a destructible object only.
    apply(
        &mut t,
        order(P1, &[1], OrderKind::Attack(ObjectId(3)), false),
        Err(Reject::NotAttackable(ObjectId(3))),
    );
    apply(&mut t, order(P1, &[1], OrderKind::Attack(ObjectId(2)), false), Ok(()));
    apply(
        &mut t,
        order(P1, &[1], OrderKind::Move(Target::Object(ObjectId(3))), false),
        Ok(()),
    );
}

/// `systems/controls/commands.md > Unit Command`: without Shift the order replaces the queue, with Shift it is
/// appended.
#[test]
fn an_order_replaces_the_queue_and_a_queued_one_waits_its_turn() {
    let (a, b, c) = (v(96.0, 352.0), v(608.0, 352.0), v(96.0, 96.0));
    for queue in [false, true] {
        let mut t = ScenarioBuilder::new(MAP).start();
        t.spawn_unit_at(1, ObjectKind::Peacekeeper, P1, a)
            .order_at(2, P1, &[ObjectId(1)], move_to(b), false)
            .order_at(10, P1, &[ObjectId(1)], move_to(c), queue);
        t.run(10);
        let mut reached_b = false;
        let mut previous = pos(&t, 1);
        if !queue {
            // The new order acts in its own tick: the unit turned north at once.
            assert!(previous.y < a.y, "{previous}");
        }
        for _ in 0..400 {
            t.step();
            reached_b |= pos(&t, 1) == b;
            previous = pos(&t, 1);
        }
        assert_eq!(previous, c, "queue: {queue}");
        assert_eq!(reached_b, queue, "queue: {queue}");
        let unit = t.sim().object(ObjectId(1)).unwrap();
        assert!(unit.orders.as_ref().unwrap().is_idle());
        assert_eq!(unit.mover.as_ref().unwrap().status, MoveStatus::Arrived);
    }
}

/// `systems/controls/behaviors.md > StoppingBehavior`: the unit stops at once (infinite deceleration).
#[test]
fn stop_halts_the_unit_where_it_is() {
    let mut t = ScenarioBuilder::new(MAP).start();
    t.spawn_unit_at(1, ObjectKind::Peacekeeper, P1, v(96.0, 352.0))
        .order_at(2, P1, &[ObjectId(1)], move_to(v(608.0, 352.0)), false)
        .order_at(10, P1, &[ObjectId(1)], OrderKind::Stop, false);
    t.run(9);
    let stopped_at = pos(&t, 1);
    assert_eq!(stopped_at, v(96.0 + 8.0 * 4.0, 352.0), "8 steps of 4 su");
    for _ in 0..50 {
        t.step();
        assert_eq!(pos(&t, 1), stopped_at);
    }
    let unit = t.sim().object(ObjectId(1)).unwrap();
    assert!(unit.orders.as_ref().unwrap().is_idle(), "Stop completed");
    assert_eq!(unit.mover.as_ref().unwrap().status, MoveStatus::Idle);
}

/// `systems/controls/behaviors.md > HoldingPosition`: never moves, never completes; orders queued behind it wait.
#[test]
fn hold_stays_put_until_a_new_order_replaces_it() {
    let start = v(96.0, 352.0);
    let mut t = ScenarioBuilder::new(MAP).start();
    t.spawn_unit_at(1, ObjectKind::Peacekeeper, P1, start)
        .order_at(2, P1, &[ObjectId(1)], OrderKind::Hold, false)
        .order_at(3, P1, &[ObjectId(1)], move_to(v(608.0, 352.0)), true);
    for _ in 0..120 {
        t.step();
        assert_eq!(pos(&t, 1), start);
    }
    let orders = t.sim().object(ObjectId(1)).unwrap().orders.clone().unwrap();
    assert_eq!(orders.current.map(|a| a.order.kind), Some(OrderKind::Hold));
    assert_eq!(orders.queue.len(), 1, "the queued move waits behind Hold");
    t.order_at(121, P1, &[ObjectId(1)], move_to(v(96.0, 96.0)), false)
        .step();
    assert_ne!(pos(&t, 1), start, "a new order ends Hold");
}

/// `docs/movement.md > 3`: spawns and teleports keep a unit's box off blocked tiles, inside the map and clear of
/// the other units of its layer; a blocking footprint can't be put on a unit. Touching is free.
#[test]
fn spawns_and_teleports_keep_units_apart_and_on_open_ground() {
    let mut t = ScenarioBuilder::new(MAP).start();
    t.spawn_unit_at(1, ObjectKind::Peacekeeper, P1, v(100.0, 100.0)).run(1);
    let sim = t.sim();
    let check = |kind, p| sim.validate(&spawn(kind, Some(P1), p));
    // Peacekeepers are 24 × 24, Guards 36 × 36.
    assert_eq!(
        check(ObjectKind::Peacekeeper, v(110.0, 100.0)),
        Err(Reject::Occupied(ObjectId(1)))
    );
    assert_eq!(check(ObjectKind::Peacekeeper, v(124.0, 100.0)), Ok(()), "touching");
    assert_eq!(
        check(ObjectKind::Guard, v(129.5, 100.0)),
        Err(Reject::Occupied(ObjectId(1)))
    );
    assert_eq!(check(ObjectKind::Guard, v(130.0, 100.0)), Ok(()), "touching");
    assert_eq!(
        check(ObjectKind::Peacekeeper, v(100.0, 123.0)),
        Err(Reject::Occupied(ObjectId(1)))
    );
    // The pond covers x 384..512, y 128..256.
    assert_eq!(check(ObjectKind::Peacekeeper, v(448.0, 192.0)), Err(Reject::Blocked));
    assert_eq!(check(ObjectKind::Peacekeeper, v(373.0, 150.0)), Err(Reject::Blocked));
    assert_eq!(check(ObjectKind::Peacekeeper, v(372.0, 150.0)), Ok(()), "touching");
    // The map's edge.
    assert_eq!(check(ObjectKind::Peacekeeper, v(5.0, 300.0)), Err(Reject::Blocked));
    assert_eq!(check(ObjectKind::Peacekeeper, v(12.0, 300.0)), Ok(()));
    assert_eq!(check(ObjectKind::Guard, v(300.0, 500.0)), Err(Reject::Blocked));
    // A crystal patch on the unit's tile, or a station over it.
    let patch = |x, y| {
        debug(DebugCommand::Spawn {
            kind: ObjectKind::CrystalPatch,
            owner: None,
            at: SpawnAt::Tile(TilePos::new(x, y)),
        })
    };
    assert_eq!(sim.validate(&patch(1, 1)), Err(Reject::Occupied(ObjectId(1))));
    assert_eq!(
        sim.validate(&patch(2, 1)),
        Ok(()),
        "the unit's box ends at x = 112, inside tile 1"
    );

    // Teleports: the same rules, except that a unit never blocks itself.
    t.spawn_unit_at(2, ObjectKind::Guard, P2, v(300.0, 400.0)).run(1);
    let teleport = |id: u32, p| {
        debug(DebugCommand::SetPosition {
            id: ObjectId(id),
            pos: p,
        })
    };
    apply(&mut t, teleport(1, v(448.0, 192.0)), Err(Reject::Blocked));
    apply(&mut t, teleport(1, v(310.0, 400.0)), Err(Reject::Occupied(ObjectId(2))));
    apply(&mut t, teleport(1, v(104.0, 100.0)), Ok(()));
    assert_eq!(pos(&t, 1), v(104.0, 100.0));

    // A patch spawned in the tick blocks a unit spawned after it in the same tick, before the grids catch up.
    let tick = t.sim().tick() + 1;
    t.at(tick, P1, patch(8, 6).command)
        .spawn_unit_at(tick, ObjectKind::Peacekeeper, P1, v(8.5 * 64.0, 6.5 * 64.0))
        .step();
    let spawned: Vec<_> = t
        .events()
        .iter()
        .filter(|(at, e)| *at == tick && matches!(e, SimEvent::Spawned(_)))
        .collect();
    assert_eq!(spawned.len(), 1, "only the patch: {spawned:?}");
}
