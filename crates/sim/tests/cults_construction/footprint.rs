//! Who may stand on the footprint as a group's site starts (`factions/cults/economy.md > Cults Building Mechanics >
//! Group construction rules`, slice M5-J1): design change 0362 (Q0130), "the site **waits** for Recruits of the same
//! order who stand on the footprint to step off, then starts. Only other units fail it", in place of design change
//! 0339's "including Recruits of the same order"; and 0339's rule for the others, "**the whole order fails** (refunded
//! once in full ...)", the check on arrival being the truth (`core/entities.md > Placement Validation > Worker-Built
//! Structures`).

use sc_sim::{Placement, TileProblem};

use super::*;

/// `factions/cults/units/recruit.md > Constructs`: "RecruitmentHall: 300 Space Crystals".
const HALL_PRICE: u32 = 300;

/// A Storage's footprint west of [`SITE`], touching it: tiles (18, 14) to (19, 15), x 1152..1280 and y 896..1024.
const SITE_WEST: TilePos = TilePos::new(18, 14);

/// Whether unit `n` stands on the footprint of a `kind` at `origin`, unturned, as the placement check counts a unit on
/// it (`sc_sim::deploy::on_site`).
fn on_footprint(t: &TestSim, n: u32, kind: ObjectKind, origin: TilePos) -> bool {
    let o = object(t, id(n));
    sc_sim::deploy::on_site(o.kind, o.pos, &Placement::new(origin, kind.def().footprint))
}

/// Whether unit `n`'s walk for its current Build is over (`SiteWalk::arrived`): at its spot, or stopped short of it.
fn walk_over(t: &TestSim, n: u32) -> bool {
    let current = object(t, id(n))
        .orders
        .as_ref()
        .and_then(|orders| orders.current.as_ref());
    current.is_some_and(|active| matches!(active.state, Behaviour::Build(walk) if walk.arrived))
}

/// Design change 0362's first verification: "Place a building under a group of Recruits: they walk off, and
/// construction starts with no failure." Nine Recruits stand on a Recruitment Hall's footprint, tiles (20, 14) to
/// (23, 17), in three rows of three 80 su apart round its middle. Given the Build, those nearest its edges are soon at
/// their spots beside it, and the site waits while the others still stand on it; they walk off to their own spots, and
/// the site starts, paid once and never refunded, every one of them on an Assist of it, and all nine build it.
#[test]
fn a_building_placed_under_a_group_of_recruits_starts_once_they_have_walked_off() {
    let middle = Vec2::new(1408.0, 1024.0);
    let mut scenario = cults();
    for row in [-1.0, 0.0, 1.0] {
        for column in [-1.0, 0.0, 1.0] {
            scenario = scenario.spawn(
                RECRUIT,
                Some(P1),
                SpawnAt::Point(middle + Vec2::new(column, row) * 80.0),
            );
        }
    }
    let mut t = scenario.start();
    t.debug_at(1, crystals_of(P1, BANK));
    let group = recruits(9);
    t.at(2, P1, build(&group, HALL, SITE, false));
    let mut waited = 0;
    for _ in 0..WALK {
        step(&mut t);
        if !started(&t).is_empty() {
            break;
        }
        let arrived = group.iter().any(|&n| walk_over(&t, n));
        if arrived && group.iter().any(|&n| on_footprint(&t, n, HALL, SITE)) {
            waited += 1;
        }
    }
    let Some(&(start, (site, _))) = started(&t).first() else {
        panic!("the site starts within {WALK} ticks");
    };
    assert!(
        waited > 0,
        "the first at their spots while others still stood on the footprint"
    );
    assert!(failures(&t).is_empty(), "no failure");
    assert!(refunds(&t).is_empty());
    assert_eq!(crystals(&t), BANK - HALL_PRICE, "paid once");
    for &n in &group {
        assert!(
            !on_footprint(&t, n, HALL, SITE),
            "#{n} off the footprint at tick {start}"
        );
        assert_eq!(current(&t, n), Some(OrderKind::Assist(site)), "#{n}");
    }
    run_to(&mut t, WALK, "all nine build it", |t| builders(t, site).len() == 9);
    assert!(failures(&t).is_empty());
}

/// Design change 0362 ("the site **waits** for Recruits of the same order who stand on the footprint to step off,
/// then starts"), in place of 0339's third verification ("A Recruit of the order standing on the ghost at the first
/// arrival fails the order with a full refund"): one Recruit of the group stands at its spot beside the footprint, the
/// other on its middle. The first is there at once, and the site waits, its ghost lingering and its price paid, while
/// the second walks off the footprint to its own spot; then it starts, the first arrival named, with no failure, and
/// the two build it.
#[test]
fn the_site_waits_for_a_recruit_of_the_order_to_step_off_the_footprint() {
    let mut scenario = cults();
    // #2 on the builders' wall west of the footprint, its box 4 su off; #3 on the footprint's middle.
    for at in [Vec2::new(1264.0, 960.0), SITE.center() + Vec2::splat(32.0)] {
        scenario = scenario.spawn(RECRUIT, Some(P1), SpawnAt::Point(at));
    }
    let mut t = scenario.start();
    t.debug_at(1, crystals_of(P1, BANK));
    t.at(2, P1, build(&[2, 3], STORAGE, SITE, false));
    let arrived = run_to(&mut t, 20, "#2 is at its spot", |t| walk_over(t, 2));
    assert!(
        on_footprint(&t, 3, STORAGE, SITE),
        "#3 still on the footprint at tick {arrived}"
    );
    assert!(started(&t).is_empty() && failures(&t).is_empty(), "the site waits");
    assert_eq!(t.sim().pending_builds(P1).len(), 1, "its ghost lingers");
    assert_eq!(crystals(&t), BANK - STORAGE_PRICE);
    let start = run_to(&mut t, WALK, "the site starts", |t| !started(t).is_empty());
    let (_, (site, first)) = started(&t)[0];
    assert!(start > arrived && !on_footprint(&t, 3, STORAGE, SITE));
    assert_eq!(first, id(2), "the first arrival");
    assert!(failures(&t).is_empty());
    run_to(&mut t, WALK, "both build it", |t| builders(t, site).len() == 2);
    run_to(&mut t, STORAGE_FRAMES, "it is built", |t| !built(t).is_empty());
    assert!(failures(&t).is_empty() && refunds(&t).is_empty());
    assert_eq!(crystals(&t), BANK - STORAGE_PRICE, "paid once, never refunded");
}

/// Design change 0362's second verification: "An enemy standing there fails it with a refund" ("Only other units fail
/// it"; 0339: "**the whole order fails** (refunded once in full ...)"). A Recruit of player 2's stands on the footprint
/// as the first of the group gets within reach: the whole order fails, once, its problems the tile under the enemy,
/// and is refunded once, its ghost gone and the group idle.
#[test]
fn an_enemy_on_the_site_fails_the_whole_order_refunded_once() {
    let mut t = world(4);
    let group = recruits(4);
    t.without_auto_targeting();
    t.at(2, P1, build(&group, STORAGE, SITE, false));
    t.spawn_unit_at(4, RECRUIT, P2, SITE.center());
    run_to(&mut t, WALK, "it fails", |t| !failures(t).is_empty());
    run(&mut t, 2);
    let (_, (_, problems)) = &failures(&t)[0];
    assert_eq!(problems.tiles, [(SITE, TileProblem::Unit)], "the enemy's tile alone");
    assert_eq!(failures(&t).len(), 1, "one failure for the group");
    assert_eq!(crystals(&t), BANK);
    assert_eq!(refunds(&t).len(), 1);
    assert!(sites(&t).is_empty() && t.sim().pending_builds(P1).is_empty());
    for &n in &group {
        assert!(idle(&t, n), "#{n}");
    }
}

/// Design change 0362 ("Only other units fail it"): a Recruit of player 1's that isn't on the order, standing on the
/// footprint as the first of the group gets within reach, fails the whole order as an enemy does, refunded once.
#[test]
fn an_own_recruit_not_on_the_order_fails_it_too() {
    let mut t = world(4);
    let group = recruits(4);
    t.at(2, P1, build(&group, STORAGE, SITE, false));
    // #6, idle, after the order.
    t.spawn_unit_at(4, RECRUIT, P1, SITE.center());
    run_to(&mut t, WALK, "it fails", |t| !failures(t).is_empty());
    run(&mut t, 2);
    let (_, (_, problems)) = &failures(&t)[0];
    assert_eq!(problems.tiles, [(SITE, TileProblem::Unit)], "#6's tile alone");
    assert_eq!(failures(&t).len(), 1);
    assert_eq!(crystals(&t), BANK);
    assert_eq!(refunds(&t).len(), 1);
    assert!(sites(&t).is_empty() && t.sim().pending_builds(P1).is_empty());
    assert!(group.iter().all(|&n| idle(&t, n)) && idle(&t, 6));
}

/// Design change 0362 for a Recruit that holds the order queued, behind another (**Placeholder (not in the design: a
/// Recruit that holds the order queued, behind another)**, `sim/builds/shared.rs`): it is one of the order's, so the
/// site waits for it while its current order keeps it on the footprint. #2 builds a Storage alone, at its spot west of
/// [`SITE`]; then #2 and #3 are given, with Shift, a second Storage touching the first on that side, [`SITE_WEST`],
/// whose footprint holds #2's spot. #3 takes it at once and is soon beside it, and the second site waits for #2, which
/// builds the first until it is built, then walks off to its own spot beside the second; then the second starts. No
/// failure, and each building paid once.
#[test]
fn a_recruit_holding_the_order_queued_holds_the_site_back_until_it_walks_off() {
    let mut scenario = cults();
    // #2 on the builders' wall west of SITE, its box 4 su off; #3 west of SITE_WEST.
    for at in [Vec2::new(1264.0, 960.0), Vec2::new(1000.0, 960.0)] {
        scenario = scenario.spawn(RECRUIT, Some(P1), SpawnAt::Point(at));
    }
    let mut t = scenario.start();
    t.debug_at(1, crystals_of(P1, BANK));
    t.at(2, P1, build(&[2], STORAGE, SITE, false));
    let tick = run_to(&mut t, 20, "the first site starts", |t| started(t).len() == 1);
    let (_, (first, _)) = started(&t)[0];
    t.at(tick + 1, P1, build(&[2, 3], STORAGE, SITE_WEST, true));
    let beside = run_to(&mut t, WALK, "#3 is beside the second footprint", |t| walk_over(t, 3));
    assert!(
        on_footprint(&t, 2, STORAGE, SITE_WEST),
        "#2's spot on the second footprint"
    );
    assert_eq!(current(&t, 2), Some(OrderKind::Assist(first)));
    run(&mut t, 50);
    assert_eq!(started(&t).len(), 1, "the second site waits from tick {beside}");
    assert!(failures(&t).is_empty());
    let done = run_to(&mut t, STORAGE_FRAMES, "the first is built", |t| !built(t).is_empty());
    let start = run_to(&mut t, WALK, "the second site starts", |t| started(t).len() == 2);
    let (_, (second, by)) = started(&t)[1];
    assert!(start > done && !on_footprint(&t, 2, STORAGE, SITE_WEST));
    assert_eq!(by, id(3), "the first arrival");
    assert_eq!(building_at(&t, SITE_WEST), Some(second));
    assert!(failures(&t).is_empty() && refunds(&t).is_empty());
    assert_eq!(crystals(&t), BANK - 2 * STORAGE_PRICE);
}
