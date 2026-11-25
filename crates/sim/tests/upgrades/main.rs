//! Tunnel upgrades (slice M4-G; `factions/syndicate/tunnels.md > Tunnel ObjectInterfaceState`,
//! `factions/syndicate/tunnels.md > Cost Scaling > Upgrade Costs`, `factions/syndicate/tunnels.md > Tunnel Area
//! Non-Overlap Rule`; design changes 0229, 0231, 0232, 0237, 0242, 0249, 0283 and 0284), through the gate, one test
//! binary:
//! - `costs`: the prices (2 then 4, 3 then 6, a Tier 3 Tunnel counted in the Tier 2 price, two upgrades started
//!   together), the refunds at the current top price in either order (0232, 0284), and the books over random orders;
//! - `blocked`: what refuses an upgrade: an own neighbour's area, an enemy's by the truth (0237), Mountain or Water in
//!   the larger area (0242), the map's edge on every side (0283), an operation under way, the top tier; and what
//!   doesn't: the owner's lingering ghost, whose build then fails on arrival;
//! - `during`: the reserved area against the owner's ghosts and builds and against an enemy's, the old tier's transit,
//!   exits and Tunnel Space (0249), a cancel, the Tunnel destroyed;
//! - `completion`: the 480 frames (0229), the HP (0231), Tunnel Space, a neutral expansion reclaimed (0258), and a game
//!   saved mid-upgrade.
//!
//! Every test runs under the invariants of `TestSim`, the operation invariant O1 among them (an upgrading Tunnel is in
//! operation, a tier below its target, with its tier's frames).

mod blocked;
mod completion;
mod costs;
mod during;

use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::tunnel::Operation;
use sc_sim::{
    BuildingOrientation, Command, DebugCommand, Faction, Object, ObjectId, ObjectKind, OrderKind, PlacementCheck,
    PlayerCommand, PlayerId, Reject, SimEvent, SpawnAt, Tick, TilePos, TunnelCommand,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const AGENT: ObjectKind = ObjectKind::Agent;
const TUNNEL: ObjectKind = ObjectKind::Tunnel;

/// Player 1's Tunnel in [`base`], on tiles (4, 8) to (7, 11): its Tier 1 area is (2, 6) to (9, 13), its Tier 2 area
/// (1, 5) to (10, 14).
const T1: ObjectId = ObjectId(1);

/// An upgrade's frames, to Tier 2 and to Tier 3 (`factions/syndicate/tunnels.md > Cost Scaling > Upgrade Costs`:
/// "**480 frames (30 seconds)**", design change 0229).
const FRAMES: Tick = 480;

fn flat(w: usize, h: usize) -> String {
    vec![".".repeat(w); h].join("\n")
}

fn at(x: i32, y: i32) -> TilePos {
    TilePos::new(x, y)
}

fn tile(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Tile(at(x, y))
}

/// The centre of tile `(x, y)`.
fn point(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Point(at(x, y).center())
}

fn id(n: u32) -> ObjectId {
    ObjectId(n)
}

/// Both players play the Syndicate on `map`.
fn syndicate(map: &str) -> ScenarioBuilder {
    ScenarioBuilder::new(map)
        .faction(P1, Faction::Syndicate)
        .faction(P2, Faction::Syndicate)
}

/// Player 1's Tier 1 Tunnel [`T1`] on a flat 40 × 20 map.
fn base() -> ScenarioBuilder {
    syndicate(&flat(40, 20)).spawn(TUNNEL, Some(P1), tile(4, 8))
}

/// `scenario` started with auto-targeting off, player 1 with `supplies` Supplies from tick 1, which has run.
fn started(scenario: ScenarioBuilder, supplies: u32) -> TestSim {
    let mut t = scenario.start();
    t.without_auto_targeting();
    t.debug_at(1, supplies_of(P1, supplies));
    t.run_until(1);
    t
}

/// Sets `player`'s Supplies to `supplies`, their other banks to 0.
fn supplies_of(player: PlayerId, supplies: u32) -> DebugCommand {
    DebugCommand::SetResources {
        player,
        crystals: 0,
        supplies,
        alloys: 0,
        essence: 0,
        flux: 0,
    }
}

/// The Upgrade of `tunnel` (its Q).
fn upgrade(tunnel: ObjectId) -> Command {
    Command::Tunnel {
        tunnel,
        command: TunnelCommand::Upgrade,
    }
}

/// `tunnel`'s X: Cancel its operation.
fn cancel(tunnel: ObjectId) -> Command {
    Command::Tunnel {
        tunnel,
        command: TunnelCommand::Cancel,
    }
}

/// Sets `tunnel`'s tier, without an upgrade's price or time (`DebugCommand::SetTier`).
fn set_tier(tunnel: ObjectId, tier: u8) -> Command {
    Command::Debug(DebugCommand::SetTier { id: tunnel, tier })
}

/// Removes `id` outright, as its destruction does (`DebugCommand::Remove`).
fn remove(id: ObjectId) -> Command {
    Command::Debug(DebugCommand::Remove { id })
}

/// A Build of a Tunnel at `origin`, unturned and without Shift, to `agent`.
fn build(agent: ObjectId, origin: TilePos) -> Command {
    let orientation = BuildingOrientation::default();
    Command::Order {
        units: vec![agent],
        kind: OrderKind::Build {
            kind: TUNNEL,
            origin,
            orientation,
        },
        queue: false,
    }
}

/// What Agent `agent`'s ghost of a Tunnel at `origin`, unturned and without Shift, shows (`Sim::build_check`), by
/// what its owner knows.
fn ghost(t: &TestSim, agent: ObjectId, origin: TilePos) -> PlacementCheck {
    let orientation = BuildingOrientation::default();
    (t.sim().build_check(agent, TUNNEL, origin, orientation, false)).expect("an Agent on the map")
}

/// What the gate says to `player`'s `command` now (`Sim::validate`).
fn gate(t: &TestSim, player: PlayerId, command: Command) -> Result<(), Reject> {
    t.sim().validate(&PlayerCommand { player, command })
}

/// `player`'s `command` in the next tick, which runs.
fn next(t: &mut TestSim, player: PlayerId, command: Command) {
    next_all(t, vec![(player, command)]);
}

/// `commands`, each a player's, in the next tick, in their order, which runs.
fn next_all(t: &mut TestSim, commands: Vec<(PlayerId, Command)>) {
    let tick = t.sim().tick() + 1;
    for (player, command) in commands {
        t.at(tick, player, command);
    }
    t.step();
}

/// Runs ticks until `done`, at most `ticks`.
fn step_until(t: &mut TestSim, ticks: u32, what: &str, done: impl Fn(&TestSim) -> bool) {
    for _ in 0..ticks {
        if done(t) {
            return;
        }
        t.step();
    }
    assert!(done(t), "{what}: not within {ticks} ticks");
}

fn object(t: &TestSim, id: ObjectId) -> &Object {
    t.sim().object(id).unwrap_or_else(|| panic!("{id:?} is on the map"))
}

/// `tunnel`'s tier now.
fn tier(t: &TestSim, tunnel: ObjectId) -> u8 {
    object(t, tunnel).tunnel.as_ref().expect("a Tunnel").tier
}

/// The tier `tunnel` upgrades to, if it upgrades.
fn upgrading(t: &TestSim, tunnel: ObjectId) -> Option<u8> {
    let state = object(t, tunnel).tunnel.as_ref()?;
    state.operation.and_then(Operation::upgrade_to)
}

/// `player`'s Supplies.
fn supplies(t: &TestSim, player: PlayerId) -> u32 {
    t.sim().player(player).expect("in the game").supplies
}

/// The price of `tunnel`'s upgrade now (`Sim::upgrade_price`): of the one under way, what its X refunds; else of the
/// next tier's, what its Q costs.
fn price(t: &TestSim, tunnel: ObjectId) -> Option<u32> {
    t.sim().upgrade_price(tunnel)
}

/// `player`'s Tunnel Space, used and available.
fn tunnel_space(t: &TestSim, player: PlayerId) -> (u32, u32) {
    let cap = t.sim().army_cap(player).expect("a Syndicate player");
    (cap.used, cap.available)
}

/// The upgrades started so far: each Tunnel, the tier it upgrades to and the price paid.
fn starts(t: &TestSim) -> Vec<(ObjectId, u8, u32)> {
    (t.events().iter())
        .filter_map(|(_, e)| match *e {
            SimEvent::UpgradeStarted { tunnel, to, price } => Some((tunnel, to, price)),
            _ => None,
        })
        .collect()
}

/// The upgrades cancelled so far: each Tunnel, the tier it was upgrading to and the refund.
fn cancels(t: &TestSim) -> Vec<(ObjectId, u8, u32)> {
    (t.events().iter())
        .filter_map(|(_, e)| match *e {
            SimEvent::UpgradeCancelled { tunnel, to, refund } => Some((tunnel, to, refund)),
            _ => None,
        })
        .collect()
}

/// The upgrades completed so far: the tick, the Tunnel and its new tier.
fn completions(t: &TestSim) -> Vec<(Tick, ObjectId, u8)> {
    (t.events().iter())
        .filter_map(|(tick, e)| match *e {
            SimEvent::UpgradeCompleted { tunnel, tier } => Some((*tick, tunnel, tier)),
            _ => None,
        })
        .collect()
}

/// The Builds that failed so far: each worker, and what the check on its arrival found as a whole.
fn failures(t: &TestSim) -> Vec<(ObjectId, Vec<sc_sim::WholeProblem>)> {
    (t.events().iter())
        .filter_map(|(_, e)| match e {
            SimEvent::BuildFailed { worker, problems, .. } => Some((*worker, problems.whole.clone())),
            _ => None,
        })
        .collect()
}
