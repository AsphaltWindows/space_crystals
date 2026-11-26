//! The start of a game (slice M2-F1): each player gets their faction's Starting Conditions (`factions/gdo/economy.md >
//! Starting Conditions`), the banks and the objects of their faction's Starting Layout at their slot, exactly where
//! the map puts them (design change 0008). The rules of the layouts (`core/map_editor.md > Starting Layouts`): each
//! one valid on its own with its faction's placement rules minus sight (0097); on a Skirmish Default map exactly the
//! Starting Conditions, complete at every start (0009, 0012), for every faction (plan decision T63: the Cults' too
//! since slice M5-B); derived "outdated" flags. And `dev_skirmish`'s GDO bases (0095; plan risk R13). The Starting
//! Conditions table is unit-tested in `scenario/starting.rs`, a Cults player's start in `cults_start.rs`, the camera's
//! start in the client.
//!
//! One test binary:
//! - `start`: each player's Starting Conditions at the start of a game (0008), and `dev_skirmish`'s GDO bases;
//! - `layout_rules`: each layout valid on its own by its faction's placement rules minus sight (0097), a
//!   structure over a unit reported once, plates onto patches, and the layouts of a game's players kept apart;
//! - `skirmish_default`: a Skirmish Default map's layouts, exactly the Starting Conditions (0009, 0012), and the
//!   "outdated" flags.

mod layout_rules;
mod skirmish_default;
mod start;

use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{
    BuildingOrientation, Faction, LayoutObject, MapSettings, MapType, ObjectId, ObjectKind, PlacementCheck, PlayerId,
    Rotation, SU_PER_GU, Scenario, ScenarioProblem, Sim, SlotId, Spawn, SpawnAt, SpawnProblem, TilePos, TileProblem,
    Vec2, VisibilityState, WholeProblem, scenarios,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const DC: ObjectKind = ObjectKind::DeploymentCenter;
const EF: ObjectKind = ObjectKind::ExtractionFacility;
const PLATE: ObjectKind = ObjectKind::ExtractionPlate;
const PATCH: ObjectKind = ObjectKind::CrystalPatch;
const PK: ObjectKind = ObjectKind::Peacekeeper;
const HALL: ObjectKind = ObjectKind::RecruitmentHall;
const RECRUIT: ObjectKind = ObjectKind::Recruit;

fn flat(w: usize, h: usize) -> String {
    vec![".".repeat(w); h].join("\n")
}

fn tile(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Tile(TilePos::new(x, y))
}

/// A point in grid units.
fn point(x: f32, y: f32) -> Vec2 {
    Vec2::new(x, y) * SU_PER_GU
}

fn laid(kind: ObjectKind, x: i32, y: i32) -> LayoutObject {
    LayoutObject {
        kind,
        at: tile(x, y),
        orientation: BuildingOrientation::default(),
    }
}

fn turned(kind: ObjectKind, x: i32, y: i32, rotation: Rotation) -> LayoutObject {
    LayoutObject {
        orientation: BuildingOrientation {
            rotation,
            ..Default::default()
        },
        ..laid(kind, x, y)
    }
}

fn unit_at(kind: ObjectKind, at: Vec2) -> LayoutObject {
    LayoutObject {
        kind,
        at: SpawnAt::Point(at),
        orientation: BuildingOrientation::default(),
    }
}

fn patch(x: i32, y: i32) -> Spawn {
    Spawn::new(PATCH, None, tile(x, y))
}

/// A GDO base in the north-west of a map: the Deployment Center from (1, 1) to (4, 4), the Extraction Facility from
/// (6, 1) to (8, 3), and plates on patches at (1, 8) and (2, 8).
fn west_base() -> Vec<LayoutObject> {
    vec![laid(DC, 1, 1), laid(EF, 6, 1), laid(PLATE, 1, 8), laid(PLATE, 2, 8)]
}

/// A Colonists base in the north-west of a map (`factions/colonists/economy.md > Starting Conditions`, slice M3-C):
/// the Egress Node from (1, 1) to (2, 2), the Robo Portal from (4, 1) to (7, 4), and four Drones on the centres of
/// tiles (1, 5) to (2, 6).
fn west_colonists() -> Vec<LayoutObject> {
    let mut base = vec![laid(ObjectKind::EgressNode, 1, 1), laid(ObjectKind::RoboPortal, 4, 1)];
    for (x, y) in [(1.5, 5.5), (2.5, 5.5), (1.5, 6.5), (2.5, 6.5)] {
        base.push(unit_at(ObjectKind::Drone, point(x, y)));
    }
    base
}

/// A Syndicate base in the north-west of a map (`factions/syndicate/tunnels.md > Tunnel > Starting Conditions`, slice
/// M4-B): the Tunnel from (3, 3) to (6, 6), whose Tier 1 Tunnel Area, (1, 1) to (8, 8), lies on the map; the Hideout
/// from (7, 3) to (8, 4), inside it; and the Agent at (5.5, 9.5) grid units.
fn west_syndicate() -> Vec<LayoutObject> {
    vec![
        laid(ObjectKind::Tunnel, 3, 3),
        laid(ObjectKind::Hideout, 7, 3),
        unit_at(ObjectKind::Agent, point(5.5, 9.5)),
    ]
}

/// A Cults base in the north-west of a map (`factions/cults/economy.md > Starting Conditions`, slice M5-B): the
/// Recruitment Hall from (3, 3) to (6, 6), whose Recruitment Area, (0, 0) to (9, 9), lies on the map (design change
/// 0341 point 1, `factions/cults/structures/recruitment_hall.md > RecruitmentArea`: "**The whole area must lie on the
/// map**"), the Storage from (6, 1) to (8, 2), and six Recruits in a row on the centres of tiles (1, 7) to (6, 7).
fn west_cults() -> Vec<LayoutObject> {
    let mut base = vec![laid(HALL, 3, 3), laid(ObjectKind::Storage, 6, 1)];
    for x in 1..=6 {
        base.push(unit_at(RECRUIT, TilePos::new(x, 7).center()));
    }
    base
}
