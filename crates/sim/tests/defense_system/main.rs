//! The Defense System and its Missile Turret (slice M6-GR; `factions/gdo/structures/defense_system.md`,
//! `factions/gdo/structures/missile_turret.md`), through the gate, one test binary:
//! - `building`: the Deployment Center constructs a Defense System only for a player with a Radar Station and an
//!   Arsenal;
//! - `flow`: the Defense System builds one Missile Turret at a time, by the Extraction Facility's flow (design change
//!   0392): build, cancel in full, ready, cancel the ready one for 75% rounded down, and place it inside the GDO build
//!   area only; at low Power its construction slows by the ratio;
//! - `turret`: the Missile Turret's homing missiles never miss, a circling Sparrow and a running Peacekeeper; placed, it
//!   fires from its first tick; at 30 generated for 51 consumed its cycle takes 41 ticks, and Generators that bring the
//!   Power back to positive give it full speed at once (design change 0423).
//!
//! Every test runs under the invariants of `TestSim`.

mod building;
mod flow;
mod turret;

use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{
    AttackPhase, Command, ConstructionCommand, Constructor, DebugCommand, Faction, ObjectId, ObjectKind, OrderKind,
    PlayerCommand, PlayerId, Reject, SimEvent, SpawnAt, Target, Tick, TilePos, Vec2,
};

use ConstructionCommand::{CancelConstruction, CancelReady, Construct};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const DEFENSE: ObjectKind = ObjectKind::DefenseSystem;
const TURRET: ObjectKind = ObjectKind::MissileTurret;

/// `factions/gdo/structures/defense_system.md > Constructs`: "Missile Turret: 100 Space Crystals, 240 frames (15
/// seconds) (placeholder)".
const TURRET_COST: u32 = 100;
const TURRET_TICKS: Tick = 240;

/// [`base`]'s Deployment Center and Defense System.
const DC: u32 = 1;
const DS: u32 = 6;

fn flat(w: usize, h: usize) -> String {
    vec![".".repeat(w); h].join("\n")
}

fn tile(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Tile(TilePos::new(x, y))
}

fn point(x: f32, y: f32) -> SpawnAt {
    SpawnAt::Point(Vec2::new(x, y))
}

fn id(n: u32) -> ObjectId {
    ObjectId(n)
}

/// Player 1's Deployment Center 1 at (2, 2) to (5, 5), Generators 2 and 3 at (8, 2) and (11, 2), Radar Station 4 at (2,
/// 8), Arsenal 5 at (6, 8) and Defense System 6 at (10, 8) to (11, 9), on a flat map of 48 × 32 tiles: Power 70
/// generated for 60 consumed. Player 1 sees nothing east of tile 14.
fn base() -> ScenarioBuilder {
    ScenarioBuilder::new(&flat(48, 32))
        .spawn(ObjectKind::DeploymentCenter, Some(P1), tile(2, 2))
        .spawn(ObjectKind::Generator, Some(P1), tile(8, 2))
        .spawn(ObjectKind::Generator, Some(P1), tile(11, 2))
        .spawn(ObjectKind::RadarStation, Some(P1), tile(2, 8))
        .spawn(ObjectKind::Arsenal, Some(P1), tile(6, 8))
        .spawn(DEFENSE, Some(P1), tile(10, 8))
}

/// `scenario` started, player 1 with 1000 Space Crystals from tick 1, which has run.
fn started(scenario: ScenarioBuilder) -> TestSim {
    let mut t = scenario.start();
    t.debug_at(1, fund(P1, 1000));
    t.run_until(1);
    t
}

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

fn construction(constructor: u32, command: ConstructionCommand) -> Command {
    Command::Construction {
        objects: vec![id(constructor)],
        command,
    }
}

fn place(origin: TilePos) -> ConstructionCommand {
    ConstructionCommand::Place {
        origin,
        orientation: Default::default(),
    }
}

fn gate(t: &TestSim, player: PlayerId, command: Command) -> Result<(), Reject> {
    t.sim().validate(&PlayerCommand { player, command })
}

/// Schedules player 1's `command` for the next tick and runs it: the tick.
fn next(t: &mut TestSim, command: Command) -> Tick {
    let tick = t.sim().tick() + 1;
    t.at(tick, P1, command).run_until(tick);
    tick
}

fn bank(t: &TestSim) -> u32 {
    t.sim().player(P1).unwrap().crystals
}

/// Constructor `n`'s state.
fn state(t: &TestSim, n: u32) -> Constructor {
    t.sim()
        .object(id(n))
        .and_then(|o| o.constructor)
        .expect("a constructor")
}

/// Runs until constructor `n` holds a building ready to place, within 2000 ticks.
fn until_ready(t: &mut TestSim, n: u32) {
    let start = t.sim().tick();
    while state(t, n).ready().is_none() {
        assert!(t.sim().tick() < start + 2000, "#{n}: {:?}", state(t, n));
        t.step();
    }
}

/// The constructions completed so far: (tick, constructor, building).
fn completions(t: &TestSim) -> Vec<(Tick, u32, ObjectKind)> {
    (t.events().iter())
        .filter_map(|(tick, e)| match *e {
            SimEvent::ConstructionCompleted { constructor, kind } => Some((*tick, constructor.0, kind)),
            _ => None,
        })
        .collect()
}

/// The missiles launched so far by `shooter`: (tick, target, homing).
fn launches(t: &TestSim, shooter: u32) -> Vec<(Tick, Option<u32>, bool)> {
    (t.events().iter())
        .filter_map(|(tick, e)| match *e {
            SimEvent::ProjectileLaunched {
                shooter: by,
                target,
                homing,
                ..
            } if by == id(shooter) => Some((*tick, target.map(|t| t.0), homing)),
            _ => None,
        })
        .collect()
}

/// The hits by `attacker` so far: (tick, target, HP taken).
fn hits_by(t: &TestSim, attacker: u32) -> Vec<(Tick, u32, f32)> {
    (t.events().iter())
        .filter_map(|(tick, e)| match *e {
            SimEvent::Hit {
                attacker: by,
                target,
                damage,
                ..
            } if by == id(attacker) => Some((*tick, target.0, damage)),
            _ => None,
        })
        .collect()
}

/// The tick `n` died in, if it did.
fn died(t: &TestSim, n: u32) -> Option<Tick> {
    (t.events().iter()).find_map(|(tick, e)| match *e {
        SimEvent::Died { id: dead, .. } if dead == id(n) => Some(*tick),
        _ => None,
    })
}

/// The projectiles `shooter` has in flight now.
fn in_flight(t: &TestSim, shooter: u32) -> usize {
    (t.sim().projectiles().iter())
        .filter(|p| p.shooter == id(shooter))
        .count()
}

/// `player`'s `units` ordered `kind` in tick `tick`.
fn order(t: &mut TestSim, tick: Tick, player: PlayerId, units: &[u32], kind: OrderKind) {
    let units: Vec<ObjectId> = units.iter().copied().map(ObjectId).collect();
    t.order_at(tick, player, &units, kind, false);
}
