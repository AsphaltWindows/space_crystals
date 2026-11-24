//! QA runs (`ARCHITECTURE.md > QA flags`): the exit tick and the state dump, timed commands from a file, and the
//! path overlay.

use bevy::prelude::*;

use sc_sim::{Command, DebugCommand, ObjectId, ObjectKind, OrderKind, PlayerCommand, PlayerId, SpawnAt, Target};
use space_crystals::AppConfig;
use space_crystals::cli;
use space_crystals::input::InputAction;
use space_crystals::path_overlay::{PathOverlay, PathOverlayMesh};
use space_crystals::qa::{QaConfig, TimedCommand};
use space_crystals::sim_driver::{RecordedInputs, SimRes};

use super::{headless, press, small_scenario, tick};

#[test]
fn a_qa_run_holds_at_its_exit_tick_and_dumps_the_state() {
    let dump = std::env::temp_dir().join(format!("sc_client_dump_{}.json", std::process::id()));
    let mut config = AppConfig::new(small_scenario());
    config.qa = QaConfig {
        exit_at_tick: Some(5),
        dump_state: Some(dump.clone()),
        ..Default::default()
    };
    let mut app = headless(config);
    let mut exit = None;
    for _ in 0..50 {
        app.update();
        exit = exit.or(app.should_exit());
    }
    assert_eq!(exit, Some(AppExit::Success));
    assert_eq!(tick(&app), 5, "the sim waits at the exit tick");

    let text = std::fs::read_to_string(&dump).unwrap();
    std::fs::remove_file(&dump).unwrap();
    let json: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(json["tick"], 5);
    assert_eq!(json["scenario"], "empty");
    assert_eq!(json["objects"].as_array().unwrap().len(), 4);
    assert_eq!(json["objects"][3]["kind"], "SupplyDeliveryStation");
    assert_eq!(json["client"]["local_player"], 1);
    assert_eq!(json["client"]["recorded_ticks"], 5);
    assert!(json["map"].is_object(), "the map is part of the dump");
}

/// `--commands`: each command goes into the step of its tick, a tick's commands in file order; `--show-paths`
/// starts with the path overlay on, and `InputAction::TogglePaths` switches it.
#[test]
fn qa_commands_apply_at_their_ticks_and_the_overlay_shows_the_paths() {
    let timed = |tick, command| TimedCommand {
        tick,
        player: PlayerId(1),
        command,
    };
    let spawn = |x| {
        Command::Debug(DebugCommand::Spawn {
            kind: ObjectKind::Peacekeeper,
            owner: Some(PlayerId(1)),
            at: SpawnAt::Point(Vec2::new(x, 100.0)),
        })
    };
    let order = Command::Order {
        units: vec![ObjectId(5), ObjectId(6)],
        kind: OrderKind::Move(Target::Point(Vec2::new(900.0, 100.0))),
        queue: false,
    };
    let mut config = AppConfig::new(small_scenario()); // objects 1 to 4
    config.qa = QaConfig {
        exit_at_tick: Some(40),
        commands: vec![timed(3, order.clone()), timed(1, spawn(100.0)), timed(1, spawn(40.0))],
        show_paths: true,
        ..Default::default()
    };
    let mut app = headless(config);
    while tick(&app) < 12 {
        app.update();
    }
    let recorded = &app.world().resource::<RecordedInputs>().ticks;
    let from_1 = |command| PlayerCommand {
        player: PlayerId(1),
        command,
    };
    assert_eq!(recorded[0].commands, [from_1(spawn(100.0)), from_1(spawn(40.0))]);
    assert_eq!(recorded[1].commands, []);
    assert_eq!(recorded[2].commands, [from_1(order)]);
    let sim = app.world().resource::<SimRes>();
    // On the way to its slot near the click, at 4 su per tick.
    let unit = sim.object(ObjectId(5)).unwrap();
    assert!(((unit.pos - unit.prev_pos).length() - 4.0).abs() < 1e-4);
    assert!(unit.pos.x > 100.0 + 9.0 * 3.9, "{}", unit.pos);

    let overlay = |app: &mut App| {
        let mut query = app.world_mut().query_filtered::<&Visibility, With<PathOverlayMesh>>();
        *query.single(app.world()).unwrap()
    };
    assert_eq!(overlay(&mut app), Visibility::Inherited);
    press(&mut app, InputAction::TogglePaths);
    app.update();
    assert!(!app.world().resource::<PathOverlay>().visible);
    assert_eq!(overlay(&mut app), Visibility::Hidden);
    press(&mut app, InputAction::TogglePaths);
    app.update();
    assert_eq!(overlay(&mut app), Visibility::Inherited);
}

#[test]
fn the_commands_flag_reads_a_list_of_timed_commands() {
    let dir = std::env::temp_dir().join(format!("sc_client_commands_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("commands.json");
    let config = |text: &str, editor: bool| {
        std::fs::write(&file, text).unwrap();
        let mut args: Vec<String> = ["--commands", file.to_str().unwrap(), "--exit-at-tick", "5"]
            .map(String::from)
            .into();
        if editor {
            args.push("--editor".into());
        }
        cli::parse(&args).and_then(cli::into_config)
    };
    let stop =
        r#"[{"tick": 2, "player": 2, "command": {"Order": {"units": [18, 19], "kind": "Stop", "queue": true}}}]"#;
    let qa = config(stop, false).unwrap().qa;
    assert_eq!(
        qa.commands,
        [TimedCommand {
            tick: 2,
            player: PlayerId(2),
            command: Command::Order {
                units: vec![ObjectId(18), ObjectId(19)],
                kind: OrderKind::Stop,
                queue: true,
            },
        }]
    );
    assert!(qa.is_active() && !qa.show_paths);
    assert!(cli::parse(&["--show-paths".to_string()]).unwrap().show_paths);

    let at_0 = r#"[{"tick": 0, "player": 1, "command": {"Order": {"units": [1], "kind": "Hold", "queue": false}}}]"#;
    let err = |text, editor| config(text, editor).err().unwrap();
    assert!(err(at_0, false).contains("tick 0"), "{}", err(at_0, false));
    assert!(err(r#"[{"tick": 1}]"#, false).contains("not a list of timed commands"));
    assert!(err(stop, true).contains("--commands needs a game"));
    std::fs::remove_dir_all(&dir).unwrap();
}
