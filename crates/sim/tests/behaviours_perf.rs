//! Timings of a whole sim tick with 500 units under the behaviours, on a 256 × 256 map, against the targets of
//! `docs/movement.md > 5.8` (whole tick ≤ 10 ms worst). Ignored by default:
//!
//! ```sh
//! cargo test -p sc_sim --release --test perf behaviours_perf -- --ignored --nocapture
//! ```
//!
//! Three loads: 500 idle units with nobody in sight (every one scans every tick); two armies of 250 on Hold
//! Position facing each other out of range (every scan finds candidates and rejects them); and the two armies
//! attack-moving into each other until one is destroyed. Timings are printed, never asserted.

use std::time::Duration;

use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{ObjectId, ObjectKind, OrderKind, PlayerId, SimEvent, Vec2};

const SIDE: usize = 256;
const UNITS: u32 = 500;

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

/// 250 units of each player in two blocks of 25 columns, `apart` su between the blocks' near edges.
fn armies(apart: f32) -> (TestSim, Vec<ObjectId>, Vec<ObjectId>) {
    let map = vec![".".repeat(SIDE); SIDE].join("\n");
    let mut t = ScenarioBuilder::new(&map).start();
    t.check_invariants(false);
    let width = 24.0 * 48.0;
    for i in 0..UNITS / 2 {
        let offset = Vec2::new((i % 25) as f32, (i / 25) as f32) * 48.0;
        let kind = if i % 5 == 0 {
            ObjectKind::Guard
        } else {
            ObjectKind::Peacekeeper
        };
        t.spawn_unit_at(1, kind, PlayerId(1), Vec2::new(2000.0, 2000.0) + offset);
        t.spawn_unit_at(1, kind, PlayerId(2), Vec2::new(2000.0 + width + apart, 2000.0) + offset);
    }
    t.step();
    assert_eq!(t.sim().objects().count(), UNITS as usize);
    let (west, east) = (1..=UNITS).map(ObjectId).partition(|id| id.0 % 2 == 1);
    (t, west, east)
}

/// Runs `ticks` ticks (or until `done`) and prints the mean and the worst tick.
fn measure(label: &str, t: &mut TestSim, ticks: u32, done: impl Fn(&TestSim) -> bool) {
    let (mut total, mut worst, mut ran) = (Duration::ZERO, Duration::ZERO, 0_u32);
    let (mut searches, mut worst_expansions) = (0_u64, 0);
    while ran < ticks && !done(t) {
        let ((), took) = timed(|| {
            t.step();
        });
        total += took;
        worst = worst.max(took);
        ran += 1;
        let nav = t.sim().nav_counters();
        searches += u64::from(nav.searches);
        worst_expansions = worst_expansions.max(nav.expansions);
    }
    println!(
        "{label}: {ran} ticks, mean {:.3} ms, worst {:.3} ms per whole sim tick; {searches} path searches, worst tick {worst_expansions} expansions",
        total.as_secs_f64() * 1e3 / f64::from(ran.max(1)),
        worst.as_secs_f64() * 1e3
    );
}

#[test]
#[ignore = "prints timings; run in release with --ignored --nocapture"]
fn perf_500_units_under_the_behaviours() {
    // Idle, nobody in sight: 20 000 su apart.
    let (mut t, ..) = armies(8000.0);
    measure("500 idle units, no enemy in sight", &mut t, 300, |_| false);

    // On Hold, in sight of each other, out of range: the gap between the blocks is 5 grid units.
    let (mut t, west, east) = armies(300.0);
    t.order_at(2, PlayerId(1), &west, OrderKind::Hold, false)
        .order_at(2, PlayerId(2), &east, OrderKind::Hold, false);
    measure(
        "500 units on Hold, enemies in sight and out of range",
        &mut t,
        300,
        |_| false,
    );
    let shots = t
        .events()
        .iter()
        .filter(|(_, e)| matches!(e, SimEvent::Attacked { .. }))
        .count();
    assert_eq!(shots, 0, "out of range");

    // The battle: both attack-move onto the other's place.
    let (mut t, west, east) = armies(3000.0);
    let centre = |x: f32| Vec2::new(x, 2000.0 + 4.5 * 48.0);
    t.order_at(
        2,
        PlayerId(1),
        &west,
        OrderKind::AttackMove(centre(2000.0 + 1152.0 + 3000.0 + 576.0)),
        false,
    )
    .order_at(2, PlayerId(2), &east, OrderKind::AttackMove(centre(2576.0)), false);
    measure("the two group orders", &mut t, 2, |_| false);
    measure("500 units attack-moving, up to the first shot", &mut t, 2000, |t| {
        t.sim().events().iter().any(|e| matches!(e, SimEvent::Attacked { .. }))
    });
    let one_side_left = |t: &TestSim| {
        let mut owners = t.sim().objects().filter_map(|o| o.owner);
        let first = owners.next();
        owners.all(|owner| Some(owner) == first)
    };
    measure("the battle, until one side is destroyed", &mut t, 6000, one_side_left);
    let alive = t.sim().objects().count();
    let deaths = t
        .events()
        .iter()
        .filter(|(_, e)| matches!(e, SimEvent::Died { .. }))
        .count();
    println!("at tick {}: {alive} units left, {deaths} destroyed", t.sim().tick());
    measure("the winners going on to their destination", &mut t, 600, |t| {
        t.sim()
            .objects()
            .all(|o| o.orders.as_ref().is_none_or(|orders| orders.is_idle()))
    });
}
