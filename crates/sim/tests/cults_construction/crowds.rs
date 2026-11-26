//! More Recruits than there is room for round a site (design change 0339, `factions/cults/economy.md > Cults Building
//! Mechanics > Group construction rules`: "Recruits still walking, waiting for a spot or doing anything else add
//! nothing. There is no cap other than room around the site"). The wall full and the others waiting, then moving in as
//! room frees, are the sim's own test (`sim/spot_work/tests.rs`), on a site that lasts as long as it needs: a Storage
//! is complete long before Recruits from one side have walked round to its far side.

use super::*;

/// The builders' wall round a Storage's 3 × 2 footprint (design change 0380), for a Recruit: each side's length and
/// (12 + 4) × 2 more, 2 × 224 + 2 × 160 = 768 su long, a room of 32 su for each builder, corners included
/// (`sim/spot_work.rs`).
const ROOM_ROUND: usize = 24;
/// The west side of that wall, a short side, corners included: 160 su, six rooms.
const WEST_SIDE: usize = 6;

/// Twenty-four Recruits build a Storage, coming from one side: never more than the wall holds at work at a time, and
/// at least the side they come from; those not at work add nothing, each site frame being a builder's at work
/// ([`step`]); nobody stands on the footprint (`TestSim`); the Storage is complete, paid once, and they are all free
/// for new orders.
#[test]
fn twenty_four_recruits_build_a_storage_as_many_at_once_as_there_is_room_for() {
    let mut t = world(24);
    let group = recruits(24);
    t.at(2, P1, build(&group, STORAGE, SITE, false));
    run_to(&mut t, WALK, "the site starts", |t| !sites(t).is_empty());
    let site = sites(&t)[0];
    let mut most = 0;
    while built(&t).is_empty() {
        step(&mut t);
        if t.sim().object(site).is_some_and(|o| o.is_site()) {
            most = most.max(builders(&t, site).len());
        }
        assert!(t.sim().tick() < 1000, "complete");
    }
    println!("{most} at work at most");
    assert!((WEST_SIDE..=ROOM_ROUND).contains(&most), "{most} at most");
    assert_eq!(crystals(&t), BANK - STORAGE_PRICE);
    step(&mut t);
    for &n in &group {
        assert!(idle(&t, n), "#{n}");
    }
}
