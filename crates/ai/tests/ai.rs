//! The computer players' integration tests, in one test program, as the sim's (`crates/sim/tests/sim.rs`) and the
//! client's (`crates/game/tests/game.rs`): every test file is a module of it. `Cargo.toml` lists it, with
//! `autotests = false`.
//!
//! - **A test file** `tests/foo.rs` is `mod foo;` here, and a folder `tests/foo/` (its root `main.rs`, its modules
//!   beside it) is `#[path = "foo/main.rs"] mod foo;`, in name order. A test file that no program includes fails the
//!   sim's `test_programs` (`crates/sim/tests/test_programs.rs`), which names it.
//! - **One file's tests**: filter by its module, `cargo test -p sc_ai --test ai seam::`.
//! - **One process**: the tests of every file run on the threads of one process, as the sim's do.

mod games;
mod seam;
