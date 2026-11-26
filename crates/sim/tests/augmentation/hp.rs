//! `factions/cults/augmentation.md > Increments`, "HP when MaxHP changes" (design changes 0330 and 0331): "MaxHP goes
//! up: HP goes up by the same amount, so the damage taken stays the same (a full unit stays full)"; "MaxHP goes down:
//! HP is unchanged, unless it is now above the new MaxHP, in which case it becomes the new MaxHP"; "An augmentation
//! therefore never kills"; "No floors on the results".

use super::*;

const UNIT: ObjectId = ObjectId(1);

/// Player 1's Recruit (1), and player 2's (2) beside it, which hits it if ordered to.
fn two_recruits() -> TestSim {
    let mut t = cults(16, 8)
        .spawn(ObjectKind::Recruit, Some(P1), point(200.0, 200.0))
        .spawn(ObjectKind::Recruit, Some(P2), point(240.0, 200.0))
        .start();
    t.without_auto_targeting();
    t
}

/// A full Recruit, 40/40: Improved Metabolism (−1 Constitution increment at no research) caps it at its new MaxHP,
/// 30/30, and Enhanced Constitution on top of it (+1 in all) takes it up as much, to 50/50, full; with neither again,
/// 40/40.
#[test]
fn a_full_unit_is_capped_as_max_hp_falls_and_stays_full_as_it_rises() {
    let mut t = two_recruits();
    assert_eq!(hp(&t, UNIT), (40.0, 40.0));
    augment(&mut t, UNIT, RecruitKind::Metabolism);
    assert_eq!(hp(&t, UNIT), (30.0, 30.0));
    augment(&mut t, UNIT, RecruitKind::Both);
    assert_eq!(hp(&t, UNIT), (50.0, 50.0));
    augment(&mut t, UNIT, RecruitKind::Plain);
    assert_eq!(hp(&t, UNIT), (40.0, 40.0));
}

/// A Recruit that an enemy hit down to 28 HP or below, its attacker then removed: Improved Metabolism (MaxHP 30)
/// leaves its HP as it is, below the new maximum, and it lives; Enhanced Constitution instead (MaxHP 60) raises it by
/// the 30 the maximum rose, the damage staying; neither again (MaxHP 40) brings it down to 40 only if it is above.
#[test]
fn a_hurt_unit_keeps_its_damage_as_max_hp_rises_and_its_hp_as_it_falls() {
    let mut t = two_recruits();
    t.at(next(&t), P2, attack(&[id(2)], UNIT));
    while hp(&t, UNIT).0 > 28.0 {
        assert!(t.sim().tick() < 300, "never hit enough");
        t.step();
    }
    t.debug_at(next(&t), DebugCommand::Remove { id: id(2) });
    t.step();
    let (hurt, _) = hp(&t, UNIT);
    assert!(hurt > 0.0);
    augment(&mut t, UNIT, RecruitKind::Metabolism);
    assert_eq!(hp(&t, UNIT), (hurt, 30.0), "unchanged, below the new MaxHP");
    augment(&mut t, UNIT, RecruitKind::Constitution);
    assert_eq!(hp(&t, UNIT), (hurt + 30.0, 60.0), "the damage stays");
    augment(&mut t, UNIT, RecruitKind::Plain);
    assert_eq!(hp(&t, UNIT), ((hurt + 30.0).min(40.0), 40.0));
}
