//! Design change 0136's Verification for a rally on an enemy unit (`systems/controls/selection_and_hud.md > What a
//! rally point makes a new unit do`): "Robo Portal rallied on a visible enemy unit: new Drones follow it; if it leaves
//! sight, they go to its last-seen spot." Since design change 0200, a Drone following the enemy as it leaves sight
//! stops where it is; the rally point has turned into a ground rally at the last-seen spot (design change 0088,
//! which 0200 leaves as it was: "a unit produced after the enemy was lost still moves to the rally location"), and
//! the next Drone goes there. (On a patch: `gathering/games.rs`, `a_drone_rallied_on_a_patch_gathers_it`; on an
//! enemy structure: `portals.rs`.)

use sc_sim::testing::{ScenarioBuilder, TestSim, dangling_references};
use sc_sim::{
    DebugCommand, Faction, MoveStatus, ObjectId, ObjectKind, OrderKind, ProductionCommand, Rally, Target, Vec2,
};

use super::queue::{crystals, produced, rally_of};
use super::{P1, P2, PK, at, flat, production, tile};

/// The Drone rallied on the enemy follows it (it is unarmed: "Enemy or other object → Move to it: Follow if it is a
/// unit", `factions/colonists/units/drone.md > Right-Click Resolution`), and waits by it, out of the portal. The
/// enemy leaves sight at once, set far off: the Drone's Follow ends in the next tick, and it stays where it is
/// (design change 0200). The rally point is a ground rally where the enemy was last seen by then (design change
/// 0088), so the next Drone, trained after the enemy was lost, walks there.
#[test]
fn drones_rallied_on_an_enemy_unit_follow_it_then_the_next_goes_where_it_was_last_seen() {
    // Player 1, Colonists: a Robo Portal (1) on tiles (12, 6) to (15, 9), an Egress Node (2); player 2's
    // Peacekeeper (3) east of the portal, in its sight (4 grid units from its centre, (896, 512)).
    let mut t = ScenarioBuilder::new(&flat(30, 20))
        .faction(P1, Faction::Colonists)
        .spawn(ObjectKind::RoboPortal, Some(P1), tile(12, 6))
        .spawn(ObjectKind::EgressNode, Some(P1), tile(1, 1))
        .spawn(PK, Some(P2), at(1100.0, 512.0))
        .start();
    t.without_auto_targeting();
    let (portal, enemy) = (ObjectId(1), ObjectId(3));
    let to_portal = |command| production(&[portal], command);
    t.debug_at(1, crystals(P1, 100));
    t.at(2, P1, to_portal(ProductionCommand::SetRally(Target::Object(enemy))));
    t.at(2, P1, to_portal(ProductionCommand::Train(ObjectKind::Drone)));
    let next_drone = |t: &mut TestSim, n: usize| loop {
        t.step();
        if let Some(&(_, _, unit)) = produced(t).get(n) {
            break ObjectId(unit);
        }
        assert!(t.sim().tick() < 1000, "Drone {n} should be out by now");
    };
    let current = |t: &TestSim, drone: ObjectId| {
        let orders = t.sim().object(drone).unwrap().orders.as_ref().unwrap();
        orders.current.as_ref().map(|active| active.order.kind)
    };
    let drone = next_drone(&mut t, 0);
    t.step();
    assert_eq!(
        current(&t, drone),
        Some(OrderKind::Move(Target::Object(enemy))),
        "it follows"
    );
    // Out through the portal's east side (x 1024), it waits by the enemy, at rest (a Drone brakes off its momentum
    // when it stops, slice M3-G).
    let waits = |t: &TestSim| {
        let mover = t.mover(drone.0);
        mover.status != MoveStatus::Moving && mover.momentum == Vec2::ZERO && t.pos(drone.0).x > 1024.0 + 18.0
    };
    while !waits(&t) {
        assert!(t.sim().tick() < 600, "the Drone never caught up");
        t.step();
    }

    let last_seen = t.pos(enemy.0);
    let tick = t.sim().tick() + 1;
    t.debug_at(
        tick,
        DebugCommand::SetPosition {
            id: enemy,
            pos: Vec2::new(1850.0, 1200.0),
        },
    );
    t.run(2);
    assert_eq!(
        rally_of(t.sim(), portal),
        Some(Rally::Point(last_seen)),
        "a ground rally there"
    );
    assert_eq!(current(&t, drone), None, "its Follow ended");
    let stopped = t.pos(drone.0);
    t.run(20);
    assert_eq!(t.pos(drone.0), stopped, "it stays where it stopped");

    t.at(tick + 22, P1, to_portal(ProductionCommand::Train(ObjectKind::Drone)));
    let next = next_drone(&mut t, 1);
    t.step();
    assert_eq!(current(&t, next), Some(OrderKind::Move(Target::Point(last_seen))));
    for _ in 0..400 {
        if current(&t, next).is_none() {
            break;
        }
        t.step();
    }
    assert_eq!(current(&t, next), None, "its order ended");
    assert!(
        t.pos(next.0).distance(last_seen) < 40.0,
        "at the last-seen spot: {}",
        t.pos(next.0)
    );
    assert_eq!(current(&t, drone), None, "the first has no order since");
    assert_eq!(dangling_references(t.sim()), []);
}
