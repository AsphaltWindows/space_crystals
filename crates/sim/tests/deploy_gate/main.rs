//! Beacon Deployment's gate and arrival (slice M3-F; `sc_sim::deploy`): the Deploy order goes to one Drone of the
//! player's; the gate judges its site by what the player knows (design change 0120, `Mode::Known`); the arrival judges
//! it on the truth, and fails the order for a ground unit, a structure or a patch there (0118), or for a Drone that
//! stopped short of it (0117); a Beacon under deployment takes only Cancel (0116); the sites of pending Deploys
//! reserve their footprints against their owner's further placements and block no one (0125); and the gate, fuzzed.
//! `tests/deploy/` is the deployment itself. Walks are judged by their outcomes, never by their ticks (the Drone's
//! DragMovement sets their pace). `TestSim` checks the movement invariants after every tick.
//!
//! One test binary:
//! - `gate`: the Deploy to one Drone of the player's, the site judged by what the player knows (0120), and a
//!   Beacon under deployment that takes only Cancel (0116);
//! - `arrival`: the site judged on the truth on arrival: never explored ground, a ground unit or a patch on it
//!   (0118), a Drone walled off from it or stopped while overlapping it (0117);
//! - `pending`: the sites of pending Deploys, reserved against their owner's placements and blocking no one
//!   (0125);
//! - `fuzz`: the gate and the arrival, fuzzed.

mod arrival;
mod fuzz;
mod gate;
mod pending;

use sc_sim::deploy::{self, on_site};
use sc_sim::testing::{ScenarioBuilder, TestSim, dangling_references, every_command};
use sc_sim::transform::warp_in_footprint;
const FOG_DELAY_TICKS: sc_sim::Tick = sc_sim::tuning::vision::BUILTIN.fog_delay_ticks;
use sc_sim::{
    Command, ConstructionCommand, DebugCommand, Faction, MoveStatus, Object, ObjectId, ObjectKind, OrderKind,
    PlacementCheck, PlacementProblem, PlayerCommand, PlayerId, Reject, Replay, Rng, Sim, SimEvent, SpawnAt, Target,
    TickInputs, TilePos, TileProblem, Vec2, VisibilityState,
};

use ConstructionCommand::{CancelTransform, WarpIn};
use ObjectKind::{Beacon, CrystalPatch, Drone, EgressNode, Peacekeeper, RoboPortal};
use TileProblem::{NotBuildable, OffMap, Reserved, Resource, Structure, Unit};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

/// `player`'s pending Deploys, each a Drone and its Beacon's site: their lingering build ghosts
/// (`Sim::pending_builds`, every worker build's since slice M4-E).
fn pending_deploys(sim: &Sim, player: PlayerId) -> Vec<(ObjectId, sc_sim::Placement)> {
    (sim.pending_builds(player).into_iter())
        .filter(|pending| pending.kind == Beacon)
        .map(|pending| (pending.worker, pending.site))
        .collect()
}

/// A site in the sight of Drone 2 of [`base`]: tiles (8, 9) to (9, 10).
const SITE: TilePos = TilePos::new(8, 9);

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

fn object(t: &TestSim, n: u32) -> &Object {
    t.sim().object(id(n)).expect("the object exists")
}

/// A Deploy to `units` (`queue` is Shift).
fn deploy(units: &[u32], origin: TilePos, queue: bool) -> Command {
    order(units, OrderKind::Deploy { origin }, queue)
}

fn order(units: &[u32], kind: OrderKind, queue: bool) -> Command {
    Command::Order {
        units: units.iter().map(|&n| id(n)).collect(),
        kind,
        queue,
    }
}

/// What the gate says to `player`'s `command` now (`Sim::validate`).
fn gate(t: &TestSim, player: PlayerId, command: Command) -> Result<(), Reject> {
    t.sim().validate(&PlayerCommand { player, command })
}

/// The gate's refusal of a site whose first problem is `problem` on tile `(x, y)`.
fn unplaceable(x: i32, y: i32, problem: TileProblem) -> Result<(), Reject> {
    Err(Reject::Unplaceable(PlacementProblem::Tile(at(x, y), problem)))
}

/// Schedules player 1's `command` for the next tick and runs it.
fn next(t: &mut TestSim, command: Command) -> &mut TestSim {
    let tick = t.sim().tick() + 1;
    t.at(tick, P1, command).step()
}

/// Unit `n`'s current order, if any.
fn current(t: &TestSim, n: u32) -> Option<OrderKind> {
    let orders = object(t, n).orders.as_ref()?;
    orders.current.as_ref().map(|active| active.order.kind)
}

/// Whether unit `n` stands idle: no order, current or queued, and no move under way.
fn idle(t: &TestSim, n: u32) -> bool {
    let o = object(t, n);
    o.orders.as_ref().is_some_and(|orders| orders.is_idle())
        && o.mover.as_ref().is_some_and(|m| m.status != MoveStatus::Moving)
}

/// Steps until Drone `n`'s Deploy is over, if it isn't yet: `Ok` with where the Drone stood when it began deploying
/// (at the end of its last tick as a Drone, which the arrival judges; where it is now, if that was before this call),
/// or `Err` with the problems its failed order reported. Panics after tick 900.
fn outcome(t: &mut TestSim, n: u32) -> Result<Vec2, PlacementCheck> {
    let mut stood = t.pos(n);
    loop {
        if object(t, n).kind != Drone {
            return Ok(stood);
        }
        let failed = t.events().iter().rev().find_map(|(_, e)| match e {
            SimEvent::DeployFailed { drone, problems, .. } if *drone == id(n) => Some(problems.clone()),
            _ => None,
        });
        if let Some(problems) = failed {
            return Err(problems);
        }
        assert!(t.sim().tick() < 900, "Drone {n} still on its way, at {}", t.pos(n));
        stood = t.pos(n);
        t.step();
    }
}

fn colonists(builder: ScenarioBuilder) -> ScenarioBuilder {
    builder.faction(P1, Faction::Colonists).faction(P2, Faction::Colonists)
}

/// `map` for two Colonists players: player 1's Egress Node 1 on (2, 2) to (3, 3) and its Drone 2 at the centre of
/// tile (5, 8), whose sight of 5 grid units covers [`SITE`]; player 2's Egress Node 3 on (36, 2) to (37, 3).
fn base_on(map: &str) -> ScenarioBuilder {
    colonists(ScenarioBuilder::new(map))
        .spawn(EgressNode, Some(P1), tile(2, 2))
        .spawn(Drone, Some(P1), point(5, 8))
        .spawn(EgressNode, Some(P2), tile(36, 2))
}

/// [`base_on`] a flat 40 × 16 map.
fn base() -> ScenarioBuilder {
    base_on(&flat(40, 16))
}
