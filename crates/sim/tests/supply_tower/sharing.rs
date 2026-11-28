//! Agents and a chopper at one station (`economy/stations.rs`'s claims, `factions/syndicate/units/agent.md >
//! Supplies`; `factions/gdo/units/supply_chopper.md > Supply Pickup`).

use sc_sim::defs::Resource;
use sc_sim::orders::state::{Behaviour, GatherPhase};
use sc_sim::{Faction, Target};

use super::*;

/// Player 2 plays the Syndicate: their Tunnel on tiles (18, 12) to (21, 15) and their Agent on tile (10, 12); the
/// station, holding 4, on tiles (14, 8) to (15, 9).
fn field() -> (TestSim, u32, u32) {
    let builder = ScenarioBuilder::new(&flat(64, 40))
        .faction(P2, Faction::Syndicate)
        .spawn(ObjectKind::DeploymentCenter, Some(P1), tile(2, 2))
        .spawn(ObjectKind::Generator, Some(P1), tile(8, 2))
        .spawn(ObjectKind::RadarStation, Some(P1), tile(8, 5))
        .spawn_with(station(4, 960), tile(14, 8))
        .spawn(ObjectKind::Tunnel, Some(P2), tile(18, 12))
        .spawn(
            ObjectKind::Agent,
            Some(P2),
            SpawnAt::Point(TilePos::new(10, 12).center()),
        );
    let mut t = builder.start();
    t.without_auto_targeting();
    t.debug_at(1, fund(5000, 0));
    t.run_until(1);
    let (_, chopper) = placed_tower(&mut t, TilePos::new(2, 8));
    (t, chopper, FIRST + 2)
}

const S: u32 = FIRST;

/// Whether Agent `n` is picking up at the station: its timed work there.
fn picking_up(t: &TestSim, n: u32) -> bool {
    let orders = object(t, n).orders.as_ref().expect("a unit");
    let current = orders.current.as_ref();
    current.is_some_and(|active| {
        active.order.kind == OrderKind::Gather(id(S))
            && matches!(active.state, Behaviour::Gather(gather) if matches!(gather.phase, GatherPhase::Mining { .. }))
    })
}

/// The plan's "Sharing": player 2's Agent starts its 48-frame pick-up of one Supply, which claims it; player 1's
/// chopper, at rest over the station, is ordered to pick up then, and down 16 frames later takes the 3 no one has
/// claimed, the station keeping the Agent's; the Agent's pick-up ends with its Supply, which empties the station.
#[test]
fn agents_and_a_chopper_share_a_station_by_its_claims() {
    let (mut t, chopper, agent) = field();
    let over = spot(&t, S);
    next(&mut t, order(&[chopper], OrderKind::Move(Target::Point(over))));
    step_until(&mut t, 400, "over the station", |t| {
        t.status(chopper) != sc_sim::MoveStatus::Moving && t.pos(chopper).distance(over) <= ON_SPOT
    });
    let gather = order(&[agent], OrderKind::Gather(id(S)));
    assert_eq!(gate_of(&t, P2, &gather), Ok(()));
    let tick = t.sim().tick() + 1;
    t.at(tick, P2, gather);
    step_until(&mut t, 400, "the Agent picks up", |t| picking_up(t, agent));
    next(&mut t, pick_up(&[chopper], S));
    step_until(&mut t, 2 * LANDING, "the chopper loaded", |t| carried(t, chopper) > 0);
    assert!(picking_up(&t, agent), "still at it");
    assert_eq!((carried(&t, chopper), held(&t, S)), (3, 1), "the Agent's claim stays");
    step_until(&mut t, 60, "the Agent loaded", |t| {
        object(t, agent).worker.is_some_and(|w| w.load == 1)
    });
    let worker = object(&t, agent).worker.expect("a worker");
    assert_eq!((worker.load, worker.resource, held(&t, S)), (1, Resource::Supplies, 0));
}
