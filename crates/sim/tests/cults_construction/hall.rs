//! What the Recruits build works as any other of its kind: a Recruitment Hall claims its tiles and makes Recruits of
//! its own lineage (`factions/cults/structures/recruitment_hall.md`, slice M5-C), a Storage takes the loads (slice
//! M5-D); and gatherers taken off to build and sent back keep gathering by M5-D's rules.

use super::*;

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
/// sent back by a Gather once it is complete: they gather on by slice M5-D's rules, each load to the nearest
/// complete Storage (**Placeholder (Q0102 points 1 and 2)**), now the one they built; the loads they carried while
/// building are delivered, not lost.
#[test]
fn gatherers_taken_off_to_build_and_sent_back_gather_on() {
    // Storage 1 on tiles (2, 2) to (3, 3); the patch 2 on (7, 2), 192 su from it; the Recruits 3 to 6 beside the
    // patch; the new Storage on (10, 2) to (11, 3), 128 su from the patch.
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
    let new = TilePos::new(10, 2);
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
