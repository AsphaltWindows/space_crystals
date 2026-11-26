//! Training a Militant (`factions/cults/structures/armory.md > Training`: "Q: Train Militant | 20 | 32 frames"; design
//! change 0337's KindMenu\[kind\]: "**Q** Train Militant", of the Recruits of that kind inside): paid and reserved when
//! queued, the Recruit that entered first first (design change 0332), out of side C as a Militant once trained, toward
//! the rally point, and X.

use super::*;

/// The plan's acceptance (slice M5-F2): "With three Recruits pooled, Q pays 20 Space Crystals and reserves; a Militant
/// comes out of side C 32 ticks later." The Recruit that entered first is reserved at once (design change 0332: "First
/// in, first trained"), and is no longer free: the pool counts two Plain Recruits. 32 ticks of training, the command's
/// tick counting as a producer's (`tests/production/queue.rs`), and it comes out into the doorway of side C as a
/// Militant, the same unit (`SimEvent::KindChanged`, no `Spawned`), and walks to the rally point; the room in the
/// Armory is one more.
#[test]
fn q_pays_20_space_crystals_and_a_militant_comes_out_32_ticks_later() {
    let (mut t, order) = filled(3);
    let start = t.sim().tick() + 1;
    t.at(start, P1, rally(&[A1], RALLY));
    t.at(start, P1, train(&[A1], RecruitKind::Plain));
    run_checked(&mut t, start);
    assert_eq!(bank(&t, P1), BANK - 20);
    assert_eq!(reserved(&t, A1), [[order[0]]]);
    assert_eq!(t.sim().free_units(A1, RecruitKind::Plain), order[1..]);
    assert_eq!(t.sim().pool_counts(A1), [2, 0, 0, 0]);
    assert_eq!(t.sim().room(A1), 7, "the reserved Recruit is inside");
    let queued = (t.events().iter()).any(|(tick, e)| {
        *tick == start
            && *e
                == SimEvent::ProductionQueued {
                    producer: A1,
                    kind: MILITANT,
                }
    });
    assert!(queued, "{:?}", t.events());
    run_checked(&mut t, start + 30);
    assert_eq!(produced(&t), [], "training");
    assert_eq!(inside(&t, A1), order);
    step_checked(&mut t);
    assert_eq!(
        produced(&t),
        [(start + 31, A1, order[0])],
        "32 ticks, the command's first"
    );
    let militant = t.sim().object(order[0]).expect("out");
    assert_eq!(militant.kind, MILITANT);
    assert!(
        militant.pos.x > EXIT.x - 0.5 && militant.pos.x < EXIT.x + 12.5 && (militant.pos.y - EXIT.y).abs() <= 52.0,
        "in the doorway of side C: {}",
        militant.pos
    );
    let changed = SimEvent::KindChanged {
        id: order[0],
        from: RECRUIT,
        to: MILITANT,
    };
    assert!(t.events().iter().any(|(tick, e)| *tick == start + 31 && *e == changed));
    assert!(
        !t.events()
            .iter()
            .any(|(tick, e)| *tick > start && *e == SimEvent::Spawned(order[0]))
    );
    assert!(exits(&t).is_empty(), "trained, not let out");
    assert_eq!((inside(&t, A1), reserved(&t, A1)), (order[1..].to_vec(), Vec::new()));
    assert_eq!(t.sim().room(A1), 8);
    run_for(&mut t, 160);
    let militant = t.sim().object(order[0]).expect("out");
    assert!(
        militant.pos.distance(RALLY) < 40.0,
        "at the rally point: {}",
        militant.pos
    );
    assert_eq!(bank(&t, P1), BANK - 20);
}

/// Design change 0332 (`factions/cults/structures/armory.md > Training`: "an order reserves the matching unit(s) that
/// **entered the building first** (first in, first trained)"; "orders are processed one at a time, in the order
/// queued"): two Militants queued in one tick reserve the first two in, in order, and come out one after the other, the
/// second's training starting the tick after the first came out.
#[test]
fn the_recruits_that_entered_first_are_trained_first() {
    let (mut t, order) = filled(3);
    let start = t.sim().tick() + 1;
    t.at(start, P1, train(&[A1], RecruitKind::Plain));
    t.at(start, P1, train(&[A1], RecruitKind::Plain));
    run_checked(&mut t, start);
    assert_eq!(bank(&t, P1), BANK - 40);
    assert_eq!(reserved(&t, A1), [[order[0]], [order[1]]]);
    assert_eq!(t.sim().free_units(A1, RecruitKind::Plain), [order[2]]);
    run_for(&mut t, 80);
    let out: Vec<(Tick, ObjectId)> = produced(&t).iter().map(|&(tick, _, unit)| (tick, unit)).collect();
    assert_eq!(out, [(start + 31, order[0]), (start + 63, order[1])]);
    assert_eq!(inside(&t, A1), [order[2]]);
}

/// `factions/cults/structures/armory.md > ObjectInterfaceState[Armory]`: "**X: Cancel Production**: removes the last
/// entry from TrainingQueue, refunds its full Space Crystals cost to the player, and returns its reserved Recruit to
/// StoredRecruits." The last of two Militants first, then the one in training, its progress lost: the Space Crystals
/// are all back, the three Recruits free and Recruits still, and nothing comes out.
#[test]
fn x_gives_back_the_recruit_and_the_20_space_crystals() {
    let (mut t, order) = filled(3);
    let start = t.sim().tick() + 1;
    t.at(start, P1, train(&[A1], RecruitKind::Plain));
    t.at(start, P1, train(&[A1], RecruitKind::Plain));
    t.at(start + 10, P1, cancel(&[A1]));
    run_checked(&mut t, start + 10);
    assert_eq!(bank(&t, P1), BANK - 20);
    assert_eq!(reserved(&t, A1), [[order[0]]], "the last entry's Recruit is free again");
    assert_eq!(t.sim().free_units(A1, RecruitKind::Plain), order[1..]);
    let cancelled = SimEvent::ProductionCancelled {
        producer: A1,
        kind: MILITANT,
        refund: 20,
    };
    assert!(
        t.events()
            .iter()
            .any(|(tick, e)| *tick == start + 10 && *e == cancelled)
    );
    t.at(start + 20, P1, cancel(&[A1]));
    run_checked(&mut t, start + 20);
    assert_eq!(bank(&t, P1), BANK);
    assert_eq!(reserved(&t, A1), Vec::<Vec<ObjectId>>::new());
    assert_eq!(t.sim().free_units(A1, RecruitKind::Plain), order);
    assert_eq!(validate(&t, cancel(&[A1])), Err(Reject::NothingQueued));
    run_for(&mut t, 64);
    assert_eq!((produced(&t), inside(&t, A1)), (Vec::new(), order.clone()));
    assert!((t.sim().held_in(A1).iter()).all(|held| held.object.kind == RECRUIT && held.leaving.is_none()));
}

/// The gate (plan decision T70: "Refusal::NoRecruit: fewer free Recruits than the order needs"): KindMenu\[kind\]'s Q
/// is refused at an Armory without a free Recruit of the kind, an empty one, one whose Recruits are all reserved, or
/// one with Recruits of another kind; for want of Space Crystals; at another player's Armory. The Armory takes no
/// plain Train, which is the other producers' (Barracks: "Q: Build Peacekeeper"), nor the Hall a TrainFrom.
#[test]
fn the_gate_refuses_training_without_a_free_recruit_of_the_kind() {
    let mut t = base()
        .spawn(ARMORY, Some(P2), tile(20, 4)) // 2
        .spawn(ObjectKind::RecruitmentHall, Some(P1), tile(2, 2)) // 3
        .start();
    t.without_auto_targeting();
    t.debug_at(1, crystals(P1, 30));
    let recruits = block(&mut t, 1, 2, 2, Vec2::new(1150.0, 704.0));
    run_checked(&mut t, 1);
    assert_eq!(
        validate(&t, train(&[A1], RecruitKind::Plain)),
        Err(Reject::NoRecruit(RecruitKind::Plain))
    );
    augment(&mut t, 2, recruits[1], RecruitKind::Both);
    t.at(2, P1, enter(&recruits, A1));
    run_checked(&mut t, 2);
    fill(&mut t, A1, 2);
    assert_eq!(t.sim().pool_counts(A1), [1, 0, 0, 1]);
    for kind in [RecruitKind::Constitution, RecruitKind::Metabolism] {
        assert_eq!(validate(&t, train(&[A1], kind)), Err(Reject::NoRecruit(kind)));
    }
    assert_eq!(validate(&t, train(&[A1], RecruitKind::Both)), Ok(()));
    assert_eq!(validate(&t, train(&[A1], RecruitKind::Plain)), Ok(()));
    assert_eq!(
        validate(&t, train(&[id(2)], RecruitKind::Plain)),
        Err(Reject::NotOwned(id(2)))
    );
    let plain = production(&[A1], ProductionCommand::Train(MILITANT));
    assert_eq!(validate(&t, plain), Err(Reject::CannotProduce(MILITANT)));
    let hall = production(
        &[id(3)],
        ProductionCommand::TrainFrom {
            kind: RECRUIT,
            from: RecruitKind::Plain,
        },
    );
    assert_eq!(validate(&t, hall), Err(Reject::CannotProduce(RECRUIT)));
    let queue = t.sim().tick() + 1;
    t.at(queue, P1, train(&[A1], RecruitKind::Plain));
    run_checked(&mut t, queue);
    assert_eq!(bank(&t, P1), 10);
    assert_eq!(
        validate(&t, train(&[A1], RecruitKind::Plain)),
        Err(Reject::NoRecruit(RecruitKind::Plain))
    );
    assert_eq!(
        validate(&t, train(&[A1], RecruitKind::Both)),
        Err(Reject::InsufficientCrystals)
    );
}

/// The plan's acceptance: "Group Q works on three Armories" (in the sim: since design change 0348 point 5, "Cults
/// production buildings aren't groupable", the panel sends it to one). One Q to three Armories, each with a Recruit,
/// queues a Militant at each as a Barracks' group Train does (design change 0083: "those with the least work queued
/// pay first, as many as the player can afford"): with Space Crystals for two, the third is refused for want of them,
/// and one without a free Recruit of the kind is passed over.
#[test]
fn group_q_trains_at_three_armories() {
    let mut t = base()
        .spawn(ARMORY, Some(P1), tile(12, 3)) // 2: y 192..320
        .spawn(ARMORY, Some(P1), tile(12, 17)) // 3: y 1088..1216
        .start();
    t.without_auto_targeting();
    let armories = [A1, id(2), id(3)];
    t.debug_at(1, crystals(P1, 50));
    let recruits = [id(4), id(5), id(6)];
    for y in [704.0, 256.0, 1152.0] {
        t.spawn_unit_at(1, RECRUIT, P1, Vec2::new(1050.0, y));
    }
    for (&recruit, &armory) in recruits.iter().zip(&armories) {
        t.at(2, P1, enter(&[recruit], armory));
    }
    run_checked(&mut t, 2);
    for armory in armories {
        fill(&mut t, armory, 1);
    }
    let start = t.sim().tick() + 1;
    t.at(start, P1, train(&armories, RecruitKind::Plain));
    run_checked(&mut t, start);
    assert_eq!(bank(&t, P1), 10);
    assert_eq!(reserved(&t, A1), [[recruits[0]]]);
    assert_eq!(reserved(&t, id(2)), [[recruits[1]]]);
    assert_eq!(reserved(&t, id(3)), Vec::<Vec<ObjectId>>::new());
    let refused = SimEvent::ProductionRefused {
        producer: id(3),
        kind: MILITANT,
        reason: sc_sim::Refusal::Crystals,
    };
    assert!(t.events().iter().any(|(tick, e)| *tick == start && *e == refused));
    t.debug_at(start + 1, crystals(P1, 100));
    t.at(start + 1, P1, train(&armories, RecruitKind::Plain));
    run_checked(&mut t, start + 1);
    assert_eq!(bank(&t, P1), 80, "only the third has a Recruit free");
    assert_eq!(reserved(&t, id(3)), [[recruits[2]]]);
    run_for(&mut t, 40);
    let mut out: Vec<(ObjectId, ObjectId)> = produced(&t)
        .iter()
        .map(|&(_, producer, unit)| (producer, unit))
        .collect();
    out.sort_unstable();
    assert_eq!(out, [(A1, recruits[0]), (id(2), recruits[1]), (id(3), recruits[2])]);
}
