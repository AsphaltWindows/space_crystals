//! The navigation library alone (`docs/movement.md > 5.8`), on the bad-case map: grid builds and rebuilds,
//! A\* searches and fields, a group order and lone requests against the path budget, and a restamp.

use super::*;

const HALF: Vec2 = Vec2::splat(12.0);

/// `docs/movement.md > 5.8`: 10 % random obstacles plus six map-high walls with one gap each.
fn bad_case_map(rng: &mut Rng) -> Map {
    let mut map = Map::new(SIDE, SIDE);
    let cliff = Tile::new(TilePresetId::CLIFF);
    for y in 0..SIDE as i32 {
        for x in 0..SIDE as i32 {
            if rng.below(100) < 10 {
                map.set_tile(t(x, y), cliff);
            }
        }
    }
    for wall in 1..=6 {
        let x = wall * 36;
        let gap = rng.below(SIDE - 2) as i32;
        for y in 0..SIDE as i32 {
            // The gap is three tiles high and cleared of random obstacles on both sides.
            let tile = if (gap..gap + 3).contains(&y) {
                Tile::new(TilePresetId::PLANE)
            } else {
                cliff
            };
            map.set_tile(t(x, y), tile);
            if (gap..gap + 3).contains(&y) {
                map.set_tile(t(x - 1, y), tile);
                map.set_tile(t(x + 1, y), tile);
            }
        }
    }
    map
}

struct Row {
    case: &'static str,
    runs: u32,
    expansions: u64,
    max_expansions: u32,
    time: Duration,
}

impl Row {
    fn print(&self) {
        let micros = self.time.as_secs_f64() * 1e6 / f64::from(self.runs);
        let average = self.expansions / u64::from(self.runs);
        let per_expansion = if self.expansions == 0 {
            String::from("-")
        } else {
            format!("{:.0} ns", self.time.as_secs_f64() * 1e9 / self.expansions as f64)
        };
        println!(
            "| {:<44} | {:>5} | {:>9} | {:>9} | {:>10.1} | {:>8} |",
            self.case, self.runs, average, self.max_expansions, micros, per_expansion
        );
    }
}

/// Runs `work` once per item and sums the search's expansions and the time.
fn measure<I>(case: &'static str, search: &mut Search, items: &[I], mut work: impl FnMut(&mut Search, &I)) -> Row {
    let mut row = Row {
        case,
        runs: items.len() as u32,
        expansions: 0,
        max_expansions: 0,
        time: Duration::ZERO,
    };
    for item in items {
        search.reset_counters();
        let ((), time) = timed(|| work(search, item));
        let expansions = search.counters().expansions;
        row.expansions += u64::from(expansions);
        row.max_expansions = row.max_expansions.max(expansions);
        row.time += time;
    }
    row
}

#[test]
#[ignore = "prints timings; run in release with --ignored --nocapture"]
fn perf_256() {
    let rules = NavRules::default();
    let open = NavGrid::build(&Map::new(SIDE, SIDE), NavClass::Ground, 1, &[], rules);
    let mut rng = Rng::from_seed(2026);
    let map = bad_case_map(&mut rng);
    let keys = [
        (NavClass::Ground, 1),
        (NavClass::GroundRugged, 1),
        (NavClass::Underground, 1),
    ];
    let (mut nav, build_time) = timed(|| Nav::build(&map, &[], &keys, rules));
    let (_, rebuild_time) = timed(|| nav.rebuild(&map, &[]));
    let grid = nav.grid(NavClass::Ground, 1).unwrap();

    // Pairs inside the biggest component.
    let main = (1..=grid.component_count())
        .max_by_key(|&c| {
            (0..SIDE * SIDE)
                .filter(|&i| grid.component(grid.tile(i)) == Some(c))
                .count()
        })
        .unwrap();
    let pick = |rng: &mut Rng, near: Option<TilePos>| loop {
        let tile = match near {
            Some(p) => t(p.x - 30 + rng.below(61) as i32, p.y - 30 + rng.below(61) as i32),
            None => t(rng.below(SIDE) as i32, rng.below(SIDE) as i32),
        };
        if grid.component(tile) == Some(main) {
            break tile;
        }
    };
    let near_pairs: Vec<(TilePos, TilePos)> = (0..500)
        .map(|_| {
            let from = pick(&mut rng, None);
            (from, pick(&mut rng, Some(from)))
        })
        .collect();
    let far_pairs: Vec<(TilePos, TilePos)> = (0..500).map(|_| (pick(&mut rng, None), pick(&mut rng, None))).collect();
    // A goal walled into a pocket of its own, far from the start.
    let pocket = (0..SIDE * SIDE)
        .map(|i| grid.tile(i))
        .find(|&p| grid.component(p).is_some_and(|c| c != main) && p.x > 200)
        .expect("10 % obstacles leave some pocket");
    let corners = [(t(0, 0), t(SIDE as i32 - 1, SIDE as i32 - 1)); 200];

    let mut search = Search::new();
    // Warm the arrays up, so the first row doesn't pay for their allocation.
    search.field(grid, near_pairs[0].0);
    let mut rows = Vec::new();
    rows.push(measure(
        "Open map, corner to corner (A*)",
        &mut search,
        &corners,
        |s, &(a, b)| {
            s.astar(&open, a, b, &[]).unwrap();
        },
    ));
    assert_eq!(rows[0].max_expansions, SIDE);
    rows.push(measure(
        "Open map, corner to corner (A* + smooth)",
        &mut search,
        &corners,
        |s, &(a, b)| {
            let path = find_path(&open, s, HALF, open.center(a), open.center(b), &[]).unwrap();
            assert_eq!(path.waypoints.len(), 2);
        },
    ));
    rows.push(measure(
        "Pairs within 30 tiles (A*)",
        &mut search,
        &near_pairs,
        |s, &(a, b)| {
            s.astar(grid, a, b, &[]).unwrap();
        },
    ));
    rows.push(measure(
        "Pairs within 30 tiles (A* + smooth)",
        &mut search,
        &near_pairs,
        |s, &(a, b)| {
            find_path(grid, s, HALF, grid.center(a), grid.center(b), &[]).unwrap();
        },
    ));
    rows.push(measure("Random pairs (A*)", &mut search, &far_pairs, |s, &(a, b)| {
        s.astar(grid, a, b, &[]).unwrap();
    }));
    rows.push(measure(
        "Random pairs (A* + smooth)",
        &mut search,
        &far_pairs,
        |s, &(a, b)| {
            find_path(grid, s, HALF, grid.center(a), grid.center(b), &[]).unwrap();
        },
    ));
    let anchors: Vec<TilePos> = far_pairs.iter().take(50).map(|&(a, _)| a).collect();
    rows.push(measure("Full Dijkstra field", &mut search, &anchors, |s, &a| {
        assert!(s.field(grid, a));
    }));
    let field_row = rows.len() - 1;
    assert!(rows[field_row].max_expansions <= grid.passable_count());
    let starts: Vec<TilePos> = far_pairs.iter().map(|&(a, _)| a).collect();
    rows.push(measure(
        "Unreachable goal (retarget + A* + smooth)",
        &mut search,
        &starts,
        |s, &a| {
            let path = find_path(grid, s, HALF, grid.center(a), grid.center(pocket), &[]).unwrap();
            assert!(path.retargeted);
        },
    ));
    rows.push(measure(
        "Unreachable goal (A* alone: no search)",
        &mut search,
        &starts,
        |s, &a| {
            assert!(s.astar(grid, a, pocket, &[]).is_none());
        },
    ));
    assert_eq!(rows[rows.len() - 1].max_expansions, 0);

    // A 200-unit group order, and 200 lone requests against the budget.
    //
    // (G) The 200 units stand all over the map, so the bounding box of their silhouettes is the map and any click
    // lies inside it: by the magic box they converge, every goal the click itself (`docs/movement.md > 6.4`), and
    // one field serves them all. Slice C gave each unit a random tile within 30 tiles of the anchor, its slot
    // of the lattice that design change 0021 replaced. Since slice M only members whose goal the anchor's tile
    // sees share the field (`> 5.5`), and through 10 % obstacles it sees next to none of those tiles: the members
    // planned alone, four of them used up the budget, and this assertion failed. That layout is measured below
    // as what it is under the magic box: goals the anchor can't see, served alone within the budget.
    let anchor = far_pairs[0].1;
    let scattered: Vec<Vec2> = (0..200).map(|_| grid.center(pick(&mut rng, Some(anchor)))).collect();
    let mut requests: Vec<PathRequest> = (0..200)
        .map(|i| PathRequest {
            id: ObjectId(i),
            since: 1,
            class: NavClass::Ground,
            size: 1,
            half: HALF,
            start: grid.center(far_pairs[i as usize].0),
            goal: grid.center(anchor),
            group: Some(PathGroup {
                id: 1,
                anchor: grid.center(anchor),
            }),
            avoid: Vec::new(),
        })
        .collect();
    search.reset_counters();
    let (group, group_time) = timed(|| serve(&nav, &mut search, &requests, PATH_BUDGET));
    assert_eq!((group.served.len(), group.counters.fields), (200, 1));
    assert!(group.counters.expansions <= grid.passable_count());
    for (request, goal) in requests.iter_mut().zip(&scattered) {
        request.goal = *goal;
    }
    let (unseen, unseen_time) = timed(|| serve(&nav, &mut search, &requests, PATH_BUDGET));
    assert!(!unseen.served.is_empty() && unseen.counters.fields <= 1);
    // A job starts within the budget and always finishes: at most the budget, a field and a search.
    assert!(unseen.counters.expansions < PATH_BUDGET + 2 * grid.passable_count());
    for request in &mut requests {
        request.group = None;
    }
    let (lone, lone_time) = timed(|| serve(&nav, &mut search, &requests, PATH_BUDGET));
    assert!(lone.counters.expansions >= PATH_BUDGET && lone.served.len() < 200);

    println!("\nNavigation on {SIDE}x{SIDE}: 10 % obstacles plus six walls with one gap each");
    println!(
        "passable {} of {}, components {}",
        grid.passable_count(),
        SIDE * SIDE,
        grid.component_count()
    );
    println!(
        "| {:<44} | {:>5} | {:>9} | {:>9} | {:>10} | {:>8} |",
        "Case", "runs", "avg exp.", "max exp.", "avg us", "per exp."
    );
    for row in &rows {
        row.print();
    }
    // (M2-E) A footprint came or went on an unchanged map: every grid again, the terrain kept.
    let footprints = [TileRect::new(t(100, 100), 3, 2), TileRect::new(t(7, 200), 2, 2)];
    let mut stamped = nav.clone();
    let ((), restamp_time) = timed(|| {
        for _ in 0..50 {
            stamped.restamp(&footprints);
            stamped.restamp(&[]);
        }
    });
    assert_eq!(stamped, nav);
    println!(
        "Build of 3 grids: {build_time:.1?}; rebuild: {rebuild_time:.1?}; restamp (footprints only): {:.1?}",
        restamp_time / 100
    );
    println!(
        "serve, 200-unit group: {group_time:.1?} ({} expansions, {} fields, {} searches)",
        group.counters.expansions, group.counters.fields, group.counters.searches
    );
    println!(
        "serve, the same group with goals the anchor can't see, one tick of budget: {unseen_time:.1?} ({} served, {} \
         expansions, {} fields, {} searches)",
        unseen.served.len(),
        unseen.counters.expansions,
        unseen.counters.fields,
        unseen.counters.searches
    );
    println!(
        "serve, 200 lone requests, one tick of budget: {lone_time:.1?} ({} served, {} expansions)",
        lone.served.len(),
        lone.counters.expansions
    );
}
