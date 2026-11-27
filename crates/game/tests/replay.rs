//! Recording games and playing them back (`ARCHITECTURE.md > Replays and golden hashes`). Headless, one tick per
//! update at the normal speed, as in `client.rs`; the first update only starts the clock. The computer players' games
//! (slice A1): `computers`.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

use bevy::ecs::message::Messages;
use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;

use sc_sim::testing::ScenarioBuilder;
use sc_sim::{
    Ability, Command, DebugCommand, Desync, ObjectId, ObjectKind, OrderKind, PlayerCommand, PlayerId, Replay, Scenario,
    ShieldMode, SimConfig, SpawnAt, Target, Tick, TickInputs, TilePos,
};
use space_crystals::command_ui::OrderIssued;
use space_crystals::fog::FogOfWar;
use space_crystals::hud::ReplayText;
use space_crystals::input::InjectedInput;
use space_crystals::platform::DisplayMode;
use space_crystals::qa::QaState;
use space_crystals::replay::playback::SPEEDS;
use space_crystals::replay::{Playback, Recorder, load_replay};
use space_crystals::selection::{ControlStates, Selection};
use space_crystals::sim_driver::{
    FAST_FRAME_BUDGET, LocalPlayer, Pace, PendingCommands, RecordedInputs, SimControl, SimRes,
};
use space_crystals::{AppConfig, build_app};

use crate::hud::notes;

mod computers;

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

fn headless(mut config: AppConfig) -> App {
    config.display = DisplayMode::Headless;
    let mut app = build_app(config);
    app.insert_resource(TimeUpdateStrategy::FixedTimesteps(1));
    app.finish();
    app.cleanup();
    app.update();
    app
}

fn tick(app: &App) -> Tick {
    app.world().resource::<SimRes>().tick()
}

fn hash(app: &App) -> u64 {
    app.world().resource::<SimRes>().state_hash()
}

fn tap(app: &mut App, key: KeyCode) {
    app.world_mut().resource_mut::<InjectedInput>().keys.press(key);
    app.update();
    app.world_mut().resource_mut::<InjectedInput>().keys.release(key);
    app.update();
}

fn centre(x: i32, y: i32) -> sc_sim::Vec2 {
    TilePos::new(x, y).center()
}

/// A flat 16 × 10 map: player 1's Peacekeepers 1 and 2 in the west, player 2's Guard 3 in the east.
fn skirmish() -> Scenario {
    ScenarioBuilder::new(&"................\n".repeat(10))
        .spawn(ObjectKind::Peacekeeper, Some(P1), SpawnAt::Point(centre(2, 4)))
        .spawn(ObjectKind::Peacekeeper, Some(P1), SpawnAt::Point(centre(2, 6)))
        .spawn(ObjectKind::Guard, Some(P2), SpawnAt::Point(centre(13, 5)))
        .build()
}

fn order(player: PlayerId, units: &[u32], kind: OrderKind) -> PlayerCommand {
    PlayerCommand {
        player,
        command: Command::Order {
            units: units.iter().map(|&id| ObjectId(id)).collect(),
            kind,
            queue: false,
        },
    }
}

/// What the players do in the recorded game, by the tick they do it in.
fn script(tick: Tick) -> Vec<PlayerCommand> {
    match tick {
        5 => vec![order(P1, &[1, 2], OrderKind::Move(Target::Point(centre(8, 5))))],
        20 => vec![order(P2, &[3], OrderKind::Move(Target::Point(centre(10, 2))))],
        40 => vec![PlayerCommand {
            player: P1,
            command: Command::Debug(DebugCommand::Spawn {
                kind: ObjectKind::Guard,
                owner: Some(P2),
                at: SpawnAt::Point(centre(12, 8)),
                orientation: Default::default(),
            }),
        }],
        90 => vec![order(P1, &[1, 2], OrderKind::Attack(ObjectId(3)))],
        _ => Vec::new(),
    }
}

/// The file `name` in a fresh directory of test `test`'s own. The tests of this file run as threads of one process,
/// and each removes its directory when it is done: when two tests shared one (named by the process only), one could
/// remove it under the other, whose recording was then gone when it read it back.
fn temp_file(test: &str, name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("sc_replay_test_{test}_{}", std::process::id()));
    // Left by an earlier run that had the same process id and failed before cleaning up.
    let _ = std::fs::remove_dir_all(&dir);
    dir.join(name)
}

/// The tick an `AppExit` and an update later: the app's last.
fn exit(app: &mut App) -> Tick {
    app.world_mut().write_message(AppExit::Success);
    app.update();
    tick(app)
}

/// The command panel's hotkeys [`play_recorded_game`] presses, by tick, with the unit selected: unit 2 raises its
/// Riot Shield (R) and lowers it again, and unit 1 stops (X).
const HOTKEYS: [(Tick, u32, KeyCode); 3] = [(30, 2, KeyCode::KeyR), (60, 1, KeyCode::KeyX), (70, 2, KeyCode::KeyR)];

/// Plays [`skirmish`] in a client that records it to `path`, through tick `until`, with the commands of [`script`]
/// and the [`HOTKEYS`]; then exits. Returns the state hash of each tick.
fn play_recorded_game(path: &Path, until: Tick) -> BTreeMap<Tick, u64> {
    let mut config = AppConfig::new(skirmish());
    config.record = Some(path.to_path_buf());
    let mut app = headless(config);
    let mut hashes = BTreeMap::from([(0, hash(&app))]);
    while tick(&app) < until {
        let next = tick(&app) + 1;
        app.world_mut().resource_mut::<PendingCommands>().0.extend(script(next));
        match HOTKEYS.iter().find(|(at, ..)| *at == next) {
            Some(&(_, unit, key)) => {
                select(&mut app, unit);
                tap(&mut app, key);
            }
            None => app.update(),
        }
        hashes.insert(tick(&app), hash(&app));
    }
    let last = exit(&mut app);
    hashes.insert(last, hash(&app));
    hashes
}

/// The Riot Shield of unit `id`, as the sim has it.
fn shield(app: &App, id: u32) -> ShieldMode {
    let sim = app.world().resource::<SimRes>();
    sc_sim::riot_shield::mode(sim.object(ObjectId(id)).unwrap()).expect("a Peacekeeper")
}

fn select(app: &mut App, id: u32) {
    let selection = Selection::new([(ObjectId(id), ObjectKind::Peacekeeper)]);
    app.world_mut().resource_mut::<ControlStates>().of_mut(P1).selection = selection;
}

/// `replay`, recorded without a client: the commands of [`script`] through tick `until`, with the checkpoints a
/// client notes.
fn sim_recording(scenario: Scenario, until: Tick) -> Replay {
    let mut replay = Replay::new(scenario);
    let mut sim = replay.start();
    replay.record_checked(None, &sim);
    while sim.tick() < until {
        let inputs = TickInputs {
            commands: script(sim.tick() + 1),
        };
        sim.step(&inputs);
        replay.record_checked(Some(inputs), &sim);
    }
    replay.checkpoint(&sim);
    replay
}

fn playback(replay: Replay) -> App {
    headless(AppConfig::playback(replay, PathBuf::from("test.json")))
}

fn top_bar(app: &mut App) -> String {
    let mut texts = app.world_mut().query_filtered::<&Text, With<ReplayText>>();
    texts.single(app.world()).unwrap().0.clone()
}

/// The client writes the recording when the game starts, every 160 ticks, and when the app exits, each time with a
/// checkpoint of the latest tick; the file is the game as the sim played it.
#[test]
fn games_are_recorded_as_they_run_and_when_the_app_exits() {
    let path = temp_file("recorded", "recorded.json");
    let mut config = AppConfig::new(skirmish());
    config.record = Some(path.clone());
    let mut app = headless(config);
    let start = load_replay(&path).expect("written when the game starts");
    assert_eq!((start.end_tick(), start.checkpoints.len()), (0, 1));
    while tick(&app) < 170 {
        let next = tick(&app) + 1;
        app.world_mut().resource_mut::<PendingCommands>().0.extend(script(next));
        app.update();
        if tick(&app) == 160 {
            let saved = load_replay(&path).unwrap();
            assert_eq!(saved.end_tick(), 160, "written at tick 160");
            assert_eq!(saved.checkpoint_at(160).map(|c| c.hash), Some(hash(&app)));
        }
    }
    assert_eq!(load_replay(&path).unwrap().end_tick(), 160, "not yet again");
    let last = exit(&mut app);
    let saved = load_replay(&path).unwrap();
    std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    assert_eq!(&saved, app.world().resource::<Recorder>().replay());
    assert_eq!(saved.end_tick(), last);
    let ticks: Vec<Tick> = saved.checkpoints.iter().map(|c| c.tick).collect();
    assert_eq!(ticks, [0, 64, 128, 160, last]);
    assert_eq!(saved.inputs, app.world().resource::<RecordedInputs>().ticks);
    assert_eq!(saved.run_checked().map(|sim| sim.state_hash()), Ok(hash(&app)));
}

/// `--replay`: the client plays the recorded game back tick for tick, matching every checkpoint, and pauses at its
/// end. The recording has the orders and the abilities given from the command panel's hotkeys. The local player
/// can't give commands meanwhile: the command panel's hotkeys don't work, and nothing else that is pending reaches
/// the sim.
#[test]
fn a_recorded_game_plays_back_as_it_was_played() {
    let path = temp_file("played", "played.json");
    let hashes = play_recorded_game(&path, 200);
    let replay = load_replay(&path).unwrap();
    std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    let end = replay.end_tick();
    assert_eq!(hashes.keys().last(), Some(&end));
    let shield_of_2 = |raise| PlayerCommand {
        player: P1,
        command: Command::Ability {
            units: vec![ObjectId(2)],
            ability: Ability::RiotShield { raise },
        },
    };
    let from_hotkeys = [30, 60, 70].map(|tick| replay.inputs[tick].commands.clone());
    let expected = [
        vec![shield_of_2(true)],
        vec![order(P1, &[1], OrderKind::Stop)],
        vec![shield_of_2(false)],
    ];
    assert_eq!(from_hotkeys, expected, "recorded from the hotkeys");

    let mut app = playback(replay.clone());
    assert_eq!(app.world().resource::<Playback>().end_tick(), end);
    let mut compared = 0;
    while tick(&app) < end {
        app.world_mut().resource_mut::<PendingCommands>().0.push(order(
            P1,
            &[1, 2],
            OrderKind::Move(Target::Point(centre(1, 1))),
        ));
        if tick(&app) == 100 {
            select(&mut app, 2);
            tap(&mut app, KeyCode::KeyX);
            tap(&mut app, KeyCode::KeyR);
            assert!(
                app.world().resource::<Messages<OrderIssued>>().is_empty(),
                "no command panel"
            );
            assert_eq!(shield(&app, 2), ShieldMode::Lowered, "nor its abilities");
        } else {
            app.update();
        }
        if tick(&app) == 50 {
            assert_eq!(shield(&app, 2), ShieldMode::Raised, "as recorded");
        }
        // The recording missed the hashes of the first of its hotkeys' two updates.
        if let Some(&recorded) = hashes.get(&tick(&app)) {
            assert_eq!(hash(&app), recorded, "tick {}", tick(&app));
            compared += 1;
        }
    }
    assert_eq!(compared, end - 6, "every tick but 30, 60, 70 and 101 to 103");
    for _ in 0..5 {
        app.update();
    }
    assert_eq!(tick(&app), end, "paused at the end");
    assert!(app.world().resource::<SimControl>().paused);
    assert_eq!(app.world().resource::<RecordedInputs>().ticks, replay.inputs);
    assert_eq!(app.world().resource::<Playback>().desync, None);
    assert!(app.world().resource::<QaState>().failures.is_empty());
    assert!(
        !app.world().contains_resource::<Recorder>(),
        "a playback isn't recorded"
    );
    assert_eq!(top_bar(&mut app), "Replay x1, end");
}

/// The playback keys (placeholders): K pauses and resumes (Space until slice M5-MM2, whose alerts took it), the period
/// plays one tick, minus and equals step through the speeds (x0.5 to as fast as possible, which stops at the end too).
/// The top bar shows them.
#[test]
fn playback_pauses_steps_and_changes_speed() {
    let mut app = playback(sim_recording(skirmish(), 300));
    while tick(&app) < 10 {
        app.update();
    }
    assert_eq!(top_bar(&mut app), "Replay x1");
    tap(&mut app, KeyCode::KeyK);
    let paused = tick(&app);
    assert!(app.world().resource::<SimControl>().paused);
    app.update();
    app.update();
    assert_eq!(tick(&app), paused);
    assert_eq!(top_bar(&mut app), "Replay x1, paused");
    tap(&mut app, KeyCode::Period);
    assert_eq!(tick(&app), paused + 1, "one tick");
    app.update();
    assert_eq!(tick(&app), paused + 1, "and paused again");
    tap(&mut app, KeyCode::KeyK);
    let resumed = tick(&app);
    app.update();
    assert_eq!(tick(&app), resumed + 1);

    let speed = |app: &App| {
        let pace = *app.world().resource::<Pace>();
        let virtual_speed = app.world().resource::<Time<Virtual>>().relative_speed();
        (pace, virtual_speed)
    };
    tap(&mut app, KeyCode::Equal);
    assert_eq!(speed(&app), (Pace::Scaled(2.0), 2.0));
    let before = tick(&app);
    app.update();
    assert_eq!(tick(&app), before + 2, "two ticks per tick of real time");
    assert_eq!(top_bar(&mut app), "Replay x2");
    tap(&mut app, KeyCode::Minus);
    tap(&mut app, KeyCode::Minus);
    tap(&mut app, KeyCode::Minus);
    assert_eq!(speed(&app), (Pace::Scaled(0.5), 0.5), "the slowest");
    let before = tick(&app);
    app.update();
    app.update();
    assert_eq!(tick(&app), before + 1, "one tick per two ticks of real time");
    for _ in 0..SPEEDS.len() {
        tap(&mut app, KeyCode::Equal);
    }
    assert_eq!(app.world().resource::<Pace>(), &Pace::Max, "the fastest");
    // How many ticks a frame runs at the fastest pace depends on how fast the machine is, but every frame runs one at
    // least while the playback may step: within as many frames as ticks are left, it is at the end.
    assert!(tick(&app) <= 300, "not past the end");
    for _ in tick(&app)..300 {
        app.update();
        assert!(tick(&app) <= 300, "not past the end");
    }
    assert_eq!(tick(&app), 300, "at the end");
    app.update();
    assert_eq!(tick(&app), 300);
    assert_eq!(top_bar(&mut app), "Replay max, end");
    tap(&mut app, KeyCode::KeyK);
    tap(&mut app, KeyCode::Period);
    assert_eq!(tick(&app), 300, "nothing after the end");
}

/// Sets the pace in `Update`, as the playback keys do.
#[derive(Resource)]
struct SetPace(Option<Pace>);

fn set_pace(mut set: ResMut<SetPace>, mut pace: ResMut<Pace>) {
    if let Some(new) = set.0.take() {
        *pace = new;
    }
}

/// Leaving the fastest pace runs no burst of ticks. Changed in `Update`, as the playback keys change it, the new
/// pace holds from the next frame on; changed between frames, the frame whose time was still taken at the fastest
/// pace runs at most a frame's worth of ticks at the new one (250 ms: five ticks).
#[test]
fn leaving_the_fastest_pace_runs_no_burst_of_ticks() {
    let mut config = AppConfig::new(skirmish());
    config.fast = true;
    let mut app = headless(config);
    app.insert_resource(SetPace(None)).add_systems(Update, set_pace);
    app.update();
    // As fast as it can: a frame runs ticks until its budget is spent. How many fit depends on how fast the machine
    // is, so the frame ran more than a second's worth of them, or else spent its whole budget.
    let before = tick(&app);
    let started = Instant::now();
    app.update();
    let (ran, took) = (tick(&app) - before, started.elapsed());
    assert!(
        ran > 16 || took >= FAST_FRAME_BUDGET,
        "as fast as it can: {ran} ticks in {took:?}"
    );

    app.world_mut().resource_mut::<SetPace>().0 = Some(Pace::Scaled(1.0));
    app.update();
    let before = tick(&app);
    app.update();
    assert_eq!(tick(&app), before + 1, "one tick per tick of real time, at once");

    *app.world_mut().resource_mut::<Pace>() = Pace::Max;
    app.update();
    app.update();
    *app.world_mut().resource_mut::<Pace>() = Pace::Scaled(1.0);
    let before = tick(&app);
    app.update();
    assert!(tick(&app) <= before + 5, "{} ticks in a frame", tick(&app) - before);
    let before = tick(&app);
    app.update();
    assert_eq!(tick(&app), before + 1);
}

/// F2 switches the player and F3 the fog in a playback as in a game with debug commands, even when the recorded game
/// had none: they only change what the replay shows. F2 is the selection's own switch (`selection::systems`), so it
/// switches once, not twice, and each player keeps their ControlState as in a game. (M5-S6) The top bar says while the
/// fog is off, as in a game.
#[test]
fn playback_switches_the_player_and_the_fog_even_without_debug_commands() {
    for debug in [false, true] {
        let mut scenario = skirmish();
        let config = SimConfig {
            debug,
            start_explored: false,
        };
        scenario.setup.config = config;
        let mut app = playback(sim_recording(scenario, 50));
        assert_eq!(app.world().resource::<SimRes>().config(), config);
        select(&mut app, 1);
        let selection_in_use = |app: &App| {
            let player = app.world().resource::<LocalPlayer>().0;
            let states = app.world().resource::<ControlStates>();
            states.selection(player).ids().collect::<Vec<_>>()
        };
        tap(&mut app, KeyCode::F2);
        assert_eq!(app.world().resource::<LocalPlayer>().0, P2, "debug {debug}");
        assert_eq!(selection_in_use(&app), [], "player 2's own, debug {debug}");
        tap(&mut app, KeyCode::F2);
        assert_eq!(app.world().resource::<LocalPlayer>().0, P1, "debug {debug}");
        assert_eq!(selection_in_use(&app), [ObjectId(1)], "debug {debug}");
        tap(&mut app, KeyCode::F3);
        assert!(!app.world().resource::<FogOfWar>().enabled, "debug {debug}");
        assert_eq!(notes(&mut app), ["Fog off (F3)"], "debug {debug}");
        tap(&mut app, KeyCode::F3);
        assert!(app.world().resource::<FogOfWar>().enabled, "debug {debug}");
        assert_eq!(notes(&mut app), [""; 0], "debug {debug}");
    }
}

/// A playback whose game departs from the recording, here at a checkpoint whose hash was changed, reports the first
/// checkpoint that differs: in the log, as a QA failure and in the top bar. It plays on to the end.
#[test]
fn a_playback_that_departs_from_the_recording_reports_a_desync() {
    let mut replay = sim_recording(skirmish(), 200);
    let wrong = replay.checkpoint_at(128).unwrap();
    replay.checkpoints[2].hash ^= 1;
    assert_eq!(replay.checkpoints[2].tick, 128);
    let mut app = playback(replay);
    while tick(&app) < 200 {
        app.update();
    }
    let desync = Desync {
        tick: 128,
        expected: wrong.hash ^ 1,
        actual: wrong.hash,
        previous: Some(64),
    };
    assert_eq!(app.world().resource::<Playback>().desync, Some(desync));
    let failures = &app.world().resource::<QaState>().failures;
    assert_eq!(failures.len(), 1, "the first desync only");
    assert!(failures[0].contains("desync at tick 128"), "{failures:?}");
    assert_eq!(top_bar(&mut app), "Replay x1, end - DESYNC at tick 128 (see the log)");
}

/// The sim's golden games (`crates/sim/tests/golden/*.json`, beside the test's sources) are replay files: the client
/// plays them back, as fast as it can (`--fast`), to their last checkpoint.
#[test]
fn golden_games_play_back_in_the_client() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../sim/tests/golden");
    let mut played = 0;
    for entry in std::fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|e| e != "json") {
            continue;
        }
        let replay = load_replay(&path).unwrap();
        let last = *replay.checkpoints.last().unwrap();
        let mut config = AppConfig::playback(replay, path.clone());
        config.fast = true;
        let mut app = headless(config);
        while !app.world().resource::<SimControl>().paused {
            app.update();
        }
        assert_eq!((tick(&app), hash(&app)), (last.tick, last.hash), "{}", path.display());
        assert_eq!(app.world().resource::<Playback>().desync, None);
        played += 1;
    }
    assert_eq!(played, 28);
}
