//! The Shrine (slice M5-N5, plan decisions T75 and T76; `factions/cults/structures/shrine.md`): Recruits inside charge
//! its energy, which it spends on two area buffs cast at a point in vision, played through the gate, one test binary:
//! - `energy`: eight Recruits inside charge 2 a second, so the first Q waits 37.5 s and 200 takes 100 s; with none
//!   inside it keeps what it has; it starts at 0;
//! - `health`: Q heals each own Cults infantry unit whose centre is in the circle 30 HP over 15 s, capped at its MaxHP
//!   with its Augmentations, not the enemy's nor another type's, nor a unit that walks in after the cast; a unit that
//!   leaves the circle or goes into a building keeps it; a recast restarts it, and the rates never add;
//! - `disciples`: W adds 1 a second to a Disciple's 0.5 for 20 s, and under it a Disciple channelling Shroud nets
//!   0.5 + 1 − 5 a second;
//! - `refusals`: a point out of sight or off the map, too little energy, a Shrine under construction or someone
//!   else's, an object that casts nothing; one Shrine casts, the first that can; Recruits build a Shrine only with a
//!   Temple of their player's;
//! - `doors`: Recruits go in at B, A/S/D/F let one of a kind out of C, V all of them one by one; Recruits only, eight
//!   of them; its destruction lets them out, and its energy goes with it;
//! - `saves`: a game with a charged Shrine and buffs under way goes on alike after a save and in a replay, the buffs
//!   saved only while a unit has one.
//!
//! Every test runs under the invariants of `TestSim`, the buff invariant F1 among them (`testing/invariants/buffs.rs`),
//! and checks that no reference to a unit that went inside is left behind.

mod disciples;
mod doors;
mod energy;
mod health;
mod refusals;
mod saves;

use sc_sim::buffs::Buff;
use sc_sim::testing::{ScenarioBuilder, TestSim, dangling_references};
use sc_sim::{
    AreaAbility, Augmentation, BuildingOrientation, Command, DebugCommand, Faction, HoldCommand, ObjectId, ObjectKind,
    OrderKind, PlayerCommand, PlayerId, RecruitKind, Reject, SimEvent, SpawnAt, Target, Tick, TilePos, Vec2,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const RECRUIT: ObjectKind = ObjectKind::Recruit;
const MILITANT: ObjectKind = ObjectKind::Militant;
const GUNNER: ObjectKind = ObjectKind::Gunner;
const DISCIPLE: ObjectKind = ObjectKind::Disciple;
const SHRINE: ObjectKind = ObjectKind::Shrine;
const TEMPLE: ObjectKind = ObjectKind::Temple;

const Q: AreaAbility = AreaAbility::HealthRegeneration;
const W: AreaAbility = AreaAbility::EnergyRegeneration;

/// A point of energy in `Object::energy`'s units.
const ONE: u32 = sc_sim::energy::ONE;
/// What eight Recruits inside add in a tick: 0.25 a second each, 2 a second (`> Placeholder stats`: "2 per second when
/// full").
const FULL: u32 = 8192;
/// A buff's 15 s and 20 s, in ticks.
const Q_TICKS: Tick = 240;
const W_TICKS: Tick = 320;

/// The Shrine of [`base`].
const S1: ObjectId = ObjectId(1);
/// The middle of [`base`]'s Shrine's entrance side, B.
const ENTRANCE: Vec2 = Vec2::new(512.0, 640.0);
/// The middle of [`base`]'s Shrine's exit side, C.
const EXIT: Vec2 = Vec2::new(384.0, 640.0);
/// Where the tests cast, east of the Shrine: further from its entrance than the circle's 4 grid units (256 su).
const AT: Vec2 = Vec2::new(900.0, 640.0);
/// The Space Crystals player 1 has from tick 1.
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

/// Player 1's Shrine (1) on a 32 × 20 map, unturned, and their Storage (2) in the north-west corner, which keeps them
/// in the game should the Shrine go. The Shrine covers tiles (6, 9) to (7, 10), x 384..512 and y 576..704: its
/// entrance, B, faces east (x = 512, the doorway's middle at [`ENTRANCE`]); its exit, C, faces west (x = 384,
/// [`EXIT`]).
fn base() -> ScenarioBuilder {
    cults(&flat(32, 20))
        .spawn(SHRINE, Some(P1), tile(6, 9))
        .spawn(ObjectKind::Storage, Some(P1), tile(1, 1))
}

/// [`base`] with [`BANK`] Space Crystals for player 1 and auto-targeting off, set in tick 1, and `count` Plain Recruits
/// of theirs ordered into the Shrine in tick 2: the game and the Recruits, in the order they went in, once they are all
/// inside.
fn manned(count: usize) -> (TestSim, Vec<ObjectId>) {
    manned_with(base(), &vec![RecruitKind::Plain; count])
}

/// [`manned`] on `scenario`, a [`base`] with more objects, with a Recruit of each kind of `kinds` (their Augmentations
/// given by a debug command in tick 2, before they are ordered in), in that order from the north.
fn manned_with(scenario: ScenarioBuilder, kinds: &[RecruitKind]) -> (TestSim, Vec<ObjectId>) {
    let mut t = scenario.start();
    t.without_auto_targeting();
    t.debug_at(1, crystals(P1, BANK));
    let recruits = block(&mut t, 1, RECRUIT, kinds.len(), Vec2::new(650.0, 640.0));
    run_checked(&mut t, 1);
    for (&unit, &kind) in recruits.iter().zip(kinds) {
        if kind != RecruitKind::Plain {
            set_augmentation(&mut t, 2, unit, kind);
        }
    }
    t.at(2, P1, enter(&recruits, S1));
    run_checked(&mut t, 2);
    fill(&mut t, S1, recruits.len());
    let order = inside(&t, S1);
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

/// Runs until [`S1`] has `points` of energy, within 2000 ticks.
fn charge(t: &mut TestSim, points: u32) {
    let start = t.sim().tick();
    while energy(t, S1) < points * ONE {
        assert!(t.sim().tick() < start + 2000, "charged to {}", energy(t, S1) / ONE);
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

/// A unit of `owner`'s of type `kind` at `at`, spawned in the next tick, which it runs: its id.
fn spawn(t: &mut TestSim, kind: ObjectKind, owner: PlayerId, at: Vec2) -> ObjectId {
    let tick = t.sim().tick() + 1;
    let first = next_id(t);
    t.spawn_unit_at(tick, kind, owner, at);
    run_checked(t, tick);
    let [spawned] = new_objects(t, first, kind)[..] else {
        panic!("{kind:?} at {at}")
    };
    spawned
}

/// A debug spawn of a `kind` structure of `owner`'s at `at`, unturned, complete, in the next tick, which it runs: its
/// id.
fn spawn_structure(t: &mut TestSim, kind: ObjectKind, owner: PlayerId, at: SpawnAt) -> ObjectId {
    let tick = t.sim().tick() + 1;
    let first = next_id(t);
    let orientation = BuildingOrientation::default();
    let spawn = DebugCommand::Spawn {
        kind,
        owner: Some(owner),
        at,
        orientation,
    };
    t.debug_at(tick, spawn);
    run_checked(t, tick);
    let [spawned] = new_objects(t, first, kind)[..] else {
        panic!("{kind:?}")
    };
    spawned
}

/// The id the next object will have, with no spawn scheduled yet and no object removed: the objects and the held
/// units have the ids below it. With one removed (a raider of [`wound`]'s), the next has a higher id ([`new_objects`]),
/// as the sim never gives an id again.
fn next_id(t: &TestSim) -> u32 {
    let on_map = t.sim().objects().map(|o| o.id.0);
    let held = t.sim().held().map(|held| held.object.id.0);
    on_map.chain(held).max().unwrap_or(0) + 1
}

/// The objects of type `kind` on the map from id `first` on, in id order: those spawned since [`next_id`] gave
/// `first`.
fn new_objects(t: &TestSim, first: u32, kind: ObjectKind) -> Vec<ObjectId> {
    (t.sim().objects())
        .filter(|o| o.id.0 >= first && o.kind == kind)
        .map(|o| o.id)
        .collect()
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

fn enter(units: &[ObjectId], target: ObjectId) -> Command {
    Command::Order {
        units: units.to_vec(),
        kind: OrderKind::Enter(target),
        queue: false,
    }
}

fn walk(units: &[ObjectId], to: Vec2) -> Command {
    Command::Order {
        units: units.to_vec(),
        kind: OrderKind::Move(Target::Point(to)),
        queue: false,
    }
}

/// The Shrine's Q or W (`Command::Cast`): `ability` from one of `casters`, at `at`.
fn cast(casters: &[ObjectId], ability: AreaAbility, at: Vec2) -> Command {
    Command::Cast {
        casters: casters.to_vec(),
        ability,
        at,
    }
}

/// "**V: Eject All** (participants leave)".
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

/// Object `id`'s energy, in `Object::energy`'s units, on the map or held.
fn energy(t: &TestSim, id: ObjectId) -> u32 {
    let o = (t.sim().object(id)).or_else(|| t.sim().held_unit(id).map(|held| &held.object));
    o.and_then(|o| o.energy).unwrap_or_else(|| panic!("no pool for {id:?}"))
}

/// Unit `unit`'s HP and MaxHP, its Augmentations' at its owner's research levels (`Sim::research`), on the map or
/// held.
fn hp(t: &TestSim, unit: ObjectId) -> (f32, f32) {
    let o = (t.sim().object(unit)).or_else(|| t.sim().held_unit(unit).map(|held| &held.object));
    let o = o.unwrap_or_else(|| panic!("no unit {unit:?}"));
    let max = o.max_hp_with(t.sim().research());
    (o.hp.expect("destructible"), max.expect("destructible"))
}

/// Unit `unit`'s buffs, on the map or held.
fn buffs(t: &TestSim, unit: ObjectId) -> Vec<Buff> {
    let o = (t.sim().object(unit)).or_else(|| t.sim().held_unit(unit).map(|held| &held.object));
    o.unwrap_or_else(|| panic!("no unit {unit:?}")).buffs.clone()
}

/// A Health or Energy regeneration buff, until `until`.
fn buff(ability: AreaAbility, until: Tick) -> Vec<Buff> {
    vec![Buff { ability, until }]
}

/// The casts so far, as `(tick, caster, ability, at)`.
fn casts(t: &TestSim) -> Vec<(Tick, ObjectId, AreaAbility, Vec2)> {
    (t.events().iter())
        .filter_map(|(tick, e)| match *e {
            SimEvent::Cast {
                caster, ability, at, ..
            } => Some((*tick, caster, ability, at)),
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

/// Whether `pos` is in the doorway of [`base`]'s Shrine's side C, half inside it, as a unit that just came out.
fn in_the_exit(pos: Vec2) -> bool {
    pos.x > EXIT.x - 0.5 && pos.x < EXIT.x + 12.5 && (pos.y - EXIT.y).abs() <= 52.0
}

/// Wounds each unit of `wounds` (on the map) to its HP or lower: a Militant of the other player's, 100 su east of it,
/// attacks it (4 a hit, every 8 ticks), and is removed once it has (no command hurts a unit at will).
fn wound(t: &mut TestSim, wounds: &[(ObjectId, f32)]) {
    let now = t.sim().tick() + 1;
    let first = next_id(t);
    for &(unit, _) in wounds {
        let o = t.sim().object(unit).expect("on the map");
        let other = if o.owner == Some(P1) { P2 } else { P1 };
        t.spawn_unit_at(now, MILITANT, other, o.pos + Vec2::new(100.0, 0.0));
    }
    run_checked(t, now);
    let raiders = new_objects(t, first, MILITANT);
    assert_eq!(raiders.len(), wounds.len(), "a raider for each");
    let mut left: Vec<(ObjectId, ObjectId, f32)> = Vec::new();
    for (&raider, &(unit, to)) in raiders.iter().zip(wounds) {
        let owner = t.sim().object(raider).and_then(|o| o.owner).expect("a raider");
        t.order_at(now + 1, owner, &[raider], OrderKind::Attack(unit), false);
        left.push((raider, unit, to));
    }
    while !left.is_empty() {
        assert!(t.sim().tick() < now + 400, "never wounded enough");
        step_checked(t);
        let next = t.sim().tick() + 1;
        let (done, more): (Vec<_>, Vec<_>) = left.into_iter().partition(|&(_, unit, to)| hp(t, unit).0 <= to);
        for (raider, ..) in done {
            remove_at(t, next, raider);
        }
        left = more;
    }
    step_checked(t);
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
