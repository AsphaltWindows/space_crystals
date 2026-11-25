//! The golden game of slice M4-O (`ARCHITECTURE.md > Replays and golden hashes`; the Director's note on the goldens):
//! test throwers in a fight (`combat::testing`), so that projectiles, their blasts and an Attack Ground are pinned by
//! state hash as the units' games are (`tests/golden/`). It is a unit test, not one of those: the test defs exist only
//! in the sim's own tests (`defs::test_defs`). Its replay is blessed in `tests/golden/test_defs/`, beside the others
//! and out of the client's way, which plays back every replay of `tests/golden/` with the game's own types. The same
//! command re-blesses it: `SC_BLESS=1 cargo test -p sc_sim golden`.

use std::collections::BTreeMap;
use std::path::PathBuf;

use glam::Vec2;

use crate::combat::testing::{GRENADE, grenadier};
use crate::command::{Command, PlayerCommand, TickInputs};
use crate::defs::ObjectKind;
use crate::events::SimEvent;
use crate::ids::{ObjectId, PlayerId};
use crate::orders::OrderKind;
use crate::player::Faction;
use crate::replay::Replay;
use crate::scenario::SpawnAt;
use crate::testing::ScenarioBuilder;
use crate::units::{Tick, TilePos};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
/// The replay notes the state hash every this many ticks, at tick 0 and at its end, as the units' goldens do.
const EVERY: Tick = 32;
const END: Tick = 480;
/// Player 1's throwers.
const THROWERS: [ObjectId; 3] = [ObjectId(1), ObjectId(2), ObjectId(3)];
const HIDEOUT: ObjectId = ObjectId(5);

fn path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden/test_defs/area_attacks.json")
}

fn at(x: f32, y: f32) -> SpawnAt {
    SpawnAt::Point(Vec2::new(x, y))
}

fn order(player: PlayerId, units: &[u32], kind: OrderKind) -> PlayerCommand {
    PlayerCommand {
        player,
        command: Command::Order {
            units: units.iter().copied().map(ObjectId).collect(),
            kind,
            queue: false,
        },
    }
}

/// Player 1's three throwers (1 to 3, the Guard with the test grenade) and a Peacekeeper of theirs ahead of them (4);
/// player 2's Hideout (5), out of sight in the north-east, and four Peacekeepers (6 to 9) holding a line, three of them
/// close together. Thrower 1 attacks the ground over the Hideout, which it doesn't see, and keeps shelling it (an
/// Attack Ground keeps firing, design change 0293): its fifth grenade destroys the Hideout, player 2's last structure,
/// which defeats player 2, and it shells the empty ground on to the end. The others fight by themselves, their grenades
/// locked where the Peacekeepers stand, player 1's own Peacekeeper among them when it walks in.
fn record() -> Replay {
    let scenario = ScenarioBuilder::new(&format!("{}\n", ".".repeat(24)).repeat(12))
        .faction(P2, Faction::Syndicate)
        .spawn(ObjectKind::Guard, Some(P1), at(300.0, 160.0))
        .spawn(ObjectKind::Guard, Some(P1), at(330.0, 400.0))
        .spawn(ObjectKind::Guard, Some(P1), at(330.0, 520.0))
        .spawn(ObjectKind::Peacekeeper, Some(P1), at(420.0, 460.0))
        .spawn(ObjectKind::Hideout, Some(P2), SpawnAt::Tile(TilePos::new(8, 0)))
        .spawn(ObjectKind::Peacekeeper, Some(P2), at(600.0, 380.0))
        .spawn(ObjectKind::Peacekeeper, Some(P2), at(630.0, 400.0))
        .spawn(ObjectKind::Peacekeeper, Some(P2), at(600.0, 420.0))
        .spawn(ObjectKind::Peacekeeper, Some(P2), at(640.0, 560.0))
        .build();
    let hideout_centre = Vec2::new(576.0, 64.0);
    let commands = [
        (1, order(P2, &[6, 7, 8, 9], OrderKind::Hold)),
        (1, order(P1, &[1], OrderKind::AttackGround(hideout_centre))),
    ];
    let mut replay = Replay::new(scenario);
    let mut sim = replay.start();
    replay.checkpoint(&sim);
    for tick in 1..=END {
        let commands = commands.iter().filter(|(t, _)| *t == tick).map(|(_, c)| c.clone());
        let inputs = TickInputs {
            commands: commands.collect(),
        };
        sim.step(&inputs);
        replay.record(inputs);
        if tick.is_multiple_of(EVERY) || tick == END {
            replay.checkpoint(&sim);
        }
    }
    replay
}

/// The replay's JSON as the units' goldens have it (`tests/golden/harness.rs`): the scenario pretty-printed, and one
/// line per tick's inputs and per checkpoint, so that a re-bless shows as a short diff.
fn json(replay: &Replay) -> String {
    let scenario = serde_json::to_string_pretty(&replay.scenario)
        .unwrap()
        .replace('\n', "\n  ");
    let lines = |items: Vec<String>| items.join(",\n    ");
    let inputs = (replay.inputs.iter())
        .map(|i| serde_json::to_string(i).unwrap())
        .collect();
    let checkpoints = (replay.checkpoints.iter())
        .map(|c| serde_json::to_string(c).unwrap())
        .collect();
    format!(
        "{{\n  \"scenario\": {scenario},\n  \"inputs\": [\n    {}\n  ],\n  \"checkpoints\": [\n    {}\n  ]\n}}\n",
        lines(inputs),
        lines(checkpoints)
    )
}

/// The game still does what it is there for: grenades flew, several at once; one blast hit two objects; the Hideout
/// was hit, again and again until it fell (design change 0293); and some died of it all.
fn check(replay: &Replay) {
    let mut sim = replay.start();
    let (mut launched, mut most_in_flight, mut most_in_one_blast) = (0, 0, 0);
    let (mut hideout_hit, mut hideout_fell, mut died) = (false, false, 0);
    while replay.step(&mut sim) {
        most_in_flight = most_in_flight.max(sim.projectiles().len());
        let mut blasts = 0;
        let mut struck = BTreeMap::new();
        for e in sim.events() {
            match *e {
                SimEvent::ProjectileLaunched { .. } => launched += 1,
                SimEvent::Blast { .. } => blasts += 1,
                SimEvent::Hit { attacker, target, .. } if THROWERS.contains(&attacker) => {
                    *struck.entry(attacker).or_insert(0) += 1;
                    hideout_hit |= target == HIDEOUT;
                }
                SimEvent::Died { id, .. } => {
                    died += 1;
                    hideout_fell |= id == HIDEOUT;
                }
                _ => {}
            }
        }
        if blasts == 1 {
            most_in_one_blast = struck.values().copied().max().unwrap_or(0).max(most_in_one_blast);
        }
    }
    assert!(launched >= 6, "{launched} grenades");
    assert!(most_in_flight >= 2, "at most {most_in_flight} in flight at once");
    assert!(most_in_one_blast >= 2, "no blast hit more than {most_in_one_blast}");
    assert!(hideout_hit, "the Hideout under the Attack Ground");
    assert!(hideout_fell, "the Attack Ground shelled the Hideout until it fell");
    assert!(died >= 2, "{died} died");
}

const BLESS: &str = "re-bless if the change is intended: SC_BLESS=1 cargo test -p sc_sim golden";

/// Plays the game with the test throwers and compares its checkpoints with the blessed file, or rewrites the file when
/// `SC_BLESS=1`; and plays the recorded replay back, checking every checkpoint (`Replay::run_checked`).
#[test]
fn golden_area_attacks_with_test_throwers() {
    let _grenadier = grenadier(GRENADE);
    let replay = record();
    check(&replay);
    replay.run_checked().expect("the replay plays back");
    let path = path();
    if std::env::var("SC_BLESS").is_ok_and(|v| v == "1") {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, json(&replay)).unwrap();
        return;
    }
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}; {BLESS}", path.display()));
    let blessed: Replay = serde_json::from_str(&text).unwrap_or_else(|e| panic!("{}: {e}; {BLESS}", path.display()));
    assert!(
        blessed.scenario == replay.scenario && blessed.inputs == replay.inputs,
        "the game's definition changed since it was blessed; {BLESS}"
    );
    let differs = (0..replay.checkpoints.len().max(blessed.checkpoints.len()))
        .find(|&i| replay.checkpoints.get(i) != blessed.checkpoints.get(i));
    if let Some(i) = differs {
        panic!(
            "the game diverges at checkpoint {:?} (blessed {:?}); {BLESS}",
            replay.checkpoints.get(i),
            blessed.checkpoints.get(i)
        );
    }
}
