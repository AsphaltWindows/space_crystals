//! Warp In (slice M3-E; `factions/colonists/economy.md > Warp In`, `core/entities.md > Placement Validation > Warp
//! In`): a Beacon transforms in place into an Egress Node, a Robo Portal or a Bio Portal. One test per acceptance
//! point of the plan (`artifacts/director/m3-plan.md` §3.3 E): the timings, the interpolated maxima and the damage
//! carried through them (design change 0110), the footprint derived from the Beacon (design change 0121), its ring
//! (ground units block it, 0118), its sight (0119) and its claim from the first frame (0109), the cancel, the
//! destruction, the gate against `Sim::warp_in_check`, saves and replays; and the gate's rules, fuzzed. The
//! scenarios are built with test maps and spawns, not with the Colonists' Starting Conditions.
//! `TestSim` checks the movement invariants after every tick.
//!
//! One test binary:
//! - `timings`: each building's WarpInTime, its cost paid as the command applies, and the building at work from
//!   the next tick;
//! - `interpolation`: MaxHP and MaxShields interpolated by the formulas, and the damage taken while warping in or
//!   before (design change 0110);
//! - `footprints`: the footprint derived from the Beacon and its orientation (0121), its ring of tiles clear (0118),
//!   every tile visible (0119), and solid from the first frame (0109);
//! - `under_warp_in`: a building under Warp In takes no drop-off (0116), keeps providing Beacon Capacity (0112) and
//!   keeps its owner undefeated (0113); destroyed, it refunds nothing (0115) and its last structure defeats its
//!   owner;
//! - `cancel`: the 75% refund and the Beacon back with the fraction of its HP (0111), and the cancel going to every
//!   transforming object named;
//! - `gate`: the gate's other rules, a transforming Beacon, the gate and `Sim::warp_in_check` in agreement, and
//!   the gate fuzzed;
//! - `saves`: a game saved in the middle of its Warp Ins, and its replay.

mod cancel;
mod footprints;
mod gate;
mod interpolation;
mod saves;
mod timings;
mod under_warp_in;

use sc_sim::economy::{RATIO_ONE, Refund};
use sc_sim::placement::{self, Method, Mode, Occupant, Site};
use sc_sim::sides::DIRECTIONS;
use sc_sim::testing::{ScenarioBuilder, TestSim, dangling_references};
use sc_sim::transform::warp_in_footprint;
use sc_sim::{
    Ability, BuildArea, BuildingOrientation, Command, ConstructionCommand, DebugCommand, Faction, Object, ObjectId,
    ObjectKind, OrderKind, Placement, PlacementCheck, PlacementProblem, PlayerCommand, PlayerId, ProductionCommand,
    Progress, Reject, Replay, Rng, Rotation, SU_PER_GU, SideLetter, Sim, SimConfig, SimEvent, SpawnAt, Target, Tick,
    TickInputs, TilePos, TileProblem, Transform, Vec2, sides,
};

use ConstructionCommand::{CancelTransform, WarpIn};
use Faction::Colonists;
use ObjectKind::{Beacon, BioPortal, CrystalPatch, Drone, EgressNode, Generator, Peacekeeper, RoboPortal};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

fn flat(w: usize, h: usize) -> String {
    vec![".".repeat(w); h].join("\n")
}

fn at(x: i32, y: i32) -> TilePos {
    TilePos::new(x, y)
}

fn tile(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Tile(at(x, y))
}

fn id(n: u32) -> ObjectId {
    ObjectId(n)
}

fn construction(objects: &[u32], command: ConstructionCommand) -> Command {
    Command::Construction {
        objects: objects.iter().map(|&n| ObjectId(n)).collect(),
        command,
    }
}

/// A Warp In of `kind`, unturned: its footprint follows from the Beacon (design change 0121).
fn warp(kind: ObjectKind) -> ConstructionCommand {
    WarpIn {
        kind,
        orientation: BuildingOrientation::default(),
    }
}

/// Every orientation a building can be given: the four turns, each with or without either flip.
fn every_orientation() -> Vec<BuildingOrientation> {
    [Rotation::R0, Rotation::R90, Rotation::R180, Rotation::R270]
        .into_iter()
        .flat_map(|rotation| {
            [(false, false), (true, false), (false, true), (true, true)].map(|(flip_h, flip_v)| BuildingOrientation {
                rotation,
                flip_h,
                flip_v,
            })
        })
        .collect()
}

/// Sets `player`'s bank to `crystals` (nothing else).
fn fund(player: PlayerId, crystals: u32) -> DebugCommand {
    DebugCommand::SetResources {
        player,
        crystals,
        supplies: 0,
        alloys: 0,
        essence: 0,
        flux: 0,
    }
}

fn bank(t: &TestSim, player: PlayerId) -> u32 {
    t.sim().player(player).unwrap().crystals
}

fn object(t: &TestSim, n: u32) -> &Object {
    t.sim().object(id(n)).expect("the object exists")
}

/// Object `n`'s transformation. Panics if it isn't transforming.
fn transform(t: &TestSim, n: u32) -> &Transform {
    object(t, n).transform.as_ref().expect("transforming")
}

/// Schedules player 1's `command` for the next tick and runs it.
fn next(t: &mut TestSim, command: Command) -> &mut TestSim {
    let tick = t.sim().tick() + 1;
    t.at(tick, P1, command).step()
}

/// The transformations completed so far: tick, object, kind.
fn completions(t: &TestSim) -> Vec<(Tick, u32, ObjectKind)> {
    t.events()
        .iter()
        .filter_map(|(tick, e)| match e {
            SimEvent::TransformCompleted { id, kind } => Some((*tick, id.0, *kind)),
            _ => None,
        })
        .collect()
}

/// The `TransformCancelled` events so far: tick, object, kind, refund.
fn cancellations(t: &TestSim) -> Vec<(Tick, u32, ObjectKind, u32)> {
    t.events()
        .iter()
        .filter_map(|(tick, e)| match e {
            SimEvent::TransformCancelled { id, kind, refund } => Some((*tick, id.0, *kind, *refund)),
            _ => None,
        })
        .collect()
}

/// The `Train` of a unit of `kind`, to the producers `objects`.
fn train(objects: &[u32], kind: ObjectKind) -> Command {
    Command::Production {
        objects: objects.iter().map(|&n| ObjectId(n)).collect(),
        command: ProductionCommand::Train(kind),
    }
}

/// Checks that the gate rejects `player`'s `command` for `reason`, then runs the next tick with it and checks that
/// it changed nothing: the state and the events are those of the same tick without it, but for the
/// `PlacementRefused` of a footprint refused by the placement check, the tick's first event. Returns that event.
fn assert_rejected(t: &mut TestSim, player: PlayerId, command: Command, reason: Reject) -> Option<SimEvent> {
    let cmd = PlayerCommand {
        player,
        command: command.clone(),
    };
    assert_eq!(t.sim().validate(&cmd), Err(reason), "{command:?}");
    let mut without = t.sim().clone();
    without.step(&TickInputs::default());
    let tick = t.sim().tick() + 1;
    t.at(tick, player, command).step();
    assert_eq!(t.sim().state_hash(), without.state_hash(), "{reason}");
    let reported = usize::from(matches!(reason, Reject::Unplaceable(_)));
    assert_eq!(&t.sim().events()[reported..], without.events(), "{reason}");
    (reported == 1).then(|| t.sim().events()[0].clone())
}

/// `builder` with both players playing the Colonists (plan decision T27), whose army cap is Beacon Capacity.
fn colonists(builder: ScenarioBuilder) -> ScenarioBuilder {
    builder.faction(P1, Colonists).faction(P2, Colonists)
}

/// `sim` saved, with object `n`'s `field` set to `value`, and loaded: a state no command makes on demand.
fn edited(sim: &Sim, n: u32, field: &str, value: serde_json::Value) -> Sim {
    let mut state = serde_json::to_value(sim).unwrap();
    let objects = state["objects"].as_array_mut().unwrap();
    let o = objects.iter_mut().find(|o| o["id"] == n).expect("the object is saved");
    o[field] = value;
    serde_json::from_value(state).unwrap()
}

/// A 24 × 16 flat map for two Colonists players: player 1's Beacons 1 at (8, 6) to (9, 7) and 2 at (16, 6), and
/// Generator 3 at (2, 2), a structure that is neither a Beacon nor a drop-off nor a provider of Beacon Capacity;
/// player 2's Beacon 4 at (16, 12) and Generator 5 at (20, 1). A Robo Portal centred on Beacon 1 covers (7, 5) to
/// (10, 8).
fn base() -> ScenarioBuilder {
    colonists(ScenarioBuilder::new(&flat(24, 16)))
        .spawn(Beacon, Some(P1), tile(8, 6))
        .spawn(Beacon, Some(P1), tile(16, 6))
        .spawn(Generator, Some(P1), tile(2, 2))
        .spawn(Beacon, Some(P2), tile(16, 12))
        .spawn(Generator, Some(P2), tile(20, 1))
}

/// A building of the Beacon's Constructs List (`factions/colonists/economy.md > Warp In > Constructs List`), as the
/// tests warp it in on Beacon 1.
#[derive(Copy, Clone)]
struct Building {
    kind: ObjectKind,
    /// The top-left tile of its footprint on Beacon 1, unturned (`> Warp In Command`: "centered on the Beacon's
    /// location"; an odd-sized one with its extras at the bottom-right, design change 0121).
    origin: (i32, i32),
    cost: u32,
    /// Its WarpInTime.
    frames: u32,
    /// What it produces, if anything.
    unit: Option<ObjectKind>,
}

const BUILDINGS: [Building; 3] = [
    Building {
        kind: EgressNode,
        origin: (8, 6),
        cost: 75,
        frames: 320,
        unit: None,
    },
    Building {
        kind: RoboPortal,
        origin: (7, 5),
        cost: 250, // design change 0167 (was 400)
        frames: 640,
        unit: Some(Drone),
    },
    Building {
        kind: BioPortal,
        origin: (8, 6),
        cost: 150,
        frames: 480,
        unit: Some(ObjectKind::SabreLegionnaire),
    },
];

/// The base with Beacon 1 warping in a Robo Portal from tick 1 on, which player 2's Peacekeeper 6, at the centre of
/// tile (12, 7), shoots from tick 100 to 130; auto-targeting is off.
fn shot_at() -> TestSim {
    let mut t = base().start();
    t.without_auto_targeting()
        .debug_at(1, fund(P1, 1000))
        .at(1, P1, construction(&[1], warp(RoboPortal)))
        .spawn_unit_at(1, Peacekeeper, P2, at(12, 7).center())
        .order_at(100, P2, &[id(6)], OrderKind::Attack(id(1)), false)
        .order_at(130, P2, &[id(6)], OrderKind::Stop, false);
    t
}
