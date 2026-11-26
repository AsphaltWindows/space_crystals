//! A group builds a building from outside (`factions/cults/economy.md > Build Command`, `> Cults Building Mechanics`,
//! `> BaseConstructionTime Scaling`; design change 0072).

use super::*;

/// Design change 0072: four Recruits build a Storage, paid once as the order is given. One off each of its sides,
/// they get there together: its site starts as they get within reach of its footprint, at 10% of its MaxHP, gains four
/// frames a tick from the next, and is complete 80 ticks later (320 frames over 4). They stand round it, each on its
/// own spot, never on it, and are idle and on the map once it is complete. Coming from one side, they get there one
/// after the other, and the site gains a frame a tick for each that has.
#[test]
fn four_recruits_build_a_storage_in_about_80_frames_paid_once() {
    let mut t = four_round();
    let group = recruits(4);
    t.at(2, P1, build(&group, STORAGE, SITE, false));
    step(&mut t);
    step(&mut t);
    assert_eq!(crystals(&t), BANK - STORAGE_PRICE, "paid as the order is given");
    let ghosts = t.sim().pending_builds(P1);
    assert_eq!(ghosts.len(), 1, "one ghost for the group: {ghosts:?}");
    assert_eq!((ghosts[0].worker, ghosts[0].site.origin), (id(2), SITE));

    let start = run_to(&mut t, WALK, "the site starts", |t| !sites(t).is_empty());
    let site = sites(&t)[0];
    assert_eq!(object(&t, site).owner, Some(P1));
    assert_eq!(object(&t, site).hp, Some(25.0), "10% of 250");
    assert_eq!(started(&t).len(), 1);
    assert!(
        t.sim().pending_builds(P1).is_empty(),
        "the ghost goes as the site starts"
    );
    for &n in &group {
        assert_eq!(current(&t, n), Some(OrderKind::Assist(site)), "#{n}");
        assert!(spot(&t, n).is_some(), "#{n} has its spot");
    }
    let spots: Vec<Vec2> = group.iter().map(|&n| spot(&t, n).unwrap()).collect();
    for (i, a) in spots.iter().enumerate() {
        for b in &spots[i + 1..] {
            assert!((*a - *b).abs().max_element() >= 24.0, "each its own spot: {spots:?}");
        }
    }

    assert_eq!(builders(&t, site).len(), 4, "all four within reach as it starts");
    let done = run_to(&mut t, 200, "the Storage is complete", |t| !built(t).is_empty());
    assert_eq!(built(&t), [(done, site)]);
    assert_eq!(done - start, STORAGE_FRAMES / 4, "80 frames with four");

    let storage = object(&t, site);
    assert!(!storage.is_site() && storage.is_operational());
    assert_eq!(storage.hp, Some(250.0));
    assert_eq!(crystals(&t), BANK - STORAGE_PRICE, "paid once");
    step(&mut t);
    for &n in &group {
        assert!(idle(&t, n), "#{n} is free for new orders");
    }
}

/// `> BaseConstructionTime Scaling`: "the time to complete construction with exactly 1 Recruit assigned": one Recruit
/// builds a Storage alone, 320 frames from its arrival.
#[test]
fn a_single_recruit_builds_alone() {
    let mut t = world(1);
    t.at(2, P1, build(&[2], STORAGE, SITE, false));
    run_to(&mut t, WALK, "the site starts", |t| !sites(t).is_empty());
    let site = sites(&t)[0];
    let there = run_to(&mut t, WALK, "within reach", |t| builders(t, site).len() == 1);
    let from = frames(&t, site);
    let done = run_to(&mut t, 400, "complete", |t| !built(t).is_empty());
    assert_eq!(done - there, STORAGE_FRAMES - from);
    assert!(idle(&t, 2));
}

/// Design change 0072: "Shift with two buildings: the group builds one, then the other", each paid as it is given
/// (design change 0071): the group's second Build waits in its queue until the first building is complete, and then
/// the second site starts.
#[test]
fn a_shift_chain_of_two_is_built_one_after_the_other() {
    let mut t = world(4);
    let group = recruits(4);
    t.at(2, P1, build(&group, STORAGE, SITE, false));
    t.at(3, P1, build(&group, STORAGE, SITE_2, true));
    run(&mut t, 3);
    assert_eq!(crystals(&t), BANK - 2 * STORAGE_PRICE, "each paid as it is given");
    assert_eq!(t.sim().pending_builds(P1).len(), 2, "a ghost for each");
    let first = run_to(&mut t, 600, "the first is complete", |t| built(t).len() == 1);
    assert_eq!(building_at(&t, SITE), Some(built(&t)[0].1));
    assert_eq!(building_at(&t, SITE_2), None, "the second waits for the first");
    assert_eq!(t.sim().pending_builds(P1).len(), 1);
    let second = run_to(&mut t, WALK, "the second starts", |t| started(t).len() == 2);
    assert!(second > first);
    run_to(&mut t, 600, "the second is complete", |t| built(t).len() == 2);
    assert_eq!(building_at(&t, SITE_2), Some(built(&t)[1].1));
    assert_eq!(crystals(&t), BANK - 2 * STORAGE_PRICE, "paid once each");
    step(&mut t);
    for &n in &group {
        assert!(idle(&t, n), "#{n}");
    }
}

/// `> BaseConstructionTime Scaling`: "Construction speed scales proportionally with the number of Recruits currently
/// assigned". One of four builders pulled off, the site gains three frames a tick; all of them, it stalls with its
/// progress and its HP kept; sent back by Assist (`> Assist Construction Command`), they build it on to the end.
#[test]
fn builders_pulled_off_slow_or_stall_the_site_and_assist_brings_them_back() {
    let mut t = four_round();
    let group = recruits(4);
    t.at(2, P1, build(&group, STORAGE, SITE, false));
    run_to(&mut t, WALK, "the site starts", |t| !sites(t).is_empty());
    let site = sites(&t)[0];
    run_to(&mut t, WALK, "all four stand within reach", |t| {
        builders(t, site).len() == 4 && group.iter().all(|&n| !moving(t, n))
    });
    let now = t.sim().tick() + 1;
    t.at(now, P1, move_to(&[2], Vec2::new(900.0, 700.0)));
    step(&mut t);
    let before = frames(&t, site);
    run(&mut t, 10);
    assert_eq!(builders(&t, site).len(), 3);
    assert_eq!(frames(&t, site), before + 30, "three frames a tick");

    let now = t.sim().tick() + 1;
    t.at(now, P1, move_to(&group[1..], Vec2::new(900.0, 1300.0)));
    step(&mut t);
    let (stalled, hp) = (frames(&t, site), object(&t, site).hp);
    assert!(
        stalled < STORAGE_FRAMES && hp.is_some_and(|hp| hp > 25.0),
        "{stalled} frames, {hp:?} HP"
    );
    run(&mut t, 100);
    assert!(builders(&t, site).is_empty());
    assert_eq!(
        (frames(&t, site), object(&t, site).hp),
        (stalled, hp),
        "stalled, its progress and HP kept"
    );
    assert!(object(&t, site).is_site());

    let now = t.sim().tick() + 1;
    t.at(now, P1, assist(&group, site));
    run_to(&mut t, WALK, "they build on", |t| !builders(t, site).is_empty());
    run_to(&mut t, WALK, "complete", |t| !built(t).is_empty());
    assert_eq!(crystals(&t), BANK - STORAGE_PRICE, "an Assist costs nothing");
}

/// The right-click (`orders::resolution`): a Recruit's default order on its owner's site built from outside is an
/// Assist of it, and not on a finished building.
#[test]
fn the_right_click_on_an_own_site_is_an_assist() {
    let mut t = world(2);
    t.at(2, P1, build(&[2], STORAGE, SITE, false));
    run_to(&mut t, WALK, "the site starts", |t| !sites(t).is_empty());
    let site = sites(&t)[0];
    let on = |t: &TestSim, n: u32, target: ObjectId| t.sim().default_order(id(n), sc_sim::Target::Object(target));
    assert_eq!(on(&t, 3, site), Some(OrderKind::Assist(site)));
    assert_ne!(on(&t, 3, S1), Some(OrderKind::Assist(S1)), "a finished building");
    let now = t.sim().tick() + 1;
    t.at(now, P1, assist(&[3], site));
    run_to(&mut t, WALK, "it builds too", |t| builders(t, site).len() == 2);
}
