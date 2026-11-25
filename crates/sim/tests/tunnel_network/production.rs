//! The Hideout's production into the network and its rally points (`factions/syndicate/structures/hideout.md >
//! Produces`, `structures/README.md > Rally Point Behavior`; design changes 0246, 0252 and 0253).

use sc_sim::network::Place;
use sc_sim::{ProductionCommand, Target};

use super::*;

/// The Hideout of [`hideout_base`].
const H1: ObjectId = ObjectId(2);

/// [`base`] with player 1's Hideout (2) inside the Tunnel's Tier 1 area (tiles (10, 8) to (17, 15)), on tiles
/// (16, 11) to (17, 12): its centre is (1088, 768).
fn hideout_base() -> ScenarioBuilder {
    base().spawn(HIDEOUT, Some(P1), tile(16, 11))
}

fn production(producer: ObjectId, command: ProductionCommand) -> Command {
    Command::Production {
        objects: vec![producer],
        command,
    }
}

/// Starts `scenario` with 1000 Space Crystals for player 1, no auto-targeting, and an Agent trained at `H1` in tick
/// 3, which is finished in tick 242 (240 frames).
fn training(scenario: ScenarioBuilder) -> TestSim {
    let mut t = scenario.start();
    t.without_auto_targeting()
        .debug_at(
            1,
            DebugCommand::SetResources {
                player: P1,
                crystals: 1000,
                supplies: 0,
                alloys: 0,
                essence: 0,
                flux: 0,
            },
        )
        .at(3, P1, production(H1, ProductionCommand::Train(AGENT)));
    t
}

const DONE: Tick = 242;

fn produced(t: &TestSim) -> Vec<(Tick, ObjectId)> {
    let events = t.events().iter();
    events
        .filter_map(|(tick, e)| match e {
            SimEvent::UnitProduced { producer, unit } if *producer == H1 => Some((*tick, *unit)),
            _ => None,
        })
        .collect()
}

/// The plan's acceptance ("With no rally, a trained Agent appears in the network") and design change 0246 ("Units
/// produced inside the network have no delay: they can exit as soon as they are produced"). The Agent costs 100
/// Space Crystals and takes 240 frames; it never appears at the Hideout.
#[test]
fn a_trained_agent_appears_in_the_network_and_can_exit_at_once() {
    let mut t = training(hideout_base());
    run_checked(&mut t, 3);
    assert_eq!(t.sim().player(P1).unwrap().crystals, 900);
    run_checked(&mut t, DONE - 1);
    assert!(produced(&t).is_empty() && inside(&t).is_empty());
    let objects = t.sim().objects().count();
    t.at(DONE + 1, P1, eject(T1, AGENT));
    run_checked(&mut t, DONE);
    let agent = id(3);
    assert_eq!(produced(&t), [(DONE, agent)]);
    assert_eq!(t.sim().objects().count(), objects, "nothing on the map");
    let held = t.sim().held_unit(agent).expect("in the network");
    assert_eq!((held.place, held.since, held.entered), (Place::Network, DONE, false));
    assert_eq!(
        (held.object.kind, held.object.owner, held.object.hp),
        (AGENT, Some(P1), Some(75.0))
    );
    assert!(!t.events().iter().any(|(_, e)| *e == SimEvent::Spawned(agent)));
    assert!(t.sim().object(T1).unwrap().tunnel.as_ref().unwrap().eject.is_empty());
    // Out in the very next tick, by a player's Eject.
    run_checked(&mut t, DONE + 1);
    assert_eq!(exits(&t, T1), [(DONE + 1, agent)]);
    assert_eq!(t.pos(3), EXIT);
}

/// `structures/README.md > Rally Point Behavior`: "Rally point set on the surface: Unit auto-ejects from the parent
/// Tunnel (Side A) and moves to the rally point." It comes out of the exit path in the tick it is finished, and walks there.
#[test]
fn a_surface_rally_sends_the_unit_out_of_the_parent_tunnel() {
    let rally = Vec2::new(500.0, 300.0);
    let mut t = training(hideout_base());
    t.at(2, P1, production(H1, ProductionCommand::SetRally(Target::Point(rally))));
    run_checked(&mut t, DONE);
    let agent = id(3);
    assert_eq!(exits(&t, T1), [(DONE, agent)]);
    assert_eq!(t.pos(3), EXIT, "on the exit spot of the parent Tunnel");
    assert!(inside(&t).is_empty());
    run_checked(&mut t, DONE + 120);
    assert_eq!(t.pos(3), rally);
    assert!(t.sim().object(agent).unwrap().orders.as_ref().unwrap().is_idle());
}

/// "No rally point, or rally point set on the parent Tunnel: Unit stays in the Tunnel Network"; design change 0252's
/// Verification: "Rally on a Tunnel: the Guard stays inside", any own Tunnel's. A rally on an enemy's Tunnel is one on
/// the surface: the unit comes out to attack it.
#[test]
fn a_rally_on_an_own_tunnel_keeps_the_unit_inside() {
    for (target, stays) in [(T1, true), (id(3), true), (id(4), false)] {
        let mut t = training(
            hideout_base()
                .spawn(TUNNEL, Some(P1), tile(22, 2))
                .spawn(TUNNEL, Some(P2), tile(6, 10)),
        );
        t.at(
            2,
            P1,
            production(H1, ProductionCommand::SetRally(Target::Object(target))),
        );
        run_checked(&mut t, DONE);
        let agent = id(5);
        assert_eq!(produced(&t), [(DONE, agent)]);
        assert_eq!(inside(&t) == [agent], stays, "rallied on {target:?}");
        assert_eq!(exits(&t, T1).is_empty(), stays, "rallied on {target:?}");
        if !stays {
            run_checked(&mut t, DONE + 1);
            let order = t.sim().object(agent).unwrap().orders.clone().unwrap().current.unwrap();
            assert_eq!(order.order.kind, OrderKind::Attack(target));
        }
    }
}

/// Design change 0252's Verification: "Rally a Hideout on another Hideout. The produced Guard comes out of the
/// parent Tunnel and stands over the other Hideout": the surface spot above it, not an object target.
#[test]
fn a_rally_on_an_expansion_is_the_surface_spot_above_it() {
    // Another Hideout (3) of player 1's on tiles (10, 14) to (11, 15), in the same area: its centre is (704, 960).
    let mut t = training(hideout_base().spawn(HIDEOUT, Some(P1), tile(10, 14)));
    t.at(
        2,
        P1,
        production(H1, ProductionCommand::SetRally(Target::Object(id(3)))),
    );
    run_checked(&mut t, DONE);
    let agent = id(4);
    assert_eq!(exits(&t, T1), [(DONE, agent)]);
    run_checked(&mut t, DONE + 1);
    let order = t.sim().object(agent).unwrap().orders.clone().unwrap().current.unwrap();
    assert_eq!(
        order.order.kind,
        OrderKind::Move(Target::Point(Vec2::new(704.0, 960.0)))
    );
    run_checked(&mut t, DONE + 160);
    assert_eq!(t.pos(4), Vec2::new(704.0, 960.0), "it stands over the other Hideout");
}

/// Design change 0253: "Auto-ejected units [...] join the parent Tunnel's exit queue in order with manual ejects,
/// first come, first served. While waiting they don't appear in the EjectMenu."
#[test]
fn auto_ejected_units_share_the_exit_queue_with_manual_ejects() {
    let rally = Vec2::new(500.0, 300.0);
    let mut t = training(hideout_base());
    t.at(2, P1, production(H1, ProductionCommand::SetRally(Target::Point(rally))))
        // A Guard goes in early, and a unit of player 2's blocks the exit spot before the Agent is finished.
        .spawn_unit_at(1, GUARD, P1, Vec2::new(866.0, 560.0)) // 3
        .at(2, P1, enter(&[id(3)], T1))
        .spawn_unit_at(200, GUARD, P2, EXIT) // 4
        .at(DONE - 2, P1, eject(T1, GUARD));
    run_checked(&mut t, DONE + 20);
    let (guard, agent) = (id(3), id(5));
    let queue: Vec<ObjectId> = (t.sim().object(T1).unwrap().tunnel.as_ref().unwrap().eject.iter())
        .map(|entry| entry.unit)
        .collect();
    assert_eq!(queue, [guard, agent], "first come, first served");
    assert_eq!(exits(&t, T1), []);
    assert_eq!(inside(&t), [guard, agent]);
    assert!(t.sim().network_counts(P1).is_empty(), "neither shows in the EjectMenu");
    // The blocker leaves: the Guard comes out first, then the Agent, which walks to the rally point.
    let away = OrderKind::Move(Target::Point(Vec2::new(1400.0, 500.0)));
    t.order_at(DONE + 21, P2, &[id(4)], away, false);
    run_checked(&mut t, DONE + 60);
    let out: Vec<ObjectId> = exits(&t, T1).iter().map(|&(_, unit)| unit).collect();
    assert_eq!(out, [guard, agent]);
    run_checked(&mut t, DONE + 200);
    assert_eq!(t.pos(agent.0), rally);
    assert_eq!(
        t.pos(guard.0),
        EXIT + Vec2::new(0.0, -64.0),
        "the ejected Guard walked straight out"
    );
}

/// Plan decision T32: the parent Tunnel is derived, "the owner's Tunnel whose current area fully contains" the
/// expansion. A Hideout outside every area has none: its unit stays in the network, whatever its rally.
#[test]
fn a_hideout_without_a_parent_tunnel_releases_into_the_network() {
    // The Hideout at tiles (18, 11) to (19, 12) lies outside the Tier 1 area, which ends with tile 17.
    let mut t = training(base().spawn(HIDEOUT, Some(P1), tile(18, 11)));
    t.at(
        2,
        P1,
        production(H1, ProductionCommand::SetRally(Target::Point(Vec2::new(500.0, 300.0)))),
    );
    run_checked(&mut t, 2);
    assert_eq!(t.sim().parent_tunnel(H1), None);
    run_checked(&mut t, DONE);
    assert_eq!(inside(&t), [id(3)]);
    assert_eq!(exits(&t, T1), []);
    // At Tier 3 the area reaches from tile 8 to tile 19: the Tunnel is its parent.
    t.debug_at(DONE + 1, DebugCommand::SetTier { id: T1, tier: 3 });
    run_checked(&mut t, DONE + 1);
    assert_eq!(t.sim().parent_tunnel(H1), Some(T1));
}
