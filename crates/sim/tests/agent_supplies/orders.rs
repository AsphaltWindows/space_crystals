//! Orders on a station (`factions/syndicate/units/agent.md > Unit Commands`: "Gather — right-click Space Crystals
//! patch or Supply Delivery Station"; `factions/syndicate/units/agent.md > Right-Click Resolution`: "Supply Delivery
//! Station → Gather supplies"): the gate, the default order, a Hideout's rally on a station
//! (`factions/syndicate/structures/README.md > Rally Point Behavior`: "the unit's default right-click action at the
//! rally target"), and a Supply carried through the Tunnel Network (design change 0249: "An Agent keeps its load").

use sc_sim::defs::TRANSIT_DELAY_TICKS;
use sc_sim::{Command, DebugCommand, PlayerCommand, ProductionCommand, Reject, Target, TunnelCommand};

use super::*;

fn gather_order(player: PlayerId, units: &[u32], target: u32) -> PlayerCommand {
    PlayerCommand {
        player,
        command: Command::Order {
            units: units.iter().map(|&n| id(n)).collect(),
            kind: OrderKind::Gather(id(target)),
            queue: false,
        },
    }
}

fn default_order(t: &TestSim, n: u32, target: u32) -> Option<OrderKind> {
    t.sim().default_order(id(n), Target::Object(id(target)))
}

/// The gate: a Gather on a station goes to the workers that gather Supplies among the units, the Agents; the others
/// ignore it, and with none it is rejected (`Reject::NoWorkers`), for a Guard and for a Drone alike.
#[test]
fn a_gather_on_a_station_goes_to_the_agents_among_the_units() {
    let mut t = flat(30, 20)
        .faction(P2, Faction::Colonists)
        .spawn(TUNNEL, Some(P1), tile(12, 6)) // 1
        .spawn_with(station_of(10), tile(7, 7)) // 2
        .spawn(AGENT, Some(P1), centre(8, 12)) // 3
        .spawn(ObjectKind::Guard, Some(P1), centre(10, 12)) // 4
        .spawn(ObjectKind::Drone, Some(P2), centre(4, 14)) // 5
        .start();
    t.run(1);
    let sim = t.sim();
    assert_eq!(
        sim.validate(&gather_order(P1, &[4], STATION)),
        Err(Reject::NoWorkers),
        "a Guard"
    );
    assert_eq!(
        sim.validate(&gather_order(P2, &[5], STATION)),
        Err(Reject::NoWorkers),
        "a Drone gathers no Supplies"
    );
    assert_eq!(sim.validate(&gather_order(P1, &[3, 4], STATION)), Ok(()));
    t.at(2, P1, gather_order(P1, &[3, 4], STATION).command);
    t.run(2);
    assert_eq!(order(&t, 3), Some(OrderKind::Gather(id(STATION))));
    assert_eq!(order(&t, 4), None, "the Guard ignores it");
}

/// The default order on a station, in a game: Gather for an Agent, empty, with crystals (design change 0256 point 4:
/// it "goes straight to the station") and with a Supply; a Move to it for a Guard. On its own Tunnel, an Agent with a
/// Supply gets Drop Off ("Own Tunnel (carrying resources) → Drop off resources"), and on a patch, Gather.
#[test]
fn an_agents_default_order_on_a_station_is_gather_whatever_it_carries() {
    let mut t = flat(30, 20)
        .spawn(TUNNEL, Some(P1), tile(12, 6)) // 1
        .spawn_with(station_of(10), tile(7, 7)) // 2
        .spawn_with(patch_of(10_000), tile(18, 8)) // 3
        .spawn(AGENT, Some(P1), centre(17, 11)) // 4
        .spawn(ObjectKind::Guard, Some(P1), centre(10, 12)) // 5
        .start();
    t.run(1);
    let gather_station = Some(OrderKind::Gather(id(STATION)));
    assert_eq!(default_order(&t, 4, STATION), gather_station, "empty");
    assert_eq!(
        default_order(&t, 5, STATION),
        Some(OrderKind::Move(Target::Object(id(STATION)))),
        "a Guard"
    );
    gather(&mut t, 2, &[4], 3);
    run_to(&mut t, &[4], &[STATION], 300, "crystals", |t| load(t, 4).0 > 0);
    assert_eq!(default_order(&t, 4, STATION), gather_station, "with crystals");
    let next = t.sim().tick() + 1;
    gather(&mut t, next, &[4], STATION);
    run_to(&mut t, &[4], &[STATION], 400, "a Supply", |t| {
        load(t, 4) == (CARRY, Resource::Supplies)
    });
    assert_eq!(default_order(&t, 4, STATION), gather_station, "with a Supply");
    assert_eq!(default_order(&t, 4, T1), Some(OrderKind::DropOff(id(T1))));
    assert_eq!(default_order(&t, 4, 3), Some(OrderKind::Gather(id(3))));
}

/// A Hideout's rally on a station: "the unit's default right-click action at the rally target" (design change 0252,
/// Q0064 point 7). The Agent trained comes out of the parent Tunnel with a Gather on the station, picks up a Supply
/// and delivers it at the Supplies hatch.
#[test]
fn an_agent_rallied_on_a_station_comes_out_and_gathers_supplies() {
    // M4-C's layout (`tunnel_network::production`): the Tunnel (1) on tiles (12, 10) to (15, 13), its Supplies hatch
    // on tile (12, 12); the Hideout (2) inside its area on tile (16, 11); a station (3) on tile (7, 12), 3 grid units
    // west of the Supplies hatch.
    let mut t = flat(30, 22)
        .spawn(TUNNEL, Some(P1), tile(12, 10))
        .spawn(ObjectKind::Hideout, Some(P1), tile(16, 11))
        .spawn_with(station_of(10), tile(7, 12))
        .start();
    let (tunnel, hideout, station) = (1, 2, 3);
    let production = |command| Command::Production {
        objects: vec![id(hideout)],
        command,
    };
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
        .at(
            2,
            P1,
            production(ProductionCommand::SetRally(Target::Object(id(station)))),
        )
        .at(3, P1, production(ProductionCommand::Train(AGENT)));
    let agent = 4;
    let produced = run_to(&mut t, &[], &[station], 400, "the Agent", |t| {
        t.sim().object(id(agent)).is_some()
    });
    t.run(1);
    assert_eq!(
        order(&t, agent),
        Some(OrderKind::Gather(id(station))),
        "out at tick {produced}"
    );
    let delivered = run_to(&mut t, &[agent], &[station], 600, "the delivery", |t| {
        !deliveries(t).is_empty()
    });
    assert_eq!(deliveries(&t), [(delivered, agent, tunnel, CARRY, Resource::Supplies)]);
    assert_eq!(banks(&t, P1).1, 1);
    assert_eq!(supplies(&t, station), 9);
}

/// Design change 0249, "An Agent keeps its load", with a Supply: an Agent that picked one up walks into its Tunnel
/// (Enter, given in place of the Drop Off a right-click gives), has it inside the network and when it comes out, and
/// then delivers it at the Supplies hatch.
#[test]
fn an_agent_keeps_a_supply_through_the_network() {
    let mut t = field(1, 3, station_of(10)).start();
    t.without_auto_targeting();
    gather(&mut t, 1, &[3], STATION);
    run_to(&mut t, &[3], &[STATION], 200, "a Supply", |t| load(t, 3).0 > 0);
    assert_eq!(default_order(&t, 3, T1), Some(OrderKind::DropOff(id(T1))), "carrying");
    let next = t.sim().tick() + 1;
    t.order_at(next, P1, &[id(3)], OrderKind::Enter(id(T1)), false);
    let entered = run_to(&mut t, &[], &[STATION], 300, "it walks in", |t| {
        t.sim().held_unit(id(3)).is_some()
    });
    let held = t.sim().held_unit(id(3)).expect("inside");
    let worker = held.object.worker.expect("a worker");
    assert_eq!((worker.load, worker.resource), (CARRY, Resource::Supplies), "inside");
    let eject = Command::Tunnel {
        tunnel: id(T1),
        command: TunnelCommand::Eject { kind: AGENT },
    };
    t.at(entered + TRANSIT_DELAY_TICKS, P1, eject);
    run_to(&mut t, &[], &[STATION], TRANSIT_DELAY_TICKS + 10, "out again", |t| {
        t.sim().object(id(3)).is_some()
    });
    assert_eq!(load(&t, 3), (CARRY, Resource::Supplies), "out again");
    assert!(deliveries(&t).is_empty());
    let before = banks(&t, P1);
    let next = t.sim().tick() + 1;
    t.order_at(next, P1, &[id(3)], OrderKind::DropOff(id(T1)), false);
    let delivered = run_to(&mut t, &[3], &[STATION], 400, "the delivery", |t| {
        !deliveries(t).is_empty()
    });
    assert_eq!(deliveries(&t), [(delivered, 3, T1, CARRY, Resource::Supplies)]);
    assert_eq!(banks(&t, P1), (before.0, before.1 + 1));
}
