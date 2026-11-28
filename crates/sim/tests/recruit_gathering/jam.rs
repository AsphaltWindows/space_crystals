//! More Recruits than a face holds (slice M5-D): they starve one another, and that is the design's behaviour (design
//! change 0320, answering Q0110 (d); `factions/cults/units/recruit.md > Gathering`: "If more Recruits work one face
//! than it has lanes, they can block each other head-on for a long time. **The player's answer is more Storages**").
//! The two tests pin that starvation (slice M6-S6a; until then they asserted that no Recruit starved, and failed): the
//! focused repro, and the measurements by layout and count, which are slow and ignored. Within what the design says a
//! face holds, nothing jams (`crowds`).
//!
//! **What happens** (traced with four Recruits on one patch a tile from the Storage, [`face`] with one patch):
//! 1. A patch's 64 su face holds two lanes of 24 su Recruits, and the gap of one tile is one Recruit's walk long
//!    (`factions/cults/units/recruit.md > Gathering`: "A 64 su gap fits two Recruits side by side (one lane in, one
//!    lane out)"). Every leg heads for the point of the patch nearest to the Recruit (`orders::gather::legs`), so a
//!    Recruit shuttles straight up and down its own lane: two work a patch at the rate of two alone, three at 87% of
//!    it, and none of them goes 10 seconds without a delivery.
//! 2. With a fourth, two Recruits end up in one lane head-on, one with its load wanting the Storage behind the other,
//!    the other wanting the patch behind the first. Nobody yields: all four hold an order
//!    (`systems/units.md > UnitCollision > Ground Collision > Who yields`: "only units with no active command yield").
//! 3. Design change 0323's contact step ends the stand (slice M4-W3, after slice M5-D's measurements;
//!    `docs/movement.md > 6.3`): a blocked Recruit steps sideways by its own width, out of the lane, and gets round.
//!    So the four starve one another by turns. Each delivers 135 to 146 loads in the 5 minutes, but Recruit 6 goes 224
//!    ticks (14 seconds) without one, standing head-on on its way to the patch and again on its way back, past the jam
//!    check's 10 seconds (plan §3.3 D). The four bring 3.90 Space Crystals per second, the fourth adding a third of
//!    what a Recruit alone brings; more bring less, eight 2.82.
//! 4. A Recruit that presses on a face's two busy lanes from beside it can stand for good. In the 8-patch field with
//!    12 Recruits a tile from its three Storages, Recruit 12 delivers at Storage 9 in tick 2459 and heads for the end
//!    of patch 2's face nearest to it, where another Recruit shuttles in its lane. Its leg never ends, so it never
//!    looks for another patch, and it delivers nothing to the end of the run, 2,341 ticks, while the others work.
//!
//! It is the Agents' jam at a hatch (`tests/agent_gathering/jam.rs`, slice M4-W) without the hatch: there every
//! return heads for one point; here the Storage takes loads anywhere, so it needs more Recruits than lanes. **No
//! existing rule resolves it**, and the design adds none: "Recruits keep collision, and there is no give-way rule"
//! (design change 0320).
//!
//! **Measured as built** in slice M6-S6a (5 minutes, from the 10th second: Space Crystals per second / the longest any
//! Recruit went without a delivery, ticks; a gap of 1 tile; [`face`]'s Storage turned so that its 2-tile short side
//! faces the patches, as the 2x2 Storage's side did):
//!
//! | Recruits | 2 | 3 | 4 | 5 | 6 | 7 | 8 |
//! |---|---|---|---|---|---|---|---|
//! | One patch | 2.66 / 45 | 3.48 / 104 | 3.90 / 224 | 3.26 / 208 | 3.21 / 292 | 3.12 / 325 | 2.82 / 351 |
//! | Two patches, the face | 2.66 / 24 | 3.94 / 41 | 5.29 / 63 | 5.46 / 205 | 4.86 / 351 | 4.83 / 265 | 4.47 / 592 |
//!
//! The 8-patch field with three Storages, 10 / 12 / 14 / 16 / 20 Recruits: 12.9 / 206, 13.5 / 2341, 17.3 / 502,
//! 19.2 / 687, 17.0 / 889. At gaps of 2 and 3 tiles the lanes are long enough to pass in, and crowds mostly slow
//! down: the face with two patches gives 2.73 / 107 with 5 Recruits and 3.25 / 165 with 8, the field 9.27 / 271 with
//! 20 at 2 tiles. Slice M5-D measured before 0323, with the 2x2 Storage, two of the Recruits standing for minutes: one
//! patch with 4 / 5 / 8 Recruits 2.30 / 2023, 1.46 / 3086, 0.41 / 4585; two patches with 6 / 8, 3.94 / 881,
//! 2.19 / 2489.
//!
//! **Design change 0315's rule for Agents, prototyped for Recruits and not adopted** (slice M5-D, before 0323; the
//! Director's note on 0315: the design doesn't extend it to Recruits). With a return leg that never counts as arrived
//! short of the Storage, and begins again at once when it fails, the same cases still jam: one patch with 4 / 5 / 6
//! Recruits 2.33 / 1513, 1.29 / 2248, 1.61 / 4122; two patches with 6 / 7 / 8 4.52 / 594, 2.66 / 2375, 2.18 / 4229;
//! the field with 20 12.8 / 2624. With the patch leg exempt too: 2.05 / 1986, 1.42 / 1601, 1.01 / 4640; 5.63 / 320,
//! 3.26 / 1503, 2.06 / 3357; 13.8 / 2864. Of 99 cases, 44 had a Recruit 10 seconds without a delivery as built, 41
//! and 37 with the prototypes: the Recruits' jam isn't a leg that arrives short, it is two Recruits head-on in a lane
//! with no room to pass, and "keep moving" doesn't make either give way.

use super::*;

/// One Recruit alone a tile from the Storage (`trips`): a load of 2 every 24 ticks, Space Crystals per second.
const ALONE: f32 = 32.0 / 24.0;

/// The focused repro (module docs): four Recruits on one patch a tile from the Storage starve one another by turns.
/// Three work it with none 10 seconds without a delivery. Of four, one goes the jam check's 10 seconds without one
/// (as built, Recruit 6: 224 ticks), yet each delivers throughout (135 to 146 loads from the 10th second), and the
/// fourth adds less than half of what a Recruit alone brings (as built, 3.48 Space Crystals per second with three and
/// 3.90 with four, where one alone brings 1.33).
#[test]
fn four_recruits_on_one_patch_starve_one_another_by_turns() {
    let five_minutes = |count| {
        let (mut t, recruits) = face_at_work(count, 1, 1, 0);
        t.run_until(FIVE_MINUTES);
        let all = deliveries(&t);
        let loads: Vec<usize> = (recruits.iter())
            .map(|&n| all.iter().filter(|d| d.1 == n && d.0 > SETTLED).count())
            .collect();
        let (longest, who) = longest_gap(&t, &recruits, SETTLED, FIVE_MINUTES);
        (income(&t, SETTLED, FIVE_MINUTES), longest, who, loads)
    };
    let (three, three_longest, _, _) = five_minutes(3);
    let (four, longest, who, loads) = five_minutes(4);
    println!(
        "three: {three:.2} Space Crystals per second, longest gap {three_longest} ticks; four: {four:.2}, \
         Recruit {who} {longest} ticks without a delivery, loads {loads:?}"
    );
    assert!(three_longest < JAM, "three: {three_longest} ticks without a delivery");
    assert!(
        longest >= JAM,
        "four: none went 10 seconds without a delivery (at most {longest} ticks): the jam is gone (module docs)"
    );
    assert!(loads.iter().all(|&n| n >= 100), "four: {loads:?} loads: one stands");
    assert!(
        four - three < ALONE / 2.0,
        "four: {four:.2} Space Crystals per second, three {three:.2}"
    );
}

// The layouts of `recruits_starve_on_an_overloaded_face_and_never_within_its_lanes`: `face` with one patch facing the
// Storage, with two, the whole face, and with one patch to the west or the east of it, and the 8-patch field with
// three Storages (`field_at_work`).
const ONE: &str = "one patch";
const TWO: &str = "two patches";
const WEST: &str = "one patch to the west";
const EAST: &str = "one patch to the east";
const FIELD: &str = "the field";

/// One five-minute run of [`recruits_starve_on_an_overloaded_face_and_never_within_its_lanes`].
#[derive(Debug)]
struct Run {
    layout: &'static str,
    patches: u32,
    gap: i32,
    recruits: u32,
    /// Space Crystals per second.
    income: f32,
    /// The longest any Recruit went without a delivery, ticks.
    longest: Tick,
}

/// The measurements, as slice M4-W's: 5 minutes of 1 to 8 Recruits on one face, with one patch and with two, facing
/// the Storage and to the side of it, at gaps of 1 to 3 tiles, and of 10 to 20 on the 8-patch field with three
/// Storages: Space Crystals per second, and the longest any Recruit went without a delivery. They pin where Recruits
/// starve, that is, go the jam check's 10 seconds without a delivery (plan §3.3 D):
/// - never within a face's lanes, two Recruits a patch, at any gap;
/// - a tile from the Storage, always with more than a face holds: from 4 Recruits on one patch, and from 6 on two,
///   the whole face (`factions/cults/structures/storage.md`: "each 2-tile face supports roughly 4–5 Recruits at full
///   efficiency"), and from 12 on the field, where one stands for minutes (module docs).
///
/// A tile from the Storage, more Recruits than a face holds also bring less: 8 less than the face's best. At 2 and 3
/// tiles crowds only slow down: 8 bring more than 4 on every face.
#[test]
#[ignore = "slow: 106 runs of 5 minutes, about 13 seconds in release"]
fn recruits_starve_on_an_overloaded_face_and_never_within_its_lanes() {
    let mut runs = Vec::new();
    let mut run = |layout: &'static str, patches: u32, gap: i32, (mut t, recruits): (TestSim, Vec<u32>)| {
        t.run_until(FIVE_MINUTES);
        let (longest, _) = longest_gap(&t, &recruits, SETTLED, FIVE_MINUTES);
        let income = income(&t, SETTLED, FIVE_MINUTES);
        let count = recruits.len() as u32;
        println!("{layout}, {gap} tiles, {count} Recruits: {income:5.2} per second, longest gap {longest:4} ticks");
        runs.push(Run {
            layout,
            patches,
            gap,
            recruits: count,
            income,
            longest,
        });
    };
    for (layout, patches, shift) in [(ONE, 1, 0), (TWO, 2, 0), (WEST, 1, -2), (EAST, 1, 2)] {
        for gap in 1..=3 {
            for count in 1..=8 {
                run(layout, patches, gap, face_at_work(count, gap, patches, shift));
            }
        }
    }
    for gap in 1..=2 {
        for count in [10, 12, 14, 16, 20] {
            run(FIELD, 8, gap, field_at_work(count, gap));
        }
    }
    let mut changed = Vec::new();
    for r in &runs {
        let within_lanes = r.layout != FIELD && r.recruits <= 2 * r.patches;
        let overloaded = r.gap == 1
            && [(ONE, 4), (TWO, 6), (FIELD, 12)]
                .iter()
                .any(|&(layout, from)| r.layout == layout && r.recruits >= from);
        if within_lanes && r.longest >= JAM {
            changed.push(format!("{r:?}: within the face's lanes, yet a Recruit starved"));
        }
        if overloaded && r.longest < JAM {
            changed.push(format!("{r:?}: more than the face holds, yet none starved"));
        }
    }
    let measured = |layout: &str, gap: i32, recruits: u32| {
        (runs.iter())
            .find(|r| (r.layout, r.gap, r.recruits) == (layout, gap, recruits))
            .expect("measured")
            .income
    };
    for layout in [ONE, TWO] {
        let best = (1..=8).map(|n| measured(layout, 1, n)).fold(0.0, f32::max);
        if measured(layout, 1, 8) >= best {
            changed.push(format!(
                "{layout}, a tile from the Storage: 8 Recruits bring the most, {best:.2}"
            ));
        }
    }
    for layout in [ONE, TWO, WEST, EAST] {
        for gap in 2..=3 {
            let (four, eight) = (measured(layout, gap, 4), measured(layout, gap, 8));
            if eight <= four {
                changed.push(format!(
                    "{layout}, {gap} tiles: 8 Recruits bring {eight:.2}, 4 {four:.2}"
                ));
            }
        }
    }
    assert!(
        changed.is_empty(),
        "the jam changed (module docs):\n{}",
        changed.join("\n")
    );
}
