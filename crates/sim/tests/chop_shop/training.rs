//! Training a Seeker (`factions/cults/structures/chop_shop.md > Production`: "queuing an order deducts its Space
//! Crystals cost and **reserves the vehicle's number of Recruits** from the pool (an order can only be queued if enough
//! unreserved Recruits are inside); when it finishes, they are consumed and the vehicle exits to the RallyPoint";
//! "**Seeker**: 2 Recruits, 75 Space Crystals, 320 frames"): paid and reserved when queued, the two Recruits that
//! entered first first (design change 0332), out of side C as a Seeker once trained, toward the rally point; the queue
//! its capacity bounds; X; a Release; and the right-click that sends Recruits in.

use sc_sim::mover::MoveStatus;

use super::*;

/// Design change 0319's Verification ("The Chop Shop needs 2 free Recruits inside to queue a Jackal", the Seeker since
/// 0364), with the kind picked (`factions/cults/structures/README.md > Choosing Recruits, Release and Eject All`: "a
/// Seeker reserves 2 Recruits of the kind"). One Plain Recruit inside: Q in KindMenu\[Plain\] is refused for want of a
/// free Recruit (`Reject::NoRecruit`). A second of another kind, Constitution, doesn't make two of the kind: both
/// KindMenus refuse it. A second Plain does: Q is taken, and reserves the two Plain ones.
#[test]
fn q_needs_two_free_recruits_of_its_kind_inside() {
    let (mut t, order) = filled(1);
    let plain = order[0];
    assert_eq!(
        validate(&t, train(&[CS], RecruitKind::Plain)),
        Err(Reject::NoRecruit(RecruitKind::Plain))
    );
    let tick = t.sim().tick() + 1;
    let strong = id(next_id(&t));
    t.spawn_unit_at(tick, RECRUIT, P1, Vec2::new(1150.0, 736.0));
    augment(&mut t, tick + 1, strong, RecruitKind::Constitution);
    t.at(tick + 2, P1, enter(&[strong], CS));
    run_checked(&mut t, tick + 2);
    fill(&mut t, CS, 2);
    assert_eq!(t.sim().pool_counts(CS), [1, 1, 0, 0]);
    for kind in [RecruitKind::Plain, RecruitKind::Constitution] {
        assert_eq!(
            validate(&t, train(&[CS], kind)),
            Err(Reject::NoRecruit(kind)),
            "{kind:?}"
        );
    }
    let tick = t.sim().tick() + 1;
    let second = id(next_id(&t));
    t.spawn_unit_at(tick, RECRUIT, P1, Vec2::new(1150.0, 736.0));
    t.at(tick + 1, P1, enter(&[second], CS));
    run_checked(&mut t, tick + 1);
    fill(&mut t, CS, 3);
    assert_eq!(validate(&t, train(&[CS], RecruitKind::Plain)), Ok(()));
    let start = t.sim().tick() + 1;
    t.at(start, P1, train(&[CS], RecruitKind::Plain));
    run_checked(&mut t, start);
    assert_eq!(reserved(&t, CS), [[plain, second]]);
    assert_eq!(t.sim().pool_counts(CS), [0, 1, 0, 0]);
    assert_eq!(bank(&t, P1), BANK - PRICE);
}

/// The plan's acceptance (slice M5-N6): "75 SC and two Recruits make a Seeker, which leaves through C after 320 frames
/// and drives to its rally point". Of three Recruits, Q reserves the two that entered first at once; 320 ticks of
/// training, the command's tick counting as a producer's, and the first comes out into the doorway of side C as a
/// Seeker, the same unit (`SimEvent::KindChanged`, no `Spawned`), half inside the middle of the side and facing out of
/// it, while the second is gone into it (plan decision T71). It drives to the rally point, at its ForwardMaxSpeed on the
/// way.
#[test]
fn q_pays_75_and_two_recruits_come_out_a_seeker_320_ticks_later_toward_the_rally_point() {
    let (mut t, order) = filled(3);
    let start = t.sim().tick() + 1;
    t.at(start, P1, rally(&[CS], RALLY));
    t.at(start, P1, train(&[CS], RecruitKind::Plain));
    run_checked(&mut t, start);
    assert_eq!(bank(&t, P1), BANK - PRICE);
    assert_eq!(reserved(&t, CS), [[order[0], order[1]]]);
    assert_eq!(t.sim().free_units(CS, RecruitKind::Plain), [order[2]]);
    assert_eq!(t.sim().pool_counts(CS), [1, 0, 0, 0]);
    assert_eq!(t.sim().room(CS), 7, "the reserved Recruits are inside");
    let queued = SimEvent::ProductionQueued {
        producer: CS,
        kind: SEEKER,
    };
    assert!(t.events().iter().any(|(tick, e)| *tick == start && *e == queued));
    run_checked(&mut t, start + FRAMES - 2);
    assert_eq!(produced(&t), [], "training");
    assert_eq!(inside(&t, CS), order);
    step_checked(&mut t);
    let out = start + FRAMES - 1;
    assert_eq!(produced(&t), [(out, CS, order[0])], "320 ticks, the command's first");
    let seeker = t.sim().object(order[0]).expect("out");
    assert_eq!(seeker.kind, SEEKER);
    assert_eq!(seeker.pos, EXIT, "half inside, at the middle of side C");
    assert!(
        sc_sim::math::facing_vector(seeker.facing).distance(-Vec2::X) < 1e-6,
        "facing out, west"
    );
    assert_eq!(
        seeker.mover.as_ref().and_then(|mover| mover.exit),
        Some(CS),
        "in the doorway"
    );
    let changed = SimEvent::KindChanged {
        id: order[0],
        from: RECRUIT,
        to: SEEKER,
    };
    assert!(t.events().iter().any(|(tick, e)| *tick == out && *e == changed));
    assert!(
        !t.events()
            .iter()
            .any(|(tick, e)| *tick > start && *e == SimEvent::Spawned(order[0]))
    );
    assert!(
        t.sim().object(order[1]).is_none() && t.sim().held_unit(order[1]).is_none(),
        "gone into it"
    );
    assert!(exits(&t).is_empty(), "trained, not let out");
    assert_eq!((inside(&t, CS), reserved(&t, CS)), (vec![order[2]], Vec::new()));
    assert_eq!(t.sim().room(CS), 9);
    let mut fastest: f32 = 0.0;
    while t.status(order[0].0) != MoveStatus::Arrived {
        assert!(t.sim().tick() < out + 200, "{:?}", t.mover(order[0].0));
        step_checked(&mut t);
        fastest = fastest.max(t.mover(order[0].0).momentum.length());
    }
    assert!(
        pos(&t, order[0]).distance(RALLY) < 32.0,
        "at the rally point: {}",
        pos(&t, order[0])
    );
    assert!(near(fastest, TOP, 1e-3), "at full speed on the way: {fastest}");
    assert_eq!(bank(&t, P1), BANK - PRICE);
}

/// The queue (`factions/cults/structures/chop_shop.md > Placeholder stats`: "InternalRecruitCapacity 10, counting
/// reserved Recruits; so at most 5 Seekers queued (bounded by capacity: reserved units count toward it; Q0104)"). Ten
/// Recruits inside: five Seekers are queued, 75 Space Crystals each, each reserving the next two in, and a sixth is
/// refused for want of free Recruits, whatever the bank. The Seekers come out one after the other, 320 ticks apart, to
/// the rally point.
#[test]
fn ten_recruits_queue_five_seekers_and_a_sixth_is_refused() {
    let (mut t, order) = filled(10);
    let start = t.sim().tick() + 1;
    t.at(start, P1, rally(&[CS], RALLY));
    for _ in 0..5 {
        t.at(start, P1, train(&[CS], RecruitKind::Plain));
    }
    run_checked(&mut t, start);
    assert_eq!(bank(&t, P1), BANK - 5 * PRICE);
    let pairs: Vec<Vec<ObjectId>> = order.chunks(2).map(<[ObjectId]>::to_vec).collect();
    assert_eq!(reserved(&t, CS), pairs, "the first in first");
    assert_eq!((t.sim().pool_counts(CS), t.sim().room(CS)), ([0; 4], 0));
    assert_eq!(
        validate(&t, train(&[CS], RecruitKind::Plain)),
        Err(Reject::NoRecruit(RecruitKind::Plain))
    );
    let (first, a) = until_produced(&mut t, CS, FRAMES + 2);
    let (second, b) = until_produced(&mut t, CS, FRAMES + 2);
    assert_eq!((a, b), (order[0], order[2]));
    assert_eq!(second - first, FRAMES, "one after the other, the doorway free again");
    run_for(&mut t, 120);
    for seeker in [a, b] {
        assert_eq!(t.sim().object(seeker).map(|o| o.kind), Some(SEEKER));
        assert!(
            pos(&t, seeker).distance(RALLY) < 96.0,
            "{seeker:?} at {}",
            pos(&t, seeker)
        );
    }
    assert_eq!(reserved(&t, CS), pairs[2..].to_vec());
}

/// "Eject All and Cancel work as at the Armory (**cancel returns the reserved Recruits to the pool**)": X takes the
/// last entry back, its 75 Space Crystals and both its Recruits, free again; X again takes the one in training, its
/// progress lost. Nothing comes out, and the four are Recruits still.
#[test]
fn x_gives_back_both_recruits_and_the_75_space_crystals() {
    let (mut t, order) = filled(4);
    let start = t.sim().tick() + 1;
    t.at(start, P1, train(&[CS], RecruitKind::Plain));
    t.at(start, P1, train(&[CS], RecruitKind::Plain));
    run_checked(&mut t, start);
    assert_eq!((bank(&t, P1), t.sim().pool_counts(CS)), (BANK - 2 * PRICE, [0; 4]));
    t.at(start + 10, P1, cancel(&[CS]));
    run_checked(&mut t, start + 10);
    assert_eq!(bank(&t, P1), BANK - PRICE, "refunded in full");
    assert_eq!(reserved(&t, CS), [[order[0], order[1]]]);
    assert_eq!(
        t.sim().free_units(CS, RecruitKind::Plain),
        [order[2], order[3]],
        "back in the pool, in order"
    );
    t.at(start + 100, P1, cancel(&[CS]));
    run_checked(&mut t, start + 100);
    assert_eq!(bank(&t, P1), BANK);
    assert_eq!((reserved(&t, CS), t.sim().pool_counts(CS)), (Vec::new(), [4, 0, 0, 0]));
    run_for(&mut t, FRAMES);
    assert_eq!(produced(&t), []);
    assert_eq!(inside(&t, CS), order);
    for unit in order {
        let held = t.sim().held_unit(unit).expect("inside");
        assert_eq!(held.object.kind, RECRUIT);
    }
}

/// A Release (`factions/cults/structures/README.md > Choosing Recruits, Release and Eject All`, design change 0337: the
/// KindMenu's "A: Release" queues one Recruit of the kind to leave, unchanged, as at the Airstrip): queued behind a
/// Seeker, which reserved the two that entered first, it reserves the free one that entered first of the others, and
/// lets it out of side C's doorway as a Recruit once the Seeker is out; Eject All lets the rest out the same way.
/// Neither costs anything.
#[test]
fn a_release_and_eject_all_let_recruits_out_of_side_c_unchanged() {
    let (mut t, order) = filled(5);
    let start = t.sim().tick() + 1;
    t.at(start, P1, rally(&[CS], RALLY));
    t.at(start, P1, train(&[CS], RecruitKind::Plain));
    t.at(start, P1, release(&[CS], RecruitKind::Plain));
    run_checked(&mut t, start);
    assert_eq!(reserved(&t, CS), [vec![order[0], order[1]], vec![order[2]]]);
    assert_eq!(bank(&t, P1), BANK - PRICE, "a Release is free");
    let (_, seeker) = until_produced(&mut t, CS, FRAMES + 2);
    assert_eq!(seeker, order[0]);
    while exits(&t).is_empty() {
        assert!(t.sim().tick() < start + FRAMES + 120, "the Release");
        step_checked(&mut t);
    }
    let [(_, released)] = exits(&t)[..] else {
        panic!("one Recruit out: {:?}", exits(&t));
    };
    assert_eq!(released, order[2], "the one that entered first of the free ones");
    let o = t.sim().object(released).expect("out");
    assert_eq!(o.kind, RECRUIT);
    assert!(
        near(o.pos.y, EXIT.y, 0.5) && near(o.pos.x, EXIT.x, 16.0),
        "in side C's doorway: {}",
        o.pos
    );
    let tick = t.sim().tick() + 1;
    t.at(tick, P1, eject_all(&[CS]));
    while exits(&t).len() < 3 {
        assert!(t.sim().tick() < tick + 200, "{:?}", exits(&t));
        step_checked(&mut t);
    }
    let out: Vec<ObjectId> = exits(&t).iter().map(|&(_, unit)| unit).collect();
    assert_eq!(out, [order[2], order[3], order[4]]);
    assert!(inside(&t, CS).is_empty());
    assert_eq!(bank(&t, P1), BANK - PRICE);
}

/// `factions/cults/units/recruit.md > Right-Click Resolution` (`systems/controls/commands.md > Enter`, "an own
/// structure that can hold units"): a right-click on the Chop Shop is a Recruit's Enter, and the Seeker's is a Move
/// there: the Chop Shop takes Recruits only (`HoldsDef::accepts`), and refuses a Seeker's Enter.
#[test]
fn a_right_click_on_it_is_a_recruit_s_enter_and_a_seeker_s_move() {
    let (mut t, order) = filled(2);
    let recruit = id(next_id(&t));
    t.spawn_unit_at(t.sim().tick() + 1, RECRUIT, P1, Vec2::new(1150.0, 736.0));
    let start = t.sim().tick() + 1;
    t.at(start, P1, rally(&[CS], RALLY));
    t.at(start, P1, train(&[CS], RecruitKind::Plain));
    let (_, seeker) = until_produced(&mut t, CS, FRAMES + 2);
    assert_eq!(seeker, order[0]);
    let on_chop_shop = |t: &TestSim, unit| t.sim().default_order(unit, Target::Object(CS));
    assert_eq!(on_chop_shop(&t, recruit), Some(OrderKind::Enter(CS)));
    assert_eq!(on_chop_shop(&t, seeker), Some(OrderKind::Move(Target::Object(CS))));
    assert!(validate(&t, enter(&[seeker], CS)).is_err());
}
