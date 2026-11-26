//! The refunds of a group's Build (`factions/cults/economy.md > Build Command`, `> Cancel Construction`; design change
//! 0340, `factions/cults/economy.md > Cults Building Mechanics > Group construction refunds`): paid once as it is
//! given, it comes back once, as the last Recruit holding it drops it before its site starts, and never after the
//! start; X on the site refunds 75% of it. A failed order is refunded once in full (design change 0339,
//! `factions/cults/economy.md > Cults Building Mechanics > Group construction rules`).

use super::*;

/// Design change 0340's verification (`> Group construction refunds`: "the cost is refunded only once **no Recruit of
/// the group still holds the order**"): "Re-order 2 of 3 builders: no refund. Re-order the third: full refund." Two of
/// three re-ordered before the site starts give nothing back, and the ghost stays; the third gives the price back,
/// once, and the ghost goes.
#[test]
fn two_of_three_reordered_give_nothing_back_and_the_third_the_full_price() {
    let mut t = world(3);
    let group = recruits(3);
    t.at(2, P1, build(&group, STORAGE, SITE, false));
    t.at(8, P1, move_to(&group[..2], Vec2::new(900.0, 700.0)));
    run(&mut t, 10);
    assert_eq!(
        crystals(&t),
        BANK - STORAGE_PRICE,
        "no refund: the third still holds it"
    );
    assert_eq!((refunds(&t), t.sim().pending_builds(P1).len()), (vec![], 1));
    t.at(12, P1, move_to(&group[2..], Vec2::new(900.0, 1300.0)));
    run(&mut t, 2);
    assert_eq!(crystals(&t), BANK, "the full price");
    assert_eq!(
        refunds(&t),
        [(12, (id(4), STORAGE_PRICE))],
        "once, as the last drops it"
    );
    assert!(t.sim().pending_builds(P1).is_empty());
    run(&mut t, WALK);
    assert!(sites(&t).is_empty());
}

/// Design change 0340 (`> Group construction refunds`): "**All builders killed** before the start: **full refund**":
/// the group killed one by one before its site starts, the price comes back once, as the last dies.
#[test]
fn the_group_killed_before_the_start_is_refunded_once() {
    let mut t = world(4);
    let group = recruits(4);
    t.at(2, P1, build(&group, STORAGE, SITE, false));
    for (i, &n) in group.iter().enumerate() {
        t.debug_at(8 + i as Tick, DebugCommand::Remove { id: id(n) });
    }
    run(&mut t, 7);
    assert_eq!(crystals(&t), BANK - STORAGE_PRICE);
    run(&mut t, 4);
    assert_eq!(crystals(&t), BANK);
    assert_eq!(refunds(&t).len(), 1);
    assert!(t.sim().pending_builds(P1).is_empty());
}

/// `> Build Command`: "Once construction has started, the order can no longer be cancelled through the Recruits":
/// once the site stands, nothing comes back when its builders are re-ordered or killed; the site stays, stalled, for
/// them or others to Assist.
#[test]
fn after_the_start_nothing_comes_back() {
    let mut t = world(4);
    let group = recruits(4);
    t.at(2, P1, build(&group, STORAGE, SITE, false));
    run_to(&mut t, WALK, "the site starts", |t| !sites(t).is_empty());
    let site = sites(&t)[0];
    let now = t.sim().tick() + 1;
    t.at(now, P1, move_to(&group[..2], Vec2::new(900.0, 700.0)));
    t.debug_at(now, DebugCommand::Remove { id: id(4) });
    t.debug_at(now, DebugCommand::Remove { id: id(5) });
    run(&mut t, 20);
    assert_eq!(crystals(&t), BANK - STORAGE_PRICE);
    assert!(refunds(&t).is_empty());
    assert!(object(&t, site).is_site());
}

/// Design change 0340 (`> Group construction refunds`): "each queued building is refunded in full when its **last
/// holder drops it** before its construction starts". A Shift chain of two, the group re-ordered once the first site
/// stands: the second building, still queued, comes back in full, once, as the last of them drops it; the first,
/// started, stays paid.
#[test]
fn a_queued_building_dropped_by_its_last_holder_is_refunded_in_full() {
    let mut t = world(3);
    let group = recruits(3);
    t.at(2, P1, build(&group, STORAGE, SITE, false));
    t.at(3, P1, build(&group, STORAGE, SITE_2, true));
    run_to(&mut t, WALK, "the first site starts", |t| !sites(t).is_empty());
    let now = t.sim().tick() + 1;
    t.at(now, P1, move_to(&group[..2], Vec2::new(900.0, 700.0)));
    run(&mut t, 2);
    assert!(refunds(&t).is_empty(), "the third still holds the queued one");
    let now = t.sim().tick() + 1;
    t.at(now, P1, move_to(&group[2..], Vec2::new(900.0, 1300.0)));
    run(&mut t, 2);
    assert_eq!(
        refunds(&t),
        [(now, (id(4), STORAGE_PRICE))],
        "the queued one, in full, once"
    );
    assert_eq!(crystals(&t), BANK - STORAGE_PRICE, "the started one stays paid");
    assert!(t.sim().pending_builds(P1).is_empty());
}

/// `> Cancel Construction`: X on the site refunds 75% of its price, 75 of a Storage's 100; the site goes, and its
/// builders are idle.
#[test]
fn cancelling_the_site_refunds_75_of_100() {
    let mut t = world(4);
    let group = recruits(4);
    t.at(2, P1, build(&group, STORAGE, SITE, false));
    run_to(&mut t, WALK, "the site starts", |t| !sites(t).is_empty());
    let site = sites(&t)[0];
    run(&mut t, 20);
    let now = t.sim().tick() + 1;
    t.at(now, P1, cancel(&[site]));
    run(&mut t, 2);
    assert_eq!(crystals(&t), BANK - STORAGE_PRICE + 75);
    assert!(t.sim().object(site).is_none());
    for &n in &group {
        assert!(idle(&t, n), "#{n}");
    }
}

/// Design change 0340 (`factions/cults/economy.md > Cults Building Mechanics > Group construction refunds`): "After
/// Cancel Construction: Recruits with more orders queued (e.g. Shift-queued buildings) **go on to them**". X on the
/// first site of a Shift chain of two: the group builds the second, paid as it was given and never refunded.
#[test]
fn after_a_cancel_the_group_goes_on_to_its_queued_building() {
    let mut t = world(4);
    let group = recruits(4);
    t.at(2, P1, build(&group, STORAGE, SITE, false));
    t.at(3, P1, build(&group, STORAGE, SITE_2, true));
    run_to(&mut t, WALK, "the first site starts", |t| !sites(t).is_empty());
    let first = sites(&t)[0];
    let now = t.sim().tick() + 1;
    t.at(now, P1, cancel(&[first]));
    run_to(&mut t, WALK, "the second site starts", |t| started(t).len() == 2);
    run_to(&mut t, 600, "the second is complete", |t| !built(t).is_empty());
    assert_eq!(building_at(&t, SITE_2), Some(built(&t)[0].1));
    assert_eq!(crystals(&t), BANK - 2 * STORAGE_PRICE + 75);
    assert!(refunds(&t).is_empty());
}

/// Design change 0339 (`> Group construction rules`: "units on the footprint at that moment ... block it, and **the
/// whole order fails** (refunded once in full ...)"), the check on arrival being the truth (`core/entities.md >
/// Placement Validation > Worker-Built Structures`): a unit of player 2's standing on the footprint as the first
/// Recruit gets within reach fails the whole order, which is refunded once, and the group is idle.
#[test]
fn a_unit_on_the_site_fails_the_whole_order_refunded_once() {
    let mut t = world(4);
    let group = recruits(4);
    t.without_auto_targeting();
    t.at(2, P1, build(&group, STORAGE, SITE, false));
    t.spawn_unit_at(4, RECRUIT, P2, SITE.center());
    run_to(&mut t, WALK, "it fails", |t| !failures(t).is_empty());
    run(&mut t, 2);
    let (_, (_, problems)) = &failures(&t)[0];
    assert!(!problems.is_valid(), "{problems:?}");
    assert_eq!(failures(&t).len(), 1, "one failure for the group");
    assert_eq!(crystals(&t), BANK);
    assert_eq!(refunds(&t).len(), 1);
    assert!(sites(&t).is_empty() && t.sim().pending_builds(P1).is_empty());
    for &n in &group {
        assert!(idle(&t, n), "#{n}");
    }
}

/// Design change 0339's third verification (`> Group construction rules`: "units on the footprint at that moment,
/// **including Recruits of the same order**, block it"): "A Recruit of the order standing on the ghost at the first
/// arrival fails the order with a full refund." One Recruit of the group stands beside the footprint, at its spot,
/// the other on the middle of the footprint: as the first gets there, at once, the second is still walking off the
/// ghost, and the whole order fails, refunded once in full; nothing retries, and both are idle.
#[test]
fn a_recruit_of_the_order_on_the_ghost_at_the_first_arrival_fails_it_with_a_full_refund() {
    let mut scenario = cults();
    // #2 on the builders' wall west of the footprint, its box 4 su off; #3 on the footprint's middle.
    for at in [Vec2::new(1264.0, 960.0), SITE.center() + Vec2::splat(32.0)] {
        scenario = scenario.spawn(RECRUIT, Some(P1), SpawnAt::Point(at));
    }
    let mut t = scenario.start();
    t.debug_at(1, crystals_of(P1, BANK));
    t.at(2, P1, build(&[2, 3], STORAGE, SITE, false));
    let failed = run_to(&mut t, 20, "it fails", |t| !failures(t).is_empty());
    // The footprint's middle (1344, 960), 64 su to each edge; a Recruit's box 12 su to its edges.
    let off_middle = (object(&t, id(3)).pos - Vec2::new(1344.0, 960.0)).abs();
    assert!(
        off_middle.cmplt(Vec2::splat(64.0 + 12.0)).all(),
        "#3 still on the ghost at tick {failed}: {off_middle}"
    );
    let (_, (worker, problems)) = &failures(&t)[0];
    assert_eq!(*worker, id(2), "the first arrival");
    assert!(!problems.is_valid(), "{problems:?}");
    run(&mut t, WALK);
    assert_eq!(failures(&t).len(), 1, "one failure for the group, and no retry");
    assert_eq!(crystals(&t), BANK, "a full refund");
    assert_eq!(refunds(&t).len(), 1, "once");
    assert!(sites(&t).is_empty() && t.sim().pending_builds(P1).is_empty());
    assert!(idle(&t, 2) && idle(&t, 3));
}
