//! The Tunnel Network (slice M4-C; `factions/syndicate/tunnels.md > Tunnel Network`, `> Tunnel > SymmetryType - ABCD`,
//! `> EjectMenu`; `factions/syndicate/structures.md > Rally Point Behavior`; `systems/controls/commands.md > Enter`),
//! through the gate, one test binary:
//! - `enter`: the Enter order (the walk into the entry path, what leaving the map ends, the gate, the tiers, the
//!   Failure rule) and the units inside (design changes 0245 to 0249);
//! - `eject`: the Eject command and the exit path (design changes 0248, 0250 and 0251);
//! - `production`: the Hideout's units and their rally points (design changes 0252 and 0253);
//! - `crowds`: lines and crowds at Side A (design change 0246's Verification, the plan's 40 Guards);
//! - `saves`: a game with units inside replayed, saved and loaded, and the hash.
//!
//! Every test runs under the invariants of `TestSim` (no two units overlap, none stands in a footprint, no reference
//! to an object that left the map is left behind: `dangling`).

mod crowds;
mod eject;
mod enter;
mod production;
mod saves;

use sc_sim::network::Paths;
use sc_sim::testing::{ScenarioBuilder, TestSim, dangling_references};
use sc_sim::{
    Command, DebugCommand, Faction, ObjectId, ObjectKind, OrderKind, PlayerId, SimEvent, SpawnAt, Tick, TilePos,
    TunnelCommand, Vec2,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const AGENT: ObjectKind = ObjectKind::Agent;
const GUARD: ObjectKind = ObjectKind::Guard;
const TUNNEL: ObjectKind = ObjectKind::Tunnel;
const HIDEOUT: ObjectKind = ObjectKind::Hideout;

/// The Tunnel of [`base`].
const T1: ObjectId = ObjectId(1);

fn flat(w: usize, h: usize) -> String {
    vec![".".repeat(w); h].join("\n")
}

fn tile(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Tile(TilePos::new(x, y))
}

fn id(n: u32) -> ObjectId {
    ObjectId(n)
}

/// Both players play the Syndicate.
fn syndicate(map: &str) -> ScenarioBuilder {
    ScenarioBuilder::new(map)
        .faction(P1, Faction::Syndicate)
        .faction(P2, Faction::Syndicate)
}

/// Player 1's Tunnel (1) on a 30 × 22 map, unturned: it covers tiles (12, 10) to (15, 13), x 768..1024 and
/// y 640..896. Side A faces north (y = 640, its middle at x = 896), Side B east, Side D west: the exit path's spot
/// is (926, 640), the entry path's (866, 640).
fn base() -> ScenarioBuilder {
    syndicate(&flat(30, 22)).spawn(TUNNEL, Some(P1), tile(12, 10))
}

/// Where a Guard or an Agent (36 × 36) stands on [`base`]'s entry and exit paths.
const ENTRY: Vec2 = Vec2::new(866.0, 640.0 - 18.0 - 1.0 / 64.0);
const EXIT: Vec2 = Vec2::new(926.0, 640.0 - 18.0 - 1.0 / 64.0);

fn paths(t: &TestSim, tunnel: ObjectId) -> Paths {
    Paths::of_object(t.sim().object(tunnel).expect("the Tunnel")).expect("a Tunnel's paths")
}

fn enter(units: &[ObjectId], tunnel: ObjectId) -> Command {
    Command::Order {
        units: units.to_vec(),
        kind: OrderKind::Enter(tunnel),
        queue: false,
    }
}

fn eject(tunnel: ObjectId, kind: ObjectKind) -> Command {
    Command::Tunnel {
        tunnel,
        command: TunnelCommand::Eject { kind },
    }
}

fn spawn(kind: ObjectKind, owner: PlayerId, at: SpawnAt) -> DebugCommand {
    DebugCommand::Spawn {
        kind,
        owner: Some(owner),
        at,
        orientation: Default::default(),
    }
}

/// The ticks in which each unit went into a network, in order.
fn entries(t: &TestSim) -> Vec<(Tick, ObjectId)> {
    let events = t.events().iter();
    events
        .filter_map(|(tick, e)| match e {
            SimEvent::EnteredNetwork { unit, .. } => Some((*tick, *unit)),
            _ => None,
        })
        .collect()
}

/// The ticks in which each unit came out of `tunnel`, in order.
fn exits(t: &TestSim, tunnel: ObjectId) -> Vec<(Tick, ObjectId)> {
    let events = t.events().iter();
    events
        .filter_map(|(tick, e)| match e {
            SimEvent::LeftNetwork { unit, tunnel: from } if *from == tunnel => Some((*tick, *unit)),
            _ => None,
        })
        .collect()
}

/// The units whose Enter ended without them going in.
fn gave_up(t: &TestSim) -> Vec<ObjectId> {
    let events = t.events().iter();
    events
        .filter_map(|(_, e)| match e {
            SimEvent::EnterFailed { unit, .. } => Some(*unit),
            _ => None,
        })
        .collect()
}

/// The ids of player 1's units inside the network, in id order.
fn inside(t: &TestSim) -> Vec<ObjectId> {
    t.sim().network(P1).map(|held| held.object.id).collect()
}

/// A held unit is never among the objects, nothing on the map names one, and every unit an eject queue reserves is
/// held in the queue's Tunnel owner's network (the plan's invariants for the held store).
fn check_held(t: &TestSim) {
    let sim = t.sim();
    for held in sim.held() {
        let unit = held.object.id;
        assert!(sim.object(unit).is_none(), "held unit {unit:?} is on the map too");
        let orders = held.object.orders.as_ref().expect("a unit");
        assert!(orders.is_idle(), "held unit {unit:?} has orders");
        for o in sim.objects() {
            let named = sc_sim::testing::referenced_ids(o);
            assert!(!named.contains(&unit), "{:?} names held unit {unit:?}", o.id);
        }
    }
    for o in sim.objects() {
        for entry in o.tunnel.iter().flat_map(|tunnel| &tunnel.eject) {
            let held = sim.held_unit(entry.unit).expect("a queued unit is held");
            assert_eq!(held.object.owner, o.owner, "queued at its owner's Tunnel");
        }
    }
    assert_eq!(dangling_references(sim), []);
}

/// Runs `t` until `tick`, checking the held store's invariants after every tick.
fn run_checked(t: &mut TestSim, tick: Tick) {
    while t.sim().tick() < tick {
        t.step();
        check_held(t);
    }
}

/// The next id a spawn gets, if the unit with the highest id so far is still in the game.
fn next_id(t: &TestSim) -> u32 {
    let on_map = t.sim().objects().map(|o| o.id.0);
    let held = t.sim().held().map(|held| held.object.id.0);
    on_map.chain(held).max().unwrap_or(0) + 1
}

/// One tick of `t`, with the held store's invariants.
fn step_checked(t: &mut TestSim) {
    t.step();
    check_held(t);
}
