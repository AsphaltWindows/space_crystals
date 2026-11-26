//! (M5-U1) Timings of the sim's ticks with turret units fighting on the move: 48 test turret units of each player
//! (`super::testing`, a 360° turret at 12° a tick) drive past each other in lanes 48 su apart, each on a Move across
//! the field, so that every turret scans each tick (`orders::turret`) and most of them turn and fire as they pass
//! (`combat::turret`). A unit test, as the turret units need the test defs; ignored by default, and printed, never
//! asserted:
//!
//! ```sh
//! cargo test -p sc_sim --release --lib turrets_perf -- --ignored --nocapture --test-threads=1
//! ```

use std::time::Duration;

use glam::Vec2;

use super::testing::{GUN, TURRET_UNIT, open, turret, turret_unit};
use crate::events::SimEvent;
use crate::ids::{ObjectId, PlayerId};
use crate::orders::{OrderKind, Target};
use crate::testing::{ScenarioBuilder, TestSim};

const WIDE: usize = 64;
const HIGH: usize = 32;
/// Units a side, in [`LANES`] lanes.
const UNITS: u32 = 48;
const LANES: u32 = 4;
/// How far the drive goes, ticks: across the field and well past each other.
const TICKS: u32 = 500;

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

/// Player 1's units (ids 1 to 48) in 4 lanes on the west side, driving east; player 2's (49 to 96) in 4 lanes just
/// south of them on the east side, driving west. Each one's Move is to the far side of the field, in its lane.
fn convoys() -> TestSim {
    let mut t = ScenarioBuilder::new(&open(WIDE, HIGH)).start();
    t.check_invariants(false);
    let per_lane = UNITS / LANES;
    let lane_y = |lane: u32| 700.0 + 48.0 * lane as f32;
    let mut orders = Vec::new();
    for (owner, west) in [(PlayerId(1), true), (PlayerId(2), false)] {
        for i in 0..UNITS {
            let (lane, place) = (i / per_lane + if west { 0 } else { LANES }, i % per_lane);
            let x = 100.0 + 56.0 * place as f32;
            let (from, to) = if west { (x, 3800.0) } else { (4000.0 - x, 300.0) };
            let y = lane_y(lane);
            t.spawn_unit_at(1, TURRET_UNIT, owner, Vec2::new(from, y));
            orders.push((owner, Vec2::new(to, y)));
        }
    }
    for (index, (owner, to)) in orders.into_iter().enumerate() {
        let id = ObjectId(index as u32 + 1);
        t.order_at(2, owner, &[id], OrderKind::Move(Target::Point(to)), false);
    }
    t.run_until(2);
    assert_eq!(t.sim().objects().count(), 2 * UNITS as usize);
    t
}

#[test]
#[ignore = "prints timings; run in release with --ignored --nocapture"]
fn turrets_perf_96_turret_units_fighting_on_the_move() {
    let _turret = turret_unit(turret(360.0, 12.0), GUN);
    let mut t = convoys();
    let mut samples = Vec::with_capacity(TICKS as usize);
    for _ in 0..TICKS {
        let ((), took) = timed(|| {
            t.step();
        });
        samples.push(took);
    }
    let shots = (t.events().iter())
        .filter(|(_, e)| matches!(e, SimEvent::Attacked { .. }))
        .count();
    let total: Duration = samples.iter().sum();
    samples.sort_unstable();
    let ms = |d: Duration| d.as_secs_f64() * 1e3;
    println!(
        "turrets_perf: {} turret units, {TICKS} ticks, {shots} shots; mean {:.3} ms, p99 {:.3} ms, worst {:.3} ms \
         per whole sim tick",
        2 * UNITS,
        ms(total) / f64::from(TICKS),
        ms(samples[samples.len() * 99 / 100]),
        ms(samples[samples.len() - 1]),
    );
}
