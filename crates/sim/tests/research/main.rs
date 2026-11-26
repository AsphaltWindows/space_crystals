//! The Temple and research (slice M5-N3, plan decisions T70 and T74; `factions/cults/structures/temple.md`,
//! `core/tech_and_research.md > Research`, `factions/cults/augmentation.md > Research applies to everyone`; design
//! changes 0335 to 0338): Recruits staff a Temple, which researches the two augmentation lines on its queue, paid as
//! they are queued, through the gate, one test binary:
//! - `staffing`: Enhanced Constitution Level 1 in 960 ticks with one Recruit inside, a third of that with three; no
//!   progress without staff, and none lost; a second research waits for the first; a Recruit on its way out no longer
//!   staffs;
//! - `queue`: 100 Space Crystals at queue, X gives the last research's back in full, and a Temple destroyed refunds
//!   every research and lets its staff out; the refusals of a research no structure of the group offers or the bank
//!   can't pay;
//! - `levels`: a Level 2 waits for its Level 1 to complete, then for a Cathedral, checked at queue only; no
//!   technology queued at two Temples, a group's research taken by one;
//! - `retroactive`: a completed level changes every unit with that Augmentation at once, on the map or held, its HP
//!   following its MaxHP, and the units augmented later; an Altar's order finishing in the same tick; walking units;
//! - `doors`: the staff go in at side B, Eject All lets them out of side C one at a time, a Release one of a kind; no
//!   rally point, and no one but Recruits;
//! - `saves`: a game with research under way goes on alike after a save and in a replay;
//! - `neutral`: a defeated player's units lose their research (question Q0142).
//!
//! Every test runs under the invariants of `TestSim`, the research invariant T1 among them
//! (`testing/invariants/research.rs`), and checks that no reference to a unit that went inside is left behind.

mod doors;
mod levels;
mod neutral;
mod queue;
mod retroactive;
mod saves;
mod staffing;

use sc_sim::economy::RATIO_ONE;
use sc_sim::testing::{ScenarioBuilder, TestSim, dangling_references};
use sc_sim::{
    Augmentation, BuildingOrientation, Command, DebugCommand, Faction, HoldCommand, ObjectId, ObjectKind, OrderKind,
    PlayerCommand, PlayerId, ProductionCommand, RecruitKind, Reject, SimEvent, SpawnAt, Target, Tech, Tick, TilePos,
    Vec2,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const RECRUIT: ObjectKind = ObjectKind::Recruit;
const MILITANT: ObjectKind = ObjectKind::Militant;
const TEMPLE: ObjectKind = ObjectKind::Temple;
const CATHEDRAL: ObjectKind = ObjectKind::Cathedral;
const ARMORY: ObjectKind = ObjectKind::Armory;
const ALTAR: ObjectKind = ObjectKind::Altar;

const EC1: Tech = Tech::EnhancedConstitutionL1;
const EC2: Tech = Tech::EnhancedConstitutionL2;
const IM1: Tech = Tech::ImprovedMetabolismL1;
const IM2: Tech = Tech::ImprovedMetabolismL2;

/// `factions/cults/structures/temple.md > Recruit Staffing`: "Research (BaseResearchTime = with 1 Recruit): Enhanced
/// Constitution L1 and Improved Metabolism L1: **100 SC, 960 frames (60 s)** each; L2 of each: **150 SC, 1440 frames
/// (90 s)**".
const L1_PRICE: u32 = 100;
const L1_FRAMES: Tick = 960;
const L2_PRICE: u32 = 150;
const L2_FRAMES: Tick = 1440;

/// The Temple of [`base`].
const T1: ObjectId = ObjectId(1);
/// The middle of [`base`]'s Temple's entrance side, B.
const ENTRANCE: Vec2 = Vec2::new(960.0, 736.0);
/// The middle of [`base`]'s Temple's exit side, C.
const EXIT: Vec2 = Vec2::new(768.0, 736.0);
/// The Space Crystals player 1 has once [`staffed`] set them.
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

/// Player 1's Temple (1) on a 30 × 22 map, unturned, and their Storage (2) in the north-west corner, which keeps them
/// in the game should the Temple go. The Temple covers tiles (12, 10) to (14, 12), x 768..960 and y 640..832: its
/// entrance, B, faces east (x = 960, the doorway's middle at [`ENTRANCE`]); its exit, C, faces west (x = 768,
/// [`EXIT`]).
fn base() -> ScenarioBuilder {
    cults(&flat(30, 22))
        .spawn(TEMPLE, Some(P1), tile(12, 10))
        .spawn(ObjectKind::Storage, Some(P1), tile(2, 2))
}

/// [`base`] with [`BANK`] Space Crystals for player 1, set in tick 1, and `count` Plain Recruits of theirs ordered into
/// the Temple in tick 2: the game and the Recruits, in the order they went in, once they are all inside.
fn staffed(count: usize) -> (TestSim, Vec<ObjectId>) {
    staffed_with(base(), &vec![RecruitKind::Plain; count])
}

/// [`staffed`] on `scenario`, a [`base`] with more objects, with a Recruit of each kind of `kinds` (their
/// Augmentations given by a debug command in tick 2, before they are ordered in), in that order from the north.
fn staffed_with(scenario: ScenarioBuilder, kinds: &[RecruitKind]) -> (TestSim, Vec<ObjectId>) {
    let mut t = scenario.start();
    t.without_auto_targeting();
    t.debug_at(1, crystals(P1, BANK));
    let recruits = block(&mut t, 1, RECRUIT, kinds.len(), Vec2::new(1150.0, 736.0));
    run_checked(&mut t, 1);
    for (&unit, &kind) in recruits.iter().zip(kinds) {
        if kind != RecruitKind::Plain {
            set_augmentation(&mut t, 2, unit, kind);
        }
    }
    t.at(2, P1, enter(&recruits, T1));
    run_checked(&mut t, 2);
    fill(&mut t, T1, recruits.len());
    let order = inside(&t, T1);
    (t, order)
}

/// Runs until `building` holds `count` units, within 300 ticks.
fn fill(t: &mut TestSim, building: ObjectId, count: usize) {
    let start = t.sim().tick();
    while inside(t, building).len() < count {
        assert!(t.sim().tick() < start + 300, "{} are in", inside(t, building).len());
        step_checked(t);
    }
}

/// `count` units of player 1's of type `kind` at `tick`, in a column 32 su apart whose middle is `at`, from the north;
/// their ids, from the next free one.
fn block(t: &mut TestSim, tick: Tick, kind: ObjectKind, count: usize, at: Vec2) -> Vec<ObjectId> {
    let first = next_id(t);
    for i in 0..count {
        let y = at.y + 32.0 * (i as f32 - (count as f32 - 1.0) / 2.0);
        t.spawn_unit_at(tick, kind, P1, Vec2::new(at.x, y));
    }
    (0..count as u32).map(|i| id(first + i)).collect()
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

/// `player`'s completed technologies (`Player::techs`), in order.
fn techs(t: &TestSim, player: PlayerId) -> Vec<Tech> {
    let p = t.sim().player(player).expect("in the game");
    p.techs.iter().copied().collect()
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

/// The Temple's Q or W, the next level of a line: a research of `tech`.
fn research(objects: &[ObjectId], tech: Tech) -> Command {
    production(objects, ProductionCommand::Research(tech))
}

/// "**X: Cancel** (last research)".
fn cancel(objects: &[ObjectId]) -> Command {
    production(objects, ProductionCommand::CancelProduction)
}

fn rally(objects: &[ObjectId], at: Vec2) -> Command {
    production(objects, ProductionCommand::SetRally(Target::Point(at)))
}

/// "**V: Eject All** (staff leave)".
fn eject_all(buildings: &[ObjectId]) -> Command {
    Command::Holding {
        objects: buildings.to_vec(),
        command: HoldCommand::EjectAll,
    }
}

/// "**A/S/D/F** release one Recruit of a kind": one of kind `kind`.
fn release_one(buildings: &[ObjectId], kind: RecruitKind) -> Command {
    Command::Holding {
        objects: buildings.to_vec(),
        command: HoldCommand::ReleaseOne(kind),
    }
}

/// A debug spawn of a `kind` structure of player 1's at `at`, unturned, complete.
fn spawn_structure(t: &mut TestSim, tick: Tick, kind: ObjectKind, at: SpawnAt) -> ObjectId {
    let spawned = id(next_id(t));
    t.debug_at(
        tick,
        DebugCommand::Spawn {
            kind,
            owner: Some(P1),
            at,
            orientation: BuildingOrientation::default(),
        },
    );
    spawned
}

/// Gives unit `unit` the Augmentations of a unit of kind `kind`, at `tick` (a debug command stands for the Altar's
/// order).
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

/// The technologies in `building`'s queue, in its order: the first in progress.
fn queue(t: &TestSim, building: ObjectId) -> Vec<Tech> {
    let o = t.sim().object(building).expect("the building");
    let production = o.production.as_ref().expect("a producer");
    production
        .queue
        .iter()
        .map(|entry| entry.tech().expect("research"))
        .collect()
}

/// The progress of `building`'s research in progress, in frames of one Recruit's (a `RATIO_ONE` each), and whether it
/// is a whole number of them.
fn progress(t: &TestSim, building: ObjectId) -> (u32, bool) {
    let o = t.sim().object(building).expect("the building");
    let progress = o.production.as_ref().expect("a producer").progress;
    (progress / RATIO_ONE, progress.is_multiple_of(RATIO_ONE))
}

/// The research completed so far, as `(tick, producer, tech)`.
fn completed(t: &TestSim) -> Vec<(Tick, ObjectId, Tech)> {
    (t.events().iter())
        .filter_map(|(tick, e)| match *e {
            SimEvent::ResearchCompleted { producer, tech } => Some((*tick, producer, tech)),
            _ => None,
        })
        .collect()
}

/// The research queued so far, as `(tick, producer, tech)`.
fn queued(t: &TestSim) -> Vec<(Tick, ObjectId, Tech)> {
    (t.events().iter())
        .filter_map(|(tick, e)| match *e {
            SimEvent::ResearchQueued { producer, tech } => Some((*tick, producer, tech)),
            _ => None,
        })
        .collect()
}

/// The research cancelled so far, as `(tick, producer, tech, refund)`.
fn cancelled(t: &TestSim) -> Vec<(Tick, ObjectId, Tech, u32)> {
    (t.events().iter())
        .filter_map(|(tick, e)| match *e {
            SimEvent::ResearchCancelled { producer, tech, refund } => Some((*tick, producer, tech, refund)),
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

/// Unit `unit`'s HP and MaxHP, its Augmentations' at its owner's research levels (`Sim::research`), on the map or
/// held.
fn hp(t: &TestSim, unit: ObjectId) -> (f32, f32) {
    let o = (t.sim().object(unit)).or_else(|| t.sim().held_unit(unit).map(|held| &held.object));
    let o = o.unwrap_or_else(|| panic!("no unit {unit:?}"));
    let max = o.max_hp_with(t.sim().research());
    (o.hp.expect("destructible"), max.expect("destructible"))
}

/// Whether `pos` is in the doorway of [`base`]'s Temple's side C, half inside it, as a unit that just came out.
fn in_the_exit(pos: Vec2) -> bool {
    pos.x > EXIT.x - 0.5 && pos.x < EXIT.x + 12.5 && (pos.y - EXIT.y).abs() <= 84.0
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

/// Player 2's Recruit, spawned beside player 1's unit `unit` on the map, hits it until its HP is `below` or lower, and
/// is removed: no command hurts a unit at will. Returns the HP it was left with.
fn hurt(t: &mut TestSim, unit: ObjectId, below: f32) -> f32 {
    let now = t.sim().tick() + 1;
    let at = t.sim().object(unit).expect("on the map").pos + Vec2::new(28.0, 0.0);
    t.spawn_unit_at(now, RECRUIT, P2, at);
    run_checked(t, now);
    // The raider before it is gone, and its id isn't given again.
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
