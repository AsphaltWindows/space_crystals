//! Design change 0136's Verification for a rally on an enemy unit (`systems/controls/selection_and_hud.md > What a
//! rally point makes a new unit do`): "Robo Portal rallied on a visible enemy unit: new Drones follow it; if it leaves
//! sight, they go to its last-seen spot." (On a patch: `gathering.rs`, `a_drone_rallied_on_a_patch_gathers_it`; on
//! an enemy structure: `portals.rs`.)

use sc_sim::testing::{ScenarioBuilder, dangling_references};
use sc_sim::{DebugCommand, Faction, ObjectId, ObjectKind, OrderKind, ProductionCommand, Rally, Target, Vec2};

use super::queue::{crystals, produced, rally_of};
use super::{P1, P2, PK, at, flat, production, tile};

/// The Drone rallied on the enemy follows it (it is unarmed: "Enemy or other object → Move to it: Follow if it is a
/// unit", `factions/colonists/units.md > Drone > Right-Click Resolution`). The enemy leaves sight at once, set far
/// off: the Drone goes on to where its owner last saw it and its order ends there (design change 0034); the rally
/// point is a ground rally there by then (design change 0088), so the next Drone walks there too.
#[test]
fn drones_rallied_on_an_enemy_unit_follow_it_then_go_where_it_was_last_seen() {
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
    let drone = loop {
        t.step();
        if let Some(&(_, _, unit)) = produced(&t).first() {
            break ObjectId(unit);
        }
        assert!(t.sim().tick() < 400, "the Drone should be out by now");
    };
    t.step();
    let current = |t: &sc_sim::testing::TestSim| {
        let orders = t.sim().object(drone).unwrap().orders.as_ref().unwrap();
        orders.current.as_ref().map(|active| active.order.kind)
    };
    assert_eq!(current(&t), Some(OrderKind::Move(Target::Object(enemy))), "it follows");

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
    for _ in 0..400 {
        if current(&t).is_none() {
            break;
        }
        t.step();
    }
    assert_eq!(current(&t), None, "its order ended");
    assert!(
        t.pos(drone.0).distance(last_seen) < 40.0,
        "at the last-seen spot: {}",
        t.pos(drone.0)
    );
    assert_eq!(dangling_references(t.sim()), []);
}
