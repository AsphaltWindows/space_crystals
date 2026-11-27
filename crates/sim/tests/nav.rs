//! The navigation library: A\*, the field and retargeting — `docs/movement.md > 5.3`, `> 5.5`, tests of `> 8`
//! (nav unit tests). Grids, line of sight and smoothing are in `nav_grid.rs`.

use sc_sim::nav::{NavClass, NavGrid, NavRules, PathFail, Search, box_clear, find_path, los};
const DIAGONAL_COST: u32 = sc_sim::tuning::paths::BUILTIN.diagonal_cost;
const STEP_COST: u32 = sc_sim::tuning::paths::BUILTIN.step_cost;
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

fn random_tile(rng: &mut Rng, grid: &NavGrid) -> TilePos {
    t(rng.below(grid.width()) as i32, rng.below(grid.height()) as i32)
}

/// The cost of one step between neighbours, or `None` if the step isn't allowed.
fn step_cost(grid: &NavGrid, a: TilePos, b: TilePos) -> Option<u32> {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    if grid.is_blocked(a) || grid.is_blocked(b) || dx.abs().max(dy.abs()) != 1 {
        return None;
    }
    if dx != 0 && dy != 0 {
        let cut = grid.is_blocked(t(a.x + dx, a.y)) || grid.is_blocked(t(a.x, a.y + dy));
        return (!cut).then_some(DIAGONAL_COST);
    }
    Some(STEP_COST)
}

/// Cheapest cost from `from` to every tile: a plain O(n²) Dijkstra, written without the library's search.
fn brute_force_costs(grid: &NavGrid, from: TilePos) -> Vec<Option<u32>> {
    let n = (grid.width() * grid.height()) as usize;
    let mut cost: Vec<Option<u32>> = vec![None; n];
    let mut closed = vec![false; n];
    if grid.is_blocked(from) {
        return cost;
    }
    cost[grid.index(from).unwrap() as usize] = Some(0);
    loop {
        let next = (0..n)
            .filter(|&i| !closed[i] && cost[i].is_some())
            .min_by_key(|&i| cost[i]);
        let Some(i) = next else { return cost };
        closed[i] = true;
        let a = grid.tile(i as u32);
        for dy in -1..=1 {
            for dx in -1..=1 {
                let b = t(a.x + dx, a.y + dy);
                let Some(step) = step_cost(grid, a, b) else { continue };
                let j = grid.index(b).unwrap() as usize;
                let candidate = cost[i].unwrap() + step;
                if cost[j].is_none_or(|c| candidate < c) {
                    cost[j] = Some(candidate);
                }
            }
        }
    }
}

/// The summed step costs of a tile path; panics on a step the rules don't allow.
fn path_cost(grid: &NavGrid, tiles: &[TilePos]) -> u32 {
    tiles
        .windows(2)
        .map(|w| step_cost(grid, w[0], w[1]).unwrap_or_else(|| panic!("illegal step {:?} -> {:?}", w[0], w[1])))
        .sum()
}

#[test]
fn astar_cost_equals_brute_force_dijkstra_on_random_maps() {
    let mut search = Search::new();
    let mut reachable_pairs = 0;
    for seed in 1..=24 {
        let mut rng = Rng::from_seed(seed);
        let grid = ground(&random_map(&mut rng, 20, 14, 20 + (seed as u32 % 3) * 10));
        for _ in 0..12 {
            let (from, to) = (random_tile(&mut rng, &grid), random_tile(&mut rng, &grid));
            let expected = brute_force_costs(&grid, from)[grid.index(to).unwrap() as usize];
            let before = search.counters();
            let found = search.astar(&grid, from, to, &[]);
            let expansions = search.counters().expansions - before.expansions;
            match (expected, found) {
                (Some(cost), Some(path)) => {
                    reachable_pairs += 1;
                    assert_eq!(path.cost, cost, "seed {seed}: {from:?} -> {to:?}");
                    assert_eq!(path_cost(&grid, &path.tiles), cost);
                    assert_eq!((path.tiles[0], *path.tiles.last().unwrap()), (from, to));
                    // The closed set: no tile is expanded twice.
                    assert!(
                        expansions <= grid.passable_count(),
                        "seed {seed}: {expansions} expansions"
                    );
                }
                (None, None) => assert_eq!(expansions, 0, "an unreachable goal must not search"),
                (expected, found) => panic!("seed {seed}: {from:?} -> {to:?}: {expected:?} vs {found:?}"),
            }
        }
    }
    assert!(reachable_pairs > 100, "the maps must leave most pairs connected");
}

#[test]
fn field_paths_cost_the_same_as_astar() {
    let mut search = Search::new();
    let mut other = Search::new();
    for seed in 1..=8 {
        let mut rng = Rng::from_seed(100 + seed);
        let grid = ground(&random_map(&mut rng, 24, 16, 25));
        let anchor = loop {
            let tile = random_tile(&mut rng, &grid);
            if !grid.is_blocked(tile) {
                break tile;
            }
        };
        let before = search.counters();
        assert!(search.field(&grid, anchor));
        let work = search.counters();
        let in_component = (0..grid.width() * grid.height())
            .filter(|&i| grid.component(grid.tile(i)) == grid.component(anchor))
            .count() as u32;
        // One Dijkstra expands every tile of the anchor's component exactly once.
        assert_eq!(work.expansions - before.expansions, in_component);
        assert_eq!(work.fields - before.fields, 1);
        for i in 0..grid.width() * grid.height() {
            let from = grid.tile(i);
            let via_field = search.field_path(&grid, from);
            let direct = other.astar(&grid, from, anchor, &[]);
            assert_eq!(via_field.is_some(), direct.is_some(), "seed {seed}: {from:?}");
            let (Some(via_field), Some(direct)) = (via_field, direct) else {
                continue;
            };
            assert_eq!(via_field.cost, direct.cost, "seed {seed}: {from:?} -> {anchor:?}");
            assert_eq!(path_cost(&grid, &via_field.tiles), direct.cost);
            assert_eq!((via_field.tiles[0], *via_field.tiles.last().unwrap()), (from, anchor));
        }
        // Another search reuses the arrays: the field is gone, not silently wrong.
        search.astar(&grid, anchor, anchor, &[]);
        assert_eq!(search.field_path(&grid, anchor), None);
    }
}

#[test]
fn a_straight_path_on_an_open_map_has_two_waypoints() {
    let grid = ground(&Map::new(32, 32));
    let mut search = Search::new();
    // Corner to corner: the tie-break walks the diagonal and expands only the tiles on it.
    let path = search.astar(&grid, t(0, 0), t(31, 31), &[]).unwrap();
    assert_eq!(path.cost, 31 * DIAGONAL_COST);
    assert_eq!(search.counters().expansions, 32);
    let half = Vec2::splat(12.0);
    for (start, goal) in [
        (Vec2::new(40.0, 40.0), Vec2::new(1900.0, 1000.0)),
        (Vec2::new(2000.0, 100.0), Vec2::new(77.0, 1333.0)),
        (Vec2::new(500.0, 1990.0), Vec2::new(510.0, 20.0)),
        (Vec2::new(96.0, 96.0), Vec2::new(96.0, 96.0)),
    ] {
        let path = find_path(&grid, &mut search, half, start, goal, &[]).unwrap();
        assert_eq!(path.waypoints, vec![start, goal], "no zigzag");
        assert!(!path.retargeted);
    }
}

#[test]
fn the_same_call_twice_gives_the_same_path() {
    let mut rng = Rng::from_seed(7);
    let grid = ground(&random_map(&mut rng, 40, 30, 20));
    let half = Vec2::splat(12.0);
    // The first and the last tile of the biggest component, so the path is long and winding.
    let tiles: Vec<TilePos> = (0..40 * 30).map(|i| grid.tile(i)).collect();
    let biggest = (1..=grid.component_count())
        .max_by_key(|&c| tiles.iter().filter(|&&p| grid.component(p) == Some(c)).count())
        .unwrap();
    let mut members = tiles.iter().filter(|&&p| grid.component(p) == Some(biggest));
    let (from, to) = (*members.next().unwrap(), *members.next_back().unwrap());
    let (start, goal) = (grid.center(from), grid.center(to));
    let mut search = Search::new();
    let first = find_path(&grid, &mut search, half, start, goal, &[]).unwrap();
    assert!(first.waypoints.len() > 3, "{first:?}");
    let tile_path = search.astar(&grid, from, to, &[]).unwrap();
    // Other searches in between leave nothing behind in the scratch arrays.
    search.astar(&grid, to, from, &[]);
    search.field(&grid, from);
    assert_eq!(find_path(&grid, &mut search, half, start, goal, &[]).unwrap(), first);
    assert_eq!(
        find_path(&grid, &mut Search::new(), half, start, goal, &[]).unwrap(),
        first
    );
    assert_eq!(search.astar(&grid, from, to, &[]).unwrap(), tile_path);
    assert_eq!(Search::new().astar(&grid, from, to, &[]).unwrap(), tile_path);
}

#[test]
fn an_unreachable_goal_costs_no_expansions_and_retargets_to_the_nearest_reachable_tile() {
    // The goal (6, 2) is walled in; (4, 2) is also in a pocket of its own.
    let grid = grid(
        "........
         ....cccc
         ...c.c.c
         ....cccc
         ........",
    );
    let mut search = Search::new();
    for goal in [t(6, 2), t(5, 2), t(40, 2), t(-3, -3)] {
        assert_eq!(search.astar(&grid, t(0, 0), goal, &[]), None);
    }
    assert_eq!(search.counters().expansions, 0);
    assert_eq!(search.counters().searches, 0);

    let component = grid.component(t(0, 0)).unwrap();
    assert_ne!(grid.component(t(6, 2)), Some(component));
    // Nearest by distance: (6, 0) and (6, 4) are both 2 away; the lower tile index wins.
    assert_eq!(grid.nearest_in_component(component, t(6, 2)), Some(t(6, 0)));
    // A blocked goal next to open ground: (3, 1), (2, 2) and (3, 3) are 1 away; the lowest tile index wins.
    assert_eq!(grid.nearest_in_component(component, t(3, 2)), Some(t(3, 1)));
    // A reachable goal is its own nearest tile, and a goal off the map is clamped onto it first.
    assert_eq!(grid.nearest_in_component(component, t(1, 3)), Some(t(1, 3)));
    assert_eq!(grid.nearest_in_component(component, t(-5, 9)), Some(t(0, 4)));
    assert_eq!(grid.nearest_in_component(component + 5, t(0, 0)), None);

    let half = Vec2::splat(12.0);
    let start = grid.center(t(0, 2));
    let path = find_path(&grid, &mut search, half, start, grid.center(t(6, 2)), &[]).unwrap();
    assert!(path.retargeted);
    assert_eq!(*path.waypoints.last().unwrap(), grid.center(t(6, 0)));
    // The retargeted search costs what a search to that tile costs, not a flood of the map.
    let retargeted = search.counters().expansions;
    let mut direct = Search::new();
    direct.astar(&grid, t(0, 2), t(6, 0), &[]).unwrap();
    assert_eq!(retargeted, direct.counters().expansions);
    assert!(retargeted < grid.passable_count());
}

#[test]
fn nothing_cuts_a_corner_between_diagonal_obstacles() {
    let mut search = Search::new();
    // Two cliffs touching at a corner separate the two free tiles completely.
    let closed = grid(
        ".c
         c.",
    );
    assert_ne!(closed.component(t(0, 0)), closed.component(t(1, 1)));
    assert_eq!(search.astar(&closed, t(0, 0), t(1, 1), &[]), None);
    // (0, 1) -> (1, 2) would be one diagonal step between the cliffs; the legal way goes round the top.
    let open = grid(
        "...
         .c.
         c..",
    );
    let path = search.astar(&open, t(0, 1), t(1, 2), &[]).unwrap();
    assert_eq!(path.cost, 6 * STEP_COST);
    assert_eq!(
        path.tiles,
        vec![t(0, 1), t(0, 0), t(1, 0), t(2, 0), t(2, 1), t(2, 2), t(1, 2)]
    );
    // A single cliff: the diagonal past its corner isn't taken either.
    let corner = grid(
        "..
         c.",
    );
    assert_eq!(
        search.astar(&corner, t(0, 0), t(1, 1), &[]).unwrap().cost,
        2 * STEP_COST
    );
    // And the smoothed path keeps the unit's box out of the cliff.
    let half = Vec2::splat(18.0);
    let (start, goal) = (Vec2::new(20.0, 100.0), Vec2::new(100.0, 170.0));
    let path = find_path(&open, &mut search, half, start, goal, &[]).unwrap();
    assert_path_clear(&open, half, &path.waypoints);
    assert_eq!(*path.waypoints.last().unwrap(), goal);
}

#[test]
fn avoid_tiles_block_a_search_and_only_they_can_make_it_fail() {
    let grid = grid(
        ".....
         cc.cc
         .....",
    );
    let mut search = Search::new();
    let through = search.astar(&grid, t(0, 0), t(0, 2), &[]).unwrap();
    assert!(through.tiles.contains(&t(2, 1)));
    // The gap is the only way: avoiding it fails, after flooding what is left.
    assert_eq!(search.astar(&grid, t(0, 0), t(0, 2), &[t(2, 1)]), None);
    // An avoided goal fails without searching; the start tile is never avoided.
    let before = search.counters();
    assert_eq!(search.astar(&grid, t(0, 0), t(0, 2), &[t(0, 2)]), None);
    assert_eq!(search.counters(), before);
    assert_eq!(search.astar(&grid, t(0, 0), t(0, 2), &[t(0, 0)]), Some(through));
    // Avoiding a tile with a way round it changes the route, and corner cutting past it is still forbidden.
    let around = search.astar(&grid, t(0, 0), t(4, 0), &[t(2, 0)]);
    assert_eq!(around, None, "row 0 is cut and the gap leads nowhere else");
    let wide = ground(&Map::new(5, 3));
    let detour = search.astar(&wide, t(0, 0), t(4, 0), &[t(2, 0)]).unwrap();
    assert_eq!(detour.cost, 2 * DIAGONAL_COST + 2 * STEP_COST);
    assert!(!detour.tiles.contains(&t(2, 0)));
    let half = Vec2::splat(12.0);
    let result = find_path(
        &grid,
        &mut search,
        half,
        grid.center(t(0, 0)),
        grid.center(t(0, 2)),
        &[t(2, 1)],
    );
    assert_eq!(result, Err(PathFail::Avoided));
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
