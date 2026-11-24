//! Timings of the vision phase (5f) and of the records at the end of the tick, with 500 moving units on a 256 × 256
//! map strewn with crystal patches, against the target of slice V: at most about 1 ms per tick. Ignored by default:
//!
//! ```sh
//! cargo test -p sc_sim --release --test vision_perf -- --ignored --nocapture
//! ```
//!
//! Each tick the test copies the vision the sim had before the step, steps the sim, and times the copy catching up
//! with the step's objects: exactly the work of phase 5f and of the records (`Vision::remember`). The assertions
//! are on the result, never on wall time.
//! It prints the 99th percentile besides the worst tick, which a busy machine's preemption can inflate.

use std::time::Duration;

use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{ObjectId, ObjectKind, OrderKind, PlayerId, Rng, SpawnAt, Target, TilePos, Vec2, Vision};

const SIDE: usize = 256;
const UNITS: u32 = 500;
const PATCHES: u32 = 168;

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

/// The tiles of the crystal patches: every other tile of two rows along the edges of the armies' way, 168 in all.
fn patch_tile(x: usize, y: usize) -> bool {
    (44..212).contains(&x) && x.is_multiple_of(2) && (y == 8 || y == 22)
}

/// 256 × 256 with 5 % scattered cliffs; the two armies' fields and the patches' tiles are clear.
fn map(rng: &mut Rng) -> String {
    let mut rows = Vec::with_capacity(SIDE);
    for y in 0..SIDE {
        let row: String = (0..SIDE)
            .map(|x| {
                let field = (8..40).contains(&y) && ((8..40).contains(&x) || (216..248).contains(&x));
                let cliff = !field && rng.below(100) < 5;
                if cliff && !patch_tile(x, y) { 'c' } else { '.' }
            })
            .collect();
        rows.push(row);
    }
    rows.join("\n")
}

/// 250 units of each player in two blocks, 13 000 su apart, each ordered to the other's field in one group order
/// (the armies of `movement_perf.rs`), through the patches.
fn armies() -> TestSim {
    let mut rng = Rng::from_seed(11);
    let mut builder = ScenarioBuilder::new(&map(&mut rng));
    for y in 0..SIDE {
        for x in (0..SIDE).filter(|&x| patch_tile(x, y)) {
            let at = SpawnAt::Tile(TilePos::new(x as i32, y as i32));
            builder = builder.spawn(ObjectKind::CrystalPatch, None, at);
        }
    }
    let mut t = builder.start();
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
    let first = PATCHES + 1;
    let (west, east): (Vec<ObjectId>, Vec<ObjectId>) = (first..first + UNITS)
        .map(ObjectId)
        .partition(|id| (id.0 - first).is_multiple_of(2));
    let to = |x: f32| OrderKind::Move(Target::Point(Vec2::new(x, 1300.0)));
    t.order_at(2, PlayerId(1), &west, to(14_600.0), false)
        .order_at(2, PlayerId(2), &east, to(1300.0), false);
    t
}

#[test]
#[ignore = "prints timings; run in release with --ignored --nocapture"]
fn perf_vision_500_units_on_256() {
    let mut t = armies();
    t.run(3);
    assert_eq!(t.sim().objects().count(), (UNITS + PATCHES) as usize);
    let players = || [PlayerId(1), PlayerId(2)];

    let (fresh, took) = timed(|| {
        let mut fresh = Vision::new(t.sim().map(), players());
        fresh.update(t.sim().map(), t.sim().objects());
        fresh
    });
    println!("a fresh build of 500 sources: {:.3} ms", took.as_secs_f64() * 1e3);
    assert_eq!(
        fresh.player(PlayerId(1)).unwrap().states().collect::<Vec<_>>(),
        t.sim()
            .vision()
            .player(PlayerId(1))
            .unwrap()
            .states()
            .collect::<Vec<_>>()
    );

    let mut times: Vec<Duration> = Vec::new();
    while times.len() < 2000 {
        let mut vision = t.sim().vision().clone();
        t.step();
        let moving = t.sim().objects().filter(|o| o.pos != o.prev_pos).count();
        let ((), took) = timed(|| {
            vision.update(t.sim().map(), t.sim().objects());
            vision.remember(t.sim().tick(), |id| t.sim().object(id));
        });
        assert!(&vision == t.sim().vision());
        if moving >= UNITS as usize * 9 / 10 {
            times.push(took);
        } else if t.sim().tick() > 100 {
            break;
        }
    }
    let ticks = times.len();
    assert!(ticks > 300, "only {ticks} ticks with 450 or more units moving");
    times.sort();
    let ms = |d: Duration| d.as_secs_f64() * 1e3;
    let mean = times.iter().sum::<Duration>().as_secs_f64() * 1e3 / ticks as f64;
    println!(
        "{ticks} ticks with >= 450 of 500 units moving: vision update and records mean {mean:.3} ms, 99th \
         percentile {:.3} ms, worst {:.3} ms",
        ms(times[ticks * 99 / 100]),
        ms(times[ticks - 1])
    );
    let records = |p: u8| t.sim().vision().player(PlayerId(p)).unwrap().records().count();
    println!(
        "records at the end: {} and {} of {PATCHES} patches",
        records(1),
        records(2)
    );
    println!("target (slice V, release): <= 1 ms per tick");
}
