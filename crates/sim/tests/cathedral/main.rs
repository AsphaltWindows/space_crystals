//! The Cathedral's training (slice M5-N4a; `factions/cults/structures/cathedral.md > Training`: "Works like the
//! Armory's Training"), the Armory's pool path as data (plan decision T71), re-run on the Cathedral through the gate,
//! one test binary:
//! - `training`: Q pays 75 Space Crystals and reserves the Recruit that entered first, a Disciple comes out of side C
//!   320 ticks later with 50 energy, toward the rally point, and reaches 200 energy 300 s after; X gives the Recruit and
//!   the Space Crystals back; group Q; the refusals;
//! - `pool`: a Release lets a Recruit out unchanged after 4 ticks, Eject All leaves the reserved Recruits inside, the
//!   capacity counts the reserved ones, and a Cathedral destroyed lets everyone out, refunded, a Disciple trained and
//!   waiting for the doorway a Disciple (design changes 0337, 0347, 0348 and 0349); a right-click on it is a Recruit's
//!   Enter, damaged or not;
//! - `disciple`: the Disciple trained keeps its Recruit's damage and augmentations, of which it gets only the MaxHP and
//!   speed parts; it has no weapon, so it never attacks by itself, and a right-click on an enemy moves it;
//! - `temple`: Recruits build the Cathedral only while their player owns a Temple, and losing the Temple cancels
//!   nothing (design change 0335);
//! - `saves`: a game with Disciples training goes on alike after a save and in a replay.
//!
//! Every test runs under the invariants of `TestSim`, the reservation invariant R1 and the energy invariant E1 among
//! them, and checks that no reference to a unit that went inside is left behind.

mod disciple;
mod pool;
mod saves;
mod temple;
mod training;

use sc_sim::testing::{ScenarioBuilder, TestSim, dangling_references};
use sc_sim::{
    Augmentation, BuildingOrientation, Command, DebugCommand, Faction, HoldCommand, ObjectId, ObjectKind, OrderKind,
    PlayerCommand, PlayerId, ProductionCommand, RecruitKind, Reject, SimEvent, SpawnAt, Target, Tick, TilePos, Vec2,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const RECRUIT: ObjectKind = ObjectKind::Recruit;
const DISCIPLE: ObjectKind = ObjectKind::Disciple;
const CATHEDRAL: ObjectKind = ObjectKind::Cathedral;
const TEMPLE: ObjectKind = ObjectKind::Temple;

/// `factions/cults/structures/cathedral.md > Placeholder stats`: "**Disciple**: 1 Recruit, 75 Space Crystals, 320
/// frames (20 seconds)".
const PRICE: u32 = 75;
const FRAMES: Tick = 320;

/// The Cathedral of [`base`].
const C1: ObjectId = ObjectId(1);
/// [`base`]'s Temple, player 1's.
const T1: ObjectId = ObjectId(2);
/// The middle of [`base`]'s Cathedral's exit side, C.
const EXIT: Vec2 = Vec2::new(768.0, 704.0);
/// A rally point west of [`base`]'s Cathedral, beyond its exit side.
const RALLY: Vec2 = Vec2::new(400.0, 704.0);
/// The Space Crystals each player has at the start of [`base`]'s games.
const BANK: u32 = 1000;

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

/// Player 1's Cathedral (1) on a 30 × 22 map, unturned, and their Temple (2) in the north-west corner, which the
/// Cathedral's TechPrerequisite names (`factions/cults/structures/cathedral.md > TechPrerequisite - player owns at least
/// one Temple`) and which keeps player 1 in the game should the Cathedral go. The Cathedral covers tiles (12, 10) to
/// (14, 11), x 768..960 and y 640..768, as the Armory of `tests/armory/` does: its entrance, B, faces east (x = 960);
/// its exit, C, faces west (x = 768, the doorway's middle at [`EXIT`]).
fn base() -> ScenarioBuilder {
    cults(&flat(30, 22))
        .spawn(CATHEDRAL, Some(P1), tile(12, 10))
        .spawn(TEMPLE, Some(P1), tile(2, 2))
}

/// [`base`] with [`BANK`] Space Crystals for player 1, set in tick 1, and `count` of their Recruits ordered into the
/// Cathedral in tick 2, returned once they are all inside, in the order they went in.
fn filled(count: u32) -> (TestSim, Vec<ObjectId>) {
    let mut t = base().start();
    t.without_auto_targeting();
    t.debug_at(1, crystals(P1, BANK));
    let recruits = block(&mut t, 1, count, 5, Vec2::new(1150.0, 704.0));
    t.at(2, P1, enter(&recruits, C1));
    run_checked(&mut t, 2);
    fill(&mut t, C1, recruits.len());
    let order = inside(&t, C1);
    (t, order)
}

/// Runs until `building` holds `count` units, within 200 ticks.
fn fill(t: &mut TestSim, building: ObjectId, count: usize) {
    let start = t.sim().tick();
    while inside(t, building).len() < count {
        assert!(t.sim().tick() < start + 200, "{} are in", inside(t, building).len());
        step_checked(t);
    }
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

fn crystals(player: PlayerId, crystals: u32) -> DebugCommand {
    DebugCommand::SetResources {
        player,
        crystals,
        supplies: 0,
        alloys: 0,
        essence: 0,
        flux: 0,
    }
}

/// `player`'s Space Crystals.
fn bank(t: &TestSim, player: PlayerId) -> u32 {
    t.sim().player(player).expect("in the game").crystals
}

fn enter(units: &[ObjectId], target: ObjectId) -> Command {
    Command::Order {
        units: units.to_vec(),
        kind: OrderKind::Enter(target),
        queue: false,
    }
}

fn production(objects: &[ObjectId], command: ProductionCommand) -> Command {
    Command::Production {
        objects: objects.to_vec(),
        command,
    }
}

/// KindMenu\[`from`\]'s "Q: Train Disciple".
fn train(objects: &[ObjectId], from: RecruitKind) -> Command {
    production(objects, ProductionCommand::TrainFrom { kind: DISCIPLE, from })
}

/// KindMenu\[`from`\]'s "A: Release".
fn release(objects: &[ObjectId], from: RecruitKind) -> Command {
    production(objects, ProductionCommand::Release(from))
}

/// "X: Cancel Production".
fn cancel(objects: &[ObjectId]) -> Command {
    production(objects, ProductionCommand::CancelProduction)
}

fn rally(objects: &[ObjectId], at: Vec2) -> Command {
    production(objects, ProductionCommand::SetRally(Target::Point(at)))
}

fn eject_all(buildings: &[ObjectId]) -> Command {
    Command::Holding {
        objects: buildings.to_vec(),
        command: HoldCommand::EjectAll,
    }
}

/// A Build of a `kind` at `origin`, unturned, to `units`.
fn build(units: &[ObjectId], kind: ObjectKind, origin: TilePos) -> Command {
    let orientation = BuildingOrientation::default();
    Command::Order {
        units: units.to_vec(),
        kind: OrderKind::Build {
            kind,
            origin,
            orientation,
        },
        queue: false,
    }
}

/// Gives unit `unit` the augmentations of a Recruit of kind `kind`, at `tick` (a debug command stands for the Altar,
/// slice M5-N1b's).
fn augment(t: &mut TestSim, tick: Tick, unit: ObjectId, kind: RecruitKind) {
    let augmentation = Augmentation::of(kind);
    t.debug_at(tick, DebugCommand::SetAugmentation { id: unit, augmentation });
}

fn validate(t: &TestSim, command: Command) -> Result<(), Reject> {
    t.sim().validate(&PlayerCommand { player: P1, command })
}

/// The units inside `building`, longest inside first.
fn inside(t: &TestSim, building: ObjectId) -> Vec<ObjectId> {
    t.sim().held_in(building).iter().map(|held| held.object.id).collect()
}

/// The units its queue's entries reserve at `building`, entry by entry.
fn reserved(t: &TestSim, building: ObjectId) -> Vec<Vec<ObjectId>> {
    let o = t.sim().object(building).expect("the building");
    o.production.as_ref().expect("a producer").reserved.clone()
}

/// The units a producer finished so far, as `(tick, producer, unit)`.
fn produced(t: &TestSim) -> Vec<(Tick, ObjectId, ObjectId)> {
    (t.events().iter())
        .filter_map(|(tick, e)| match *e {
            SimEvent::UnitProduced { producer, unit } => Some((*tick, producer, unit)),
            _ => None,
        })
        .collect()
}

/// The units that came out of a building's doorway unchanged so far (Eject All, a Release), as `(tick, unit)`.
fn exits(t: &TestSim) -> Vec<(Tick, ObjectId)> {
    (t.events().iter())
        .filter_map(|(tick, e)| match *e {
            SimEvent::LeftBuilding { unit, .. } => Some((*tick, unit)),
            _ => None,
        })
        .collect()
}

/// Unit `unit`'s energy, in points, on the map or held (`sc_sim::energy`).
fn energy(t: &TestSim, unit: ObjectId) -> Option<f32> {
    let o = (t.sim().object(unit)).or_else(|| t.sim().held_unit(unit).map(|held| &held.object));
    sc_sim::energy::of(o.expect("in the game")).map(|(now, _)| now)
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
