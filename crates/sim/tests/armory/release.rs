//! Release (design change 0337, `factions/cults/structures/README.md > Choosing Recruits, Release and Eject All`: "a
//! **no-op order**: **free**, very fast (**4 frames**, placeholder), queued like any order; when it completes the
//! reserved unit walks out of the exit **unchanged** and goes to the RallyPoint"), and its place in the queue.

use super::*;

/// [`filled`] with three Recruits, one of which has Enhanced Constitution, given before it went in: the Plain ones in
/// the order they went in, and the Constitution one.
fn two_plain_and_one_constitution() -> (TestSim, Vec<ObjectId>, ObjectId) {
    let mut t = base().start();
    t.without_auto_targeting();
    t.debug_at(1, crystals(P1, BANK));
    let recruits = block(&mut t, 1, 3, 3, Vec2::new(1150.0, 704.0));
    let constitution = recruits[2];
    augment(&mut t, 2, constitution, RecruitKind::Constitution);
    t.at(2, P1, enter(&recruits, A1));
    run_checked(&mut t, 2);
    fill(&mut t, A1, 3);
    let plain = inside(&t, A1)
        .into_iter()
        .filter(|&unit| unit != constitution)
        .collect();
    (t, plain, constitution)
}

/// Design change 0337's Verification, as the sim has it (the panel's half is the client's,
/// `crates/game/tests/commands/armory.rs`): "An Armory with 2 Plain and 1 Constitution Recruit: W shows 1. Pressing W,
/// A, A queues one Release; the second press is greyed. The Constitution Recruit walks out after ¼ s." The pool counts
/// one Constitution Recruit; a Release of one costs nothing and reserves it, so a second has none left; 4 ticks later,
/// the command's counting, it comes out of side C a Recruit with its augmentation, the same unit
/// (`SimEvent::LeftBuilding`), and walks to the rally point. The Plain Recruits stay inside.
#[test]
fn a_release_lets_the_constitution_recruit_out_after_a_quarter_second() {
    let (mut t, plain, constitution) = two_plain_and_one_constitution();
    assert_eq!(t.sim().pool_counts(A1), [2, 1, 0, 0], "W shows 1");
    let start = t.sim().tick() + 1;
    t.at(start, P1, rally(&[A1], RALLY));
    t.at(start, P1, release(&[A1], RecruitKind::Constitution));
    run_checked(&mut t, start);
    assert_eq!(bank(&t, P1), BANK, "free");
    assert_eq!(reserved(&t, A1), [[constitution]]);
    let queued = SimEvent::ReleaseQueued {
        producer: A1,
        unit: constitution,
    };
    assert!(t.events().iter().any(|(tick, e)| *tick == start && *e == queued));
    assert_eq!(t.sim().pool_counts(A1), [2, 0, 0, 0]);
    let again = release(&[A1], RecruitKind::Constitution);
    assert_eq!(
        validate(&t, again),
        Err(Reject::NoRecruit(RecruitKind::Constitution)),
        "greyed"
    );
    run_checked(&mut t, start + 2);
    assert!(exits(&t).is_empty() && t.sim().object(constitution).is_none());
    step_checked(&mut t);
    assert_eq!(exits(&t), [(start + 3, constitution)], "4 ticks: ¼ s at 16 a second");
    let o = t.sim().object(constitution).expect("out");
    assert_eq!(
        (o.kind, o.augmentation),
        (RECRUIT, Augmentation::of(RecruitKind::Constitution))
    );
    assert!(o.pos.x < EXIT.x + 12.5, "out of side C: {}", o.pos);
    assert_eq!(produced(&t), [], "nothing trained");
    assert_eq!((inside(&t, A1), reserved(&t, A1)), (plain, Vec::new()));
    run_for(&mut t, 160);
    let o = t.sim().object(constitution).expect("out");
    assert!(o.pos.distance(RALLY) < 40.0, "at the rally point: {}", o.pos);
}

/// "X: Cancel" takes a Release as it takes a training: the last entry goes, and its Recruit is free again, inside
/// (`SimEvent::ReleaseCancelled`); nothing was paid, nothing is refunded.
#[test]
fn x_takes_back_a_release() {
    let (mut t, plain, _) = two_plain_and_one_constitution();
    let order = inside(&t, A1);
    let start = t.sim().tick() + 1;
    t.at(start, P1, release(&[A1], RecruitKind::Plain));
    t.at(start + 1, P1, cancel(&[A1]));
    run_checked(&mut t, start + 1);
    let cancelled = SimEvent::ReleaseCancelled {
        producer: A1,
        unit: plain[0],
    };
    assert!(t.events().iter().any(|(tick, e)| *tick == start + 1 && *e == cancelled));
    assert_eq!((bank(&t, P1), reserved(&t, A1)), (BANK, Vec::new()));
    assert_eq!(t.sim().pool_counts(A1), [2, 1, 0, 0]);
    run_for(&mut t, 16);
    assert_eq!((exits(&t), inside(&t, A1)), (Vec::new(), order));
}

/// A Release is "queued like any order": behind a training it waits its turn, and the training behind it waits for its
/// Recruit to be out (`factions/cults/structures/armory.md > Training`: "orders are processed one at a time, in the
/// order queued"; "Only the front of the queue actively trains"). The doorway is the one way out, so the Released
/// Recruit comes out once the Militant before it has walked clear of it (`core/entities.md > Structure Type > Exit
/// Doorway` 3: "the next unit can only appear once the doorway is free"); the second Militant 32 ticks after that.
#[test]
fn a_release_is_queued_like_any_order() {
    let (mut t, plain, constitution) = two_plain_and_one_constitution();
    let start = t.sim().tick() + 1;
    t.at(start, P1, train(&[A1], RecruitKind::Plain));
    t.at(start, P1, release(&[A1], RecruitKind::Constitution));
    t.at(start, P1, train(&[A1], RecruitKind::Plain));
    run_checked(&mut t, start);
    assert_eq!(reserved(&t, A1), [[plain[0]], [constitution], [plain[1]]]);
    assert_eq!(bank(&t, P1), BANK - 40);
    run_for(&mut t, 120);
    let trained: Vec<(Tick, ObjectId)> = produced(&t).iter().map(|&(tick, _, unit)| (tick, unit)).collect();
    let [(first, militant), (second, last)] = trained[..] else {
        panic!("two Militants: {trained:?}");
    };
    let [(released, recruit)] = exits(&t)[..] else {
        panic!("one Release: {:?}", exits(&t));
    };
    assert_eq!((militant, recruit, last), (plain[0], constitution, plain[1]));
    assert_eq!(first, start + 31);
    assert!(
        released >= first + 4 && released <= first + 20,
        "its 4 ticks, then the doorway: {released}"
    );
    assert_eq!(
        second,
        released + 32,
        "the next starts once the Release's Recruit is out"
    );
    assert!(inside(&t, A1).is_empty());
}
