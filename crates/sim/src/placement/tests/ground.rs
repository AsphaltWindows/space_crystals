//! The tile rules of the ground and of sight (rules 1 to 4): tiles off the map, tiles that are not Buildable,
//! sloped tiles, and tiles out of the placing player's sight.

use super::*;

/// Rule 1: the tiles off the map, each listed; nothing else is checked on them.
#[test]
fn tiles_off_the_map() {
    let (map, area) = open(8, 8);
    let check = place(&map, &area, &[], Generator, 7, 3);
    assert_eq!(check.tiles, [(t(8, 3), OffMap), (t(8, 4), OffMap)]);
    assert!(check.tile_is_valid(t(7, 3)) && !check.is_valid());
    let check = place(&map, &area, &[], Generator, -1, -1);
    assert_eq!(
        check.tiles,
        [(t(-1, -1), OffMap), (t(0, -1), OffMap), (t(-1, 0), OffMap)]
    );
    // A Barracks turned a quarter is 2 wide and 3 tall.
    let turned = BuildingOrientation {
        rotation: Rotation::R90,
        ..Default::default()
    };
    let site = Site {
        map: &map,
        area: &area,
        mode: Mode::Layout,
        method: Method::Direct,
    };
    assert!(check_at(&site, Barracks, t(6, 5), turned).is_valid());
    assert_eq!(
        check_at(&site, Barracks, t(6, 6), turned).tiles,
        [(t(6, 8), OffMap), (t(7, 8), OffMap)]
    );
    // No origin overflows: what can be named is off the map, and that is enough.
    let check = place(&map, &area, &[], DeploymentCenter, i32::MAX - 1, i32::MAX - 1);
    let named = [
        t(i32::MAX - 1, i32::MAX - 1),
        t(i32::MAX, i32::MAX - 1),
        t(i32::MAX - 1, i32::MAX),
        t(i32::MAX, i32::MAX),
    ];
    assert_eq!(check.tiles, named.map(|tile| (tile, OffMap)));
    for (x, y) in [(i32::MIN, i32::MIN), (i32::MAX, 0), (0, i32::MIN)] {
        let check = place(&map, &area, &[on(Generator, 0, 0)], ExtractionPlate, x, y);
        assert_eq!(check.tiles, [(t(x, y), OffMap)], "({x}, {y})");
        assert_eq!(check.whole, [OutsideBuildArea, NotOnPatch]);
    }
}

fn check_at(site: &Site<'_>, kind: ObjectKind, origin: TilePos, orientation: BuildingOrientation) -> PlacementCheck {
    check(site, [], kind, origin, orientation)
}

/// Rule 2: only Buildable presets (`core/map.md > DefaultTilePresets`: Plane alone of the defaults), by their flag,
/// not their look.
#[test]
fn tiles_that_are_not_buildable() {
    let mut map = Map::from_ascii(
        "
        .r..
        .c..
        .M~.
        ....",
    )
    .unwrap();
    let area = dc_area(&map, 0, 0);
    let check = place(&map, &area, &[], DeploymentCenter, 0, 0);
    let unbuildable = [t(1, 0), t(1, 1), t(1, 2), t(2, 2)];
    assert_eq!(check.tiles, unbuildable.map(|tile| (tile, NotBuildable)));
    let custom = |buildable, look| TilePreset {
        name: "Custom".into(),
        buildable,
        traversible: true,
        rugged: false,
        drillable: true,
        recruitable: true,
        look,
    };
    let looks_rugged = map.add_preset(custom(true, TileLook::Rugged));
    let looks_plane = map.add_preset(custom(false, TileLook::Plane));
    map.set_tile(t(3, 0), Tile::new(looks_rugged));
    map.set_tile(t(3, 1), Tile::new(looks_plane));
    assert!(
        place(&map, &area, &[], Generator, 2, 0)
            .problems_at(t(3, 0))
            .next()
            .is_none()
    );
    assert_eq!(
        place(&map, &area, &[], Generator, 2, 0)
            .problems_at(t(3, 1))
            .collect::<Vec<_>>(),
        [NotBuildable]
    );
}

/// Rule 3 (design change 0004's verification): a footprint with one tile on a slope has that tile sloped; on a
/// flat plateau it is valid.
#[test]
fn sloped_tiles() {
    // A plateau at level 2 in the east half, a slope of one level down to level 1 in the west.
    let map = Map::from_ascii(
        "
        ......
        ......
        ......

        1122222
        1122222
        1122222
        1122222",
    )
    .unwrap();
    let area = dc_area(&map, 0, 0);
    assert_eq!(
        place(&map, &area, &[], Generator, 1, 0).tiles,
        [(t(1, 0), NotFlat), (t(1, 1), NotFlat)]
    );
    assert!(place(&map, &area, &[], Generator, 2, 0).is_valid(), "on the plateau");
    assert!(place(&map, &area, &[], Generator, 4, 1).is_valid(), "on the plateau");
    assert!(place(&map, &area, &[], ExtractionFacility, 3, 0).is_valid());
    assert_eq!(
        place(&map, &area, &[], ExtractionFacility, 0, 0).tiles.len(),
        3,
        "the slope's column"
    );
}

/// Rule 4 (design change 0095): every tile Visible to the placing player in a game, as their sight is now; no sight
/// is needed for a Starting Layout.
#[test]
fn tiles_out_of_sight() {
    const P1: PlayerId = PlayerId(1);
    const P2: PlayerId = PlayerId(2);
    let unturned = BuildingOrientation::default();
    let mut rows = vec![".".repeat(24); 16];
    rows[12].replace_range(20..21, "r");
    let scenario = ScenarioBuilder::new(&rows.join("\n"))
        .spawn(Generator, Some(P1), crate::SpawnAt::Tile(t(4, 4)))
        .spawn(Barracks, Some(P2), crate::SpawnAt::Tile(t(14, 10)))
        .build();
    let sim = Sim::new(&scenario);
    let area = dc_area(sim.map(), 0, 0);
    let in_game = |player| Site {
        map: sim.map(),
        area: &area,
        mode: Mode::Command {
            vision: sim.vision(),
            player,
        },
        method: Method::Direct,
    };
    // An Extraction Facility at (6, 3) is partly in the Generator's sight of 3.
    let check = check_at(&in_game(P1), ExtractionFacility, t(6, 3), unturned);
    let unseen: Vec<TilePos> = (3..6)
        .flat_map(|y| (6..9).map(move |x| t(x, y)))
        .filter(|&tile| sim.visibility(P1, tile) != VisibilityState::Visible)
        .collect();
    assert!((1..9).contains(&unseen.len()), "partly in sight: {unseen:?}");
    assert_eq!(
        check.tiles,
        unseen.iter().map(|&tile| (tile, NotVisible)).collect::<Vec<_>>()
    );
    let layout = Site {
        mode: Mode::Layout,
        ..in_game(P1)
    };
    assert!(check_at(&layout, ExtractionFacility, t(6, 3), unturned).is_valid());
    // A player who isn't in the game sees nothing.
    let check = check_at(&in_game(PlayerId(9)), Generator, t(4, 7), unturned);
    assert_eq!(
        check.tiles,
        [t(4, 7), t(5, 7), t(4, 8), t(5, 8)].map(|tile| (tile, NotVisible))
    );
    // An unseen tile tells nothing else: neither player 2's Barracks nor the rugged tile at (20, 12) shows to player
    // 1, who sees neither; player 2 sees its own Barracks.
    let among_objects =
        |site: &Site<'_>, origin| super::check(site, sim.objects().map(Occupant::of), Generator, origin, unturned);
    let barracks = [t(14, 10), t(15, 10), t(14, 11), t(15, 11)];
    let check = among_objects(&in_game(P1), t(14, 10));
    assert_eq!(check.tiles, barracks.map(|tile| (tile, NotVisible)));
    let check = among_objects(&in_game(P2), t(14, 10));
    assert_eq!(check.tiles, barracks.map(|tile| (tile, Structure)));
    let check = among_objects(&layout, t(14, 10));
    assert_eq!(check.tiles, barracks.map(|tile| (tile, Structure)));
    let check = check_at(&in_game(P1), Generator, t(20, 12), unturned);
    assert!(check.tiles.iter().all(|&(_, problem)| problem == NotVisible));
    let check = check_at(&layout, Generator, t(20, 12), unturned);
    assert_eq!(check.tiles, [(t(20, 12), NotBuildable)]);
}
