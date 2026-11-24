//! One test per placement rule (the `placement` module docs list them), on the pure check: hand-made maps, areas and
//! occupants, in Layout mode unless the test is about sight. `crates/sim/tests/placement.rs` checks them in games.

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
    }
}

/// A unit centred on `(x, y)`, space units.
fn unit(kind: ObjectKind, x: f32, y: f32) -> Occupant {
    Occupant {
        kind,
        pos: Vec2::new(x, y),
        placement: None,
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

#[test]
fn a_footprint_on_open_ground_is_valid() {
    let (map, area) = open(16, 16);
    for kind in [DeploymentCenter, Generator, Barracks, ExtractionFacility] {
        assert_eq!(
            place(&map, &area, &[], kind, 4, 4),
            PlacementCheck::default(),
            "{kind:?}"
        );
    }
    assert!(place(&map, &area, &[], Generator, 0, 14).is_valid(), "in the corner");
}

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

/// Rule 5: no tile under another structure, whoever owns it.
#[test]
fn tiles_under_structures() {
    let (map, area) = open(16, 16);
    let check = place(&map, &area, &[on(Generator, 5, 5)], Generator, 6, 6);
    assert_eq!(check.tiles, [(t(6, 6), Structure)]);
    let check = place(&map, &area, &[on(DeploymentCenter, 2, 2)], Barracks, 4, 5);
    assert_eq!(check.tiles, [(t(4, 5), Structure), (t(5, 5), Structure)]);
    assert!(
        place(&map, &area, &[on(Generator, 5, 5)], Generator, 7, 5).is_valid(),
        "side by side"
    );
}

/// Rule 6 (design change 0096): no tile under a map resource, but for a plate on a patch.
#[test]
fn tiles_under_resources() {
    let (map, area) = open(16, 16);
    let resources = [on(CrystalPatch, 3, 3), on(SupplyDeliveryStation, 8, 8)];
    assert_eq!(
        place(&map, &area, &resources, Generator, 2, 2).tiles,
        [(t(3, 3), Resource)]
    );
    assert_eq!(
        place(&map, &area, &resources, Barracks, 6, 9).tiles,
        [(t(8, 9), Resource)],
        "the station's corner"
    );
    let check = place(&map, &area, &resources, ExtractionPlate, 9, 9);
    assert_eq!(
        (check.tiles, check.whole),
        (vec![(t(9, 9), Resource)], vec![NotOnPatch])
    );
    assert!(
        place(&map, &area, &resources, ExtractionPlate, 3, 3).is_valid(),
        "a plate on the patch"
    );
}

/// Rule 7 (design change 0094): no tile a ground unit's box overlaps, whoever owns it; a box only touching the
/// footprint leaves it free.
#[test]
fn tiles_under_ground_units() {
    let (map, area) = open(16, 16);
    let at = |x: i32, y: i32| t(x, y).center();
    let centred = at(5, 5);
    let check = place(&map, &area, &[unit(Peacekeeper, centred.x, centred.y)], Generator, 5, 5);
    assert_eq!(check.tiles, [(t(5, 5), Unit)]);
    // On the corner of four tiles, all four.
    let check = place(&map, &area, &[unit(Guard, 384.0, 384.0)], Generator, 5, 5);
    assert_eq!(
        check.tiles,
        [t(5, 5), t(6, 5), t(5, 6), t(6, 6)].map(|tile| (tile, Unit))
    );
    // A Peacekeeper (24 × 24 su) touching the east edge (x = 448) and the south edge (y = 448).
    let touching = [unit(Peacekeeper, 460.0, 352.0), unit(Peacekeeper, 352.0, 460.0)];
    assert!(place(&map, &area, &touching, Generator, 5, 5).is_valid());
    let overlapping = unit(Peacekeeper, 459.0, 352.0);
    assert_eq!(
        place(&map, &area, &[overlapping], Generator, 5, 5).tiles,
        [(t(6, 5), Unit)]
    );
}

/// The build area (`factions/gdo/structures.md > GDOBuildArea`): a GDO building needs one tile in it, the
/// Deployment Center, which seeds it, none.
#[test]
fn a_footprint_must_reach_the_build_area() {
    let map = Map::new(64, 64);
    let area = dc_area(&map, 20, 20);
    assert_eq!(area.parts(), [TileRect::new(t(8, 8), 28, 28)]);
    for (x, y, inside) in [
        (35, 20, true),
        (36, 20, false),
        (35, 35, true),
        (36, 35, false),
        (6, 6, false),
        (7, 7, true),
    ] {
        let check = place(&map, &area, &[], Generator, x, y);
        assert_eq!(check.whole.is_empty(), inside, "({x}, {y})");
        assert!(check.tiles.is_empty());
    }
    assert_eq!(place(&map, &area, &[], Barracks, 40, 40).whole, [OutsideBuildArea]);
    assert!(place(&map, &BuildArea::default(), &[], DeploymentCenter, 40, 40).is_valid());
}

/// An Extraction Plate goes exactly onto a patch that has no plate yet, inside the area.
#[test]
fn plates_go_onto_uncovered_patches() {
    let map = Map::new(64, 64);
    let area = dc_area(&map, 20, 20);
    let patches = [on(CrystalPatch, 30, 30), on(CrystalPatch, 40, 40)];
    assert!(place(&map, &area, &patches, ExtractionPlate, 30, 30).is_valid());
    assert_eq!(
        place(&map, &area, &patches, ExtractionPlate, 31, 30).whole,
        [NotOnPatch]
    );
    assert_eq!(
        place(&map, &area, &patches, ExtractionPlate, 40, 40).whole,
        [OutsideBuildArea]
    );
    assert_eq!(
        place(&map, &area, &patches, ExtractionPlate, 41, 41).whole,
        [OutsideBuildArea, NotOnPatch]
    );
    // Covered: the plate on it is in the way.
    let covered = [on(CrystalPatch, 30, 30), on(ExtractionPlate, 30, 30)];
    let check = place(&map, &area, &covered, ExtractionPlate, 30, 30);
    assert_eq!((check.tiles, check.whole), (vec![(t(30, 30), Structure)], vec![]));
}

/// Units and map resources aren't placed by this check.
#[test]
fn only_structures_are_placed() {
    let (map, area) = open(16, 16);
    for kind in [Peacekeeper, Guard, CrystalPatch, SupplyDeliveryStation] {
        let check = place(&map, &area, &[], kind, 2, 2);
        assert_eq!((check.tiles, check.whole), (vec![], vec![NotAStructure]), "{kind:?}");
    }
}

/// Problems come tile by tile, row by row, each tile's in rule order, whatever the occupants' order.
#[test]
fn problems_come_in_a_fixed_order() {
    let map = Map::from_ascii(
        "
        ....
        .r..
        ....
        ....

        00000
        00000
        00000
        00010
        00000",
    )
    .unwrap();
    let area = dc_area(&map, 0, 0);
    let occupants = vec![
        unit(Peacekeeper, 100.0, 100.0),
        on(CrystalPatch, 2, 1),
        on(Generator, 2, 2),
        unit(Guard, 64.0, 140.0),
    ];
    let check = place(&map, &area, &occupants, DeploymentCenter, 0, 0);
    let expected = vec![
        (t(0, 1), Unit),
        (t(1, 1), NotBuildable),
        (t(1, 1), Unit),
        (t(2, 1), Resource),
        (t(0, 2), Unit),
        (t(1, 2), Unit),
        (t(2, 2), NotFlat),
        (t(2, 2), Structure),
        (t(3, 2), NotFlat),
        (t(3, 2), Structure),
        (t(2, 3), NotFlat),
        (t(2, 3), Structure),
        (t(3, 3), NotFlat),
        (t(3, 3), Structure),
    ];
    assert_eq!(check.tiles, expected);
    let reversed: Vec<Occupant> = occupants.into_iter().rev().collect();
    assert_eq!(place(&map, &area, &reversed, DeploymentCenter, 0, 0), check);
}

/// Design change 0125 (`> Lingering build ghosts`): the tiles of the sites the placing player has reserved are
/// `Reserved`, in their place among the other problems; a site only touching the footprint reserves none of it.
#[test]
fn tiles_under_reserved_sites() {
    let (map, area) = open(16, 16);
    let site = Site {
        map: &map,
        area: &area,
        mode: Mode::Layout,
        method: Method::WorkerBuilt,
    };
    let beacon = |x, y| Placement::new(t(x, y), ObjectKind::Beacon.def().footprint);
    let reserved = [beacon(5, 5), beacon(2, 4)];
    let unturned = BuildingOrientation::default();
    let check = check_reserving(
        &site,
        [on(CrystalPatch, 4, 5)],
        &reserved,
        ObjectKind::Beacon,
        t(4, 4),
        unturned,
    );
    assert_eq!(check.tiles, [(t(4, 5), Resource), (t(5, 5), Reserved)]);
    assert!(check_reserving(&site, [], &reserved, ObjectKind::Beacon, t(7, 7), unturned).is_valid());
}

/// `Mode::Known` (plan decision T16; design change 0120: "The ghost judges by what the player knows"): what stands on
/// a tile the player never explored, and the ground, pass; its reservations and the map's edge never do. Player 1's
/// Generator at (4, 4) explored the Mountain at (7, 5), not the one at (20, 4) nor player 2's Barracks at (18, 10).
#[test]
fn never_explored_tiles_pass_but_reservations() {
    let mut rows = vec![".".repeat(24); 16];
    rows[5].replace_range(7..8, "M");
    rows[4].replace_range(20..21, "M");
    let scenario = ScenarioBuilder::new(&rows.join("\n"))
        .spawn(Generator, Some(PlayerId(1)), crate::SpawnAt::Tile(t(4, 4)))
        .spawn(Barracks, Some(PlayerId(2)), crate::SpawnAt::Tile(t(18, 10)))
        .build();
    let sim = Sim::new(&scenario);
    let area = BuildArea::default();
    let known = Site {
        map: sim.map(),
        area: &area,
        mode: Mode::Known {
            vision: sim.vision(),
            player: PlayerId(1),
        },
        method: Method::WorkerBuilt,
    };
    let unturned = BuildingOrientation::default();
    let reserved = [Placement::new(t(20, 5), ObjectKind::Beacon.def().footprint)];
    let beacon = |x, y| {
        let objects = sim.objects().map(Occupant::of);
        check_reserving(&known, objects, &reserved, ObjectKind::Beacon, t(x, y), unturned)
    };
    assert_eq!(sim.visibility(PlayerId(1), t(7, 5)), VisibilityState::Visible);
    assert_eq!(beacon(6, 5).tiles, [(t(7, 5), NotBuildable)]);
    assert!(beacon(20, 2).is_valid() && beacon(18, 10).is_valid(), "never explored");
    assert_eq!(beacon(20, 4).tiles, [(t(20, 5), Reserved), (t(21, 5), Reserved)]);
    assert_eq!(
        beacon(23, 15).tiles,
        [(t(24, 15), OffMap), (t(23, 16), OffMap), (t(24, 16), OffMap)]
    );
}
