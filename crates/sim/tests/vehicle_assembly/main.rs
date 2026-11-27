//! The Vehicle Assembly and the Assault Bike (slice M6-SV; `factions/syndicate/structures/vehicle_assembly.md`,
//! `factions/syndicate/units/assault_bike.md`, design change 0367's placeholders), through the gate, one test binary:
//! - `building`: a Tier 2 Tunnel builds it behind a Chemical Plant (refused at Tier 1 and without one), for 250 Space
//!   Crystals in 480 frames, from 10% of its HP; its armour is the expansions' 6 / 54, and it regenerates 1% of its
//!   MaxHP a second;
//! - `production`: it trains the Assault Bike into the Tunnel Network for 125 Space Crystals in 240 frames, Tunnel
//!   Space 2 reserved as production starts; its rally points (`factions/syndicate/structures/README.md > Tunnel
//!   Expansions > Rally Point Behavior`): on the surface the Bike is auto-ejected, on a Tunnel it stays inside, on an
//!   expansion it goes to the spot above;
//! - `exit`: the Bike leaves only through a Tier 2+ Tunnel ("Tier 2+: Vehicles"), out of Side A's exit path, its box
//!   turned to face out (plan decision T100, a 72 × 48 one too), whichever way the Tunnel faces; five come out one by
//!   one; five ejected with no order (design change 0440: an idle vehicle never makes way) stall the exit path, a
//!   reading for the Director; it walks back in through the entry path;
//! - `mortar`: its round, launched at the end of its Firing for where the target stood at the end of its Aiming, hits
//!   a group by the area rule, a target that drives off escapes it, it can Attack Ground, and its blast hits ground
//!   objects only (its TargetDomain's, design change 0445's `BlastDomain::Attack`); it fires from its 40° turret,
//!   whose arc the hull must turn to; and it crushes nothing, being wheeled;
//! - `saves`: a game with Bikes in training, inside, queued and driving out goes on alike after a save and in a
//!   replay.
//!
//! Every test runs under the invariants of `TestSim` (the wheeled invariant W1 among them: no Bike ever turns on the
//! spot), and this binary's held-store checks ([`check_held`]) after each tick it steps.

mod building;
mod exit;
mod mortar;
mod production;
mod saves;

use sc_sim::network::Paths;
use sc_sim::shape::Shape;
use sc_sim::testing::{ScenarioBuilder, TestSim, dangling_references};
use sc_sim::{
    BuildingOrientation, Command, DebugCommand, Faction, Object, ObjectId, ObjectKind, OrderKind, PlayerCommand,
    PlayerId, ProductionCommand, Reject, SimEvent, SpawnAt, Target, Tick, TilePos, TunnelCommand, Vec2,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const BIKE: ObjectKind = ObjectKind::AssaultBike;
const ASSEMBLY: ObjectKind = ObjectKind::VehicleAssembly;
const TUNNEL: ObjectKind = ObjectKind::Tunnel;

/// `factions/syndicate/structures/vehicle_assembly.md > Placeholder stats`: "**Assault Bike** 125 SC, 240 f, 2".
const BIKE_PRICE: u32 = 125;
const BIKE_FRAMES: Tick = 240;
/// `factions/syndicate/units/assault_bike.md > Placeholder stats`: "Silhouette 56x32 space units".
const BIKE_LENGTH: f32 = 56.0;
const BIKE_WIDTH: f32 = 32.0;

/// Player 1's Tunnel in [`base`].
const T1: ObjectId = ObjectId(1);
/// Player 1's Chemical Plant in [`plant`], the Vehicle Assembly's TechPrerequisite.
const CP: ObjectId = ObjectId(2);
/// Player 1's Vehicle Assembly in [`assembly`].
const VA: ObjectId = ObjectId(3);
/// Where [`assembly`]'s Vehicle Assembly stands, and the place [`plant`]'s Tunnel builds one at: tiles (9, 7) to
/// (11, 9), x 576..768 and y 448..640, north-west of the Tunnel, in its Tier 2 area but not its Tier 1 one.
const VA_AT: TilePos = TilePos::new(9, 7);
/// The Space Crystals player 1 has from tick 1.
const BANK: u32 = 5000;

/// The middle of [`base`]'s Tunnel's Side A, facing north, and the exit path's spot on it, 30 su towards Side B
/// (east); the entry path's is 30 su towards Side D (west).
const SIDE_A: Vec2 = Vec2::new(896.0, 640.0);
const EXIT_SPOT: Vec2 = Vec2::new(926.0, 640.0);
const ENTRY_SPOT: Vec2 = Vec2::new(866.0, 640.0);
/// Where a Bike appears on [`base`]'s exit path (design change 0250: "at the exit path's spot just outside Side A"):
/// facing north, out of Side A, its 56 su length along its facing (plan decision T100), its box against the wall.
const BIKE_OUT: Vec2 = Vec2::new(926.0, 640.0 - BIKE_LENGTH / 2.0 - 1.0 / 64.0);

fn flat(w: usize, h: usize) -> String {
    vec![".".repeat(w); h].join("\n")
}

fn tile(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Tile(TilePos::new(x, y))
}

fn id(n: u32) -> ObjectId {
    ObjectId(n)
}

/// Player 1 plays the Syndicate, player 2 the Syndicate too, on a flat 40 × 24 map, with player 1's Tunnel ([`T1`]),
/// unturned, on tiles (12, 10) to (15, 13), x 768..1024 and y 640..896: Side A faces north ([`SIDE_A`]). Its Tier 1
/// area is (10, 8) to (17, 15), its Tier 2 area (9, 7) to (18, 16).
fn base() -> ScenarioBuilder {
    ScenarioBuilder::new(&flat(40, 24))
        .faction(P1, Faction::Syndicate)
        .faction(P2, Faction::Syndicate)
        .spawn(TUNNEL, Some(P1), tile(12, 10))
}

/// [`base`] with player 1's Chemical Plant ([`CP`]) on tiles (16, 14) to (17, 15), in the Tier 1 area.
fn plant() -> ScenarioBuilder {
    base().spawn(ObjectKind::ChemicalPlant, Some(P1), tile(16, 14))
}

/// `scenario` started with auto-targeting off, [`BANK`] Space Crystals for player 1 and, if `tier` is above 1, the
/// Tunnel set to it (a debug command stands for its upgrades, slice M4-G's), all in tick 1, which has run.
fn started(scenario: ScenarioBuilder, tier: u8) -> TestSim {
    let mut t = scenario.start();
    t.without_auto_targeting();
    t.debug_at(1, crystals(P1, BANK));
    if tier > 1 {
        t.debug_at(1, DebugCommand::SetTier { id: T1, tier });
    }
    run_checked(&mut t, 1);
    t
}

/// [`plant`] with player 1's finished Vehicle Assembly ([`VA`]) at [`VA_AT`], the Tunnel at Tier 2 (its parent from
/// then on), [`started`].
fn assembly() -> TestSim {
    started(plant().spawn(ASSEMBLY, Some(P1), SpawnAt::Tile(VA_AT)), 2)
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

/// `player`'s Tunnel Space, used and available.
fn tunnel_space(t: &TestSim, player: PlayerId) -> (u32, u32) {
    let cap = t.sim().army_cap(player).expect("a Syndicate player");
    (cap.used, cap.available)
}

/// What the gate says to `player`'s `command` now (`Sim::validate`).
fn gate(t: &TestSim, player: PlayerId, command: Command) -> Result<(), Reject> {
    t.sim().validate(&PlayerCommand { player, command })
}

/// An Expand of a Vehicle Assembly at `origin`, unturned, to `tunnel`.
fn expand(tunnel: ObjectId, origin: TilePos) -> Command {
    let command = TunnelCommand::Expand {
        kind: ASSEMBLY,
        origin,
        orientation: BuildingOrientation::default(),
    };
    Command::Tunnel { tunnel, command }
}

fn eject(tunnel: ObjectId, kind: ObjectKind) -> Command {
    Command::Tunnel {
        tunnel,
        command: TunnelCommand::Eject { kind },
    }
}

fn production(producer: ObjectId, command: ProductionCommand) -> Command {
    Command::Production {
        objects: vec![producer],
        command,
    }
}

/// The Vehicle Assembly's "**Q** Assault Bike".
fn train_bike(producer: ObjectId) -> Command {
    production(producer, ProductionCommand::Train(BIKE))
}

fn rally(producer: ObjectId, target: Target) -> Command {
    production(producer, ProductionCommand::SetRally(target))
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

/// Object `id`'s shape where it stands (`sc_sim::shape`): a Bike's Silhouette turned with its facing.
fn shape(t: &TestSim, id: ObjectId) -> Shape {
    Shape::of(object(t, id))
}

/// The paths of Tunnel `tunnel`'s Side A.
fn paths(t: &TestSim, tunnel: ObjectId) -> Paths {
    Paths::of_object(object(t, tunnel)).expect("a Tunnel's paths")
}

/// The ids of player 1's units inside the network, in id order.
fn inside(t: &TestSim) -> Vec<ObjectId> {
    t.sim().network(P1).map(|held| held.object.id).collect()
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

/// The ticks in which each unit came out of `tunnel`, in order.
fn exits(t: &TestSim, tunnel: ObjectId) -> Vec<(Tick, ObjectId)> {
    (t.events().iter())
        .filter_map(|(tick, e)| match *e {
            SimEvent::LeftNetwork { unit, tunnel: from } if from == tunnel => Some((*tick, unit)),
            _ => None,
        })
        .collect()
}

/// The units of `tunnel`'s eject queue, in order.
fn queued(t: &TestSim, tunnel: ObjectId) -> Vec<ObjectId> {
    let state = object(t, tunnel).tunnel.as_ref().expect("a Tunnel");
    state.eject.iter().map(|entry| entry.unit).collect()
}

/// The id the next object will have, with no spawn scheduled yet: the objects and the held units have the ids below
/// it.
fn next_id(t: &TestSim) -> u32 {
    let on_map = t.sim().objects().map(|o| o.id.0);
    let held = t.sim().held().map(|held| held.object.id.0);
    on_map.chain(held).max().unwrap_or(0) + 1
}

/// The held store's invariants (`tests/tunnel_network`'s): a held unit is never among the objects, nothing on the map
/// names one, and every unit an eject queue reserves is held in the queue's Tunnel owner's network.
fn check_held(t: &TestSim) {
    let sim = t.sim();
    for held in sim.held() {
        let unit = held.object.id;
        assert!(sim.object(unit).is_none(), "held unit {unit:?} is on the map too");
        for o in sim.objects() {
            let named = sc_sim::testing::referenced_ids(o);
            assert!(!named.contains(&unit), "{:?} names held unit {unit:?}", o.id);
        }
    }
    for o in sim.objects() {
        for entry in o.tunnel.iter().flat_map(|tunnel| &tunnel.eject) {
            let held = sim.held_unit(entry.unit).expect("a queued unit is held");
            assert_eq!(held.object.owner, o.owner, "queued at its owner's Tunnel");
        }
    }
    assert_eq!(dangling_references(sim), []);
}

/// One tick of `t`, with the held store's invariants.
fn step_checked(t: &mut TestSim) {
    t.step();
    check_held(t);
}

/// Runs `t` until `tick`, checking the held store's invariants after every tick.
fn run_checked(t: &mut TestSim, tick: Tick) {
    while t.sim().tick() < tick {
        step_checked(t);
    }
}

/// [`run_checked`] for `ticks` more ticks.
fn run_for(t: &mut TestSim, ticks: Tick) {
    let until = t.sim().tick() + ticks;
    run_checked(t, until);
}

/// Steps until `done`, at most `ticks` ticks; panics naming `what` otherwise.
fn step_until(t: &mut TestSim, ticks: Tick, what: &str, done: impl Fn(&TestSim) -> bool) {
    let end = t.sim().tick() + ticks;
    while !done(t) {
        assert!(t.sim().tick() < end, "{what}: not by tick {end}\n{}", t.ascii(None));
        step_checked(t);
    }
}

/// `count` Bikes trained at [`VA`] from the next tick, with no rally point, each into the network: their ids, in the
/// order they were finished, once the last is.
fn bikes_inside(t: &mut TestSim, count: usize) -> Vec<ObjectId> {
    let start = t.sim().tick() + 1;
    for _ in 0..count {
        t.at(start, P1, train_bike(VA));
    }
    let limit = BIKE_FRAMES * count as Tick + 2;
    step_until(t, limit, "the Bikes are trained", |t| produced(t, VA).len() == count);
    let bikes: Vec<ObjectId> = produced(t, VA).iter().map(|&(_, unit)| unit).collect();
    assert_eq!(inside(t), bikes, "all in the network");
    bikes
}
