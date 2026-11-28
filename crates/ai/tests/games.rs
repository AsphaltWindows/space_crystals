//! AI against AI, headless (slices A1 and A2; `docs/architecture/computer-players.md > Computer players > Tests`): each faction with a
//! computer player against itself and the others, each game played and checked by `harness` after every step (the
//! invariants, the gate, design change 0402's Verification).
//!
//! - **The default run** ([`a_standard_computer_attacks_a_passive_one`], and each faction's own, as
//!   [`a_standard_cults_computer_attacks_a_passive_one`] and [`a_standard_syndicate_computer_attacks_a_passive_one`]):
//!   standard against passive of one faction on `dev_skirmish` for [`DEFAULT_RUN`] of game time, with every
//!   invariant. At the end, the standard player's waves have gone out and fought.
//! - **The rush's default run** (slice A2d, [`the_rush_attacks_before_standard_would`], for each faction): its rush
//!   against its passive, the same way; its first wave leaves before standard's would, and fights.
//! - **The Syndicate's growth** (slice A2d, [`a_standard_syndicate_computer_staffs_a_second_crystal_hatch`]): on
//!   `maps/playtest.json`, two crystal hatches staffed by 5:00.
//! - **0402's Verification** ([`computers_attack_only_buildings_they_know`]): standard against standard, where both
//!   attack, to a defeat; and the check itself, on a planted target ([`an_attack_on_a_building_never_seen_is_caught`]).
//! - **Determinism** ([`the_same_game_ends_the_same`], and each faction's own, as
//!   [`the_same_cults_game_ends_the_same`] and [`the_same_syndicate_game_ends_the_same`], and the rush's,
//!   [`the_same_rush_game_ends_the_same`]): the same game twice ends with the same state hash.
//! - **`ai_games`** (ignored, a report, in release): every pairing whose factions both have a computer player, on every
//!   Skirmish Default map, standard against passive from either slot and against standard, and in a faction's mirror
//!   its rush against passive and against standard from either slot ([`pairings`]), to a defeat or [`LIMIT`]: standard
//!   must beat its own faction's passive within [`STANDARD_BEATS_PASSIVE`] from either slot, standard against standard
//!   must end in a defeat, and the rush's first wave must leave before standard's in its place against the same
//!   passive. The Colonists' standard banks less than 500 Space Crystals on average from 3:00 to 10:00 in their mirror,
//!   and the Cults' less than 1,000 against the GDO's passive (the A2d brief). Standard beating another faction's
//!   passive within [`LIMIT`] is wanted, not required (the A2 brief's "Wanted"): the report lists whether it did. It
//!   prints a table of who won, when, how many units each side trained, when its first wave left, its mean bank and
//!   income from 3:00 to 10:00, and what `Ai::think` cost: `cargo test -p sc_ai --release --test ai games::ai_games --
//!   --ignored --nocapture`.
//! - **The Colonists** (slice A2c, [`colonists`]): their default run and determinism.
//! - **The map explored from the start** (slice M5-S7, design change 0416, `core/vision.md > Starting exploration`):
//!   the client starts its games so by default, while these games keep the sim's default, unexplored, and `ai_games`
//!   pins it off for its rows, so that they compare with the earlier ones. One game is played explored
//!   ([`a_standard_computer_beats_a_passive_one_on_a_map_explored_from_the_start`]): the GDO's standard must beat its
//!   passive.

mod colonists;
mod harness;

use std::fs;
use std::path::Path;

use sc_ai::tuning::syndicate::AGENTS_PER_HATCH;
use sc_ai::{Ai, Profile, Start, View};
use sc_sim::economy::drop_off::{Resource, hatch_square};
use sc_sim::{
    Category, Command, Faction, GameSetup, MapDef, MapType, Object, ObjectId, ObjectKind, OrderKind, PlayerCommand,
    PlayerId, Scenario, Sim, TICKS_PER_SECOND, Tick, Vec2, scenarios,
};

use harness::{Aim, Checks, Cost, Knowledge, Report, play, watch};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

/// The default run's game time: 6 minutes. The standard player's first wave leaves at 2:00 and fights the passive
/// player's army at its staging point half a minute later; the next two, of 32, leave at about 3:40 and 5:00.
const DEFAULT_RUN: Tick = 6 * 60 * TICKS_PER_SECOND;
/// `ai_games`' limit: 20 minutes (the brief's).
const LIMIT: Tick = 20 * 60 * TICKS_PER_SECOND;
/// By when standard must have beaten passive: 15 minutes (the brief's).
const STANDARD_BEATS_PASSIVE: Tick = 15 * 60 * TICKS_PER_SECOND;

use Profile::{Passive, Rush, Standard};

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

/// The Cults' default run (slice A2a): as the GDO's. The standard player's first wave, of 8 Militants, leaves at about
/// 3:00, and its first units fall at about 4:20.
#[test]
fn a_standard_cults_computer_attacks_a_passive_one() {
    let scenario = with_factions(scenarios::dev_skirmish(), [Faction::Cults, Faction::Cults]);
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

/// The Syndicate's default run (slice A2b): its first wave, of 20 Guards (`sc_ai::tuning::syndicate::WAVE_SIZE`),
/// leaves at about 4:55 and fights the passive player's army at its staging point half a minute later.
#[test]
fn a_standard_syndicate_computer_attacks_a_passive_one() {
    let scenario = with_factions(scenarios::dev_skirmish(), [Faction::Syndicate; 2]);
    let report = play(&scenario, &[(P1, Standard), (P2, Passive)], DEFAULT_RUN, Checks::All);
    let (standard, passive) = (report.side(P1), report.side(P2));
    assert!(standard.waves >= 1, "the standard player sent no wave: {report:?}");
    assert_eq!(passive.waves, 0, "the passive player never attacks: {report:?}");
    assert!(passive.lost >= 1 && standard.lost >= 1, "the wave fought: {report:?}");
}

/// The rush's default run (slice A2d), for `faction`: its rush against its passive on `dev_skirmish` for
/// [`DEFAULT_RUN`] of game time, with every invariant. The rush's first wave leaves before standard's would in its place
/// (standard's game, played to that tick, has sent none yet), and fights.
fn the_rush_attacks_before_standard_would(faction: Faction) {
    let scenario = with_factions(scenarios::dev_skirmish(), [faction; 2]);
    let report = play(&scenario, &[(P1, Rush), (P2, Passive)], DEFAULT_RUN, Checks::All);
    let (rush, passive) = (report.side(P1), report.side(P2));
    let first = rush.first_wave.expect("the rush sent a wave");
    assert_eq!(passive.waves, 0, "the passive player never attacks: {report:?}");
    assert!(passive.lost >= 1 && rush.lost >= 1, "the wave fought: {report:?}");
    let standard = play(&scenario, &[(P1, Standard), (P2, Passive)], first, Checks::Rules);
    assert_eq!(
        standard.side(P1).first_wave,
        None,
        "standard's first wave leaves later than the rush's, at {}: {report:?}",
        clock(Some(first))
    );
}

#[test]
fn a_gdo_rush_attacks_before_standard_would() {
    the_rush_attacks_before_standard_would(Faction::Gdo);
}

#[test]
fn a_syndicate_rush_attacks_before_standard_would() {
    the_rush_attacks_before_standard_would(Faction::Syndicate);
}

#[test]
fn a_cults_rush_attacks_before_standard_would() {
    the_rush_attacks_before_standard_would(Faction::Cults);
}

#[test]
fn a_colonists_rush_attacks_before_standard_would() {
    the_rush_attacks_before_standard_would(Faction::Colonists);
}

/// The Syndicate's growth (slice A2d): standard builds a second Tunnel whose crystal hatch takes in patches of its
/// starting field that the first hatch's Agents don't mine, and by 5:00 staffs both hatches with [`AGENTS_PER_HATCH`]
/// Agents each (its third Tunnel, toward the staging point for its Tunnel Space, has no patch near), on
/// `maps/playtest.json`, explored from the start as the client starts its games, or not. On `dev_skirmish` its default
/// run keeps one: the starting field lies along the map's edge against the first Tunnel's area, where no second
/// Tunnel's area fits (`factions/syndicate/tunnel_area_and_space.md > Tunnel Area Non-Overlap Rule`).
#[test]
fn a_standard_syndicate_computer_staffs_a_second_crystal_hatch() {
    let (_, playtest) = (maps().into_iter())
        .find(|(flag, _)| flag.ends_with("maps/playtest.json"))
        .expect("maps/playtest.json");
    for explored in [false, true] {
        let mut scenario = with_factions(playtest.clone(), [Faction::Syndicate; 2]);
        scenario.setup.config.start_explored = explored;
        let at = 5 * 60 * TICKS_PER_SECOND;
        let mut staffed = Vec::new();
        watch(&scenario, &[(P1, Standard), (P2, Passive)], at, Checks::All, |sim| {
            if sim.tick() == at {
                staffed = hatches_staffed(sim, P1);
            }
        });
        let full = staffed
            .iter()
            .filter(|&&(_, agents)| agents >= AGENTS_PER_HATCH)
            .count();
        assert!(
            full >= 2,
            "explored {explored}: two crystal hatches, each with its Agents: {staffed:?}"
        );
    }
}

/// `player`'s Tunnels in operation, each with the Agents gathering Space Crystals at a patch nearer its crystal
/// hatch than any other of the player's: those that deliver there (`factions/syndicate/units/agent.md > Gathering`).
fn hatches_staffed(sim: &Sim, player: PlayerId) -> Vec<(ObjectId, usize)> {
    let own = |o: &&Object| o.owner == Some(player);
    let hatches: Vec<(ObjectId, Vec2)> = (sim.objects().filter(own))
        .filter(|o| o.kind == ObjectKind::Tunnel && o.is_operational())
        .filter_map(|o| Some((o.id, hatch_square(o, Resource::Crystals)?.center())))
        .collect();
    let mut staffed: Vec<(ObjectId, usize)> = hatches.iter().map(|&(id, _)| (id, 0)).collect();
    for agent in (sim.objects().filter(own)).filter(|o| o.kind == ObjectKind::Agent) {
        let current = agent.orders.as_ref().and_then(|orders| orders.current.as_ref());
        let Some(OrderKind::Gather(patch)) = current.map(|active| active.order.kind) else {
            continue;
        };
        let Some(at) = sim
            .object(patch)
            .filter(|o| o.kind == ObjectKind::CrystalPatch)
            .map(|o| o.pos)
        else {
            continue;
        };
        let nearest = (0..hatches.len()).min_by(|&a, &b| {
            let d = |i: usize| hatches[i].1.distance_squared(at);
            d(a).total_cmp(&d(b)).then(hatches[a].0.cmp(&hatches[b].0))
        });
        if let Some(i) = nearest {
            staffed[i].1 += 1;
        }
    }
    staffed
}

/// Design change 0416: the client starts its games with the map explored (`SimConfig::start_explored`), and a
/// computer player plays them as any other: the GDO's standard beats its passive on `dev_skirmish` explored from the
/// start, within [`STANDARD_BEATS_PASSIVE`] as `ai_games` requires of the games without it, the harness checking
/// every step (the rules' invariants, the gate, 0402's aims: the map explored shows no building).
#[test]
fn a_standard_computer_beats_a_passive_one_on_a_map_explored_from_the_start() {
    let mut scenario = gdo_vs_gdo(scenarios::dev_skirmish());
    scenario.setup.config.start_explored = true;
    let report = play(
        &scenario,
        &[(P1, Standard), (P2, Passive)],
        STANDARD_BEATS_PASSIVE,
        Checks::Rules,
    );
    assert_eq!(report.winner(), Some(P1), "standard beats passive: {report:?}");
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

/// The Cults' determinism (slice A2a), as the GDO's.
#[test]
fn the_same_cults_game_ends_the_same() {
    let scenario = with_factions(scenarios::dev_skirmish(), [Faction::Cults, Faction::Cults]);
    let game = || play(&scenario, &[(P1, Standard), (P2, Standard)], DEFAULT_RUN, Checks::Rules);
    let (first, second) = (game(), game());
    assert_eq!(first.hash, second.hash);
    assert_eq!(first.sides, second.sides);
    assert!(first.sides.iter().all(|s| s.waves >= 1), "both attacked: {first:?}");
}

/// Determinism with the Syndicate (slice A2b): its mirror, both standard, to the first waves.
#[test]
fn the_same_syndicate_game_ends_the_same() {
    let scenario = with_factions(scenarios::dev_skirmish(), [Faction::Syndicate; 2]);
    let game = || play(&scenario, &[(P1, Standard), (P2, Standard)], DEFAULT_RUN, Checks::Rules);
    let (first, second) = (game(), game());
    assert_eq!(first.hash, second.hash);
    assert_eq!(first.sides, second.sides);
    assert!(first.sides.iter().all(|s| s.waves >= 1), "both attacked: {first:?}");
}

/// Determinism with the rush (slice A2d): every faction's, two to a game, rush against rush, each game twice.
#[test]
fn the_same_rush_game_ends_the_same() {
    for factions in [[Faction::Gdo, Faction::Cults], [Faction::Syndicate, Faction::Colonists]] {
        let scenario = with_factions(scenarios::dev_skirmish(), factions);
        let game = || play(&scenario, &[(P1, Rush), (P2, Rush)], DEFAULT_RUN, Checks::Rules);
        let (first, second) = (game(), game());
        assert_eq!(first.hash, second.hash, "{factions:?}");
        assert_eq!(first.sides, second.sides, "{factions:?}");
        assert!(first.sides.iter().all(|s| s.waves >= 1), "both attacked: {first:?}");
    }
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
                let mut scenario = with_factions(scenario.clone(), [a, b]);
                // Slice M5-S7: the map unexplored at the start, as the rows have always been played (design change
                // 0416 made explored the client's default), so that they compare with master's.
                scenario.setup.config.start_explored = false;
                for profiles in pairings(a == b) {
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
    let wanted: Vec<String> = (rows.iter())
        .filter(|row| row.factions[0] != row.factions[1] && row.profiles != [Standard, Standard])
        .map(|row| {
            let met = if row.standard_won() { "yes" } else { "no" };
            format!("  {}: {met}", row.label())
        })
        .collect();
    if !wanted.is_empty() {
        println!(
            "Wanted, not required: standard beats another faction's passive within 20 minutes\n{}",
            wanted.join("\n")
        );
    }
    let rushes: Vec<String> = (rows.iter())
        .filter_map(|row| {
            let rush = row.player(Rush)?;
            let first = |row: &Row| clock(row.report.side(rush).first_wave);
            let standard = rows.iter().find(|r| r.is_standard_of(row))?;
            let winner = row.report.winner().map_or("none".to_owned(), |p| format!("{}", p.0));
            Some(format!(
                "  {}: first wave {} (standard's {}), winner {winner} at {}",
                row.label(),
                first(row),
                first(standard),
                clock(Some(row.report.ticks))
            ))
        })
        .collect();
    println!(
        "The rush: its first wave, and standard's in its place\n{}",
        rushes.join("\n")
    );
    for row in &rows {
        let ended = row.report.winner().is_some();
        let mirror = row.factions[0] == row.factions[1];
        match row.profiles {
            [Standard, Standard] => assert!(ended, "{}: standard against standard ends in a defeat", row.label()),
            [Standard, Passive] | [Passive, Standard] if mirror => assert!(
                row.won(Standard) && row.report.ticks <= STANDARD_BEATS_PASSIVE,
                "{}: standard beats passive within 15 minutes",
                row.label()
            ),
            [Rush, Passive] | [Passive, Rush] => {
                let rush = row.player(Rush).expect("a rush");
                let standard = rows.iter().find(|r| r.is_standard_of(row)).expect("standard's row");
                let (early, late) = (row.report.side(rush).first_wave, standard.report.side(rush).first_wave);
                assert!(
                    early.is_some_and(|early| late.is_none_or(|late| early < late)),
                    "{}: the rush attacks before standard's first wave would ({early:?}, {late:?})",
                    row.label()
                );
            }
            _ => {}
        }
        for (side, profile) in row.report.sides.iter().zip(row.profiles) {
            let faction = row.factions[side.player.0 as usize - 1];
            let enemy = row.factions[2 - side.player.0 as usize];
            let bank = side.bank.unwrap_or(0);
            if profile == Standard && faction == Faction::Colonists && mirror {
                assert!(bank < 500, "{}: the Colonists' bank, {bank} on average", row.label());
            }
            if profile == Standard && faction == Faction::Cults && enemy == Faction::Gdo && !mirror {
                let passive = row.profiles.contains(&Passive);
                assert!(
                    !passive || bank < 1000,
                    "{}: the Cults' bank, {bank} on average",
                    row.label()
                );
            }
        }
    }
}

/// The profiles `ai_games` plays a pairing with: standard against passive from either slot, and against standard; in
/// a faction's `mirror`, the rush too (slice A2d), against passive and against standard from either slot. The A2d
/// brief's cut: a rush in every pairing would double the table's time.
fn pairings(mirror: bool) -> Vec<[Profile; 2]> {
    let mut pairings = vec![[Standard, Passive], [Passive, Standard], [Standard, Standard]];
    if mirror {
        pairings.extend([[Rush, Passive], [Passive, Rush], [Rush, Standard], [Standard, Rush]]);
    }
    pairings
}

/// A game time as the table prints it, minutes and seconds; "-" for none.
fn clock(tick: Option<Tick>) -> String {
    let Some(tick) = tick else {
        return "-".to_owned();
    };
    let seconds = tick / TICKS_PER_SECOND;
    format!("{}:{:02}", seconds / 60, seconds % 60)
}

/// One of `ai_games`' games.
struct Row {
    map: String,
    factions: [Faction; 2],
    profiles: [Profile; 2],
    report: Report,
}

impl Row {
    /// The player of `profile`, the first if both are.
    fn player(&self, profile: Profile) -> Option<PlayerId> {
        let slot = self.profiles.iter().position(|&p| p == profile)?;
        Some(PlayerId(slot as u8 + 1))
    }

    /// Whether the player of `profile` won.
    fn won(&self, profile: Profile) -> bool {
        self.player(profile).is_some_and(|p| self.report.winner() == Some(p))
    }

    /// Whether this is the game `rush` would be with standard in the rush's place.
    fn is_standard_of(&self, rush: &Row) -> bool {
        let profiles = rush.profiles.map(|p| if p == Rush { Standard } else { p });
        self.map == rush.map && self.factions == rush.factions && self.profiles == profiles
    }

    /// Whether the standard player won, against a passive one.
    fn standard_won(&self) -> bool {
        self.won(Standard)
    }

    fn label(&self) -> String {
        let [a, b] = self.factions.map(|f| format!("{f:?}").to_lowercase());
        let [x, y] = self.profiles.map(|p| p.name());
        format!("{} --factions {a},{b} --ai 1:{x},2:{y}", self.map)
    }
}

/// `ai_games`' table: a line for each game, with each side's first wave, mean bank and income from 3:00 to 10:00
/// (`harness::Side`).
fn table(rows: &[Row]) -> String {
    let mut lines = vec![format!(
        "{:<44} {:<9} {:<9} {:>6} {:>6} {:>11} {:>7} {:>13} {:>13} {:>9} {:>15}",
        "map, factions",
        "player 1",
        "player 2",
        "winner",
        "at",
        "trained",
        "waves",
        "first wave",
        "bank 3-10",
        "SC/s",
        "think mean/max"
    )];
    let or_dash = |n: Option<u32>| n.map_or("-".to_owned(), |n| n.to_string());
    for row in rows {
        let r = &row.report;
        let [a, b] = row.factions.map(|f| format!("{f:?}").to_lowercase());
        let winner = r.winner().map_or("none".to_owned(), |p| format!("{}", p.0));
        let (one, two) = (r.side(P1), r.side(P2));
        lines.push(format!(
            "{:<44} {:<9} {:<9} {:>6} {:>6} {:>11} {:>7} {:>13} {:>13} {:>9} {:>15}",
            format!("{} {a},{b}", row.map),
            row.profiles[0].name(),
            row.profiles[1].name(),
            winner,
            clock(Some(r.ticks)),
            format!("{} / {}", one.trained, two.trained),
            format!("{} / {}", one.waves, two.waves),
            format!("{} / {}", clock(one.first_wave), clock(two.first_wave)),
            format!("{} / {}", or_dash(one.bank), or_dash(two.bank)),
            format!("{} / {}", or_dash(one.income), or_dash(two.income)),
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
