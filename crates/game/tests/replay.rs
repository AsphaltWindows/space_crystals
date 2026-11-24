//! Recording games and playing them back (`ARCHITECTURE.md > Replays and golden hashes`). Headless, one tick per
//! update at the normal speed, as in `client.rs`; the first update only starts the clock.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use bevy::ecs::message::Messages;
use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;

use sc_sim::testing::ScenarioBuilder;
use sc_sim::{
    Command, DebugCommand, Desync, ObjectId, ObjectKind, OrderKind, PlayerCommand, PlayerId, Replay, Scenario,
    SimConfig, SpawnAt, Target, Tick, TickInputs, TilePos,
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
use space_crystals::sim_driver::{LocalPlayer, Pace, PendingCommands, RecordedInputs, SimControl, SimRes};
use space_crystals::{AppConfig, build_app};

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
            }),
        }],
        90 => vec![order(P1, &[1, 2], OrderKind::Attack(ObjectId(3)))],
        _ => Vec::new(),
    }
}

fn temp_file(name: &str) -> PathBuf {
    std::env::temp_dir()
        .join(format!("sc_replay_test_{}", std::process::id()))
        .join(name)
}

/// The tick an `AppExit` and an update later: the app's last.
fn exit(app: &mut App) -> Tick {
    app.world_mut().write_message(AppExit::Success);
    app.update();
    tick(app)
}

/// Plays [`skirmish`] in a client that records it to `path`, through tick `until`, with the commands of [`script`]
/// and a Stop of unit 1 from the command panel's hotkey (X) at tick 60; then exits. Returns the state hash of each
/// tick.
fn play_recorded_game(path: &Path, until: Tick) -> BTreeMap<Tick, u64> {
    let mut config = AppConfig::new(skirmish());
    config.record = Some(path.to_path_buf());
    let mut app = headless(config);
    let mut hashes = BTreeMap::from([(0, hash(&app))]);
    while tick(&app) < until {
        let next = tick(&app) + 1;
        app.world_mut().resource_mut::<PendingCommands>().0.extend(script(next));
        if next == 60 {
            select(&mut app, 1);
            tap(&mut app, KeyCode::KeyX);
            assert!(
                !app.world().resource::<Messages<OrderIssued>>().is_empty(),
                "the Stop was given"
            );
        } else {
            app.update();
        }
        hashes.insert(tick(&app), hash(&app));
    }
    let last = exit(&mut app);
    hashes.insert(last, hash(&app));
    hashes
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
    let path = temp_file("recorded.json");
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
/// end. The local player can't give commands meanwhile: the command panel's hotkeys don't work, and nothing else
/// that is pending reaches the sim.
#[test]
fn a_recorded_game_plays_back_as_it_was_played() {
    let path = temp_file("played.json");
    let hashes = play_recorded_game(&path, 200);
    let replay = load_replay(&path).unwrap();
    std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    let end = replay.end_tick();
    assert_eq!(hashes.keys().last(), Some(&end));
    let stop = replay.inputs[60].commands.clone();
    assert_eq!(stop, [order(P1, &[1], OrderKind::Stop)], "recorded from the hotkey");

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
            assert!(
                app.world().resource::<Messages<OrderIssued>>().is_empty(),
                "no command panel"
            );
        } else {
            app.update();
        }
        // The recording missed the hash of tick 60, the first of its hotkey's two updates.
        if let Some(&recorded) = hashes.get(&tick(&app)) {
            assert_eq!(hash(&app), recorded, "tick {}", tick(&app));
            compared += 1;
        }
    }
    assert_eq!(compared, end - 2, "every tick but 60 and 101");
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

/// The playback keys (placeholders): Space pauses and resumes, the period plays one tick, minus and equals step
/// through the speeds (x0.5 to as fast as possible, which stops at the end too). The top bar shows them.
#[test]
fn playback_pauses_steps_and_changes_speed() {
    let mut app = playback(sim_recording(skirmish(), 300));
    while tick(&app) < 10 {
        app.update();
    }
    assert_eq!(top_bar(&mut app), "Replay x1");
    tap(&mut app, KeyCode::Space);
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
    tap(&mut app, KeyCode::Space);
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
    assert_eq!(tick(&app), 300, "at the end at once, and not past it");
    app.update();
    assert_eq!(tick(&app), 300);
    assert_eq!(top_bar(&mut app), "Replay max, end");
    tap(&mut app, KeyCode::Space);
    tap(&mut app, KeyCode::Period);
    assert_eq!(tick(&app), 300, "nothing after the end");
}

/// F2 switches the player and F3 the fog in a playback as in a game with debug commands, even when the recorded game
/// had none: they only change what the replay shows. With debug commands, F2 switches once, not twice.
#[test]
fn playback_switches_the_player_and_the_fog_even_without_debug_commands() {
    for debug in [false, true] {
        let mut scenario = skirmish();
        scenario.setup.config = SimConfig { debug };
        let mut app = playback(sim_recording(scenario, 50));
        assert_eq!(app.world().resource::<SimRes>().config(), SimConfig { debug });
        tap(&mut app, KeyCode::F2);
        assert_eq!(app.world().resource::<LocalPlayer>().0, P2, "debug {debug}");
        tap(&mut app, KeyCode::F2);
        assert_eq!(app.world().resource::<LocalPlayer>().0, P1, "debug {debug}");
        tap(&mut app, KeyCode::F3);
        assert!(!app.world().resource::<FogOfWar>().enabled, "debug {debug}");
        tap(&mut app, KeyCode::F3);
        assert!(app.world().resource::<FogOfWar>().enabled, "debug {debug}");
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

/// The sim's golden games (`crates/sim/tests/golden/`) are replay files: the client plays them back, as fast as it
/// can (`--fast`), to their last checkpoint.
#[test]
fn golden_games_play_back_in_the_client() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../sim/tests/golden");
    let mut played = 0;
    for entry in std::fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
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
    assert_eq!(played, 5);
}
