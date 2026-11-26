//! Training a Disciple (`factions/cults/structures/cathedral.md > Training`: "queuing an order deducts its Space
//! Crystals cost and reserves one Recruit; when the order finishes, the Recruit is consumed and the trained unit exits
//! to the RallyPoint"; "**Disciple**: 1 Recruit, 75 Space Crystals, 320 frames"): paid and reserved when queued, the
//! Recruit that entered first first (design change 0332), out of side C as a Disciple once trained, with its energy
//! (`factions/cults/units/disciple.md > Placeholder stats`: "Energy - MaxEnergy 200; starts at 50; regenerates 0.5 per
//! second"), toward the rally point, and X.

use super::*;

/// The plan's acceptance (slice M5-N4a): "with a Temple owned and a Recruit inside, KindMenu\[Plain\] Q costs 75 SC and
/// takes 320 frames. A Disciple leaves through C with 50 energy". The Recruit that entered first is reserved at once
/// and no longer free; 320 ticks of training, the command's tick counting as a producer's, and it comes out into the
/// doorway of side C as a Disciple, the same unit (`SimEvent::KindChanged`, no `Spawned`), its energy its type's start,
/// and walks to the rally point, regenerating on the way.
#[test]
fn q_pays_75_space_crystals_and_a_disciple_comes_out_320_ticks_later_with_50_energy() {
    let (mut t, order) = filled(3);
    assert!(t.sim().requirement_met(P1, TEMPLE), "a Temple owned");
    let start = t.sim().tick() + 1;
    t.at(start, P1, rally(&[C1], RALLY));
    t.at(start, P1, train(&[C1], RecruitKind::Plain));
    run_checked(&mut t, start);
    assert_eq!(bank(&t, P1), BANK - PRICE);
    assert_eq!(reserved(&t, C1), [[order[0]]]);
    assert_eq!(t.sim().free_units(C1, RecruitKind::Plain), order[1..]);
    assert_eq!(t.sim().pool_counts(C1), [2, 0, 0, 0]);
    assert_eq!(t.sim().room(C1), 7, "the reserved Recruit is inside");
    let queued = SimEvent::ProductionQueued {
        producer: C1,
        kind: DISCIPLE,
    };
    assert!(t.events().iter().any(|(tick, e)| *tick == start && *e == queued));
    assert_eq!(energy(&t, order[0]), None, "a Recruit has no energy");
    run_checked(&mut t, start + FRAMES - 2);
    assert_eq!(produced(&t), [], "training");
    assert_eq!(inside(&t, C1), order);
    step_checked(&mut t);
    let out = start + FRAMES - 1;
    assert_eq!(produced(&t), [(out, C1, order[0])], "320 ticks, the command's first");
    let disciple = t.sim().object(order[0]).expect("out");
    assert_eq!(disciple.kind, DISCIPLE);
    assert!(
        disciple.pos.x > EXIT.x - 0.5 && disciple.pos.x < EXIT.x + 12.5 && (disciple.pos.y - EXIT.y).abs() <= 52.0,
        "in the doorway of side C: {}",
        disciple.pos
    );
    assert_eq!(energy(&t, order[0]), Some(50.0), "its start, in the tick it comes out");
    let changed = SimEvent::KindChanged {
        id: order[0],
        from: RECRUIT,
        to: DISCIPLE,
    };
    assert!(t.events().iter().any(|(tick, e)| *tick == out && *e == changed));
    assert!(
        !t.events()
            .iter()
            .any(|(tick, e)| *tick > start && *e == SimEvent::Spawned(order[0]))
    );
    assert!(exits(&t).is_empty(), "trained, not let out");
    assert_eq!((inside(&t, C1), reserved(&t, C1)), (order[1..].to_vec(), Vec::new()));
    assert_eq!(t.sim().room(C1), 8);
    run_for(&mut t, 160);
    let disciple = t.sim().object(order[0]).expect("out");
    assert!(
        disciple.pos.distance(RALLY) < 40.0,
        "at the rally point: {}",
        disciple.pos
    );
    assert_eq!(energy(&t, order[0]), Some(55.0), "0.5 a second for 10 s");
    assert_eq!(bank(&t, P1), BANK - PRICE);
}

/// The plan's acceptance: the Disciple "reaches 200 after 300 s". At 0.5 a second, 2048 of `Object::energy`'s units a
/// tick (plan decision T75), the 150 points from its start take 4800 ticks to the tick: below 200 a tick before, at
/// 200 then, and there it stays.
#[test]
fn the_disciple_trained_reaches_200_energy_300_seconds_after_it_comes_out() {
    let (mut t, order) = filled(1);
    let start = t.sim().tick() + 1;
    t.at(start, P1, train(&[C1], RecruitKind::Plain));
    run_checked(&mut t, start + FRAMES - 1);
    let [(out, _, disciple)] = produced(&t)[..] else {
        panic!("one Disciple: {:?}", produced(&t));
    };
    assert_eq!((out, disciple), (start + FRAMES - 1, order[0]));
    let raw = |t: &TestSim| t.sim().object(disciple).expect("on the map").energy;
    assert_eq!(raw(&t), Some(50 * sc_sim::energy::ONE));
    run_checked(&mut t, out + 4799);
    assert_eq!(raw(&t), Some(200 * sc_sim::energy::ONE - 2048), "a tick short of 300 s");
    assert_eq!(energy(&t, disciple), Some(199.968_75));
    step_checked(&mut t);
    assert_eq!(raw(&t), Some(200 * sc_sim::energy::ONE), "300 s: MaxEnergy");
    run_for(&mut t, 16);
    assert_eq!(energy(&t, disciple), Some(200.0), "never above it");
}

/// Design change 0332 (`factions/cults/structures/README.md`: "first in, first trained"; the Cathedral's "training
/// orders are queueable and serial"): two Disciples queued in one tick reserve the first two in, in order, and come out
/// one after the other, the second's training starting the tick after the first came out.
#[test]
fn the_recruits_that_entered_first_are_trained_first() {
    let (mut t, order) = filled(3);
    let start = t.sim().tick() + 1;
    t.at(start, P1, train(&[C1], RecruitKind::Plain));
    t.at(start, P1, train(&[C1], RecruitKind::Plain));
    run_checked(&mut t, start);
    assert_eq!(bank(&t, P1), BANK - 2 * PRICE);
    assert_eq!(reserved(&t, C1), [[order[0]], [order[1]]]);
    assert_eq!(t.sim().free_units(C1, RecruitKind::Plain), [order[2]]);
    run_for(&mut t, 2 * FRAMES + 20);
    let out: Vec<(Tick, ObjectId)> = produced(&t).iter().map(|&(tick, _, unit)| (tick, unit)).collect();
    assert_eq!(
        out,
        [(start + FRAMES - 1, order[0]), (start + 2 * FRAMES - 1, order[1])]
    );
    assert_eq!(inside(&t, C1), [order[2]]);
}

/// `factions/cults/structures/cathedral.md > Placeholder stats`: "**X: Cancel Production**", as the Armory's: the last
/// entry goes, its Space Crystals back and its Recruit free again; the one in training then, its progress lost.
/// Nothing comes out, and the three are Recruits still.
#[test]
fn x_gives_back_the_recruit_and_the_75_space_crystals() {
    let (mut t, order) = filled(3);
    let start = t.sim().tick() + 1;
    t.at(start, P1, train(&[C1], RecruitKind::Plain));
    t.at(start, P1, train(&[C1], RecruitKind::Plain));
    t.at(start + 10, P1, cancel(&[C1]));
    run_checked(&mut t, start + 10);
    assert_eq!(bank(&t, P1), BANK - PRICE);
    assert_eq!(reserved(&t, C1), [[order[0]]], "the last entry's Recruit is free again");
    let cancelled = SimEvent::ProductionCancelled {
        producer: C1,
        kind: DISCIPLE,
        refund: PRICE,
    };
    assert!(
        t.events()
            .iter()
            .any(|(tick, e)| *tick == start + 10 && *e == cancelled)
    );
    t.at(start + 200, P1, cancel(&[C1]));
    run_checked(&mut t, start + 200);
    assert_eq!(bank(&t, P1), BANK);
    assert_eq!(reserved(&t, C1), Vec::<Vec<ObjectId>>::new());
    assert_eq!(t.sim().free_units(C1, RecruitKind::Plain), order);
    assert_eq!(validate(&t, cancel(&[C1])), Err(Reject::NothingQueued));
    run_for(&mut t, FRAMES);
    assert_eq!((produced(&t), inside(&t, C1)), (Vec::new(), order.clone()));
    assert!((t.sim().held_in(C1).iter()).all(|held| held.object.kind == RECRUIT && held.leaving.is_none()));
}

/// The gate (plan decision T70), as at the Armory: Q is refused at a Cathedral without a free Recruit of the kind, for
/// want of Space Crystals, at another player's Cathedral. The Cathedral trains Disciples only, and nothing else trains
/// them: a Militant is refused there, a Disciple at an Armory, and a plain Train, the other producers', at both.
#[test]
fn the_gate_refuses_what_the_cathedral_does_not_train() {
    let mut t = base()
        .spawn(CATHEDRAL, Some(P2), tile(20, 4)) // 3
        .spawn(ObjectKind::Armory, Some(P1), tile(20, 16)) // 4
        .start();
    t.without_auto_targeting();
    t.debug_at(1, crystals(P1, 100));
    let recruits = block(&mut t, 1, 2, 2, Vec2::new(1150.0, 704.0));
    run_checked(&mut t, 1);
    assert_eq!(
        validate(&t, train(&[C1], RecruitKind::Plain)),
        Err(Reject::NoRecruit(RecruitKind::Plain)),
        "empty"
    );
    t.at(2, P1, enter(&recruits, C1));
    run_checked(&mut t, 2);
    fill(&mut t, C1, 2);
    assert_eq!(validate(&t, train(&[C1], RecruitKind::Plain)), Ok(()));
    assert_eq!(
        validate(&t, train(&[C1], RecruitKind::Both)),
        Err(Reject::NoRecruit(RecruitKind::Both))
    );
    assert_eq!(
        validate(&t, train(&[id(3)], RecruitKind::Plain)),
        Err(Reject::NotOwned(id(3)))
    );
    let militant = ProductionCommand::TrainFrom {
        kind: ObjectKind::Militant,
        from: RecruitKind::Plain,
    };
    assert_eq!(
        validate(&t, production(&[C1], militant)),
        Err(Reject::CannotProduce(ObjectKind::Militant))
    );
    assert_eq!(
        validate(&t, train(&[id(4)], RecruitKind::Plain)),
        Err(Reject::CannotProduce(DISCIPLE)),
        "an Armory"
    );
    let plain = production(&[C1], ProductionCommand::Train(DISCIPLE));
    assert_eq!(validate(&t, plain), Err(Reject::CannotProduce(DISCIPLE)));
    let queue = t.sim().tick() + 1;
    t.at(queue, P1, train(&[C1], RecruitKind::Plain));
    run_checked(&mut t, queue);
    assert_eq!(bank(&t, P1), 25);
    assert_eq!(
        validate(&t, train(&[C1], RecruitKind::Plain)),
        Err(Reject::InsufficientCrystals)
    );
}

/// Group Q, as at the Armory (`tests/armory/training.rs`; design change 0348 point 5: "Cults production buildings
/// aren't groupable", so the panel sends it to one, and the sim takes it for several as a Barracks' group Train,
/// design change 0083): one Q to three Cathedrals, each with a Recruit, with Space Crystals for two, queues a Disciple
/// at two and refuses the third for want of them; with more, the third.
#[test]
fn group_q_trains_at_three_cathedrals() {
    let mut t = base()
        .spawn(CATHEDRAL, Some(P1), tile(12, 3)) // 3: y 192..320
        .spawn(CATHEDRAL, Some(P1), tile(12, 17)) // 4: y 1088..1216
        .start();
    t.without_auto_targeting();
    let cathedrals = [C1, id(3), id(4)];
    t.debug_at(1, crystals(P1, 2 * PRICE + 10));
    let recruits = [id(5), id(6), id(7)];
    for y in [704.0, 256.0, 1152.0] {
        t.spawn_unit_at(1, RECRUIT, P1, Vec2::new(1050.0, y));
    }
    for (&recruit, &cathedral) in recruits.iter().zip(&cathedrals) {
        t.at(2, P1, enter(&[recruit], cathedral));
    }
    run_checked(&mut t, 2);
    for cathedral in cathedrals {
        fill(&mut t, cathedral, 1);
    }
    let start = t.sim().tick() + 1;
    t.at(start, P1, train(&cathedrals, RecruitKind::Plain));
    run_checked(&mut t, start);
    assert_eq!(bank(&t, P1), 10);
    assert_eq!(reserved(&t, C1), [[recruits[0]]]);
    assert_eq!(reserved(&t, id(3)), [[recruits[1]]]);
    assert_eq!(reserved(&t, id(4)), Vec::<Vec<ObjectId>>::new());
    let refused = SimEvent::ProductionRefused {
        producer: id(4),
        kind: DISCIPLE,
        reason: sc_sim::Refusal::Crystals,
    };
    assert!(t.events().iter().any(|(tick, e)| *tick == start && *e == refused));
    t.debug_at(start + 1, crystals(P1, 100));
    t.at(start + 1, P1, train(&cathedrals, RecruitKind::Plain));
    run_checked(&mut t, start + 1);
    assert_eq!(bank(&t, P1), 25, "only the third has a Recruit free");
    assert_eq!(reserved(&t, id(4)), [[recruits[2]]]);
    run_for(&mut t, FRAMES + 10);
    let mut out: Vec<(ObjectId, ObjectId)> = produced(&t)
        .iter()
        .map(|&(_, producer, unit)| (producer, unit))
        .collect();
    out.sort_unstable();
    assert_eq!(out, [(C1, recruits[0]), (id(3), recruits[1]), (id(4), recruits[2])]);
}
