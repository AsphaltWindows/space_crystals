//! X at the Altar (`factions/cults/augmentation.md > Altar > ObjectInterfaceState[Altar]`: "**X: Cancel Production**:
//! removes the last entry from AugmentationQueue, refunds its full Space Crystals cost, and returns its reserved unit to
//! StoredUnits. Only available if AugmentationQueue is not empty.").

use super::*;

/// The brief's acceptance: "Cancel: X refunds the last order and returns its unit." Enhanced Constitution for a Recruit
/// and Both for a Militant: X takes the Both back, its 50 Space Crystals and its Militant free again
/// (`SimEvent::AugmentCancelled`); a second X the order under way, its progress lost. All 75 back, every unit free and
/// as it went in, nothing comes out; a third X has nothing to take.
#[test]
fn x_refunds_the_last_order_and_returns_its_unit() {
    let (mut t, order) = filled(&[RECRUIT, RECRUIT, MILITANT], &[]);
    let recruits = inside_of(&t, A1, RECRUIT, RecruitKind::Plain);
    let [militant] = inside_of(&t, A1, MILITANT, RecruitKind::Plain)[..] else {
        panic!("one Militant: {order:?}");
    };
    let start = t.sim().tick() + 1;
    let (enhanced, both) = (AugmentOrder::EnhancedConstitution, AugmentOrder::Both);
    t.at(start, P1, augment(&[A1], RECRUIT, RecruitKind::Plain, enhanced));
    t.at(start, P1, augment(&[A1], MILITANT, RecruitKind::Plain, both));
    run_checked(&mut t, start);
    assert_eq!(bank(&t, P1), BANK - 75);
    assert_eq!(reserved(&t, A1), [[recruits[0]], [militant]]);
    let cancelled_in = |t: &TestSim, tick| -> Vec<SimEvent> {
        (t.events().iter())
            .filter(|(at, e)| *at == tick && matches!(e, SimEvent::AugmentCancelled { .. }))
            .map(|(_, e)| e.clone())
            .collect()
    };
    t.at(start + 10, P1, cancel(&[A1]));
    run_checked(&mut t, start + 10);
    assert_eq!(bank(&t, P1), BANK - 25);
    assert_eq!(reserved(&t, A1), [[recruits[0]]]);
    assert_eq!(t.sim().free_units_of(A1, MILITANT, RecruitKind::Plain), [militant]);
    let last = SimEvent::AugmentCancelled {
        producer: A1,
        unit: militant,
        order: both,
        refund: 50,
    };
    assert_eq!(cancelled_in(&t, start + 10), [last]);
    t.at(start + 100, P1, cancel(&[A1]));
    run_checked(&mut t, start + 100);
    assert_eq!(bank(&t, P1), BANK);
    assert_eq!(reserved(&t, A1), Vec::<Vec<ObjectId>>::new());
    let under_way = SimEvent::AugmentCancelled {
        producer: A1,
        unit: recruits[0],
        order: enhanced,
        refund: 25,
    };
    assert_eq!(cancelled_in(&t, start + 100), [under_way]);
    assert_eq!(validate(&t, cancel(&[A1])), Err(Reject::NothingQueued));
    run_for(&mut t, 200);
    assert_eq!((produced(&t), inside(&t, A1)), (Vec::new(), order));
    assert!((t.sim().held_in(A1).iter()).all(|held| held.object.augmentation == Augmentation::default()));
    assert!((t.sim().held_in(A1).iter()).all(|held| held.leaving.is_none()));
}
