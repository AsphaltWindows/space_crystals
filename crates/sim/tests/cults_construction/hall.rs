//! What the Recruits build works as any other of its kind: a Recruitment Hall claims its tiles and makes Recruits of
//! its own lineage (`factions/cults/structures/recruitment_hall.md`, slice M5-C), at the map's edge too, at the
//! Effectiveness the edge leaves it (slice M5-S4, design change 0387); a Storage takes the loads (slice M5-D); and
//! gatherers taken off to build and sent back keep gathering by M5-D's rules.

use sc_sim::{ProductionCommand, Target};

use super::*;

/// The Recruits `hall` made so far: `(tick, unit)`.
fn made_by(t: &TestSim, hall: ObjectId) -> Vec<(Tick, ObjectId)> {
    events(t, |e| match *e {
        SimEvent::UnitProduced { producer, unit } if producer == hall => Some(unit),
        _ => None,
    })
}

/// Design change 0387's Verification: "Place a Hall flush against the map edge: it is accepted", and "the built Hall's
/// capacity and production follow it" (`factions/cults/structures/recruitment_hall.md > RecruitmentArea`: "The part of
/// its area that lies **off the map yields no Recruitable tiles**"; `factions/cults/structures/recruitment_hall.md >
/// Effectiveness`: "The total is always the full 10x10 = 100, **off-map tiles included**"). Four Recruits build a Hall
/// flush against the north edge, and four others one in the north-east corner, both edges cutting its area; the exit
/// of each, side B, faces west, on the map, and its rally point is in the south. Complete, the first claims the 70
/// tiles of its area on the map, 70%: a capacity of 14 (20 × 70%), and 70 tiles a frame, a Recruit each 275 ticks (192
/// × 100 / 70, rounded up); the corner one claims 49, 49%: a capacity of 9 (9.8 rounded down, design change 0341 point
/// 2), a Recruit each 392 ticks. Each makes as many Recruits as its capacity, of its lineage, and then none.
#[test]
fn halls_built_at_the_map_edge_follow_their_effectiveness() {
    let mut t = world(8);
    let (edge, corner) = (TilePos::new(20, 0), TilePos::new(36, 0));
    t.at(2, P1, build(&[2, 3, 4, 5], HALL, edge, false));
    t.at(2, P1, build(&[6, 7, 8, 9], HALL, corner, false));
    run(&mut t, 2);
    assert_eq!(crystals(&t), BANK - 600, "both Builds taken");
    run_to(&mut t, 1500, "both Halls are complete", |t| built(t).len() == 2);
    let hall_at = |t: &TestSim, origin| building_at(t, origin).expect("built");
    let (edge_hall, corner_hall) = (hall_at(&t, edge), hall_at(&t, corner));
    let now = t.sim().tick() + 1;
    let rally = |hall: ObjectId, x: f32| Command::Production {
        objects: vec![hall],
        command: ProductionCommand::SetRally(Target::Point(Vec2::new(x, 1600.0))),
    };
    t.at(now, P1, rally(edge_hall, 1300.0));
    t.at(now, P1, rally(corner_hall, 2200.0));
    t.at(now, P1, move_to(&recruits(8), Vec2::new(600.0, 1600.0)));
    run(&mut t, 2);
    for (hall, claimed, capacity) in [(edge_hall, 70, 14), (corner_hall, 49, 9)] {
        let state = t.sim().hall_state(hall).expect("a Hall");
        assert_eq!(
            (state.claimed, state.tiles, state.effectiveness_percent()),
            (claimed, 100, claimed),
            "{hall:?}"
        );
        assert_eq!(state.capacity.floor(), capacity, "{hall:?}");
        let tiles = t.sim().claimed_tiles(hall);
        assert_eq!(tiles.len(), claimed as usize, "{hall:?}");
        assert!(
            tiles.iter().all(|&tile| t.sim().map().in_bounds(tile)),
            "{hall:?}: on the map"
        );
    }
    run_to(&mut t, 15 * 275, "the edge Hall makes 14", |t| {
        made_by(t, edge_hall).len() == 14
    });
    run(&mut t, 2 * 392);
    for (hall, every, capacity) in [(edge_hall, 275, 14), (corner_hall, 392, 9)] {
        let made = made_by(&t, hall);
        assert_eq!(made.len(), capacity, "{hall:?}: as many as its capacity, then none");
        let ticks: Vec<Tick> = made.iter().map(|&(tick, _)| tick).collect();
        assert!(ticks.windows(2).all(|w| w[1] - w[0] == every), "{hall:?}: {ticks:?}");
        for &(_, unit) in &made {
            assert_eq!(object(&t, unit).lineage, [hall]);
        }
        let state = t.sim().hall_state(hall).expect("a Hall");
        assert_eq!(state.usage, capacity as u32, "{hall:?}");
        assert_eq!(t.sim().production_status(hall), None, "{hall:?}: at capacity");
    }
}

/// Four Recruits build a Recruitment Hall (300 Space Crystals, 1440 frames with one): once complete it claims the
/// tiles of its area, and makes Recruits whose lineage is that Hall (plan decision T56).
#[test]
fn a_hall_the_recruits_build_makes_recruits_of_its_lineage() {
    let mut t = world(4);
    t.at(2, P1, build(&recruits(4), HALL, SITE, false));
    run(&mut t, 2);
    assert_eq!(crystals(&t), BANK - 300);
    let done = run_to(&mut t, 1000, "the Hall is complete", |t| !built(t).is_empty());
    let hall = built(&t)[0].1;
    assert_eq!(object(&t, hall).kind, HALL);
    step(&mut t);
    let state = t.sim().hall_state(hall).expect("a Hall");
    assert!(state.claimed > 0 && state.claimed == state.tiles, "{state:?}");
    let made = |t: &TestSim| {
        events(t, |e| match *e {
            SimEvent::UnitProduced { producer, unit } if producer == hall => Some(unit),
            _ => None,
        })
    };
    run_to(&mut t, 400, "the Hall makes a Recruit", |t| !made(t).is_empty());
    let (tick, recruit) = made(&t)[0];
    println!("complete at {done}, first Recruit at {tick}");
    assert_eq!(object(&t, recruit).kind, RECRUIT);
    assert_eq!(object(&t, recruit).lineage, [hall]);
}

/// Four Recruits gathering from a patch for Storage 1 are taken off by a Build of a Storage nearer to the patch, and
/// sent back by a Gather once it is complete: they gather on by slice M5-D's rules, each load to the complete Storage
/// nearest to the Recruit (design changes 0345 and 0375), now the one they built, on the side of the patch they come
/// back to; the loads they carried while building are delivered, not lost.
#[test]
fn gatherers_taken_off_to_build_and_sent_back_gather_on() {
    // Storage 1 on tiles (2, 2) to (4, 3); the patch 2 on (7, 2), 128 su from it; the Recruits 3 to 6 beside the
    // patch; the new Storage on (9, 2) to (11, 3), 64 su from the patch (design change 0380: each 3x2).
    let mut scenario = cults().spawn_with(
        sc_sim::SpawnSettings::CrystalPatch {
            starting_amount: 1_000_000,
        },
        tile(7, 2),
    );
    for i in 0..4 {
        scenario = scenario.spawn(
            RECRUIT,
            Some(P1),
            SpawnAt::Point(Vec2::new(400.0 + 32.0 * i as f32, 300.0)),
        );
    }
    let mut t = scenario.start();
    t.debug_at(1, crystals_of(P1, BANK));
    let (patch, group) = (id(2), [3, 4, 5, 6]);
    let gather = |units: &[u32]| order(units, OrderKind::Gather(patch), false);
    t.at(2, P1, gather(&group));
    let deliveries = |t: &TestSim| {
        events(t, |e| match *e {
            SimEvent::ResourcesDelivered {
                worker,
                drop_off,
                amount,
                ..
            } => Some((worker, drop_off, amount)),
            _ => None,
        })
    };
    run_to(&mut t, 600, "each delivers", |t| deliveries(t).len() >= 8);
    let new = TilePos::new(9, 2);
    let now = t.sim().tick() + 1;
    let bank = crystals(&t);
    t.at(now, P1, build(&group, STORAGE, new, false));
    run(&mut t, 2);
    assert_eq!(crystals(&t), bank - STORAGE_PRICE);
    let carried: u32 = group
        .iter()
        .map(|&n| object(&t, id(n)).worker.expect("a worker").load)
        .sum();
    run_to(&mut t, 600, "the Storage is complete", |t| !built(t).is_empty());
    let storage = built(&t)[0].1;
    let back = t.sim().tick() + 1;
    let after_build = deliveries(&t).len();
    let loads: u32 = group
        .iter()
        .map(|&n| object(&t, id(n)).worker.expect("a worker").load)
        .sum();
    assert_eq!(loads, carried, "the loads are kept while building");
    t.at(back, P1, gather(&group));
    run_to(&mut t, 600, "each delivers again", |t| {
        deliveries(t).len() >= after_build + 8
    });
    let since: Vec<(Tick, (ObjectId, ObjectId, u32))> = deliveries(&t).split_off(after_build);
    for &n in &group {
        assert!(
            since.iter().any(|&(_, (worker, _, _))| worker == id(n)),
            "#{n} gathers on"
        );
    }
    assert!(
        since.iter().all(|&(_, (_, at, _))| at == storage),
        "to the nearest Storage: {since:?}"
    );
}
