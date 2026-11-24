use super::*;

fn args(s: &str) -> Vec<String> {
    s.split_whitespace().map(str::to_string).collect()
}

#[test]
fn no_flags_start_the_default_game_in_a_window() {
    let config = into_config(parse(&[]).unwrap()).unwrap();
    assert_eq!(config.scenario.name(), DEFAULT_SCENARIO);
    assert_eq!(config.player, PlayerId(1));
    assert_eq!(config.display, DisplayMode::Window);
    assert_eq!(config.window_size, DEFAULT_WINDOW_SIZE);
    assert!(!config.qa.is_active() && !config.fast);
    assert_eq!(config.scenario.setup.config, SimConfig::DEBUG);
    // The default setup: a GDO player in every slot.
    let mut expected = sc_sim::GameSetup::default_for(&config.scenario.map);
    expected.config = SimConfig::DEBUG;
    assert_eq!(config.scenario.setup, expected);
}

#[test]
fn every_flag_parses() {
    let o = parse(&args(
        "--scenario empty --player 2 --window-size 800X600 --fast --screenshot 5:/tmp/a:b.png \
         --screenshot 0:x.png --exit-at-tick 9 --dump-state d.json --windowed",
    ))
    .unwrap();
    assert_eq!(o.scenario.as_deref(), Some("empty"));
    assert_eq!(o.player, 2);
    assert_eq!(o.window_size, UVec2::new(800, 600));
    assert!(o.fast && o.windowed && !o.headless);
    assert_eq!(o.screenshots[0].tick, 5);
    assert_eq!(o.screenshots[0].path, PathBuf::from("/tmp/a:b.png"));
    assert_eq!(o.exit_at_tick, Some(9));
    assert_eq!(o.dump_state, Some(PathBuf::from("d.json")));
    assert!(o.debug);
    let off = into_config(parse(&args("--no-debug")).unwrap()).unwrap();
    assert_eq!(off.scenario.setup.config, SimConfig { debug: false });
}

#[test]
fn bad_flags_are_errors() {
    for bad in [
        "--bogus",
        "--player",
        "--player 0",
        "--window-size 800",
        "--window-size 0x600",
        "--screenshot 5",
        "--screenshot x:a.png",
        "--exit-at-tick -1",
    ] {
        assert!(parse(&args(bad)).is_err(), "{bad}");
    }
}

/// `--factions` (plan decision T24): each slot's player plays the faction it names, in slot order, any case, the
/// slots it doesn't name the GDO; the default scenario, `--scenario` and `--map` take it, and the game must still be
/// valid. A replay brings its own players, and the editor's play-tests have their own setup.
#[test]
fn factions_set_each_slots_faction() {
    use sc_sim::Faction::{Colonists, Gdo};
    let o = parse(&args("--factions colonists,GDO")).unwrap();
    assert_eq!(o.factions, Some(vec![Colonists, Gdo]));
    let factions = |config: &AppConfig| -> Vec<sc_sim::Faction> {
        config.scenario.setup.players.iter().map(|p| p.faction).collect()
    };
    let default = into_config(o).unwrap();
    assert_eq!(default.scenario.name(), DEFAULT_SCENARIO);
    assert_eq!(factions(&default), [Colonists, Gdo]);
    assert_eq!(
        default.scenario.setup.config,
        SimConfig::DEBUG,
        "the flags' game options still apply"
    );
    let scenario = into_config(parse(&args("--scenario dev_skirmish --factions gdo,colonists")).unwrap()).unwrap();
    assert_eq!(factions(&scenario), [Gdo, Colonists]);
    let one = into_config(parse(&args("--scenario empty --factions colonists")).unwrap()).unwrap();
    assert_eq!(factions(&one), [Colonists, Gdo], "the GDO in the slot not named");

    let path = std::env::temp_dir().join(format!("sc_cli_factions_{}.json", std::process::id()));
    std::fs::write(&path, serde_json::to_string(&scenarios::dev_skirmish_map()).unwrap()).unwrap();
    let map = into_config(
        parse(&args(&format!(
            "--map {} --factions colonists,colonists",
            path.display()
        )))
        .unwrap(),
    );
    std::fs::remove_file(&path).unwrap();
    assert_eq!(factions(&map.unwrap()), [Colonists, Colonists]);

    for bad in ["--factions", "--factions gdo,elves", "--factions gdo,,gdo"] {
        assert!(parse(&args(bad)).is_err(), "{bad}");
    }
    let error = parse(&args("--factions orcs")).unwrap_err();
    assert!(error.contains("gdo, syndicate, cults, colonists"), "{error}");
    for bad in [
        "--factions gdo,gdo,gdo",
        "--editor --factions colonists",
        "--editor --scenario dev_skirmish --factions colonists",
    ] {
        assert!(into_config(parse(&args(bad)).unwrap()).is_err(), "{bad}");
    }
}

#[test]
fn qa_flags_render_offscreen_and_windowless_runs_must_end() {
    let config = into_config(parse(&args("--screenshot 3:s.png --screenshot 1:t.png")).unwrap()).unwrap();
    assert_eq!(config.display, DisplayMode::Offscreen);
    assert_eq!(config.qa.exit_at_tick, Some(3));
    let windowed = into_config(parse(&args("--screenshot 3:s.png --windowed")).unwrap()).unwrap();
    assert_eq!(
        (windowed.display, windowed.qa.exit_at_tick),
        (DisplayMode::Window, None)
    );
    let headless = into_config(parse(&args("--headless --exit-at-tick 4")).unwrap()).unwrap();
    assert_eq!(headless.display, DisplayMode::Headless);
    assert!(into_config(parse(&args("--headless")).unwrap()).is_err());
}

#[test]
fn inconsistent_flags_are_errors() {
    for bad in [
        "--scenario nowhere",
        "--player 3",
        "--scenario empty --map m.json",
        "--map /nonexistent/map.json",
        "--headless --windowed --exit-at-tick 1",
        "--headless --screenshot 1:a.png",
        "--screenshot 5:a.png --exit-at-tick 4",
    ] {
        assert!(into_config(parse(&args(bad)).unwrap()).is_err(), "{bad}");
    }
}

#[test]
fn editor_flags_parse() {
    let o = parse(&args("--editor maps/a.json --player 2")).unwrap();
    assert!(o.editor);
    assert_eq!(o.editor_path, Some(PathBuf::from("maps/a.json")));
    assert_eq!(o.player, 2);
    let o = parse(&args("--editor --scenario empty --editor-script s.json")).unwrap();
    assert!(o.editor && o.editor_path.is_none());
    assert_eq!(o.editor_script, Some(PathBuf::from("s.json")));
}

#[test]
fn the_editor_starts_on_a_file_a_new_map_or_a_built_in_scenario() {
    let dir = std::env::temp_dir().join(format!("sc_cli_editor_{}", std::process::id()));
    let missing = dir.join("Frozen Lake.json");
    let editor_args = |path: &Path| vec![String::from("--editor"), path.display().to_string()];
    let config = into_config(parse(&editor_args(&missing)).unwrap()).unwrap();
    assert_eq!(config.editor.unwrap().path, Some(missing));
    assert_eq!(config.scenario.name(), "Frozen Lake");
    let terrain = config.scenario.terrain();
    assert_eq!((terrain.width(), terrain.height()), files::NEW_MAP_SIZE);

    let config = into_config(parse(&args("--editor")).unwrap()).unwrap();
    assert_eq!(config.editor.unwrap().path, None);
    assert_eq!(config.scenario.name(), files::NEW_MAP_NAME);
    let config = into_config(parse(&args("--editor --scenario dev_skirmish")).unwrap()).unwrap();
    assert_eq!(config.scenario.map, scenarios::dev_skirmish_map());

    // An invalid file opens in the editor, to be fixed there; the game refuses it.
    std::fs::create_dir_all(&dir).unwrap();
    let bad = dir.join("bad.json");
    let mut map = scenarios::empty_map(8, 6);
    map.slots[1].start = sc_sim::TilePos::new(50, 50);
    std::fs::write(&bad, serde_json::to_string(&map).unwrap()).unwrap();
    let opened = into_config(parse(&editor_args(&bad)).unwrap()).unwrap();
    assert_eq!(opened.scenario.map, map);
    let map_args = vec![String::from("--map"), bad.display().to_string()];
    let error = into_config(parse(&map_args).unwrap()).unwrap_err();
    assert!(
        error.contains("1 problem(s)") && error.contains("off the map"),
        "{error}"
    );

    // Review finding: `--player` doesn't stop the editor from opening a file, even one without that slot or
    // without slots (play-tests start as its first slot); the game checks the player.
    let mut slotless = map.clone();
    slotless.slots.clear();
    std::fs::write(&bad, serde_json::to_string(&slotless).unwrap()).unwrap();
    let mut with_player = editor_args(&bad);
    with_player.extend(args("--player 2"));
    let opened = into_config(parse(&with_player).unwrap()).unwrap();
    assert_eq!(opened.scenario.map, slotless);
    assert_eq!(opened.player, PlayerId(2));
    let error = into_config(parse(&args("--scenario empty --player 3")).unwrap()).unwrap_err();
    assert!(error.contains("has no player 3"), "{error}");
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn editor_scripts_are_json_action_lists() {
    let path = std::env::temp_dir().join(format!("sc_cli_script_{}.json", std::process::id()));
    std::fs::write(
        &path,
        r#"["FitMap", {"SetTool": "Paint"}, {"Press": [100.0, 80.0]}, "Release"]"#,
    )
    .unwrap();
    let script = load_editor_script(&path);
    std::fs::remove_file(&path).unwrap();
    assert_eq!(
        script.unwrap(),
        vec![
            EditorAction::FitMap,
            EditorAction::SetTool(crate::editor::tools::Tool::Paint),
            EditorAction::Press(bevy::math::Vec2::new(100.0, 80.0)),
            EditorAction::Release,
        ]
    );
}

#[test]
fn editor_flags_are_checked() {
    for bad in [
        "--editor-script s.json",
        "--editor m.json --scenario empty",
        "--editor --map m.json",
        "--editor --scenario nowhere",
        "--editor --screenshot 5:a.png",
        "--editor --exit-at-tick 3 --headless",
        "--editor --input-script s.json",
    ] {
        assert!(into_config(parse(&args(bad)).unwrap()).is_err(), "{bad}");
    }
    assert!(into_config(parse(&args("--editor --headless --exit-at-tick 0")).unwrap()).is_ok());
}

/// Review finding: a file claiming a map larger than the sim's largest side fails cleanly (exit code 2 in
/// `main`), before the map is allocated.
#[test]
fn a_map_file_over_the_largest_side_is_an_error() {
    let path = std::env::temp_dir().join(format!("sc_cli_huge_{}.json", std::process::id()));
    let mut json = serde_json::to_value(scenarios::empty_map(5, 4)).unwrap();
    json["terrain"] = serde_json::json!({ "tiles": [".".repeat(files::MAX_MAP_SIDE as usize + 1)] });
    std::fs::write(&path, json.to_string()).unwrap();
    let game = into_config(parse(&args(&format!("--map {}", path.display()))).unwrap()).err();
    let editor = into_config(parse(&args(&format!("--editor {}", path.display()))).unwrap()).err();
    std::fs::remove_file(&path).unwrap();
    for error in [game, editor] {
        let error = error.expect("refused");
        assert!(error.contains("too big"), "{error}");
    }
}

#[test]
fn games_are_recorded_unless_told_otherwise() {
    let config = into_config(parse(&[]).unwrap()).unwrap();
    let path = config.record.expect("a game is recorded");
    let name = path.file_name().unwrap().to_string_lossy().into_owned();
    assert!(
        path.starts_with(replay::REPLAYS_DIR) && name.ends_with("_dev_skirmish.json"),
        "{name}"
    );
    for unrecorded in [
        "--no-record",
        "--headless --exit-at-tick 5",
        "--screenshot 3:s.png",
        "--editor",
    ] {
        let config = into_config(parse(&args(unrecorded)).unwrap()).unwrap();
        assert_eq!(config.record, None, "{unrecorded}");
    }
    let file = std::env::temp_dir()
        .join(format!("sc_cli_record_{}", std::process::id()))
        .join("q.json");
    let qa = format!("--headless --exit-at-tick 5 --record {}", file.display());
    assert_eq!(
        into_config(parse(&args(&qa)).unwrap()).unwrap().record,
        Some(file.clone())
    );
    std::fs::remove_dir_all(file.parent().unwrap()).unwrap();
    for bad in ["--record a.json --no-record", "--editor --record a.json"] {
        assert!(into_config(parse(&args(bad)).unwrap()).is_err(), "{bad}");
    }
}

#[test]
fn a_replay_plays_back_with_its_own_scenario_and_options() {
    let dir = std::env::temp_dir().join(format!("sc_cli_replay_{}", std::process::id()));
    let path = dir.join("game.json");
    let mut scenario = scenarios::empty(8, 6);
    scenario.setup.config = SimConfig { debug: false };
    let mut recorded = sc_sim::Replay::new(scenario);
    let mut sim = recorded.start();
    recorded.record_checked(None, &sim);
    for _ in 0..10 {
        sim.step(&Default::default());
        recorded.record_checked(Some(Default::default()), &sim);
    }
    replay::save_replay(&path, &recorded).unwrap();
    let with = |flags: &str| into_config(parse(&args(&format!("--replay {} {flags}", path.display()))).unwrap());

    let config = with("").unwrap();
    assert_eq!(config.scenario, recorded.scenario, "game options included");
    assert_eq!((config.display, config.record), (DisplayMode::Window, None));
    assert_eq!(config.playback.map(|p| p.replay), Some(recorded.clone()));
    assert_eq!(with("--headless").unwrap().qa.exit_at_tick, Some(10), "the end");
    assert_eq!(with("--screenshot 4:x.png").unwrap().qa.exit_at_tick, Some(4));
    assert_eq!(with("--player 2 --fast --no-fog").unwrap().player, PlayerId(2));
    for bad in [
        "--scenario empty",
        "--map m.json",
        "--factions colonists",
        "--editor",
        "--no-debug",
        "--commands c.json",
        "--record r.json",
        "--headless --exit-at-tick 11",
        "--screenshot 11:x.png",
        "--player 3",
    ] {
        assert!(with(bad).is_err(), "{bad}");
    }
    std::fs::write(&path, "{}").unwrap();
    assert!(with("").unwrap_err().contains("is not a replay file"));
    std::fs::remove_dir_all(&dir).unwrap();
    assert!(with("").unwrap_err().contains("can't read"));
}

#[test]
fn a_map_file_round_trips_through_json() {
    let path = std::env::temp_dir().join(format!("sc_cli_map_{}.json", std::process::id()));
    let map = scenarios::empty_map(5, 4);
    std::fs::write(&path, serde_json::to_string(&map).unwrap()).unwrap();
    let loaded = load_map_file(&path);
    // A game setup is no part of a map file.
    std::fs::write(&path, serde_json::to_string(&scenarios::empty(5, 4)).unwrap()).unwrap();
    let scenario = load_map_file(&path);
    std::fs::remove_file(&path).unwrap();
    assert_eq!(loaded.unwrap(), map);
    assert!(scenario.unwrap_err().contains("is not a map file"));
}
