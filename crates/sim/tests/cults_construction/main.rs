//! The Cults' construction (slice M5-E, plan decisions T59 and T62; design changes 0071, 0072, 0339, 0340 and, slice
//! M5-J1, 0362), through the gate: `factions/cults/economy.md > Cults Building Mechanics`, `factions/cults/economy.md >
//! Cults Building Mechanics > Group construction rules`, `factions/cults/economy.md > Cults Building Mechanics > Group
//! construction refunds`, `factions/cults/economy.md > BaseConstructionTime Scaling`, `factions/cults/economy.md >
//! Build Command`, `factions/cults/economy.md > Assist Construction Command`, `factions/cults/economy.md > Cancel
//! Construction`, `factions/cults/units/recruit.md > Building` and `factions/cults/units/recruit.md > Constructs`. One
//! test binary:
//! - `flow`: a group builds a building from outside, faster with more builders, paid once; five from one side spread
//!   along the wall, the first building at once; two builders build twice as fast; one Recruit alone; a Shift chain;
//!   builders pulled off (the stall, the slowdown) and back by Assist; the right-click;
//! - `refunds`: the refunds before the start (re-ordered, killed, a queued building dropped), none after it, the 75%
//!   of X, the group going on to its queued building after X;
//! - `footprint`: who may stand on the footprint as the site starts (design changes 0362 and 0372): the site waits for
//!   the order's own Recruits to step off (a group under its building, one walking off the middle, one holding the
//!   order queued, and 0372's Verification, a Hall Shift-placed over a builder's spot at a Storage); any other unit on
//!   it, an enemy or an own one, fails the whole order, refunded once;
//! - `gate`: the gate's checks of a Build and an Assist, the Armory's Storage, the ghosts;
//! - `crowds`: more Recruits than the builders' wall holds wait further out; the builders never stand on the
//!   footprint;
//! - `hall`: a Recruitment Hall the Recruits build makes Recruits of its own lineage; gatherers taken off to build and
//!   sent back keep gathering by slice M5-D's rules; a game saved and replayed;
//! - `tech`: the Gunworks, the Machine Shop and the Scrap Hangar (slice M5-G2), each at its price and in its time,
//!   behind its TechPrerequisite; the Gunworks unlocking the Gunner once complete; a right-click on one a Move.
//!
//! Every test runs under the invariants of `TestSim` (no unit stands on a footprint, so no builder does), and [`step`]
//! checks after each tick that a site built from outside gained exactly one frame for each builder at work: on an
//! Assist of it, within reach, its walk over.

mod crowds;
mod flow;
mod footprint;
mod gate;
mod hall;
mod refunds;
mod tech;

use sc_sim::orders::state::Behaviour;
use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{
    BuildingOrientation, Command, ConstructionCommand, DebugCommand, Faction, Object, ObjectId, ObjectKind, OrderKind,
    PlayerCommand, PlayerId, Reject, Shape, SimEvent, SpawnAt, Tick, TilePos, Vec2,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const RECRUIT: ObjectKind = ObjectKind::Recruit;
const STORAGE: ObjectKind = ObjectKind::Storage;
const ARMORY: ObjectKind = ObjectKind::Armory;
const HALL: ObjectKind = ObjectKind::RecruitmentHall;

/// Player 1's Storage on tiles (2, 2) to (4, 3), standing from the start, in [`world`]: their structure, so that a
/// cancelled site doesn't defeat them, and the Armory's TechPrerequisite.
const S1: ObjectId = ObjectId(1);

/// Where the tests build: a Storage's footprint, unturned (design change 0380: 3x2), tiles (20, 14) to (22, 15),
/// x 1280..1472 and y 896..1024.
const SITE: TilePos = TilePos::new(20, 14);
/// A second site, east of [`SITE`] across a gap of two tiles: tiles (25, 14) to (27, 15).
const SITE_2: TilePos = TilePos::new(25, 14);
/// A third, south of [`SITE`]: tiles (20, 18) to (22, 19).
const SITE_3: TilePos = TilePos::new(20, 18);

/// `factions/cults/units/recruit.md > Constructs`: "Storage: 100 Space Crystals, 320 frames (20 seconds)
/// BaseConstructionTime (with 1 Recruit)".
const STORAGE_PRICE: u32 = 100;
const STORAGE_FRAMES: u32 = 320;
/// The starting bank of [`world`].
const BANK: u32 = 1000;
/// `tuning::construction::BUILD_REACH_SU`, design change 0339: "within its 8 su reach of the edge".
const REACH: f32 = 8.0;
/// The longest walk to a site in these tests, ticks.
const WALK: Tick = 400;

fn flat(w: usize, h: usize) -> String {
    vec![".".repeat(w); h].join("\n")
}

fn tile(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Tile(TilePos::new(x, y))
}

fn id(n: u32) -> ObjectId {
    ObjectId(n)
}

/// A flat 40 × 30 map on which players 1 and 2 play the Cults, player 1 with their Storage [`S1`].
fn cults() -> ScenarioBuilder {
    ScenarioBuilder::new(&flat(40, 30))
        .faction(P1, Faction::Cults)
        .faction(P2, Faction::Cults)
        .spawn(STORAGE, Some(P1), tile(2, 2))
}

/// Where the `i`th of `count` Recruits of [`world`] starts: in columns of eight 32 su apart, the first at x = 1100
/// west of [`SITE`] and centred on its row, the next ones further west.
fn start_of(i: u32, count: u32) -> Vec2 {
    let per_column = count.min(8);
    let (column, row) = (i / 8, i % 8);
    let y = 960.0 + 32.0 * (row as f32 - (per_column - 1) as f32 / 2.0);
    Vec2::new(1100.0 - 32.0 * column as f32, y)
}

/// [`cults`] with `count` Recruits of player 1's, ids 2 to `count + 1`, where [`start_of`] puts them; player 1's bank
/// is [`BANK`] Space Crystals from tick 1: orders go from tick 2.
fn world(count: u32) -> TestSim {
    let mut scenario = cults();
    for i in 0..count {
        scenario = scenario.spawn(RECRUIT, Some(P1), SpawnAt::Point(start_of(i, count)));
    }
    let mut t = scenario.start();
    t.debug_at(1, crystals_of(P1, BANK));
    t
}

/// [`cults`] with four Recruits of player 1's, ids 2 to 5, one off each side of [`SITE`], 150 su from its edge, in
/// line with its middle (west, east, north, south), so that they get there together; player 1's bank is [`BANK`] from
/// tick 1.
fn four_round() -> TestSim {
    // The footprint's middle, and its half extents: 3 tiles across, 2 down.
    let (centre, half) = (Vec2::new(1376.0, 960.0), Vec2::new(96.0, 64.0));
    let mut scenario = cults();
    for out in [Vec2::NEG_X, Vec2::X, Vec2::NEG_Y, Vec2::Y] {
        scenario = scenario.spawn(RECRUIT, Some(P1), SpawnAt::Point(centre + out * (half + 150.0)));
    }
    let mut t = scenario.start();
    t.debug_at(1, crystals_of(P1, BANK));
    t
}

/// The ids of [`world`]'s `count` Recruits.
fn recruits(count: u32) -> Vec<u32> {
    (2..count + 2).collect()
}

/// Sets `player`'s Space Crystals to `crystals`, their other banks to 0.
fn crystals_of(player: PlayerId, crystals: u32) -> DebugCommand {
    DebugCommand::SetResources {
        player,
        crystals,
        supplies: 0,
        alloys: 0,
        essence: 0,
        flux: 0,
    }
}

fn order(units: &[u32], kind: OrderKind, queue: bool) -> Command {
    Command::Order {
        units: units.iter().map(|&n| id(n)).collect(),
        kind,
        queue,
    }
}

/// A Build of a `kind` at `origin`, unturned, to `units` (`queue` is Shift).
fn build(units: &[u32], kind: ObjectKind, origin: TilePos, queue: bool) -> Command {
    let orientation = BuildingOrientation::default();
    order(
        units,
        OrderKind::Build {
            kind,
            origin,
            orientation,
        },
        queue,
    )
}

/// An Assist of `site` to `units`.
fn assist(units: &[u32], site: ObjectId) -> Command {
    order(units, OrderKind::Assist(site), false)
}

/// A Move of `units` to `to`.
fn move_to(units: &[u32], to: Vec2) -> Command {
    order(units, OrderKind::Move(sc_sim::Target::Point(to)), false)
}

/// Cancel Construction on `objects`.
fn cancel(objects: &[ObjectId]) -> Command {
    Command::Construction {
        objects: objects.to_vec(),
        command: ConstructionCommand::CancelConstruction,
    }
}

/// What the gate says to `player`'s `command` now (`Sim::validate`).
fn gate(t: &TestSim, player: PlayerId, command: Command) -> Result<(), Reject> {
    t.sim().validate(&PlayerCommand { player, command })
}

/// Player 1's Space Crystals.
fn crystals(t: &TestSim) -> u32 {
    t.sim().player(P1).expect("in the game").crystals
}

fn object(t: &TestSim, id: ObjectId) -> &Object {
    t.sim().object(id).unwrap_or_else(|| panic!("{id:?} is on the map"))
}

/// The sites in the game, in id order.
fn sites(t: &TestSim) -> Vec<ObjectId> {
    t.sim().objects().filter(|o| o.is_site()).map(|o| o.id).collect()
}

/// The site on `origin`'s footprint, or the building it became.
fn building_at(t: &TestSim, origin: TilePos) -> Option<ObjectId> {
    (t.sim().objects())
        .find(|o| o.placement.is_some_and(|p| p.origin == origin) && o.def().structure.is_some())
        .map(|o| o.id)
}

/// The frames site `site` has done.
fn frames(t: &TestSim, site: ObjectId) -> u32 {
    object(t, site).construction.expect("a site").progress.ticks_done()
}

/// Unit `n`'s current order, if any.
fn current(t: &TestSim, n: u32) -> Option<OrderKind> {
    let orders = object(t, id(n)).orders.as_ref()?;
    orders.current.as_ref().map(|active| active.order.kind)
}

/// Unit `n`'s queued orders.
fn queued(t: &TestSim, n: u32) -> Vec<OrderKind> {
    let orders = object(t, id(n)).orders.as_ref().expect("a unit");
    orders.queue.iter().map(|order| order.kind).collect()
}

/// Whether unit `n` is on its way somewhere.
fn moving(t: &TestSim, n: u32) -> bool {
    walking(object(t, id(n)))
}

/// Whether unit `n` is on the map with no order, current or queued.
fn idle(t: &TestSim, n: u32) -> bool {
    (t.sim().object(id(n))).is_some_and(|o| o.orders.as_ref().is_some_and(|orders| orders.is_idle()))
}

/// Whether unit `o` stands within reach of the footprint of `site` (design change 0339: 8 su of its edge).
fn in_reach(o: &Object, site: &Object) -> bool {
    let footprint = Shape::footprint(&site.placement.expect("a footprint"));
    Shape::of(o).gap(&footprint) <= REACH
}

/// Whether unit `o` is on its way somewhere.
fn walking(o: &Object) -> bool {
    (o.mover.as_ref()).is_some_and(|mover| mover.status == sc_sim::MoveStatus::Moving)
}

/// The builders at work on `site`: the units on an Assist of it within reach of its footprint, their walk over
/// (design change 0339: "Recruits still walking, waiting for a spot or doing anything else add nothing"), in id order.
fn builders(t: &TestSim, site: ObjectId) -> Vec<ObjectId> {
    let s = object(t, site);
    (t.sim().objects())
        .filter(|o| {
            let current = o.orders.as_ref().and_then(|orders| orders.current.as_ref());
            current.is_some_and(|active| active.order.kind == OrderKind::Assist(site)) && in_reach(o, s) && !walking(o)
        })
        .map(|o| o.id)
        .collect()
}

/// The spot unit `n` claims for its work, if it has one.
fn spot(t: &TestSim, n: u32) -> Option<Vec2> {
    let active = object(t, id(n)).orders.as_ref()?.current.as_ref()?;
    match active.state {
        Behaviour::Build(walk) => walk.spot,
        Behaviour::Assist(work) => work.spot,
        _ => None,
    }
}

/// The frames of each site built from outside, for [`step`]'s check.
fn site_frames(t: &TestSim) -> Vec<(ObjectId, u32, u32)> {
    (t.sim().objects())
        .filter(|o| {
            o.construction
                .is_some_and(|c| c.builder == sc_sim::construction::site::Builder::Recruits)
        })
        .map(|o| (o.id, frames(t, o.id), builders(t, o.id).len() as u32))
        .collect()
}

/// One tick, and the builders' rule after it: each site built from outside gained one frame for each of its builders
/// at work ([`builders`]), as they stand now (the count is taken after the movement, `sim/sites.rs`). `TestSim`
/// checks that no unit stands on a footprint.
fn step(t: &mut TestSim) {
    let before = site_frames(t);
    t.step();
    let tick = t.sim().tick();
    for (site, frames_before, _) in before {
        let Some(o) = t.sim().object(site).filter(|o| o.is_site()) else {
            continue;
        };
        let built = builders(t, o.id).len() as u32;
        let done = frames(t, site);
        let expected = (frames_before + built).min(o.construction.unwrap().progress.total);
        assert_eq!(done, expected, "tick {tick}: {site:?} with {built} builders");
    }
}

/// Steps until `done` holds, at most `limit` ticks, and returns the tick it first held in.
fn run_to(t: &mut TestSim, limit: Tick, what: &str, done: impl Fn(&TestSim) -> bool) -> Tick {
    for _ in 0..limit {
        step(t);
        if done(t) {
            return t.sim().tick();
        }
    }
    panic!("{what}: not within {limit} ticks (tick {})", t.sim().tick());
}

/// Steps `ticks` ticks.
fn run(t: &mut TestSim, ticks: Tick) {
    for _ in 0..ticks {
        step(t);
    }
}

/// The ticks of the events `pick` picks so far.
fn events<T>(t: &TestSim, pick: impl Fn(&SimEvent) -> Option<T>) -> Vec<(Tick, T)> {
    (t.events().iter())
        .filter_map(|(tick, e)| Some((*tick, pick(e)?)))
        .collect()
}

/// The sites started so far: `(tick, site, first arrival)`.
fn started(t: &TestSim) -> Vec<(Tick, (ObjectId, ObjectId))> {
    events(t, |e| match *e {
        SimEvent::SiteStarted { worker, site } => Some((site, worker)),
        _ => None,
    })
}

/// The buildings the Recruits finished so far: `(tick, site)`.
fn built(t: &TestSim) -> Vec<(Tick, ObjectId)> {
    events(t, |e| match *e {
        SimEvent::SiteBuilt { site } => Some(site),
        _ => None,
    })
}

/// The Builds refunded so far: `(tick, (worker, refund))`.
fn refunds(t: &TestSim) -> Vec<(Tick, (ObjectId, u32))> {
    events(t, |e| match *e {
        SimEvent::BuildRefunded { worker, refund, .. } => Some((worker, refund)),
        _ => None,
    })
}

/// The Builds that failed so far: `(tick, (worker, problems))`.
fn failures(t: &TestSim) -> Vec<(Tick, (ObjectId, sc_sim::PlacementCheck))> {
    events(t, |e| match e {
        SimEvent::BuildFailed { worker, problems, .. } => Some((*worker, problems.clone())),
        _ => None,
    })
}
