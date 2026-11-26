//! A research's cost (`core/tech_and_research.md > Research > Shared Rules`): "2. **Cost and time**: each technology
//! has a resource cost and a research time. The cost is deducted when the technology is queued"; "6. **Cancel**:
//! cancelling a queued or in-progress technology refunds its full cost (standard **X: Cancel Upgrade** slot — removes
//! the last entry in the research queue)"; "7. **Destruction**: if a research structure is destroyed, all of its
//! in-progress and queued research is cancelled, with a full refund of each — the same as cancelling"; and the staff
//! of a Temple destroyed come out (`factions/cults/economy.md > Cults Building Mechanics > Evacuation on Destruction`).
//! One queue engine (plan decision T70): the research is an entry of the Temple's production queue.

use super::*;

/// The plan's acceptance (the M5 addendum's N3): "100 SC at queue, refunded in full by X (the last entry)". Both lines'
/// Level 1s queued, 200 Space Crystals paid; X takes the last one off, Improved Metabolism, waiting, and gives its 100
/// back; a second X takes off Enhanced Constitution, in progress for 11 ticks, and gives its 100 back too, its progress
/// gone. With nothing queued X is refused (`Reject::NothingQueued`).
#[test]
fn x_takes_the_last_research_off_and_refunds_it_in_full() {
    let (mut t, _) = staffed(1);
    let start = t.sim().tick() + 1;
    t.at(start, P1, research(&[T1], EC1));
    t.at(start, P1, research(&[T1], IM1));
    run_checked(&mut t, start);
    assert_eq!(queue(&t, T1), [EC1, IM1], "both lines' Level 1s");
    assert_eq!(bank(&t, P1), BANK - 2 * L1_PRICE);
    t.at(start + 10, P1, cancel(&[T1]));
    run_checked(&mut t, start + 10);
    assert_eq!(cancelled(&t), [(start + 10, T1, IM1, L1_PRICE)], "the last one");
    assert_eq!(queue(&t, T1), [EC1]);
    assert_eq!(bank(&t, P1), BANK - L1_PRICE);
    assert_eq!(progress(&t, T1), (11, true), "the first goes on");
    t.at(start + 11, P1, cancel(&[T1]));
    run_checked(&mut t, start + 11);
    assert_eq!(
        cancelled(&t)[1..],
        [(start + 11, T1, EC1, L1_PRICE)],
        "in progress, in full"
    );
    assert_eq!(queue(&t, T1), []);
    assert_eq!(bank(&t, P1), BANK);
    assert_eq!(t.sim().production_status(T1), None);
    assert_eq!(validate(&t, cancel(&[T1])), Err(Reject::NothingQueued));
    run_for(&mut t, L1_FRAMES);
    assert_eq!(completed(&t), [], "nothing completes");
    assert_eq!(techs(&t, P1), []);
}

/// Shared Rules 7 and the evacuation: the Temple destroyed 50 ticks into Enhanced Constitution Level 1, with Improved
/// Metabolism Level 1 behind it, gives both back in full (`SimEvent::ResearchCancelled`), and its three Recruits come
/// out onto the map around it (`SimEvent::Evacuated`), player 1's still. Nothing was completed.
#[test]
fn a_temple_destroyed_refunds_its_research_and_lets_its_staff_out() {
    let (mut t, order) = staffed(3);
    let start = t.sim().tick() + 1;
    t.at(start, P1, research(&[T1], EC1));
    t.at(start, P1, research(&[T1], IM1));
    remove_at(&mut t, start + 50, T1);
    run_checked(&mut t, start + 50);
    assert!(t.sim().object(T1).is_none(), "destroyed");
    let mut refunded = cancelled(&t);
    refunded.sort_by_key(|&(.., tech, _)| tech);
    assert_eq!(
        refunded,
        [(start + 50, T1, EC1, L1_PRICE), (start + 50, T1, IM1, L1_PRICE)]
    );
    assert_eq!(bank(&t, P1), BANK, "both in full");
    let evacuated: Vec<ObjectId> = (t.events().iter())
        .filter_map(|(_, e)| match *e {
            SimEvent::Evacuated { unit, building } if building == T1 => Some(unit),
            _ => None,
        })
        .collect();
    let mut sorted = evacuated.clone();
    sorted.sort();
    let mut staff = order.clone();
    staff.sort();
    assert_eq!(sorted, staff, "all three out");
    for unit in order {
        let o = t.sim().object(unit).expect("on the map");
        assert_eq!((o.kind, o.owner), (RECRUIT, Some(P1)));
    }
    assert_eq!(techs(&t, P1), []);
    assert_eq!(completed(&t), []);
}

/// The gate's refusals of a research (`Sim::plan_research`): a structure of the group that doesn't research it
/// (`Reject::CannotResearch`: an Armory), one that produces nothing (`Reject::NoProducers`: the Storage), and a bank
/// short of its cost (`Reject::InsufficientCrystals`), which Shared Rules 2 deducts at queue. With 100 the Temple takes
/// it, and the bank is empty.
#[test]
fn a_research_is_refused_where_it_isnt_offered_or_cant_be_paid() {
    let armory = id(3);
    let scenario = base().spawn(ARMORY, Some(P1), tile(20, 14)); // 3
    let (mut t, _) = staffed_with(scenario, &[RecruitKind::Plain]);
    assert_eq!(validate(&t, research(&[armory], EC1)), Err(Reject::CannotResearch(EC1)));
    assert_eq!(validate(&t, research(&[id(2)], EC1)), Err(Reject::NoProducers));
    assert_eq!(
        validate(&t, research(&[armory, T1], EC1)),
        Ok(()),
        "the Temple of the two"
    );
    let now = t.sim().tick() + 1;
    t.debug_at(now, crystals(P1, L1_PRICE - 1));
    run_checked(&mut t, now);
    assert_eq!(validate(&t, research(&[T1], EC1)), Err(Reject::InsufficientCrystals));
    t.debug_at(now + 1, crystals(P1, L1_PRICE));
    t.at(now + 1, P1, research(&[armory, T1], EC1));
    run_checked(&mut t, now + 1);
    assert_eq!(queued(&t), [(now + 1, T1, EC1)]);
    assert_eq!(bank(&t, P1), 0);
    let armory_queue = &t.sim().object(armory).expect("the Armory").production;
    assert!(
        armory_queue.as_ref().is_some_and(|p| p.queue.is_empty()),
        "nothing at the Armory"
    );
}
