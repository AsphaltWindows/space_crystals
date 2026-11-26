//! The Altar (slice M5-N1b, plan decisions T70 and T71; `factions/cults/augmentation.md > Altar`,
//! `factions/cults/structures/README.md > Choosing Recruits, Release and Eject All`, `factions/cults/economy.md > Cults
//! Building Mechanics > Evacuation on Destruction`; design changes 0332, 0336, 0337 and 0348): Cults infantry go in and
//! are augmented there, by a serial queue paid as it is queued, through the gate, one test binary:
//! - `orders`: Enhanced Constitution on a Militant, 25 Space Crystals and 48 ticks (design change 0391), out of side C
//!   at 60 MaxHP with its damage; Both on a Recruit, 50 and 96; Improved Metabolism on a hurt Recruit; the unit of the
//!   type and state that entered first first;
//! - `refusals`: an Augmentation the unit has, no such unit, a full queue, too few Space Crystals, a building that
//!   takes no order;
//! - `release`: a Release, free, lets a unit of a type and a state out unchanged 4 ticks later, a fully augmented one
//!   too, and Eject All the units no order reserves;
//! - `cancel`: X gives the last order's Space Crystals back and its unit to the pool;
//! - `destruction`: everyone comes out, unaugmented mid-augmentation, every order refunded;
//! - `entering`: the four infantry types go in at the entrance, on a right-click too, even with the Altar damaged;
//! - `training`: a Recruit augmented here and trained at the Armory keeps its Augmentation;
//! - `saves`: a game with orders under way goes on alike after a save and in a replay.
//!
//! Every test runs under the invariants of `TestSim`, the reservation invariant R1 among them
//! (`testing/invariants/reservations.rs`), and checks that no reference to a unit that went inside is left behind.

mod cancel;
mod destruction;
mod entering;
mod orders;
mod refusals;
mod release;
mod saves;
mod training;

use sc_sim::testing::{ScenarioBuilder, TestSim, dangling_references};
use sc_sim::{
    AugmentOrder, Augmentation, Command, DebugCommand, Faction, HoldCommand, ObjectId, ObjectKind, OrderKind,
    PlayerCommand, PlayerId, ProductionCommand, RecruitKind, Reject, SimEvent, SpawnAt, Target, Tick, TilePos, Vec2,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const RECRUIT: ObjectKind = ObjectKind::Recruit;
const MILITANT: ObjectKind = ObjectKind::Militant;
const GUNNER: ObjectKind = ObjectKind::Gunner;
const DISCIPLE: ObjectKind = ObjectKind::Disciple;
const ALTAR: ObjectKind = ObjectKind::Altar;

/// The Altar of [`base`].
const A1: ObjectId = ObjectId(1);
/// The middle of [`base`]'s Altar's exit side, C.
const EXIT: Vec2 = Vec2::new(768.0, 704.0);
/// A rally point west of [`base`]'s Altar, beyond its exit side.
const RALLY: Vec2 = Vec2::new(400.0, 704.0);
/// The Space Crystals player 1 has once [`filled`] set them.
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

/// Player 1's Altar (1) on a 30 × 22 map, unturned: it covers tiles (12, 10) to (14, 11), x 768..960 and y 640..768.
/// Its entrance, B, faces east (x = 960); its exit, C, faces west (x = 768, the doorway's middle at [`EXIT`]).
fn base() -> ScenarioBuilder {
    cults(&flat(30, 22)).spawn(ALTAR, Some(P1), tile(12, 10))
}

/// [`base`] with [`BANK`] Space Crystals for player 1, and its units of `kinds` east of the Altar, both set in tick 1,
/// which has run: the game and the units' ids, in that order.
fn spawned(kinds: &[ObjectKind]) -> (TestSim, Vec<ObjectId>) {
    spawned_on(base(), kinds)
}

/// [`spawned`] on `scenario`, a [`base`] with more objects.
fn spawned_on(scenario: ScenarioBuilder, kinds: &[ObjectKind]) -> (TestSim, Vec<ObjectId>) {
    let mut t = scenario.start();
    t.without_auto_targeting();
    t.debug_at(1, crystals(P1, BANK));
    let units = block(&mut t, 1, kinds, 5, Vec2::new(1150.0, 704.0));
    run_checked(&mut t, 1);
    (t, units)
}

/// Orders `units` into the Altar in the next tick and runs until they are all inside: the units inside then, longest
/// inside first.
fn go_in(t: &mut TestSim, units: &[ObjectId]) -> Vec<ObjectId> {
    let count = inside(t, A1).len() + units.len();
    let now = t.sim().tick() + 1;
    t.at(now, P1, enter(units, A1));
    run_checked(t, now);
    fill(t, A1, count);
    inside(t, A1)
}

/// [`spawned`] units of `kinds`, those of `augmented` given the Augmentations of that kind (a debug command, in tick
/// 2), all ordered into the Altar in tick 2; returned once they are all inside, in the order they went in.
fn filled(kinds: &[ObjectKind], augmented: &[(usize, RecruitKind)]) -> (TestSim, Vec<ObjectId>) {
    let (mut t, units) = spawned(kinds);
    for &(i, kind) in augmented {
        set_augmentation(&mut t, 2, units[i], kind);
    }
    let order = go_in(&mut t, &units);
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

/// Player 1's units of `kinds`, in rows of `per_row`, 32 su apart, the first row's middle at `front`, the rows behind
/// it to the east; their ids, from the next free one.
fn block(t: &mut TestSim, tick: Tick, kinds: &[ObjectKind], per_row: u32, front: Vec2) -> Vec<ObjectId> {
    let first = next_id(t);
    for (i, &kind) in (0..).zip(kinds) {
        let (row, col) = (i / per_row, i % per_row);
        let y = front.y + 32.0 * (col as f32 - (per_row - 1) as f32 / 2.0);
        t.spawn_unit_at(tick, kind, P1, Vec2::new(front.x + 32.0 * row as f32, y));
    }
    (0..kinds.len() as u32).map(|i| id(first + i)).collect()
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

/// AugmentMenu\[`kind`, `from`\]'s order `order`: the panel's type tile, state tile and order (W, Q, Q: a Plain
/// Militant's Enhanced Constitution).
fn augment(objects: &[ObjectId], kind: ObjectKind, from: RecruitKind, order: AugmentOrder) -> Command {
    production(objects, ProductionCommand::Augment { kind, from, order })
}

/// AugmentMenu\[`kind`, `from`\]'s "A: Release".
fn release(objects: &[ObjectId], kind: ObjectKind, from: RecruitKind) -> Command {
    production(objects, ProductionCommand::ReleaseOf { kind, from })
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

/// Gives unit `unit` the Augmentations of a unit of kind `kind`, at `tick` (a debug command: units augmented before
/// they go in).
fn set_augmentation(t: &mut TestSim, tick: Tick, unit: ObjectId, kind: RecruitKind) {
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

/// The units inside `building` of type `kind` with the Augmentations of kind `state`, longest inside first.
fn inside_of(t: &TestSim, building: ObjectId, kind: ObjectKind, state: RecruitKind) -> Vec<ObjectId> {
    (t.sim().held_in(building).iter())
        .filter(|held| held.object.kind == kind && held.object.augmentation.kind() == state)
        .map(|held| held.object.id)
        .collect()
}

/// The units its queue's entries reserve at `building`, entry by entry.
fn reserved(t: &TestSim, building: ObjectId) -> Vec<Vec<ObjectId>> {
    let o = t.sim().object(building).expect("the building");
    o.production.as_ref().expect("a producer").reserved.clone()
}

/// The Augmentations of unit `unit`, on the map or inside.
fn augmentation(t: &TestSim, unit: ObjectId) -> Augmentation {
    let on_map = t.sim().object(unit).map(|o| o.augmentation);
    on_map
        .or_else(|| t.sim().held_unit(unit).map(|held| held.object.augmentation))
        .unwrap_or_else(|| panic!("no unit {unit:?}"))
}

/// Unit `unit`'s HP and MaxHP, its Augmentations' (at no research), on the map.
fn hp(t: &TestSim, unit: ObjectId) -> (f32, f32) {
    let o = t.sim().object(unit).unwrap_or_else(|| panic!("{unit:?} on the map"));
    let max = o.max_hp_with(t.sim().research());
    (o.hp.expect("destructible"), max.expect("destructible"))
}

/// The units the Altar finished so far, as `(tick, producer, unit)`.
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

/// Whether `pos` is in the doorway of [`base`]'s Altar's side C, half inside it, as a unit that just came out.
fn in_the_exit(pos: Vec2) -> bool {
    pos.x > EXIT.x - 0.5 && pos.x < EXIT.x + 12.5 && (pos.y - EXIT.y).abs() <= 52.0
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

/// Player 2's Recruit, spawned beside player 1's unit `unit`, hits it until its HP is `below` or lower, and is
/// removed: no command hurts a unit at will. Returns the HP it was left with.
fn hurt(t: &mut TestSim, unit: ObjectId, below: f32) -> f32 {
    let now = t.sim().tick() + 1;
    let at = t.sim().object(unit).expect("on the map").pos + Vec2::new(28.0, 0.0);
    t.spawn_unit_at(now, RECRUIT, P2, at);
    run_checked(t, now);
    let raider = (t.sim().objects())
        .find(|o| o.owner == Some(P2) && o.kind == RECRUIT)
        .expect("the raider")
        .id;
    t.order_at(now + 1, P2, &[raider], OrderKind::Attack(unit), false);
    run_checked(t, now + 1);
    while hp(t, unit).0 > below {
        assert!(t.sim().tick() < now + 300, "never hit enough");
        step_checked(t);
    }
    remove_at(t, t.sim().tick() + 1, raider);
    step_checked(t);
    hp(t, unit).0
}
