//! The rate and the price (`factions/cults/units/recruit.md > Repair`, design change 0321): "several repairers on one
//! target repair **faster, scaling linearly** (2 repairers = twice the rate), as construction does"; "repairing from 0
//! to full HP takes **half the target's build time** with one repairer, and costs **half its price** in the resource
//! it was bought with, paid gradually as HP is restored".

use super::*;

/// Design change 0321's verification: "two Recruits repair an Armory from 150 to 300 HP in 60 frames, one in 120.
/// Crystals drop one per 3 HP, 50 in total", and the rate is linear up to four (the Armory's 480 frames and 200 Space
/// Crystals: 240 frames and 100 from 0 to full with one). From 150 HP with all of them at work, `n` Recruits restore
/// `n` × 1.25 HP a tick, so the 150 HP take 120 / `n` ticks; the bank pays the crystal of every 3 HP as it is entered,
/// 50 in all whoever repairs. At full HP they stop (design change 0329: "repairers **stop once the target is at full
/// HP**"), and it stays at full, nothing more paid.
#[test]
fn one_to_four_recruits_repair_an_armory_linearly_for_a_crystal_every_3_hp() {
    for n in 1..=4 {
        let mut sim = at_work(n, 150.0);
        let bank = crystals(&sim);
        let rate = ARMORY_RATE * n as f32;
        let paid = |sim: &Sim| {
            let restored = hp(sim, ARMORY) - 150.0;
            assert_eq!(
                bank - crystals(sim),
                (restored / 3.0).ceil() as u32,
                "{n}: at {restored} restored"
            );
        };
        let ticks = restored_to(&mut sim, ARMORY, 300.0, rate, paid);
        assert_eq!(ticks, 120 / n as u32, "{n} Recruits");
        assert_eq!(bank - crystals(&sim), 50, "{n} Recruits");
        for recruit in &RECRUITS[..n] {
            assert_eq!(current(&sim, *recruit), None, "{n}: {recruit} stopped at full HP");
        }
        run(&mut sim, 16);
        assert_eq!((hp(&sim, ARMORY), bank - crystals(&sim)), (300.0, 50), "{n} Recruits");
    }
}

/// Each object is repaired by its own terms: the Storage's build is 100 Space Crystals and 320 frames
/// (`factions/cults/units/recruit.md > Constructs`), so 50 crystals and 160 frames from 0 to full, a crystal for every
/// 5 of its 250 HP. Two Recruits repair the Armory from 150 HP and two the Storage from 125 at once: 50 crystals for
/// the one and 25 for the other, and all four stop.
#[test]
fn each_object_is_repaired_by_its_own_terms() {
    let sim = damaged(cults().sim(), ARMORY, 150.0);
    let mut sim = damaged(&sim, STORAGE, 125.0);
    let bank = crystals(&sim);
    step(
        &mut sim,
        vec![repair(&RECRUITS[..2], ARMORY), repair(&RECRUITS[2..], STORAGE)],
    );
    let mut ticks = 0;
    while hp(&sim, ARMORY) < 300.0 || hp(&sim, STORAGE) < 250.0 {
        run(&mut sim, 1);
        ticks += 1;
        assert!(ticks < WALK, "both repaired");
    }
    assert_eq!(
        bank - crystals(&sim),
        50 + 25,
        "half of 300 HP at 3 a crystal, half of 250 at 5"
    );
    assert!(RECRUITS.iter().all(|&r| current(&sim, r).is_none()), "all four stopped");
}
