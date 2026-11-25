//! Agents gather Supplies (slice M4-I; `factions/syndicate/units/agent.md > Gathering > Supplies`, design changes
//! 0248, 0255 and 0256; `core/map.md > SupplyDeliveryStation`; `factions/syndicate/tunnels.md > Tunnel > SymmetryType -
//! ABCD`): the Gather job of slice M4-D at a Supply Delivery Station, the delivery at the Supplies hatch on Side D, and
//! design change 0256's rules. `TestSim` checks the invariants after every tick, and [`step`] checks the station's
//! claims too: no pick-up runs without a Supply of its own.
//!
//! In modules by topic, one test binary:
//! - `trips`: one Agent empties a station of 4 in four trips, the countdown runs, 4 Supplies arrive again, and the bank
//!   gains 4 (the slice's acceptance); two Agents at one station at once;
//! - `station`: 0256's Verification: several Agents picking up at once, a Supply leaving only as its pick-up ends, the
//!   empty station's wait and its event, for every player gathering there;
//! - `hatches`: a crystal delivery at B and a Supplies delivery at D in the same ticks, one Agent at a time at D, and
//!   the nearest Tunnel by its Supplies hatch (Placeholder: Q0082 point 2);
//! - `switching`: the load lost on switching resources, both ways, an interrupted pick-up that keeps it (Placeholder:
//!   Q0082 point 3), and a Supply carried to a station;
//! - `orders`: the gate and the default order on a station, a rally on a station, a Supply carried through the Tunnel
//!   Network and delivered by a Drop Off order.
//!
//! The stations of the trips stand 3 grid units from the Supplies hatch, or more: closer, Agents jam head-on there
//! (question Q0076).

mod hatches;
mod orders;
mod station;
mod switching;
mod trips;

use sc_sim::defs::Resource;
use sc_sim::economy::drop_off::drop_point_of;
use sc_sim::orders::state::{Behaviour, GatherPhase};
use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{
    Faction, Object, ObjectId, ObjectKind, OrderKind, PlayerId, ResourceState, Shape, SimEvent, SpawnAt, SpawnSettings,
    Tick, TilePos,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const AGENT: ObjectKind = ObjectKind::Agent;
const TUNNEL: ObjectKind = ObjectKind::Tunnel;
/// `factions/syndicate/units/agent.md > Gathering > Supplies`: CarryCapacity, PickUpDuration, DropOffDuration, and the
/// reaches (Placeholder: Q0082 point 1).
const CARRY: u32 = 1;
const PICK_UP: Tick = 48;
const DROP_OFF: Tick = 48;
const REACH: f32 = 8.0;
/// `core/map.md > SupplyDeliveryStation`: DeliverySize and DeliveryInterval.
const DELIVERY_SIZE: u32 = 4;
const DELIVERY_INTERVAL: Tick = 960;

/// A flat map on which player 1 plays the Syndicate.
fn flat(w: usize, h: usize) -> ScenarioBuilder {
    ScenarioBuilder::new(&vec![".".repeat(w); h].join("\n")).faction(P1, Faction::Syndicate)
}

fn tile(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Tile(TilePos::new(x, y))
}

fn centre(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Point(TilePos::new(x, y).center())
}

fn id(n: u32) -> ObjectId {
    ObjectId(n)
}

/// A station holding `supplies`, with the design's deliveries (DeliverySize 4, DeliveryInterval 960).
fn station_of(supplies: u32) -> SpawnSettings {
    station_with(supplies, DELIVERY_SIZE, DELIVERY_INTERVAL)
}

fn station_with(supplies: u32, delivery_size: u32, delivery_interval: Tick) -> SpawnSettings {
    SpawnSettings::SupplyDeliveryStation {
        starting_supplies: supplies,
        delivery_size,
        delivery_interval,
    }
}

fn patch_of(amount: u32) -> SpawnSettings {
    SpawnSettings::CrystalPatch {
        starting_amount: amount,
    }
}

/// The reference layout of the trips: player 1's Tunnel (1), unturned on tiles (12, 6) to (15, 9), Side A to the north,
/// its crystal hatch on tile (15, 8) (Side B, east) and its Supplies hatch on tile (12, 8) (Side D, west); a station
/// (2) holding `supplies`, `distance` grid units west of the Supplies hatch, edge to edge, in rows 7 and 8; and the
/// `agents` in a row south of the Tunnel, from id 3.
fn field(agents: u32, distance: i32, supplies: SpawnSettings) -> ScenarioBuilder {
    let mut builder = flat(30, 20)
        .spawn(TUNNEL, Some(P1), tile(12, 6))
        .spawn_with(supplies, tile(10 - distance, 7));
    for i in 0..agents as i32 {
        builder = builder.spawn(AGENT, Some(P1), centre(8 + i, 12));
    }
    builder
}

/// The ids of the Agents of [`field`].
fn agents(count: u32) -> Vec<u32> {
    (3..3 + count).collect()
}

/// The Tunnel and the station of [`field`].
const T1: u32 = 1;
const STATION: u32 = 2;

fn object(t: &TestSim, n: u32) -> &Object {
    t.sim().object(id(n)).expect("exists")
}

/// The Supplies station `n` holds.
fn supplies(t: &TestSim, n: u32) -> u32 {
    match object(t, n).resource {
        Some(ResourceState::SupplyStation { current_supplies, .. }) => current_supplies,
        _ => panic!("object {n} is no station"),
    }
}

/// The ticks until station `n`'s next delivery, while it is empty.
fn countdown(t: &TestSim, n: u32) -> Option<u32> {
    match object(t, n).resource {
        Some(ResourceState::SupplyStation { next_delivery_in, .. }) => next_delivery_in,
        _ => panic!("object {n} is no station"),
    }
}

/// Player `p`'s banks: Space Crystals and Supplies.
fn banks(t: &TestSim, p: PlayerId) -> (u32, u32) {
    let player = t.sim().player(p).expect("in the game");
    (player.crystals, player.supplies)
}

/// What worker `n` carries: how much, and of what.
fn load(t: &TestSim, n: u32) -> (u32, Resource) {
    let worker = object(t, n).worker.expect("a worker");
    (worker.load, worker.resource)
}

/// The order a unit is carrying out.
fn order(t: &TestSim, n: u32) -> Option<OrderKind> {
    let orders = object(t, n).orders.as_ref().expect("a unit");
    orders.current.as_ref().map(|active| active.order.kind)
}

/// The phase of a worker's job, if it has one.
fn phase(t: &TestSim, n: u32) -> Option<GatherPhase> {
    let orders = object(t, n).orders.as_ref().expect("a unit");
    match orders.current.as_ref()?.state {
        Behaviour::Gather(gather) => Some(gather.phase),
        _ => None,
    }
}

/// The drop-off a worker's job heads for or unloads at.
fn node(t: &TestSim, n: u32) -> Option<u32> {
    let orders = object(t, n).orders.as_ref()?;
    match orders.current.as_ref()?.state {
        Behaviour::Gather(gather) => gather.node.map(|node| node.0),
        _ => None,
    }
}

/// Whether worker `n` is at its timed work: mining, or picking up at a station.
fn is_picking_up(t: &TestSim, n: u32) -> bool {
    matches!(phase(t, n), Some(GatherPhase::Mining { .. }))
}

fn is_unloading(t: &TestSim, n: u32) -> bool {
    matches!(phase(t, n), Some(GatherPhase::DroppingOff { .. }))
}

fn is_waiting(t: &TestSim, n: u32) -> bool {
    matches!(phase(t, n), Some(GatherPhase::Waiting { .. }))
}

fn gather(t: &mut TestSim, tick: Tick, agents: &[u32], target: u32) {
    let units: Vec<ObjectId> = agents.iter().map(|&n| id(n)).collect();
    t.order_at(tick, P1, &units, OrderKind::Gather(id(target)), false);
}

/// A delivery: its tick, the worker, the drop-off, the amount and what it was.
type Delivery = (Tick, u32, u32, u32, Resource);

/// The deliveries so far.
fn deliveries(t: &TestSim) -> Vec<Delivery> {
    t.events()
        .iter()
        .filter_map(|(tick, e)| match *e {
            SimEvent::ResourcesDelivered {
                worker,
                drop_off,
                amount,
                resource,
            } => Some((*tick, worker.0, drop_off.0, amount, resource)),
            _ => None,
        })
        .collect()
}

/// The deliveries of Supplies so far.
fn supply_deliveries(t: &TestSim) -> Vec<Delivery> {
    (deliveries(t).into_iter())
        .filter(|d| d.4 == Resource::Supplies)
        .collect()
}

/// The notifications of an empty station so far: tick, station, owner.
fn empties(t: &TestSim) -> Vec<(Tick, u32, PlayerId)> {
    t.events()
        .iter()
        .filter_map(|(tick, e)| match *e {
            SimEvent::SupplyStationEmpty { station, owner } => Some((*tick, station.0, owner)),
            _ => None,
        })
        .collect()
}

/// The stations' deliveries so far: tick, station, amount.
fn refills(t: &TestSim) -> Vec<(Tick, u32, u32)> {
    t.events()
        .iter()
        .filter_map(|(tick, e)| match *e {
            SimEvent::SuppliesDelivered { station, amount } => Some((*tick, station.0, amount)),
            _ => None,
        })
        .collect()
}

/// The hatch of Tunnel `n` that takes `resource`: its square.
fn hatch(t: &TestSim, n: u32, resource: Resource) -> Shape {
    let point = drop_point_of(object(t, n), resource).expect("a drop-off");
    assert!(point.side.is_some(), "a hatch");
    point.shape
}

/// The gap between unit `n` and object `m`.
fn gap(t: &TestSim, n: u32, m: u32) -> f32 {
    Shape::of(object(t, n)).gap(&Shape::of(object(t, m)))
}

/// The workers among `agents` whose pick-up at station `station` runs.
fn picking_up_at(t: &TestSim, agents: &[u32], station: u32) -> Vec<u32> {
    (agents.iter().copied())
        .filter(|&n| is_picking_up(t, n) && order(t, n) == Some(OrderKind::Gather(id(station))))
        .collect()
}

/// One tick, and the claims of design change 0256 point 2 ("A pick-up starts only if a Supply remains that no running
/// pick-up has claimed"; "A Supply leaves the station when the pick-up ends"): at the end of every tick, the pick-ups
/// running at each station of `stations` are no more than the Supplies it still holds, each of them a Supply of its
/// own.
fn step(t: &mut TestSim, agents: &[u32], stations: &[u32]) {
    t.step();
    for &station in stations {
        let running = picking_up_at(t, agents, station).len() as u32;
        let held = supplies(t, station);
        assert!(
            running <= held,
            "tick {}: {running} pick-ups at station {station}, which holds {held}",
            t.sim().tick()
        );
    }
}

/// Steps until `done` holds, at most `limit` ticks, checking the claims ([`step`]), and returns the tick it first held
/// in.
fn run_to(
    t: &mut TestSim,
    agents: &[u32],
    stations: &[u32],
    limit: Tick,
    what: &str,
    done: impl Fn(&TestSim) -> bool,
) -> Tick {
    for _ in 0..limit {
        step(t, agents, stations);
        if done(t) {
            return t.sim().tick();
        }
    }
    panic!("{what}: not within {limit} ticks\n{}", t.ascii(None));
}
