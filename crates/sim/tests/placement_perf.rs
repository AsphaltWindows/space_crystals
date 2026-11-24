//! Timings of `Sim::placement_check` and `Sim::build_area` (slice M2-C), which the placement ghost will call every
//! frame and the command gate on every `Place` command: on a 256 × 256 map with 500 units, 168 crystal patches and
//! 101 buildings of player 1's. Ignored by default:
//!
//! ```sh
//! cargo test -p sc_sim --release --test placement_perf -- --ignored --nocapture
//! ```
//!
//! The assertions are on the results, never on wall time.

use std::time::Duration;

use sc_sim::testing::ScenarioBuilder;
use sc_sim::{BuildingOrientation, ObjectKind, PlacementCheck, PlayerId, SpawnAt, TilePos};

const SIDE: usize = 256;
const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

/// Wall-clock time of `work`. Only this test reads a clock: it measures the sim from outside, and nothing in the
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

#[test]
#[ignore = "perf test: run with --release -- --ignored --nocapture"]
fn placement_check_on_a_crowded_map() {
    let tile = |x: i32, y: i32| SpawnAt::Tile(TilePos::new(x, y));
    let mut builder = ScenarioBuilder::new(&vec![".".repeat(SIDE); SIDE].join("\n")).spawn(
        ObjectKind::DeploymentCenter,
        Some(P1),
        tile(8, 8),
    );
    for y in (40..80).step_by(8) {
        for x in (40..200).step_by(8) {
            builder = builder.spawn(ObjectKind::Generator, Some(P1), tile(x, y));
        }
    }
    for y in [100, 110] {
        for x in (44..212).step_by(2) {
            builder = builder.spawn(ObjectKind::CrystalPatch, None, tile(x, y));
        }
    }
    for (player, rows) in [(P1, 150..160), (P2, 170..180)] {
        for y in rows {
            for x in 120..145 {
                let pos = TilePos::new(x, y).center();
                builder = builder.spawn(ObjectKind::Peacekeeper, Some(player), SpawnAt::Point(pos));
            }
        }
    }
    let mut t = builder.start();
    t.without_auto_targeting().run(2);
    let sim = t.sim();
    assert_eq!(sim.objects().count(), 1 + 100 + 168 + 500);

    let kinds = [
        ObjectKind::Generator,
        ObjectKind::Barracks,
        ObjectKind::ExtractionFacility,
        ObjectKind::ExtractionPlate,
    ];
    let sweep = || -> Vec<PlacementCheck> {
        let mut checks = Vec::new();
        for kind in kinds {
            for y in (0..SIDE as i32).step_by(4) {
                for x in (0..SIDE as i32).step_by(4) {
                    let origin = TilePos::new(x, y);
                    checks.push(sim.placement_check(P1, kind, origin, BuildingOrientation::default()));
                }
            }
        }
        checks
    };
    let (checks, took) = timed(sweep);
    let calls = checks.len() as u32;
    assert_eq!(sweep(), checks, "the same answers every time");
    assert!(checks.iter().any(PlacementCheck::is_valid) && !checks.iter().all(PlacementCheck::is_valid));
    println!(
        "placement_check: {calls} calls in {took:?}, {:?} per call, {} objects",
        took / calls,
        sim.objects().count()
    );

    let (areas, took) = timed(|| (0..1000).map(|_| sim.build_area(P1)).collect::<Vec<_>>());
    assert!(areas.windows(2).all(|w| w[0] == w[1]));
    let (tiles, runs_took) = timed(|| areas[0].tiles().count());
    println!(
        "build_area: {:?} per call ({} parts, {tiles} tiles; listing its tiles took {runs_took:?})",
        took / 1000,
        areas[0].parts().len()
    );
}
