//! Recruits gather Space Crystals (slice M5-D, plan decision T58; `factions/cults/units/recruit.md > Gathering`,
//! `factions/cults/structures/storage.md`; design change 0106: Recruits keep normal collision while they mine): the
//! one Gather job of the Drone and the Agent, with the Recruit's data. It shares patches, delivers anywhere at a
//! Storage, any number at once, and takes the Storage nearest to itself, edge to edge, as each return starts (design
//! changes 0345 and 0375, answering Q0102 and Q0136). `TestSim` checks the invariants after every tick.
//!
//! In modules by topic, one test binary:
//! - `trips`: one Recruit's trip at gaps of 1, 2 and 3 tiles against the design's formula, and a trip phase by phase;
//! - `storages`: which Storage (design changes 0345 and 0375), edge to edge, a new one, one completed beside a busy
//!   patch (0345's Verification), the ends of faces on adjacent patches (0375's), a tie, a lost one, none;
//! - `rules`: patch sharing, unloading together, collision kept, the Drop Off order, the gate, a patch that runs out;
//! - `crowds`: 4 and 5 Recruits on one 2-tile face, the lanes of a 3-tile face (0380's Verification), 10 on an
//!   8-patch field with three Storages, for 5 minutes, and the jam check (no Recruit goes 10 seconds without a
//!   delivery);
//! - `jam`: more Recruits than a face holds starve one another, the design's behaviour (design change 0320): pins of
//!   that starvation, four Recruits on one patch, and the measurements by layout and count (ignored).

mod crowds;
mod jam;
mod rules;
mod storages;
mod trips;

use sc_sim::Sim;
use sc_sim::orders::state::{Behaviour, GatherPhase};
use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{
    Ability, Command, DebugCommand, Faction, Object, ObjectId, ObjectKind, OrderKind, PlayerCommand, PlayerId, Reject,
    ResourceState, Rotation, Shape, SimEvent, SpawnAt, SpawnSettings, Target, Tick, TilePos, Vec2,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const RECRUIT: ObjectKind = ObjectKind::Recruit;
const STORAGE: ObjectKind = ObjectKind::Storage;
/// `factions/cults/units/recruit.md > Gathering`: CarryCapacity, MiningDuration, DropOffDuration and the two reaches.
const LOAD: u32 = 2;
const MINING: Tick = 8;
const DROP_OFF: Tick = 2;
const REACH: f32 = 8.0;
/// Five minutes of game.
const FIVE_MINUTES: Tick = 5 * 60 * 16;
/// The jam check (plan §3.3 D): "a Recruit that delivers nothing for 10 s fails the test".
const JAM: Tick = 10 * 16;
/// The Recruits walk to the field from their start, and find their lanes: the jam check and the incomes count from
/// here. 10 seconds.
const SETTLED: Tick = 160;

/// A flat map on which players 1 and 2 play the Cults.
fn flat(w: usize, h: usize) -> ScenarioBuilder {
    ScenarioBuilder::new(&vec![".".repeat(w); h].join("\n"))
        .faction(P1, Faction::Cults)
        .faction(P2, Faction::Cults)
}

fn tile(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Tile(TilePos::new(x, y))
}

fn centre(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Point(TilePos::new(x, y).center())
}

fn patch_of(amount: u32) -> SpawnSettings {
    SpawnSettings::CrystalPatch {
        starting_amount: amount,
    }
}

fn id(n: u32) -> ObjectId {
    ObjectId(n)
}

fn object(t: &TestSim, n: u32) -> &Object {
    t.sim().object(id(n)).expect("exists")
}

fn bank(t: &TestSim) -> u32 {
    t.sim().player(P1).expect("in the game").crystals
}

fn load(t: &TestSim, n: u32) -> u32 {
    object(t, n).worker.expect("a worker").load
}

fn remaining(t: &TestSim, n: u32) -> u32 {
    match object(t, n).resource {
        Some(ResourceState::CrystalPatch { remaining }) => remaining,
        _ => panic!("object {n} is no patch"),
    }
}

/// The order a unit is carrying out.
fn order(t: &TestSim, n: u32) -> Option<OrderKind> {
    let orders = object(t, n).orders.as_ref().expect("a unit");
    orders.current.as_ref().map(|active| active.order.kind)
}

/// The phase of a worker's job, if it has one.
fn phase(t: &TestSim, n: u32) -> Option<GatherPhase> {
    let orders = object(t, n).orders.as_ref().expect("a unit");
    match orders.current.as_ref()?.state {
        Behaviour::Gather(gather) => Some(gather.phase),
        _ => None,
    }
}

fn is_unloading(t: &TestSim, n: u32) -> bool {
    matches!(phase(t, n), Some(GatherPhase::DroppingOff { .. }))
}

fn is_mining(t: &TestSim, n: u32) -> bool {
    matches!(phase(t, n), Some(GatherPhase::Mining { .. }))
}

fn gather(t: &mut TestSim, tick: Tick, recruits: &[u32], patch: u32) {
    let units: Vec<ObjectId> = recruits.iter().map(|&n| id(n)).collect();
    t.order_at(tick, P1, &units, OrderKind::Gather(id(patch)), false);
}

fn order_of(units: &[u32], kind: OrderKind) -> PlayerCommand {
    let units = units.iter().map(|&n| id(n)).collect();
    let command = Command::Order {
        units,
        kind,
        queue: false,
    };
    PlayerCommand { player: P1, command }
}

/// The deliveries so far, as `(tick, worker, drop-off, amount)`.
fn deliveries(t: &TestSim) -> Vec<(Tick, u32, u32, u32)> {
    t.events()
        .iter()
        .filter_map(|(tick, e)| match e {
            SimEvent::ResourcesDelivered {
                worker,
                drop_off,
                amount,
                ..
            } => Some((*tick, worker.0, drop_off.0, *amount)),
            _ => None,
        })
        .collect()
}

/// Steps until `done` holds, at most `limit` ticks, and returns the tick it first held in.
fn run_to(t: &mut TestSim, limit: Tick, what: &str, done: impl Fn(&TestSim) -> bool) -> Tick {
    for _ in 0..limit {
        t.step();
        if done(t) {
            return t.sim().tick();
        }
    }
    panic!("{what}: not within {limit} ticks\n{}", t.ascii(None));
}

/// The income of the deliveries after tick `from` up to tick `to`, Space Crystals per second.
fn income(t: &TestSim, from: Tick, to: Tick) -> f32 {
    let amount: u32 = (deliveries(t).iter())
        .filter(|d| d.0 > from && d.0 <= to)
        .map(|d| d.3)
        .sum();
    amount as f32 * 16.0 / (to - from) as f32
}

/// The longest any of `recruits` went without a delivery from tick `from` to tick `to`, ticks, and which one. The
/// stretch up to its first delivery after `from` counts, and the one from its last to `to`.
fn longest_gap(t: &TestSim, recruits: &[u32], from: Tick, to: Tick) -> (Tick, u32) {
    let all = deliveries(t);
    let mut worst = (0, 0);
    for &n in recruits {
        let mut ticks: Vec<Tick> = (all.iter())
            .filter(|d| d.1 == n && d.0 > from && d.0 <= to)
            .map(|d| d.0)
            .collect();
        ticks.insert(0, from);
        ticks.push(to);
        let gap = ticks.windows(2).map(|pair| pair[1] - pair[0]).max().unwrap_or(0);
        if gap > worst.0 {
            worst = (gap, n);
        }
    }
    worst
}

/// One face of a Storage and the patches in front of it (`factions/cults/structures/storage.md`: "The best layout
/// leaves a 1-tile gap between patch and Storage; each 2-tile face supports roughly 4–5 Recruits at full
/// efficiency"): Storage 1 turned a quarter on tiles (10, 5) to (11, 7), so that its south face is one of its short
/// sides, 2 tiles, as the whole south face of the 2x2 Storage on (10, 6) to (11, 7) was until design change 0380 made
/// it 3x2; `patches` patches (ids from 2) in a row facing its south face, `gap` tiles from it, from tile (10 +
/// `shift`, 8 + `gap`) eastwards (`shift` 0 and two patches: the whole face); the `recruits` (ids after the patches')
/// in a row farther south.
fn face(recruits: u32, gap: i32, patches: u32, shift: i32) -> TestSim {
    faced(tile(10, 5), Rotation::R90, recruits, gap, patches, shift)
}

/// [`face`] with Storage 1 unturned on tiles (10, 6) to (12, 7), so that the patches face one of its long sides, 3
/// tiles (design change 0380): `shift` 0 and three patches, the whole face.
fn long_face(recruits: u32, gap: i32, patches: u32, shift: i32) -> TestSim {
    faced(tile(10, 6), Rotation::R0, recruits, gap, patches, shift)
}

/// [`face`] and [`long_face`]: Storage 1 at `storage`, turned by `rotation`, its south edge on y = 512 su.
fn faced(storage: SpawnAt, rotation: Rotation, recruits: u32, gap: i32, patches: u32, shift: i32) -> TestSim {
    let mut builder = flat(24, 20).spawn(STORAGE, Some(P1), storage);
    for i in 0..patches as i32 {
        builder = builder.spawn_with(patch_of(1_000_000), tile(10 + shift + i, 8 + gap));
    }
    for i in 0..recruits {
        let at = Vec2::new(520.0 + 40.0 * i as f32, 920.0);
        builder = builder.spawn(RECRUIT, Some(P1), SpawnAt::Point(at));
    }
    let mut scenario = builder.build();
    scenario.map.spawns[0].orientation.rotation = rotation;
    TestSim::new(scenario)
}

/// The ids of [`face`]'s Recruits.
fn face_recruits(recruits: u32, patches: u32) -> Vec<u32> {
    (2 + patches..2 + patches + recruits).collect()
}

/// [`face`] with its Recruits dealt to its patches in turn, sent in tick 1.
fn face_at_work(recruits: u32, gap: i32, patches: u32, shift: i32) -> (TestSim, Vec<u32>) {
    at_work(face(recruits, gap, patches, shift), recruits, patches)
}

/// [`long_face`] with its Recruits dealt to its patches in turn, sent in tick 1.
fn long_face_at_work(recruits: u32, gap: i32, patches: u32, shift: i32) -> (TestSim, Vec<u32>) {
    at_work(long_face(recruits, gap, patches, shift), recruits, patches)
}

/// The `recruits` of a face with `patches` patches dealt to them in turn, sent in tick 1.
fn at_work(mut t: TestSim, recruits: u32, patches: u32) -> (TestSim, Vec<u32>) {
    let ids = face_recruits(recruits, patches);
    for (i, &n) in ids.iter().enumerate() {
        gather(&mut t, 1, &[n], 2 + i as u32 % patches);
    }
    (t, ids)
}

/// An 8-patch field with three Storages (`factions/cults/structures/storage.md`: "an efficient crystal field take[s]
/// more Storages (3–4 for an 8-patch field)"): patches 1 to 8 in a row on tiles (4, 3) to (11, 3); Storages 9, 10
/// and 11 `gap` tiles south of the row, unturned on columns 4-6, 7-9 and 10-12 (on 4-5, 7-8 and 10-11 until design
/// change 0380 made them 3x2); the `recruits` (ids from 12) in rows farther south, dealt to the patches in turn and
/// sent in tick 1.
fn field_at_work(recruits: u32, gap: i32) -> (TestSim, Vec<u32>) {
    let mut builder = flat(20, 20);
    for x in 0..8 {
        builder = builder.spawn_with(patch_of(1_000_000), tile(4 + x, 3));
    }
    for x in [4, 7, 10] {
        builder = builder.spawn(STORAGE, Some(P1), tile(x, 4 + gap));
    }
    for i in 0..recruits {
        let at = Vec2::new(200.0 + 40.0 * (i % 12) as f32, 720.0 + 40.0 * (i / 12) as f32);
        builder = builder.spawn(RECRUIT, Some(P1), SpawnAt::Point(at));
    }
    let mut t = builder.start();
    let ids: Vec<u32> = (12..12 + recruits).collect();
    for (i, &n) in ids.iter().enumerate() {
        gather(&mut t, 1, &[n], 1 + i as u32 % 8);
    }
    (t, ids)
}
