//! More Recruits than a face holds (slice M5-D): they starve one another, and at the worst the face nearly stops.
//! Both tests are ignored: they fail as built, and that is the design's behaviour (design change 0320, answering
//! Q0110 (d); `factions/cults/units/recruit.md > Gathering`: "If more Recruits work one face than it has lanes, they
//! can block each other head-on for a long time. **The player's answer is more Storages**"). They stay as the record
//! of what overloading a face does. Within what the design says a face holds, nothing jams (`crowds`).
//!
//! **What happens** (traced with four Recruits on one patch a tile from the Storage, [`face`] with one patch):
//! 1. A patch's 64 su face holds two lanes of 24 su Recruits, and the gap of one tile is one Recruit's walk long
//!    (`factions/cults/units/recruit.md > Gathering`: "A 64 su gap fits two Recruits side by side (one lane in, one
//!    lane out)"). Every leg heads for the point of the face nearest to the Recruit (`orders::gather::legs`), so a
//!    Recruit shuttles straight up and down its own lane, and two or three work a patch at the rate of one alone.
//! 2. With a fourth, two Recruits end up in one lane head-on: one with its load wanting the Storage behind the
//!    other, the other wanting the patch behind the first, and a working Recruit shuttling on either side, so that
//!    neither can step round. Their legs end short (`systems/controls/command_behaviors.md > MovingToLocation >
//!    Occupied target location`); the loaded one tries again every `tuning::gathering::LEG_RETRY_TICKS`, the other
//!    waits at "its patch" out of reach and tries again every `tuning::gathering::RECHECK_TICKS` (design change 0141),
//!    both for the same point, for ever.
//! 3. Nobody yields: all four hold an order (`systems/units.md > UnitCollision > Ground Collision > Who yields`:
//!    "only units with no active command yield"). The two that work go on; the two that stand deliver nothing until
//!    a neighbour's jitter happens to open a lane, minutes later.
//!
//! It is the Agents' jam at a hatch (`tests/agent_gathering/jam.rs`, slice M4-W) without the hatch: there every
//! return heads for one point; here the Storage takes loads anywhere, so it needs more Recruits than lanes. **No
//! existing rule resolves it**, and the design adds none: "Recruits keep collision, and there is no give-way rule"
//! (design change 0320). Slice M5-D measured and stopped here.
//!
//! **Measured as built** in slice M5-D, with the 2x2 Storage of the time (5 minutes, from the 10th second: Space
//! Crystals per second / the longest any Recruit went without a delivery, ticks; a gap of 1 tile):
//!
//! | Recruits | 2 | 3 | 4 | 5 | 6 | 7 | 8 |
//! |---|---|---|---|---|---|---|---|
//! | One patch facing the Storage | 2.65 / 32 | 3.77 / 110 | 2.30 / 2023 | 1.46 / 3086 | 1.60 / 3287 | 0.42 / 4597 | 0.41 / 4585 |
//! | Two patches, the whole face | 2.66 / 24 | 3.99 / 40 | 5.32 / 40 | 6.10 / 132 | 3.94 / 881 | 3.00 / 1193 | 2.19 / 2489 |
//!
//! The 8-patch field with three Storages, 10 / 12 / 14 / 16 / 20 Recruits: 12.9 / 107, 14.7 / 172, 15.7 / 471,
//! 15.8 / 965, 13.6 / 2618. At a gap of 2 tiles the lanes are long enough to pass in, and crowds only slow down:
//! the face with two patches gives 2.74 / 79 with 5 Recruits and 3.23 / 208 with 8, the field 8.9 / 266 with 20.
//!
//! **Design change 0315's rule for Agents, prototyped for Recruits and not adopted** (the Director's note on 0315: the
//! design doesn't extend it to Recruits). With a return leg that never counts as arrived short of the Storage, and
//! begins again at once when it fails, the same cases still jam: one patch with 4 / 5 / 6 Recruits 2.33 / 1513,
//! 1.29 / 2248, 1.61 / 4122; two patches with 6 / 7 / 8 4.52 / 594, 2.66 / 2375, 2.18 / 4229; the field with 20
//! 12.8 / 2624. With the patch leg exempt too: 2.05 / 1986, 1.42 / 1601, 1.01 / 4640; 5.63 / 320, 3.26 / 1503,
//! 2.06 / 3357; 13.8 / 2864. Of 99 cases, 44 have a Recruit 10 seconds without a delivery as built, 41 and 37 with
//! the prototypes: the Recruits' jam isn't a leg that arrives short, it is two Recruits head-on in a lane with no
//! room to pass, and "keep moving" doesn't make either give way.

use super::*;

/// The focused repro (module docs): four Recruits on one patch a tile from the Storage. As built two of them work,
/// and Recruit 6 delivers nothing for 2,000 ticks (over 2 minutes); the face gives 2.4 Space Crystals per second
/// where three Recruits give 3.8.
#[test]
#[ignore = "M5-D: fails as built (more Recruits than a face has lanes), the design's behaviour (0320)"]
fn four_recruits_on_one_patch_keep_delivering() {
    let (mut t, recruits) = face_at_work(4, 1, 1, 0);
    t.run_until(FIVE_MINUTES);
    let (longest, who) = longest_gap(&t, &recruits, SETTLED, FIVE_MINUTES);
    assert!(
        longest < JAM,
        "Recruit {who} delivered nothing for {longest} ticks; {:.2} Space Crystals per second",
        income(&t, SETTLED, FIVE_MINUTES)
    );
}

/// The measurements, as slice M4-W's: 5 minutes of 1 to 8 Recruits on one face, with one patch and with two, facing
/// the Storage and to the side of it, at gaps of 1 to 3 tiles, and of 10 to 20 on the 8-patch field with three
/// Storages: Space Crystals per second, and the longest any Recruit went without a delivery. It fails on every case
/// in which a Recruit goes 10 seconds without one (the jam check, plan §3.3 D).
#[test]
#[ignore = "M5-D: fails as built (more Recruits than a face has lanes), the design's behaviour (0320)"]
fn no_recruit_goes_ten_seconds_without_a_delivery_however_many_work_a_face() {
    let mut jams = Vec::new();
    let mut note = |what: String, t: &TestSim, recruits: &[u32]| {
        let (longest, _) = longest_gap(t, recruits, SETTLED, FIVE_MINUTES);
        let income = income(t, SETTLED, FIVE_MINUTES);
        println!("{what}: {income:5.2} per second, longest gap {longest:4} ticks");
        if longest >= JAM {
            jams.push((what, longest));
        }
    };
    for (name, patches, shift) in [
        ("one patch", 1, 0),
        ("two patches", 2, 0),
        ("one patch to the west", 1, -2),
        ("one patch to the east", 1, 2),
    ] {
        for gap in 1..=3 {
            for count in 1..=8 {
                let (mut t, recruits) = face_at_work(count, gap, patches, shift);
                t.run_until(FIVE_MINUTES);
                note(format!("{name}, {gap} tiles, {count} Recruits"), &t, &recruits);
            }
        }
    }
    for gap in 1..=2 {
        for count in [10, 12, 14, 16, 20] {
            let (mut t, recruits) = field_at_work(count, gap);
            t.run_until(FIVE_MINUTES);
            note(format!("the field, {gap} tiles, {count} Recruits"), &t, &recruits);
        }
    }
    assert_eq!(jams, [], "(the case, the longest gap in ticks)");
}
