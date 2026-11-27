//! The Factory, Advanced Mechanics, the Industrial Powerplant, the Raptor and the Twister (slice M6-GF;
//! `factions/gdo/structures/factory.md`, `factions/gdo/structures/advanced_mechanics.md`,
//! `factions/gdo/structures/industrial_powerplant.md`, `factions/gdo/units/raptor.md`,
//! `factions/gdo/units/twister.md`; design changes 0366, 0441, 0446, 0447 and 0451), through the gate, one test
//! binary:
//! - `buildings`: the Deployment Center constructs the Factory behind a Barracks, Advanced Mechanics behind a Factory
//!   and the Industrial Powerplant behind Advanced Mechanics, each refused without its TechPrerequisite, at 0366's cost
//!   and time; the Powerplant's +60 counts in the Power summary;
//! - `production`: the Factory trains the Raptor for 100 Space Crystals in 240 frames and the Twister for 125 and a
//!   Supply in 320, Unit Control 2 each, reserved as production starts; with no Supply the Twister is refused;
//! - `exit`: a Factory starts with its rally point 3 grid units out of side B, however it is turned (0446); five
//!   Raptors queued with no orders come out of side B one by one, facing out, their boxes turned with them, never
//!   overlapping, and drive off to the rally point, the doorway never stalling;
//! - `parking`: design change 0451's Verification at the Factory, five Raptors and five Twisters parking round the
//!   default rally point, all ten out with the doorway never stalling; with the point cleared, the second stands in
//!   the doorway and the third waits inside (0447);
//! - `raptor`: its homing missile hits a running Peacekeeper at Range 6, one every 32 frames, its turret tracking
//!   while its hull drives; `raptor::close_range`, question Q0184's cases for it, the Raptor as built: a target near
//!   the rim of its turning circle, which it gives up, and Attack Ground, which its attack can't take;
//! - `twister`: its machine gun, on a turret that turns all the way round, hits a Sparrow; an Attack on a Peacekeeper
//!   is refused.
//!
//! Every test runs under the invariants of `TestSim`, the wheeled invariant W1 among them: no Raptor or Twister ever
//! turns on the spot.

mod buildings;
mod exit;
mod parking;
mod production;
mod raptor;
mod twister;

use sc_sim::production::{Door, Doorway, Rally};
use sc_sim::shape::Shape;
use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{
    Command, DebugCommand, Faction, Object, ObjectId, ObjectKind, OrderKind, PlayerCommand, PlayerId,
    ProductionCommand, Reject, SimEvent, SpawnAt, Target, Tick, TilePos, Vec2,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const FACTORY: ObjectKind = ObjectKind::Factory;
const RAPTOR: ObjectKind = ObjectKind::Raptor;
const TWISTER: ObjectKind = ObjectKind::Twister;
const PK: ObjectKind = ObjectKind::Peacekeeper;

/// `factions/gdo/structures/factory.md > Produces`: "Raptor: 100 Space Crystals, 240 frames (15 seconds)
/// (placeholder)", "Twister: 125 Space Crystals, 320 frames (20 seconds) (placeholder)", and its "+ 1 Supply" (design
/// change 0441).
const RAPTOR_PRICE: u32 = 100;
const RAPTOR_FRAMES: Tick = 240;
const TWISTER_PRICE: u32 = 125;
const TWISTER_FRAMES: Tick = 320;

/// "3 grid units" (`core/entities.md > Entrance and exit > Exit Doorway` point 7, placeholder distance), space units.
const RALLY_OUT: f32 = 3.0 * 64.0;

// The objects of [`works`], by id: 1 player 1's Deployment Center on tiles (2, 2) to (5, 5) (Power +30), 2 and 3 their
// Generators on tiles (2, 8) and (5, 8) (+20 each), 4 their Barracks on tiles (2, 12) to (4, 13) (−30), 5 player 2's
// Deployment Center, far off on tiles (34, 24) to (37, 27), and 6 player 1's Factory.
/// Player 1's Barracks in [`base`].
const BARRACKS: ObjectId = ObjectId(4);
/// Player 1's Factory in [`works`], unturned, on tiles (16, 10) to (18, 12), x 1024..1216 and y 640..832.
const WORKS: ObjectId = ObjectId(6);
/// Where [`works`]'s Factory stands.
const WORKS_AT: TilePos = TilePos::new(16, 10);
/// The Space Crystals player 1 has from tick 1.
const BANK: u32 = 5000;

fn flat(w: usize, h: usize) -> String {
    vec![".".repeat(w); h].join("\n")
}

fn tile(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Tile(TilePos::new(x, y))
}

fn id(n: u32) -> ObjectId {
    ObjectId(n)
}

/// Both players play the GDO on a flat 40 × 30 map: player 1 with their Deployment Center, two Generators and a
/// Barracks (Power 70 generated, 30 used), player 2 with a Deployment Center far off.
fn base() -> ScenarioBuilder {
    ScenarioBuilder::new(&flat(40, 30))
        .faction(P1, Faction::Gdo)
        .faction(P2, Faction::Gdo)
        .spawn(ObjectKind::DeploymentCenter, Some(P1), tile(2, 2))
        .spawn(ObjectKind::Generator, Some(P1), tile(2, 8))
        .spawn(ObjectKind::Generator, Some(P1), tile(5, 8))
        .spawn(ObjectKind::Barracks, Some(P1), tile(2, 12))
        .spawn(ObjectKind::DeploymentCenter, Some(P2), tile(34, 24))
}

/// [`base`] with player 1's Factory ([`WORKS`]) at [`WORKS_AT`] (Power 70 generated, 60 used: production at full
/// speed), [`started`] with `supplies` Supplies.
fn works(supplies: u32) -> TestSim {
    started(base().spawn(FACTORY, Some(P1), SpawnAt::Tile(WORKS_AT)), supplies)
}

/// `scenario` started with auto-targeting off, [`BANK`] Space Crystals and `supplies` Supplies for player 1 in tick 1,
/// which has run.
fn started(scenario: ScenarioBuilder, supplies: u32) -> TestSim {
    let mut t = scenario.start();
    t.without_auto_targeting();
    t.debug_at(1, banks(P1, BANK, supplies));
    t.run_until(1);
    t
}

fn banks(player: PlayerId, crystals: u32, supplies: u32) -> DebugCommand {
    DebugCommand::SetResources {
        player,
        crystals,
        supplies,
        alloys: 0,
        essence: 0,
        flux: 0,
    }
}

/// Player 1's Space Crystals and Supplies.
fn bank(t: &TestSim) -> (u32, u32) {
    let player = t.sim().player(P1).expect("in the game");
    (player.crystals, player.supplies)
}

/// What the gate says to `player`'s `command` now (`Sim::validate`).
fn gate(t: &TestSim, player: PlayerId, command: Command) -> Result<(), Reject> {
    t.sim().validate(&PlayerCommand { player, command })
}

fn production(producer: ObjectId, command: ProductionCommand) -> Command {
    Command::Production {
        objects: vec![producer],
        command,
    }
}

/// The Factory's Q (a Raptor) or W (a Twister).
fn train(kind: ObjectKind) -> Command {
    production(WORKS, ProductionCommand::Train(kind))
}

fn order(units: &[ObjectId], kind: OrderKind) -> Command {
    Command::Order {
        units: units.to_vec(),
        kind,
        queue: false,
    }
}

fn object(t: &TestSim, id: ObjectId) -> &Object {
    t.sim().object(id).unwrap_or_else(|| panic!("{id:?} is on the map"))
}

/// Object `id`'s shape where it stands (`sc_sim::shape`): a vehicle's Silhouette turned with its facing.
fn shape(t: &TestSim, id: ObjectId) -> Shape {
    Shape::of(object(t, id))
}

/// The Factory `factory`'s one door, the middle of its side B.
fn door(t: &TestSim, factory: ObjectId) -> Door {
    let doorway = Doorway::of(object(t, factory)).expect("an exit doorway");
    doorway.doors().next().expect("one door")
}

/// `building`'s rally point, if it has one.
fn rally_of(t: &TestSim, building: ObjectId) -> Option<Rally> {
    object(t, building).production.as_ref()?.rally
}

/// The units `producer` finished so far, with their ticks.
fn produced(t: &TestSim, producer: ObjectId) -> Vec<(Tick, ObjectId)> {
    (t.events().iter())
        .filter_map(|(tick, e)| match *e {
            SimEvent::UnitProduced { producer: by, unit } if by == producer => Some((*tick, unit)),
            _ => None,
        })
        .collect()
}

/// The hits on `target` so far: (tick, attacker, HP taken).
fn hits_on(t: &TestSim, target: ObjectId) -> Vec<(Tick, ObjectId, f32)> {
    (t.events().iter())
        .filter_map(|(tick, e)| match *e {
            SimEvent::Hit {
                attacker,
                target: hit,
                damage,
                ..
            } if hit == target => Some((*tick, attacker, damage)),
            _ => None,
        })
        .collect()
}

/// The id the next object will have, with no spawn scheduled yet.
fn next_id(t: &TestSim) -> u32 {
    t.sim().objects().map(|o| o.id.0).max().unwrap_or(0) + 1
}

/// Steps until `done`, at most `ticks` ticks; panics naming `what` otherwise.
fn step_until(t: &mut TestSim, ticks: Tick, what: &str, done: impl Fn(&TestSim) -> bool) {
    let end = t.sim().tick() + ticks;
    while !done(t) {
        assert!(t.sim().tick() < end, "{what}: not by tick {end}\n{}", t.ascii(None));
        t.step();
    }
}

/// Runs `t` for `ticks` more ticks.
fn run_for(t: &mut TestSim, ticks: Tick) {
    let until = t.sim().tick() + ticks;
    t.run_until(until);
}

/// Whether `a` is within `tolerance` of `b`.
fn near(a: f32, b: f32, tolerance: f32) -> bool {
    (a - b).abs() <= tolerance
}

/// Unit `id`'s current order.
fn current(t: &TestSim, id: ObjectId) -> Option<OrderKind> {
    let orders = object(t, id).orders.as_ref()?;
    orders.current.as_ref().map(|active| active.order.kind)
}

/// Whether unit `id` has stopped with no order, out of its building's doorway.
fn parked(t: &TestSim, id: ObjectId) -> bool {
    let o = object(t, id);
    let mover = o.mover.as_ref().expect("a unit");
    mover.exit.is_none() && mover.momentum == Vec2::ZERO && o.orders.as_ref().is_some_and(|orders| orders.is_idle())
}

/// Target point `p`.
fn at(p: Vec2) -> Target {
    Target::Point(p)
}
