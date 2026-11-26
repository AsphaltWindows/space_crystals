//! Eject All and the orders (`factions/cults/structures/armory.md > Eject All`: "Recruits already reserved by a queued
//! training order are not ejected and remain committed to that order"; design change 0348 point 4,
//! `factions/cults/structures/README.md > Choosing Recruits, Release and Eject All`: "units sharing the exit doorway
//! come out **in the order they are ready**").

use super::*;

/// The plan's acceptance (slice M5-F2): "Eject All leaves reserved Recruits inside." Two of four Recruits reserved by
/// two Militants, Eject All lets the other two out and the reserved ones stay, to be trained in their turn; with only
/// reserved Recruits inside, Eject All has nothing to eject ("Only available if StoredRecruits is not empty").
#[test]
fn eject_all_leaves_the_reserved_recruits_inside() {
    let (mut t, order) = filled(4);
    let start = t.sim().tick() + 1;
    t.at(start, P1, train(&[A1], RecruitKind::Plain));
    t.at(start, P1, train(&[A1], RecruitKind::Plain));
    t.at(start + 1, P1, eject_all(&[A1]));
    run_checked(&mut t, start + 1);
    assert_eq!(reserved(&t, A1), [[order[0]], [order[1]]]);
    run_for(&mut t, 24);
    let ejected: Vec<ObjectId> = exits(&t).iter().map(|&(_, unit)| unit).collect();
    assert_eq!(ejected, order[2..]);
    assert_eq!(inside(&t, A1), order[..2]);
    assert_eq!(produced(&t), [], "still training");
    assert_eq!(validate(&t, eject_all(&[A1])), Err(Reject::NothingInside));
    run_for(&mut t, 64);
    let trained: Vec<ObjectId> = produced(&t).iter().map(|&(_, _, unit)| unit).collect();
    assert_eq!(trained, order[..2]);
    assert!(inside(&t, A1).is_empty());
}

/// Design change 0348 point 4: "Units come out in the order they are ready": a Militant's Recruit is ready once its
/// training completes, an ejected one at once (plan reading). With a Militant 6 ticks from done, Eject All lets nine
/// Recruits out one after another, all ready before it, so it comes out after the ninth, later than its training ends,
/// its entry holding the front of the queue meanwhile.
#[test]
fn a_trained_unit_waits_for_the_units_ready_before_it() {
    let (mut t, order) = filled(10);
    let start = t.sim().tick() + 1;
    t.at(start, P1, train(&[A1], RecruitKind::Plain));
    t.at(start + 25, P1, eject_all(&[A1]));
    run_checked(&mut t, start + 25);
    while produced(&t).is_empty() {
        assert!(t.sim().tick() < start + 300, "{} are out", exits(&t).len());
        step_checked(&mut t);
    }
    let ejected: Vec<ObjectId> = exits(&t).iter().map(|&(_, unit)| unit).collect();
    assert_eq!(ejected, order[1..], "every ejected one first");
    let [(trained, _, militant)] = produced(&t)[..] else {
        panic!("one Militant: {:?}", produced(&t));
    };
    assert_eq!(militant, order[0]);
    assert!(trained > start + 31, "it waited for the doorway: {trained}");
    assert!(exits(&t).iter().all(|&(tick, _)| tick < trained));
}
