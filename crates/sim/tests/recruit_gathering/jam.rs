//! More Recruits than a face holds (slice M5-D): they starve one another, and at the worst the face nearly stops.
//! Both tests are ignored: they fail as built, for the reason question Q0109 asks about (who gives way among workers
//! that all hold an order), and the answer to it is design. Within what the design says a face holds, nothing jams
//! (`crowds`).
//!
//! **What happens** (traced with four Recruits on one patch a tile from the Storage, [`face`] with one patch):
//! 1. A patch's 64 su face holds two lanes of 24 su Recruits, and the gap of one tile is one Recruit's walk long
//!    (`factions/cults/units/recruit.md > Gathering`: "A 64 su gap fits two Recruits side by side (one lane in, one
//!    lane out)"). Every leg heads for the point of the face nearest to the Recruit (`orders::gather::legs`), so a
//!    Recruit shuttles straight up and down its own lane, and two or three work a patch at the rate of one alone.
//! 2. With a fourth, two Recruits end up in one lane head-on: one with its load wanting the Storage behind the
//!    other, the other wanting the patch behind the first, and a working Recruit shuttling on either side, so that
//!    neither can step round. Their legs end short (`systems/controls/behaviors.md > MovingToLocation > Occupied
//!    target location`); the loaded one tries again every `tuning::gathering::LEG_RETRY_TICKS`, the other waits at
//!    "its patch" out of reach and tries again every `tuning::gathering::RECHECK_TICKS` (design change 0141), both
//!    for the same point, for ever.
//! 3. Nobody yields: all four hold an order (`systems/units.md > UnitCollision > Ground Collision > Who yields`:
//!    "only units with no active command yield"). The two that work go on; the two that stand deliver nothing until
//!    a neighbour's jitter happens to open a lane, minutes later.
//!
//! It is the Agents' jam at a hatch (`tests/agent_gathering/jam.rs`, slice M4-W) without the hatch: there every
//! return heads for one point; here the Storage takes loads anywhere, so it needs more Recruits than lanes. **No
//! existing rule resolves it**, and a new one (who makes way for whom) is design: Q0109. Slice M5-D measured and
//! stopped here.

use super::*;

/// The focused repro (module docs): four Recruits on one patch a tile from the Storage. As built two of them work,
/// and Recruit 6 delivers nothing for 2,000 ticks (over 2 minutes); the face gives 2.4 Space Crystals per second
/// where three Recruits give 3.8.
#[test]
#[ignore = "M5-D: fails as built (more Recruits than a face has lanes); the fix needs Q0109's answer"]
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
#[ignore = "M5-D: fails as built (more Recruits than a face has lanes); the fix needs Q0109's answer"]
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
