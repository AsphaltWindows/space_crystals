//! (A1) A game that computer players play is recorded like any other (`space_crystals::ai_players`: the step records
//! their commands with every other command): a QA run of two computer players, as the command line starts it
//! (`--factions gdo,gdo --ai 1,2 --headless --fast --exit-at-tick 2400 --record <file>`), and its recording, played
//! back by the sim and by the client, which ends at the state hash the game ended at.

use bevy::app::AppExit;
use space_crystals::cli;

use super::*;

/// The QA run's exit tick: 2:30 of game time, by when each computer player's first wave has gone out and met the
/// other's.
const EXIT_AT: Tick = 2400;

#[test]
fn a_computer_players_game_plays_back_as_it_was_played() {
    let path = temp_file("computers", "computers.json");
    let exit_at = EXIT_AT.to_string();
    let flags = [
        "--factions",
        "gdo,gdo",
        "--ai",
        "1,2",
        "--headless",
        "--fast",
        "--exit-at-tick",
        &exit_at,
    ];
    let mut flags: Vec<String> = flags.iter().map(|&f| f.to_owned()).collect();
    flags.extend(["--record".to_owned(), path.display().to_string()]);
    let options = cli::parse(&flags).unwrap_or_else(|e| panic!("the flags don't parse: {e}"));
    let config = cli::into_config(options).unwrap_or_else(|e| panic!("the flags start no game: {e}"));
    let mut app = build_app(config);
    app.insert_resource(TimeUpdateStrategy::FixedTimesteps(1));
    app.finish();
    app.cleanup();
    // Each update steps at least once (`--fast`: as many ticks as fit in its frame).
    let mut exit = None;
    for _ in 0..EXIT_AT + 16 {
        app.update();
        exit = app.should_exit();
        if exit.is_some() {
            break;
        }
    }
    assert_eq!(exit, Some(AppExit::Success), "no exit by tick {}", tick(&app));
    let (end, played) = (tick(&app), hash(&app));
    assert_eq!(end, EXIT_AT, "the exit tick");
    drop(app);

    let replay = load_replay(&path).expect("the recording");
    std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    assert_eq!(replay.end_tick(), end, "recorded to the exit");
    for player in [P1, P2] {
        let given = (replay.inputs.iter().flat_map(|t| &t.commands)).filter(|c| c.player == player);
        assert!(given.count() > 0, "player {}'s computer gave commands", player.0);
    }
    assert_eq!(
        replay.run_checked().map(|sim| sim.state_hash()),
        Ok(played),
        "the sim plays it back"
    );

    let mut config = AppConfig::playback(replay, path.clone());
    config.fast = true;
    let mut app = headless(config);
    for _ in 0..end + 16 {
        if app.world().resource::<SimControl>().paused {
            break;
        }
        app.update();
    }
    assert_eq!(
        (tick(&app), hash(&app)),
        (end, played),
        "the client plays it back to its end"
    );
    assert_eq!(app.world().resource::<Playback>().desync, None);
}
