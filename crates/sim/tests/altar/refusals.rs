//! The gate's refusals of an augmentation order (plan decision T70; `factions/cults/augmentation.md > Altar >
//! ObjectInterfaceState[Altar] > AugmentMenu[UnitType, State]`: "Available only if the group still has an unreserved
//! unit, the AugmentationQueue is not full, and the player has sufficient Space Crystals"; `> Augmentation Orders`:
//! "An order can only target a unit that is missing every Augmentation the order applies").

use super::*;

/// "An order can only target a unit that is missing every Augmentation the order applies" (`> AugmentMenu[UnitType,
/// State]`: "From Enhanced Constitution only: **W: Improved Metabolism**"; "From Improved Metabolism only: **Q:
/// Enhanced Constitution**"; Fully augmented, "only Release is possible"): of a Militant of each augmented state, each
/// order is taken or refused as it fits the state, whatever units are inside ([`Reject::AlreadyAugmented`]).
#[test]
fn an_order_for_a_unit_with_an_augmentation_it_applies_is_refused() {
    let states = [RecruitKind::Constitution, RecruitKind::Metabolism, RecruitKind::Both];
    let (t, _) = filled(&[MILITANT; 3], &[(0, states[0]), (1, states[1]), (2, states[2])]);
    for from in states {
        for order in AugmentOrder::ALL {
            let expected = if order.fits(from) {
                Ok(())
            } else {
                Err(Reject::AlreadyAugmented(from))
            };
            let command = augment(&[A1], MILITANT, from, order);
            assert_eq!(validate(&t, command), expected, "{order:?} from {from:?}");
        }
    }
    let taken: Vec<(RecruitKind, AugmentOrder)> = (states.into_iter())
        .flat_map(|from| AugmentOrder::ALL.map(|order| (from, order)))
        .filter(|&(from, order)| order.fits(from))
        .collect();
    assert_eq!(
        taken,
        [
            (RecruitKind::Constitution, AugmentOrder::ImprovedMetabolism),
            (RecruitKind::Metabolism, AugmentOrder::EnhancedConstitution),
        ]
    );
    let both = augment(&[A1], GUNNER, RecruitKind::Both, AugmentOrder::Both);
    assert_eq!(
        validate(&t, both),
        Err(Reject::AlreadyAugmented(RecruitKind::Both)),
        "no Gunner inside"
    );
}

/// "Available only if the group still has an unreserved unit": an order for a type and a state of which no unit is
/// free inside is refused ([`Reject::NoRecruit`], of the state): no Gunner at all, a Recruit of another state, a
/// Militant reserved by the order before. A Release of the reserved Militant alike.
#[test]
fn an_order_without_a_free_unit_of_the_type_and_state_is_refused() {
    let (mut t, _) = filled(&[MILITANT, RECRUIT], &[(1, RecruitKind::Constitution)]);
    let [plain] = inside_of(&t, A1, MILITANT, RecruitKind::Plain)[..] else {
        unreachable!()
    };
    let enhanced = AugmentOrder::EnhancedConstitution;
    let metabolism = AugmentOrder::ImprovedMetabolism;
    let refused = [
        (GUNNER, RecruitKind::Plain, enhanced),
        (RECRUIT, RecruitKind::Plain, enhanced),
        (MILITANT, RecruitKind::Constitution, metabolism),
        (DISCIPLE, RecruitKind::Metabolism, enhanced),
    ];
    for (kind, from, order) in refused {
        let command = augment(&[A1], kind, from, order);
        assert_eq!(validate(&t, command), Err(Reject::NoRecruit(from)), "{kind:?} {from:?}");
    }
    let militant = augment(&[A1], MILITANT, RecruitKind::Plain, enhanced);
    assert_eq!(validate(&t, militant.clone()), Ok(()));
    let start = t.sim().tick() + 1;
    t.at(start, P1, militant.clone());
    run_checked(&mut t, start);
    assert_eq!(reserved(&t, A1), [[plain]]);
    assert_eq!(
        validate(&t, militant),
        Err(Reject::NoRecruit(RecruitKind::Plain)),
        "reserved"
    );
    assert_eq!(
        validate(&t, release(&[A1], MILITANT, RecruitKind::Plain)),
        Err(Reject::NoRecruit(RecruitKind::Plain))
    );
    let recruit = augment(&[A1], RECRUIT, RecruitKind::Constitution, metabolism);
    assert_eq!(validate(&t, recruit), Ok(()), "the Constitution Recruit is free");
}

/// "AugmentationQueue: [...] (bounded by InternalUnitCapacity, reserved units included, Q0104; nominal max 20,
/// placeholder)": ten Recruits fill the Altar, and ten orders take them all; an eleventh order is refused for the full
/// queue ([`Reject::QueueFull`], which the gate gives before the want of a free unit: **Placeholder**, not in the
/// design), and no more can go in. The nominal 20 is never reached.
#[test]
fn the_queue_holds_as_many_orders_as_the_altar_holds_units() {
    let (mut t, order) = filled(&[RECRUIT; 10], &[]);
    assert_eq!(t.sim().room(A1), 0, "full");
    let start = t.sim().tick() + 1;
    let enhanced = augment(&[A1], RECRUIT, RecruitKind::Plain, AugmentOrder::EnhancedConstitution);
    for _ in 0..10 {
        t.at(start, P1, enhanced.clone());
    }
    run_checked(&mut t, start);
    assert_eq!(bank(&t, P1), BANK - 250);
    let each: Vec<Vec<ObjectId>> = order.iter().map(|&unit| vec![unit]).collect();
    assert_eq!(reserved(&t, A1), each, "first in, first reserved");
    assert_eq!(validate(&t, enhanced), Err(Reject::QueueFull));
    let both = augment(&[A1], RECRUIT, RecruitKind::Plain, AugmentOrder::Both);
    assert_eq!(validate(&t, both), Err(Reject::QueueFull));
    assert_eq!(t.sim().room(A1), 0, "the reserved Recruits are inside");
}

/// "and the player has sufficient Space Crystals": with 49, Both (50) is refused ([`Reject::InsufficientCrystals`])
/// and Enhanced Constitution (25) taken; with the 24 left, Improved Metabolism (25) is refused too.
#[test]
fn an_order_the_player_cannot_pay_for_is_refused() {
    let (mut t, _) = filled(&[RECRUIT; 3], &[]);
    let now = t.sim().tick() + 1;
    t.debug_at(now, crystals(P1, 49));
    run_checked(&mut t, now);
    let order = |order| augment(&[A1], RECRUIT, RecruitKind::Plain, order);
    assert_eq!(
        validate(&t, order(AugmentOrder::Both)),
        Err(Reject::InsufficientCrystals)
    );
    assert_eq!(validate(&t, order(AugmentOrder::EnhancedConstitution)), Ok(()));
    t.at(now + 1, P1, order(AugmentOrder::EnhancedConstitution));
    run_checked(&mut t, now + 1);
    assert_eq!((bank(&t, P1), reserved(&t, A1).len()), (24, 1));
    assert_eq!(
        validate(&t, order(AugmentOrder::ImprovedMetabolism)),
        Err(Reject::InsufficientCrystals)
    );
}

/// Only an Altar augments: an order is refused at player 1's Armory and Recruitment Hall, which apply none
/// ([`Reject::CannotAugment`]), and at player 2's Altar ([`Reject::NotOwned`]); the Altar trains nothing
/// ([`Reject::CannotProduce`]).
#[test]
fn only_an_own_altar_takes_an_order() {
    let scenario = base()
        .spawn(ObjectKind::Armory, Some(P1), tile(4, 4)) // 2
        .spawn(ObjectKind::RecruitmentHall, Some(P1), tile(4, 14)) // 3
        .spawn(ALTAR, Some(P2), tile(20, 4)); // 4
    let (mut t, units) = spawned_on(scenario, &[RECRUIT]);
    assert_eq!(go_in(&mut t, &units), units);
    let both = AugmentOrder::Both;
    let order = |at: ObjectId| augment(&[at], RECRUIT, RecruitKind::Plain, both);
    for producer in [id(2), id(3)] {
        assert_eq!(
            validate(&t, order(producer)),
            Err(Reject::CannotAugment(both)),
            "{producer:?}"
        );
    }
    assert_eq!(validate(&t, order(id(4))), Err(Reject::NotOwned(id(4))));
    assert_eq!(validate(&t, order(A1)), Ok(()));
    let train = ProductionCommand::TrainFrom {
        kind: MILITANT,
        from: RecruitKind::Plain,
    };
    assert_eq!(
        validate(&t, production(&[A1], train)),
        Err(Reject::CannotProduce(MILITANT))
    );
    assert_eq!(
        validate(&t, production(&[A1], ProductionCommand::Train(MILITANT))),
        Err(Reject::CannotProduce(MILITANT))
    );
}
