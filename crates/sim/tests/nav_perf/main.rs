//! Timings of the navigation library on the 256×256 cases of `docs/movement.md > 5.8`. Ignored by default:
//!
//! ```sh
//! cargo test -p sc_sim --release --test nav_perf -- --ignored --nocapture
//! ```
//!
//! The assertions are on the deterministic counters only, never on wall time.
//!
//! One test binary:
//! - `library`: the library alone on the bad case: grid builds, searches and fields, the path budget, a restamp;
//! - `knowledge`: path knowledge in a game (M2-E): the tick that rebuilds the true and the known grids.
//!
//! [`timed`], here, is the one helper that reads the clock.

mod knowledge;
mod library;

use std::time::Duration;

use sc_sim::nav::{
    Nav, NavClass, NavGrid, NavRules, PATH_BUDGET, PathGroup, PathRequest, Search, TileRect, find_path, serve,
};
use sc_sim::{Map, ObjectId, Rng, Tile, TilePos, TilePresetId, Vec2};

const SIDE: u32 = 256;

/// Wall-clock time of `work`. Only this test reads a clock: it measures the library from outside, and nothing
/// in the sim can call it.
#[expect(
    clippy::disallowed_methods,
    reason = "a perf test has to read the clock; the sim itself never does"
)]
fn timed<T>(work: impl FnOnce() -> T) -> (T, Duration) {
    let start = std::time::Instant::now();
    let result = work();
    (result, start.elapsed())
}

fn t(x: i32, y: i32) -> TilePos {
    TilePos::new(x, y)
}
