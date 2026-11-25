//! (M4-O) Timings of the sim's ticks with 200 grenades in flight (the Director's note on performance): 200 test
//! throwers (`combat::testing`) attack the ground under a line of Peacekeepers, all at once, in volleys; 200 more
//! Peacekeepers stand away from it, so that each blast takes its overlap with 400 objects (`super::super::area`). A
//! unit test, as the throwers need the test defs; ignored by default, and printed, never asserted:
//!
//! ```sh
//! cargo test -p sc_sim --release --lib projectiles_perf -- --ignored --nocapture --test-threads=1
//! ```

use std::time::Duration;

use glam::Vec2;

use crate::combat::testing::{GRENADE, grenadier};
use crate::defs::ObjectKind;
use crate::ids::{ObjectId, PlayerId};
use crate::orders::OrderKind;
use crate::testing::{ScenarioBuilder, TestSim};

const SIDE: usize = 64;
const THROWERS: u32 = 200;
const ROWS: u32 = 40;
const VOLLEYS: u32 = 4;
/// Where the line the throwers aim at stands: in range of every column.
const LINE_X: f32 = 650.0;

/// Wall-clock time of `work`. Only perf tests read a clock: they measure the sim from outside, and nothing in the
/// sim can call it (`ARCHITECTURE.md > Determinism`).
#[expect(
    clippy::disallowed_methods,
    reason = "a perf test has to read the clock; the sim itself never does"
)]
fn timed<T>(work: impl FnOnce() -> T) -> (T, Duration) {
    let start = std::time::Instant::now();
    let result = work();
    (result, start.elapsed())
}

/// Row `row`'s height, space units.
fn row_y(row: u32) -> f32 {
    100.0 + 48.0 * row as f32
}

/// The throwers (ids 1 to 200) in 5 columns of 40 rows, 48 su apart, facing the line; player 2's Peacekeepers, one
/// on the line at each row, and 160 bystanders far to the east. Auto-targeting off, and everyone holds, so that
/// nothing but the volleys happens.
fn battlefield() -> TestSim {
    let map = vec![".".repeat(SIDE); SIDE].join("\n");
    let mut t = ScenarioBuilder::new(&map).start();
    t.check_invariants(false).without_auto_targeting();
    for i in 0..THROWERS {
        let (column, row) = (i / ROWS, i % ROWS);
        let at = Vec2::new(400.0 + 48.0 * column as f32, row_y(row));
        t.spawn_unit_at(1, ObjectKind::Guard, PlayerId(1), at);
    }
    for row in 0..ROWS {
        t.spawn_unit_at(1, ObjectKind::Peacekeeper, PlayerId(2), Vec2::new(LINE_X, row_y(row)));
    }
    for i in 0..THROWERS - ROWS {
        let at = Vec2::new(2400.0 + 40.0 * (i % 10) as f32, row_y(i / 10));
        t.spawn_unit_at(1, ObjectKind::Peacekeeper, PlayerId(2), at);
    }
    t.step();
    assert_eq!(t.sim().objects().count(), 2 * THROWERS as usize);
    t
}

#[test]
#[ignore = "prints timings; run in release with --ignored --nocapture"]
fn projectiles_perf_200_grenades_in_flight() {
    let _grenadier = grenadier(GRENADE);
    let mut t = battlefield();
    let (mut flying, mut flying_ticks, mut worst_flying) = (Duration::ZERO, 0_u32, Duration::ZERO);
    let (mut landing, mut landing_ticks, mut worst_landing) = (Duration::ZERO, 0_u32, Duration::ZERO);
    let mut most = 0;
    for _ in 0..VOLLEYS {
        let tick = t.sim().tick() + 1;
        for i in 0..THROWERS {
            let at = Vec2::new(LINE_X, row_y(i % ROWS));
            t.order_at(
                tick,
                PlayerId(1),
                &[ObjectId(i + 1)],
                OrderKind::AttackGround(at),
                false,
            );
        }
        let mut launched = false;
        loop {
            let before = t.sim().projectiles().len();
            let ((), took) = timed(|| {
                t.step();
            });
            let after = t.sim().projectiles().len();
            most = most.max(after);
            launched |= after > 0;
            if after < before {
                landing += took;
                landing_ticks += 1;
                worst_landing = worst_landing.max(took);
            } else if after > 0 {
                flying += took;
                flying_ticks += 1;
                worst_flying = worst_flying.max(took);
            }
            if launched && after == 0 {
                break;
            }
            assert!(t.sim().tick() < 2000, "the volley never landed");
        }
    }
    let ms = |d: Duration, n: u32| d.as_secs_f64() * 1e3 / f64::from(n.max(1));
    println!(
        "projectiles_perf: {VOLLEYS} volleys of {THROWERS}, at most {most} in flight; in flight: mean {:.3} ms, worst \
         {:.3} ms per whole sim tick ({flying_ticks} ticks); landing: mean {:.3} ms, worst {:.3} ms ({landing_ticks} \
         ticks)",
        ms(flying, flying_ticks),
        ms(worst_flying, 1),
        ms(landing, landing_ticks),
        ms(worst_landing, 1),
    );
}
