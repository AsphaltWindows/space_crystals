//! Augmenting, then training (`factions/cults/augmentation.md > Persistence`: "**Training** (Armory or Cathedral): a
//! trained unit inherits its Recruit's AugmentationState unchanged. Augmentation and training can happen in either
//! order with the same result").

use super::*;

/// The plan's acceptance (slice M5-N1b): "An augmented Recruit trained at the Armory keeps its augmentation." A
/// Recruit receives Enhanced Constitution at the Altar, comes out, walks into player 1's Armory and is trained there
/// from the Constitution Recruits (KindMenu\[Constitution\]'s Q): the Militant has Enhanced Constitution, at the
/// Militant's 60 MaxHP and full, as a Militant augmented at the Altar would be (`tests/altar/orders.rs`).
#[test]
fn a_recruit_augmented_here_and_trained_at_the_armory_keeps_its_augmentation() {
    let armory = id(2);
    let scenario = base().spawn(ObjectKind::Armory, Some(P1), tile(4, 10)); // 2: x 256..448, its entrance at x 448.
    let (mut t, units) = spawned_on(scenario, &[RECRUIT]);
    let recruit = units[0];
    go_in(&mut t, &units);
    let start = t.sim().tick() + 1;
    t.at(start, P1, rally(&[A1], Vec2::new(600.0, 704.0)));
    let enhanced = AugmentOrder::EnhancedConstitution;
    t.at(start, P1, augment(&[A1], RECRUIT, RecruitKind::Plain, enhanced));
    run_checked(&mut t, start + 47);
    assert_eq!(produced(&t), [(start + 47, A1, recruit)], "48 ticks");
    assert_eq!(augmentation(&t, recruit), Augmentation::of(RecruitKind::Constitution));
    assert_eq!(hp(&t, recruit), (60.0, 60.0));
    let now = t.sim().tick() + 1;
    t.at(now, P1, enter(&[recruit], armory));
    run_checked(&mut t, now);
    fill(&mut t, armory, 1);
    assert_eq!(t.sim().pool_counts(armory), [0, 1, 0, 0]);
    let now = t.sim().tick() + 1;
    let train = ProductionCommand::TrainFrom {
        kind: MILITANT,
        from: RecruitKind::Constitution,
    };
    t.at(now, P1, production(&[armory], train));
    run_checked(&mut t, now + 31);
    assert_eq!(produced(&t).last(), Some(&(now + 31, armory, recruit)));
    let militant = t.sim().object(recruit).expect("out");
    assert_eq!(
        (militant.kind, militant.augmentation),
        (MILITANT, Augmentation::of(RecruitKind::Constitution))
    );
    assert_eq!(hp(&t, recruit), (60.0, 60.0));
    assert_eq!(bank(&t, P1), BANK - 25 - 20);
}
