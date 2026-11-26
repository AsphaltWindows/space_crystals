//! The Field Engineer and the Career Campus (slice M5-N9, design change 0328; `factions/gdo/units/field_engineer.md`,
//! `factions/gdo/structures/career_campus.md`), through the gate. No command deals damage on demand: a test damages
//! its target in a state dump of the game ([`damaged`], as the repairs' tests do) and plays on from it. One test
//! binary:
//! - `verification`: 0328's verification, "A Field Engineer from a Barracks (with a Career Campus) repairs a damaged
//!   Barracks, spending crystals", end to end: the Deployment Center constructs and places a Career Campus, which
//!   draws 20 Power, and the Barracks trains two Field Engineers, each reserving its Unit Control as its production
//!   starts; one repairs the Barracks from 150 to 300 HP in 40 frames, a crystal for every 3 HP, 50 in all, and the
//!   two together in 20 frames for the same 50; a right-click on the damaged Barracks is that Repair;
//! - `prerequisites`: E refused without a Career Campus, the Career Campus without a Barracks; losing the Career
//!   Campus refuses E again, and the Field Engineers already queued still come out; the Career Campus extends the
//!   build area by 2;
//! - `refusals`: the gate's Repair refusals, as for Recruits: a building at full HP, an enemy's building, a site,
//!   infantry.

mod prerequisites;
mod refusals;
mod verification;

use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{
    Command, DebugCommand, ObjectId, ObjectKind, OrderKind, PlayerCommand, PlayerId, ProductionCommand, Reject, Sim,
    SimEvent, SpawnAt, Tick, TickInputs, TilePos,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const FIELD_ENGINEER: ObjectKind = ObjectKind::FieldEngineer;
const CAREER_CAMPUS: ObjectKind = ObjectKind::CareerCampus;

// The objects of [`gdo`], by id: 1 and 3 below, 2 player 1's Generator on tiles (8, 2) to (9, 3) (Power +20, for the
// Career Campus's −20), 4 player 2's Deployment Center, far off on tiles (34, 24) to (37, 27).
/// Player 1's Deployment Center, on tiles (2, 2) to (5, 5) (Power +30; its build area reaches tile 17 east and south).
const DC: u32 = 1;
/// Player 1's Barracks, on tiles (12, 3) to (14, 4) (Power −30).
const BARRACKS: u32 = 3;
/// Player 1's bank in [`gdo`], Space Crystals.
const BANK: u32 = 1000;

/// `factions/gdo/structures/barracks.md > Produces`: "Field Engineer: 75 Space Crystals, 160 frames (10 seconds)
/// (placeholder)".
const FIELD_ENGINEER_COST: u32 = 75;
const FIELD_ENGINEER_TICKS: u32 = 160;
/// `factions/gdo/structures/deployment_center.md > Constructs`: "Career Campus: 150 Space Crystals, 320 frames (20
/// seconds) (placeholder)".
const CAREER_CAMPUS_COST: u32 = 150;

/// What a Field Engineer at work on the Barracks restores in a tick, by the repairers' terms (`sc_sim::repair::terms`,
/// design change 0321): its 300 HP in half its 160 frames.
const BARRACKS_RATE: f32 = 3.75;
/// The longest walk, training or repair in these tests, ticks.
const LONGEST: u32 = 1000;

fn flat(w: usize, h: usize) -> String {
    vec![".".repeat(w); h].join("\n")
}

fn tile(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Tile(TilePos::new(x, y))
}

fn id(n: u32) -> ObjectId {
    ObjectId(n)
}

/// Both players play the GDO on a flat 40 × 30 map: player 1 with [`DC`], a Generator and [`BARRACKS`] and [`BANK`]
/// Space Crystals, player 2 with a Deployment Center far off; `more` adds to the scenario after them. Tick 1 has run.
fn gdo(more: impl FnOnce(ScenarioBuilder) -> ScenarioBuilder) -> TestSim {
    let scenario = ScenarioBuilder::new(&flat(40, 30))
        .spawn(ObjectKind::DeploymentCenter, Some(P1), tile(2, 2))
        .spawn(ObjectKind::Generator, Some(P1), tile(8, 2))
        .spawn(ObjectKind::Barracks, Some(P1), tile(12, 3))
        .spawn(ObjectKind::DeploymentCenter, Some(P2), tile(34, 24));
    let mut t = more(scenario).start();
    t.debug_at(
        1,
        DebugCommand::SetResources {
            player: P1,
            crystals: BANK,
            supplies: 0,
            alloys: 0,
            essence: 0,
            flux: 0,
        },
    );
    t.run_until(1);
    t
}

/// What the gate says to player 1's `command` now (`Sim::validate`).
fn gate(sim: &Sim, command: Command) -> Result<(), Reject> {
    sim.validate(&PlayerCommand { player: P1, command })
}

/// Player 1's `command` in the next tick, which runs.
fn next(t: &mut TestSim, command: Command) {
    let tick = t.sim().tick() + 1;
    t.at(tick, P1, command);
    t.step();
}

/// A Train of a Field Engineer at the Barracks: its E.
fn train() -> Command {
    Command::Production {
        objects: vec![id(BARRACKS)],
        command: ProductionCommand::Train(FIELD_ENGINEER),
    }
}

/// Player 1's Space Crystals.
fn crystals(sim: &Sim) -> u32 {
    sim.player(P1).expect("in the game").crystals
}

/// The Field Engineers the Barracks produced so far: (tick, unit).
fn produced(t: &TestSim) -> Vec<(Tick, ObjectId)> {
    let produced = |(tick, e): &(Tick, SimEvent)| match *e {
        SimEvent::UnitProduced { producer, unit } if producer == id(BARRACKS) => Some((*tick, unit)),
        _ => None,
    };
    t.events().iter().filter_map(produced).collect()
}

/// Runs `t` until the Barracks has produced `n` units in all; panics after [`LONGEST`] ticks of waiting for each.
fn until_produced(t: &mut TestSim, n: usize) -> Vec<(Tick, ObjectId)> {
    for _ in 0..n as u32 * LONGEST {
        if produced(t).len() >= n {
            return produced(t);
        }
        t.step();
    }
    panic!("the Barracks produced {} of {n}", produced(t).len());
}

/// `sim` saved, with object `n`'s HP set to `hp`, and loaded: damage that no command deals on demand.
fn damaged(sim: &Sim, n: u32, hp: f32) -> Sim {
    let mut state = serde_json::to_value(sim).unwrap();
    let objects = state["objects"].as_array_mut().unwrap();
    let o = objects.iter_mut().find(|o| o["id"] == n).expect("the object is saved");
    o["hp"] = serde_json::json!(hp);
    serde_json::from_value(state).unwrap()
}

/// One tick of `sim` with player 1's `commands`.
fn step(sim: &mut Sim, commands: Vec<Command>) {
    let commands = commands
        .into_iter()
        .map(|command| PlayerCommand { player: P1, command })
        .collect();
    sim.step(&TickInputs { commands });
}

fn hp(sim: &Sim, n: u32) -> f32 {
    let o = sim.object(id(n)).unwrap_or_else(|| panic!("{n} is on the map"));
    o.hp.expect("it has HP")
}

/// A Repair of `target` to `units`, without Shift.
fn repair(units: &[ObjectId], target: u32) -> Command {
    Command::Order {
        units: units.to_vec(),
        kind: OrderKind::Repair(id(target)),
        queue: false,
    }
}
