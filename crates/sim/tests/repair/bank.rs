//! The bank running out (design change 0329, `factions/cults/units/recruit.md > Repair`: "if the player **runs out of
//! Space Crystals** mid-repair, the order **ends** (they stop; a free Tunnel repair never runs out)"): the repair is
//! paid in whole crystals as it goes, the bank never below 0.

use super::*;

/// Where the Recruits' queued Move goes, south of the Armory.
const AWAY: Vec2 = Vec2::new(736.0, 1300.0);

/// Design change 0329's verification: "With the bank at 0, the repairers stop: the order ends, the bank stays 0, and
/// Shift-queued orders go on." Two Recruits at work on the Armory at 150 HP, the bank set to 10 crystals and a Move
/// queued (Shift) for each; the fifth Recruit, far off, ordered to repair it too. The ten crystals pay for 30 HP, a
/// crystal every 3 HP; at the next crystal, which the bank can't pay, the Repairs end, the far Recruit's on its way
/// included (**Placeholder (not in the design: whether the repairers on their way stop too)**), the Armory at 180 HP,
/// the top of the last step paid, and the bank at 0. The queued Moves start the next tick; the far Recruit, which had
/// nothing queued, stands. Nothing more is restored.
#[test]
fn the_repairs_end_when_the_bank_runs_out_and_the_queued_orders_go_on() {
    let mut sim = at_work(2, 150.0);
    let (a, b) = (RECRUITS[0], RECRUITS[1]);
    step(
        &mut sim,
        vec![
            Command::Debug(banks(10, 0)),
            order(&[a, b], OrderKind::Move(Target::Point(AWAY)), true),
            repair(&[FAR_RECRUIT], ARMORY),
        ],
    );
    let mut ticks = 1;
    let mut bank = crystals(&sim);
    while repairing(&sim, a) {
        run(&mut sim, 1);
        ticks += 1;
        assert!(crystals(&sim) <= bank, "the bank only goes down");
        bank = crystals(&sim);
        assert!(ticks < 20, "the bank pays for 12 ticks of two");
    }
    assert_eq!(ticks, 13, "12 ticks of 2.5 HP, then the crystal it can't pay");
    assert_eq!((hp(&sim, ARMORY), crystals(&sim)), (180.0, 0));
    for n in [a, b, FAR_RECRUIT] {
        assert!(!repairing(&sim, n), "{n}'s Repair ended");
    }
    assert!(
        sim.object(id(FAR_RECRUIT)).unwrap().pos.y > 1200.0,
        "the far Recruit stopped on its way"
    );
    run(&mut sim, 1);
    for n in [a, b] {
        assert!(matches!(current(&sim, n), Some(OrderKind::Move(_))), "{n} goes on");
    }
    assert_eq!(current(&sim, FAR_RECRUIT), None);
    run(&mut sim, 32);
    assert_eq!((hp(&sim, ARMORY), crystals(&sim)), (180.0, 0));
}

/// A Repair given with no crystals: the gate takes it (**Placeholder (not in the design: a Repair ordered with no
/// crystals)**: it doesn't judge the bank, which may fill on the way), the two Recruits walk to the Armory, and their
/// Repairs end as soon as one of them is at work, at the first crystal: nothing restored, nothing paid.
#[test]
fn a_repair_given_with_no_crystals_ends_at_work_restoring_nothing() {
    let mut sim = damaged(cults().sim(), ARMORY, 150.0);
    step(&mut sim, vec![Command::Debug(banks(0, 0))]);
    let units = &RECRUITS[..2];
    assert_eq!(gate(&sim, repair(units, ARMORY)), Ok(()));
    step(&mut sim, vec![repair(units, ARMORY)]);
    let mut ticks = 0;
    while units.iter().any(|&n| repairing(&sim, n)) {
        run(&mut sim, 1);
        ticks += 1;
        assert!(ticks < WALK, "the Repairs end");
    }
    assert!(ticks > 1, "they walked there first");
    assert_eq!((hp(&sim, ARMORY), crystals(&sim)), (150.0, 0));
}
