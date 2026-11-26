//! Augmentation orders (`factions/cults/augmentation.md > Altar > Augmentation Orders`: "Queuing an order immediately
//! deducts its Space Crystals cost and reserves one unit of the selected type and AugmentationState from StoredUnits
//! [...]. When an order finishes, the unit's AugmentationState is updated [...], and the unit **automatically exits**
//! from the exit side and moves to the RallyPoint"; "**Enhanced Constitution** and **Improved Metabolism**: 25 Space
//! Crystals, **48 frames (3 s)** each; **Both**: the sum, 50 Space Crystals, **96 frames (6 s)**", design change
//! 0391's), the unit's HP as its MaxHP changes (`factions/cults/augmentation.md > Increments`), and which unit an order
//! takes (design change 0332).

use sc_sim::combat;

use super::*;

/// Unit `unit`'s MaxHP, Damage, FiringDuration and MaxSpeed, its Augmentations' at its owner's research levels ("No
/// research" until the Temple, slice M5-N3), on the map.
fn stats(t: &TestSim, unit: ObjectId) -> (Option<f32>, f32, u32, f32) {
    let research = t.sim().research();
    let o = t.sim().object(unit).expect("on the map");
    let weapon = combat::weapon(o, research).expect("it strikes");
    (
        o.max_hp_with(research),
        weapon.damage,
        weapon.firing_ticks,
        o.max_speed(research),
    )
}

/// The units that came out augmented so far, as `(tick, unit)`.
fn augmented(t: &TestSim) -> Vec<(Tick, ObjectId)> {
    produced(t).into_iter().map(|(tick, _, unit)| (tick, unit)).collect()
}

/// The plan's acceptance (slice M5-N1b): "With three Militants inside, W, Q, Q queues Enhanced Constitution for one: 25
/// SC and 160 frames. The Militant leaves through C with MaxHP 60." (W, Q, Q, AugmentMenu\[Militant, Unaugmented\]'s Q,
/// is the client's: `crates/game/tests/commands/altar.rs`; the frames are 48 since design change 0391.) The Militant
/// that entered first is reserved at once (design change 0332), the pool counting two Plain Militants; 48 ticks (3 s)
/// later, the command's tick counting as a producer's, it comes out into the doorway of side C, the same unit
/// (`SimEvent::UnitProduced`, no kind change), with Enhanced Constitution, "+2 Constitution / −1 Metabolism" at no
/// research (`> Augmentation Effects`): MaxHP 60, full as it was full; its Damage 4 and its FiringDuration 2 kept, an
/// armed unit's ("Armed infantry (Militant, Gunner) receive only the MaxHP part of Constitution increments and only the
/// movement-speed part of Metabolism increments"); MaxSpeed 3.5. It walks to the rally point; the two others stay
/// inside, Plain.
#[test]
fn w_q_q_gives_the_first_militant_in_enhanced_constitution_for_25_space_crystals_in_48_ticks() {
    let (mut t, order) = filled(&[MILITANT; 3], &[]);
    let start = t.sim().tick() + 1;
    t.at(start, P1, rally(&[A1], RALLY));
    let constitution = AugmentOrder::EnhancedConstitution;
    t.at(start, P1, augment(&[A1], MILITANT, RecruitKind::Plain, constitution));
    run_checked(&mut t, start);
    assert_eq!(bank(&t, P1), BANK - 25);
    assert_eq!(reserved(&t, A1), [[order[0]]]);
    assert_eq!(t.sim().free_units_of(A1, MILITANT, RecruitKind::Plain), order[1..]);
    assert_eq!(t.sim().pool_counts_of(A1, MILITANT), [2, 0, 0, 0]);
    assert_eq!(t.sim().room(A1), 7, "the reserved Militant is inside");
    let queued = SimEvent::AugmentQueued {
        producer: A1,
        unit: order[0],
        order: constitution,
    };
    assert!(t.events().iter().any(|(tick, e)| *tick == start && *e == queued));
    run_checked(&mut t, start + 46);
    assert_eq!(produced(&t), [], "augmenting");
    assert_eq!(augmentation(&t, order[0]), Augmentation::default(), "not yet");
    step_checked(&mut t);
    assert_eq!(
        produced(&t),
        [(start + 47, A1, order[0])],
        "48 ticks, the command's first"
    );
    let militant = t.sim().object(order[0]).expect("out");
    assert_eq!(
        (militant.kind, militant.augmentation),
        (MILITANT, Augmentation::of(RecruitKind::Constitution))
    );
    assert!(in_the_exit(militant.pos), "in the doorway of side C: {}", militant.pos);
    assert_eq!(hp(&t, order[0]), (60.0, 60.0));
    assert_eq!(stats(&t, order[0]), (Some(60.0), 4.0, 2, 3.5));
    assert!(!(t.events().iter()).any(|(_, e)| matches!(e, SimEvent::KindChanged { .. })));
    assert!(exits(&t).is_empty(), "augmented, not let out");
    assert_eq!((inside(&t, A1), reserved(&t, A1)), (order[1..].to_vec(), Vec::new()));
    assert_eq!(t.sim().pool_counts_of(A1, MILITANT), [2, 0, 0, 0]);
    run_for(&mut t, 160);
    let militant = t.sim().object(order[0]).expect("out");
    assert!(
        militant.pos.distance(RALLY) < 40.0,
        "at the rally point: {}",
        militant.pos
    );
    assert_eq!(bank(&t, P1), BANK - 25);
}

/// `> Increments`, "HP when MaxHP changes": "**MaxHP goes up**: HP goes up **by the same amount**, so the damage taken
/// stays the same (a full unit stays full; 25/40 → 45/60)". A Militant an enemy hurt before it went in comes out of
/// Enhanced Constitution with the damage it took: 20 HP more, of 60.
#[test]
fn a_hurt_militant_keeps_the_damage_it_took() {
    let (mut t, units) = spawned(&[MILITANT]);
    let left = hurt(&mut t, units[0], 32.0);
    assert!(left > 0.0 && left <= 32.0, "{left}");
    go_in(&mut t, &units);
    let start = t.sim().tick() + 1;
    let constitution = AugmentOrder::EnhancedConstitution;
    t.at(start, P1, augment(&[A1], MILITANT, RecruitKind::Plain, constitution));
    run_checked(&mut t, start + 47);
    assert_eq!(augmented(&t), [(start + 47, units[0])]);
    assert_eq!(hp(&t, units[0]), (left + 20.0, 60.0), "the damage stays");
}

/// The brief's acceptance: "Both, on a Recruit: 50 SC and 320 frames. Result: MaxHP 50, Damage 4, speed 4.5,
/// FiringDuration 7" (`> Augmentation Effects`: "Both at no research: +1 Constitution / +1 Metabolism"; a Plain
/// Recruit's are 40, 3, 4 and 8, and the Recruit is unarmed: "+1 melee Damage", "−1 frame of FiringDuration"). One
/// order, "a single order that applies both", for the Recruit that entered first: out after 96 ticks (6 s, design
/// change 0391), full.
#[test]
fn both_gives_a_recruit_50_max_hp_4_damage_a_speed_of_4_5_and_7_frames_of_firing_for_50_space_crystals() {
    let (mut t, order) = filled(&[RECRUIT; 2], &[]);
    let start = t.sim().tick() + 1;
    t.at(
        start,
        P1,
        augment(&[A1], RECRUIT, RecruitKind::Plain, AugmentOrder::Both),
    );
    run_checked(&mut t, start);
    assert_eq!(bank(&t, P1), BANK - 50);
    assert_eq!(reserved(&t, A1), [[order[0]]]);
    run_checked(&mut t, start + 94);
    assert_eq!(produced(&t), [], "augmenting");
    step_checked(&mut t);
    assert_eq!(augmented(&t), [(start + 95, order[0])], "96 ticks");
    let recruit = t.sim().object(order[0]).expect("out");
    assert_eq!(
        (recruit.kind, recruit.augmentation),
        (RECRUIT, Augmentation::of(RecruitKind::Both))
    );
    assert_eq!(stats(&t, order[0]), (Some(50.0), 4.0, 7, 4.5));
    assert_eq!(hp(&t, order[0]), (50.0, 50.0));
    assert_eq!(inside(&t, A1), [order[1]]);
    assert_eq!(t.sim().pool_counts_of(A1, RECRUIT), [1, 0, 0, 0]);
}

/// The brief's acceptance: "Improved Metabolism on a damaged Recruit: HP is capped, and the Recruit survives"
/// (`> Increments`: "**MaxHP goes down**: HP is **unchanged**, unless it is now above the new MaxHP, in which case it
/// becomes the new MaxHP (40/40 → 10/10; 8/40 → 8/10)"; "An augmentation therefore never kills"). Improved Metabolism
/// is "+2 Metabolism / −1 Constitution" at no research: a Recruit's MaxHP 30, Damage 2, FiringDuration 6 and MaxSpeed
/// 5. Of two hurt Recruits, the one above 30 HP comes out at 30, capped; the one at 10 or less with the HP it had; both
/// alive, one after the other.
#[test]
fn improved_metabolism_caps_a_hurt_recruit_s_hp_and_never_kills() {
    let (mut t, units) = spawned(&[RECRUIT; 2]);
    let (scratched, wounded) = (units[0], units[1]);
    let above = hurt(&mut t, scratched, 36.0);
    assert!(above > 30.0, "{above}");
    let below = hurt(&mut t, wounded, 10.0);
    assert!(below > 0.0 && below <= 10.0, "{below}");
    assert_eq!(go_in(&mut t, &[scratched]), [scratched]);
    assert_eq!(go_in(&mut t, &[wounded]), [scratched, wounded]);
    let start = t.sim().tick() + 1;
    let metabolism = AugmentOrder::ImprovedMetabolism;
    t.at(start, P1, augment(&[A1], RECRUIT, RecruitKind::Plain, metabolism));
    t.at(start, P1, augment(&[A1], RECRUIT, RecruitKind::Plain, metabolism));
    run_checked(&mut t, start);
    assert_eq!(reserved(&t, A1), [[scratched], [wounded]]);
    run_checked(&mut t, start + 95);
    assert_eq!(augmented(&t), [(start + 47, scratched), (start + 95, wounded)]);
    assert_eq!(hp(&t, scratched), (30.0, 30.0), "capped");
    assert_eq!(hp(&t, wounded), (below, 30.0), "unchanged");
    for unit in [scratched, wounded] {
        assert_eq!(stats(&t, unit), (Some(30.0), 2.0, 6, 5.0));
    }
    assert!(!(t.events().iter()).any(|(_, e)| matches!(e, SimEvent::Died { .. })));
}

/// Design change 0332 (`> Augmentation Orders`: "an order reserves the matching unit(s) that **entered the building
/// first** (first in, first trained)"; "processed one at a time, in the order queued"), within the type and the state
/// the order names: of two Recruits, three Plain Militants and one with Enhanced Constitution, in that order, an
/// Improved Metabolism for a Plain Militant reserves the first Plain Militant in, passing the Recruit before it; one
/// for the Constitution Militant that one; an Enhanced Constitution for a Plain Militant the second Plain one. They
/// come out one after the other, in the order queued, each order starting the tick after the one before it came out:
/// with Improved Metabolism, Both and Enhanced Constitution.
#[test]
fn the_unit_of_the_type_and_state_that_entered_first_is_augmented_first() {
    let kinds = [RECRUIT, MILITANT, RECRUIT, MILITANT, MILITANT, MILITANT];
    let (mut t, order) = filled(&kinds, &[(5, RecruitKind::Constitution)]);
    let plain = inside_of(&t, A1, MILITANT, RecruitKind::Plain);
    let constitution = inside_of(&t, A1, MILITANT, RecruitKind::Constitution);
    let recruits = inside_of(&t, A1, RECRUIT, RecruitKind::Plain);
    assert_eq!((plain.len(), constitution.len(), recruits.len()), (3, 1, 2));
    let start = t.sim().tick() + 1;
    let metabolism = AugmentOrder::ImprovedMetabolism;
    t.at(start, P1, augment(&[A1], MILITANT, RecruitKind::Plain, metabolism));
    t.at(
        start,
        P1,
        augment(&[A1], MILITANT, RecruitKind::Constitution, metabolism),
    );
    let enhanced = AugmentOrder::EnhancedConstitution;
    t.at(start, P1, augment(&[A1], MILITANT, RecruitKind::Plain, enhanced));
    run_checked(&mut t, start);
    assert_eq!(bank(&t, P1), BANK - 75);
    assert_eq!(reserved(&t, A1), [[plain[0]], [constitution[0]], [plain[1]]]);
    assert_eq!(t.sim().free_units_of(A1, MILITANT, RecruitKind::Plain), [plain[2]]);
    assert_eq!(t.sim().pool_counts_of(A1, RECRUIT), [2, 0, 0, 0]);
    run_for(&mut t, 150);
    assert_eq!(
        augmented(&t),
        [
            (start + 47, plain[0]),
            (start + 95, constitution[0]),
            (start + 143, plain[1])
        ]
    );
    let out = [plain[0], constitution[0], plain[1]];
    assert_eq!(
        out.map(|unit| augmentation(&t, unit).kind()),
        [RecruitKind::Metabolism, RecruitKind::Both, RecruitKind::Constitution]
    );
    let left: Vec<ObjectId> = order.into_iter().filter(|unit| !out.contains(unit)).collect();
    assert_eq!(inside(&t, A1), left, "the two Recruits and the third Plain Militant");
}
