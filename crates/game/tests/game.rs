//! The client's integration tests, in one test program (slice M5-T): every test file is a module of it, so that an
//! edit links Bevy into one program rather than into one for each file. `Cargo.toml` lists it, with
//! `autotests = false`.
//!
//! - **A test file** `tests/foo.rs` is `mod foo;` here, and a folder `tests/foo/` (its root `main.rs`, its modules
//!   beside it) is `#[path = "foo/main.rs"] mod foo;`, in name order. A test file that no program includes fails the
//!   sim's `test_programs` (`crates/sim/tests/test_programs.rs`), which names it.
//! - **One file's tests**: filter by its module, `cargo test -p space_crystals --test game commands::`.
//! - **One process**: the tests of every file run on the threads of one process, as the sim's do
//!   (`crates/sim/tests/sim.rs`).

#[path = "client/main.rs"]
mod client;
#[path = "combat/main.rs"]
mod combat;
#[path = "commands/main.rs"]
mod commands;
#[path = "editor/main.rs"]
mod editor;
mod factions;
mod fog;
#[path = "fog_memory/main.rs"]
mod fog_memory;
#[path = "game_end/main.rs"]
mod game_end;
mod hud;
mod minimap_alerts;
mod pairings;
#[path = "playtest_menu/main.rs"]
mod playtest_menu;
mod replay;
#[path = "selection/main.rs"]
mod selection;
