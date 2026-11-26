//! The sim's timings, the `*_perf` test files, in a test program of their own (slice M5-T), so that a release build
//! for them compiles nothing else. Their tests are ignored, and each runs in release, alone, by name:
//! `cargo test -p sc_sim --release --test perf -- --ignored --nocapture --test-threads=1 agents_perf`. A file is a
//! module here as in `sim.rs`.

mod agents_perf;
mod attack_move_perf;
mod behaviours_perf;
mod drones_perf;
mod garrisons_perf;
mod holding_perf;
mod movement_perf;
#[path = "nav_perf/main.rs"]
mod nav_perf;
mod placement_perf;
mod popup_perf;
mod recruits_perf;
mod repairs_perf;
mod structures_perf;
mod vision_perf;
