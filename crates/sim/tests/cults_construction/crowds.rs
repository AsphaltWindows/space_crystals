//! More Recruits than there is room for round a site (**Placeholder (Q0097 point 8)**: "Only those within reach
//! count; others wait further out and don't count"; plan decision T62). The waiting Recruits moving in as room frees
//! are the sim's own test (`sim/spot_work/tests.rs`), on a site that lasts as long as it needs.

use super::*;

/// Ring 0 round a Storage's 2 × 2 footprint, for a Recruit: the middle of each side and a spot either side of it,
/// the corners left free (`sim/spot_work.rs`).
const RING_0: usize = 12;

/// Twenty Recruits build a Storage, coming from one side: never more than ring 0 holds build at a time, the others
/// wait further out, each site frame is a builder's within reach ([`step`]), nobody stands on the footprint
/// (`TestSim`), the Storage is complete, paid once, and they are all free for new orders.
#[test]
fn twenty_recruits_build_a_storage_twelve_at_most() {
    let mut t = world(20);
    let group = recruits(20);
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
    assert!((8..=RING_0).contains(&most), "{most} at most");
    assert_eq!(crystals(&t), BANK - STORAGE_PRICE);
    step(&mut t);
    for &n in &group {
        assert!(idle(&t, n), "#{n}");
    }
}
