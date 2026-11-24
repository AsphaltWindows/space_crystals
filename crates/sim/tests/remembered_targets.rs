//! Orders on remembered objects (`core/vision.md > Explored > Targeting remembered objects`, design change 0139,
//! which answers question Q0017 point 2): a structure the player saw, out of its sight now, is a target of a
//! right-click (its default order, `Sim::default_order`) and of Attack and Move. "The order goes to the object's
//! **last-known position**." Once the units see the place, the order carries on if the object stands, and ends "the
//! way it would for a vanished target" if it doesn't: "the unit stops there and takes its next queued order or
//! idles". "Nothing about the real state leaks": until the place is seen, the units walk the same, tick by tick,
//! whether or not the object still stands, and the gate takes the same orders. (Rally points on remembered
//! structures: `production/rally.rs`; the gate's rule for each kind of target: `behaviours_gate.rs`.)
//!
//! Every `TestSim` checks the invariants I1–I7 and V1 after each tick.

use sc_sim::testing::{ScenarioBuilder, TestSim, dangling_references};
use sc_sim::{
    Command, DebugCommand, MoveGoal, Object, ObjectId, ObjectKind, OrderKind, PlayerCommand, PlayerId,
    ProductionCommand, Reject, SimEvent, SpawnAt, Target, TargetView, TilePos, Vec2,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const PK: ObjectKind = ObjectKind::Peacekeeper;

/// Player 1's Peacekeepers 1 and 2, which attack, and 3, which moves to the target.
const ATTACKERS: [u32; 2] = [1, 2];
const MOVER: u32 = 3;
/// Player 2's Generator, on tiles (34, 4) to (35, 5).
const GENERATOR: ObjectId = ObjectId(5);
/// The Generator's last-known position: the centre of its footprint.
const LAST_KNOWN: Vec2 = Vec2::new(2240.0, 320.0);
/// Where unit 1's queued Move goes.
const BACK: Vec2 = Vec2::new(1200.0, 700.0);

fn id(n: u32) -> ObjectId {
    ObjectId(n)
}

fn ids(ns: &[u32]) -> Vec<ObjectId> {
    ns.iter().map(|&n| id(n)).collect()
}

fn order(units: &[u32], kind: OrderKind, queue: bool) -> PlayerCommand {
    PlayerCommand {
        player: P1,
        command: Command::Order {
            units: ids(units),
            kind,
            queue,
        },
    }
}

/// 40 × 24 open tiles. Player 1's Peacekeepers 1 to 3 in the west (the mover south of the others, out of their way);
/// its scout 4 by player 2's Generator 5 in the east in tick 1, removed in tick 2, so that player 1 remembers the
/// Generator out of its sight; and player 2's Generator 6 in the south, which nobody sees, so that player 2 keeps a
/// structure when 5 goes. No unit fights by itself.
fn remembered() -> TestSim {
    let builder = ScenarioBuilder::new(&format!("{}\n", ".".repeat(40)).repeat(24))
        .spawn(PK, Some(P1), SpawnAt::Point(Vec2::new(200.0, 300.0)))
        .spawn(PK, Some(P1), SpawnAt::Point(Vec2::new(200.0, 400.0)))
        .spawn(PK, Some(P1), SpawnAt::Point(Vec2::new(200.0, 1000.0)))
        .spawn(PK, Some(P1), SpawnAt::Point(Vec2::new(2100.0, 320.0)))
        .spawn(ObjectKind::Generator, Some(P2), SpawnAt::Tile(TilePos::new(34, 4)))
        .spawn(ObjectKind::Generator, Some(P2), SpawnAt::Tile(TilePos::new(20, 20)));
    let mut t = builder.start();
    t.without_auto_targeting();
    t.run(1);
    assert!(t.sim().is_visible(P1, t.sim().object(GENERATOR).unwrap()));
    t.debug_at(2, DebugCommand::Remove { id: id(4) }).run_until(2);
    assert_eq!(
        t.sim().target_view(P1, GENERATOR),
        TargetView::Remembered {
            position: LAST_KNOWN,
            footprint: t.sim().object(GENERATOR).unwrap().placement.unwrap()
        }
    );
    t
}

/// In tick 3: the attackers get their default right-click order on the Generator, an Attack, unit 1 with a Move
/// queued after it, and the mover a Move to it.
fn ordered(t: &mut TestSim) {
    let kind = t.sim().default_order(id(1), Target::Object(GENERATOR));
    assert_eq!(
        kind,
        Some(OrderKind::Attack(GENERATOR)),
        "the right-click on a remembered enemy building"
    );
    for (cmd, queue) in [
        (order(&ATTACKERS, OrderKind::Attack(GENERATOR), false), false),
        (order(&[1], OrderKind::Move(Target::Point(BACK)), true), true),
        (
            order(&[MOVER], OrderKind::Move(Target::Object(GENERATOR)), false),
            false,
        ),
    ] {
        assert_eq!(t.sim().validate(&cmd), Ok(()), "{cmd:?} (queued: {queue})");
        t.at(3, P1, cmd.command);
    }
    t.run_until(3);
}

fn unit(t: &TestSim, n: u32) -> &Object {
    t.sim().object(id(n)).unwrap()
}

fn current(t: &TestSim, n: u32) -> Option<OrderKind> {
    unit(t, n)
        .orders
        .as_ref()
        .unwrap()
        .current
        .as_ref()
        .map(|a| a.order.kind)
}

/// Whether player 1 still has its record of the Generator.
fn remembers(t: &TestSim) -> bool {
    t.sim().vision().player(P1).unwrap().record(GENERATOR).is_some()
}

/// Design change 0139: Peacekeepers right-clicked onto a remembered enemy building attack it. They walk to its
/// last-known position, a move to the point, engaging nothing on the way (so nothing reads the building); the unit
/// sent to it with a Move walks there the same way. Once they see the Generator standing, the attackers engage it and
/// shoot it, and the mover goes on to it and stops within proximity.
#[test]
fn peacekeepers_walk_to_a_remembered_building_and_attack_it_if_it_stands() {
    let mut t = remembered();
    ordered(&mut t);
    let mut seen = None;
    for tick in 4..900 {
        t.step();
        if t.sim().is_visible(P1, t.sim().object(GENERATOR).unwrap()) {
            seen = Some(tick);
            break;
        }
        for n in ATTACKERS.into_iter().chain([MOVER]) {
            let o = unit(&t, n);
            assert_eq!(
                o.mover.as_ref().unwrap().goal,
                MoveGoal::Point(LAST_KNOWN),
                "#{n}, tick {tick}"
            );
            assert_eq!(o.attack.as_ref().unwrap().target, None, "#{n}, tick {tick}");
        }
    }
    let seen = seen.expect("they come in sight of it");
    t.run_until(seen + 200);
    for n in ATTACKERS {
        assert_eq!(current(&t, n), Some(OrderKind::Attack(GENERATOR)), "#{n}");
        let shots = t.events().iter().filter(|(tick, e)| {
            *tick > seen
                && matches!(e, SimEvent::Attacked { attacker, target, .. } if *attacker == id(n) && *target == GENERATOR)
        });
        assert!(shots.count() > 0, "#{n} shoots the Generator");
    }
    let mover = unit(&t, MOVER);
    assert!(
        mover.orders.as_ref().unwrap().is_idle(),
        "the Move completes by the Generator"
    );
    let gap = sc_sim::Shape::of(mover).gap(&sc_sim::Shape::of(unit(&t, GENERATOR.0)));
    assert!(gap <= sc_sim::orders::PROXIMITY_SU, "{gap}");
}

/// Design change 0139, "Nothing about the real state leaks", and its Verification: "Right-click a remembered enemy
/// building with Peacekeepers: they walk there and attack if it still stands, or stop on arrival if it was destroyed
/// unseen." Two games: in the second the Generator is destroyed in the fog in tick 4, after the orders. Until the
/// units see its place, they walk the same in both, tick by tick (position, move and orders), with the Generator
/// Remembered to player 1 in both, the same default order on it, and the gate taking the same orders on it. In the
/// game where it is gone, seeing its place ends the orders on it there: the attacker without a queue and the mover
/// stop and idle, and unit 1 takes its queued Move.
#[test]
fn the_walk_to_a_remembered_building_is_the_same_whether_or_not_it_stands() {
    let mut games: Vec<TestSim> = (0..2).map(|_| remembered()).collect();
    for t in &mut games {
        ordered(t);
    }
    games[1].debug_at(4, DebugCommand::Remove { id: GENERATOR });
    let [standing, gone] = &mut games[..] else {
        unreachable!()
    };
    let rally = PlayerCommand {
        player: P1,
        command: Command::Production {
            objects: vec![id(1)],
            command: ProductionCommand::SetRally(Target::Object(GENERATOR)),
        },
    };
    let mut seen_gone = None;
    for tick in 4..900 {
        standing.step();
        gone.step();
        assert_eq!(dangling_references(gone.sim()), [], "tick {tick}");
        for n in ATTACKERS.into_iter().chain([MOVER]) {
            let (a, b) = (unit(standing, n), unit(gone, n));
            assert_eq!(a.pos, b.pos, "#{n}, tick {tick}");
            if remembers(gone) {
                assert_eq!(
                    (&a.orders, &a.mover, &a.attack),
                    (&b.orders, &b.mover, &b.attack),
                    "#{n}, tick {tick}"
                );
            }
        }
        if !remembers(gone) {
            seen_gone = Some(tick);
            break;
        }
        assert!(gone.sim().object(GENERATOR).is_none());
        for sim in [standing.sim(), gone.sim()] {
            assert!(
                matches!(sim.target_view(P1, GENERATOR), TargetView::Remembered { .. }),
                "tick {tick}"
            );
            assert_eq!(
                sim.default_order(id(2), Target::Object(GENERATOR)),
                Some(OrderKind::Attack(GENERATOR))
            );
            for kind in [OrderKind::Attack(GENERATOR), OrderKind::Move(Target::Object(GENERATOR))] {
                assert_eq!(sim.validate(&order(&[2], kind, true)), Ok(()), "{kind:?}, tick {tick}");
            }
            // Not a producer: the gate refuses the rally for that, before it looks at the target.
            assert_eq!(sim.validate(&rally), Err(Reject::NoProducers));
        }
    }
    let seen_gone = seen_gone.expect("the units see the Generator's place");
    assert!(standing.sim().is_visible(P1, standing.sim().object(GENERATOR).unwrap()));
    // Seen gone: the orders on it are over, at the end of the tick it was seen in.
    assert_eq!(gone.sim().target_view(P1, GENERATOR), TargetView::Gone);
    assert_eq!(gone.sim().default_order(id(2), Target::Object(GENERATOR)), None);
    assert_eq!(
        gone.sim().validate(&order(&[2], OrderKind::Attack(GENERATOR), false)),
        Err(Reject::UnknownObject(GENERATOR))
    );
    for n in [2, MOVER] {
        let o = unit(gone, n);
        assert!(o.orders.as_ref().unwrap().is_idle(), "#{n}");
        assert_eq!(o.mover.as_ref().unwrap().goal, MoveGoal::None, "#{n} stops there");
    }
    assert_eq!(current(gone, 1), None);
    assert_eq!(
        unit(gone, 1).orders.as_ref().unwrap().queue.len(),
        1,
        "its queued Move is next"
    );
    let stood: Vec<Vec2> = [2, MOVER].iter().map(|&n| unit(gone, n).pos).collect();
    gone.run(30);
    assert_eq!(current(gone, 1), Some(OrderKind::Move(Target::Point(BACK))));
    let now: Vec<Vec2> = [2, MOVER].iter().map(|&n| unit(gone, n).pos).collect();
    assert_eq!(now, stood, "they stand where they saw the place (tick {seen_gone})");
    standing.run(30);
    assert_eq!(
        current(standing, 2),
        Some(OrderKind::Attack(GENERATOR)),
        "where it stands, they attack on"
    );
}

/// The gate judges a remembered object by its record (design change 0139: no leak of the real state): an Attack on
/// a remembered patch is `NotAttackable` whether or not the patch still exists, and a Move to it is taken; an object
/// the player never saw stays out of reach (`NotVisible`, or `UnknownObject` for an id that names nothing).
#[test]
fn the_gate_takes_remembered_objects_by_their_records() {
    let scenario = ScenarioBuilder::new(&format!("{}\n", ".".repeat(40)).repeat(12))
        .spawn(PK, Some(P1), SpawnAt::Point(Vec2::new(200.0, 300.0)))
        .spawn(ObjectKind::CrystalPatch, None, SpawnAt::Tile(TilePos::new(5, 4)))
        .spawn(ObjectKind::CrystalPatch, None, SpawnAt::Tile(TilePos::new(35, 4)))
        .build();
    let mut games: Vec<TestSim> = (0..2).map(|_| TestSim::new(scenario.clone())).collect();
    let (patch, unseen) = (id(2), id(3));
    for t in &mut games {
        t.without_auto_targeting();
        t.debug_at(
            2,
            DebugCommand::SetPosition {
                id: id(1),
                pos: Vec2::new(1200.0, 600.0),
            },
        )
        .order_at(2, P1, &[id(1)], OrderKind::Stop, false)
        .run_until(2);
        assert!(matches!(t.sim().target_view(P1, patch), TargetView::Remembered { .. }));
    }
    games[1].debug_at(3, DebugCommand::Remove { id: patch }).run_until(3);
    games[0].run_until(3);
    for t in &games {
        let sim = t.sim();
        assert_eq!(
            sim.validate(&order(&[1], OrderKind::Attack(patch), false)),
            Err(Reject::NotAttackable(patch))
        );
        assert_eq!(
            sim.validate(&order(&[1], OrderKind::Move(Target::Object(patch)), false)),
            Ok(())
        );
        assert_eq!(
            sim.validate(&order(&[1], OrderKind::Move(Target::Object(unseen)), false)),
            Err(Reject::NotVisible(unseen))
        );
        let nothing = id(99);
        assert_eq!(
            sim.validate(&order(&[1], OrderKind::Attack(nothing), false)),
            Err(Reject::UnknownObject(nothing))
        );
        assert_eq!(sim.target_view(P1, unseen), TargetView::Gone);
    }
}
