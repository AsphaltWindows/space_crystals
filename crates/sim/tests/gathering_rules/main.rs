//! The gathering rules of the design answers to questions Q0037 and Q0044 (slice M3-H3; in
//! `factions/colonists/units/drone.md`: `> Patch Occupancy (one Drone per patch)`, `> Last load of a patch`,
//! `> Interrupted mining`, `> Other orders keep the load`, `> Gather while carrying a load`, `> Drop Off order`,
//! `> When a Gather ends`, `> Assigned Drop-off`): one test per Verification point of design changes 0123, 0127 to
//! 0135, 0137, 0138, 0140, 0141 and 0170 that `gathering/` doesn't already cover, one test binary:
//! - `ends`: when a Gather ends (0123), a Deploy queued behind a Drop Off (0127), a patch that runs out under a
//!   Drone on its way (0134);
//! - `loads`: Gather while carrying (0128), other orders keep the load (0129), the Drop Off of a mixed group (0140),
//!   the last load of a patch (0138);
//! - `waiting`: waiting at a crowded patch and re-checking, returning to the last mined patch (0132), which waiting
//!   Drone mines next (0133);
//! - `reach`: a patch walled off by units or by terrain (0141);
//! - `knowledge`: only known patches count, never a plated one (0131), and a plate placed on a patch being mined
//!   (0137);
//! - `drop_offs`: a group's node destroyed, then every node, then a node warped in (0135); a node removed, and one
//!   destroyed in combat, while Drones are on their way back to it (0170);
//! - `remembered` (slice M3-R): a Gather on a patch out of sight, by what the player knows of it (0139), and the
//!   map's patches, known from the start (0159).
//!
//! `gathering/` has the others: 0130 (`trip.rs`, `interrupted_mining_collects_nothing_and_frees_the_patch_at_once`)
//! and 0134's last patch (`depletion.rs`, `with_no_patch_left_a_drone_delivers_its_load_and_idles`). Every test
//! runs under the invariants of `TestSim`, checked after every tick.

mod drop_offs;
mod ends;
mod knowledge;
mod loads;
mod reach;
mod remembered;
mod waiting;

use sc_sim::orders::state::{Behaviour, GatherPhase};
use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{
    Command, Object, ObjectId, ObjectKind, OrderKind, PlayerCommand, PlayerId, ResourceState, SimEvent, SpawnAt,
    SpawnSettings, Target, Tick, TilePos, Vec2,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const DRONE: ObjectKind = ObjectKind::Drone;
const NODE: ObjectKind = ObjectKind::EgressNode;
const PATCH: ObjectKind = ObjectKind::CrystalPatch;
/// `> Gathering`: CarryCapacity and MiningDuration.
const LOAD: u32 = 8;
const MINING: Tick = 32;
/// `> Patch Occupancy > Waiting and re-checking`: the interval of a waiting Drone's re-check (design change 0132).
const RECHECK: Tick = 16;
/// A player's starting bank in these scenarios.
const BANK: u32 = 100;

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

fn ids(ns: &[u32]) -> Vec<ObjectId> {
    ns.iter().map(|&n| id(n)).collect()
}

fn object(t: &TestSim, n: u32) -> &Object {
    t.sim().object(id(n)).expect("exists")
}

fn exists(t: &TestSim, n: u32) -> bool {
    t.sim().object(id(n)).is_some()
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

/// The orders queued behind it.
fn queued(t: &TestSim, n: u32) -> Vec<OrderKind> {
    let orders = object(t, n).orders.as_ref().expect("a unit");
    orders.queue.iter().map(|order| order.kind).collect()
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

/// Worker `n` mines patch `patch`.
fn mines(t: &TestSim, n: u32, patch: u32) -> bool {
    is_mining(t, n) && order(t, n) == Some(OrderKind::Gather(id(patch)))
}

fn load(t: &TestSim, n: u32) -> u32 {
    object(t, n).worker.expect("a worker").load
}

fn drop_off(t: &TestSim, n: u32) -> Option<ObjectId> {
    object(t, n).worker.expect("a worker").drop_off
}

fn gather(t: &mut TestSim, tick: Tick, player: PlayerId, drones: &[u32], patch: u32) {
    t.order_at(tick, player, &ids(drones), OrderKind::Gather(id(patch)), false);
}

fn move_to(x: f32, y: f32) -> OrderKind {
    OrderKind::Move(Target::Point(Vec2::new(x, y)))
}

/// `player`'s order of `kind` to `units`, not queued.
fn order_of(player: PlayerId, units: &[u32], kind: OrderKind) -> PlayerCommand {
    PlayerCommand {
        player,
        command: Command::Order {
            units: ids(units),
            kind,
            queue: false,
        },
    }
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
                ..
            } => Some((*tick, worker.0, drop_off.0, *amount)),
            _ => None,
        })
        .collect()
}

/// The deliveries of worker `n` so far.
fn deliveries_of(t: &TestSim, n: u32) -> Vec<(Tick, u32, u32, u32)> {
    deliveries(t).into_iter().filter(|d| d.1 == n).collect()
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

/// The distance from the centre of unit `n` to tile `(x, y)`'s square. A Drone's Silhouette is 36 × 36, so it is
/// within MiningReach (8 space units) of a patch on that tile at most 18 + 8 from it when it faces the patch, and
/// 34 at worst when turned.
fn off_tile(t: &TestSim, n: u32, x: i32, y: i32) -> f32 {
    let at = object(t, n).pos;
    let lo = Vec2::new(x as f32, y as f32) * 64.0;
    let nearest = at.clamp(lo, lo + Vec2::splat(64.0));
    at.distance(nearest)
}

/// Unit `n` stands beside tile `(x, y)`, within a Drone's MiningReach of a patch there ([`off_tile`]).
fn beside(t: &TestSim, n: u32, x: i32, y: i32) -> bool {
    off_tile(t, n, x, y) <= 34.0
}
