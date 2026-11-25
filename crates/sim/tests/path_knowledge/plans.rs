//! What a plan goes by (`> PathKnowledge`): a unit heads for a hidden wall, sees it and re-routes (design change
//! 0040); its owner's own structures are known from the start; and it routes around a crystal field it has
//! never seen (0042).

use super::*;

/// Design change 0040, first verification: "Build a hidden wall of enemy buildings across a gap in fog. Order a unit
/// through the gap: it heads for the gap, sees the wall, and re-routes. It never avoids the gap before seeing the
/// wall." No move fails on the way.
#[test]
fn a_unit_heads_for_a_hidden_wall_sees_it_and_re_routes() {
    let mut t = ScenarioBuilder::new(&map(true))
        .spawn(ObjectKind::Generator, Some(P2), tile(WALL, GAP)) // 1: seals the gap, tiles (25, 6) to (26, 7)
        .spawn(PK, Some(P1), SpawnAt::Point(at(5.5, 7.0))) // 2
        .start();
    t.without_auto_targeting();
    let generator = (WALL, GAP, 2, 2);
    assert!(
        t.sim()
            .nav()
            .grid(NavClass::GroundRugged, 1)
            .unwrap()
            .is_blocked(TilePos::new(WALL, GAP))
    );
    assert!(
        !known_blocked(t.sim(), P1, WALL, GAP),
        "never seen: open ground for player 1"
    );

    let goal = point(EAST);
    move_to(&mut t, 2, goal);
    let mut track = Vec::new();
    // Until its owner sees the Generator, the unit walks straight at the gap, on the one straight leg it planned.
    while version(t.sim(), P1) == 0 {
        assert!(t.sim().tick() < 400);
        track.push(t.step().pos(2));
        assert_eq!(t.mover(2).path, [at(5.5, 7.0), goal], "tick {}", t.sim().tick());
        assert_eq!(t.pos(2).y, at(0.0, 7.0).y, "straight at the gap");
    }
    let seen_from = t.pos(2);
    let record = t.sim().vision().player(P1).unwrap().record(ObjectId(1));
    assert!(record.is_some(), "seen: remembered");
    // The SightRange is 5 GU to a tile's centre: the west column of the Generator, 5.5 tiles east of the unit.
    let reach = at(25.5, 6.5).distance(seen_from) / SU_PER_GU;
    assert!((4.9..=5.0).contains(&reach), "seen from {seen_from}, {reach} GU away");
    assert!(known_blocked(t.sim(), P1, WALL, GAP));
    // Within its next staggered check it plans again, around the wall's south end, and gets there.
    t.run(4);
    assert!(t.mover(2).path.len() > 2, "{:?}", t.mover(2).path);
    track.extend(walk(&mut t, 2));
    assert_eq!((t.status(2), t.pos(2)), (MoveStatus::Arrived, goal));
    assert!(went_south(&track) && !went_through_the_gap(&track));
    assert!(track.iter().all(|&p| !overlaps(p, generator)));
    assert_eq!(failed_moves(&t), 0);
    assert_eq!(
        t.mover(2).repaths_left,
        sc_sim::tuning::movement::MAX_REPATHS,
        "the re-route isn't a counted repath"
    );
}

/// `> PathKnowledge`: "Own structures are always known." The same wall of the player's own: its unit goes around
/// from the first step.
#[test]
fn a_unit_plans_around_its_owners_own_structures_from_the_start() {
    let mut t = ScenarioBuilder::new(&map(true))
        .spawn(ObjectKind::Generator, Some(P1), tile(WALL, GAP)) // 1
        .spawn(PK, Some(P1), SpawnAt::Point(at(5.5, 7.0))) // 2
        .start();
    assert!(known_blocked(t.sim(), P1, WALL, GAP));
    move_to(&mut t, 2, point(EAST));
    t.step();
    assert!(t.mover(2).path.len() > 2, "{:?}", t.mover(2).path);
    let track = walk(&mut t, 2);
    assert_eq!((t.status(2), t.pos(2)), (MoveStatus::Arrived, point(EAST)));
    assert!(went_south(&track) && !went_through_the_gap(&track));
}

/// Design change 0042, first verification: "A unit ordered across unexplored ground routes around crystal fields it
/// has never seen." A field of 18 patches down column 25 of open ground, 17 tiles from the unit: its first plan
/// already leads around the field's south end.
#[test]
fn a_unit_routes_around_a_crystal_field_it_has_never_seen() {
    let mut builder = ScenarioBuilder::new(&map(false));
    for y in 0..SOUTH {
        builder = builder.spawn(ObjectKind::CrystalPatch, None, tile(WALL, y)); // 1–18
    }
    let mut t = builder.spawn(PK, Some(P1), SpawnAt::Point(point(WEST))).start(); // 19
    let knowledge = |t: &TestSim| {
        let records = || t.sim().vision().player(P1).unwrap().records();
        (records().count(), records().filter(|r| r.seen > 0).count())
    };
    assert_eq!(knowledge(&t), (18, 0), "all known, none seen");
    assert!(
        t.sim().path_knowledge().player(P1).unwrap().assumed().is_empty(),
        "nothing assumed"
    );
    assert!(
        t.sim().vision().player(P2).unwrap().records().count() == 18,
        "a player without units knows the map too"
    );
    assert!(
        t.sim().known_nav(P2).is_none(),
        "but has no grids while nothing of its plans"
    );
    for y in 0..SOUTH {
        assert!(known_blocked(t.sim(), P1, WALL, y));
    }
    move_to(&mut t, 19, point(EAST));
    t.step();
    let path = t.mover(19).path.clone();
    assert!(
        path.len() > 2 && path.iter().any(|p| p.y > SOUTH as f32 * SU_PER_GU),
        "{path:?}"
    );
    let track = walk(&mut t, 19);
    assert_eq!((t.status(19), t.pos(19)), (MoveStatus::Arrived, point(EAST)));
    assert!(went_south(&track));
    assert!(track.iter().all(|&p| !overlaps(p, (WALL, 0, 1, SOUTH))));
    // Each patch is known from the start, and its record refreshed once seen: the unit passed the south end, and saw
    // the patches there.
    let (records, seen) = knowledge(&t);
    assert_eq!(records, 18);
    assert!(seen > 0 && seen < 18, "{seen} seen");
    assert_eq!(failed_moves(&t), 0);
}
