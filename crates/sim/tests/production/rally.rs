//! Rally points: `factions/gdo/structures.md > Barracks > RallyPoint behavior`,
//! `systems/controls/selection_and_hud.md > Rally points on enemy objects` (design changes 0086 and 0088).

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

/// Design change 0088's TBD, the placeholder: a rally on a remembered enemy structure out of sight is a ground
/// rally at its last-known position. One the player never saw is no target.
#[test]
fn a_rally_on_a_remembered_enemy_structure_is_a_ground_rally() {
    // A scout of player 1 next to player 2's Generator, far from the Barracks.
    let mut t = base()
        .spawn(ObjectKind::Generator, Some(P2), tile(24, 10))
        .spawn(PK, Some(P1), at(1480.0, 700.0))
        .start();
    t.without_auto_targeting();
    let (generator, scout) = (ObjectId(3), ObjectId(4));
    t.run(2);
    assert!(t.sim().is_visible(P1, t.sim().object(generator).unwrap()));
    t.order_at(
        3,
        P1,
        &[scout],
        OrderKind::Move(Target::Point(Vec2::new(900.0, 900.0))),
        false,
    );
    t.run_until(200);
    assert!(!t.sim().is_visible(P1, t.sim().object(generator).unwrap()));
    t.at(201, P1, rally(&[BARRACKS], Target::Object(generator)));
    t.run(1);
    // The Generator covers tiles (24, 10) and (25, 11).
    assert_eq!(
        rally_of(t.sim(), BARRACKS),
        Some(Rally::Point(Vec2::new(1600.0, 704.0)))
    );
}
