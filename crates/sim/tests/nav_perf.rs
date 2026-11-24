//! Timings of the navigation library on the 256×256 cases of `docs/movement.md > 5.8`. Ignored by default:
//!
//! ```sh
//! cargo test -p sc_sim --release --test nav_perf -- --ignored --nocapture
//! ```
//!
//! The assertions are on the deterministic counters only, never on wall time.

use std::time::Duration;

use sc_sim::nav::{
    Nav, NavClass, NavGrid, NavRules, PATH_BUDGET, PathGroup, PathRequest, Search, TileRect, find_path, serve,
};
use sc_sim::{Map, ObjectId, Rng, Tile, TilePos, TilePresetId, Vec2};

const SIDE: u32 = 256;
const HALF: Vec2 = Vec2::splat(12.0);

/// Wall-clock time of `work`. Only this test reads a clock: it measures the library from outside, and nothing
/// in the sim can call it.
#[expect(
    clippy::disallowed_methods,
    reason = "a perf test has to read the clock; the sim itself never does"
)]
fn timed<T>(work: impl FnOnce() -> T) -> (T, Duration) {
    let start = std::time::Instant::now();
    let result = work();
    (result, start.elapsed())
}

fn t(x: i32, y: i32) -> TilePos {
    TilePos::new(x, y)
}

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

/// (M2-E) Path knowledge, risk R1 of the M2 plan: four players on 256 × 256, each with 100 units on the move and
/// observers around a block of ten Generators, which all go in one tick in sight of all four. That tick rebuilds
/// the true grids (phase 5b) and, at its end, each player's known grids: five sets. The target is a worst tick of
/// 5 ms in release; the assertions are on the counts only.
#[test]
#[ignore = "prints timings; run in release with --ignored --nocapture"]
fn knowledge_rebuilds_256() {
    use sc_sim::testing::TestSim;
    use sc_sim::{
        DebugCommand, MapSettings, MapType, ObjectKind, OrderKind, PlayerId, Scenario, SlotId, Spawn, SpawnAt,
        Symmetry, Target,
    };

    const DEATHS_AT: u32 = 80;
    let mut rng = Rng::from_seed(404);
    let mut map = MapSettings {
        map_type: MapType::Custom,
        width: SIDE,
        height: SIDE,
        slots: 4,
        symmetry: Symmetry::None,
        name: "knowledge perf".into(),
        description: String::new(),
    }
    .create()
    .unwrap();
    // 4 % scattered cliffs, away from the middle and from the four armies' corners.
    let clear = |x: i32, y: i32| {
        let middle = (112..144).contains(&x) && (112..144).contains(&y);
        let corner = !(44..212).contains(&x) && !(44..212).contains(&y);
        middle || corner
    };
    for y in 0..SIDE as i32 {
        for x in 0..SIDE as i32 {
            if !clear(x, y) && rng.below(100) < 4 {
                map.terrain.set_tile(t(x, y), Tile::new(TilePresetId::CLIFF));
            }
        }
    }
    // 160 crystal patches in the clear corners: what every player knows of the map from the start.
    for i in 0..160 {
        let (cx, cy) = [(4, 40), (216, 40), (4, 250), (216, 250)][i % 4];
        map.spawns.push(Spawn::new(
            ObjectKind::CrystalPatch,
            None,
            SpawnAt::Tile(t(cx + (i as i32 / 4) % 20 * 2, cy + (i as i32 / 80) * 2)),
        ));
    }
    // Ten Generators of player 1 in a block of five by two: tiles (124, 126) to (133, 129).
    for i in 0..10 {
        map.spawns.push(Spawn::new(
            ObjectKind::Generator,
            Some(SlotId(1)),
            SpawnAt::Tile(t(124 + (i % 5) * 2, 126 + (i / 5) * 2)),
        ));
    }
    let first_generator = 161;
    let mut sim = TestSim::new(Scenario::new(map));
    sim.check_invariants(false);
    sim.without_auto_targeting();
    // Three observers per player along the block's north or south side: each sees all ten.
    let pk = ObjectKind::Peacekeeper;
    for player in 1..=4u8 {
        let row = if player % 2 == 1 { 124.5 } else { 131.5 };
        for i in 0..3 {
            let x = 125.5 + 4.0 * i as f32 + f32::from(player / 3);
            sim.spawn_unit_at(1, pk, PlayerId(player), Vec2::new(x, row) * 64.0);
        }
    }
    // 100 units per player in the corners, sent across the map in one group order each.
    let mut armies: Vec<Vec<ObjectId>> = vec![Vec::new(); 4];
    for i in 0..400u32 {
        let player = (i % 4) as usize;
        let (cx, cy) = [(8, 8), (224, 8), (8, 224), (224, 224)][player];
        let n = i / 4;
        let at = Vec2::new((cx + (n % 10) as i32) as f32 + 0.5, (cy + (n / 10) as i32) as f32 + 0.5) * 64.0;
        sim.spawn_unit_at(1, pk, PlayerId(player as u8 + 1), at);
        armies[player].push(ObjectId(first_generator + 10 + 12 + i));
    }
    for (player, army) in armies.iter().enumerate() {
        let (gx, gy) = [(230.0, 230.0), (12.0, 230.0), (230.0, 12.0), (12.0, 12.0)][player];
        let order = OrderKind::Move(Target::Point(Vec2::new(gx, gy) * 64.0));
        // One army per tick, so no tick pays for two fields.
        sim.order_at(4 + 4 * player as u32, PlayerId(player as u8 + 1), army, order, false);
    }
    for i in 0..10 {
        sim.debug_at(
            DEATHS_AT,
            DebugCommand::Remove {
                id: ObjectId(first_generator + i),
            },
        );
    }

    let versions = |sim: &TestSim| -> Vec<u32> {
        (1..=4)
            .map(|p| sim.sim().path_knowledge().version(PlayerId(p)).unwrap())
            .collect()
    };
    let mut times: Vec<(u32, Duration, u32)> = Vec::new();
    let mut before = Vec::new();
    for tick in 1..=DEATHS_AT + 60 {
        if tick == DEATHS_AT {
            before = versions(&sim);
            let records = |p: u8| sim.sim().vision().player(PlayerId(p)).unwrap().records().count();
            assert_eq!(
                [records(2), records(3), records(4)],
                [10; 3],
                "all ten seen by the others"
            );
        }
        let ((), time) = timed(|| {
            sim.step();
        });
        times.push((tick, time, sim.sim().nav_rebuilds()));
    }
    assert_eq!(sim.sim().objects().filter(|o| o.mover.is_some()).count(), 412);
    let (_, death_time, rebuilds) = times[DEATHS_AT as usize - 1];
    assert_eq!(rebuilds, 5, "the true grids and each of the four players'");
    let after = versions(&sim);
    assert!(
        before.iter().zip(&after).all(|(b, a)| a == &(b + 1)),
        "one change of knowledge for each player: {before:?} to {after:?}"
    );
    let moving = sim
        .sim()
        .objects()
        .filter(|o| o.mover.as_ref().is_some_and(|m| m.status == sc_sim::MoveStatus::Moving))
        .count();
    assert!(moving >= 380, "the armies are still on their way: {moving}");

    let worst = |range: std::ops::RangeInclusive<u32>| {
        times
            .iter()
            .filter(|(tick, ..)| range.contains(tick))
            .max_by_key(|&&(_, time, _)| time)
            .copied()
            .unwrap()
    };
    let mean = |range: std::ops::RangeInclusive<u32>| {
        let in_range: Vec<Duration> = times
            .iter()
            .filter(|(tick, ..)| range.contains(tick))
            .map(|&(_, time, _)| time)
            .collect();
        in_range.iter().sum::<Duration>() / in_range.len() as u32
    };
    let grids = sim.sim().nav().grids().len();
    println!(
        "\nPath knowledge on {SIDE}x{SIDE}: 4 players, 412 units (400 moving), 160 patches, {grids} grid(s) per set"
    );
    println!(
        "ticks {}..{} (before the deaths): mean {:.3?}, worst {:.3?}",
        DEATHS_AT - 40,
        DEATHS_AT - 1,
        mean(DEATHS_AT - 40..=DEATHS_AT - 1),
        worst(DEATHS_AT - 40..=DEATHS_AT - 1).1
    );
    println!(
        "tick {DEATHS_AT} (10 Generators gone, seen by all 4 players): {death_time:.3?}, {rebuilds} sets of grids rebuilt"
    );
    let (tick, time, _) = worst(DEATHS_AT + 1..=DEATHS_AT + 60);
    println!(
        "ticks {}..{} (the units' path checks after it): mean {:.3?}, worst {time:.3?} at tick {tick}",
        DEATHS_AT + 1,
        DEATHS_AT + 60,
        mean(DEATHS_AT + 1..=DEATHS_AT + 60)
    );
    let (tick, time, _) = worst(1..=DEATHS_AT + 60);
    println!("worst tick of the whole run: {time:.3?} at tick {tick} (ticks 4, 8, 12 and 16 are the group orders)");
}
