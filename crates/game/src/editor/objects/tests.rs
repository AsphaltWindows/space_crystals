use sc_sim::{MapSettings, MapType, Symmetry, scenarios};

use super::*;

fn t(x: i32, y: i32) -> TilePos {
    TilePos::new(x, y)
}

const UNTURNED: BuildingOrientation = BuildingOrientation {
    rotation: sc_sim::Rotation::R0,
    flip_h: false,
    flip_v: false,
};

fn tile_spawn(kind: ObjectKind, x: i32, y: i32) -> Spawn {
    Spawn::new(kind, None, SpawnAt::Tile(t(x, y)))
}

/// Slice M2-I: a quarter turn swaps a footprint's sides for the snap, the area and the tiles (the Barracks is 3
/// wide and 2 tall unturned, design change 0079).
#[test]
fn turned_structures_snap_and_cover_their_turned_footprint() {
    let barracks = ObjectKind::Barracks;
    let quarter = crate::editor::orientation::Turn::Clockwise.apply(UNTURNED);
    assert_eq!(oriented_size(barracks, UNTURNED), Some((3, 2)));
    assert_eq!(oriented_size(barracks, quarter), Some((2, 3)));
    // Centred on the grid point nearest the cursor, (3, 3) tiles: 3 wide around x 3 rounds east.
    assert_eq!(
        snap_spawn(barracks, UNTURNED, Vec2::splat(192.0)),
        SpawnAt::Tile(t(2, 2))
    );
    assert_eq!(
        snap_spawn(barracks, quarter, Vec2::splat(192.0)),
        SpawnAt::Tile(t(2, 2))
    );
    let at = SpawnAt::Tile(t(2, 2));
    assert_eq!(spawn_rect(barracks, at, quarter).size(), Vec2::new(128.0, 192.0));
    assert_eq!(footprint_tiles(barracks, at, quarter).len(), 6);
    assert!(footprint_tiles(barracks, at, quarter).contains(&t(3, 4)));
    let map = Map::new(6, 5);
    assert_eq!(
        dragged_spawn(&map, barracks, quarter, Vec2::splat(1e4)),
        SpawnAt::Tile(t(4, 2)),
        "kept on the map, 2 wide and 3 tall"
    );
}

/// Slice M2-I: the one overlap the sim allows (`ObjectKind::may_overlap`), a plate onto an uncovered patch.
#[test]
fn a_plate_goes_onto_an_uncovered_patch_only() {
    let mut map = scenarios::empty_map(8, 6);
    map.spawns = vec![tile_spawn(ObjectKind::CrystalPatch, 2, 2)];
    let plate = tile_spawn(ObjectKind::ExtractionPlate, 2, 2);
    assert_eq!(check_placement(&map, &plate, None), Ok(()));
    map.spawns.push(plate.clone());
    assert_eq!(
        check_placement(&map, &plate, None),
        Err(PlaceProblem::Overlaps(1)),
        "the patch is covered"
    );
    assert_eq!(
        check_placement(&map, &tile_spawn(ObjectKind::CrystalPatch, 2, 2), None),
        Err(PlaceProblem::Overlaps(0)),
        "no patch on a patch"
    );
    assert_eq!(
        check_placement(&map, &tile_spawn(ObjectKind::Generator, 1, 1), None),
        Err(PlaceProblem::Overlaps(0)),
        "no other building on a patch"
    );
    assert_eq!(
        spawn_at_point(&map, t(2, 2).center()),
        Some(1),
        "the plate covers its patch"
    );
    map.spawns.swap(0, 1);
    assert_eq!(spawn_at_point(&map, t(2, 2).center()), Some(0), "whatever their order");
    assert_eq!(map.validate(), Vec::new());
}

#[test]
fn placements_snap_to_the_grid_and_stay_on_the_map() {
    let patch = ObjectKind::CrystalPatch; // 1×1
    let station = ObjectKind::SupplyDeliveryStation; // 2×2
    assert_eq!(
        snap_spawn(patch, UNTURNED, Vec2::new(100.0, 70.0)),
        SpawnAt::Tile(t(1, 1))
    );
    // A 2×2 footprint centres on the nearest grid point: (128, 128) is the corner of tiles 1 and 2.
    assert_eq!(
        snap_spawn(station, UNTURNED, Vec2::new(120.0, 140.0)),
        SpawnAt::Tile(t(1, 1))
    );
    let map = Map::new(6, 5);
    assert_eq!(
        dragged_spawn(&map, station, UNTURNED, Vec2::new(-500.0, 9000.0)),
        SpawnAt::Tile(t(0, 3))
    );
    assert_eq!(
        spawn_rect(station, SpawnAt::Tile(t(1, 1)), UNTURNED).center(),
        Vec2::splat(128.0)
    );
    let p = clamp_to_map(&map, Vec2::new(1e9, f32::NAN));
    assert!(map.contains_point(p));
}

#[test]
fn placement_checks_the_map_and_other_footprints() {
    let mut map = scenarios::empty_map(8, 6);
    map.spawns = vec![tile_spawn(ObjectKind::SupplyDeliveryStation, 2, 2)];
    let station = |x, y| tile_spawn(ObjectKind::SupplyDeliveryStation, x, y);
    let patch = |x, y| tile_spawn(ObjectKind::CrystalPatch, x, y);
    assert_eq!(check_placement(&map, &patch(0, 0), None), Ok(()));
    assert_eq!(
        check_placement(&map, &patch(3, 3), None),
        Err(PlaceProblem::Overlaps(0))
    );
    assert_eq!(check_placement(&map, &station(7, 0), None), Err(PlaceProblem::OffMap));
    assert_eq!(check_placement(&map, &station(-1, 0), None), Err(PlaceProblem::OffMap));
    assert_eq!(
        check_placement(&map, &station(3, 3), Some(0)),
        Ok(()),
        "moving onto itself"
    );
    assert_eq!(check_placement(&map, &station(4, 2), None), Ok(()), "touching is fine");
    // `core/map.md > Elevation > Map resources only on flat terrain`: one sloped tile is enough to refuse it.
    map.terrain.set_corner(sc_sim::CornerPos::new(6, 1), 1);
    assert_eq!(
        check_placement(&map, &station(5, 0), None),
        Err(PlaceProblem::NotFlat(t(5, 0)))
    );
    assert_eq!(
        check_placement(&map, &patch(6, 1), None),
        Err(PlaceProblem::NotFlat(t(6, 1)))
    );
    assert_eq!(check_placement(&map, &patch(7, 1), None), Ok(()));
    assert!(
        PlaceProblem::NotFlat(t(5, 0))
            .message(ObjectKind::SupplyDeliveryStation)
            .contains("flat ground")
    );
    // Skirmish Default maps take neutral objects only.
    let mut owned = patch(0, 0);
    owned.owner = Some(SlotId(1));
    map.map_type = MapType::SkirmishDefault;
    assert_eq!(
        check_placement(&map, &owned, None),
        Err(PlaceProblem::OwnedOnSkirmishMap)
    );
}

/// On a symmetric map an object goes in the editable part, and neither it nor its images may overlap another
/// object or partly overlap each other (the placeholder rule for objects on the axes).
#[test]
fn placement_on_a_symmetric_map() {
    let map = MapSettings {
        symmetry: Symmetry::MirrorVerticalAxis,
        width: 17,
        height: 16,
        ..MapSettings::default()
    }
    .create()
    .unwrap();
    let station = |x, y| tile_spawn(ObjectKind::SupplyDeliveryStation, x, y);
    assert_eq!(check_placement(&map, &station(2, 2), None), Ok(()));
    assert_eq!(check_placement(&map, &station(12, 2), None), Err(PlaceProblem::Locked));
    // The axis runs through column 8: a 2x2 station at column 7 or 8 straddles it and overlaps its image.
    assert_eq!(
        check_placement(&map, &station(7, 2), None),
        Err(PlaceProblem::OverlapsOwnImage)
    );
    let patch = tile_spawn(ObjectKind::CrystalPatch, 8, 2);
    assert_eq!(
        check_placement(&map, &patch, None),
        Ok(()),
        "on the axis: its own image"
    );
    // An image that would overlap another object.
    let mut map = map;
    map.spawns = vec![station(13, 5)];
    assert_eq!(
        check_placement(&map, &station(2, 5), None),
        Err(PlaceProblem::Overlaps(0))
    );
}

/// Review finding: a spawn of a damaged file at the end of the `i32` range must not overflow.
#[test]
fn footprints_at_the_ends_of_the_coordinate_range_never_overflow() {
    let station = ObjectKind::SupplyDeliveryStation;
    assert_eq!(
        footprint_tiles(station, SpawnAt::Tile(t(i32::MAX, i32::MAX)), UNTURNED).len(),
        1
    );
    assert_eq!(
        footprint_tiles(station, SpawnAt::Tile(t(i32::MAX - 1, 0)), UNTURNED).len(),
        4
    );
    let mut map = scenarios::empty_map(8, 6);
    map.spawns = vec![
        tile_spawn(station, i32::MAX, i32::MAX),
        tile_spawn(station, i32::MIN, 3),
    ];
    assert_eq!(check_placement(&map, &tile_spawn(station, 1, 1), None), Ok(()));
    let _ = spawn_at_point(&map, Vec2::new(1e30, -1e30));
    map.symmetry = Symmetry::Rotate180;
    let _ = check_placement(&map, &tile_spawn(station, i32::MAX, i32::MIN), None);
}

#[test]
fn hit_tests_find_the_frontmost_object() {
    let mut map = scenarios::empty_map(8, 6);
    map.spawns = vec![
        tile_spawn(ObjectKind::SupplyDeliveryStation, 1, 1),
        tile_spawn(ObjectKind::CrystalPatch, 2, 2),
        tile_spawn(ObjectKind::CrystalPatch, 5, 5),
    ];
    assert_eq!(spawn_at_point(&map, Vec2::new(100.0, 100.0)), Some(0));
    assert_eq!(
        spawn_at_point(&map, Vec2::new(150.0, 150.0)),
        Some(1),
        "the overlapping patch"
    );
    assert_eq!(spawn_at_point(&map, Vec2::new(5.0, 5.0)), None);
}

#[test]
fn problems_point_at_their_spawn() {
    let mut map = scenarios::empty_map(8, 6);
    map.spawns = vec![
        tile_spawn(ObjectKind::CrystalPatch, 2, 2),
        tile_spawn(ObjectKind::CrystalPatch, 2, 2),
        tile_spawn(ObjectKind::CrystalPatch, 40, 2),
    ];
    let problems = map.validate();
    assert_eq!(problems.len(), 2);
    assert_eq!(problem_focus(&map, &problems[0]), (Some(t(2, 2).center()), Some(1)));
    let (center, spawn) = problem_focus(&map, &problems[1]);
    assert_eq!(spawn, Some(2));
    assert!(
        map.terrain.contains_point(center.unwrap()),
        "off-map spots are clamped onto the map"
    );
}

#[test]
fn the_palette_lists_every_object_type_once() {
    let groups = palette_groups();
    let listed: Vec<ObjectKind> = groups.iter().flat_map(|g| g.kinds.iter().copied()).collect();
    let mut sorted = listed.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(sorted.len(), listed.len(), "no type twice");
    assert_eq!(sorted.len(), ObjectKind::ALL.len(), "every type");
    assert_eq!(groups[0].title, "Resources");
    // (M6-GK) The GDO's mine, of a category of its own (design change 0426), in a group of its own.
    let mines = groups.iter().find(|g| g.title == "GDO mines").expect("the GDO's mines");
    assert_eq!(mines.kinds, [ObjectKind::Mine]);
}
