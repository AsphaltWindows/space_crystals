//! The Armory's training (slice M5-F2, plan decision T61; `factions/cults/structures/armory.md > Training`,
//! `factions/cults/structures/armory.md > ObjectInterfaceState[Armory]`, `factions/cults/structures/README.md >
//! Choosing Recruits, Release and Eject All`; design changes 0332, 0337, 0347 and 0348), through the gate, one test
//! binary:
//! - `training`: Q pays 20 Space Crystals and reserves the Recruit that entered first, a Militant comes out of side C
//!   32 ticks later toward the rally point, X gives the Recruit and the Space Crystals back, and the refusals;
//! - `gunner`: W, refused while its player owns no Gunworks, trains a Gunner for 40 Space Crystals in 32 ticks once one
//!   stands, and the Gunners queued as the last Gunworks is lost complete (slice M5-G2);
//! - `release`: a Release lets a Recruit of a kind out unchanged, 4 ticks after it is queued (0337's Verification);
//! - `eject`: Eject All leaves the reserved Recruits inside;
//! - `capacity`: the capacity counts the reserved Recruits too, and a trained unit drops its load (0347); a right-click
//!   on a full damaged Armory is a Repair (0378);
//! - `destruction`: an Armory destroyed mid-training lets its Recruits out as Recruits and refunds every entry, but a
//!   Militant trained and waiting for the doorway comes out a Militant, unrefunded (design change 0349);
//! - `control`: the Unit Control a Militant uses is its Recruit's, an orphan's if the Recruit's Hall is gone;
//! - `saves`: a game with training under way goes on alike after a save and in a replay.
//!
//! The units consumed into a unit of several Recruits, a hurt Recruit's damage and a vehicle's full HP are the sim's
//! unit tests' (`sim/producers/pool/tests.rs`), which can give a unit type other numbers. Every test runs under the
//! invariants of `TestSim`, the reservation invariant R1 among them (`testing/invariants/reservations.rs`), and checks
//! that no reference to a unit that went inside is left behind.

mod capacity;
mod control;
mod destruction;
mod eject;
mod gunner;
mod release;
mod saves;
mod training;

use sc_sim::testing::{ScenarioBuilder, TestSim, dangling_references};
use sc_sim::{
    Augmentation, Command, DebugCommand, Faction, HoldCommand, ObjectId, ObjectKind, OrderKind, PlayerCommand,
    PlayerId, ProductionCommand, RecruitKind, Reject, SimEvent, SpawnAt, Target, Tick, TilePos, Vec2,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const RECRUIT: ObjectKind = ObjectKind::Recruit;
const MILITANT: ObjectKind = ObjectKind::Militant;
const ARMORY: ObjectKind = ObjectKind::Armory;

/// The Armory of [`base`].
const A1: ObjectId = ObjectId(1);
/// The middle of [`base`]'s Armory's exit side, C.
const EXIT: Vec2 = Vec2::new(768.0, 704.0);
/// A rally point west of [`base`]'s Armory, beyond its exit side.
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

/// Player 1's Armory (1) on a 30 × 22 map, unturned: it covers tiles (12, 10) to (14, 11), x 768..960 and y 640..768.
/// Its entrance, B, faces east (x = 960); its exit, C, faces west (x = 768, the doorway's middle at [`EXIT`]).
fn base() -> ScenarioBuilder {
    cults(&flat(30, 22)).spawn(ARMORY, Some(P1), tile(12, 10))
}

/// [`base`] with [`BANK`] Space Crystals for player 1, set in tick 1, and `count` of their Recruits ordered into the
/// Armory in tick 2, returned once they are all inside, in the order they went in.
fn filled(count: u32) -> (TestSim, Vec<ObjectId>) {
    let mut t = base().start();
    t.without_auto_targeting();
    t.debug_at(1, crystals(P1, BANK));
    let recruits = block(&mut t, 1, count, 5, Vec2::new(1150.0, 704.0));
    t.at(2, P1, enter(&recruits, A1));
    run_checked(&mut t, 2);
    fill(&mut t, A1, recruits.len());
    let order = inside(&t, A1);
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

/// KindMenu\[`from`\]'s "Q: Train Militant".
fn train(objects: &[ObjectId], from: RecruitKind) -> Command {
    production(objects, ProductionCommand::TrainFrom { kind: MILITANT, from })
}

/// KindMenu\[`from`\]'s "A: Release".
fn release(objects: &[ObjectId], from: RecruitKind) -> Command {
    production(objects, ProductionCommand::Release(from))
}

/// "X: Cancel".
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

/// Gives unit `unit` the augmentations of a Recruit of kind `kind`, at `tick` (a debug command stands for the
/// Augmentation Center, slice M5-G's).
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
