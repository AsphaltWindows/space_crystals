//! The economy oracle: openings played from a real start, the way a player would, and checked against the design's
//! numbers. One binary, in modules:
//! - **The GDO** (slice M2-J1; `played.rs`, `gdo.rs`): `factions/gdo/economy.md > Opening Economy Baseline`, with
//!   the reference line's income at the times of `factions/gdo/economy.md > Economy Targets (GDO vs Colonists)`'s
//!   [`TABLE`].
//! - **The Colonists** (slices M3-K1 and M3-K4; `colonists.rs`, `curve.rs`): [`TABLE`], both its columns, and the
//!   targets 1 to 4 of `factions/gdo/economy.md > Economy Targets (GDO vs Colonists)`, on the reference field, against
//!   the GDO's line, and the saturation curve of `factions/colonists/units/drone.md > Patch Occupancy`.
//! - **The Syndicate** (slice M4-L; `syndicate.rs`): the Syndicate economy check of `factions/syndicate/units/agent.md >
//!   Gathering > Space Crystals` (design changes 0259 and 0294), its table, ceiling, opening and stacked Tunnels, on the
//!   reference layouts of `sc_sim::testing::syndicate`.
//! - **The Cults** (slice M5-L; `cults.rs`): the Cults economy check of `factions/cults/economy.md > Economy check`
//!   (design change 0346), its ideal and realistic scenarios, on the reference layouts of `sc_sim::testing::cults`.
//!
//! **The GDO's lines** check design changes 0098 (plates pay lumps of 10; the cycle timer advances by the power ratio)
//! and 0089 (the power penalty; the Deployment Center is never slowed). Each line is player 1's on
//! `maps/dev_skirmish.json` from its Starting Conditions (a Deployment Center, an Extraction Facility, two plates and
//! 100 Space Crystals, all 8 patches of the field in range: the reference base), played by a scripted build order
//! (`sc_sim::testing::build_order`): the player's own commands through the gate, each order as soon as the bank covers
//! it and its builder is free, each building placed at once at the first spot the placement check takes, plates onto
//! patches. Player 2 does nothing. Player 1's books balance every tick (`Books`), every lump is followed to its patch,
//! and each line is played only to the last time it checks.
//!
//! The design computed its numbers with a trickle of income; the sim pays plates in lumps of 10 at the end of each
//! 48-frame cycle (0098) and counts whole frames. The tolerances follow from that, each named where it is used:
//! - **Income** at a time is the average over the 9 seconds (3 cycles, `played::WINDOW`) ending then: a plate that
//!   mined through the window counts exactly 10/3 per second, as in the design's plates landing every 25 seconds
//!   (`gdo::model`). A plate that lands within the 12 seconds before counts less, as its first lump comes 3 seconds
//!   after it lands. (The GDO column of [`TABLE`] measures the line otherwise: `colonists.rs`.)
//! - **A wait for money** ends up to one cycle (3 s) later than with a trickle: the time the lumps take to cover it.
//! - **A bank** lags a trickle's by less than one lump (10) per plate.
//! - **Costs, build times, lumps and the power penalty** are checked exactly against the design's numbers, to the
//!   whole frame, so that a wrong cost, rate, build time, penalty or refund fails here whatever the tolerances above.
//!
//! **The Colonists' line** and how it is measured: `colonists.rs`.
//!
//! `cargo test -p sc_sim --test sim economy_oracle -- --nocapture` prints each line's timeline (when each order
//! started, finished and was placed, and the bank, income and power at the line's checkpoints) and the Colonists'
//! table. The measured results, both factions', are in `docs/economy_oracle.md`: a change that moves one updates it.

mod colonists;
mod cults;
mod curve;
mod gdo;
mod played;
mod syndicate;

use sc_sim::{PlayerId, TICKS_PER_SECOND, Tick};

const P1: PlayerId = PlayerId(1);

/// `factions/gdo/economy.md > Economy Targets (GDO vs Colonists)`'s table: the time, the Colonists' income and the
/// GDO's. Since design change 0236 (answering Q0055 and Q0072: "the measured economy is the design") it is "the
/// simulated measurement [...], not an estimate": the Colonists' column the reference line of `colonists.rs` as
/// slice M3-K3 measured it, rounded, and the GDO's M2's reference line (`played::plates_first`) measured the same
/// way. `colonists.rs` has the table as measured, and the table before 0236.
const TABLE: [(Tick, f64, f64); 9] = [
    (at(0, 0), 6.9, 6.7),
    (at(0, 40), 11.4, 10.0),
    (at(1, 0), 13.3, 12.9),
    (at(1, 20), 16.0, 15.2),
    (at(1, 40), 18.7, 17.6),
    (at(2, 20), 22.9, 23.3),
    (at(2, 40), 25.1, 26.2),
    (at(3, 0), 26.3, 26.7),
    (at(4, 0), 27.8, 26.7),
];
/// Design change 0108 (`factions/gdo/economy.md > Economy Targets (GDO vs Colonists) > Matching the table in a
/// simulated check`): "each income figure must be within ±15% of the table. A larger gap is brought back to the
/// designer with the measured table; game numbers are not adjusted to force a match."
const TOLERANCE: f64 = 0.15;

/// The tick at `m` minutes and `s` seconds of the game.
const fn at(m: u32, s: u32) -> Tick {
    (m * 60 + s) * TICKS_PER_SECOND
}

fn seconds(tick: Tick) -> f64 {
    f64::from(tick) / f64::from(TICKS_PER_SECOND)
}

/// "m:ss.ss".
fn clock(tick: Tick) -> String {
    let s = seconds(tick);
    format!("{}:{:05.2}", (s / 60.0).floor(), s % 60.0)
}
