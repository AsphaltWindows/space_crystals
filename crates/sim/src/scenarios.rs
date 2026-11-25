//! Built-in maps for development, QA and tests, each with its default game setup ([`Scenario::new`]: a GDO player in
//! every slot).

use crate::defs::ObjectKind;
use crate::ids::SlotId;
use crate::map::{Map, Tile, TilePresetId};
use crate::object::Rotation;
use crate::player::Faction;
use crate::scenario::{BuildingOrientation, LayoutObject, MapDef, MapType, Scenario, Slot, Spawn, SpawnAt};
use crate::symmetry::Symmetry;
use crate::units::{CornerPos, SU_PER_GU, TilePos};

/// Names accepted by [`by_name`] (e.g. for the client's `--scenario` flag).
pub const NAMES: &[&str] = &["dev_skirmish", "dev_structures", "empty"];

pub fn by_name(name: &str) -> Option<Scenario> {
    match name {
        "dev_skirmish" => Some(dev_skirmish()),
        "dev_structures" => Some(dev_structures()),
        "empty" => Some(empty(32, 24)),
        _ => None,
    }
}

/// A flat, featureless Custom map without symmetry, with two slots and no objects, and its default setup. The
/// starts sit a quarter of the way in from the top-left and bottom-right corners.
pub fn empty(width: u32, height: u32) -> Scenario {
    Scenario::new(empty_map(width, height))
}

/// The map of [`empty`].
pub fn empty_map(width: u32, height: u32) -> MapDef {
    let terrain = Map::new(width, height); // Panics first if the size is 0.
    let (x, y) = (width / 4, height / 4);
    let starts = [
        TilePos::new(x as i32, y as i32),
        TilePos::new((width - 1 - x) as i32, (height - 1 - y) as i32),
    ];
    MapDef {
        map_type: MapType::Custom,
        name: "empty".into(),
        description: String::new(),
        terrain,
        symmetry: Symmetry::None,
        slots: starts.into_iter().map(Slot::new).collect(),
        spawns: Vec::new(),
    }
}

/// [`dev_structures_map`] with its default setup.
pub fn dev_structures() -> Scenario {
    Scenario::new(dev_structures_map())
}

/// 30 × 12 flat Custom QA map with one of each GDO structure for each of its two slots: slot 1's base in the west,
/// slot 2's in the east, 10 open tiles apart, each with three crystal patches and a plate on one of them
/// (`factions/gdo/economy.md > ExtractionPlate`). Slot 1's Barracks is unturned (its exit side B faces east), slot
/// 2's turned 90° clockwise (B faces south). Both cameras start at the middle of the map, where the view shows both
/// bases.
pub fn dev_structures_map() -> MapDef {
    use ObjectKind::{Barracks, CrystalPatch, DeploymentCenter, ExtractionFacility, ExtractionPlate, Generator};
    let (w, h) = (30, 12);
    let mut spawns = Vec::new();
    let mut place = |kind, slot: Option<u8>, x, y, rotation| {
        let mut spawn = Spawn::new(kind, slot.map(SlotId), SpawnAt::Tile(TilePos::new(x, y)));
        spawn.orientation = BuildingOrientation {
            rotation,
            ..Default::default()
        };
        spawns.push(spawn);
    };
    // Slot 1, in the west.
    place(DeploymentCenter, Some(1), 1, 1, Rotation::R0); // (1, 1) to (4, 4)
    place(Generator, Some(1), 6, 1, Rotation::R0); // (6, 1) to (7, 2)
    place(Barracks, Some(1), 6, 4, Rotation::R0); // (6, 4) to (8, 5)
    place(ExtractionFacility, Some(1), 1, 7, Rotation::R0); // (1, 7) to (3, 9)
    for x in 5..=7 {
        place(CrystalPatch, None, x, 8, Rotation::R0);
    }
    place(ExtractionPlate, Some(1), 5, 8, Rotation::R0);
    // Slot 2, in the east.
    place(DeploymentCenter, Some(2), 25, 7, Rotation::R0); // (25, 7) to (28, 10)
    place(Generator, Some(2), 19, 2, Rotation::R0); // (19, 2) to (20, 3)
    place(Barracks, Some(2), 19, 7, Rotation::R90); // (19, 7) to (20, 9)
    place(ExtractionFacility, Some(2), 26, 1, Rotation::R0); // (26, 1) to (28, 3)
    for x in 22..=24 {
        place(CrystalPatch, None, x, 5, Rotation::R0);
    }
    place(ExtractionPlate, Some(2), 23, 5, Rotation::R0);

    let middle = glam::Vec2::new(w as f32, h as f32) * (SU_PER_GU / 2.0);
    let slots = [TilePos::new(5, 6), TilePos::new(24, 4)]
        .into_iter()
        .map(|start| {
            let mut slot = Slot::new(start);
            slot.camera_start = Some(middle);
            slot
        })
        .collect();
    MapDef {
        map_type: MapType::Custom,
        name: "dev_structures".into(),
        description: "QA map: one of each GDO structure for each of two players.".into(),
        terrain: Map::new(w, h),
        symmetry: Symmetry::None,
        slots,
        spawns,
    }
}

/// [`dev_skirmish_map`] with its default setup.
pub fn dev_skirmish() -> Scenario {
    Scenario::new(dev_skirmish_map())
}

/// 64×48 Skirmish Default development map with 180° symmetry: its west half is built here and the east half
/// generated from it ([`MapDef::regenerate`]), so it is symmetric by construction. The two slots start in the
/// top-left and bottom-right corners, each with an 8-patch crystal field (`factions/gdo/economy.md > Reference
/// starting base`) between the start and the map edge, a GDO Starting Layout (Deployment Center, Extraction
/// Facility and two plates) whose Deployment Center sees and reaches the whole field, a Colonists one (Egress
/// Node, Robo Portal and four Drones) whose Egress Node stands within the field's arc, and a Syndicate one (Tunnel,
/// Hideout and Agent) whose Tunnel's crystal hatch faces the field. Terrain (`core/map.md > Elevation`): a lake and a
/// mountain
/// massif on each side, rugged ground on the flanks, a gentle hill of slopes only on each side, and a central
/// plateau 3 levels up ringed by cliffs, reached only by a slope through a 2-tile gap in the west and in the east
/// (the chokepoints). A Supply Delivery Station stands on the plateau, in the middle of the map.
pub fn dev_skirmish_map() -> MapDef {
    let (w, h) = (64, 48);
    let mut map = Map::new(w, h);
    let mut fill = |x0: i32, y0: i32, x1: i32, y1: i32, preset: TilePresetId| {
        for y in y0..=y1 {
            for x in x0..=x1 {
                map.set_tile(TilePos::new(x, y), Tile::new(preset));
            }
        }
    };
    // A lake north of the middle and a mountain south of it, between the bases.
    fill(22, 6, 29, 13, TilePresetId::WATER);
    fill(18, 34, 23, 42, TilePresetId::MOUNTAIN);
    // Rugged ground on the flank.
    fill(4, 30, 12, 38, TilePresetId::RUGGED);
    // The central plateau: a ring of cliffs around Plane ground, with a 2-tile gap in the west (and, as its image,
    // in the east).
    fill(26, 18, 37, 29, TilePresetId::CLIFF);
    fill(27, 19, 36, 28, TilePresetId::PLANE);
    fill(26, 22, 26, 23, TilePresetId::PLANE);

    // The mountain rises 2 levels per corner from its edge, whose corners stay at 0 for the ground around it.
    for y in 35..=42 {
        for x in 19..=23 {
            let from_edge = (x - 18).min(24 - x).min(y - 34).min(43 - y);
            map.set_corner(CornerPos::new(x, y), 2 * from_edge as u8);
        }
    }
    // A gentle hill, 3 levels high, one level per 2 tiles: a corner is at least level k if it lies within 7 - 2k
    // tiles of the top (integer arithmetic on squared distances).
    let (cx, cy) = (14, 22);
    for y in cy - 6..=cy + 6 {
        for x in cx - 6..=cx + 6 {
            let d2 = (x - cx) * (x - cx) + (y - cy) * (y - cy);
            let level = (1..=3).filter(|&k| d2 <= (7 - 2 * k) * (7 - 2 * k)).count();
            map.set_corner(CornerPos::new(x, y), level as u8);
        }
    }

    let mut spawns = Vec::new();
    // The first slot's field: an arc up and left of the start.
    for (x, y) in [(2, 5), (2, 6), (2, 7), (3, 3), (4, 2), (5, 2), (6, 2), (7, 2)] {
        spawns.push(Spawn::new(
            ObjectKind::CrystalPatch,
            None,
            SpawnAt::Tile(TilePos::new(x, y)),
        ));
    }
    // Its 2×2 footprint is centered on the map: it is its own image.
    spawns.push(Spawn::new(
        ObjectKind::SupplyDeliveryStation,
        None,
        SpawnAt::Tile(TilePos::new(31, 23)),
    ));
    // Starts in opposite corners (design change 0007). The first slot's GDO Starting Layout (`factions/gdo/
    // economy.md > Starting Conditions`; the second slot's is its image): the Deployment Center on (5, 5) to (8, 8),
    // inside the field's arc, so that its sight of 6 grid units covers every patch of the field (design change 0095:
    // their centres lie 4.5 to 5.2 grid units from its centre, all at level 0) and its build area holds all eight
    // (`factions/gdo/economy.md > Opening Economy Baseline > Reference starting base`); the Extraction Facility on
    // (9, 1) to (11, 3), past the east end of the field's northern row; the two plates on that row's eastern patches,
    // (6, 2) and (7, 2). No camera start: the camera starts on the Deployment Center (`core/camera.md > Starting
    // Position`).
    let start = TilePos::new(8, 8);
    let mut slot = Slot::new(start);
    let laid = |kind, x, y| LayoutObject {
        kind,
        at: SpawnAt::Tile(TilePos::new(x, y)),
        orientation: BuildingOrientation::default(),
    };
    *slot.layouts.get_mut(Faction::Gdo) = vec![
        laid(ObjectKind::DeploymentCenter, 5, 5),
        laid(ObjectKind::ExtractionFacility, 9, 1),
        laid(ObjectKind::ExtractionPlate, 6, 2),
        laid(ObjectKind::ExtractionPlate, 7, 2),
    ];
    // The first slot's Colonists Starting Layout (`factions/colonists/economy.md > Starting Conditions`; slice M3-C,
    // the second slot's is its image): the Egress Node on (4, 4) to (5, 5), inside the field's arc, its centre 2.1 to
    // 3.5 grid units from those of the eight patches; the Robo Portal on (6, 6) to (9, 9), south-east of the node; the
    // four Drones south-west of the node, centred 1 grid unit apart around (4, 9.5) grid units, clear of the node's
    // sides (2.7 grid units below its south side) and of every side of the portal, the doors it will exit by (1.2
    // west of its west side), so that the gatherers set off from there and the portal's Drones come out. They stand
    // on the unit-start rule's open ground (`docs/movement.md > 3`). No camera start: the camera starts on the Robo
    // Portal (`core/camera.md > Starting Position`).
    let drone = |x: f32, y: f32| LayoutObject {
        kind: ObjectKind::Drone,
        at: SpawnAt::Point(glam::Vec2::new(x, y) * SU_PER_GU),
        orientation: BuildingOrientation::default(),
    };
    *slot.layouts.get_mut(Faction::Colonists) = vec![
        laid(ObjectKind::EgressNode, 4, 4),
        laid(ObjectKind::RoboPortal, 6, 6),
        drone(3.5, 9.0),
        drone(4.5, 9.0),
        drone(3.5, 10.0),
        drone(4.5, 10.0),
    ];
    // The first slot's Syndicate Starting Layout (`factions/syndicate/tunnels.md > Tunnel > Starting Conditions`;
    // slice M4-B, the second slot's is its image). The Tunnel on (4, 4) to (7, 7), turned 180° so that its sides
    // (design change 0248: "A: entrance/exit; B: crystal hatch; C: back wall (no function); D: Supplies hatch") face A
    // south, toward the open map, B west, C north and D east, toward the middle of the map and its station. The crystal
    // hatch, (4, 5) (`tunnel::hatch_square`, slice M4-D: Side B's middle square farther from Side A; design change 0255
    // point 6: "The hatch is 1 grid square, near the middle of the side"), faces the patch (2, 5) across the empty
    // column x = 3, 1 grid unit edge to edge (0259: the patch "1 GU from the hatch"). That close, 3 or more Agents jam
    // at the hatch (question Q0076): the layout keeps 0259's 1 GU until the design decides. Its Tier 1 Tunnel Area,
    // (2, 2) to (9, 9), is all on the map and flat Plane (design change 0242: on the map and Drillable). The Hideout
    // on (8, 8) to (9, 9), in that area, off the Tunnel and the patches; the Agent 1.5 grid units south of Side A's
    // middle, at (6, 9.5) grid units, clear of the doorway the Tunnel's units use (slice M4-C). No camera start: the
    // camera starts on the Tunnel (`core/camera.md > Starting Position`: "Syndicate: starting Tunnel").
    let turned = BuildingOrientation {
        rotation: Rotation::R180,
        ..BuildingOrientation::default()
    };
    *slot.layouts.get_mut(Faction::Syndicate) = vec![
        LayoutObject {
            orientation: turned,
            ..laid(ObjectKind::Tunnel, 4, 4)
        },
        laid(ObjectKind::Hideout, 8, 8),
        LayoutObject {
            kind: ObjectKind::Agent,
            at: SpawnAt::Point(glam::Vec2::new(6.0, 9.5) * SU_PER_GU),
            orientation: BuildingOrientation::default(),
        },
    ];

    let mut dev = MapDef {
        map_type: MapType::SkirmishDefault,
        name: "dev_skirmish".into(),
        description: "Development map: two bases in opposite corners, a central plateau with two ramps.".into(),
        terrain: map,
        symmetry: Symmetry::Rotate180,
        slots: vec![slot.clone(), slot],
        spawns,
    };
    dev.regenerate();
    // Raising the plateau's top lets the slopes form in the gaps on their own; the cliffs take the rest. The target
    // corners are their own set of images, so the slopes come out symmetric.
    let top = (27..=37).flat_map(|x| (19..=29).map(move |y| (CornerPos::new(x, y), 3)));
    dev.terrain.set_corner_heights(&top.collect());
    dev.regenerate();
    dev
}
