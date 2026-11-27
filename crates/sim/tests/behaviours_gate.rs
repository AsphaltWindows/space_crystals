//! The command gate's rule for object targets (`core/vision.md > Explored > Targeting remembered objects`, design
//! change 0139, which answers question Q0017 point 2): an Attack or a Move on an object is taken while the issuing
//! player sees the object, or remembers it: a structure or a resource it has seen, by its last-known record, on
//! Explored ground. Units aren't remembered, so one out of sight is no target; nor is anything the player never saw.
//! (What the orders on a remembered object do, and that nothing about the real state leaks: `remembered_targets.rs`.)

use sc_sim::testing::{ScenarioBuilder, TestSim};
const FOG_DELAY_TICKS: sc_sim::Tick = sc_sim::tuning::vision::BUILTIN.fog_delay_ticks;
use sc_sim::{
    Command, DebugCommand, Faction, ObjectId, ObjectKind, OrderKind, PlayerCommand, PlayerId, Reject, SpawnAt, Target,
    TickInputs, TilePos, Vec2, VisibilityState,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const PK: ObjectKind = ObjectKind::Peacekeeper;

/// 40 × 12 open tiles.
fn open() -> String {
    "........................................\n".repeat(12)
}

fn v(x: f32, y: f32) -> Vec2 {
    Vec2::new(x, y)
}

fn id(n: u32) -> ObjectId {
    ObjectId(n)
}

fn order(player: PlayerId, unit: u32, kind: OrderKind, queue: bool) -> PlayerCommand {
    PlayerCommand {
        player,
        command: Command::Order {
            units: vec![id(unit)],
            kind,
            queue,
        },
    }
}

fn attack(n: u32) -> OrderKind {
    OrderKind::Attack(id(n))
}

fn move_on(n: u32) -> OrderKind {
    OrderKind::Move(Target::Object(id(n)))
}

/// Steps `t` with `cmd` and checks the gate's verdict; a rejected command changes nothing (compared with a copy that
/// ran the tick without it).
#[track_caller]
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

fn state(t: &TestSim, player: PlayerId, n: u32) -> VisibilityState {
    t.sim().vision().object_state(player, t.sim().object(id(n)).unwrap())
}

/// The brief's "the gate rejecting unseen targets": an enemy out of sight can't be attacked or followed, plainly or
/// queued; one in sight can; the player's own units always can.
#[test]
fn orders_on_units_out_of_sight_are_rejected() {
    let mut t = ScenarioBuilder::new(&open()).start();
    t.without_auto_targeting()
        .spawn_unit_at(1, PK, P1, v(200.0, 300.0)) // 1
        .spawn_unit_at(1, PK, P2, v(450.0, 300.0)) // 2, in sight of 1
        .spawn_unit_at(1, PK, P2, v(1500.0, 300.0)) // 3, out of sight
        .spawn_unit_at(1, PK, P1, v(200.0, 600.0)) // 4
        .run(1);
    assert_eq!(state(&t, P1, 2), VisibilityState::Visible);
    assert_eq!(state(&t, P1, 3), VisibilityState::Unexplored);
    for queue in [false, true] {
        for kind in [attack(3), move_on(3)] {
            apply(&mut t, order(P1, 1, kind, queue), Err(Reject::NotVisible(id(3))));
        }
        for kind in [attack(2), move_on(2), move_on(4), attack(4)] {
            apply(&mut t, order(P1, 1, kind, queue), Ok(()));
        }
    }
    // Each player has its own sight: player 2's far unit doesn't see what its near one does.
    apply(&mut t, order(P2, 3, attack(1), false), Ok(()));
    apply(&mut t, order(P2, 3, attack(4), false), Err(Reject::NotVisible(id(4))));
    // The other rules come first.
    apply(
        &mut t,
        order(P1, 1, attack(99), false),
        Err(Reject::UnknownObject(id(99))),
    );
    apply(&mut t, order(P1, 2, attack(3), false), Err(Reject::NotOwned(id(2))));
}

/// An enemy that was seen and isn't any more (its tiles are Explored) is out of sight like any other: the order is
/// rejected, where an order given while it was in sight was taken. For the fog delay after the player's sight leaves
/// it (design change 0178) it is still Visible, and a target.
#[test]
fn a_unit_seen_before_is_no_target_once_it_is_out_of_sight() {
    let mut t = ScenarioBuilder::new(&open()).start();
    t.without_auto_targeting()
        .spawn_unit_at(1, PK, P1, v(200.0, 300.0))
        .spawn_unit_at(1, PK, P2, v(450.0, 300.0))
        .run(1);
    apply(&mut t, order(P1, 1, OrderKind::Stop, false), Ok(()));
    let away = DebugCommand::SetPosition {
        id: id(1),
        pos: v(200.0, 700.0),
    };
    t.debug_at(t.sim().tick() + 1, away).step();
    for _ in 0..FOG_DELAY_TICKS {
        assert_eq!(state(&t, P1, 2), VisibilityState::Visible, "tick {}", t.sim().tick());
        assert_eq!(t.sim().validate(&order(P1, 1, attack(2), false)), Ok(()));
        t.step();
    }
    assert_eq!(state(&t, P1, 2), VisibilityState::Explored);
    apply(&mut t, order(P1, 1, attack(2), false), Err(Reject::NotVisible(id(2))));
    apply(&mut t, order(P1, 1, move_on(2), true), Err(Reject::NotVisible(id(2))));
}

/// Design change 0139: a Supply Delivery Station and a crystal patch on Explored ground, both seen before, can be
/// targeted by anyone who remembers them, the GDO player and the Syndicate one alike (the GDO exception of `>
/// Explored Selectability` is about selecting it); neither on Unexplored ground. In sight, anyone may move to either.
/// The units walk back to what they remember, and their orders complete there.
#[test]
fn remembered_objects_on_explored_ground_are_targets() {
    let mut scenario = ScenarioBuilder::new(&open()).build();
    scenario.setup.players[1].faction = Faction::Syndicate;
    let mut t = TestSim::new(scenario);
    let tiles = |kind, x, y| DebugCommand::Spawn {
        kind,
        owner: None,
        at: SpawnAt::Tile(TilePos::new(x, y)),
        orientation: Default::default(),
    };
    t.without_auto_targeting()
        .debug_at(1, tiles(ObjectKind::SupplyDeliveryStation, 5, 4)) // 1: near the units
        .debug_at(1, tiles(ObjectKind::CrystalPatch, 5, 7)) // 2: near the units
        .debug_at(1, tiles(ObjectKind::SupplyDeliveryStation, 30, 4)) // 3: never seen
        .spawn_unit_at(1, PK, P1, v(200.0, 350.0)) // 4, GDO
        .spawn_unit_at(1, ObjectKind::Guard, P2, v(200.0, 420.0)) // 5, Syndicate
        .run(1);
    assert_eq!(t.sim().player(P1).unwrap().faction, Faction::Gdo);
    assert_eq!(t.sim().player(P2).unwrap().faction, Faction::Syndicate);
    for (player, unit) in [(P1, 4), (P2, 5)] {
        assert_eq!(state(&t, player, 1), VisibilityState::Visible);
        assert_eq!(state(&t, player, 3), VisibilityState::Unexplored);
        apply(&mut t, order(player, unit, move_on(1), true), Ok(()));
        apply(&mut t, order(player, unit, move_on(2), true), Ok(()));
        apply(
            &mut t,
            order(player, unit, move_on(3), false),
            Err(Reject::NotVisible(id(3))),
        );
        // Indestructible, whoever sees it.
        apply(
            &mut t,
            order(player, unit, attack(1), false),
            Err(Reject::NotAttackable(id(1))),
        );
    }
    // Both walk away, far out of sight: the station and the patch are on Explored ground once the fog delay is over.
    let tick = t.sim().tick() + 1;
    for (unit, x) in [(4, 1100.0), (5, 1200.0)] {
        let pos = v(x, 600.0);
        t.debug_at(tick, DebugCommand::SetPosition { id: id(unit), pos })
            .order_at(tick, PlayerId(unit as u8 - 3), &[id(unit)], OrderKind::Stop, false);
    }
    t.step().run(FOG_DELAY_TICKS);
    for player in [P1, P2] {
        assert_eq!(state(&t, player, 1), VisibilityState::Explored);
        assert_eq!(state(&t, player, 2), VisibilityState::Explored);
    }
    apply(&mut t, order(P2, 5, move_on(1), false), Ok(()));
    apply(&mut t, order(P1, 4, move_on(2), false), Ok(()));
    apply(&mut t, order(P1, 4, move_on(3), false), Err(Reject::NotVisible(id(3))));
    apply(&mut t, order(P1, 4, move_on(1), false), Ok(()));
    apply(&mut t, order(P2, 5, move_on(2), false), Ok(()));
    // The GDO unit walks back to the station it remembers, the Syndicate one to the patch, and their orders complete
    // there.
    t.run(500);
    for (unit, target) in [(4, 1), (5, 2)] {
        let unit = t.sim().object(id(unit)).unwrap();
        assert!(unit.orders.as_ref().unwrap().is_idle(), "#{}", unit.id.0);
        let target = sc_sim::Shape::of(t.sim().object(id(target)).unwrap());
        assert!(
            sc_sim::Shape::of(unit).gap(&target) <= sc_sim::tuning::orders::BUILTIN.proximity_su,
            "#{}",
            unit.id.0
        );
    }
}

/// Design change 0139 goes by what the player knows, its last-known record of the station, not by the live station's
/// tiles. A station debug-spawned on tiles the player explored earlier lies on Explored ground, but the player never
/// saw it: it has no record, so it is no target until the player sees it. Once seen, it stays a target out of
/// sight.
#[test]
fn a_station_the_gdo_player_never_saw_is_no_target_on_explored_ground() {
    let mut t = ScenarioBuilder::new(&open()).start();
    let far = v(2000.0, 350.0);
    t.without_auto_targeting()
        .spawn_unit_at(1, PK, P1, v(200.0, 350.0)) // 1, GDO
        .spawn_unit_at(1, ObjectKind::Guard, P2, v(2000.0, 600.0)) // 2, far away
        .run(1);
    t.debug_at(2, DebugCommand::SetPosition { id: id(1), pos: far })
        .step()
        .run(FOG_DELAY_TICKS);
    let station = DebugCommand::Spawn {
        kind: ObjectKind::SupplyDeliveryStation,
        owner: None,
        at: SpawnAt::Tile(TilePos::new(3, 4)),
        orientation: Default::default(),
    };
    // 3, on the tiles unit 1 explored at tick 1, once the fog delay is over.
    t.debug_at(t.sim().tick() + 1, station).step();
    assert_eq!(t.sim().object(id(3)).unwrap().kind, ObjectKind::SupplyDeliveryStation);
    assert_eq!(state(&t, P1, 3), VisibilityState::Explored);
    let vision = |t: &TestSim| t.sim().vision().player(P1).unwrap().record(id(3)).cloned();
    assert_eq!(vision(&t), None, "placed on Explored tiles: unknown to the player");
    for queue in [false, true] {
        apply(&mut t, order(P1, 1, move_on(3), queue), Err(Reject::NotVisible(id(3))));
    }

    // The unit comes back and sees it: now the player knows it, and it stays a target once out of sight again.
    let near = DebugCommand::SetPosition {
        id: id(1),
        pos: v(200.0, 600.0),
    };
    t.debug_at(t.sim().tick() + 1, near).step();
    assert_eq!(state(&t, P1, 3), VisibilityState::Visible);
    assert!(vision(&t).is_some());
    apply(&mut t, order(P1, 1, move_on(3), false), Ok(()));
    let tick = t.sim().tick() + 1;
    t.debug_at(tick, DebugCommand::SetPosition { id: id(1), pos: far })
        .order_at(tick, P1, &[id(1)], OrderKind::Stop, false)
        .step()
        .run(FOG_DELAY_TICKS);
    assert_eq!(state(&t, P1, 3), VisibilityState::Explored);
    assert_eq!(vision(&t).map(|r| r.kind), Some(ObjectKind::SupplyDeliveryStation));
    apply(&mut t, order(P1, 1, move_on(3), false), Ok(()));
    // Player 2 never saw it: it has no record of it.
    apply(&mut t, order(P2, 2, move_on(3), false), Err(Reject::NotVisible(id(3))));
}
