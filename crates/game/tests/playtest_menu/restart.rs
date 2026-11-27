//! Restart (design change 0414's Verification: "the same map, factions and computer opponents start again from 0:00,
//! starting bases as at launch"): a restarted game plays out as a fresh launch of it does, the computer players and
//! every input started over, with the launch's game options (the map explored at the start, design change 0416); each
//! game keeps its own replay (the Director's call); and nothing of the old game stays in the client: its selections,
//! control groups, camera, local player, visuals, fog and minimap are a fresh launch's.

use std::path::{Path, PathBuf};

use sc_sim::{ObjectKind, Sim, VisibilityState, scenarios};
use space_crystals::camera::CameraView;
use space_crystals::cli;
use space_crystals::fog::FogAssets;
use space_crystals::hud::minimap::MinimapView;
use space_crystals::input::ControlGroup;
use space_crystals::object_render::RememberedVisuals;
use space_crystals::replay::load_replay;
use space_crystals::selection::{ControlStates, Selection};
use space_crystals::sim_driver::LocalPlayer;

use super::*;

/// The game of `cargo run -- --factions gdo,syndicate --ai 1,2`, recorded to `record` (or not at all): the default
/// map, the GDO's standard computer against the Syndicate's, and the client's game options, the map explored at the
/// start among them.
fn ai_game(record: Option<&Path>) -> AppConfig {
    let mut flags = vec!["--factions", "gdo,syndicate", "--ai", "1,2"]
        .into_iter()
        .map(String::from)
        .collect::<Vec<_>>();
    match record {
        Some(path) => flags.extend(["--record".to_string(), path.display().to_string()]),
        None => flags.push("--no-record".to_string()),
    }
    cli::into_config(cli::parse(&flags).unwrap()).unwrap()
}

/// A directory of this test's own.
fn temp_dir(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("sc_playtest_menu_{name}_{}", std::process::id()))
}

fn sim(app: &App) -> &Sim {
    app.world().resource::<SimRes>()
}

/// Starts the game over: F10, and the menu's Restart.
fn restart(app: &mut App) {
    f10(app);
    click(app, MenuButton::Restart);
}

/// The brief's acceptance: the AI-vs-AI game restarted at tick 600 reaches, at tick 1200 after the restart, the state
/// hash a fresh launch reaches at tick 1200, so that the computer players and every input were started over; the
/// launch's game options come back with it, the map explored for both players at 0:00. While the menu is open before
/// it, the computer players don't think. Each game keeps its own replay: the first one's, complete to its tick 600, and
/// the restarted one's, beside it, numbered (`--record`), which Quit writes and which replays to the same hash.
#[test]
fn a_restarted_game_plays_out_as_a_fresh_launch_and_keeps_its_own_replay() {
    let dir = temp_dir("replays");
    let first = dir.join("game.json");
    let mut app = headless(ai_game(Some(&first)));
    run_until(&mut app, 600);
    f10(&mut app);
    let at_600 = sim(&app).state_hash();
    updates(&mut app, 60);
    assert_eq!((tick(&app), sim(&app).state_hash()), (600, at_600), "paused");
    let pending = &app.world().resource::<PendingCommands>().0;
    assert!(pending.is_empty(), "the computer players don't think");

    click(&mut app, MenuButton::Restart);
    assert!(tick(&app) <= 1, "from 0:00");
    assert_eq!(menu(&app), PlaytestMenu::Closed);
    let config = sim(&app).config();
    assert!(config.start_explored && config.debug, "the launch's options");
    for player in [P1, P2] {
        let vision = sim(&app).vision().player(player).unwrap();
        assert!(
            vision.states().all(|s| s != VisibilityState::Unexplored),
            "{player:?}: the map explored"
        );
    }
    run_until(&mut app, 1200);
    let restarted = sim(&app).state_hash();
    let mut fresh = headless(ai_game(None));
    run_until(&mut fresh, 1200);
    assert_eq!(restarted, sim(&fresh).state_hash(), "a fresh launch's game");

    let replay = load_replay(&first).unwrap();
    assert_eq!(replay.end_tick(), 600, "the first game's, complete");
    assert!(replay.run_checked().is_ok());
    assert_eq!(replay.run().state_hash(), at_600);
    let given = |player| {
        (replay.inputs.iter().flat_map(|t| &t.commands))
            .filter(|c| c.player == player)
            .count()
    };
    assert!(given(P1) > 0 && given(P2) > 0, "both computer players played");
    let second = dir.join("game.restart1.json");
    f10(&mut app);
    click(&mut app, MenuButton::Quit);
    assert_eq!(exits(&app), [AppExit::Success]);
    let replay = load_replay(&second).unwrap();
    assert_eq!(replay.end_tick(), 1200, "written at the exit");
    assert!(replay.run_checked().is_ok());
    assert_eq!(
        replay.run().state_hash(),
        restarted,
        "the restarted game's replay plays it back"
    );
    std::fs::remove_dir_all(&dir).unwrap();
}

/// The objects of `kind` that `player` owns, in id order.
fn own(app: &App, player: PlayerId, kind: ObjectKind) -> Vec<ObjectId> {
    let objects = sim(app).objects();
    objects
        .filter(|o| o.owner == Some(player) && o.kind == kind)
        .map(|o| o.id)
        .collect()
}

fn images_data(app: &App, image: &Handle<Image>) -> Vec<u8> {
    let images = app.world().resource::<Assets<Image>>();
    images.get(image).and_then(|i| i.data.clone()).expect("drawn")
}

/// What the client shows of the game, for a comparison with a fresh launch's: the sim's hash, the local player, the
/// camera, the selections and control groups, the objects drawn, live and remembered, and the fog's and the minimap's
/// images, drawn at once with the game held still.
fn shown(app: &mut App) -> impl PartialEq + std::fmt::Debug {
    app.world_mut().resource_mut::<SimControl>().paused = true;
    updates(app, 2);
    let world = app.world();
    let states = world.resource::<ControlStates>();
    let chosen: Vec<(Vec<ObjectId>, usize)> = [P1, P2]
        .map(|p| {
            let groups = ControlGroup::all()
                .map(|g| states.of(p).groups.get(g).iter().count())
                .sum();
            (states.selection(p).ids().collect(), groups)
        })
        .into();
    let drawn: Vec<ObjectId> = world.resource::<ObjectVisuals>().by_id.keys().copied().collect();
    let remembered: Vec<ObjectId> = world.resource::<RememberedVisuals>().by_id.keys().copied().collect();
    let minimap = world.resource::<MinimapView>().image.clone().expect("a minimap");
    let fog = world.resource::<FogAssets>().texture.clone();
    let shown = (
        sim(app).state_hash(),
        world.resource::<LocalPlayer>().0,
        world.resource::<CameraView>().center,
        chosen,
        drawn,
        remembered,
        images_data(app, &minimap),
        images_data(app, &fog),
    );
    app.world_mut().resource_mut::<SimControl>().paused = false;
    shown
}

/// Nothing of the old game stays in the client after a restart: player 1 selected their Deployment Center and made it
/// control group 3, player 2's view was switched to (F2), the camera moved, a debug command spawned a unit, and 40
/// ticks went by; restarted, the game shows at tick 5 exactly what a fresh launch shows at tick 5: player 1's view at
/// their start, no selection nor control group, the same objects drawn and remembered, the same fog and minimap.
#[test]
fn nothing_of_the_old_game_stays_after_a_restart() {
    let mut app = headless(AppConfig::new(scenarios::dev_skirmish()));
    run_until(&mut app, 2);
    let dc = own(&app, P1, ObjectKind::DeploymentCenter)[0];
    let selection = Selection::new([(dc, ObjectKind::DeploymentCenter)]);
    app.world_mut().resource_mut::<ControlStates>().of_mut(P1).selection = selection;
    input(&mut app).keys.press(KeyCode::ControlLeft);
    input(&mut app).tap_key(KeyCode::Digit3);
    app.update();
    input(&mut app).keys.release(KeyCode::ControlLeft);
    app.update();
    let group = ControlGroup::new(3).unwrap();
    assert_eq!(
        app.world()
            .resource::<ControlStates>()
            .of(P1)
            .groups
            .get(group)
            .to_vec(),
        [dc]
    );
    app.world_mut().resource_mut::<CameraView>().center = Vec2::new(2000.0, 1500.0);
    input(&mut app).tap_key(KeyCode::F2);
    app.update();
    assert_eq!(app.world().resource::<LocalPlayer>().0, P2);
    let spawn = sc_sim::DebugCommand::Spawn {
        kind: ObjectKind::Peacekeeper,
        owner: Some(P1),
        at: SpawnAt::Point(centre(10, 10)),
        orientation: Default::default(),
    };
    app.world_mut().resource_mut::<PendingCommands>().0.push(PlayerCommand {
        player: P1,
        command: Command::Debug(spawn),
    });
    run_until(&mut app, 40);

    restart(&mut app);
    run_until(&mut app, 5);
    let mut fresh = headless(AppConfig::new(scenarios::dev_skirmish()));
    run_until(&mut fresh, 5);
    assert_eq!(shown(&mut app), shown(&mut fresh));
}
