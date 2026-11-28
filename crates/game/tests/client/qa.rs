//! QA runs (`docs/architecture/qa-mode.md > QA mode`): the exit tick and the state dump, timed commands from a file, the path
//! overlay, and (slice M6-S6a) an input script under `--fast`.

use bevy::prelude::*;

use sc_sim::testing::ScenarioBuilder;
use sc_sim::{
    Command, DebugCommand, Faction, ObjectId, ObjectKind, OrderKind, PlayerCommand, PlayerId, SpawnAt, Target, Tick,
    TilePos,
};
use space_crystals::AppConfig;
use space_crystals::camera::{CameraView, ViewportInsets, viewport_rect, visible_extent_su};
use space_crystals::cli;
use space_crystals::input::InputAction;
use space_crystals::path_overlay::{PathOverlay, PathOverlayMesh};
use space_crystals::platform::SurfaceSize;
use space_crystals::qa::input_script::ScriptButton;
use space_crystals::qa::{InputStep, QaConfig, TimedCommand};
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
            orientation: Default::default(),
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

/// Where the camera looks in [`scripted_build`]: the middle of its map, where its Recruit stands.
const MIDDLE: Vec2 = Vec2::new(2048.0, 1536.0);

/// The cursor position over the view point `view`, the camera at [`MIDDLE`].
fn screen_over(app: &App, view: Vec2) -> Vec2 {
    let size = *app.world().resource::<SurfaceSize>();
    let insets = *app.world().resource::<ViewportInsets>();
    let (position, viewport) = viewport_rect(size, insets);
    let su_per_pixel = visible_extent_su(viewport, size.physical.x, 1.0).x / viewport.x as f32;
    let center = app.world().resource::<CameraView>().center;
    ((view - center) / su_per_pixel + position.as_vec2() + viewport.as_vec2() / 2.0) / size.scale_factor
}

/// A QA run of player 1, the Cults, on a flat 64 × 48 map, its camera on the map's middle: its Storage (the Armory's
/// TechPrerequisite) and a Recruit there, player 2's Deployment Center far off, 600 Space Crystals by a debug command
/// in tick 1, and `script`, whose cursor positions [`screen_over`] gives, to tick 40, `--fast` or in real time. The
/// commands the sim took, with their ticks.
fn scripted_build(fast: bool, script: impl Fn(&App) -> Vec<InputStep>) -> Vec<(Tick, Command)> {
    let mut scenario = ScenarioBuilder::new(&vec![".".repeat(64); 48].join("\n"))
        .faction(PlayerId(1), Faction::Cults)
        .faction(PlayerId(2), Faction::Gdo)
        .spawn(
            ObjectKind::Storage,
            Some(PlayerId(1)),
            SpawnAt::Tile(TilePos::new(26, 20)),
        )
        .spawn(ObjectKind::Recruit, Some(PlayerId(1)), SpawnAt::Point(MIDDLE))
        .spawn(
            ObjectKind::DeploymentCenter,
            Some(PlayerId(2)),
            SpawnAt::Tile(TilePos::new(56, 40)),
        )
        .build();
    scenario.map.slots[0].camera_start = Some(MIDDLE);
    let mut probe = headless(AppConfig::new(scenario.clone()));
    probe.update();
    assert_eq!(probe.world().resource::<CameraView>().center, MIDDLE);
    let mut config = AppConfig::new(scenario);
    config.fast = fast;
    config.qa = QaConfig {
        exit_at_tick: Some(40),
        no_fog: true,
        commands: vec![TimedCommand {
            tick: 1,
            player: PlayerId(1),
            command: Command::Debug(DebugCommand::SetResources {
                player: PlayerId(1),
                crystals: 600,
                supplies: 0,
                alloys: 0,
                essence: 0,
                flux: 0,
            }),
        }],
        input_script: script(&probe),
        ..Default::default()
    };
    let mut app = headless(config);
    for _ in 0..2000 {
        app.update();
        if app.should_exit().is_some() {
            break;
        }
    }
    assert_eq!(app.should_exit(), Some(AppExit::Success));
    assert_eq!(tick(&app), 40, "the run ends at its exit tick");
    let recorded = &app.world().resource::<RecordedInputs>().ticks;
    (recorded.iter().enumerate())
        .flat_map(|(i, inputs)| inputs.commands.iter().map(move |c| (i as Tick + 1, c.command.clone())))
        .collect()
}

/// Slice M6-S6a (Z4a): under `--fast` an input script acts as in real time, the key and the click right after a menu
/// key included (slice M5-Mb's walkthrough found them lost: W, then A, then a click placed no Armory). A click in tick
/// 3 selects the Recruit, W in tick 10 opens its Basic Construct menu, A in tick 11 awaits the Armory's placement, and
/// a click in tick 13 places it, the run going on to tick 40: at both paces the Armory's Build goes in tick 14, the
/// tick after the click's (`qa::input_script`: the sim waits a frame at a step's tick, in which a tapped click is
/// read). Under `--fast` that frame used to run the sim on to the exit tick before the click was read, its order never
/// given.
#[test]
fn under_fast_a_menu_key_and_the_key_and_click_after_it_act_as_in_real_time() {
    let script = |app: &App| {
        let step = |tick, cursor: Option<Vec2>, tap: &[ScriptButton]| InputStep {
            tick,
            cursor: cursor.map(|view| screen_over(app, view)),
            press: Vec::new(),
            tap: tap.to_vec(),
            release: Vec::new(),
        };
        let (left, w, a) = (
            ScriptButton::Mouse(MouseButton::Left),
            ScriptButton::Key(KeyCode::KeyW),
            ScriptButton::Key(KeyCode::KeyA),
        );
        let site = MIDDLE + Vec2::new(256.0, -320.0);
        vec![
            step(3, Some(MIDDLE), &[left]),
            step(10, None, &[w]),
            step(11, None, &[a]),
            step(12, Some(site), &[]),
            step(13, Some(site), &[left]),
        ]
    };
    let real = scripted_build(false, script);
    let built: Vec<Tick> = (real.iter())
        .filter(|(_, c)| {
            matches!(
                c,
                Command::Order {
                    kind: OrderKind::Build {
                        kind: ObjectKind::Armory,
                        ..
                    },
                    ..
                }
            )
        })
        .map(|&(tick, _)| tick)
        .collect();
    assert_eq!(built, [14], "{real:?}");
    assert_eq!(scripted_build(true, script), real, "--fast");
}
