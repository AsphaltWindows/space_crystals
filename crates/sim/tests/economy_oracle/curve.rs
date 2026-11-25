//! The saturation curve of the reference field (slices M3-K1 and M3-K4): its income with a number of Drones, against
//! the design's. Since design change 0236 (answering Q0055 and Q0072) the design's curve is slice M3-K3's
//! measurement: `factions/colonists/units.md > Drone > Patch Occupancy`, "8 patches in a row with the Egress Node 2
//! grid units from the row [...] saturate at about 24 working Drones (3 per patch, 32 Space Crystals per second
//! [...]). 2 per patch (16 Drones, ≈25 per second) is the efficient point. Measured quirk, accepted for now: Drones
//! 17–20 add almost nothing (≈26 at 20), and 21–24 add the rest, because third Drones mostly wait their turn until
//! every patch has one", where "one patch yields at most 8 crystals per 32 frames (4 Space Crystals per second)";
//! and `factions/gdo/economy.md > Economy Targets (GDO vs Colonists)`, 6.9 per second for the four starting Drones
//! (the table's 0:00) and "Colonists fully saturated = 32 per second (24 working Drones)".
//!
//! The Drones stand beside the node and are dealt out over the patches as `tests/gathering_soak.rs` deals them (it
//! measured the curve first, slices M3-H to M3-H3), and the income is what they deliver from 50 seconds on, once the
//! field has settled. **Measured** over the soak's 4 minutes 10 seconds (the ignored `saturation_curve`, in release),
//! with the Drone's numbers of design change 0205 (slice M3-K3), per second: 6.91 with 4 Drones, 12.67 with 8,
//! 18.75 with 12, 25.34 with 16, 25.34 with 18, 25.95 with 20, 26.78 with 22, and 32.00 from 24 on: the row is
//! mined without a break, 8 patches at 4 per second, with the design's 24 Drones. On the default test's 100 seconds
//! the design's figures measure 6.88 with 4 Drones (−0.3% on the 6.9), 25.20 with 16 (+0.8% on the ≈25), 26.00 with
//! 20 (+0.0% on the ≈26) and 32.00 with 24. From 16 to 22 Drones the income hardly grows, the plateau the design
//! accepts: the Drones are dealt out from the middle patches, so the 17th to 22nd are third Drones on the middle
//! six, which two Drones already keep busy most of the time (a trip there is 73 to 83 ticks, two Drones' mining 64);
//! a third mostly waits, or is sent on from patch to patch and loses time (`tests/gathering_soak.rs`, module docs).
//! The 23rd and 24th give the end patches, whose trips of 95 ticks three Drones' 96 of mining just cover, their
//! third, and the row is mined without a break. Sent with one command, 24 Drones give 29.60 and 25 on 32.00. Target
//! 4 holds at full saturation: 32 against the GDO's 26.67.
//!
//! **Before design change 0236** the design estimated 6.1 per second for the four starting Drones (the table's
//! 0:00), "≈ 23" at 2 Drones per patch and "≈ 32 per second (~24 working Drones)" fully saturated (`> Reference
//! check`), and "roughly 18–22 working Drones" to saturate the row (`> Patch Occupancy`): the curve above was within
//! ±15% of the first three (+13.3%, +10.2% and +0.0%), and the 18–22 was low (Q0072).
//!
//! **Before design change 0205** (slice M3-K1, Q0055): 6.34 with 4 Drones, 11.65 with 8, 17.22 with 12, 23.10 with
//! 16, 24.45 with 20, 26.24 with 24 (**−18.0%** on the 32), 26.91 with 25, 30.72 with 26, and 32.00 from 27 on: the
//! end patches' trips of 104 ticks were longer than three Drones' mining covered, and target 4 failed at 24 Drones.

use sc_sim::testing::TestSim;
use sc_sim::testing::colonists::{FIELD_PATCHES, FIELD_PLAYER, field_drones, reference_field};
use sc_sim::{ObjectId, OrderKind, SimConfig, SimEvent, TICKS_PER_SECOND, Tick};

use super::TOLERANCE;

/// The field settles over its first 50 seconds (`tests/gathering_soak.rs`).
const SETTLED: Tick = 800;
/// The soak's measurement: to its 5 minutes.
const SOAK: Tick = 4000;
/// The default test's: 100 seconds.
const SHORT: Tick = 1600;
/// The GDO's income fully plated: 8 plates at 10 Space Crystals per 48 frames (`factions/gdo/economy.md >
/// ExtractionPlate > MiningRate`).
const GDO_FULL: f64 = 8.0 * 10.0 * 16.0 / 48.0;
/// The row mined without a break: 8 patches, each yielding "at most 8 crystals per 32 frames (4 Space Crystals per
/// second)" (`> Patch Occupancy`).
const ROW_FULL: f64 = 8.0 * 4.0;

/// The income of the reference field with `drones` Drones, dealt out over its patches as `tests/gathering_soak.rs`
/// deals them: those west of the node over the western patches and those east of it over the eastern ones, the
/// middle patches first; or, unless `spread`, all sent to the middle patch with one command, which the occupancy rule
/// deals out. Space Crystals per second, over the `measured` ticks after [`SETTLED`].
fn income(drones: u32, spread: bool, measured: Tick) -> f64 {
    let mut t = TestSim::with_config(reference_field(drones), SimConfig::default());
    let middle = FIELD_PATCHES.div_ceil(2);
    let all = field_drones(drones);
    if !spread {
        t.order_at(1, FIELD_PLAYER, &all, OrderKind::Gather(ObjectId(middle)), false);
    }
    let (western, eastern) = (middle, FIELD_PATCHES - middle);
    for (i, &drone) in all.iter().enumerate().filter(|_| spread) {
        let turn = i as u32 / 2;
        let patch = if i % 2 == 0 {
            middle - turn % western
        } else {
            middle + 1 + turn % eastern
        };
        t.order_at(1, FIELD_PLAYER, &[drone], OrderKind::Gather(ObjectId(patch)), false);
    }
    t.run_until(SETTLED + measured);
    let delivered: u32 = t
        .events()
        .iter()
        .filter(|(tick, _)| *tick > SETTLED)
        .filter_map(|(_, e)| match e {
            SimEvent::ResourcesDelivered { amount, .. } => Some(*amount),
            _ => None,
        })
        .sum();
    f64::from(delivered) * f64::from(TICKS_PER_SECOND) / f64::from(measured)
}

/// The design's incomes by Drones below saturation (module docs, design change 0236): the four starting Drones' at
/// the table's 0:00, and `> Patch Occupancy`'s 16 Drones, 2 per patch ("the efficient point"), and 20, on the
/// plateau. (`> Reference check`'s "Colonists at 2 Drones per patch ≈ 23 Space Crystals per second for 18 Drones" is
/// the reference line's at 2:20, with Drones still on their first trip: `colonists.rs`.)
const DESIGN: [(u32, f64); 3] = [(4, 6.9), (16, 25.0), (20, 26.0)];
/// "about 24 working Drones (3 per patch, 32 Space Crystals per second)" saturate the row: [`ROW_FULL`], to less
/// than a load (8 Space Crystals, 0.08 per second over 100 seconds) as the measure starts and ends mid-trip.
const SATURATED: u32 = 24;

/// The design's curve on 100 seconds of the dealt-out field (module docs): its incomes for 4, 16 and 20 Drones
/// within ±15% (design change 0108), and saturation at 24 Drones, 3 per patch: every patch mined without a break,
/// 32 per second, above the GDO's fully plated 26.67 (target 4).
#[test]
fn the_saturation_curve_against_the_design() {
    for (drones, design) in DESIGN {
        let measured = income(drones, true, SHORT);
        let off = measured / design - 1.0;
        println!(
            "{drones:2} Drones: {measured:5.2} per second, the design's {design}: {:+.1}%",
            100.0 * off
        );
        assert!(
            off.abs() <= TOLERANCE,
            "{drones} Drones: {measured:.2} per second against the design's {design}, {:+.1}%: a question for the \
             Designer (design change 0108)",
            100.0 * off
        );
    }
    let full = income(SATURATED, true, SHORT);
    println!("{SATURATED} Drones: {full:5.2} per second, the row {ROW_FULL}, the GDO fully plated {GDO_FULL:.2}");
    assert!(
        (ROW_FULL - 0.1..=ROW_FULL).contains(&full),
        "{SATURATED} Drones, 3 per patch, mine the row without a break: {full:.2}"
    );
    assert!(
        full > GDO_FULL,
        "target 4: the Colonists' full saturation above the GDO's"
    );
}

/// The curve as the module docs give it, printed: the income by the number of Drones, dealt out and sent with one
/// command, over the soak's 4 minutes 10 seconds.
#[test]
#[ignore = "a measurement: prints the saturation curve; run it in release"]
fn saturation_curve() {
    for spread in [true, false] {
        println!("8 patches, {}:", if spread { "dealt out" } else { "one command" });
        for drones in [4, 8, 12, 16, 18, 20, 22, 24, 25, 26, 27, 28, 32] {
            println!("  {drones:2} Drones: {:5.2} per second", income(drones, spread, SOAK));
        }
    }
}
