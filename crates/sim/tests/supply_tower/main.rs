//! The GDO's supply loop (`sc_sim::supply_run`; `factions/gdo/economy.md > SupplyTower`,
//! `factions/gdo/units/supply_chopper.md`, `core/map.md > SupplyDeliveryStation`), through the gate. One test binary:
//! - `placement`: the Deployment Center constructs the Supply Tower once a Radar Station stands, and placing it brings
//!   its free Supply Chopper, down on it and attached, at no cost; the Tower trains more, 100 Space Crystals and 160
//!   frames each, over its centre;
//! - `runs`: a Pick Up Supplies lands the chopper on the station and takes all its Supplies, a Drop Off Supplies pays
//!   them in at a Tower, and the right-click gives each; the gate's refusals;
//! - `schedule`: a scheduled run is down on its station as the delivery comes, and back on its Tower, the bank up by
//!   the delivery; too far or too frequent, the chopper leaves as soon as it is home; two Towers on one station take
//!   turns, the closer first;
//! - `attachment`: the landing rules: an attaching chopper evicts the one on its Tower, a command breaks the
//!   attachment, the chopper is an air target while it lands and a ground one down; repair while landed, free, at the
//!   repairers' rate and slower at low Power;
//! - `sharing`: Agents and a chopper share a station by its pick-ups' claims (`factions/syndicate/units/agent.md >
//!   Supplies`);
//! - `supplies`: the GDO's Supplies: 1 at the start (`factions/gdo/economy.md > Starting Conditions`), and those a run
//!   brings, which train the units that cost Supplies.
//!
//! Every test runs under the invariants of `TestSim`, L1's chopper arm among them (a chopper on a spot is within reach
//! of it, one chopper on a spot at a time); auto-targeting is off, so that only orders fight.

mod attachment;
mod placement;
mod runs;
mod schedule;
mod sharing;
mod supplies;

use sc_sim::landing::{self, Landing};
use sc_sim::supply_run;
use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{
    Command, ConstructionCommand, DebugCommand, Object, ObjectId, ObjectKind, OrderKind, PlayerCommand, PlayerId,
    Reject, ResourceState, SimEvent, SpawnAt, SpawnSettings, Tick, TilePos, Vec2,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const TOWER: ObjectKind = ObjectKind::SupplyTower;
const CHOPPER: ObjectKind = ObjectKind::SupplyChopper;

/// `factions/gdo/units/supply_chopper.md > Unit Transport`: "**Landing and taking off take 16 frames (1 s) each** (as
/// at the Airfield)".
const LANDING: Tick = 16;
/// `factions/gdo/structures/deployment_center.md > Constructs`: "Supply Tower: 150 Space Crystals, 320 frames".
const TOWER_COST: u32 = 150;
const TOWER_TICKS: Tick = 320;
/// `factions/gdo/economy.md > SupplyTower > Produces`: "Supply Chopper: 100 Space Crystals, 160 frames".
const CHOPPER_COST: u32 = 100;
const CHOPPER_TICKS: Tick = 160;
/// `core/map.md > SupplyDeliveryStation`: DeliverySize.
const DELIVERY_SIZE: u32 = 4;
/// How near its spot a chopper down on it is: within the reach it lands from (`Air::landing_reach_su`).
const ON_SPOT: f32 = 8.0;

// The objects of [`base`], by id: 1 player 1's Deployment Center on tiles (2, 2) to (5, 5) (Power +30), 2 its
// Generator on tiles (8, 2) to (9, 3) (+20), 3 its Radar Station on tiles (8, 5) to (9, 6) (−20); a Tower placed costs
// 15 more. Player 1's Power is at full speed with two Towers.
const DC: u32 = 1;
/// The first id after [`base`]'s.
const FIRST: u32 = 4;

fn id(n: u32) -> ObjectId {
    ObjectId(n)
}

fn flat(w: usize, h: usize) -> String {
    vec![".".repeat(w); h].join("\n")
}

fn tile(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Tile(TilePos::new(x, y))
}

/// A Supply Delivery Station holding `supplies`, delivering [`DELIVERY_SIZE`] every `interval` ticks.
fn station(supplies: u32, interval: Tick) -> SpawnSettings {
    SpawnSettings::SupplyDeliveryStation {
        starting_supplies: supplies,
        delivery_size: DELIVERY_SIZE,
        delivery_interval: interval,
    }
}

/// Player 1 plays the GDO on a flat map of 64 × 40 tiles, with [`base`]'s objects and `more` after them (from id
/// [`FIRST`]), and 5000 Space Crystals from tick 1, which has run; player 2's Deployment Center is far off, on tiles
/// (58, 34) to (61, 37), the last of the spawns; auto-targeting off.
fn base(more: impl FnOnce(ScenarioBuilder) -> ScenarioBuilder) -> TestSim {
    let builder = ScenarioBuilder::new(&flat(64, 40))
        .spawn(ObjectKind::DeploymentCenter, Some(P1), tile(2, 2))
        .spawn(ObjectKind::Generator, Some(P1), tile(8, 2))
        .spawn(ObjectKind::RadarStation, Some(P1), tile(8, 5));
    let mut t = more(builder)
        .spawn(ObjectKind::DeploymentCenter, Some(P2), tile(58, 34))
        .start();
    t.without_auto_targeting();
    t.debug_at(1, fund(5000, 0));
    t.run_until(1);
    t
}

/// Player 1's banks set to `crystals` and `supplies`.
fn fund(crystals: u32, supplies: u32) -> DebugCommand {
    DebugCommand::SetResources {
        player: P1,
        crystals,
        supplies,
        alloys: 0,
        essence: 0,
        flux: 0,
    }
}

/// What the gate says to player 1's `command` now (`Sim::validate`).
fn gate(t: &TestSim, command: &Command) -> Result<(), Reject> {
    gate_of(t, P1, command)
}

fn gate_of(t: &TestSim, player: PlayerId, command: &Command) -> Result<(), Reject> {
    t.sim().validate(&PlayerCommand {
        player,
        command: command.clone(),
    })
}

/// Player 1's `command` in the next tick, which runs; the gate takes it. The tick.
fn next(t: &mut TestSim, command: Command) -> Tick {
    assert_eq!(gate(t, &command), Ok(()), "{command:?}");
    let tick = t.sim().tick() + 1;
    t.at(tick, P1, command);
    t.step();
    tick
}

/// The debug `command` in the next tick, which runs.
fn debug(t: &mut TestSim, command: DebugCommand) {
    let tick = t.sim().tick() + 1;
    t.debug_at(tick, command);
    t.step();
}

fn order(units: &[u32], kind: OrderKind) -> Command {
    Command::Order {
        units: units.iter().copied().map(id).collect(),
        kind,
        queue: false,
    }
}

fn pick_up(units: &[u32], station: u32) -> Command {
    order(units, OrderKind::PickUpSupplies(id(station)))
}

fn drop_off(units: &[u32], tower: u32) -> Command {
    order(units, OrderKind::DropOffSupplies(id(tower)))
}

fn attach(units: &[u32], tower: u32) -> Command {
    order(units, OrderKind::AttachToTower(id(tower)))
}

fn schedule(towers: &[u32], station: u32) -> Command {
    Command::Schedule {
        towers: towers.iter().copied().map(id).collect(),
        station: id(station),
    }
}

/// The right-click of `units` on object `target` (`Sim::default_order`, the first unit's, as the client sends it).
fn right_click(t: &TestSim, units: &[u32], target: u32) -> Command {
    let kind = (t.sim().default_order(id(units[0]), sc_sim::Target::Object(id(target)))).expect("a default order");
    order(units, kind)
}

fn object(t: &TestSim, n: u32) -> &Object {
    t.sim().object(id(n)).unwrap_or_else(|| panic!("#{n} is on the map"))
}

/// Chopper `n`'s landing, if it is landing or down on a spot.
fn landing_of(t: &TestSim, n: u32) -> Option<Landing> {
    landing::of(object(t, n))
}

/// Whether chopper `n` is down on object `at`'s spot.
fn down_on(t: &TestSim, n: u32, at: u32) -> bool {
    landing_of(t, n).is_some_and(|landing| landing.at == id(at) && landing.descent.done())
}

/// The Supplies chopper `n` carries.
fn carried(t: &TestSim, n: u32) -> u32 {
    supply_run::carried(object(t, n))
}

/// The tower chopper `n` is attached to, by number.
fn attached(t: &TestSim, n: u32) -> Option<u32> {
    supply_run::attached_to(object(t, n)).map(|tower| tower.0)
}

/// The kind of what unit `n` is doing: its current order's, else its first queued one's.
fn doing(t: &TestSim, n: u32) -> Option<OrderKind> {
    let orders = object(t, n).orders.as_ref().expect("orders");
    (orders.current.as_ref().map(|active| &active.order))
        .or(orders.queue.front())
        .map(|order| order.kind)
}

/// The Supplies station `n` holds.
fn held(t: &TestSim, n: u32) -> u32 {
    match object(t, n).resource {
        Some(ResourceState::SupplyStation { current_supplies, .. }) => current_supplies,
        _ => panic!("#{n} is no station"),
    }
}

/// Player `p`'s banks: Space Crystals and Supplies.
fn banks(t: &TestSim, p: PlayerId) -> (u32, u32) {
    let player = t.sim().player(p).expect("in the game");
    (player.crystals, player.supplies)
}

/// Object `n`'s spot: a Tower's or a station's (`supply_run::spot`).
fn spot(t: &TestSim, n: u32) -> Vec2 {
    supply_run::spot(object(t, n)).unwrap_or_else(|| panic!("#{n} has no spot"))
}

/// Steps until `done` holds, at most `ticks` ticks; the tick it held.
fn step_until(t: &mut TestSim, ticks: Tick, what: &str, done: impl Fn(&TestSim) -> bool) -> Tick {
    let until = t.sim().tick() + ticks;
    while !done(t) {
        assert!(t.sim().tick() < until, "{what}: not by tick {until}");
        t.step();
    }
    t.sim().tick()
}

/// The objects spawned so far, in order: (tick, id, kind).
fn spawned(t: &TestSim) -> Vec<(Tick, u32, ObjectKind)> {
    (t.events().iter())
        .filter_map(|(tick, e)| match e {
            SimEvent::Spawned(n) => Some((*tick, n.0, t.sim().object(*n)?.kind)),
            _ => None,
        })
        .collect()
}

/// The Deployment Center constructs a Supply Tower and places it with its footprint's corner on `origin`: the Tower's
/// id and its free chopper's.
fn placed_tower(t: &mut TestSim, origin: TilePos) -> (u32, u32) {
    let construct = Command::Construction {
        objects: vec![id(DC)],
        command: ConstructionCommand::Construct(TOWER),
    };
    next(t, construct);
    step_until(t, 2 * TOWER_TICKS, "the Tower ready", |t| {
        (object(t, DC).constructor).is_some_and(|state| state.ready() == Some(TOWER))
    });
    let place = Command::Construction {
        objects: vec![id(DC)],
        command: ConstructionCommand::Place {
            origin,
            orientation: Default::default(),
        },
    };
    let tick = next(t, place);
    let new: Vec<(u32, ObjectKind)> = (spawned(t).into_iter())
        .filter(|&(at, ..)| at == tick)
        .map(|(_, n, kind)| (n, kind))
        .collect();
    match new[..] {
        [(tower, TOWER), (chopper, CHOPPER)] => (tower, chopper),
        _ => panic!("placed: {new:?}"),
    }
}
