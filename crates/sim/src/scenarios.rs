//! Built-in scenarios for development, QA and tests.

use crate::defs::ObjectKind;
use crate::ids::PlayerId;
use crate::map::{Map, Tile, TilePresetId};
use crate::player::Faction;
use crate::scenario::{PlayerSetup, Scenario, Spawn, SpawnAt};
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

/// A flat, featureless map with two players and no objects. The start locations (Q0004
/// placeholder) sit a quarter of the way in from the top-left and bottom-right corners.
pub fn empty(width: u32, height: u32) -> Scenario {
    let map = Map::new(width, height); // Panics first if the size is 0.
    let mut players = two_players();
    let (x, y) = (width / 4, height / 4);
    players[0].start_location = Some(TilePos::new(x as i32, y as i32));
    players[1].start_location = Some(TilePos::new((width - 1 - x) as i32, (height - 1 - y) as i32));
    Scenario {
        name: "empty".into(),
        map,
        players,
        spawns: Vec::new(),
        seed: 1,
    }
}

/// 64×48 development map, point-symmetric around its center apart from the lake and the mountain. The start
/// locations sit in the top-left and bottom-right corners, each with an 8-patch crystal field
/// (`factions/gdo/economy.md > Reference starting base`) between it and the map edge. Terrain
/// (`core/map.md > Elevation`): a lake, a mountain massif, rugged ground on the flanks, a gentle hill of slopes
/// only on each side, and a central plateau 3 levels up ringed by cliffs, reached only by a slope through a
/// 2-tile gap in the west and in the east (the chokepoints). A Supply Delivery Station stands on the plateau, in
/// the middle of the map.
pub fn dev_skirmish() -> Scenario {
    let (w, h) = (64, 48);
    let mut map = Map::new(w, h);
    let mut fill = |x0: i32, y0: i32, x1: i32, y1: i32, preset: TilePresetId| {
        for y in y0..=y1 {
            for x in x0..=x1 {
                map.set_tile(TilePos::new(x, y), Tile::new(preset));
            }
        }
    };
    // Lake and mountain between the bases.
    fill(22, 6, 29, 13, TilePresetId::WATER);
    fill(36, 33, 41, 41, TilePresetId::MOUNTAIN);
    // Rugged ground on the flanks.
    fill(4, 30, 12, 38, TilePresetId::RUGGED);
    fill(50, 8, 58, 15, TilePresetId::RUGGED);
    // The central plateau: a ring of cliffs around Plane ground, with a 2-tile gap in the west and in the east.
    fill(26, 18, 37, 29, TilePresetId::CLIFF);
    fill(27, 19, 36, 28, TilePresetId::PLANE);
    fill(26, 22, 26, 23, TilePresetId::PLANE);
    fill(37, 24, 37, 25, TilePresetId::PLANE);

    // The mountain rises 2 levels per corner from its edge, whose corners stay at 0 for the ground around it.
    for y in 34..=41 {
        for x in 37..=41 {
            let from_edge = (x - 36).min(42 - x).min(y - 33).min(42 - y);
            map.set_corner(CornerPos::new(x, y), 2 * from_edge as u8);
        }
    }
    // Two gentle hills, 3 levels high, one level per 2 tiles: a corner is at least level k if it lies within
    // 7 - 2k tiles of the top (integer arithmetic on squared distances).
    for (cx, cy) in [(14, 22), (50, 26)] {
        for y in cy - 6..=cy + 6 {
            for x in cx - 6..=cx + 6 {
                let d2 = (x - cx) * (x - cx) + (y - cy) * (y - cy);
                let level = (1..=3).filter(|&k| d2 <= (7 - 2 * k) * (7 - 2 * k)).count();
                map.set_corner(CornerPos::new(x, y), level as u8);
            }
        }
    }
    // Raising the plateau's top lets the slopes form in the gaps on their own; the cliffs take the rest.
    let top = (27..=37).flat_map(|x| (19..=29).map(move |y| (CornerPos::new(x, y), 3)));
    map.set_corner_heights(&top.collect());

    let mut spawns = Vec::new();
    let mut patches = |tiles: &[(i32, i32)]| {
        for &(x, y) in tiles {
            spawns.push(Spawn::new(
                ObjectKind::CrystalPatch,
                None,
                SpawnAt::Tile(TilePos::new(x, y)),
            ));
        }
    };
    // Player 1's field: an arc up and left of the start.
    patches(&[(2, 5), (2, 6), (2, 7), (3, 3), (4, 2), (5, 2), (6, 2), (7, 2)]);
    // Player 2's field: the same, mirrored through the map center.
    patches(&[
        (61, 42),
        (61, 41),
        (61, 40),
        (60, 44),
        (59, 45),
        (58, 45),
        (57, 45),
        (56, 45),
    ]);
    // Its 2×2 footprint is centered on the map.
    spawns.push(Spawn::new(
        ObjectKind::SupplyDeliveryStation,
        None,
        SpawnAt::Tile(TilePos::new(31, 23)),
    ));

    let mut players = two_players();
    // Start locations (Q0004 placeholder) in opposite corners (design change 0007), mirrored through the center.
    let starts = [TilePos::new(8, 8), TilePos::new(55, 39)];
    for (p, start) in players.iter_mut().zip(starts) {
        p.start_location = Some(start);
        p.camera_start = Some(start.center());
    }
    Scenario {
        name: "dev_skirmish".into(),
        map,
        players,
        spawns,
        seed: 1,
    }
}

fn two_players() -> Vec<PlayerSetup> {
    vec![
        PlayerSetup {
            id: PlayerId(1),
            name: "Player 1".into(),
            faction: Faction::Gdo,
            camera_start: None,
            start_location: None,
        },
        PlayerSetup {
            id: PlayerId(2),
            name: "Player 2".into(),
            faction: Faction::Gdo,
            camera_start: None,
            start_location: None,
        },
    ]
}
