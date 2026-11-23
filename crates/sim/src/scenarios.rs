//! Built-in maps for development, QA and tests, each with its default game setup ([`Scenario::new`]: a GDO player in
//! every slot).

use crate::defs::ObjectKind;
use crate::map::{Map, Tile, TilePresetId};
use crate::scenario::{MapDef, MapType, Scenario, Slot, Spawn, SpawnAt};
use crate::symmetry::Symmetry;
use crate::units::{CornerPos, TilePos};

/// Names accepted by [`by_name`] (e.g. for the client's `--scenario` flag).
pub const NAMES: &[&str] = &["dev_skirmish", "empty"];

pub fn by_name(name: &str) -> Option<Scenario> {
    match name {
        "dev_skirmish" => Some(dev_skirmish()),
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

/// [`dev_skirmish_map`] with its default setup.
pub fn dev_skirmish() -> Scenario {
    Scenario::new(dev_skirmish_map())
}

/// 64×48 Skirmish Default development map with 180° symmetry: its west half is built here and the east half
/// generated from it ([`MapDef::regenerate`]), so it is symmetric by construction. The two slots start in the
/// top-left and bottom-right corners, each with an 8-patch crystal field (`factions/gdo/economy.md > Reference
/// starting base`) between the start and the map edge. Terrain (`core/map.md > Elevation`): a lake and a mountain
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
    // Starts in opposite corners (design change 0007).
    let start = TilePos::new(8, 8);
    let mut slot = Slot::new(start);
    slot.camera_start = Some(start.center());

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
