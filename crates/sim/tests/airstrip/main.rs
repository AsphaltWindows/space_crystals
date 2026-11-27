//! The Airstrip and the Sparrow (slice M5-N7; `factions/cults/structures/airstrip.md`,
//! `factions/cults/units/sparrow.md`): the Armory's pool path as data (plan decision T71), whose trained aircraft takes
//! off along the runway (`ExitRule::Runway`, `production::Runway`), one test binary:
//! - `takeoff`: a Sparrow appears over the runway's B end facing C at IdleSpeed, speeds up along the runway and, past
//!   C, flies to the rally point; with none it circles 2 grid units past C (design change 0354); it never pauses: an
//!   occupied doorway holds the Recruits back, never a Sparrow (`core/entities.md > Structure Type > Exit Doorway`:
//!   "Aircraft [...] are not covered by this rule");
//! - `training`: Q pays 75 Space Crystals and reserves the Recruit inside the longest (design change 0332), which comes
//!   out a Sparrow at full HP, its augmentations lost (design change 0330); X gives both back; Eject All lets the
//!   Recruits out of side C's doorway, onto the ground; a right-click on the Airstrip is a Recruit's Enter, and a
//!   Sparrow's Move: it never lands;
//! - `combat`: it strafes a group of Agents, which can't hit it; Militants, Guards, the Laser Turret and (slice M5-N6)
//!   the Seeker hit it, and Peacekeepers, Legionnaires and Agents can't (their TargetDomain is Ground);
//! - `destruction`: a destroyed Airstrip evacuates its pool, refunded, and leaves the Sparrows in the air alone, one
//!   still on its takeoff run, which flies on to its rally point, or circles 2 grid units past C without one, never
//!   standing;
//! - `saves`: a game with a Sparrow on its takeoff run goes on alike after a save and in a replay.
//!
//! Every test runs under the invariants of `TestSim`, the Glider invariant G1 and the reservation invariant R1 among
//! them, and checks that no reference to a unit that went inside is left behind.

mod combat;
mod destruction;
mod saves;
mod takeoff;
mod training;

use sc_sim::testing::{ScenarioBuilder, TestSim, dangling_references};
use sc_sim::{
    Augmentation, Command, DebugCommand, Faction, HoldCommand, ObjectId, ObjectKind, OrderKind, PlayerId,
    ProductionCommand, RecruitKind, SimEvent, SpawnAt, Target, Tick, TilePos, Vec2, math,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const RECRUIT: ObjectKind = ObjectKind::Recruit;
const SPARROW: ObjectKind = ObjectKind::Sparrow;
const AIRSTRIP: ObjectKind = ObjectKind::Airstrip;

/// `factions/cults/structures/airstrip.md > Placeholder stats`: "**Sparrow**: 1 Recruit, 75 Space Crystals, 320 frames
/// (20 seconds)".
const PRICE: u32 = 75;
const FRAMES: Tick = 320;

/// The Sparrow's GliderMovement (`factions/cults/units/sparrow.md`): IdleSpeed and MaxSpeed, su a tick; its idle
/// circle's radius, 1 grid unit (design change 0354).
const IDLE_SPEED: f32 = 4.0;
const MAX_SPEED: f32 = 10.0;
const IDLE_RADIUS: f32 = 64.0;

/// The Airstrip of [`base`].
const A1: ObjectId = ObjectId(1);
/// The middle of [`base`]'s Airstrip's side C, facing west: the end its aircraft take off toward and its Recruits'
/// exit. (B, its entrance and the runway's start, faces east, its middle at (1024, 704).)
const C_END: Vec2 = Vec2::new(768.0, 704.0);
/// Where a Sparrow appears (40 × 40): wholly over the runway, against its B end, centred across it.
const START: Vec2 = Vec2::new(1004.0, 704.0);
/// The point a Sparrow circles without a rally point: 2 grid units past the middle of side C.
const CIRCLE: Vec2 = Vec2::new(640.0, 704.0);
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

/// Player 1's Airstrip (1) on `map`, unturned, and their Storage (2) in the north-west corner, which keeps player 1 in
/// the game should the Airstrip go. The Airstrip covers tiles (12, 10) to (15, 11), x 768..1024 and y 640..768: 4 wide
/// and 2 tall, as "Size 2x4 (a runway)" is given in the default orientation; its entrance and the runway's start, B,
/// face east (x = 1024, its middle at (1024, 704)); its far end and its Recruits' exit, C, face west (x = 768,
/// [`C_END`]).
fn on(map: &str) -> ScenarioBuilder {
    cults(map)
        .spawn(AIRSTRIP, Some(P1), tile(12, 10))
        .spawn(ObjectKind::Storage, Some(P1), tile(2, 2))
}

/// [`on`] a flat 30 × 22 map.
fn base() -> ScenarioBuilder {
    on(&flat(30, 22))
}

/// `scenario` with [`BANK`] Space Crystals for player 1, set in tick 1, and `count` of their Recruits ordered into the
/// Airstrip in tick 2, returned once they are all inside, in the order they went in; auto-targeting off.
fn filled_on(scenario: ScenarioBuilder, count: u32) -> (TestSim, Vec<ObjectId>) {
    let mut t = scenario.start();
    t.without_auto_targeting();
    t.debug_at(1, crystals(P1, BANK));
    let recruits = block(&mut t, 1, count, 5, Vec2::new(1150.0, 704.0));
    t.at(2, P1, enter(&recruits, A1));
    run_checked(&mut t, 2);
    fill(&mut t, A1, recruits.len());
    let order = inside(&t, A1);
    (t, order)
}

/// [`filled_on`] [`base`].
fn filled(count: u32) -> (TestSim, Vec<ObjectId>) {
    filled_on(base(), count)
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

/// KindMenu\[`from`\]'s "Q: Build Sparrow".
fn train(objects: &[ObjectId], from: RecruitKind) -> Command {
    production(objects, ProductionCommand::TrainFrom { kind: SPARROW, from })
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

/// Gives unit `unit` the augmentations of a Recruit of kind `kind`, at `tick` (a debug command stands for the Altar).
fn augment(t: &mut TestSim, tick: Tick, unit: ObjectId, kind: RecruitKind) {
    let augmentation = Augmentation::of(kind);
    t.debug_at(tick, DebugCommand::SetAugmentation { id: unit, augmentation });
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

/// Unit `unit`'s position.
fn pos(t: &TestSim, unit: ObjectId) -> Vec2 {
    t.sim().object(unit).expect("on the map").pos
}

/// Unit `unit`'s speed: the length of its momentum, a Glider's speed (`Mover::momentum`).
fn speed(t: &TestSim, unit: ObjectId) -> f32 {
    t.mover(unit.0).momentum.length()
}

/// Unit `unit`'s current order.
fn current(t: &TestSim, unit: ObjectId) -> Option<OrderKind> {
    let orders = t.sim().object(unit)?.orders.as_ref()?;
    orders.current.as_ref().map(|active| active.order.kind)
}

/// Unit `unit`'s queued orders, after its current one: a producer's unit gets its rally's order in the queue, current
/// from the next tick's orders step on.
fn queued(t: &TestSim, unit: ObjectId) -> Vec<OrderKind> {
    let orders = t.sim().object(unit).and_then(|o| o.orders.as_ref());
    orders.map_or(Vec::new(), |orders| {
        orders.queue.iter().map(|order| order.kind).collect()
    })
}

/// Runs until a unit has been produced, within `limit` ticks; returns it, with its tick.
fn until_produced(t: &mut TestSim, limit: Tick) -> (Tick, ObjectId) {
    let (start, before) = (t.sim().tick(), produced(t).len());
    while produced(t).len() == before {
        assert!(t.sim().tick() < start + limit, "nothing produced");
        step_checked(t);
    }
    let (tick, _, unit) = produced(t)[before];
    (tick, unit)
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

/// The direction unit `unit` faces.
fn heading(t: &TestSim, unit: ObjectId) -> Vec2 {
    math::facing_vector(t.sim().object(unit).expect("on the map").facing)
}
