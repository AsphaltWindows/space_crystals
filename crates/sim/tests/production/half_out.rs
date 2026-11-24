//! A unit stuck half out of a portal's energy doorway (design change 0145, `factions/colonists/structures.md >
//! Portals > Portal Exits (energy doorway)`, point 4: it "walks out as far as there is room and stays there, half out
//! at the edge of the footprint. It is a normal unit there (selectable, attackable, able to attack)"; slice M3-K2a):
//! a Sabre Legionnaire that can't get out of its Bio Portal auto-targets an enemy in its reach and hits it, carries out
//! an explicit Attack on one, and an enemy's Attack on it hits it; it stays in its doorway all along. (A click on it
//! selects it: the client's `selection::draw_order`.)

use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{
    Command, Faction, ObjectId, ObjectKind, OrderKind, PlayerCommand, PlayerId, SimEvent, Target, Tick, Vec2,
};

use super::doors::exit_of;
use super::portals::{PORTAL, SABRE, run_exits, set_rally, train};
use super::queue::crystals;
use super::{P1, P2, at, tile};

/// Where the Sabre of [`walled_in`] stops: its box against the Drones east of the portal, its centre on the
/// footprint's east side.
const HALF_OUT: Vec2 = Vec2::new(960.0, 480.0);
/// The Drones of player 2's that its box touches ([`walled_in`]).
const IN_REACH: [ObjectId; 2] = [ObjectId(6), ObjectId(7)];

/// Player 1, Colonists, with an unturned Bio Portal (1), open north and east, on tiles (12, 6) to (14, 8) of a 30 × 20
/// map (x 768..960, y 384..576, its centre (864, 480)), and two Egress Nodes (2, 3) in the far corner: Beacon
/// Capacity 15. Mountains on the five tiles along its north side, (11, 5) to (15, 5), and a column of six Drones of
/// player 2's (4 to 9, Colonists: unarmed) east of it, touching, their west faces 18 su off its east side: no open
/// direction is free, so the Sabre it trains in tick 1, rallied due east, walks out east all the same and stops half
/// out against Drones 6 and 7, its order to the rally point failing in time. Then `more`.
fn walled_in(more: fn(ScenarioBuilder) -> ScenarioBuilder) -> TestSim {
    let mut rows = vec![".".repeat(30); 20];
    rows[5].replace_range(11..16, "MMMMM");
    let mut builder = ScenarioBuilder::new(&rows.join("\n"))
        .faction(P1, Faction::Colonists)
        .faction(P2, Faction::Colonists)
        .spawn(ObjectKind::BioPortal, Some(P1), tile(12, 6))
        .spawn(ObjectKind::EgressNode, Some(P1), tile(1, 1))
        .spawn(ObjectKind::EgressNode, Some(P1), tile(1, 4));
    for k in 0..6 {
        builder = builder.spawn(ObjectKind::Drone, Some(P2), at(996.0, 402.0 + 36.0 * k as f32));
    }
    let mut t = more(builder).start();
    t.debug_at(1, crystals(P1, 100));
    set_rally(&mut t, 1, Target::Point(Vec2::new(1500.0, 480.0)));
    train(&mut t, 1, SABRE, 1);
    t
}

/// Runs [`walled_in`]'s game to tick 200: its Sabre, out of production and heading due east, stands half
/// out, still trying to reach its rally point. Returns the Sabre.
fn stopped(t: &mut TestSim) -> ObjectId {
    let exits = run_exits(t, 200);
    assert_eq!(exits.len(), 1);
    let sabre = exits[0].unit;
    assert!(
        exits[0].heading.distance(Vec2::X) < 1e-6,
        "due east: {}",
        exits[0].heading
    );
    still_half_out(t, sabre);
    assert_eq!(failed(t, sabre), None, "its move goes on");
    sabre
}

/// When `unit`'s move failed, if it has.
fn failed(t: &TestSim, unit: ObjectId) -> Option<Tick> {
    let mut events = t.events().iter();
    events
        .find(|(_, e)| matches!(e, SimEvent::MoveFailed { unit: u } if *u == unit))
        .map(|(tick, _)| *tick)
}

/// [`stopped`], and on until the Sabre's order to the rally point has failed: it is idle there. Returns the Sabre and
/// the tick its order failed.
fn stuck(t: &mut TestSim) -> (ObjectId, Tick) {
    let sabre = stopped(t);
    while failed(t, sabre).is_none() {
        assert!(t.sim().tick() < 400, "its order never failed");
        run_exits(t, t.sim().tick() + 1);
    }
    (sabre, failed(t, sabre).unwrap())
}

/// The hits `attacker` dealt `target` after tick `after`.
fn hits(t: &TestSim, attacker: ObjectId, target: ObjectId, after: Tick) -> usize {
    t.events()
        .iter()
        .filter(|(tick, e)| {
            *tick > after && matches!(e, SimEvent::Hit { attacker: a, target: b, .. } if *a == attacker && *b == target)
        })
        .count()
}

/// `player`'s order of `kind` for `unit`, as the gate sees it.
fn order(player: PlayerId, unit: ObjectId, kind: OrderKind) -> PlayerCommand {
    let command = Command::Order {
        units: vec![unit],
        kind,
        queue: false,
    };
    PlayerCommand { player, command }
}

/// `sabre` is still where it stopped, in its doorway.
fn still_half_out(t: &TestSim, sabre: ObjectId) {
    assert_eq!(
        (t.pos(sabre.0), exit_of(t.sim(), sabre)),
        (HALF_OUT, Some(PORTAL)),
        "tick {}",
        t.sim().tick()
    );
}

/// "Able to attack", on its own: idle half out, the Sabre picks the enemy in its reach (`systems/combat.md >
/// AutoTargeting`) and hits it, again and again, without leaving its doorway.
#[test]
fn a_unit_half_out_of_a_portal_auto_targets_an_enemy_in_reach_and_hits_it() {
    let mut t = walled_in(|b| b);
    let (sabre, idle) = stuck(&mut t);
    run_exits(&mut t, idle + 40);
    let dealt = IN_REACH.map(|drone| hits(&t, sabre, drone, idle));
    assert!(dealt.iter().sum::<usize>() >= 2, "{dealt:?}");
    still_half_out(&t, sabre);
}

/// "Able to attack", ordered: with auto-targeting off, the Sabre takes an explicit Attack on a Drone in its reach,
/// through the gate, and hits it without leaving its doorway; the other one it leaves alone. Ordered while its move
/// to the rally point still tries, which the Attack replaces, and again once it is idle there.
#[test]
fn a_unit_half_out_of_a_portal_carries_out_an_attack_on_an_enemy_in_reach() {
    let mut t = walled_in(|b| b);
    t.without_auto_targeting();
    let sabre = stopped(&mut t);
    let attack = OrderKind::Attack;
    assert_eq!(t.sim().validate(&order(P1, sabre, attack(IN_REACH[0]))), Ok(()));
    t.order_at(201, P1, &[sabre], attack(IN_REACH[0]), false);
    run_exits(&mut t, 240);
    assert!(hits(&t, sabre, IN_REACH[0], 200) >= 2);
    assert_eq!(hits(&t, sabre, IN_REACH[1], 200), 0);
    assert_eq!(failed(&t, sabre), None, "the move gave way to the Attack");
    still_half_out(&t, sabre);

    let mut t = walled_in(|b| b);
    t.without_auto_targeting();
    let (sabre, idle) = stuck(&mut t);
    run_exits(&mut t, idle + 10);
    assert_eq!(hits(&t, sabre, IN_REACH[1], idle), 0, "holding its fire");
    t.order_at(idle + 11, P1, &[sabre], attack(IN_REACH[1]), false);
    run_exits(&mut t, idle + 50);
    assert!(hits(&t, sabre, IN_REACH[1], idle) >= 2);
    assert_eq!(hits(&t, sabre, IN_REACH[0], idle), 0);
    still_half_out(&t, sabre);
}

/// "Attackable": a Rifle Legionnaire of player 2's east of the Drones (10, 4 grid units of Range) takes an Attack on
/// the Sabre through the gate and hits it, auto-targeting off.
#[test]
fn a_unit_half_out_of_a_portal_can_be_attacked() {
    let mut t = walled_in(|b| b.spawn(ObjectKind::RifleLegionnaire, Some(P2), at(1150.0, 480.0)));
    t.without_auto_targeting();
    let (sabre, idle) = stuck(&mut t);
    let rifle = ObjectId(10);
    assert_eq!(t.sim().validate(&order(P2, rifle, OrderKind::Attack(sabre))), Ok(()));
    t.order_at(idle + 1, P2, &[rifle], OrderKind::Attack(sabre), false);
    run_exits(&mut t, idle + 40);
    assert!(hits(&t, rifle, sabre, idle) >= 1);
    still_half_out(&t, sabre);
}
