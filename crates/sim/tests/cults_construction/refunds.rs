//! The refunds of a group's Build (`factions/cults/economy.md > Build Command`, `> Cancel Construction`): paid once
//! as it is given, it comes back once, as the last Recruit holding it drops it before its site starts
//! (**Placeholder (Q0098 points 1 and 2)**), and never after the start (**Placeholder (Q0098 point 3)**); X on the
//! site refunds 75% of it.

use super::*;

/// Q0098 point 1: re-ordered before the site starts, every Recruit of the group: the price comes back once, as the
/// last of them drops the Build. Two of four re-ordered give nothing back, and the ghost stays.
#[test]
fn the_whole_group_reordered_before_the_start_refunds_it_once() {
    let mut t = world(4);
    let group = recruits(4);
    t.at(2, P1, build(&group, STORAGE, SITE, false));
    t.at(8, P1, move_to(&group[..2], Vec2::new(900.0, 700.0)));
    run(&mut t, 10);
    assert_eq!(crystals(&t), BANK - STORAGE_PRICE, "two still hold it");
    assert_eq!((refunds(&t), t.sim().pending_builds(P1).len()), (vec![], 1));
    t.at(12, P1, move_to(&group[2..], Vec2::new(900.0, 1300.0)));
    run(&mut t, 2);
    assert_eq!(crystals(&t), BANK);
    assert_eq!(
        refunds(&t),
        [(12, (id(5), STORAGE_PRICE))],
        "once, as the last drops it"
    );
    assert!(t.sim().pending_builds(P1).is_empty());
    run(&mut t, WALK);
    assert!(sites(&t).is_empty());
}

/// Q0098 point 2: the group killed before its site starts, one by one: the price comes back once, as the last dies.
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

/// Q0098 point 3: once the site stands, nothing comes back when its builders are re-ordered or killed; the site
/// stays, stalled, for them or others to Assist.
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

/// Design change 0340 point 4 (`factions/cults/economy.md > Group construction refunds`): "Recruits with more orders
/// queued (e.g. Shift-queued buildings) go on to them". X on the first site of a Shift chain of two: the group builds
/// the second, paid as it was given and never refunded.
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

/// **Placeholder (Q0097 point 4)**: the check on arrival is the truth (`core/entities.md > Placement Validation >
/// Worker-Built Structures`); a unit of player 2's standing on the footprint as the first Recruit gets within reach
/// fails the whole order, which is refunded once, and the group is idle.
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
