//! Timings of a whole sim tick with 500 moving units on a 256 × 256 map, against the targets of
//! `docs/movement.md > 5.8`. Ignored by default:
//!
//! ```sh
//! cargo test -p sc_sim --release --test perf movement_perf -- --ignored --nocapture
//! ```
//!
//! The assertions are on the deterministic counters only, never on wall time; `movement_collision/` asserts the
//! same counters in the normal test run.

use std::time::Duration;

use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{MoveStatus, ObjectId, ObjectKind, OrderKind, PlayerId, Rng, Target, Vec2};

const SIDE: usize = 256;
const UNITS: u32 = 500;

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

/// 256 × 256 with 5 % scattered cliffs; the two armies' fields are clear.
fn map(rng: &mut Rng) -> String {
    let mut rows = Vec::with_capacity(SIDE);
    for y in 0..SIDE {
        let row: String = (0..SIDE)
            .map(|x| {
                let field = (8..40).contains(&y) && ((8..40).contains(&x) || (216..248).contains(&x));
                if !field && rng.below(100) < 5 { 'c' } else { '.' }
            })
            .collect();
        rows.push(row);
    }
    rows.join("\n")
}

/// 250 units of each player in two blocks, 13 000 su apart, each ordered to the other's field in one group
/// order, so they also meet head-on half way.
fn armies() -> TestSim {
    let mut rng = Rng::from_seed(11);
    let mut t = ScenarioBuilder::new(&map(&mut rng)).start();
    t.check_invariants(false);
    for i in 0..UNITS / 2 {
        let offset = Vec2::new((i % 25) as f32, (i / 25) as f32) * 48.0;
        let kind = if i % 5 == 0 {
            ObjectKind::Guard
        } else {
            ObjectKind::Peacekeeper
        };
        t.spawn_unit_at(1, kind, PlayerId(1), Vec2::new(700.0, 700.0) + offset);
        t.spawn_unit_at(1, kind, PlayerId(2), Vec2::new(14_000.0, 700.0) + offset);
    }
    let (west, east): (Vec<ObjectId>, Vec<ObjectId>) = (1..=UNITS).map(ObjectId).partition(|id| id.0 % 2 == 1);
    let to = |x: f32| OrderKind::Move(Target::Point(Vec2::new(x, 1300.0)));
    t.order_at(2, PlayerId(1), &west, to(14_600.0), false)
        .order_at(2, PlayerId(2), &east, to(1300.0), false);
    t
}

#[test]
#[ignore = "prints timings; run in release with --ignored --nocapture"]
fn perf_500_units_on_256() {
    let mut t = armies();
    t.step();
    assert_eq!(t.sim().objects().count(), UNITS as usize);

    // The two group orders: one field each (`docs/movement.md > 5.5`). (G) A field reaches only as far as its
    // group stands, so both may fit into the budget of one tick; a field that is more than the budget leaves the
    // second group to the next tick ("budget plus one field" is the worst tick of 5.8).
    let mut fields = 0;
    while fields < 2 {
        let ((), took) = timed(|| {
            t.step();
        });
        let nav = t.sim().nav_counters();
        assert!(nav.fields >= 1 && nav.searches == 0, "{nav:?}");
        assert!(nav.expansions <= nav.fields * (SIDE * SIDE) as u32);
        fields += nav.fields;
        println!(
            "order tick ({} field(s), {} expansions, {} paths): {:.2} ms",
            nav.fields,
            nav.expansions,
            nav.fields * UNITS / 2,
            took.as_secs_f64() * 1e3
        );
    }

    let (mut total, mut worst, mut ticks) = (Duration::ZERO, Duration::ZERO, 0_u32);
    let (mut sweeps, mut pairs, mut steered, mut searches) = (0_u64, 0_u64, 0_u64, 0_u32);
    let mut worst_expansions = 0;
    while ticks < 4000 {
        let ((), took) = timed(|| {
            t.step();
        });
        let moves = t.sim().move_counters();
        // `movement_collision/` asserts the same bounds in the normal test run.
        assert!(moves.unit_sweeps <= 8 * moves.steered && moves.tile_sweeps <= 8 * moves.steered);
        assert!(moves.pair_tests <= 24 * moves.unit_sweeps);
        if moves.steered >= UNITS * 9 / 10 {
            total += took;
            worst = worst.max(took);
            ticks += 1;
            sweeps += u64::from(moves.unit_sweeps + moves.tile_sweeps);
            pairs += u64::from(moves.pair_tests);
            steered += u64::from(moves.steered);
        } else if t.sim().tick() > 100 {
            break;
        }
        let nav = t.sim().nav_counters();
        searches += nav.searches;
        worst_expansions = worst_expansions.max(nav.expansions);
    }
    assert!(ticks > 300, "only {ticks} ticks with 450 or more units moving");
    let per = |n: u64| n as f64 / steered as f64;
    println!(
        "{ticks} ticks with >= 450 of 500 units moving: mean {:.3} ms, worst {:.3} ms per whole sim tick",
        total.as_secs_f64() * 1e3 / f64::from(ticks),
        worst.as_secs_f64() * 1e3
    );
    println!(
        "per moving unit and tick: {:.2} sweeps, {:.2} box tests; {searches} repath searches, worst tick {worst_expansions} expansions",
        per(sweeps),
        per(pairs)
    );
    println!("targets (docs/movement.md > 5.8, release): movement <= 1 ms, whole tick <= 10 ms worst");
    let moving = t.sim().objects().filter_map(|o| o.mover.as_ref());
    println!(
        "at tick {}: {} units still moving",
        t.sim().tick(),
        moving.filter(|m| m.status == MoveStatus::Moving).count()
    );
}
