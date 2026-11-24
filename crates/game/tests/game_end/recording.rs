//! The recording at the end of the game: saved, complete; a QA run's later stops come down to the last tick,
//! and a playback pauses there.

use super::*;

/// `core/game_rules.md > End of Game`: "the replay is saved". The recording is written at the last tick, complete,
/// and plays back to the same end.
#[test]
fn the_replay_is_saved_when_the_game_ends() {
    let path = std::env::temp_dir()
        .join(format!("sc_game_end_test_{}", std::process::id()))
        .join("end.json");
    let mut config = AppConfig::new(duel());
    config.record = Some(path.clone());
    let mut app = game_as(config, P1);
    run_until(&mut app, 30);
    remove(&mut app, 2);
    let end = tick(&app);
    let recorder = app.world().resource::<Recorder>();
    assert!(recorder.is_finished());
    assert_eq!(
        recorder.saved_at(),
        Some(end),
        "written at once, not at the next autosave"
    );
    let replay = load_replay(&path).unwrap();
    std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    assert_eq!(replay.end_tick(), end);
    assert_eq!(replay.checkpoints.last().map(|c| c.tick), Some(end));
    let sim = app.world().resource::<SimRes>();
    let played = replay.run();
    assert_eq!(played.outcome(), sim.outcome());
    assert_eq!(played.state_hash(), sim.state_hash());
}

/// A QA run whose exit tick lies after the end of the game exits at the end instead of waiting for ever.
#[test]
fn a_qa_run_exits_at_the_end_of_the_game() {
    let mut config = AppConfig::new(duel());
    config.qa = QaConfig {
        exit_at_tick: Some(200),
        ..Default::default()
    };
    let mut app = game_as(config, P1);
    run_until(&mut app, 3);
    remove(&mut app, 2);
    let end = tick(&app);
    let mut exit = None;
    for _ in 0..20 {
        app.update();
        exit = exit.or(app.should_exit());
    }
    assert_eq!(exit, Some(AppExit::Success));
    assert_eq!(tick(&app), end);
}

/// A recording made outside the client may go on after the end of its game, as a sim steps whenever it is asked
/// (here: a removal in tick 10, and 30 ticks more): its playback stops where the game ended, and pauses there.
#[test]
fn a_playback_pauses_at_the_end_of_the_game() {
    let mut scenario = duel();
    scenario.setup.config = SimConfig::DEBUG;
    let mut replay = Replay::new(scenario);
    let mut sim = replay.start();
    replay.record_checked(None, &sim);
    while sim.tick() < 40 {
        let mut inputs = TickInputs::default();
        if sim.tick() + 1 == 10 {
            inputs.commands.push(PlayerCommand {
                player: P1,
                command: Command::Debug(DebugCommand::Remove { id: ObjectId(2) }),
            });
        }
        sim.step(&inputs);
        replay.record_checked(Some(inputs), &sim);
    }
    replay.checkpoint(&sim);
    assert_eq!(sim.outcome().map(|o| o.tick), Some(10));

    let mut app = game_as(AppConfig::playback(replay, PathBuf::from("end.json")), P1);
    for _ in 0..60 {
        app.update();
    }
    assert_eq!(tick(&app), 10);
    assert!(app.world().resource::<SimControl>().paused);
    assert_eq!(app.world().resource::<Playback>().desync, None);
    assert_eq!(banner(&mut app), shown("VICTORY", "The game is over"));
}
