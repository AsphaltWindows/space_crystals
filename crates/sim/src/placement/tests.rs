//! One test per placement rule (the `placement` module docs list them), on the pure check: hand-made maps, areas and
//! occupants, in Layout mode unless the test is about sight. `crates/sim/tests/placement.rs` checks them in games.
//!
//! By topic:
//! - `ground`: the tile rules of the ground and sight: on the map, Buildable, flat, Visible (rules 1 to 4);
//! - `occupants`: the tile rules of what is on a tile: structures, resources, ground units (rules 5 to 7), the
//!   player's reserved sites, and what a player who never explored a tile knows of it (`Mode::Known`);
//! - `footprint`: the rules of the footprint as a whole (the build area, plates onto patches, structures only),
//!   and the order the problems come in;
//! - `tunnel_area`: the Tunnel Area's rules (slice M4-B): on the map and Drillable (design change 0242), and a
//!   layout's expansions inside their Tunnel's area.

mod footprint;
mod ground;
mod occupants;
mod tunnel_area;

use glam::Vec2;

use super::*;
use crate::map::{Map, Tile, TileLook, TilePreset};
use crate::nav::TileRect;
use crate::object::{Placement, Rotation};
use crate::sim::Sim;
use crate::testing::ScenarioBuilder;
use crate::{PlayerId, VisibilityState};

use ObjectKind::{
    Barracks, CrystalPatch, DeploymentCenter, ExtractionFacility, ExtractionPlate, Generator, Guard, Peacekeeper,
    SupplyDeliveryStation,
};
use TileProblem::{NotBuildable, NotFlat, NotVisible, OffMap, Reserved, Resource, Structure, Unit};
use WholeProblem::{NotAStructure, NotOnPatch, OutsideBuildArea};

fn t(x: i32, y: i32) -> TilePos {
    TilePos::new(x, y)
}

/// A tile-based object with its footprint's top-left tile at `(x, y)`, unturned.
fn on(kind: ObjectKind, x: i32, y: i32) -> Occupant {
    let placement = Placement::new(t(x, y), kind.def().footprint);
    Occupant {
        kind,
        pos: placement.center(),
        placement: Some(placement),
        tier: None,
    }
}

/// A unit centred on `(x, y)`, space units.
fn unit(kind: ObjectKind, x: f32, y: f32) -> Occupant {
    Occupant {
        kind,
        pos: Vec2::new(x, y),
        placement: None,
        tier: None,
    }
}

/// The area of a Deployment Center at `(x, y)` (which needn't be among the occupants): 12 tiles around its 4 × 4.
fn dc_area(map: &Map, x: i32, y: i32) -> BuildArea {
    let dc = Placement::new(t(x, y), DeploymentCenter.def().footprint);
    BuildArea::of(map, [(DeploymentCenter, &dc)])
}

/// [`check`] in Layout mode, unturned.
fn place(map: &Map, area: &BuildArea, occupants: &[Occupant], kind: ObjectKind, x: i32, y: i32) -> PlacementCheck {
    let site = Site {
        map,
        area,
        mode: Mode::Layout,
        method: Method::Direct,
    };
    check(
        &site,
        occupants.iter().copied(),
        kind,
        t(x, y),
        BuildingOrientation::default(),
    )
}

/// A flat Plane map of at most 16 × 16 tiles, all of it in the build area of a Deployment Center in its corner.
fn open(w: u32, h: u32) -> (Map, BuildArea) {
    let map = Map::new(w, h);
    let area = dc_area(&map, 0, 0);
    (map, area)
}
