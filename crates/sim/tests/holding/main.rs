//! Buildings that hold units (slice M5-F1, plan decision T60; `systems/controls/commands.md > Enter`, its TargetObject
//! "the structure being entered" since design change 0319; `factions/cults/structures/armory.md > EntranceSide - one
//! short side (B)`, `factions/cults/structures/armory.md > ExitSide - opposite short side (C)`,
//! `factions/cults/structures/armory.md > InternalRecruitCapacity - 10`, `factions/cults/structures/armory.md > Eject
//! All`; `factions/cults/economy.md > Cults Building Mechanics > Evacuation on Destruction`), on the Armory, through
//! the gate, one test binary:
//! - `entering`: ten Recruits file in, the eleventh's Enter fails, the gate, the right-click;
//! - `eject`: Eject All streams the pool out of side C, one at a time;
//! - `evacuation`: everyone inside comes out alive around the footprint, in a fixed order;
//! - `loads`: a Recruit keeps what it carries inside, and out again;
//! - `saves`: a game with units inside replayed, saved and loaded, and the hash.
//!
//! Every test runs under the invariants of `TestSim` (no two units overlap, none stands in a footprint but a unit in
//! the doorway its building let it into), and checks that no reference to a unit that went inside is left behind.

mod eject;
mod entering;
mod evacuation;
mod loads;
mod saves;

use sc_sim::testing::{ScenarioBuilder, TestSim, dangling_references};
use sc_sim::{
    Command, DebugCommand, Faction, HoldCommand, ObjectId, ObjectKind, OrderKind, PlayerCommand, PlayerId, Reject,
    SimEvent, SpawnAt, Tick, TilePos, Vec2,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const RECRUIT: ObjectKind = ObjectKind::Recruit;
const ARMORY: ObjectKind = ObjectKind::Armory;

/// The Armory of [`base`].
const A1: ObjectId = ObjectId(1);
/// [`base`]'s Armory's entry path's spot, the middle of its side B, and the middle of its exit side, C.
const ENTRY: Vec2 = Vec2::new(960.0, 704.0);
const EXIT: Vec2 = Vec2::new(768.0, 704.0);
/// A Recruit's half extents (`factions/cults/units/recruit.md`: 24 x 24).
const HALF: Vec2 = Vec2::new(12.0, 12.0);

fn flat(w: usize, h: usize) -> String {
    vec![".".repeat(w); h].join("\n")
}

fn tile(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Tile(TilePos::new(x, y))
}

fn id(n: u32) -> ObjectId {
    ObjectId(n)
}

/// Both players play the Cults.
fn cults(map: &str) -> ScenarioBuilder {
    ScenarioBuilder::new(map)
        .faction(P1, Faction::Cults)
        .faction(P2, Faction::Cults)
}

/// Player 1's Armory (1) on a 30 × 22 map, unturned: it covers tiles (12, 10) to (14, 11), x 768..960 and y 640..768.
/// Its entrance, B, faces east (x = 960, the entry path's spot at [`ENTRY`]); its exit, C, faces west (x = 768, the
/// doorway's middle at [`EXIT`]).
fn base() -> ScenarioBuilder {
    cults(&flat(30, 22)).spawn(ARMORY, Some(P1), tile(12, 10))
}

/// `count` Recruits of player 1's in rows of `per_row`, 32 su apart, the first row's middle at `front`, the rows behind
/// it to the east; their ids, from the next free one.
fn block(t: &mut TestSim, tick: Tick, count: u32, per_row: u32, front: Vec2) -> Vec<ObjectId> {
    let first = next_id(t);
    for i in 0..count {
        let (row, col) = (i / per_row, i % per_row);
        let y = front.y + 32.0 * (col as f32 - (per_row - 1) as f32 / 2.0);
        t.spawn_unit_at(tick, RECRUIT, P1, Vec2::new(front.x + 32.0 * row as f32, y));
    }
    (0..count).map(|i| id(first + i)).collect()
}

/// The id the next object will have, with no spawn scheduled yet: the objects and the held units have the ids below
/// it.
fn next_id(t: &TestSim) -> u32 {
    let on_map = t.sim().objects().map(|o| o.id.0);
    let held = t.sim().held().map(|held| held.object.id.0);
    on_map.chain(held).max().unwrap_or(0) + 1
}

fn enter(units: &[ObjectId], target: ObjectId) -> Command {
    Command::Order {
        units: units.to_vec(),
        kind: OrderKind::Enter(target),
        queue: false,
    }
}

fn eject_all(buildings: &[ObjectId]) -> Command {
    Command::Holding {
        objects: buildings.to_vec(),
        command: HoldCommand::EjectAll,
    }
}

fn validate(t: &TestSim, command: Command) -> Result<(), Reject> {
    t.sim().validate(&PlayerCommand { player: P1, command })
}

/// The units inside `building`, longest inside first.
fn inside(t: &TestSim, building: ObjectId) -> Vec<ObjectId> {
    t.sim().held_in(building).iter().map(|held| held.object.id).collect()
}

/// The units that went into a building so far, as `(tick, unit)`.
fn entries(t: &TestSim) -> Vec<(Tick, ObjectId)> {
    (t.events().iter())
        .filter_map(|(tick, e)| match e {
            SimEvent::EnteredBuilding { unit, .. } => Some((*tick, *unit)),
            _ => None,
        })
        .collect()
}

/// The units that came out of a building's doorway so far, as `(tick, unit)`.
fn exits(t: &TestSim) -> Vec<(Tick, ObjectId)> {
    (t.events().iter())
        .filter_map(|(tick, e)| match e {
            SimEvent::LeftBuilding { unit, .. } => Some((*tick, *unit)),
            _ => None,
        })
        .collect()
}

/// The Enters that failed so far, as `(unit, full)`.
fn failed(t: &TestSim) -> Vec<(ObjectId, bool)> {
    (t.events().iter())
        .filter_map(|(_, e)| match e {
            SimEvent::EnterFailed { unit, full, .. } => Some((*unit, *full)),
            _ => None,
        })
        .collect()
}

/// One tick, then: nothing names a unit that left the map.
fn step_checked(t: &mut TestSim) {
    t.step();
    assert_eq!(
        dangling_references(t.sim()),
        [],
        "tick {}\n{}",
        t.sim().tick(),
        t.ascii(None)
    );
}

fn run_checked(t: &mut TestSim, until: Tick) {
    while t.sim().tick() < until {
        step_checked(t);
    }
}

/// [`run_checked`] for `ticks` more ticks.
fn run_for(t: &mut TestSim, ticks: Tick) {
    let until = t.sim().tick() + ticks;
    run_checked(t, until);
}

/// Removes object `id` at `tick` (a debug removal stands for a destruction).
fn remove_at(t: &mut TestSim, tick: Tick, id: ObjectId) {
    t.debug_at(tick, DebugCommand::Remove { id });
}
