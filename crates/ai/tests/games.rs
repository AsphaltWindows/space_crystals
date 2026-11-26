//! AI against AI, headless (slice A1; `ARCHITECTURE.md > Computer players > Tests`): the GDO against the GDO, A1's
//! only faction with a computer player, each game played and checked by `harness` after every step (the invariants,
//! the gate, design change 0402's Verification).
//!
//! - **The default run** ([`a_standard_computer_attacks_a_passive_one`]): standard against passive on `dev_skirmish`
//!   for [`DEFAULT_RUN`] of game time, with every invariant. At the end, the standard player's waves have gone out and
//!   fought.
//! - **0402's Verification** ([`computers_attack_only_buildings_they_know`]): standard against standard, where both
//!   attack, to a defeat; and the check itself, on a planted target ([`an_attack_on_a_building_never_seen_is_caught`]).
//! - **Determinism** ([`the_same_game_ends_the_same`]): the same game twice ends with the same state hash.
//! - **`ai_games`** (ignored, a report, in release): every pairing whose factions both have a computer player, on every
//!   Skirmish Default map, each profile against each that attacks, to a defeat or [`LIMIT`]. In each faction's
//!   mirror, standard must beat passive within [`STANDARD_BEATS_PASSIVE`] from either slot, and standard against
//!   standard must end in a defeat; between two factions the table only reports. It prints a table of who won, when,
//!   and how many units each side trained, and what `Ai::think` cost:
//!   `cargo test -p sc_ai --release --test ai games::ai_games -- --ignored --nocapture`.
//! - **The Colonists** (slice A2c, [`colonists`]): their default run and determinism.

mod colonists;
mod harness;

use std::fs;
use std::path::Path;

use sc_ai::{Ai, Profile, Start, View};
use sc_sim::{
    Category, Command, Faction, GameSetup, MapDef, MapType, OrderKind, PlayerCommand, PlayerId, Scenario,
    TICKS_PER_SECOND, Tick, scenarios,
};

use harness::{Aim, Checks, Cost, Knowledge, Report, play};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

/// The default run's game time: 6 minutes. The standard player's first wave leaves at 2:00 and fights the passive
/// player's army at its staging point half a minute later; the next two, of 32, leave at about 3:40 and 5:00.
const DEFAULT_RUN: Tick = 6 * 60 * TICKS_PER_SECOND;
/// `ai_games`' limit: 20 minutes (the brief's).
const LIMIT: Tick = 20 * 60 * TICKS_PER_SECOND;
/// By when standard must have beaten passive: 15 minutes (the brief's).
const STANDARD_BEATS_PASSIVE: Tick = 15 * 60 * TICKS_PER_SECOND;

use Profile::{Passive, Standard};

/// `scenario` with `factions` in its slots, as `--factions a,b` starts it.
fn with_factions(mut scenario: Scenario, factions: [Faction; 2]) -> Scenario {
    scenario.setup = GameSetup::per_slot(&scenario.map, &factions, scenario.setup.seed);
    scenario
}

/// `scenario` with the GDO in both slots.
fn gdo_vs_gdo(scenario: Scenario) -> Scenario {
    with_factions(scenario, [Faction::Gdo, Faction::Gdo])
}

#[test]
fn a_standard_computer_attacks_a_passive_one() {
    let scenario = gdo_vs_gdo(scenarios::dev_skirmish());
    let report = play(&scenario, &[(P1, Standard), (P2, Passive)], DEFAULT_RUN, Checks::All);
    let (standard, passive) = (report.side(P1), report.side(P2));
    assert!(standard.waves >= 1, "the standard player sent no wave: {report:?}");
    assert_eq!(passive.waves, 0, "the passive player never attacks: {report:?}");
    assert!(
        standard.trained >= 8 && passive.trained >= 8,
        "both trained a wave's worth: {report:?}"
    );
    assert!(passive.lost >= 1 && standard.lost >= 1, "the wave fought: {report:?}");
}

#[test]
fn computers_attack_only_buildings_they_know() {
    let scenario = gdo_vs_gdo(scenarios::dev_skirmish());
    let report = play(&scenario, &[(P1, Standard), (P2, Standard)], LIMIT, Checks::Rules);
    assert!(
        report.winner().is_some(),
        "standard against standard ends in a defeat: {report:?}"
    );
    for side in &report.sides {
        assert!(side.waves >= 1, "both attack: {report:?}");
    }
    let loser = report
        .sides
        .iter()
        .find(|s| Some(s.player) != report.winner())
        .expect("a loser");
    let winner = report.side(report.winner().expect("a winner"));
    assert!(
        winner.at_buildings >= 1 && loser.buildings == 0,
        "the winner's waves aimed at the buildings it saw, and took them all: {report:?}"
    );
}

#[test]
fn an_attack_on_a_building_never_seen_is_caught() {
    let scenario = gdo_vs_gdo(scenarios::dev_skirmish());
    let sim = sc_sim::Sim::new(&scenario);
    let starts = Start::all(&scenario);
    let ai = Ai::new(&scenario, &sim, P1, Standard).expect("player 1's computer");
    let mut known = Knowledge::default();
    known.learn(&View::new(&sim, P1, &starts));
    let theirs: Vec<_> = (sim.objects())
        .filter(|o| o.owner == Some(P2) && o.kind.def().category == Category::Structure)
        .collect();
    let facility = (theirs.iter())
        .find(|o| o.kind == sc_sim::ObjectKind::ExtractionFacility)
        .expect("theirs");
    let order = |kind| PlayerCommand {
        player: P1,
        command: Command::Order {
            units: Vec::new(),
            kind,
            queue: false,
        },
    };
    let aim = |kind| known.aim(&sim, &starts, ai.army().staging(), &order(kind));
    let start = starts
        .iter()
        .find(|s| s.player == P2)
        .expect("player 2's start")
        .point();
    assert_eq!(
        aim(OrderKind::AttackMove(start)),
        Ok(Some(Aim::Start)),
        "a start is known"
    );
    let staging = ai.army().staging().expect("player 1's staging point");
    assert_eq!(
        aim(OrderKind::AttackMove(staging)),
        Ok(Some(Aim::Own)),
        "its staging point is its own"
    );
    assert!(
        aim(OrderKind::AttackMove(facility.pos)).is_err(),
        "player 2's facility, unseen"
    );
    assert!(
        aim(OrderKind::Attack(facility.id)).is_err(),
        "player 2's facility, unseen"
    );
}

#[test]
fn the_same_game_ends_the_same() {
    let scenario = gdo_vs_gdo(scenarios::dev_skirmish());
    let game = || play(&scenario, &[(P1, Standard), (P2, Standard)], DEFAULT_RUN, Checks::Rules);
    let (first, second) = (game(), game());
    assert_eq!(first.hash, second.hash);
    assert_eq!(first.sides, second.sides);
    assert!(first.sides.iter().all(|s| s.waves >= 1), "both attacked: {first:?}");
}

#[test]
#[ignore = "a report, in release: cargo test -p sc_ai --release --test ai games::ai_games -- --ignored --nocapture"]
fn ai_games() {
    let with_ai = sc_ai::factions::with_ai();
    let mut rows = Vec::new();
    let mut think = Cost::default();
    for (flag, scenario) in maps() {
        for &a in &with_ai {
            for &b in &with_ai {
                let scenario = with_factions(scenario.clone(), [a, b]);
                for profiles in [[Standard, Passive], [Passive, Standard], [Standard, Standard]] {
                    let ais = [(P1, profiles[0]), (P2, profiles[1])];
                    let report = play(&scenario, &ais, LIMIT, Checks::All);
                    think.add(&report.think);
                    rows.push(Row {
                        map: flag.clone(),
                        factions: [a, b],
                        profiles,
                        report,
                    });
                }
            }
        }
    }
    println!("{}", table(&rows));
    println!(
        "Ai::think over {} calls: mean {:?}, worst {:?}",
        think.calls,
        think.mean(),
        think.worst
    );
    // Required of each faction against itself; between two factions the table reports the balance, which the user
    // judges (the A2 slices' brief).
    for row in rows.iter().filter(|row| row.factions[0] == row.factions[1]) {
        let standard = row
            .profiles
            .iter()
            .position(|&p| p == Standard)
            .map(|i| PlayerId(i as u8 + 1));
        let ended = row.report.winner().is_some();
        match row.profiles {
            [Standard, Standard] => assert!(ended, "{}: standard against standard ends in a defeat", row.label()),
            _ => assert!(
                row.report.winner() == standard && row.report.ticks <= STANDARD_BEATS_PASSIVE,
                "{}: standard beats passive within 15 minutes",
                row.label()
            ),
        }
    }
}

/// One of `ai_games`' games.
struct Row {
    map: String,
    factions: [Faction; 2],
    profiles: [Profile; 2],
    report: Report,
}

impl Row {
    fn label(&self) -> String {
        let [a, b] = self.factions.map(|f| format!("{f:?}").to_lowercase());
        let [x, y] = self.profiles.map(|p| p.name());
        format!("{} --factions {a},{b} --ai 1:{x},2:{y}", self.map)
    }
}

/// `ai_games`' table: a line for each game.
fn table(rows: &[Row]) -> String {
    let mut lines = vec![format!(
        "{:<44} {:<9} {:<9} {:>6} {:>6} {:>13} {:>9} {:>15}",
        "map, factions", "player 1", "player 2", "winner", "at", "trained", "waves", "think mean/max"
    )];
    for row in rows {
        let r = &row.report;
        let [a, b] = row.factions.map(|f| format!("{f:?}").to_lowercase());
        let winner = r.winner().map_or("none".to_owned(), |p| format!("{}", p.0));
        let at = format!(
            "{}:{:02}",
            r.ticks / TICKS_PER_SECOND / 60,
            r.ticks / TICKS_PER_SECOND % 60
        );
        let (one, two) = (r.side(P1), r.side(P2));
        lines.push(format!(
            "{:<44} {:<9} {:<9} {:>6} {:>6} {:>13} {:>9} {:>15}",
            format!("{} {a},{b}", row.map),
            row.profiles[0].name(),
            row.profiles[1].name(),
            winner,
            at,
            format!("{} / {}", one.trained, two.trained),
            format!("{} / {}", one.waves, two.waves),
            format!("{:.0?} / {:.0?}", r.think.mean(), r.think.worst),
        ));
    }
    lines.join("\n")
}

/// Every Skirmish Default map, as the pairings tests list them (`crates/game/tests/pairings.rs`): the built-in
/// scenarios of that type by `--scenario` name, then `maps/`'s files by name.
fn maps() -> Vec<(String, Scenario)> {
    let built_in =
        (scenarios::NAMES.iter()).filter_map(|&name| Some((format!("--scenario {name}"), scenarios::by_name(name)?)));
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../maps");
    let mut files: Vec<String> = fs::read_dir(&dir)
        .map(|entries| entries.filter_map(|e| e.ok()?.file_name().into_string().ok()).collect())
        .unwrap_or_default();
    files.retain(|name| name.ends_with(".json"));
    files.sort();
    let in_files = files.into_iter().map(|name| {
        let text = fs::read_to_string(dir.join(&name)).expect("a map file");
        let map: MapDef = serde_json::from_str(&text).expect("a map");
        (format!("--map maps/{name}"), Scenario::new(map))
    });
    (built_in.chain(in_files))
        .filter(|(_, s)| s.map.map_type == MapType::SkirmishDefault)
        .collect()
}
