//! The navigation library: grids, line of sight, smoothing and the rebuild — `docs/movement.md > 5.2`, `> 5.4`,
//! `> 5.7`, tests of `> 8` (nav unit tests). The searches are in `nav.rs`.

use sc_sim::nav::{
    BlockerKind, Nav, NavClass, NavGrid, NavRules, Search, TileRect, box_clear, find_path, los, size_class, smooth,
};
use sc_sim::{Map, Rng, Tile, TilePos, TilePresetId, Vec2};

fn t(x: i32, y: i32) -> TilePos {
    TilePos::new(x, y)
}

fn ground(map: &Map) -> NavGrid {
    NavGrid::build(map, NavClass::Ground, 1, &[], NavRules::default())
}

fn grid(ascii: &str) -> NavGrid {
    ground(&Map::from_ascii(ascii).unwrap())
}

/// A map with about `percent` % cliffs.
fn random_map(rng: &mut Rng, w: u32, h: u32, percent: u32) -> Map {
    let mut map = Map::new(w, h);
    for y in 0..h as i32 {
        for x in 0..w as i32 {
            if rng.below(100) < percent {
                map.set_tile(t(x, y), Tile::new(TilePresetId::CLIFF));
            }
        }
    }
    map
}

/// Every leg of a path must have line of sight for the box, and its end must be a place the box fits.
fn assert_path_clear(grid: &NavGrid, half: Vec2, waypoints: &[Vec2]) {
    assert!(waypoints.len() >= 2);
    for leg in waypoints.windows(2) {
        assert!(
            los(grid, half, leg[0], leg[1]),
            "no line of sight {} -> {}",
            leg[0],
            leg[1]
        );
    }
    let end = *waypoints.last().unwrap();
    assert!(box_clear(grid, half, end), "the box doesn't fit at {end}");
}

#[test]
fn line_of_sight_against_hand_drawn_corners() {
    let grid = grid(
        ".....
         ..c..
         .....
         c....",
    );
    let at = |x: f32, y: f32| Vec2::new(x, y);
    let point = Vec2::ZERO;
    let (small, guard, full) = (Vec2::splat(12.0), Vec2::splat(18.0), Vec2::splat(32.0));
    // Straight through the cliff, and along open rows.
    assert!(!los(&grid, point, at(96.0, 96.0), at(224.0, 96.0)));
    assert!(los(&grid, full, at(32.0, 32.0), at(288.0, 32.0)));
    assert!(los(&grid, full, at(32.0, 160.0), at(288.0, 160.0)));
    // A point grazes the cliff's corner (128, 64): touching is free. Any box there is not.
    assert!(los(&grid, point, at(64.0, 128.0), at(192.0, 0.0)));
    assert!(!los(&grid, small, at(76.0, 140.0), at(180.0, 36.0)));
    // Past the cliff's north-west corner: the diagonal offset decides. The cliff is [128, 192] x [64, 128].
    assert!(!los(&grid, small, at(32.0, 96.0), at(160.0, 32.0)));
    assert!(los(&grid, small, at(32.0, 40.0), at(160.0, 40.0)));
    assert!(
        los(&grid, small, at(32.0, 52.0), at(160.0, 52.0)),
        "the box's edge slides along the cliff's"
    );
    assert!(!los(&grid, small, at(32.0, 52.5), at(160.0, 52.0)));
    assert!(!los(&grid, guard, at(32.0, 52.0), at(160.0, 52.0)));
    // Tile centres leave `32 - half` of corner clearance: between the two cliffs' corners, centre to centre.
    assert!(los(&grid, full, at(96.0, 160.0), at(96.0, 32.0)));
    assert!(los(&grid, guard, at(96.0, 160.0), at(32.0, 96.0)));
    assert!(
        !los(&grid, guard, at(96.0, 224.0), at(32.0, 160.0)),
        "cuts the corner of the cliff at (0, 3)"
    );
    assert!(los(&grid, guard, at(96.0, 224.0), at(96.0, 160.0)));
    // A diagonal that passes exactly through a tile corner next to a cliff.
    assert!(!los(&grid, small, at(96.0, 160.0), at(224.0, 32.0)));
    assert!(los(&grid, point, at(224.0, 160.0), at(288.0, 96.0)));
    // The map edge is a wall: the box must stay inside.
    assert!(los(&grid, small, at(12.0, 12.0), at(308.0, 12.0)));
    assert!(!los(&grid, small, at(12.0, 11.0), at(308.0, 12.0)));
    assert!(!los(&grid, small, at(32.0, 32.0), at(330.0, 32.0)));
    assert!(!los(&grid, point, at(32.0, 32.0), at(-1.0, 32.0)));
    assert!(!los(&grid, point, at(32.0, 32.0), at(f32::NAN, 32.0)));
    // A point is its own line of sight wherever the box fits.
    assert!(los(&grid, small, at(100.0, 100.0), at(100.0, 100.0)));
    assert!(!los(&grid, small, at(120.0, 100.0), at(120.0, 100.0)));
    assert!(box_clear(&grid, small, at(116.0, 100.0)));
    assert!(!box_clear(&grid, small, at(116.5, 100.0)));
    assert!(!box_clear(&grid, small, at(160.0, 96.0)));
    assert!(!box_clear(&grid, small, at(11.0, 100.0)));
    // Line of sight is symmetric.
    let mut rng = Rng::from_seed(3);
    for _ in 0..500 {
        let mut random = || at(rng.next_f32() * 320.0, rng.next_f32() * 256.0);
        let (a, b) = (random(), random());
        assert_eq!(los(&grid, small, a, b), los(&grid, small, b, a), "{a} <-> {b}");
    }
}

#[test]
fn smoothing_goes_round_a_wall_through_tile_centres() {
    let grid = grid(
        ".....
         .ccc.
         .....",
    );
    let half = Vec2::splat(18.0);
    let (start, goal) = (Vec2::new(100.0, 20.0), Vec2::new(100.0, 170.0));
    let mut search = Search::new();
    let tiles = search.astar(&grid, t(1, 0), t(1, 2), &[]).unwrap().tiles;
    assert_eq!(tiles, vec![t(1, 0), t(0, 0), t(0, 1), t(0, 2), t(1, 2)]);
    let waypoints = smooth(&grid, half, start, &tiles, goal);
    assert_eq!(
        waypoints,
        vec![start, Vec2::new(32.0, 32.0), Vec2::new(32.0, 160.0), goal]
    );
    assert_path_clear(&grid, half, &waypoints);
    assert_eq!(
        find_path(&grid, &mut search, half, start, goal, &[]).unwrap().waypoints,
        waypoints
    );
}

#[test]
fn smoothed_paths_keep_the_box_clear_on_random_maps() {
    let mut search = Search::new();
    let mut paths = 0;
    for seed in 1..=30 {
        let mut rng = Rng::from_seed(500 + seed);
        let grid = ground(&random_map(&mut rng, 24, 18, 22));
        let (w, h) = (grid.width() as f32 * 64.0, grid.height() as f32 * 64.0);
        for half in [Vec2::splat(12.0), Vec2::splat(18.0), Vec2::new(32.0, 20.0)] {
            for _ in 0..12 {
                let start = Vec2::new(rng.next_f32() * w, rng.next_f32() * h);
                let goal = Vec2::new(rng.next_f32() * w, rng.next_f32() * h);
                if !box_clear(&grid, half, start) {
                    // Units never stand where their box doesn't fit.
                    continue;
                }
                let path = find_path(&grid, &mut search, half, start, goal, &[]).unwrap();
                paths += 1;
                assert_eq!(path.waypoints[0], start);
                assert_path_clear(&grid, half, &path.waypoints);
                let end = *path.waypoints.last().unwrap();
                let goal_tile = grid.tile_of(goal);
                if path.retargeted {
                    assert_ne!(grid.component(goal_tile), grid.component(grid.tile_of(start)));
                } else {
                    // The end is in the goal's tile, and exactly the goal wherever the box fits there.
                    assert_eq!(grid.tile_of(end), goal_tile, "seed {seed}: {goal} -> {end}");
                    assert!(
                        end == goal || !box_clear(&grid, half, goal),
                        "seed {seed}: {goal} -> {end}"
                    );
                }
            }
        }
    }
    assert!(paths > 500);
}

#[test]
fn grids_follow_the_class_the_footprints_and_the_switches() {
    // `r` Rugged, `c` Cliff (Drillable), `M` Mountain, `~` Water (neither Traversible nor Drillable).
    let map = Map::from_ascii(
        "..r.c.
         ..r.M~
         ......",
    )
    .unwrap();
    let rules = NavRules::default();
    let build = |class, rules| NavGrid::build(&map, class, 1, &[], rules);
    let blocked =
        |grid: &NavGrid| -> Vec<TilePos> { (0..18).map(|i| grid.tile(i)).filter(|&p| grid.is_blocked(p)).collect() };
    // Q0011 (G11): Rugged is impassable for a base without RuggedTerrain.
    assert_eq!(
        blocked(&build(NavClass::Ground, rules)),
        vec![t(2, 0), t(4, 0), t(2, 1), t(4, 1), t(5, 1)]
    );
    assert_eq!(
        blocked(&build(NavClass::GroundRugged, rules)),
        vec![t(4, 0), t(4, 1), t(5, 1)]
    );
    assert_eq!(blocked(&build(NavClass::Underground, rules)), vec![t(4, 1), t(5, 1)]);
    let lenient = NavRules {
        rugged_impassable: false,
        ..rules
    };
    assert_eq!(
        blocked(&build(NavClass::Ground, lenient)),
        blocked(&build(NavClass::GroundRugged, rules))
    );
    // The map edge is blocked.
    let ground = build(NavClass::Ground, rules);
    for outside in [t(-1, 0), t(0, -1), t(6, 0), t(0, 3)] {
        assert!(ground.is_blocked(outside) && ground.is_solid(outside));
        assert_eq!(ground.component(outside), None);
    }
    // Components: labels follow the first tile in row-major order. (5, 0) is cut off by the cliff column.
    assert_eq!(ground.component_count(), 2);
    assert_eq!(ground.component(t(0, 0)), Some(1));
    assert_eq!(ground.component(t(3, 0)), Some(1));
    assert_eq!(ground.component(t(5, 0)), Some(2));
    assert_eq!(ground.component(t(2, 0)), None);
    assert_eq!(ground.passable_count(), 13);
    // Footprints block the surface classes, never the underground one. Parts off the map are ignored.
    let footprints = [
        TileRect::new(t(0, 1), 2, 2),
        TileRect::new(t(5, 2), 3, 3),
        TileRect::new(t(-2, -2), 1, 1),
    ];
    let with = |class| NavGrid::build(&map, class, 1, &footprints, rules);
    assert!(with(NavClass::Ground).is_blocked(t(1, 2)) && with(NavClass::Ground).is_blocked(t(5, 2)));
    assert!(with(NavClass::GroundRugged).is_blocked(t(0, 1)));
    assert_eq!(with(NavClass::Underground), build(NavClass::Underground, rules));
    // Q0011 (G10): resources block; structures always do; underground expansions never.
    assert!(rules.footprint_blocks(BlockerKind::Structure));
    assert!(rules.footprint_blocks(BlockerKind::Resource));
    assert!(!rules.footprint_blocks(BlockerKind::UndergroundExpansion));
    let walkable = NavRules {
        resources_block: false,
        ..rules
    };
    assert!(!walkable.footprint_blocks(BlockerKind::Resource) && walkable.footprint_blocks(BlockerKind::Structure));
}

#[test]
fn larger_size_classes_use_true_clearance() {
    assert_eq!(size_class(Vec2::splat(24.0)), 1);
    assert_eq!(size_class(Vec2::new(36.0, 64.0)), 1);
    assert_eq!(size_class(Vec2::new(65.0, 20.0)), 2);
    let map = Map::from_ascii(
        ".....
         .....
         ..c..
         .....
         c....",
    )
    .unwrap();
    let grid = NavGrid::build(&map, NavClass::Ground, 2, &[], NavRules::default());
    // A tile is passable when the 2 x 2 block anchored on it is free and on the map.
    let passable: Vec<TilePos> = (0..25).map(|i| grid.tile(i)).filter(|&p| !grid.is_blocked(p)).collect();
    assert_eq!(
        passable,
        vec![
            t(0, 0),
            t(1, 0),
            t(2, 0),
            t(3, 0),
            t(0, 1),
            t(3, 1),
            t(0, 2),
            t(3, 2),
            t(1, 3),
            t(2, 3),
            t(3, 3)
        ]
    );
    // The one-tile gap west of the cliff is too narrow: the way from (0, 2) to (1, 3) leads round the east.
    assert_eq!(grid.component_count(), 1);
    assert_eq!(grid.passable_count(), 11);
    assert!(!grid.is_solid(t(1, 1)) && grid.is_blocked(t(1, 1)));
    // A unit anchored on a tile is centred on its block.
    assert_eq!(grid.center(t(3, 2)), Vec2::new(256.0, 192.0));
    assert_eq!(grid.tile_of(Vec2::new(256.0, 192.0)), t(3, 2));
    let half = Vec2::splat(40.0);
    let path = find_path(
        &grid,
        &mut Search::new(),
        half,
        grid.center(t(0, 2)),
        grid.center(t(1, 3)),
        &[],
    )
    .unwrap();
    assert!(path.waypoints.len() > 2);
    assert_path_clear(&grid, half, &path.waypoints);
}

#[test]
fn a_rebuild_follows_a_footprint_change() {
    let map = Map::new(9, 7);
    let keys = [(NavClass::Ground, 1), (NavClass::Underground, 1), (NavClass::Ground, 1)];
    let rules = NavRules::default();
    // A structure wall across the whole map, then the same wall with a door.
    let wall = [TileRect::new(t(4, 0), 1, 7)];
    let door = [TileRect::new(t(4, 0), 1, 3), TileRect::new(t(4, 4), 1, 3)];
    let mut nav = Nav::build(&map, &[], &keys, rules);
    assert_eq!(nav.grids().len(), 2, "duplicate keys build one grid");
    assert_eq!(nav.grid(NavClass::GroundRugged, 1), None);
    let half = Vec2::splat(12.0);
    let (start, goal) = (Vec2::new(32.0, 96.0), Vec2::new(544.0, 96.0));
    let mut search = Search::new();
    let mut route = |nav: &Nav| {
        find_path(
            nav.grid(NavClass::Ground, 1).unwrap(),
            &mut search,
            half,
            start,
            goal,
            &[],
        )
    };
    assert_eq!(route(&nav).unwrap().waypoints, vec![start, goal]);

    nav.rebuild(&map, &wall);
    let ground = nav.grid(NavClass::Ground, 1).unwrap();
    assert_eq!(ground.component_count(), 2);
    assert_ne!(ground.component(t(0, 1)), ground.component(t(8, 1)));
    let cut_off = route(&nav).unwrap();
    assert!(cut_off.retargeted);
    assert_eq!(*cut_off.waypoints.last().unwrap(), Vec2::new(224.0, 96.0));
    // The underground grid ignores the structure.
    assert_eq!(nav.grid(NavClass::Underground, 1).unwrap().component_count(), 1);

    nav.rebuild(&map, &door);
    let ground = nav.grid(NavClass::Ground, 1).unwrap();
    assert_eq!(ground.component_count(), 1);
    let through = route(&nav).unwrap();
    assert!(!through.retargeted);
    assert_eq!(*through.waypoints.last().unwrap(), goal);
    assert!(through.waypoints.len() > 2, "through the door: {through:?}");
    assert_path_clear(ground, half, &through.waypoints);
    // A rebuild is exactly a fresh build, and the map counts too.
    assert_eq!(nav, Nav::build(&map, &door, &keys, rules));
    let mut flooded = map.clone();
    flooded.set_tile(t(4, 3), Tile::new(TilePresetId::WATER));
    nav.rebuild(&flooded, &door);
    assert_eq!(nav.grid(NavClass::Ground, 1).unwrap().component_count(), 2);
    assert_eq!(nav.grid(NavClass::Underground, 1).unwrap().passable_count(), 62);
    assert_eq!(nav, Nav::build(&flooded, &door, &keys, rules));
}

/// (G) `NavGrid::open_around` is the movement step's short cut past `box_clear` and `los`: where it holds, a box up
/// to a tile wide is clear, and so is every move of it shorter than a tile less its half extent. Off the map, on
/// the map's edge and next to a solid tile it doesn't hold.
#[test]
fn open_ground_is_clear_without_a_look_at_the_tiles() {
    let mut rng = Rng::from_seed(77);
    let (mut open, mut near_a_cliff) = (0, 0);
    for _ in 0..6 {
        let map = random_map(&mut rng, 20, 16, 6);
        let grid = ground(&map);
        for _ in 0..4000 {
            let p = Vec2::new(rng.next_f32() * 1400.0 - 60.0, rng.next_f32() * 1140.0 - 60.0);
            let tile = TilePos::containing(p);
            let around = (-1..=1).all(|dy| (-1..=1).all(|dx| !grid.is_solid(t(tile.x + dx, tile.y + dy))));
            assert_eq!(grid.open_around(p), around, "{p}");
            if !around {
                near_a_cliff += 1;
                continue;
            }
            open += 1;
            for half in [Vec2::splat(12.0), Vec2::splat(18.0), Vec2::new(32.0, 20.0)] {
                assert!(box_clear(&grid, half, p), "{p} for {half}");
                let reach = 64.0 - half.max_element();
                let d = Vec2::new(rng.next_f32() * 2.0 - 1.0, rng.next_f32() * 2.0 - 1.0) * reach;
                assert!(los(&grid, half, p, p + d), "{p} along {d} for {half}");
            }
        }
    }
    assert!(open > 2000 && near_a_cliff > 2000, "{open} open, {near_a_cliff} not");
    let grid = grid("...\n...\n...");
    assert!(grid.open_around(Vec2::new(96.0, 96.0)));
    for p in [
        Vec2::new(32.0, 96.0),
        Vec2::new(-1.0, 96.0),
        Vec2::new(96.0, -0.5),
        Vec2::new(500.0, 96.0),
        Vec2::new(f32::NAN, 96.0),
        Vec2::new(96.0, f32::INFINITY),
    ] {
        assert!(!grid.open_around(p), "{p}");
    }
}
