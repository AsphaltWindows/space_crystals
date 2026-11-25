//! Construction (slice M2-D1; `factions/gdo/structures/deployment_center.md`, `factions/gdo/economy.md >
//! ExtractionFacility`): the Deployment Center and the Extraction Facility build one building at a time and hold it
//! ready to place. One test per acceptance point of the plan (`artifacts/director/m2-plan.md` §3.3 D1) and per
//! verification point of design changes 0084 (a destroyed constructor refunds as if cancelled) and 0089 (the power
//! penalty slows the Extraction Facility, never the Deployment Center); Place through the placement check as it
//! applies (plan decision T4); the gate's rules, fuzzed; saves and replays. `TestSim` checks the invariants after
//! every tick. The openings on the reference base, this slice's mini-oracle of the clean Barracks among them, are the
//! economy oracle's (`tests/economy_oracle/`, slice M2-J1).
//!
//! One test binary:
//! - `timings`: construction times at full power, and the power penalty, which slows the Extraction Facility and
//!   not the Deployment Center (0089);
//! - `refunds`: the refunds of a cancel, while constructing and once ready, and of a constructor destroyed, by a
//!   removal or in combat (0084);
//! - `gate`: the refused commands, a command to several constructors in id order (design change 0071), and the
//!   gate fuzzed;
//! - `placing`: a valid Place, a refused one, and a Place checked as it applies (T4);
//! - `saves`: a game saved in the middle of its constructions, and its replay.

mod gate;
mod placing;
mod refunds;
mod saves;
mod timings;

use sc_sim::economy::{RATIO_ONE, Refund};
use sc_sim::nav::NavClass;
use sc_sim::orders::MAX_ORDER_UNITS;
use sc_sim::testing::{ScenarioBuilder, TestSim, dangling_references};
use sc_sim::{
    BuildingOrientation, Command, ConstructionCommand, Constructor, DebugCommand, ObjectId, ObjectKind, OrderKind,
    Placement, PlacementCheck, PlacementProblem, PlayerCommand, PlayerId, Reject, Replay, Rng, Rotation, Sim,
    SimConfig, SimEvent, SpawnAt, Tick, TickInputs, TilePos, TileProblem, WholeProblem,
};

use ConstructionCommand::{CancelConstruction, CancelReady, Construct, Place};
use ObjectKind::{
    Barracks, CrystalPatch, DeploymentCenter, ExtractionFacility, ExtractionPlate, Generator, Peacekeeper,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

fn flat(w: usize, h: usize) -> String {
    vec![".".repeat(w); h].join("\n")
}

fn at(x: i32, y: i32) -> TilePos {
    TilePos::new(x, y)
}

fn tile(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Tile(at(x, y))
}

fn id(n: u32) -> ObjectId {
    ObjectId(n)
}

fn construction(objects: &[u32], command: ConstructionCommand) -> Command {
    Command::Construction {
        objects: objects.iter().map(|&n| ObjectId(n)).collect(),
        command,
    }
}

fn place(x: i32, y: i32) -> ConstructionCommand {
    Place {
        origin: at(x, y),
        orientation: BuildingOrientation::default(),
    }
}

/// Sets `player`'s bank to `crystals` (no Supplies).
fn fund(player: PlayerId, crystals: u32) -> DebugCommand {
    DebugCommand::SetResources {
        player,
        crystals,
        supplies: 0,
        alloys: 0,
        essence: 0,
        flux: 0,
    }
}

fn bank(t: &TestSim, player: PlayerId) -> u32 {
    t.sim().player(player).unwrap().crystals
}

/// Constructor `n`'s state. Panics if it is gone or no constructor.
fn state(t: &TestSim, n: u32) -> Constructor {
    t.sim()
        .object(id(n))
        .and_then(|o| o.constructor)
        .expect("a constructor")
}

/// Schedules player 1's `command` for the next tick and runs it.
fn next(t: &mut TestSim, command: Command) -> &mut TestSim {
    let tick = t.sim().tick() + 1;
    t.at(tick, P1, command).step()
}

/// Runs until constructor `n` holds a building ready to place.
fn until_ready(t: &mut TestSim, n: u32) -> ObjectKind {
    for _ in 0..2000 {
        if let Some(kind) = state(t, n).ready() {
            return kind;
        }
        t.step();
    }
    panic!("#{n} never finished: {:?}", state(t, n));
}

/// The constructions completed so far: tick, constructor, building.
fn completions(t: &TestSim) -> Vec<(Tick, u32, ObjectKind)> {
    t.events()
        .iter()
        .filter_map(|(tick, e)| match e {
            SimEvent::ConstructionCompleted { constructor, kind } => Some((*tick, constructor.0, *kind)),
            _ => None,
        })
        .collect()
}

/// 24 × 16 tiles, level 1 but for a pit, corner (10, 7) at level 0, which slopes the 4 tiles around it, (9, 6) to (10,
/// 7); tile (3, 4) is water.
fn pitted_map() -> String {
    let mut rows = vec![".".repeat(24); 16];
    rows[4].replace_range(3..4, "~");
    let heights: Vec<String> = (0..=16)
        .map(|y| (0..=24).map(|x| if (x, y) == (10, 7) { '0' } else { '1' }).collect())
        .collect();
    format!("{}\n\n{}", rows.join("\n"), heights.join("\n"))
}

/// On the pitted map, player 1's Deployment Center 1 at (6, 1) to (9, 4), seeing 6 tiles around (8, 3), and its
/// Extraction Facility 2 at (11, 1) to (13, 3); patch 3 at (5, 7); player 1's Peacekeeper 4 on tile (3, 1). The
/// build area is x 0 to 21, y 0 to 15. Power 30 / 15; no income.
fn base() -> ScenarioBuilder {
    ScenarioBuilder::new(&pitted_map())
        .spawn(DeploymentCenter, Some(P1), tile(6, 1))
        .spawn(ExtractionFacility, Some(P1), tile(11, 1))
        .spawn(CrystalPatch, None, tile(5, 7))
        .spawn(Peacekeeper, Some(P1), SpawnAt::Point(at(3, 1).center()))
}
