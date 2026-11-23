//! Built-in scenarios for development, QA and tests.

use crate::defs::ObjectKind;
use crate::ids::PlayerId;
use crate::map::{Map, Tile, TilePresetId};
use crate::player::Faction;
use crate::scenario::{PlayerSetup, Scenario, Spawn, SpawnAt};
use crate::units::TilePos;

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

/// 64×48 development map: two start areas (top-left, bottom-right) with an 8-patch
/// crystal field each (`factions/gdo/economy.md > Reference starting base`), a lake,
/// a mountain ridge, rugged ground, a raised plateau ringed by cliffs with two ramps,
/// and a Supply Delivery Station in the middle.
pub fn dev_skirmish() -> Scenario {
    let (w, h) = (64, 48);
    let mut map = Map::new(w, h);
    let mut fill = |x0: i32, y0: i32, x1: i32, y1: i32, tile: Tile| {
        for y in y0..=y1 {
            for x in x0..=x1 {
                let p = TilePos::new(x, y);
                if map.in_bounds(p) {
                    map.set_tile(p, tile);
                }
            }
        }
    };
    let plane = |e| Tile::new(TilePresetId::PLANE, e);
    // Lake and mountain ridge between the bases.
    fill(22, 6, 29, 13, Tile::new(TilePresetId::WATER, 0));
    fill(36, 33, 41, 41, Tile::new(TilePresetId::MOUNTAIN, 0));
    // Rugged ground on the flanks.
    fill(4, 30, 12, 38, Tile::new(TilePresetId::RUGGED, 0));
    fill(50, 8, 58, 15, Tile::new(TilePresetId::RUGGED, 0));
    // Central plateau (elevation 2) ringed by cliffs, with ramps (elevation 1) west and east.
    fill(26, 18, 37, 29, Tile::new(TilePresetId::CLIFF, 1));
    fill(27, 19, 36, 28, plane(2));
    fill(26, 23, 26, 24, plane(1));
    fill(37, 23, 37, 24, plane(1));
    // Lower-right corner sits one level up, so the two bases differ in elevation.
    fill(44, 30, 63, 47, plane(1));
    fill(44, 30, 44, 47, Tile::new(TilePresetId::CLIFF, 1));
    fill(44, 38, 44, 39, plane(1));

    let mut spawns = Vec::new();
    let mut patches = |tiles: &[(i32, i32)]| {
        for &(x, y) in tiles {
            spawns.push(Spawn {
                kind: ObjectKind::CrystalPatch,
                owner: None,
                at: SpawnAt::Tile(TilePos::new(x, y)),
            });
        }
    };
    // Player 1 field: an arc above-left of the base area around (10, 10).
    patches(&[(3, 6), (3, 7), (3, 8), (4, 4), (5, 3), (6, 3), (7, 3), (8, 3)]);
    // Player 2 field: mirrored around the map center.
    patches(&[
        (60, 41),
        (60, 40),
        (60, 39),
        (59, 43),
        (58, 44),
        (57, 44),
        (56, 44),
        (55, 44),
    ]);
    spawns.push(Spawn {
        kind: ObjectKind::SupplyDeliveryStation,
        owner: None,
        at: SpawnAt::Tile(TilePos::new(31, 23)),
    });

    let mut players = two_players();
    // Start locations (Q0004 placeholder): the base areas, mirrored around the map center.
    let starts = [TilePos::new(10, 10), TilePos::new(53, 37)];
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
