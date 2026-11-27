//! Serving path requests within the expansion budget, and group fields — `docs/movement.md > 5.5`.

use sc_sim::nav::{
    Nav, NavClass, NavGrid, NavRules, Path, PathFail, PathGroup, PathRequest, Search, ServeOutput, box_clear,
    find_path, los, serve,
};
const GROUP_FIELD_MIN: usize = sc_sim::tuning::paths::BUILTIN.group_field_min;
const PATH_BUDGET: u32 = sc_sim::tuning::paths::BUILTIN.path_budget;
use sc_sim::{Map, ObjectId, Rng, Tile, TilePos, TilePresetId, Vec2};

const HALF: Vec2 = Vec2::splat(12.0);

fn t(x: i32, y: i32) -> TilePos {
    TilePos::new(x, y)
}

fn nav(map: &Map) -> Nav {
    Nav::build(map, &[], &[(NavClass::Ground, 1)], NavRules::default())
}

fn ground(nav: &Nav) -> &NavGrid {
    nav.grid(NavClass::Ground, 1).unwrap()
}

fn request(id: u32, since: u32, start: TilePos, goal: TilePos) -> PathRequest {
    PathRequest {
        id: ObjectId(id),
        since,
        class: NavClass::Ground,
        size: 1,
        half: HALF,
        start: start.center(),
        goal: goal.center(),
        group: None,
        avoid: Vec::new(),
    }
}

fn grouped(id: u32, group: u32, anchor: TilePos, start: TilePos, goal: TilePos) -> PathRequest {
    PathRequest {
        group: Some(PathGroup {
            id: group,
            anchor: anchor.center(),
        }),
        ..request(id, 1, start, goal)
    }
}

fn ids(out: &ServeOutput) -> Vec<u32> {
    out.served.iter().map(|s| s.id.0).collect()
}

fn path_of(out: &ServeOutput, id: u32) -> &Path {
    let served = out.served.iter().find(|s| s.id == ObjectId(id)).expect("served");
    served.result.as_ref().expect("a path")
}

fn assert_path_clear(grid: &NavGrid, path: &Path) {
    assert!(path.waypoints.len() >= 2);
    for leg in path.waypoints.windows(2) {
        assert!(
            los(grid, HALF, leg[0], leg[1]),
            "no line of sight {} -> {}",
            leg[0],
            leg[1]
        );
    }
    assert!(box_clear(grid, HALF, *path.waypoints.last().unwrap()));
}

#[test]
fn requests_are_served_oldest_first_and_a_started_job_finishes() {
    let nav = nav(&Map::new(16, 16));
    let requests = [
        request(5, 2, t(0, 0), t(15, 15)),
        request(9, 1, t(15, 0), t(0, 15)),
        request(3, 2, t(0, 8), t(15, 8)),
    ];
    let mut search = Search::new();
    let all = serve(&nav, &mut search, &requests, PATH_BUDGET);
    assert_eq!(ids(&all), [9, 3, 5], "by (since, id), whatever the input order");
    assert_eq!((all.counters.searches, all.counters.fields), (3, 0));
    assert_eq!(all.counters.expansions, 3 * 16);
    assert_eq!(search.counters(), all.counters);
    for served in &all.served {
        assert!(!served.via_field);
        assert_eq!(served.result.as_ref().unwrap().waypoints.len(), 2);
    }
    // One expansion of budget: the first job starts and finishes, although it needs 16; the rest wait.
    let one = serve(&nav, &mut search, &requests, 1);
    assert_eq!(ids(&one), [9]);
    assert_eq!(one.counters.expansions, 16);
    assert_eq!(one.served[0], all.served[0]);
    // The budget is spent exactly: no new job starts. 32 expansions pay for two.
    assert_eq!(ids(&serve(&nav, &mut search, &requests, 16)), [9]);
    assert_eq!(ids(&serve(&nav, &mut search, &requests, 17)), [9, 3]);
    assert_eq!(ids(&serve(&nav, &mut search, &requests, 0)), [] as [u32; 0]);
    assert_eq!(serve(&nav, &mut search, &[], PATH_BUDGET), ServeOutput::default());
}

#[test]
fn every_request_is_served_within_bounded_ticks() {
    let mut rng = Rng::from_seed(11);
    let mut map = Map::new(48, 48);
    for y in 0..48 {
        for x in 0..48 {
            if rng.below(100) < 25 {
                map.set_tile(t(x, y), Tile::new(TilePresetId::CLIFF));
            }
        }
    }
    let nav = nav(&map);
    let grid = ground(&nav);
    let passable = |rng: &mut Rng| loop {
        let tile = t(rng.below(48) as i32, rng.below(48) as i32);
        if !grid.is_blocked(tile) {
            break tile;
        }
    };
    let mut pending: Vec<PathRequest> = (0..40)
        .map(|i| request(100 - i, i / 4, passable(&mut rng), passable(&mut rng)))
        .collect();
    // Shuffled input: the order must not matter.
    pending.reverse();
    let budget = 300;
    let mut search = Search::new();
    let (mut ticks, mut deferred_once) = (0, false);
    while !pending.is_empty() {
        ticks += 1;
        let out = serve(&nav, &mut search, &pending, budget);
        // The oldest request is always served, so the queue drains.
        assert!(!out.served.is_empty());
        let mut expected: Vec<(u32, u32)> = pending.iter().map(|r| (r.since, r.id.0)).collect();
        expected.sort_unstable();
        let expected: Vec<u32> = expected.iter().map(|&(_, id)| id).take(out.served.len()).collect();
        assert_eq!(ids(&out), expected, "tick {ticks}: served out of order");
        // At most the last job runs past the budget, and a job is at most one visit of every tile.
        assert!(out.counters.expansions < budget + grid.passable_count());
        if out.served.len() < pending.len() {
            deferred_once = true;
            assert!(
                out.counters.expansions >= budget,
                "tick {ticks}: stopped with budget left"
            );
        }
        for served in &out.served {
            assert_path_clear(grid, served.result.as_ref().unwrap());
        }
        pending.retain(|r| !out.served.iter().any(|s| s.id == r.id));
    }
    assert!(deferred_once, "the budget must bite for this test to mean anything");
    assert!(ticks <= 40, "at least one request per tick");
}

/// A wall with open ground round both ends.
fn walled() -> Map {
    Map::from_ascii(
        "............
         ............
         .....c......
         .....c......
         .....c......
         .....c......
         ............
         ............",
    )
    .unwrap()
}

#[test]
fn a_group_shares_one_field() {
    let nav = nav(&walled());
    let grid = ground(&nav);
    let anchor = t(9, 3);
    let slots = [t(9, 3), t(10, 3), t(9, 4), t(10, 4), t(8, 3), t(8, 4)];
    let group: Vec<PathRequest> = (0..6)
        .map(|i| grouped(10 + i as u32, 1, anchor, t(i % 3, 2 + i / 3), slots[i as usize]))
        .collect();
    let mut search = Search::new();
    let out = serve(&nav, &mut search, &group, PATH_BUDGET);
    assert_eq!(ids(&out), [10, 11, 12, 13, 14, 15]);
    // One field and no search per member. (G) The field stops when every member's tile is expanded, so it costs
    // at most the whole component.
    assert_eq!((out.counters.fields, out.counters.searches), (1, 0));
    assert!(out.counters.expansions <= grid.passable_count());
    for (served, request) in out.served.iter().zip(&group) {
        assert!(served.via_field);
        let path = served.result.as_ref().unwrap();
        assert_eq!(path.waypoints[0], request.start);
        assert_eq!(
            *path.waypoints.last().unwrap(),
            request.goal,
            "each member ends on its own slot"
        );
        assert!(!path.retargeted);
        assert_path_clear(grid, path);
        assert!(path.waypoints.len() > 2, "round the wall");
    }
    // Fewer than GROUP_FIELD_MIN: each member searches for itself.
    let few = &group[..GROUP_FIELD_MIN - 1];
    let out = serve(&nav, &mut search, few, PATH_BUDGET);
    assert_eq!((out.counters.fields, out.counters.searches), (0, few.len() as u32));
    assert!(out.served.iter().all(|s| !s.via_field));
    // A field is one job: it starts on the oldest member with budget left and serves the whole group.
    let out = serve(&nav, &mut search, &group, 1);
    assert_eq!(ids(&out).len(), 6);
}

#[test]
fn a_field_serves_only_its_own_group_class_and_component() {
    // The east part is cut off by a full wall.
    let map = Map::from_ascii(
        "......c...
         ......c...
         ......c...
         ......c...",
    )
    .unwrap();
    let nav = nav(&map);
    let grid = ground(&nav);
    let anchor = t(4, 1);
    let mut requests: Vec<PathRequest> = (0..4)
        .map(|i| grouped(20 + i, 7, anchor, t(0, i as i32), t(4, i as i32)))
        .collect();
    // An older lone request, a member that repaths with an avoid tile, two members east of the wall, and
    // another group of two.
    requests.push(request(1, 0, t(1, 0), t(5, 3)));
    requests.push(PathRequest {
        avoid: vec![t(2, 1)],
        ..grouped(24, 7, anchor, t(1, 1), t(4, 2))
    });
    requests.push(grouped(25, 7, anchor, t(8, 0), t(4, 0)));
    requests.push(grouped(26, 7, anchor, t(8, 1), t(4, 1)));
    requests.push(grouped(30, 8, anchor, t(1, 2), t(3, 2)));
    requests.push(grouped(31, 8, anchor, t(1, 3), t(3, 3)));
    let mut search = Search::new();
    let out = serve(&nav, &mut search, &requests, PATH_BUDGET);
    assert_eq!(ids(&out), [1, 20, 21, 22, 23, 24, 25, 26, 30, 31]);
    let via_field: Vec<u32> = out.served.iter().filter(|s| s.via_field).map(|s| s.id.0).collect();
    assert_eq!(via_field, [20, 21, 22, 23]);
    assert_eq!((out.counters.fields, out.counters.searches), (1, 6));
    // The avoid tile is respected by the member that searched alone.
    assert_path_clear(grid, path_of(&out, 24));
    let alone = find_path(
        grid,
        &mut Search::new(),
        HALF,
        t(1, 1).center(),
        t(4, 2).center(),
        &[t(2, 1)],
    );
    assert_eq!(Ok(path_of(&out, 24)), alone.as_ref());
    // East of the wall the slots are unreachable: each path ends on the nearest tile of its own side.
    for id in [25, 26] {
        let path = path_of(&out, id);
        assert!(path.retargeted);
        assert_eq!(grid.tile_of(*path.waypoints.last().unwrap()).x, 7);
    }
    // Four members east of the wall share a field of their own component, anchored on its nearest tile.
    let east: Vec<PathRequest> = (0..4)
        .map(|i| grouped(40 + i, 7, anchor, t(9, i as i32), t(4, i as i32)))
        .collect();
    let out = serve(&nav, &mut search, &east, PATH_BUDGET);
    assert_eq!((out.counters.fields, out.counters.searches), (1, 0));
    assert!(out.counters.expansions <= 12, "at most the east side");
    for (i, served) in out.served.iter().enumerate() {
        let path = served.result.as_ref().unwrap();
        assert!(served.via_field && path.retargeted);
        assert_eq!(*path.waypoints.last().unwrap(), t(7, i as i32).center());
        assert_path_clear(grid, path);
    }
}

/// A member whose own destination lies behind an obstacle from the group's anchor plans alone: through the anchor,
/// its way would be a detour (`docs/movement.md > 5.5`, `> 6.4`).
#[test]
fn a_goal_the_anchor_cannot_see_is_planned_alone() {
    let nav = nav(&walled());
    let grid = ground(&nav);
    // The anchor is just east of the wall. Two members start west of the wall and go to points west of it; four
    // start in the south-east and go to points east of it.
    let anchor = t(6, 3);
    let west = [(t(1, 3), t(4, 3)), (t(1, 4), t(4, 4))];
    let east = [t(6, 3), t(7, 3), t(6, 4), t(7, 4)];
    let mut group: Vec<PathRequest> = west
        .iter()
        .enumerate()
        .map(|(i, &(start, goal))| grouped(i as u32, 3, anchor, start, goal))
        .collect();
    group.extend((0..4).map(|i| grouped(2 + i as u32, 3, anchor, t(8 + i as i32, 7), east[i])));
    for (start, goal) in west {
        assert!(
            !los(grid, HALF, anchor.center(), goal.center()),
            "the wall hides {goal:?}"
        );
        assert!(los(grid, HALF, start.center(), goal.center()));
    }
    let mut search = Search::new();
    let out = serve(&nav, &mut search, &group, PATH_BUDGET);
    assert_eq!(ids(&out), [0, 1, 2, 3, 4, 5]);
    // The western members search alone, and the others still share one field.
    assert_eq!((out.counters.fields, out.counters.searches), (1, 2));
    for (served, request) in out.served.iter().zip(&group) {
        let path = served.result.as_ref().unwrap();
        assert_eq!(served.via_field, request.id.0 >= 2);
        assert_eq!(*path.waypoints.last().unwrap(), request.goal);
        assert!(!path.retargeted);
        assert_path_clear(grid, path);
    }
    // Straight to their points, not round the wall to the anchor and back.
    for id in [0, 1] {
        let request = &group[id as usize];
        let alone = find_path(grid, &mut Search::new(), HALF, request.start, request.goal, &[]);
        assert_eq!(Ok(path_of(&out, id)), alone.as_ref());
        assert_eq!(path_of(&out, id).waypoints, [request.start, request.goal]);
    }
}

#[test]
fn requests_that_cannot_be_served_fail_without_searching() {
    let map = Map::from_ascii(
        ".....
         cc.cc
         .....",
    )
    .unwrap();
    let nav = nav(&map);
    let mut search = Search::new();
    let requests = [
        // No grid for this class.
        PathRequest {
            class: NavClass::GroundRugged,
            ..request(1, 0, t(0, 0), t(4, 2))
        },
        // No grid for this size class.
        PathRequest {
            size: 2,
            ..request(2, 0, t(0, 0), t(4, 2))
        },
        // Standing in a cliff, off the map, and nowhere at all.
        request(3, 0, t(0, 1), t(4, 2)),
        request(4, 0, t(-1, 0), t(4, 2)),
        PathRequest {
            start: Vec2::new(f32::NAN, 0.0),
            ..request(5, 0, t(0, 0), t(4, 2))
        },
        // The goal itself is avoided.
        PathRequest {
            avoid: vec![t(4, 2)],
            ..request(6, 0, t(0, 0), t(4, 2))
        },
    ];
    let out = serve(&nav, &mut search, &requests, PATH_BUDGET);
    let results: Vec<Result<Path, PathFail>> = out.served.iter().map(|s| s.result.clone()).collect();
    assert_eq!(
        results,
        [
            Err(PathFail::NoGrid),
            Err(PathFail::NoGrid),
            Err(PathFail::StartBlocked),
            Err(PathFail::StartBlocked),
            Err(PathFail::StartBlocked),
            Err(PathFail::Avoided),
        ]
    );
    assert_eq!(out.counters.expansions, 0);
    // A goal that isn't a point at all still gets a path to somewhere reachable, and never a panic.
    let wild = PathRequest {
        goal: Vec2::new(f32::INFINITY, f32::NAN),
        ..request(7, 0, t(0, 0), t(0, 0))
    };
    let out = serve(&nav, &mut search, &[wild], PATH_BUDGET);
    assert!(path_of(&out, 7).waypoints.iter().all(|p| p.is_finite()));
    // Cutting the only gap fails after a search: the one search that can.
    let cut = PathRequest {
        avoid: vec![t(2, 1)],
        ..request(8, 0, t(0, 0), t(4, 2))
    };
    let out = serve(&nav, &mut search, &[cut], PATH_BUDGET);
    assert_eq!(out.served[0].result, Err(PathFail::Avoided));
    assert_eq!((out.counters.searches, out.counters.expansions), (1, 5));
}

#[test]
fn serving_is_deterministic_and_paths_round_trip_through_serde() {
    let nav = nav(&walled());
    let anchor = t(9, 3);
    let mut requests: Vec<PathRequest> = (0..5)
        .map(|i| grouped(i as u32, 1, anchor, t(i % 3, 2 + i / 3), t(8 + i % 2, 2 + i / 2)))
        .collect();
    requests.push(PathRequest {
        avoid: vec![t(5, 1)],
        ..request(9, 0, t(0, 5), t(11, 2))
    });
    let mut search = Search::new();
    let first = serve(&nav, &mut search, &requests, PATH_BUDGET);
    // The scratch arrays carry nothing over: the same again, and the same from fresh ones.
    assert_eq!(serve(&nav, &mut search, &requests, PATH_BUDGET), first);
    assert_eq!(serve(&nav, &mut Search::new(), &requests, PATH_BUDGET), first);
    requests.reverse();
    assert_eq!(serve(&nav, &mut search, &requests, PATH_BUDGET), first);
    // Paths and requests are state: they serialize and compare.
    for served in &first.served {
        let path = served.result.as_ref().unwrap();
        let json = serde_json::to_string(path).unwrap();
        assert_eq!(&serde_json::from_str::<Path>(&json).unwrap(), path);
    }
    let json = serde_json::to_string(&requests).unwrap();
    assert_eq!(serde_json::from_str::<Vec<PathRequest>>(&json).unwrap(), requests);
}
