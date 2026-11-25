//! The Tunnel Area's rules (slice M4-B, `placement/tunnel_area.rs`): a Tunnel's whole Tier 1 area on the map and
//! Drillable (design change 0242), in every mode, `Mode::Known` judging the ground by what the player explored; and a
//! Starting Layout's underground structure wholly inside the area of a Tunnel of its layout.

use super::*;
use crate::map::TilePresetId;
use crate::scenario::SpawnAt;

use ObjectKind::{Hideout, RoboPortal, Tunnel};
use WholeProblem::{OutsideTunnelArea, TunnelAreaNotDrillable, TunnelAreaOffMap};

/// A 16 × 12 map of Plane but for one column of the default preset `symbol` at x = 2.
fn column(symbol: char) -> Map {
    let row: String = (0..16).map(|x| if x == 2 { symbol } else { '.' }).collect();
    Map::from_ascii(&vec![row; 12].join("\n")).unwrap()
}

/// [`check`] of a Tunnel at `(x, y)` on `sim`'s map in `mode`: the problems as a whole.
fn tunnel_in(sim: &Sim, mode: Mode<'_>, x: i32, y: i32) -> Vec<WholeProblem> {
    let area = BuildArea::default();
    let site = Site {
        map: sim.map(),
        area: &area,
        mode,
        method: Method::WorkerBuilt,
    };
    check(&site, [], Tunnel, t(x, y), BuildingOrientation::default()).whole
}

/// What player 1 knows in `sim` ([`Mode::Known`]).
fn known(sim: &Sim) -> Mode<'_> {
    Mode::Known {
        vision: sim.vision(),
        player: PlayerId(1),
    }
}

/// `factions/syndicate/tunnels.md > Tunnel Area Non-Overlap Rule > Drillable area` (design change 0242): "a Tunnel can
/// only be placed if every cell of its Tunnel Area is on the map and Drillable (no Water or Mountain ...)". A Tunnel
/// at (4, 6) has its 8x8 Tier 1 area from x = 2: a column of Water or Mountain there refuses it as a whole, its own
/// tiles being fine; Rugged ground or a Cliff there doesn't, being Drillable (`core/map.md > DefaultTilePresets`). A
/// tile farther east, at (5, 6), the area misses the column.
#[test]
fn a_tunnels_area_must_be_drillable() {
    let area = BuildArea::default();
    for symbol in ['~', 'M'] {
        let map = column(symbol);
        let check = place(&map, &area, &[], Tunnel, 4, 6);
        assert_eq!(
            (check.tiles, check.whole),
            (vec![], vec![TunnelAreaNotDrillable]),
            "{symbol}"
        );
        assert!(place(&map, &area, &[], Tunnel, 5, 6).is_valid(), "{symbol}");
        // Only a kind with a Tunnel Area has the rule: a Robo Portal or a Hideout by the lake is fine.
        assert!(place(&map, &area, &[], RoboPortal, 3, 6).is_valid(), "{symbol}");
        assert!(place(&map, &area, &[], Hideout, 3, 6).is_valid(), "{symbol}");
    }
    for symbol in ['r', 'c'] {
        assert!(place(&column(symbol), &area, &[], Tunnel, 4, 6).is_valid(), "{symbol}");
    }
}

/// `core/map.md > Tile`: "Any combination of properties is valid": Drillable is the preset's flag, not its look. A
/// Plane-looking preset that isn't Drillable, under one cell of the area, refuses the Tunnel.
#[test]
fn a_custom_undrillable_preset_refuses_a_tunnel() {
    let mut map = Map::new(16, 16);
    let plane = map.preset(TilePresetId::PLANE).clone();
    let hard = map.add_preset(TilePreset {
        name: "Hard plane".into(),
        drillable: false,
        ..plane
    });
    map.set_tile(t(9, 9), Tile::new(hard));
    let area = BuildArea::default();
    assert_eq!(place(&map, &area, &[], Tunnel, 4, 4).whole, [TunnelAreaNotDrillable]);
    assert!(
        place(&map, &area, &[], Tunnel, 3, 3).is_valid(),
        "the area ends at (8, 8)"
    );
}

/// 0242: "So no Tunnel near the map edge (closer than its area reaches)". 1 tile from an edge, a row or a column of the
/// area lies off the map; 2 tiles from it, the area just reaches the edge and the Tunnel is valid (design change 0283:
/// "a footprint 2 tiles from the edge is valid, and 1 tile is red"). With both problems, they come in the order of
/// `WholeProblem`.
#[test]
fn a_tunnels_area_must_be_on_the_map() {
    let map = Map::new(16, 12);
    let area = BuildArea::default();
    let tunnel = |x, y| place(&map, &area, &[], Tunnel, x, y);
    for (x, y) in [(1, 4), (4, 1), (11, 4), (4, 7), (0, 0)] {
        let check = tunnel(x, y);
        assert_eq!(
            (check.tiles, check.whole),
            (vec![], vec![TunnelAreaOffMap]),
            "({x}, {y})"
        );
    }
    for (x, y) in [(2, 2), (10, 6), (2, 6), (10, 2)] {
        assert!(tunnel(x, y).is_valid(), "2 tiles from the edges: ({x}, {y})");
    }
    assert_eq!(
        place(&column('~'), &area, &[], Tunnel, 4, 1).whole,
        [TunnelAreaOffMap, TunnelAreaNotDrillable]
    );
    // Past the edge, the footprint's own tiles are off the map as well.
    let off = tunnel(14, 4);
    assert_eq!(off.whole, [TunnelAreaOffMap]);
    assert!(!off.tiles.is_empty() && off.tiles.iter().all(|&(_, p)| p == OffMap));
}

/// 0242's ghost: "terrain is always known to the player as far as explored; never-explored tiles pass on the ghost,
/// and the real check happens on arrival". In `Mode::Known`, the lake at x = 3 that player 1 never explored passes,
/// and refuses the Tunnel once a Guard of theirs has seen it; a cell off the map always fails, the map's edge being
/// known. On arrival (`Mode::Arrival`) and in a command (`Mode::Command`), the truth.
#[test]
fn the_ghost_judges_the_area_by_what_the_player_explored() {
    let ascii = vec!["...~....................".to_string(); 16].join("\n");
    let unexplored = Sim::new(&ScenarioBuilder::new(&ascii).build());
    let scout = SpawnAt::Point(t(5, 8).center());
    let explored = Sim::new(
        &ScenarioBuilder::new(&ascii)
            .spawn(Guard, Some(PlayerId(1)), scout)
            .build(),
    );
    assert_eq!(unexplored.visibility(PlayerId(1), t(3, 8)), VisibilityState::Unexplored);
    assert_ne!(explored.visibility(PlayerId(1), t(3, 8)), VisibilityState::Unexplored);
    assert_eq!(tunnel_in(&unexplored, known(&unexplored), 5, 6), []);
    assert_eq!(tunnel_in(&explored, known(&explored), 5, 6), [TunnelAreaNotDrillable]);
    assert_eq!(tunnel_in(&unexplored, known(&unexplored), 21, 6), [TunnelAreaOffMap]);
    assert_eq!(tunnel_in(&unexplored, Mode::Arrival, 5, 6), [TunnelAreaNotDrillable]);
    let command = Mode::Command {
        vision: unexplored.vision(),
        player: PlayerId(1),
    };
    assert_eq!(tunnel_in(&unexplored, command, 5, 6), [TunnelAreaNotDrillable]);
}

/// The layout rule for expansions (`factions/syndicate/tunnels.md > Tunnel > Starting Conditions`: "One pre-built
/// Tier 1 Tunnel containing one pre-built Hideout"): a layout's Hideout lies wholly inside the Tier 1 Tunnel Area of
/// a Tunnel of its layout, here (2, 2) to (9, 9). Outside it, or half inside, it is refused as a whole
/// (`OutsideTunnelArea`); on the Tunnel it is under a structure, and on a patch under a resource, as any building.
/// Without a Tunnel in the layout it is outside: a map's Tunnel doesn't count. Its tiles follow the expansion method
/// (slice M4-F): Rugged ground is Drillable (design change 0239), though not Buildable.
#[test]
fn a_layout_hideout_lies_inside_its_tunnels_area() {
    let mut rows = vec![".".repeat(16); 16];
    rows[9].replace_range(2..3, "r");
    let map = Map::from_ascii(&rows.join("\n")).unwrap();
    let patch = [on(CrystalPatch, 8, 2)];
    let hideout = |x, y| {
        let layout = [on(Tunnel, 4, 4), on(Hideout, x, y)];
        check_layout(&map, patch, &layout, 1).expect("a structure")
    };
    let problems = |x, y| {
        let check = hideout(x, y);
        (check.tiles, check.whole)
    };
    assert!(hideout(8, 8).is_valid() && hideout(2, 2).is_valid() && hideout(8, 4).is_valid());
    assert_eq!(problems(10, 4), (vec![], vec![OutsideTunnelArea]), "outside");
    assert_eq!(problems(9, 4), (vec![], vec![OutsideTunnelArea]), "half inside");
    assert_eq!(problems(1, 5), (vec![], vec![OutsideTunnelArea]), "half inside, west");
    assert_eq!(
        problems(7, 7),
        (vec![(t(7, 7), Structure)], vec![]),
        "on the Tunnel's corner"
    );
    assert_eq!(problems(8, 1).1, [OutsideTunnelArea]);
    assert_eq!(problems(8, 2), (vec![(t(8, 2), Resource)], vec![]), "on a patch");
    assert!(hideout(2, 8).is_valid(), "Rugged");
    let alone = [on(Hideout, 8, 8)];
    assert_eq!(check_layout(&map, [], &alone, 0).unwrap().whole, [OutsideTunnelArea]);
    let map_tunnel = [on(Tunnel, 4, 4)];
    assert_eq!(
        check_layout(&map, map_tunnel, &alone, 0).unwrap().whole,
        [OutsideTunnelArea]
    );
    // The Tunnel itself has no such rule, nor any other kind.
    let layout = [on(Tunnel, 4, 4), on(RoboPortal, 11, 11)];
    assert!(check_layout(&map, [], &layout, 0).unwrap().is_valid());
    assert!(check_layout(&map, [], &layout, 1).unwrap().is_valid());
}
