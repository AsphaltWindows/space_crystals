//! The Colonists' computer player against itself (slice A2c), played and checked by `super::harness` after every step
//! as the GDO's games are (`super` module docs): the invariants, the gate, design change 0402's Verification.
//!
//! - **The default run** ([`a_colonists_standard_computer_attacks_a_passive_one`]): standard against passive on
//!   `dev_skirmish` for [`DEFAULT_RUN`] of game time, with every invariant. At the end, the standard player's waves
//!   have gone out and fought, and both players keep Beacon Capacity ahead of their armies.
//! - **Determinism** ([`the_same_colonists_game_ends_the_same`]): the same game twice ends with the same state hash.
//!
//! `super::ai_games` plays the Colonists' pairings on every Skirmish Default map.

use sc_ai::Profile::{Passive, Standard};
use sc_sim::{Faction, scenarios};

use super::harness::{Checks, play};
use super::{DEFAULT_RUN, P1, P2, with_factions};

/// `dev_skirmish` with the Colonists in both slots.
fn colonists_vs_colonists() -> sc_sim::Scenario {
    with_factions(scenarios::dev_skirmish(), [Faction::Colonists, Faction::Colonists])
}

#[test]
fn a_colonists_standard_computer_attacks_a_passive_one() {
    let report = play(
        &colonists_vs_colonists(),
        &[(P1, Standard), (P2, Passive)],
        DEFAULT_RUN,
        Checks::All,
    );
    let (standard, passive) = (report.side(P1), report.side(P2));
    assert!(standard.waves >= 1, "the standard player sent no wave: {report:?}");
    assert_eq!(passive.waves, 0, "the passive player never attacks: {report:?}");
    // The Drones and Legionnaires each trained: a field's worth of Drones and two waves' worth of Legionnaires take
    // Beacons all along.
    assert!(
        standard.trained >= 40 && passive.trained >= 40,
        "both kept training, Beacon Capacity ahead of the army: {report:?}"
    );
    assert!(passive.lost >= 1 && standard.lost >= 1, "the waves fought: {report:?}");
}

#[test]
fn the_same_colonists_game_ends_the_same() {
    let scenario = colonists_vs_colonists();
    let game = || play(&scenario, &[(P1, Standard), (P2, Standard)], DEFAULT_RUN, Checks::Rules);
    let (first, second) = (game(), game());
    assert_eq!(first.hash, second.hash);
    assert_eq!(first.sides, second.sides);
    assert!(first.sides.iter().all(|s| s.waves >= 1), "both attacked: {first:?}");
}
