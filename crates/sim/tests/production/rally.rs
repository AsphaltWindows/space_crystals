//! Rally points: `factions/gdo/structures.md > Barracks > RallyPoint behavior`,
//! `systems/controls/selection_and_hud.md > Rally points on enemy objects`, `> What a rally point makes a new unit do`
//! (design changes 0086, 0088 and 0136), and on remembered structures (`core/vision.md > Explored > Targeting
//! remembered objects`, design change 0139).

use sc_sim::testing::{TestSim, dangling_references};
use sc_sim::{DebugCommand, MoveStatus, ObjectId, ObjectKind, OrderKind, Rally, SimEvent, Target, Tick, Vec2};

use super::queue::{crystals, produced, rally, rally_of, train};
use super::{BARRACKS, P1, P2, PK, at, base, tile};

/// The unit produced first, and the tick it came out in.
fn first_unit(t: &TestSim) -> (Tick, ObjectId) {
    let (tick, _, unit) = produced(t)[0];
    (tick, ObjectId(unit))
}

/// Plan acceptance, rally on the ground: "the unit moves there". And the spawn-tick rule: it acts from the tick
/// after it appears.
#[test]
fn a_ground_rally_sends_the_unit_there() {
    let mut t = base().start();
    let point = Vec2::new(1200.0, 700.0);
    t.debug_at(1, crystals(P1, 50));
    t.at(1, P1, rally(&[BARRACKS], Target::Point(point)));
    t.at(1, P1, train(&[BARRACKS]));
    t.run_until(80);
    assert_eq!(rally_of(t.sim(), BARRACKS), Some(Rally::Point(point)));
    let (tick, unit) = first_unit(&t);
    assert_eq!(tick, 80);
    let o = t.sim().object(unit).unwrap();
    assert_eq!(o.pos, Vec2::new(704.0, 320.0), "in the doorway, half inside");
    assert_eq!(o.pos, o.prev_pos);
    assert_eq!(
        o.orders.as_ref().unwrap().queue.front().map(|order| order.kind),
        Some(OrderKind::Move(Target::Point(point))),
        "the default right-click command on the rally point"
    );
    t.run(1);
    assert_ne!(t.pos(unit.0), Vec2::new(704.0, 320.0), "it acts from the next tick");
    t.run_until(400);
    assert_eq!(t.pos(unit.0), point);
    assert_eq!(t.status(unit.0), MoveStatus::Arrived);
    assert!(t.sim().object(unit).unwrap().orders.as_ref().unwrap().is_idle());
}

/// Plan acceptance, rally on an enemy: "the unit attacks it". Once the enemy is destroyed in sight, the rally
/// point resets to None.
#[test]
fn a_rally_on_an_enemy_makes_the_unit_attack_it() {
    // An enemy Generator a tile east of the doorway, in the Barracks' sight (4 grid units from its centre).
    let mut t = base().spawn(ObjectKind::Generator, Some(P2), tile(12, 4)).start();
    let enemy = ObjectId(3);
    t.debug_at(1, crystals(P1, 100));
    t.at(2, P1, rally(&[BARRACKS], Target::Object(enemy)));
    t.at(2, P1, train(&[BARRACKS])).at(2, P1, train(&[BARRACKS]));
    t.run_until(81);
    assert!(matches!(rally_of(t.sim(), BARRACKS), Some(Rally::Object { id, .. }) if id == enemy));
    let (_, unit) = first_unit(&t);
    assert_eq!(
        t.sim()
            .object(unit)
            .unwrap()
            .orders
            .as_ref()
            .unwrap()
            .queue
            .front()
            .map(|o| o.kind),
        Some(OrderKind::Attack(enemy))
    );
    let gone = (82..4000).find(|_| {
        t.step();
        t.sim().object(enemy).is_none()
    });
    assert!(gone.is_some(), "the produced units destroy it");
    assert!(t.events().iter().any(
        |(_, e)| matches!(e, SimEvent::Attacked { attacker, target, .. } if *attacker == unit && *target == enemy)
    ));
    assert_eq!(rally_of(t.sim(), BARRACKS), None, "the object no longer exists");
    assert_eq!(dangling_references(t.sim()), []);
}

/// Plan acceptance, rally on an own unit: "the unit follows it" (`systems/controls/commands.md > Move`: a unit is
/// followed).
#[test]
fn a_rally_on_an_own_unit_makes_the_unit_follow_it() {
    let mut t = base().spawn(PK, Some(P1), at(1000.0, 320.0)).start();
    let leader = ObjectId(3);
    t.debug_at(1, crystals(P1, 50));
    t.at(1, P1, rally(&[BARRACKS], Target::Object(leader)));
    t.at(1, P1, train(&[BARRACKS]));
    t.run_until(160);
    let (_, unit) = first_unit(&t);
    let gap = |t: &TestSim| t.pos(unit.0).distance(t.pos(leader.0)) - 24.0;
    assert!(gap(&t) <= 8.5, "it walked up to the leader: {}", gap(&t));
    // The leader walks off; the rally point follows it, and so does the unit.
    t.order_at(
        161,
        P1,
        &[leader],
        OrderKind::Move(Target::Point(Vec2::new(1500.0, 800.0))),
        false,
    );
    t.run_until(500);
    assert_eq!(t.pos(leader.0), Vec2::new(1500.0, 800.0));
    assert!(t.pos(unit.0).distance(t.pos(leader.0)) < 60.0, "{}", t.pos(unit.0));
    let orders = t.sim().object(unit).unwrap().orders.as_ref().unwrap();
    assert_eq!(
        orders.current.as_ref().map(|active| active.order.kind),
        Some(OrderKind::Move(Target::Object(leader))),
        "a Follow never completes by itself"
    );
    assert_eq!(
        rally_of(t.sim(), BARRACKS),
        Some(Rally::Object {
            id: leader,
            at: Vec2::new(1500.0, 800.0)
        })
    );
}

/// Plan acceptance, rally on a neutral station: "the unit moves to it"; and on a removed target: "the rally
/// becomes None, and later units spawn with no command."
#[test]
fn a_rally_on_a_neutral_station_and_a_target_that_is_removed() {
    let mut t = base()
        .spawn(ObjectKind::SupplyDeliveryStation, None, tile(12, 4))
        .start();
    let station = ObjectId(3);
    t.debug_at(1, crystals(P1, 100));
    t.at(2, P1, rally(&[BARRACKS], Target::Object(station)));
    t.at(2, P1, train(&[BARRACKS])).at(2, P1, train(&[BARRACKS]));
    t.run_until(150);
    let (_, unit) = first_unit(&t);
    // The station covers tiles (12, 4) to (13, 5): its west wall is at x 768.
    let o = t.sim().object(unit).unwrap();
    assert!(
        768.0 - (o.pos.x + 12.0) <= 8.0 && o.pos.x < 768.0,
        "at its wall: {}",
        o.pos
    );
    assert!(
        o.orders.as_ref().unwrap().is_idle(),
        "a static object completes the move"
    );

    t.debug_at(151, DebugCommand::Remove { id: station });
    t.run_until(170);
    assert_eq!(rally_of(t.sim(), BARRACKS), None);
    assert_eq!(dangling_references(t.sim()), []);
    let second = ObjectId(produced(&t)[1].2);
    assert_eq!(produced(&t)[1].0, 161);
    let o = t.sim().object(second).unwrap();
    assert!(o.orders.as_ref().unwrap().is_idle(), "no command");
    // Design change 0086: "units walk straight out of the exit doorway and then have no command."
    assert_eq!(o.pos, Vec2::new(716.0, 320.0));
}

/// Design change 0086's Verification: a rally point on the ground, then the Barracks itself as the target: the
/// rally is cleared, and "the next Peacekeeper walks just out of side B and stops."
#[test]
fn a_rally_on_the_building_itself_clears_it() {
    let mut t = base().start();
    t.debug_at(1, crystals(P1, 50));
    t.at(1, P1, rally(&[BARRACKS], Target::Point(Vec2::new(1200.0, 700.0))));
    t.at(1, P1, train(&[BARRACKS]));
    t.run(1);
    assert!(rally_of(t.sim(), BARRACKS).is_some());
    t.at(2, P1, rally(&[BARRACKS], Target::Object(BARRACKS)));
    t.run_until(120);
    assert_eq!(rally_of(t.sim(), BARRACKS), None);
    let (_, unit) = first_unit(&t);
    let o = t.sim().object(unit).unwrap();
    assert_eq!(o.pos, Vec2::new(716.0, 320.0), "just outside side B, touching it");
    assert!(o.orders.as_ref().unwrap().is_idle());
    let mover = o.mover.as_ref().unwrap();
    assert_eq!((mover.exit, mover.velocity), (None, Vec2::ZERO));
    // With several producers selected, only the one clicked clears; the others rally on it.
    let mut two = base().spawn(ObjectKind::Barracks, Some(P1), tile(8, 8)).start();
    let other = ObjectId(3);
    two.at(1, P1, rally(&[BARRACKS, other], Target::Object(BARRACKS)));
    two.run(1);
    assert_eq!(rally_of(two.sim(), BARRACKS), None);
    assert!(matches!(rally_of(two.sim(), other), Some(Rally::Object { id, .. }) if id == BARRACKS));
}

/// Design change 0088's Verification: a rally on a visible enemy unit; the enemy walks into the fog, and the rally
/// becomes a ground rally where it was last seen; "the next unit walks there"; and "have the enemy unit die in the
/// fog. The rally marker doesn't change."
#[test]
fn a_rally_on_an_enemy_that_leaves_sight_becomes_a_ground_rally() {
    let mut t = base().spawn(PK, Some(P2), at(800.0, 320.0)).start();
    t.without_auto_targeting();
    let enemy = ObjectId(3);
    t.debug_at(1, crystals(P1, 50));
    t.at(2, P1, rally(&[BARRACKS], Target::Object(enemy)));
    t.run(2);
    assert_eq!(
        rally_of(t.sim(), BARRACKS),
        Some(Rally::Object {
            id: enemy,
            at: Vec2::new(800.0, 320.0)
        })
    );
    // The enemy walks east, out of the Barracks' sight (4 grid units from its centre, at x 608).
    t.order_at(
        3,
        P2,
        &[enemy],
        OrderKind::Move(Target::Point(Vec2::new(1700.0, 320.0))),
        false,
    );
    let mut last_seen = t.pos(enemy.0);
    let lost = (3..400).find(|_| {
        t.step();
        let seen = t.sim().is_visible(P1, t.sim().object(enemy).unwrap());
        if seen {
            last_seen = t.pos(enemy.0);
        }
        !seen
    });
    assert!(lost.is_some());
    assert_eq!(
        rally_of(t.sim(), BARRACKS),
        Some(Rally::Point(last_seen)),
        "where it was last seen"
    );
    assert!(last_seen.x > 830.0 && last_seen.x < 900.0, "{last_seen}");
    // It dies in the fog: nothing changes, so the marker gives nothing away.
    t.run_until(300);
    let tick = t.sim().tick();
    t.debug_at(tick + 1, DebugCommand::Remove { id: enemy });
    t.at(tick + 1, P1, train(&[BARRACKS]));
    t.run(100);
    assert_eq!(rally_of(t.sim(), BARRACKS), Some(Rally::Point(last_seen)));
    let (_, unit) = first_unit(&t);
    t.run(200);
    assert_eq!(t.pos(unit.0), last_seen, "the next unit walks there");
}

/// Player 2's Generator (3) on tiles (24, 10) to (25, 11), its centre (1600, 704), far from the Barracks, with a scout
/// of player 1 (4) next to it, which walks away in tick 3; and a Generator of player 2's (5) far in the other corner,
/// which nobody sees, so that player 2 has a structure left when the first goes. No unit fights by itself.
fn remembered_generator() -> TestSim {
    let mut t = base()
        .spawn(ObjectKind::Generator, Some(P2), tile(24, 10))
        .spawn(PK, Some(P1), at(1480.0, 700.0))
        .spawn(ObjectKind::Generator, Some(P2), tile(0, 14))
        .start();
    t.without_auto_targeting();
    t.debug_at(1, crystals(P1, 50));
    let away = OrderKind::Move(Target::Point(Vec2::new(900.0, 900.0)));
    t.order_at(3, P1, &[ObjectId(4)], away, false);
    t
}

/// Design change 0139: "A rally point on an enemy structure stays on it after it leaves sight", and design change
/// 0136: a new unit gets its own default right-click command on it. Set while the scout sees the Generator, the rally
/// stays on it, at its last-known position, once nobody sees it; set on it while it is out of sight, the same. The
/// Peacekeeper trained then gets Attack, walks there and, once it sees the Generator, shoots it.
#[test]
fn a_rally_on_an_enemy_structure_stays_on_it_out_of_sight_and_new_units_attack_it() {
    let generator = ObjectId(3);
    let on_it = Some(Rally::Object {
        id: generator,
        at: Vec2::new(1600.0, 704.0),
    });
    for set_in_sight in [true, false] {
        let mut t = remembered_generator();
        let tick = if set_in_sight { 2 } else { 201 };
        t.at(tick, P1, rally(&[BARRACKS], Target::Object(generator)));
        t.run_until(200);
        assert!(!t.sim().is_visible(P1, t.sim().object(generator).unwrap()));
        assert!(
            t.sim()
                .vision()
                .player(P1)
                .unwrap()
                .remembered_record(generator)
                .is_some()
        );
        t.at(201, P1, train(&[BARRACKS]));
        t.run_until(201);
        assert_eq!(rally_of(t.sim(), BARRACKS), on_it, "set in sight: {set_in_sight}");
        t.run_until(290);
        let (_, unit) = first_unit(&t);
        assert_eq!(
            t.sim()
                .object(unit)
                .unwrap()
                .orders
                .as_ref()
                .unwrap()
                .current
                .as_ref()
                .map(|a| a.order.kind),
            Some(OrderKind::Attack(generator)),
            "the default right-click command on the rally target"
        );
        let shot = (290..900).find(|_| {
            t.step();
            t.events()
                .iter()
                .any(|(_, e)| matches!(e, SimEvent::Attacked { attacker, target, .. } if *attacker == unit && *target == generator))
        });
        assert!(
            shot.is_some(),
            "it walks there and shoots the Generator (set in sight: {set_in_sight})"
        );
        assert_eq!(rally_of(t.sim(), BARRACKS), on_it);
    }
}

/// Design change 0139: "Nothing about the real state leaks: [...] any rally marker treat[s] the remembered object as
/// present until the player sees otherwise." The Generator is destroyed in the fog right after the rally is set on
/// it: the rally stays on it, and the Peacekeeper trained walks to it with its Attack exactly as it does in a game
/// where the Generator stands, tick by tick, until it sees the Generator's place. Then, in the game where it is gone,
/// the rally resets to None ("the order ends the way it would for a vanished target") and the unit stops where it
/// is, its order over; where it stands, the unit goes on with its Attack.
#[test]
fn a_rally_on_a_structure_destroyed_unseen_holds_until_its_place_is_seen() {
    let generator = ObjectId(3);
    let mut games: Vec<TestSim> = (0..2).map(|_| remembered_generator()).collect();
    for t in &mut games {
        t.at(201, P1, rally(&[BARRACKS], Target::Object(generator)));
        t.at(201, P1, train(&[BARRACKS]));
        t.run_until(200);
    }
    games[1].debug_at(202, DebugCommand::Remove { id: generator });
    let [standing, gone] = &mut games[..] else {
        unreachable!()
    };
    let remembers = |t: &TestSim| t.sim().vision().player(P1).unwrap().record(generator).is_some();
    let mut seen_gone = None;
    for tick in 201..900 {
        standing.step();
        gone.step();
        assert_eq!(dangling_references(gone.sim()), [], "tick {tick}");
        let units: Vec<_> = [&*standing, &*gone]
            .map(|t| {
                produced(t)
                    .first()
                    .map(|&(_, _, unit)| t.sim().object(ObjectId(unit)).cloned().unwrap())
            })
            .into_iter()
            .collect();
        if let (Some(a), Some(b)) = (&units[0], &units[1]) {
            assert_eq!(a.pos, b.pos, "tick {tick}");
            if remembers(gone) {
                assert_eq!(a.orders, b.orders, "tick {tick}");
                assert_eq!(a.mover, b.mover, "tick {tick}");
            }
        }
        if !remembers(gone) {
            seen_gone = Some(tick);
            break;
        }
        assert_eq!(
            rally_of(gone.sim(), BARRACKS),
            rally_of(standing.sim(), BARRACKS),
            "tick {tick}"
        );
        assert!(matches!(rally_of(gone.sim(), BARRACKS), Some(Rally::Object { id, .. }) if id == generator));
    }
    let seen_gone = seen_gone.expect("the unit sees the Generator's place");
    assert!(
        produced(gone).first().is_some_and(|&(tick, _, _)| tick < seen_gone),
        "the unit was on its way"
    );
    let unit = ObjectId(produced(gone)[0].2);
    assert_eq!(rally_of(gone.sim(), BARRACKS), None, "seen gone in tick {seen_gone}");
    let o = gone.sim().object(unit).unwrap();
    assert!(o.orders.as_ref().unwrap().is_idle(), "its Attack ended");
    assert_eq!(o.mover.as_ref().unwrap().goal, sc_sim::MoveGoal::None, "it stops there");
    assert!(standing.sim().is_visible(P1, standing.sim().object(generator).unwrap()));
    let o = standing.sim().object(unit).unwrap();
    assert_eq!(
        o.orders.as_ref().unwrap().current.as_ref().map(|a| a.order.kind),
        Some(OrderKind::Attack(generator))
    );
    gone.run(20);
    assert_eq!(gone.pos(unit.0), o.pos, "it stood where it saw the place");
}
