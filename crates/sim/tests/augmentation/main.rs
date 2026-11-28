//! A Cults infantry unit's Augmentations in the game (slice M5-N1a, plan decision T73;
//! `factions/cults/augmentation.md > Increments`, `factions/cults/augmentation.md > Augmentation Effects`), through
//! the gate: a debug command gives them (`DebugCommand::SetAugmentation`, until the Altar does in slice M5-N1b), and
//! the unit's stats follow at once, read through the accessors at its owner's research levels: "No research" for
//! everyone until the Temple (slice M5-N3), other levels through `Research::at`.
//! - `stats`: the amounts, an unarmed unit's and an armed one's, and units that move and strike by them;
//! - `hp`: a unit's HP when its MaxHP changes;
//! - `kinds`: a unit trained into infantry keeps its Augmentations;
//! - `gate`: the debug command's refusals;
//! - `raw_reads`: no code reads the stats past the accessors (`docs/architecture/object-model-state.md > Object model > Augmented stats`).
//!
//! The table at every level, and a kind change into a vehicle, are the sim's unit tests' (`augmentation/tests.rs`,
//! `sim/kinds/tests.rs`). Every test runs under the invariants of `TestSim`.

mod gate;
mod hp;
mod kinds;
mod raw_reads;
mod stats;

use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{
    Augmentation, Command, DebugCommand, Faction, Object, ObjectId, ObjectKind, OrderKind, PlayerId, RecruitKind,
    SpawnAt, Tick, Vec2,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

fn id(n: u32) -> ObjectId {
    ObjectId(n)
}

fn point(x: f32, y: f32) -> SpawnAt {
    SpawnAt::Point(Vec2::new(x, y))
}

/// Both players play the Cults, on `w` × `h` open tiles.
fn cults(w: usize, h: usize) -> ScenarioBuilder {
    ScenarioBuilder::new(&vec![".".repeat(w); h].join("\n"))
        .faction(P1, Faction::Cults)
        .faction(P2, Faction::Cults)
}

/// The tick the next step runs.
fn next(t: &TestSim) -> Tick {
    t.sim().tick() + 1
}

/// Gives `unit` the Augmentations of a unit of kind `kind` in the next tick, and runs it.
fn augment(t: &mut TestSim, unit: ObjectId, kind: RecruitKind) {
    let augmentation = Augmentation::of(kind);
    t.debug_at(next(t), DebugCommand::SetAugmentation { id: unit, augmentation });
    t.step();
}

fn object(t: &TestSim, unit: ObjectId) -> &Object {
    t.sim().object(unit).expect("on the map")
}

/// `unit`'s HP and MaxHP, its Augmentations' at its owner's levels.
fn hp(t: &TestSim, unit: ObjectId) -> (f32, f32) {
    let o = object(t, unit);
    (
        o.hp.expect("destructible"),
        o.max_hp_with(t.sim().research()).expect("destructible"),
    )
}

fn attack(units: &[ObjectId], target: ObjectId) -> Command {
    Command::Order {
        units: units.to_vec(),
        kind: OrderKind::Attack(target),
        queue: false,
    }
}

/// The units of `kind` on the map.
fn of_kind(t: &TestSim, kind: ObjectKind) -> Vec<ObjectId> {
    t.sim().objects().filter(|o| o.kind == kind).map(|o| o.id).collect()
}
