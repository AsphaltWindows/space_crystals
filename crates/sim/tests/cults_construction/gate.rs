//! The gate's checks of a group's Build and of an Assist (`sim/gate/orders.rs`), the ghosts, and a Recruitment Hall
//! flush against the map's edge, which slice M5-J1's rule refused (slice M5-S4, design change 0387).

use super::*;

/// Four Recruits of player 1's, ids 1 to 4, where [`start_of`] puts them, on [`cults`]' map with no Storage, and a
/// bank of [`BANK`] Space Crystals.
fn without_storage() -> TestSim {
    let mut scenario = ScenarioBuilder::new(&flat(40, 30))
        .faction(P1, Faction::Cults)
        .faction(P2, Faction::Cults);
    for i in 0..4 {
        scenario = scenario.spawn(RECRUIT, Some(P1), SpawnAt::Point(start_of(i, 4)));
    }
    let mut t = scenario.start();
    t.debug_at(1, crystals_of(P1, BANK));
    step(&mut t);
    step(&mut t);
    t
}

/// `factions/cults/structures/armory.md > TechPrerequisite - player owns at least one Storage`: the gate refuses an
/// Armory (`Reject::Requires`) while its owner has no complete Storage, a Storage under construction included, and
/// takes it once one is complete; the Recruits then build it.
#[test]
fn the_armory_needs_a_complete_storage() {
    let mut t = without_storage();
    let group = [1, 2, 3, 4];
    let armory = build(&group, ARMORY, SITE_3, false);
    assert_eq!(gate(&t, P1, armory.clone()), Err(Reject::Requires(STORAGE)));
    let now = t.sim().tick() + 1;
    t.at(now, P1, build(&group, STORAGE, SITE, false));
    run_to(&mut t, WALK, "the Storage's site starts", |t| !sites(t).is_empty());
    assert_eq!(
        gate(&t, P1, armory.clone()),
        Err(Reject::Requires(STORAGE)),
        "a site is no Storage yet"
    );
    run_to(&mut t, 400, "the Storage is complete", |t| !built(t).is_empty());
    assert_eq!(gate(&t, P1, armory.clone()), Ok(()));
    let now = t.sim().tick() + 1;
    t.at(now, P1, armory);
    run_to(&mut t, 800, "the Armory is complete", |t| built(t).len() == 2);
    assert_eq!(object(&t, built(&t)[1].1).kind, ARMORY);
    assert_eq!(crystals(&t), BANK - STORAGE_PRICE - 200);
}

/// Design change 0071: three Shift-clicks place three ghosts, each paid as it is given, and the gate refuses a
/// fourth that the bank can't pay (`Reject::InsufficientCrystals`).
#[test]
fn each_shift_click_is_paid_as_it_is_given() {
    let mut t = world(4);
    let group = recruits(4);
    t.at(2, P1, build(&group, STORAGE, SITE, false));
    t.at(3, P1, build(&group, STORAGE, SITE_2, true));
    t.at(4, P1, build(&group, STORAGE, SITE_3, true));
    t.debug_at(5, crystals_of(P1, 50));
    run(&mut t, 5);
    let ghosts: Vec<TilePos> = t
        .sim()
        .pending_builds(P1)
        .iter()
        .map(|ghost| ghost.site.origin)
        .collect();
    assert_eq!(ghosts, [SITE, SITE_2, SITE_3]);
    assert_eq!(
        gate(&t, P1, build(&group, STORAGE, TilePos::new(28, 14), true)),
        Err(Reject::InsufficientCrystals)
    );
    for &n in &group {
        assert_eq!(queued(&t, n).len(), 2, "#{n} holds the other two in its queue");
    }
}

/// An Assist is of a site of its owner's built from outside (`Reject::NotASite`: not of a finished building), and
/// only by the units that build its kind (`Reject::NoWorkers`).
#[test]
fn an_assist_is_of_an_own_site_by_its_builders() {
    let mut t = world(1);
    t.spawn_unit_at(1, ObjectKind::Militant, P1, Vec2::new(1100.0, 1100.0));
    t.at(2, P1, build(&[2], STORAGE, SITE, false));
    run_to(&mut t, WALK, "the site starts", |t| !sites(t).is_empty());
    let site = sites(&t)[0];
    let militant = (t.sim().objects())
        .find(|o| o.kind == ObjectKind::Militant)
        .expect("spawned")
        .id;
    assert_eq!(gate(&t, P1, assist(&[2], S1)), Err(Reject::NotASite(S1)));
    assert_eq!(gate(&t, P1, assist(&[militant.0], site)), Err(Reject::NoWorkers));
    assert_eq!(gate(&t, P1, assist(&[2, militant.0], site)), Ok(()));
}

/// Design change 0387's Verification: "Place a Hall flush against the map edge: it is accepted"
/// (`factions/cults/structures/recruitment_hall.md > RecruitmentArea`: "a Hall can be placed anywhere its 4x4 footprint
/// fits on the map, including right at the edge"), where slice M5-J1's gate refused a Hall 2 tiles from it (design
/// change 0341 point 1). The gate takes the Build of a Recruitment Hall flush against the north edge, of one in the
/// north-east corner, both edges cutting its area, and of one 2 tiles from the edge, and the site check of their ghost
/// finds nothing wrong with any of them.
#[test]
fn the_gate_takes_a_hall_flush_against_the_map_edge() {
    let mut t = world(4);
    run(&mut t, 2);
    let group = recruits(4);
    let workers: Vec<ObjectId> = group.iter().map(|&n| id(n)).collect();
    for origin in [TilePos::new(20, 0), TilePos::new(36, 0), TilePos::new(20, 2)] {
        assert_eq!(gate(&t, P1, build(&group, HALL, origin, false)), Ok(()), "{origin:?}");
        let check = (t.sim())
            .group_build_check(&workers, HALL, origin, BuildingOrientation::default(), false)
            .expect("Recruits build Halls");
        assert!(check.is_valid(), "{origin:?}: {check:?}");
    }
}
