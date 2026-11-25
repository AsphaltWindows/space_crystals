//! Gathering (slice M3-H, brought in line with the design answers by slice M3-H3; `factions/colonists/units.md >
//! Drone > Gathering`, `> Patch Occupancy (one Drone per patch)`, `> When a Gather ends`, `> Assigned Drop-off`;
//! `factions/colonists/economy.md > Egress Node > Drop Off`): the loop and its trip time, occupancy, waiting and
//! redirects, Set Drop-off Location, depletion, the Drop Off order, a lost Egress Node, the gate, a rally on a patch,
//! and a game replayed and saved. `TestSim` checks the invariants after every tick. The Verification points of the
//! design answers are `gathering_rules`; the crowd is `gathering_soak.rs`.
//!
//! In modules by topic, one test binary:
//! - `trip`: one Drone's loop and its trip time, and interrupted mining (0130);
//! - `occupancy`: two Drones arriving together, waiting with no patch free, an enemy Drone on the patch;
//! - `drop_offs`: Set Drop-off Location, the Drop Off order, a lost Egress Node;
//! - `depletion`: a depleted patch and the Drones going on, the last patch (0134);
//! - `gate`: the invalid worker commands;
//! - `games`: a Drone rallied on a patch, and a game replayed and saved.

mod depletion;
mod drop_offs;
mod games;
mod gate;
mod occupancy;
mod trip;

use sc_sim::nav::NavClass;
use sc_sim::orders::state::{Behaviour, GatherPhase};
use sc_sim::testing::{ScenarioBuilder, TestSim, dangling_references};
use sc_sim::{
    Ability, Command, DebugCommand, Object, ObjectId, ObjectKind, OrderKind, PlayerCommand, PlayerId,
    ProductionCommand, Reject, Replay, ResourceState, Sim, SimEvent, SpawnAt, SpawnSettings, Target, Tick, TilePos,
    Vec2,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const DRONE: ObjectKind = ObjectKind::Drone;
const NODE: ObjectKind = ObjectKind::EgressNode;
const PATCH: ObjectKind = ObjectKind::CrystalPatch;
/// `> Gathering`: CarryCapacity, MiningDuration and DropOffDuration.
const LOAD: u32 = 8;
const MINING: Tick = 32;
const DROP_OFF: Tick = 2;
/// One trip of the reference geometry with DragMovement (`economy/gathering.rs`, module docs), with the Drone's
/// numbers of design change 0205: 81 before them.
const TRIP: Tick = 73;

fn flat(w: usize, h: usize) -> ScenarioBuilder {
    ScenarioBuilder::new(&vec![".".repeat(w); h].join("\n"))
}

fn tile(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Tile(TilePos::new(x, y))
}

fn centre(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Point(TilePos::new(x, y).center())
}

fn patch_of(amount: u32) -> SpawnSettings {
    SpawnSettings::CrystalPatch {
        starting_amount: amount,
    }
}

fn id(n: u32) -> ObjectId {
    ObjectId(n)
}

fn object(t: &TestSim, n: u32) -> &Object {
    t.sim().object(id(n)).expect("exists")
}

fn bank(t: &TestSim, player: PlayerId) -> u32 {
    t.sim().player(player).expect("in the game").crystals
}

fn remaining(t: &TestSim, n: u32) -> u32 {
    match object(t, n).resource {
        Some(ResourceState::CrystalPatch { remaining }) => remaining,
        _ => panic!("object {n} is no patch"),
    }
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

fn is_mining(t: &TestSim, n: u32) -> bool {
    matches!(phase(t, n), Some(GatherPhase::Mining { .. }))
}

fn is_waiting(t: &TestSim, n: u32) -> bool {
    matches!(phase(t, n), Some(GatherPhase::Waiting { .. }))
}

fn load(t: &TestSim, n: u32) -> u32 {
    object(t, n).worker.expect("a worker").load
}

fn drop_off(t: &TestSim, n: u32) -> Option<ObjectId> {
    object(t, n).worker.expect("a worker").drop_off
}

fn gather(t: &mut TestSim, tick: Tick, player: PlayerId, drones: &[u32], patch: u32) {
    let units: Vec<ObjectId> = drones.iter().map(|&n| id(n)).collect();
    t.order_at(tick, player, &units, OrderKind::Gather(id(patch)), false);
}

/// The deliveries so far, as `(tick, worker, drop-off, amount)`.
fn deliveries(t: &TestSim) -> Vec<(Tick, u32, u32, u32)> {
    t.events()
        .iter()
        .filter_map(|(tick, e)| match e {
            SimEvent::ResourcesDelivered {
                worker,
                drop_off,
                amount,
            } => Some((*tick, worker.0, drop_off.0, *amount)),
            _ => None,
        })
        .collect()
}

/// Steps until `done` holds, at most `limit` ticks, and returns the tick it first held in.
fn run_to(t: &mut TestSim, limit: Tick, what: &str, done: impl Fn(&TestSim) -> bool) -> Tick {
    for _ in 0..limit {
        t.step();
        if done(t) {
            return t.sim().tick();
        }
    }
    panic!("{what}: not within {limit} ticks\n{}", t.ascii(None));
}

/// The first tick in which worker `n` mines.
fn mines_from(t: &mut TestSim, n: u32) -> Tick {
    run_to(t, 400, "it mines", |t| is_mining(t, n))
}

/// The reference geometry of one trip (design change 0107): a patch at tile (5, 2) and an Egress Node 2 grid
/// units south of it, edge to edge, with a Drone of player 1 between them. Ids: patch 1, node 2, Drone 3.
fn one_patch(amount: u32) -> ScenarioBuilder {
    flat(16, 10)
        .spawn_with(patch_of(amount), tile(5, 2))
        .spawn(NODE, Some(P1), tile(5, 5))
        .spawn(DRONE, Some(P1), centre(5, 4))
}
