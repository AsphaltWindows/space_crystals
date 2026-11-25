//! Agents gather Space Crystals (slice M4-D; `factions/syndicate/units/agent.md > Gathering`, design changes 0248,
//! 0255 and 0259; `factions/syndicate/tunnels.md > Tunnel > SymmetryType - ABCD`; plan decision T34): one Gather job
//! for the Drone and the Agent, which shares patches, delivers at the 1-square crystal hatch on its Tunnel's Side B,
//! one Agent at a time, and takes the nearest Tunnel anew with each Gather. `TestSim` checks the invariants after
//! every tick.
//!
//! In modules by topic, one test binary:
//! - `trips`: one Agent's trip at 0 to 6 grid units against the design's table, and the income of two, three and
//!   four Agents, with the ceiling of 12.5 Space Crystals per second per Tunnel;
//! - `line`: the line at the hatch (0255: the first to arrive delivers next, nobody gives up), an Agent blocked from
//!   the hatch, where an Agent stands, a turned Tunnel;
//! - `tunnels`: which Tunnel (the nearest by its hatch, kept for the Gather, picked again by a new one, a lost one);
//! - `rules`: the Drone's job rules on the Agent, patch sharing, the Drop Off order, the gate and the default order;
//! - `soak`: six Agents on two stacked Tunnels for ten minutes (ignored).

mod line;
mod rules;
mod soak;
mod trips;
mod tunnels;

use sc_sim::orders::state::{Behaviour, GatherPhase};
use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{
    Ability, BuildingOrientation, Command, DebugCommand, Faction, Object, ObjectId, ObjectKind, OrderKind,
    PlayerCommand, PlayerId, Reject, ResourceState, Rotation, Scenario, Shape, SimEvent, SpawnAt, SpawnSettings,
    Target, Tick, TilePos, Vec2,
};

const P1: PlayerId = PlayerId(1);
const AGENT: ObjectKind = ObjectKind::Agent;
const TUNNEL: ObjectKind = ObjectKind::Tunnel;
/// `> Gathering > Space Crystals`: CarryCapacity, MiningDuration, DropOffDuration and the two reaches.
const LOAD: u32 = 50;
const MINING: Tick = 80;
const DROP_OFF: Tick = 64;
const REACH: f32 = 8.0;

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

fn bank(t: &TestSim) -> u32 {
    t.sim().player(P1).expect("in the game").crystals
}

fn load(t: &TestSim, n: u32) -> u32 {
    object(t, n).worker.expect("a worker").load
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

fn is_unloading(t: &TestSim, n: u32) -> bool {
    matches!(phase(t, n), Some(GatherPhase::DroppingOff { .. }))
}

fn is_mining(t: &TestSim, n: u32) -> bool {
    matches!(phase(t, n), Some(GatherPhase::Mining { .. }))
}

fn gather(t: &mut TestSim, tick: Tick, agents: &[u32], patch: u32) {
    let units: Vec<ObjectId> = agents.iter().map(|&n| id(n)).collect();
    t.order_at(tick, P1, &units, OrderKind::Gather(id(patch)), false);
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

/// The crystal hatch of Tunnel `n`: its square.
fn hatch(t: &TestSim, n: u32) -> Shape {
    let point = sc_sim::economy::drop_off::crystal_point_of(object(t, n)).expect("a drop-off");
    assert!(point.side.is_some(), "a hatch");
    point.shape
}

/// The gap between unit `n` and the crystal hatch of Tunnel `tunnel`.
fn gap_to_hatch(t: &TestSim, n: u32, tunnel: u32) -> f32 {
    Shape::of(object(t, n)).gap(&hatch(t, tunnel))
}

/// The reference layout of design change 0259: "one Tunnel with its crystal hatch facing a single patch at each
/// distance", `distance` grid units from the hatch square, edge to edge. The Tunnel is unturned at tiles (6, 6) to
/// (9, 9), Side A to the north and Side B to the east, so its crystal hatch is tile (9, 8); the patch lies east of
/// it in the same row. The `agents` start in a row south-east of the Tunnel. Ids: Tunnel 1, patch 2, Agents from 3.
fn field(agents: u32, distance: i32) -> ScenarioBuilder {
    let mut builder = flat(30, 20)
        .spawn(TUNNEL, Some(P1), tile(6, 6))
        .spawn_with(patch_of(1_000_000), tile(10 + distance, 8));
    for i in 0..agents as i32 {
        builder = builder.spawn(AGENT, Some(P1), centre(12 + i, 12));
    }
    builder
}

/// The ids of the Agents of [`field`].
fn agents(count: u32) -> Vec<u32> {
    (3..3 + count).collect()
}

/// `scenario` with its spawn of index `spawn` turned and flipped.
fn turned(mut scenario: Scenario, spawn: usize, rotation: Rotation, flip_h: bool) -> Scenario {
    scenario.map.spawns[spawn].orientation = BuildingOrientation {
        rotation,
        flip_h,
        flip_v: false,
    };
    scenario
}

/// What a watched game showed at the hatches ([`watch`]).
#[derive(Debug, Default)]
struct Watched {
    /// The longest an Agent waited within reach of a hatch (`GatherPhase::AtHatch`), ticks.
    longest_wait: Tick,
    /// The longest an Agent waited within reach of a hatch that nobody unloaded at, ticks.
    longest_idle_wait: Tick,
    /// The longest an Agent stood with its load for a hatch that nobody unloaded at, ticks: within reach of it, or
    /// short of it, its leg back ended (`orders::gather::LEG_RETRY_TICKS`).
    longest_idle_stand: Tick,
}

/// Runs `t` for `ticks` and checks the hatch rules after every tick, for the `agents` and the `tunnels`:
/// - `tunnels.md > SymmetryType - ABCD`: "Only one Agent may drop off resources at a side at a time";
/// - design change 0255: "the first to arrive delivers next": an Agent that begins to unload was the first of
///   those waiting within reach, by arrival tick, then id.
fn watch(t: &mut TestSim, ticks: Tick, agents: &[u32], tunnels: &[u32]) -> Watched {
    let node = |t: &TestSim, n: u32| -> Option<u32> {
        let orders = object(t, n).orders.as_ref()?;
        match orders.current.as_ref()?.state {
            Behaviour::Gather(gather) => gather.node.map(|node| node.0),
            _ => None,
        }
    };
    let waiting = |t: &TestSim, n: u32| match phase(t, n) {
        Some(GatherPhase::AtHatch { since }) => Some(since),
        _ => None,
    };
    let mut watched = Watched::default();
    let mut idle_waits = vec![0; agents.len()];
    let mut idle_stands = vec![0; agents.len()];
    for _ in 0..ticks {
        // The line as this tick finds it.
        let before: Vec<(u32, Option<u32>, Option<Tick>)> =
            agents.iter().map(|&n| (n, node(t, n), waiting(t, n))).collect();
        let was_unloading: Vec<u32> = agents.iter().copied().filter(|&n| is_unloading(t, n)).collect();
        t.step();
        let now = t.sim().tick();
        for &tunnel in tunnels {
            let at: Vec<u32> = (agents.iter().copied())
                .filter(|&n| is_unloading(t, n) && node(t, n) == Some(tunnel))
                .collect();
            assert!(at.len() <= 1, "tick {now}: {at:?} unload at Tunnel {tunnel} at once");
            for &n in at.iter().filter(|n| !was_unloading.contains(n)) {
                let mine = before.iter().find(|entry| entry.0 == n).and_then(|entry| entry.2);
                let ahead: Vec<u32> = (before.iter())
                    .filter(|&&(other, at, since)| {
                        let earlier = (since, other) < (Some(mine.unwrap_or(now)), n);
                        other != n && at == Some(tunnel) && since.is_some() && earlier
                    })
                    .map(|entry| entry.0)
                    .collect();
                assert!(ahead.is_empty(), "tick {now}: Agent {n} unloads ahead of {ahead:?}");
            }
        }
        for (i, &n) in agents.iter().enumerate() {
            let busy = (agents.iter()).any(|&other| is_unloading(t, other) && node(t, other) == node(t, n));
            let short = phase(t, n) == Some(GatherPhase::ToDropOff)
                && node(t, n).is_some()
                && t.status(n) != sc_sim::MoveStatus::Moving;
            let since = waiting(t, n);
            idle_stands[i] = if (since.is_some() || short) && !busy {
                idle_stands[i] + 1
            } else {
                0
            };
            watched.longest_idle_stand = watched.longest_idle_stand.max(idle_stands[i]);
            let Some(since) = since else {
                idle_waits[i] = 0;
                continue;
            };
            watched.longest_wait = watched.longest_wait.max(now - since);
            idle_waits[i] = if busy { 0 } else { idle_waits[i] + 1 };
            watched.longest_idle_wait = watched.longest_idle_wait.max(idle_waits[i]);
        }
    }
    watched
}

/// The deliveries at `tunnel` come no closer together than DropOffDuration: "a Tunnel takes in at most 50 Space
/// Crystals per 4 seconds = 12.5 Space Crystals per second", exactly (design change 0259).
fn assert_ceiling(t: &TestSim, tunnel: u32) {
    let ticks: Vec<Tick> = (deliveries(t).iter()).filter(|d| d.2 == tunnel).map(|d| d.0).collect();
    for pair in ticks.windows(2) {
        assert!(pair[1] - pair[0] >= DROP_OFF, "Tunnel {tunnel}: deliveries at {pair:?}");
    }
}

/// The income of the deliveries after tick `from` up to tick `to`, Space Crystals per second.
fn income(t: &TestSim, from: Tick, to: Tick) -> f32 {
    let amount: u32 = (deliveries(t).iter())
        .filter(|d| d.0 > from && d.0 <= to)
        .map(|d| d.3)
        .sum();
    amount as f32 * 16.0 / (to - from) as f32
}
