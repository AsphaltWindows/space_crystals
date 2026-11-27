//! A vehicle's repair by its real recipe (slice M5-N6: the Chop Shop's Seeker, "2 Recruits, 75 Space Crystals, 320
//! frames"; `factions/cults/units/recruit.md > Repair`: "Vehicle repair costs a share of its **Space Crystals only**,
//! not the Recruits it consumed"): from 0 to full in half its 320 frames with one Recruit, for half its 75 Space
//! Crystals, so a Recruit restores 120 / 160 = 0.75 HP a tick and a crystal buys 120 / 37.5 = 3.2 HP.

use super::*;

/// Player 1's Seeker on a flat 40 × 30 map.
const SEEKER: u32 = 2;
/// Player 1's two Recruits, west of it.
const REPAIRERS: [u32; 2] = [3, 4];
/// What a Recruit at work on a Seeker restores in a tick.
const SEEKER_RATE: f32 = 0.75;

/// Player 1's Storage (1), Seeker (2) and two Recruits (3, 4), and player 2's Storage, which keeps them in the game, both
/// players the Cults: [`BANK`] Space Crystals for player 1 and auto-targeting off, from tick 1, which has run.
fn vehicle() -> TestSim {
    let mut t = ScenarioBuilder::new(&flat(40, 30))
        .faction(P1, Faction::Cults)
        .faction(P2, Faction::Cults)
        .spawn(ObjectKind::Storage, Some(P1), tile(2, 2))
        .spawn(ObjectKind::Seeker, Some(P1), point(900.0, 800.0))
        .spawn(ObjectKind::Recruit, Some(P1), point(780.0, 780.0))
        .spawn(ObjectKind::Recruit, Some(P1), point(780.0, 830.0))
        .spawn(ObjectKind::Storage, Some(P2), tile(34, 24))
        .start();
    t.debug_at(1, banks(BANK, 0));
    t.debug_at(1, DebugCommand::SetAutoTargeting { on: false });
    t.run(1);
    t
}

/// The whole crystals a Seeker's repair has paid by `hp` HP from 0 (`repair::payment`: a crystal at each step of 3.2
/// HP that the HP enters).
fn steps(hp: f32) -> u32 {
    (hp * 37.5 / 120.0).ceil() as u32
}

/// The plan's acceptance (slice M5-N6): "one Recruit repairs a Seeker from 60 to 120 HP in 80 frames, and two in 40.
/// Either way it costs 19 SC, one per 3.2 HP (half its 75 SC over its 120 HP, in whole crystals)". The repair's terms
/// are the Chop Shop's recipe's (`sc_sim::repair::terms`: 160 frames and 37.5 Space Crystals from 0 to full), with no
/// test recipe; from 60 HP with its repairers at work, `n` Recruits restore `n` × 0.75 HP a tick, so the 60 HP take
/// 80 / `n` ticks, and the bank pays a crystal each time the HP enters the next 3.2 HP: 38 by full HP from 0, 19 by 60,
/// so 19 whoever repairs. At full HP they stop.
#[test]
fn one_recruit_repairs_a_seeker_from_60_to_120_hp_in_80_frames_and_two_in_40_for_19_crystals() {
    let terms = sc_sim::repair::terms(ObjectKind::Seeker).map(|terms| (terms.frames, terms.price));
    assert_eq!(
        terms,
        Some((160.0, Some(37.5))),
        "half the Chop Shop's 320 frames and 75 Space Crystals"
    );
    assert_eq!((steps(60.0), steps(120.0)), (19, 38));
    for n in 1..=2 {
        let mut sim = damaged(vehicle().sim(), SEEKER, 1.0);
        step(&mut sim, vec![repair(&REPAIRERS[..n], SEEKER)]);
        let rate = SEEKER_RATE * n as f32;
        until_gaining(&mut sim, SEEKER, rate, |_| {});
        let mut sim = damaged(&sim, SEEKER, 60.0);
        let bank = crystals(&sim);
        let paid = |sim: &Sim| {
            let now = hp(sim, SEEKER);
            assert_eq!(bank - crystals(sim), steps(now) - steps(60.0), "{n}: at {now} HP");
        };
        let ticks = restored_to(&mut sim, SEEKER, 120.0, rate, paid);
        assert_eq!(ticks, 80 / n as u32, "{n} Recruits");
        assert_eq!(bank - crystals(&sim), 19, "{n} Recruits");
        for recruit in &REPAIRERS[..n] {
            assert_eq!(current(&sim, *recruit), None, "{n}: {recruit} stopped at full HP");
        }
    }
}
