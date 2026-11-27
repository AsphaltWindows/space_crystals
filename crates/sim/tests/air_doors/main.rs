//! The Tier 3 air doors, the Forge, the Aerospace Assembly and the Phantom (slice M6-SD, plan decision T96;
//! `factions/syndicate/tunnels.md > Tunnel > Look by tier`, `factions/syndicate/tunnel_menus.md > EjectMenu`,
//! `factions/syndicate/structures/forge.md`, `factions/syndicate/structures/aerospace_assembly.md`,
//! `factions/syndicate/units/phantom.md`; design changes 0367's and 0431's placeholders), through the gate, one test
//! binary:
//! - `building`: a Tier 2 Tunnel builds the Forge behind a Vehicle Assembly, for 200 Space Crystals in 480 frames, and
//!   a Tier 3 one the Aerospace Assembly behind a Forge, for 300 in 640, each refused a tier lower and without its
//!   prerequisite;
//! - `production`: the Aerospace Assembly trains the Phantom (Q) behind an Optics Lab, for 175 Space Crystals in 320
//!   frames, Tunnel Space 2, into the network; with a surface rally point the Phantom is auto-ejected by the air exit
//!   door, rises out of it and flies to the point;
//! - `out`: an air unit's lane, its door's rate of one every 8 frames beside Side A's ground lane (five Phantoms out
//!   while Guards walk out), its 16 frames' rise, holding still as an air target, two Phantoms rising at once;
//! - `entering`: a right-click on an own Tier 3 Tunnel takes a Phantom to the air entry door, where it descends for
//!   16 frames, an air target, the door's rate holding the next one back, and is held;
//! - `tiers`: what each tier lets in and out, and says, for a Guard, a Bike and a Phantom; a Tunnel upgrading to Tier 3
//!   admits no air unit (0249), and the last Tier 3 Tunnel lost strands the aircraft inside;
//! - `edge`: a Tier 3 Tunnel at the map's edge lets its Phantoms out and in inside the map (a door's offset tuned past
//!   the edge is `tests/rules.rs`'s, as a SetRules changes the process's rules);
//! - `phantom`: the Phantom's laser hits a Sparrow and a Peacekeeper;
//! - `saves`: a game with Phantoms in training, rising, inside, queued and descending goes on alike after a save and in
//!   a replay.
//!
//! Every test runs under the invariants of `TestSim`, and this binary's held-store checks ([`check_held`]) after each
//! tick it steps.

mod building;
mod edge;
mod entering;
mod out;
mod phantom;
mod production;
mod saves;
mod tiers;

use sc_sim::network::{self, Paths};
use sc_sim::testing::{ScenarioBuilder, TestSim, dangling_references};
use sc_sim::{
    BuildingOrientation, Command, DebugCommand, Faction, Object, ObjectId, ObjectKind, OrderKind, PlayerCommand,
    PlayerId, ProductionCommand, Reject, SimEvent, SpawnAt, Target, Tick, TilePos, TunnelCommand, Vec2,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const PHANTOM: ObjectKind = ObjectKind::Phantom;
const GUARD: ObjectKind = ObjectKind::Guard;
const TUNNEL: ObjectKind = ObjectKind::Tunnel;

/// `factions/syndicate/structures/aerospace_assembly.md > Placeholder stats`: "**Phantom** 175 SC, 320 f, 2".
const PHANTOM_PRICE: u32 = 175;
const PHANTOM_FRAMES: Tick = 320;
/// `factions/syndicate/units/phantom.md > Placeholder stats` (design change 0431): "it **descends** into the entry door
/// and **rises** out of the exit door over **16 frames (1 s) each**".
const DOOR_FRAMES: Tick = 16;
/// `factions/syndicate/tunnel_menus.md > EjectMenu`: "one aircraft per 8 frames (0.5 s), placeholder (the same rate
/// for the air entry door)".
const DOOR_RATE: Tick = 8;
/// `factions/syndicate/structures/hideout.md`: "Guard: 125 Space Crystals", "120 frames (7.5 seconds)".
const GUARD_FRAMES: Tick = 120;

/// Player 1's Tunnel in [`base`].
const T1: ObjectId = ObjectId(1);
/// Player 1's Optics Lab in [`hangar`]: the Phantom's and the Guard's prerequisite.
const LAB: ObjectId = ObjectId(2);
/// Player 1's Hideout in [`hangar`], which trains Guards.
const HIDEOUT: ObjectId = ObjectId(3);
/// Player 1's Vehicle Assembly in [`hangar`]: the Forge's prerequisite.
const VA: ObjectId = ObjectId(4);
/// Player 1's Forge in [`hangar`]: the Aerospace Assembly's prerequisite.
const FORGE: ObjectId = ObjectId(5);
/// Player 1's Aerospace Assembly in [`hangar`], which trains the Phantom.
const AA: ObjectId = ObjectId(6);
/// Where [`hangar`]'s buildings stand, in [`base`]'s Tunnel's areas: the Optics Lab on (16, 14) to (17, 15) and the
/// Hideout on (12, 14) to (13, 15), in the Tier 1 area; the Vehicle Assembly on (9, 7) to (11, 9), in the Tier 2 one;
/// the Forge on (16, 8) to (17, 9); the Aerospace Assembly on (8, 14) to (10, 16), in the Tier 3 one only.
const LAB_AT: TilePos = TilePos::new(16, 14);
const HIDEOUT_AT: TilePos = TilePos::new(12, 14);
const VA_AT: TilePos = TilePos::new(9, 7);
const FORGE_AT: TilePos = TilePos::new(16, 8);
const AA_AT: TilePos = TilePos::new(8, 14);
/// The Space Crystals player 1 has from tick 1.
const BANK: u32 = 10_000;

/// [`base`]'s Tunnel's centre, and the centres of its air doors (`Paths::air_entry`, `Paths::air_exit`): "the entry
/// door over the entry path's half of the Tunnel, the exit door over the exit path's half", 52 su west and east of the
/// centre, as Side A faces north (Side D west, Side B east).
const CENTRE: Vec2 = Vec2::new(896.0, 768.0);
const AIR_ENTRY: Vec2 = Vec2::new(844.0, 768.0);
const AIR_EXIT: Vec2 = Vec2::new(948.0, 768.0);

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
/// unturned, on tiles (12, 10) to (15, 13), x 768..1024 and y 640..896: Side A faces north. Its Tier 1 area is (10, 8)
/// to (17, 15), its Tier 2 area (9, 7) to (18, 16), its Tier 3 area (8, 6) to (19, 17).
fn base() -> ScenarioBuilder {
    ScenarioBuilder::new(&flat(40, 24))
        .faction(P1, Faction::Syndicate)
        .faction(P2, Faction::Syndicate)
        .spawn(TUNNEL, Some(P1), tile(12, 10))
}

/// [`base`] with player 1's Optics Lab ([`LAB`]), Hideout ([`HIDEOUT`]), Vehicle Assembly ([`VA`]), Forge ([`FORGE`])
/// and Aerospace Assembly ([`AA`]), finished, where [`LAB_AT`] and the others say.
fn hangar_scenario() -> ScenarioBuilder {
    base()
        .spawn(ObjectKind::OpticsLab, Some(P1), SpawnAt::Tile(LAB_AT))
        .spawn(ObjectKind::Hideout, Some(P1), SpawnAt::Tile(HIDEOUT_AT))
        .spawn(ObjectKind::VehicleAssembly, Some(P1), SpawnAt::Tile(VA_AT))
        .spawn(ObjectKind::Forge, Some(P1), SpawnAt::Tile(FORGE_AT))
        .spawn(ObjectKind::AerospaceAssembly, Some(P1), SpawnAt::Tile(AA_AT))
}

/// `scenario` started with auto-targeting off, [`BANK`] Space Crystals and 100 Supplies for player 1 and, if `tier` is
/// above 1, the Tunnel set to it (a debug command stands for its upgrades, slice M4-G's), all in tick 1, which has run.
fn started(scenario: ScenarioBuilder, tier: u8) -> TestSim {
    let mut t = scenario.start();
    t.without_auto_targeting();
    t.debug_at(1, banks(P1, BANK, 100));
    if tier > 1 {
        t.debug_at(1, DebugCommand::SetTier { id: T1, tier });
    }
    run_checked(&mut t, 1);
    t
}

/// [`hangar_scenario`] [`started`] at Tier 3: the Aerospace Assembly's parent Tunnel from then on, its air doors where
/// [`AIR_ENTRY`] and [`AIR_EXIT`] say.
fn hangar() -> TestSim {
    let t = started(hangar_scenario(), 3);
    let kinds = [LAB, HIDEOUT, VA, FORGE, AA].map(|id| object(&t, id).kind);
    use ObjectKind::{AerospaceAssembly, Forge, Hideout, OpticsLab, VehicleAssembly};
    assert_eq!(kinds, [OpticsLab, Hideout, VehicleAssembly, Forge, AerospaceAssembly]);
    assert_eq!(t.sim().parent_tunnel(AA), Some(T1));
    let doors = paths(&t, T1);
    assert_eq!(
        (object(&t, T1).pos, doors.air_entry, doors.air_exit),
        (CENTRE, AIR_ENTRY, AIR_EXIT)
    );
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

/// An Expand of `kind` at `origin`, unturned, to `tunnel`.
fn expand(tunnel: ObjectId, kind: ObjectKind, origin: TilePos) -> Command {
    let command = TunnelCommand::Expand {
        kind,
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

/// The Aerospace Assembly's "**Q** Phantom", or the Hideout's Guard: a Train of `kind` at `producer`.
fn train(producer: ObjectId, kind: ObjectKind) -> Command {
    production(producer, ProductionCommand::Train(kind))
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

fn spawn(kind: ObjectKind, owner: PlayerId, at: SpawnAt) -> DebugCommand {
    DebugCommand::Spawn {
        kind,
        owner: Some(owner),
        at,
        orientation: BuildingOrientation::default(),
    }
}

fn object(t: &TestSim, id: ObjectId) -> &Object {
    t.sim().object(id).unwrap_or_else(|| panic!("{id:?} is on the map"))
}

/// The paths and doors of Tunnel `tunnel`.
fn paths(t: &TestSim, tunnel: ObjectId) -> Paths {
    Paths::of_object(object(t, tunnel)).expect("a Tunnel's paths")
}

/// How far up out of an air door unit `id` is (`network::door_lift`): `None` once it is neither rising nor descending.
fn lift(t: &TestSim, id: ObjectId) -> Option<f32> {
    network::door_lift(object(t, id))
}

/// The ticks done of unit `id`'s rise out of the air exit door, while it rises (`Mover::rise`).
fn rising(t: &TestSim, id: ObjectId) -> Option<u32> {
    let mover = object(t, id).mover.as_ref().expect("a unit");
    mover.rise.map(|rise| rise.ticks_done())
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

/// The ticks in which each unit went into the network by `tunnel`, in order.
fn entries(t: &TestSim, tunnel: ObjectId) -> Vec<(Tick, ObjectId)> {
    (t.events().iter())
        .filter_map(|(tick, e)| match *e {
            SimEvent::EnteredNetwork { unit, tunnel: by } if by == tunnel => Some((*tick, unit)),
            _ => None,
        })
        .collect()
}

/// The hits on `target` so far: their ticks, attackers and damage.
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

/// `count` units of `kind` trained at `producer` from the next tick, with no rally point, each into the network: their
/// ids, in the order they were finished, once the last is. The others inside stay.
fn trained_inside(t: &mut TestSim, producer: ObjectId, kind: ObjectKind, count: usize) -> Vec<ObjectId> {
    let start = t.sim().tick() + 1;
    let before = produced(t, producer).len();
    for _ in 0..count {
        t.at(start, P1, train(producer, kind));
    }
    let frames = kind_frames(kind);
    let limit = frames * count as Tick + 2;
    step_until(t, limit, "the units are trained", |t| {
        produced(t, producer).len() == before + count
    });
    let units: Vec<ObjectId> = produced(t, producer)[before..].iter().map(|&(_, unit)| unit).collect();
    let held = inside(t);
    assert!(units.iter().all(|unit| held.contains(unit)), "all in the network");
    units
}

/// The frames a Train of `kind` takes ([`PHANTOM_FRAMES`], [`GUARD_FRAMES`]).
fn kind_frames(kind: ObjectKind) -> Tick {
    match kind {
        PHANTOM => PHANTOM_FRAMES,
        GUARD => GUARD_FRAMES,
        other => panic!("no Train of {other:?} here"),
    }
}
