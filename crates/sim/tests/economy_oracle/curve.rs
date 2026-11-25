//! The saturation curve of the reference field (slice M3-K1): its income with a number of Drones, against the design's
//! estimates. `factions/gdo/economy.md > Economy Targets (GDO vs Colonists) > Reference check`: 6.1 per second for
//! the four starting Drones (the table's 0:00), "Colonists at 2 Drones per patch ≈ 23 Space Crystals per second", and
//! "Colonists fully saturated ≈ 32 per second (~24 working Drones)"; `factions/colonists/units.md > Drone > Patch
//! Occupancy`: "8 patches in a row with the Egress Node 2 grid units from the row [...] saturate at roughly 18–22
//! working Drones", where "one patch yields at most 8 crystals per 32 frames (4 Space Crystals per second)".
//!
//! The Drones stand beside the node and are dealt out over the patches as `tests/gathering_soak.rs` deals them (it
//! measured the curve first, slices M3-H to M3-H3), and the income is what they deliver from 50 seconds on, once the
//! field has settled. **Measured** over the soak's 4 minutes 10 seconds (the ignored `saturation_curve`, in release),
//! per second: 6.34 with 4 Drones (+3.9% on the 6.1), 11.65 with 8, 17.22 with 12, 23.10 with 16 (+0.4% on the 23),
//! 24.45 with 20, 26.24 with 24 (**−18.0%** on the 32), 26.91 with 25, 30.72 with 26, and 32.00 from 27 on: the row
//! is mined without a break, 8 patches at 4 per second, with 27 Drones, not about 24 (nor 18 to 22). With 24 the end
//! patches, whose trips are the longest (104 ticks, against 81 at the middle), aren't kept busy, and the Drones sent
//! on from an occupied patch lose time (`tests/gathering_soak.rs`, module docs). Target 4 holds at full saturation
//! (32 against the GDO's 26.67) and fails at 24 Drones (`colonists.rs`).

use sc_sim::testing::TestSim;
use sc_sim::testing::colonists::{FIELD_PATCHES, FIELD_PLAYER, field_drones, reference_field};
use sc_sim::{ObjectId, OrderKind, SimConfig, SimEvent, TICKS_PER_SECOND, Tick};

/// The field settles over its first 50 seconds (`tests/gathering_soak.rs`).
const SETTLED: Tick = 800;
/// The soak's measurement: to its 5 minutes.
const SOAK: Tick = 4000;
/// The default test's: 100 seconds.
const SHORT: Tick = 1600;
/// The GDO's income fully plated: 8 plates at 10 Space Crystals per 48 frames (`factions/gdo/economy.md >
/// ExtractionPlate > MiningRate`).
const GDO_FULL: f64 = 8.0 * 10.0 * 16.0 / 48.0;
/// Design change 0108: "within ±15% of the table".
const TOLERANCE: f64 = 0.15;

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

/// The design's estimates by Drones (module docs), with whether the measured figure is within ±15% of it (the 24
/// Drones' isn't: the question draft of slice M3-K1, pinned until the Designer answers).
const ESTIMATES: [(u32, f64, bool); 3] = [(4, 6.1, true), (16, 23.0, true), (24, 32.0, false)];

/// The design's three estimates on 100 seconds of the dealt-out field (module docs): within ±15% for 4 and 16
/// Drones, outside it for 24, as measured; and full saturation with 28: every patch mined without a break, 32 per
/// second, above the GDO's fully plated 26.67 (target 4).
#[test]
fn the_saturation_curve_against_the_design() {
    for (drones, design, within) in ESTIMATES {
        let measured = income(drones, true, SHORT);
        let off = measured / design - 1.0;
        println!(
            "{drones:2} Drones: {measured:5.2} per second, the design's {design}: {:+.1}%",
            100.0 * off
        );
        assert_eq!(
            off.abs() <= TOLERANCE,
            within,
            "{drones} Drones: {measured:.2} per second against {design}, {:+.1}%; update ESTIMATES and the docs, and \
             tell the Director",
            100.0 * off
        );
    }
    let full = income(28, true, SHORT);
    println!("28 Drones: {full:5.2} per second, the GDO fully plated {GDO_FULL:.2}");
    assert!(
        (31.9..=32.0).contains(&full),
        "28 Drones mine the row without a break: {full:.2}"
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
