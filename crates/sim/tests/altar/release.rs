//! Release at the Altar (design change 0337, `factions/cults/augmentation.md > Altar > ObjectInterfaceState[Altar] >
//! AugmentMenu[UnitType, State]`: "In every state: **A: Release** (free no-op, 4 frames, placeholder; the unit walks
//! out unchanged; README > Choosing Recruits, Release and Eject All)"), of a unit of a type and a state
//! (`ProductionCommand::ReleaseOf`), a fully augmented one's too, and Eject All (`factions/cults/augmentation.md >
//! Altar > Eject All`: "ejects all StoredUnits from the exit side in a rapid stream. Units reserved by a queued order
//! are not ejected").

use super::*;

/// The brief's acceptance: "Release: free, 4 frames; the unit comes out unchanged, to the rally." Of a Militant and a
/// Gunner with Improved Metabolism and a Plain Militant, AugmentMenu\[Militant, Improved Metabolism only\]'s A reserves
/// the Metabolism Militant, and costs nothing; a second has no Metabolism Militant left, the Gunner being of another
/// type. 4 ticks later, the command's counting, the Militant comes out of side C as it went in (`SimEvent::LeftBuilding`,
/// nothing augmented) and walks to the rally point.
#[test]
fn a_release_lets_a_unit_of_the_type_and_state_out_unchanged_4_ticks_later() {
    let metabolism = RecruitKind::Metabolism;
    let (mut t, order) = filled(&[MILITANT, GUNNER, MILITANT], &[(0, metabolism), (1, metabolism)]);
    let [militant] = inside_of(&t, A1, MILITANT, metabolism)[..] else {
        panic!("one Metabolism Militant: {order:?}");
    };
    let before = t.sim().held_unit(militant).expect("inside").object.clone();
    assert_eq!(t.sim().pool_counts_of(A1, MILITANT), [1, 0, 1, 0]);
    let start = t.sim().tick() + 1;
    t.at(start, P1, rally(&[A1], RALLY));
    t.at(start, P1, release(&[A1], MILITANT, metabolism));
    run_checked(&mut t, start);
    assert_eq!(bank(&t, P1), BANK, "free");
    assert_eq!(reserved(&t, A1), [[militant]]);
    let queued = SimEvent::ReleaseQueued {
        producer: A1,
        unit: militant,
    };
    assert!(t.events().iter().any(|(tick, e)| *tick == start && *e == queued));
    assert_eq!(
        validate(&t, release(&[A1], MILITANT, metabolism)),
        Err(Reject::NoRecruit(metabolism)),
        "the Gunner isn't a Militant"
    );
    run_checked(&mut t, start + 2);
    assert!(exits(&t).is_empty() && t.sim().object(militant).is_none());
    step_checked(&mut t);
    assert_eq!(exits(&t), [(start + 3, militant)], "4 ticks: ¼ s at 16 a second");
    let o = t.sim().object(militant).expect("out");
    assert!(in_the_exit(o.pos), "in the doorway of side C: {}", o.pos);
    assert_eq!(
        (o.kind, o.augmentation, o.hp),
        (before.kind, before.augmentation, before.hp),
        "unchanged"
    );
    assert_eq!(produced(&t), [], "nothing augmented");
    let left: Vec<ObjectId> = order.iter().copied().filter(|&unit| unit != militant).collect();
    assert_eq!((inside(&t, A1), reserved(&t, A1)), (left, Vec::new()));
    run_for(&mut t, 160);
    let o = t.sim().object(militant).expect("out");
    assert!(o.pos.distance(RALLY) < 40.0, "at the rally point: {}", o.pos);
}

/// The brief's acceptance (design change 0337): "Fully augmented units: one enters; its R tile is enabled with Release
/// only; Release or Eject All brings it out." (`> Entry`: "Fully augmented units (both Augmentations already applied)
/// can also enter, but there is nothing to apply to them"; the tile is the client's, `crates/game/tests/commands/
/// altar.rs`.) Two Disciples with both go in, and every order is refused them; a Release lets the first one in out,
/// unchanged; Eject All the other.
#[test]
fn a_fully_augmented_unit_goes_in_and_a_release_or_eject_all_brings_it_out() {
    let both = RecruitKind::Both;
    let (mut t, order) = filled(&[DISCIPLE, DISCIPLE], &[(0, both), (1, both)]);
    assert_eq!(order.len(), 2, "both in");
    assert_eq!(t.sim().pool_counts_of(A1, DISCIPLE), [0, 0, 0, 2]);
    for augmentation in AugmentOrder::ALL {
        let command = augment(&[A1], DISCIPLE, both, augmentation);
        assert_eq!(validate(&t, command), Err(Reject::AlreadyAugmented(both)));
    }
    let start = t.sim().tick() + 1;
    t.at(start, P1, release(&[A1], DISCIPLE, both));
    run_checked(&mut t, start + 3);
    assert_eq!(exits(&t), [(start + 3, order[0])]);
    assert_eq!(augmentation(&t, order[0]), Augmentation::of(both));
    let eject = start + 30;
    t.at(eject, P1, eject_all(&[A1]));
    run_checked(&mut t, eject + 30);
    let out: Vec<ObjectId> = exits(&t).iter().map(|&(_, unit)| unit).collect();
    assert_eq!(out, order);
    assert_eq!(augmentation(&t, order[1]), Augmentation::of(both));
    assert_eq!(inside(&t, A1), []);
    assert_eq!(produced(&t), []);
}

/// `> Eject All`: "Units reserved by a queued order are not ejected". Eject All while a Gunner is augmented: the
/// Recruit and the Militant come out one after the other, unchanged; the Gunner stays inside, and comes out with its
/// Augmentation once its order is done.
#[test]
fn eject_all_leaves_the_unit_an_order_reserved_inside() {
    let (mut t, order) = filled(&[RECRUIT, MILITANT, GUNNER], &[]);
    let [gunner] = inside_of(&t, A1, GUNNER, RecruitKind::Plain)[..] else {
        panic!("one Gunner: {order:?}");
    };
    let start = t.sim().tick() + 1;
    let enhanced = AugmentOrder::EnhancedConstitution;
    t.at(start, P1, augment(&[A1], GUNNER, RecruitKind::Plain, enhanced));
    t.at(start + 1, P1, eject_all(&[A1]));
    run_checked(&mut t, start + 46);
    let out: Vec<ObjectId> = exits(&t).iter().map(|&(_, unit)| unit).collect();
    let others: Vec<ObjectId> = order.iter().copied().filter(|&unit| unit != gunner).collect();
    assert_eq!(out, others);
    for &unit in &others {
        assert_eq!(augmentation(&t, unit), Augmentation::default(), "{unit:?} unchanged");
    }
    assert_eq!((inside(&t, A1), reserved(&t, A1)), (vec![gunner], vec![vec![gunner]]));
    run_checked(&mut t, start + 47);
    assert_eq!(produced(&t), [(start + 47, A1, gunner)], "its 48 ticks");
    assert_eq!(augmentation(&t, gunner), Augmentation::of(RecruitKind::Constitution));
}
