//! The Chop Shop and the Seeker (slice M5-N6; `factions/cults/structures/chop_shop.md`,
//! `factions/cults/units/seeker.md`): the Armory's pool path as data (plan decision T71), whose trained vehicle is two
//! Recruits and drives out of the doorway of side C on its FixedTurnRadiusMovement (slice M5-V2), one test binary:
//! - `training`: Q needs two free Recruits of its kind (design change 0319's Verification), pays 75 Space Crystals and
//!   reserves the two inside the longest; 320 ticks later the first comes out a Seeker, the second gone into it, and
//!   drives to the rally point; ten Recruits queue five Seekers and a sixth is refused; X gives both Recruits and the
//!   Space Crystals back; a Release lets a Recruit out as at the Airstrip; a right-click on it is a Recruit's Enter;
//! - `exit`: the Seeker appears half inside the middle of side C, facing out, its box turned with it whichever way the
//!   building is turned, moved in from a unit in front of the doorway, and drives straight out with its rally point
//!   cleared, a second one then stalling in the doorway behind the first (design change 0447);
//! - `rally` (slice M5-S8, design changes 0446 and 0447): a Chop Shop starts with a rally point 3 grid units out of
//!   side C, however it is turned, its site from its placement on; three Seekers drive to it, the doorway never
//!   stalling; a moved rally point takes the next ones there; one off the map is drawn in to its edge; a save with
//!   none loads with none;
//! - `seeker`: two damaged, augmented Recruits make a Seeker at full HP with no augmentation (design change 0330); it
//!   counts one Unit Control against each of its Recruits' Halls, half of it an orphan once one Hall is gone;
//! - `combat`: it kills a Militant while driving past, its turret firing on a Move; rear hits do more than frontal ones
//!   (DirectionalArmor); it drives round Rugged ground, never onto it; the Sparrow's case is `tests/airstrip/combat.rs`;
//! - `destruction`: a destroyed Chop Shop evacuates its Recruits, refunded, and a Seeker finished and waiting for the
//!   doorway comes out a Seeker inside the footprint;
//! - `normal_game`: from the Cults' start, Recruits build an Armory and then a Chop Shop, which trains a Seeker, and
//!   the Airstrip and the Machine Shop are no longer behind a missing Chop Shop;
//! - `saves`: a game with Seekers training and driving out goes on alike after a save and in a replay.
//!
//! Every test runs under the invariants of `TestSim`, the wheeled invariant W1 and the reservation invariant R1 among
//! them, and checks that no reference to a unit that went inside is left behind.

mod combat;
mod destruction;
mod exit;
mod normal_game;
mod rally;
mod saves;
mod seeker;
mod training;

use sc_sim::shape::Shape;
use sc_sim::testing::{ScenarioBuilder, TestSim, dangling_references};
use sc_sim::{
    Augmentation, Command, DebugCommand, Faction, HoldCommand, ObjectId, ObjectKind, OrderKind, PlayerCommand,
    PlayerId, ProductionCommand, RecruitKind, Reject, SimEvent, SpawnAt, Target, Tick, TilePos, Vec2,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const RECRUIT: ObjectKind = ObjectKind::Recruit;
const SEEKER: ObjectKind = ObjectKind::Seeker;
const CHOP_SHOP: ObjectKind = ObjectKind::ChopShop;

/// `factions/cults/structures/chop_shop.md > Placeholder stats`: "**Seeker**: 2 Recruits, 75 Space Crystals, 320
/// frames (20 seconds)".
const PRICE: u32 = 75;
const FRAMES: Tick = 320;

/// The Seeker's MaxHP and ForwardMaxSpeed (`factions/cults/units/seeker.md > Placeholder stats`).
const SEEKER_HP: f32 = 120.0;
const TOP: f32 = 8.0;

/// The Chop Shop of [`base`].
const CS: ObjectId = ObjectId(1);
/// The middle of [`base`]'s Chop Shop's exit side, C, facing west. (B, its entrance, faces east, its middle at
/// (960, 736).)
const EXIT: Vec2 = Vec2::new(768.0, 736.0);
/// A rally point west of [`base`]'s Chop Shop, beyond its exit side.
const RALLY: Vec2 = Vec2::new(400.0, 736.0);
/// The Space Crystals player 1 has from tick 1 of [`filled`]'s games.
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

/// Player 1's Chop Shop (1) on `map`, unturned, and their Armory (2) in the north-west corner: the Chop Shop's
/// TechPrerequisite (`factions/cults/structures/chop_shop.md > TechPrerequisite - player owns at least one Armory`),
/// which keeps player 1 in the game should the Chop Shop go. The Chop Shop covers tiles (12, 10) to (14, 12), x
/// 768..960 and y 640..832: its entrance, B, faces east (x = 960); its exit, C, faces west (x = 768, [`EXIT`]).
fn on(map: &str) -> ScenarioBuilder {
    cults(map)
        .spawn(CHOP_SHOP, Some(P1), tile(12, 10))
        .spawn(ObjectKind::Armory, Some(P1), tile(2, 2))
}

/// [`on`] a flat 30 × 22 map.
fn base() -> ScenarioBuilder {
    on(&flat(30, 22))
}

/// `scenario` with [`BANK`] Space Crystals for player 1, set in tick 1, and `count` of their Recruits ordered into the
/// Chop Shop in tick 2, returned once they are all inside, in the order they went in; auto-targeting off.
fn filled_on(scenario: ScenarioBuilder, count: u32) -> (TestSim, Vec<ObjectId>) {
    let mut t = scenario.start();
    t.without_auto_targeting();
    t.debug_at(1, crystals(P1, BANK));
    let recruits = block(&mut t, 1, count, 5, Vec2::new(1150.0, 736.0));
    t.at(2, P1, enter(&recruits, CS));
    run_checked(&mut t, 2);
    fill(&mut t, CS, recruits.len());
    let order = inside(&t, CS);
    (t, order)
}

/// [`filled_on`] [`base`].
fn filled(count: u32) -> (TestSim, Vec<ObjectId>) {
    filled_on(base(), count)
}

/// Runs until `building` holds `count` units, within 300 ticks.
fn fill(t: &mut TestSim, building: ObjectId, count: usize) {
    let start = t.sim().tick();
    while inside(t, building).len() < count {
        assert!(t.sim().tick() < start + 300, "{} are in", inside(t, building).len());
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

/// KindMenu\[`from`\]'s "**Q: Build Seeker**".
fn train(objects: &[ObjectId], from: RecruitKind) -> Command {
    production(objects, ProductionCommand::TrainFrom { kind: SEEKER, from })
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

/// (M5-S8) Clears `building`'s rally point, its default one among them: Set Rally Point on the building itself, as a
/// right-click on it or C and a click on it does (design change 0447: "clear it by rallying on the building itself").
fn clear_rally(building: ObjectId) -> Command {
    production(&[building], ProductionCommand::SetRally(Target::Object(building)))
}

fn eject_all(buildings: &[ObjectId]) -> Command {
    Command::Holding {
        objects: buildings.to_vec(),
        command: HoldCommand::EjectAll,
    }
}

/// Gives unit `unit` the augmentations of a Recruit of kind `kind`, at `tick` (a debug command stands for the Altar,
/// slice M5-N1b's).
fn augment(t: &mut TestSim, tick: Tick, unit: ObjectId, kind: RecruitKind) {
    let augmentation = Augmentation::of(kind);
    t.debug_at(tick, DebugCommand::SetAugmentation { id: unit, augmentation });
}

/// Whether the gate takes `command` from player 1 now.
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

/// Every hit so far: its tick, its attacker, its target and the HP it took.
fn hits(t: &TestSim) -> Vec<(Tick, ObjectId, ObjectId, f32)> {
    (t.events().iter())
        .filter_map(|(tick, e)| match *e {
            SimEvent::Hit {
                attacker,
                target,
                damage,
                ..
            } => Some((*tick, attacker, target, damage)),
            _ => None,
        })
        .collect()
}

/// Runs until `producer` has finished a unit more, within `limit` ticks; returns it, with its tick.
fn until_produced(t: &mut TestSim, producer: ObjectId, limit: Tick) -> (Tick, ObjectId) {
    let of = |t: &TestSim| {
        produced(t)
            .into_iter()
            .filter(|&(_, by, _)| by == producer)
            .collect::<Vec<_>>()
    };
    let (start, before) = (t.sim().tick(), of(t).len());
    while of(t).len() == before {
        assert!(t.sim().tick() < start + limit, "nothing produced");
        step_checked(t);
    }
    let (tick, _, unit) = of(t)[before];
    (tick, unit)
}

/// Unit `unit`'s position.
fn pos(t: &TestSim, unit: ObjectId) -> Vec2 {
    t.sim().object(unit).expect("on the map").pos
}

/// Object `id`'s shape where it stands (`sc_sim::shape`): a Seeker's Silhouette turned with its facing.
fn shape(t: &TestSim, id: ObjectId) -> Shape {
    Shape::of(t.sim().object(id).expect("on the map"))
}

/// Whether `a` and `b` differ by at most `tolerance`.
fn near(a: f32, b: f32, tolerance: f32) -> bool {
    (a - b).abs() <= tolerance
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
