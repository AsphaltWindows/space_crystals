//! The Garrison (slice M5-N2, plan decision T72; `factions/cults/structures/garrison.md`), through the gate, one test
//! binary:
//! - `entering`: Recruits, Militants, Gunners and Disciples go in through the door on side B, up to four; the fifth's
//!   Enter ends and it idles; a site can't be entered; the right-click, a Repair on a full damaged Garrison (design
//!   change 0378); Unit Control;
//! - `eject`: Eject All lets everyone out through the door at once, passing through each other until they are clear of
//!   the footprint, idle; the empty Garrison refuses it;
//! - `fire`: the occupants fire from the footprint: 0319's verification, the stand-in's attack, the Gunner's MinRange,
//!   no attack while empty, each picks its own target;
//! - `protection`: units inside can't be targeted, nor hit by a blast over the Garrison, and come out alive when it is
//!   destroyed.
//!
//! Every test runs under the invariants of `TestSim` (no two units overlap but by an exception, none stands in a
//! footprint but a unit in the doorway its building let it into), and checks that no reference to a unit that went
//! inside is left behind.

mod eject;
mod entering;
mod fire;
mod protection;

use sc_sim::testing::{ScenarioBuilder, TestSim, dangling_references};
use sc_sim::{
    Command, DebugCommand, Faction, HoldCommand, ObjectId, ObjectKind, OrderKind, PlayerCommand, PlayerId, Reject,
    SimEvent, SpawnAt, Tick, TilePos, Vec2,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const GARRISON: ObjectKind = ObjectKind::Garrison;
const RECRUIT: ObjectKind = ObjectKind::Recruit;
const MILITANT: ObjectKind = ObjectKind::Militant;
const GUNNER: ObjectKind = ObjectKind::Gunner;
const DISCIPLE: ObjectKind = ObjectKind::Disciple;

/// The Garrison of [`base`].
const G1: ObjectId = ObjectId(1);
/// [`base`]'s Garrison's door, the middle of its side B (x = 768), entrance and exit.
const DOOR: Vec2 = Vec2::new(768.0, 704.0);
/// Its footprint: x 768..896, y 640..768.
const LO: Vec2 = Vec2::new(768.0, 640.0);
const HI: Vec2 = Vec2::new(896.0, 768.0);
/// An infantry unit's half extents (`factions/cults/units/militant.md > Silhouette - 24x24 space units`; the Recruit,
/// the Gunner and the Disciple alike).
const HALF: Vec2 = Vec2::new(12.0, 12.0);
/// Space units in a grid unit.
const GU: f32 = 64.0;

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

/// Player 1's Garrison (1) on a 30 × 22 map, unturned (SymmetryType AAAB, B on the left): it covers tiles (12, 10) to
/// (13, 11), x 768..896 and y 640..768. Its door, B, faces west, its middle at [`DOOR`].
fn base() -> ScenarioBuilder {
    cults(&flat(30, 22)).spawn(GARRISON, Some(P1), tile(12, 10))
}

/// The id the next object will have, with no spawn scheduled yet: the objects and the held units have the ids below
/// it.
fn next_id(t: &TestSim) -> u32 {
    let on_map = t.sim().objects().map(|o| o.id.0);
    let held = t.sim().held().map(|held| held.object.id.0);
    on_map.chain(held).max().unwrap_or(0) + 1
}

/// Units of player 1's of `kinds`, spawned at `tick` in a column 32 su apart, west of [`base`]'s Garrison's door and
/// facing it, the first at `front`; their ids, from the next free one.
fn column(t: &mut TestSim, tick: Tick, kinds: &[ObjectKind], front: Vec2) -> Vec<ObjectId> {
    let first = next_id(t);
    for (i, &kind) in kinds.iter().enumerate() {
        t.spawn_unit_at(tick, kind, P1, front - Vec2::new(32.0 * i as f32, 0.0));
    }
    (0..kinds.len() as u32).map(|i| id(first + i)).collect()
}

fn enter(units: &[ObjectId], target: ObjectId) -> Command {
    Command::Order {
        units: units.to_vec(),
        kind: OrderKind::Enter(target),
        queue: false,
    }
}

fn eject_all(buildings: &[ObjectId]) -> Command {
    Command::Holding {
        objects: buildings.to_vec(),
        command: HoldCommand::EjectAll,
    }
}

fn validate(t: &TestSim, command: Command) -> Result<(), Reject> {
    t.sim().validate(&PlayerCommand { player: P1, command })
}

/// The units inside `building`, longest inside first.
fn inside(t: &TestSim, building: ObjectId) -> Vec<ObjectId> {
    t.sim().held_in(building).iter().map(|held| held.object.id).collect()
}

/// The units that came out of a building's doorway so far, as `(tick, unit)`.
fn exits(t: &TestSim) -> Vec<(Tick, ObjectId)> {
    (t.events().iter())
        .filter_map(|(tick, e)| match e {
            SimEvent::LeftBuilding { unit, .. } => Some((*tick, *unit)),
            _ => None,
        })
        .collect()
}

/// The Enters that failed so far, as `(unit, full)`.
fn failed(t: &TestSim) -> Vec<(ObjectId, bool)> {
    (t.events().iter())
        .filter_map(|(_, e)| match e {
            SimEvent::EnterFailed { unit, full, .. } => Some((*unit, *full)),
            _ => None,
        })
        .collect()
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

/// Player 1's units of `kinds` spawned in tick 1 in a [`column`] west of the door and ordered into `t`'s Garrison
/// [`G1`] at tick 2; run until they all are in, within 200 ticks. They are in the pool's order, longest inside first.
fn fill(t: &mut TestSim, kinds: &[ObjectKind]) -> Vec<ObjectId> {
    let units = column(t, 1, kinds, DOOR - Vec2::new(60.0, 0.0));
    t.at(2, P1, enter(&units, G1));
    run_checked(t, 2);
    while inside(t, G1).len() < units.len() {
        assert!(
            t.sim().tick() < 200,
            "{} are in\n{}",
            inside(t, G1).len(),
            t.ascii(Some(DOOR))
        );
        step_checked(t);
    }
    inside(t, G1)
}

/// Whether a box at `pos` with an infantry unit's half extents overlaps the Garrison's footprint (touching is free).
fn on_footprint(pos: Vec2) -> bool {
    pos.x + HALF.x > LO.x && pos.x - HALF.x < HI.x && pos.y + HALF.y > LO.y && pos.y - HALF.y < HI.y
}
