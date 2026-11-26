//! Path knowledge in a game (M2-E, risk R1 of the M2 plan): the tick in which ten Generators go in sight of
//! four players rebuilds the true grids and each player's known grids.

use super::*;

/// (M2-E) Path knowledge, risk R1 of the M2 plan: four players on 256 × 256, each with 100 units on the move and
/// observers around a block of ten Generators, which all go in one tick in sight of all four. That tick rebuilds
/// the true grids (phase 5b) and, at its end, each player's known grids: five sets. The target is a worst tick of
/// 5 ms in release; the assertions are on the counts only.
#[test]
#[ignore = "prints timings; run in release with --ignored --nocapture"]
fn knowledge_rebuilds_256() {
    knowledge_rebuilds(false);
}

/// (M5-Z2) The same with a vehicle in the game, player 1's idle Seeker, a wheeled vehicle, which doesn't cross Rugged
/// terrain: each set then holds its grid, `(Ground, 1)`, beside the infantry's, and the tick of the deaths rebuilds
/// both in each of the five.
#[test]
#[ignore = "prints timings; run in release with --ignored --nocapture"]
fn knowledge_rebuilds_256_with_a_vehicle() {
    knowledge_rebuilds(true);
}

fn knowledge_rebuilds(vehicle: bool) {
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
    // And one out of everyone else's way, which keeps player 1 in the game when the ten go (`core/game_rules.md >
    // Defeat Condition`): a defeated player's units become nobody's, and a player without units has no grids to
    // rebuild (slice M2-J2).
    map.spawns.push(Spawn::new(
        ObjectKind::Generator,
        Some(SlotId(1)),
        SpawnAt::Tile(t(30, 4)),
    ));
    let first_generator = 161;
    let first_unit = first_generator + 11;
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
        armies[player].push(ObjectId(first_unit + 12 + i));
    }
    // Out of everyone's way, in player 1's corner.
    if vehicle {
        sim.spawn_unit_at(1, ObjectKind::Seeker, PlayerId(1), Vec2::new(36.5, 20.5) * 64.0);
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
            // The Generators' records: each player also has those of the 160 patches, known from the start (design
            // change 0159).
            let records = |p: u8| {
                let sight = sim.sim().vision().player(PlayerId(p)).unwrap();
                sight.records().filter(|r| r.kind == ObjectKind::Generator).count()
            };
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
    let units = 412 + usize::from(vehicle);
    assert_eq!(sim.sim().objects().filter(|o| o.mover.is_some()).count(), units);
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
        "\nPath knowledge on {SIDE}x{SIDE}: 4 players, {units} units (400 moving), 160 patches, {grids} grid(s) per set"
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
