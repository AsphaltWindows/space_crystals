//! A Colonists start in the client (slice M3-C), headless, one tick per update: `--factions` picks each slot's
//! faction (plan decision T24); the camera starts on the Colonists' Robo Portal (`core/camera.md > Starting
//! Position`); the resource bar shows the faction's DisplayHud, and the debug player switch (F2) the other player's
//! (`factions/colonists/overview.md > DisplayHud`, plan decision T23); a Train refused for the army cap names Beacon
//! Capacity (T20); a recorded game keeps its factions, and its playback reproduces them.

use std::path::{Path, PathBuf};

use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use sc_sim::testing::ScenarioBuilder;
use sc_sim::{
    Command, DebugCommand, Faction, ObjectId, ObjectKind, PlayerCommand, PlayerId, ProductionCommand, SU_PER_GU,
    SpawnAt, Tick, TilePos,
};
use space_crystals::camera::{
    CameraView, CameraZoom, ViewportInsets, clamp_center, start_center, start_view, viewport_rect, visible_extent_su,
};
use space_crystals::cli;
use space_crystals::command_ui::feedback::{BEACON_CAPACITY_TEXT, TargetFeedback};
use space_crystals::coords::drawn_bounds;
use space_crystals::hud::resources::ResourceFigure;
use space_crystals::input::InjectedInput;
use space_crystals::platform::{DisplayMode, SurfaceSize};
use space_crystals::replay::{Playback, load_replay};
use space_crystals::sim_driver::{LoadedScenario, LocalPlayer, PendingCommands, SimControl, SimRes};
use space_crystals::{AppConfig, build_app};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const COLONISTS_BAR: [&str; 5] = [
    "Space Crystals 100",
    "Alloys 0",
    "Essence 0",
    "Flux 0",
    "Beacon Capacity 4/10",
];
const GDO_BAR: [&str; 4] = ["Space Crystals 100", "Supplies 0", "Power 21/30", "Unit Control 0/200"];

fn headless(mut config: AppConfig) -> App {
    config.display = DisplayMode::Headless;
    let mut app = build_app(config);
    app.insert_resource(TimeUpdateStrategy::FixedTimesteps(1));
    app.finish();
    app.cleanup();
    app.update();
    app
}

/// The game the flags `flags` set up, as `cargo run` would start it.
fn game(flags: &str) -> App {
    let args: Vec<String> = flags.split_whitespace().map(str::to_string).collect();
    headless(cli::into_config(cli::parse(&args).unwrap()).unwrap())
}

fn tick(app: &App) -> Tick {
    app.world().resource::<SimRes>().tick()
}

fn hash(app: &App) -> u64 {
    app.world().resource::<SimRes>().state_hash()
}

fn run_until(app: &mut App, end: Tick) {
    while tick(app) < end {
        app.update();
    }
}

fn tap(app: &mut App, key: KeyCode) {
    app.world_mut().resource_mut::<InjectedInput>().keys.press(key);
    app.update();
    app.world_mut().resource_mut::<InjectedInput>().keys.release(key);
    app.update();
}

/// The figures the resource bar shows, left to right.
fn bar(app: &mut App) -> Vec<String> {
    let mut query = app.world_mut().query::<(&ResourceFigure, &Text, &Node)>();
    let mut shown: Vec<(ResourceFigure, String)> = query
        .iter(app.world())
        .filter(|(.., node)| node.display != Display::None)
        .map(|(figure, text, _)| (*figure, text.0.clone()))
        .collect();
    shown.sort();
    shown.into_iter().map(|(_, text)| text).collect()
}

fn factions(app: &App) -> Vec<Faction> {
    let sim = app.world().resource::<SimRes>();
    sim.players().iter().map(|p| p.faction).collect()
}

fn train_drone(producers: &[u32]) -> PlayerCommand {
    PlayerCommand {
        player: P1,
        command: Command::Production {
            objects: producers.iter().map(|&id| ObjectId(id)).collect(),
            command: ProductionCommand::Train(ObjectKind::Drone),
        },
    }
}

/// The file `name` in a fresh directory of test `test`'s own.
fn temp_file(test: &str, name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("sc_factions_test_{test}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir.join(name)
}

/// The plan's acceptance for slice M3-C, start and HUD: `cargo run -- --factions colonists,gdo` starts the Colonists
/// in slot 1 of `dev_skirmish`; the camera starts on their Robo Portal, on tiles (6, 6) to (9, 9), and the bar reads
/// the Colonists' figures, in ASCII. F2 shows player 2's, the GDO's, unchanged, and back.
#[test]
fn a_colonists_game_starts_on_the_robo_portal_with_their_bar() {
    let mut app = game("--factions colonists,gdo --no-record");
    assert_eq!(factions(&app), [Faction::Colonists, Faction::Gdo]);
    let portal = TilePos::new(6, 6).corner() + Vec2::splat(2.0 * SU_PER_GU);
    assert_eq!(camera_start(&app, P1), portal);
    run_until(&mut app, 1);
    let shown = bar(&mut app);
    assert_eq!(shown, COLONISTS_BAR);
    assert!(shown.iter().all(|text| text.is_ascii()));
    tap(&mut app, KeyCode::F2);
    assert_eq!(app.world().resource::<LocalPlayer>().0, P2);
    assert_eq!(bar(&mut app), GDO_BAR);
    tap(&mut app, KeyCode::F2);
    assert_eq!(bar(&mut app), COLONISTS_BAR);

    // The GDO in slot 1, the Colonists in slot 2: their camera starts on their portal there, the image of slot 1's
    // through the center of the 64 × 48 map.
    let mut app = game("--scenario dev_skirmish --factions gdo,colonists --player 2 --no-record");
    assert_eq!(factions(&app), [Faction::Gdo, Faction::Colonists]);
    assert_eq!(camera_start(&app, P2), Vec2::new(64.0, 48.0) * SU_PER_GU - portal);
    run_until(&mut app, 1);
    assert_eq!(bar(&mut app), COLONISTS_BAR);
}

/// Where the camera of `player`, the local player, starts: the ground point [`start_center`] picks, which the view
/// centers on ([`start_view`]) as far as it stays on the map ([`clamp_center`]; a corner start is near its edges).
fn camera_start(app: &App, player: PlayerId) -> Vec2 {
    let world = app.world();
    let scenario = &world.resource::<LoadedScenario>().0;
    let map = world.resource::<SimRes>().map();
    assert_eq!(world.resource::<LocalPlayer>().0, player);
    let (size, insets) = (*world.resource::<SurfaceSize>(), *world.resource::<ViewportInsets>());
    let visible = visible_extent_su(
        viewport_rect(size, insets).1,
        size.physical.x,
        world.resource::<CameraZoom>().0,
    );
    let shown = clamp_center(start_view(scenario, player, map), visible, drawn_bounds(map));
    assert_eq!(world.resource::<CameraView>().center, shown);
    start_center(scenario, player, map)
}

/// Plan decision T20, the faction's cap named in the feedback after the player's, as design change 0143 has it: two
/// Robo Portals provide 10 Beacon Capacity, and nine Drones take 9. The first portal's Drone starts in the last room;
/// of a Train to both after it, both queue, and the idle portal waits for room, which the band names as the wait
/// begins, "Beacon Capacity limit reached" (placeholder for 0143's notification).
#[test]
fn a_waiting_portal_names_the_beacon_capacity() {
    let mut builder = ScenarioBuilder::new(&vec![".".repeat(24); 16].join("\n"))
        .faction(P1, Faction::Colonists)
        .spawn(ObjectKind::RoboPortal, Some(P1), SpawnAt::Tile(TilePos::new(1, 1)))
        .spawn(ObjectKind::RoboPortal, Some(P1), SpawnAt::Tile(TilePos::new(8, 1)));
    for x in 1..10 {
        let at = TilePos::new(x, 12).center();
        builder = builder.spawn(ObjectKind::Drone, Some(P1), SpawnAt::Point(at));
    }
    let mut app = headless(AppConfig::new(builder.build()));
    let set = DebugCommand::SetResources {
        player: P1,
        crystals: 5000,
        supplies: 0,
        alloys: 0,
        essence: 0,
        flux: 0,
    };
    let pending = |app: &mut App, commands: Vec<PlayerCommand>| {
        app.world_mut().resource_mut::<PendingCommands>().0.extend(commands);
        app.update();
    };
    let debug = PlayerCommand {
        player: P1,
        command: Command::Debug(set),
    };
    pending(&mut app, vec![debug, train_drone(&[1])]);
    assert_eq!(app.world().resource::<TargetFeedback>().since, None);
    pending(&mut app, vec![train_drone(&[1, 2])]);
    let feedback = app.world().resource::<TargetFeedback>();
    assert_eq!((feedback.since.is_some(), feedback.text), (true, BEACON_CAPACITY_TEXT));
    let cap = app.world().resource::<SimRes>().army_cap(P1).unwrap();
    assert_eq!((cap.used, cap.available), (10, 10));
}

/// The plan's acceptance for slice M3-C, factions: a game set up with `--factions` is recorded with its players, and
/// `--replay` plays it back with them, tick for tick, without a desync; the playback's bar is the Colonists'.
#[test]
fn a_recorded_colonists_game_plays_back_with_its_factions() {
    let path = temp_file("record", "colonists.json");
    let mut app = game(&format!("--factions colonists,gdo --record {}", path.display()));
    let portal = {
        let sim = app.world().resource::<SimRes>();
        let found = sim
            .objects()
            .find(|o| o.owner == Some(P1) && o.kind == ObjectKind::RoboPortal);
        found.unwrap().id.0
    };
    app.world_mut()
        .resource_mut::<PendingCommands>()
        .0
        .push(train_drone(&[portal]));
    run_until(&mut app, 200);
    app.world_mut().write_message(AppExit::Success);
    app.update();
    let (end, last) = (tick(&app), hash(&app));
    let replay = load_replay(&path).unwrap();
    let recorded: Vec<Faction> = replay.scenario.setup.players.iter().map(|p| p.faction).collect();
    assert_eq!(recorded, [Faction::Colonists, Faction::Gdo]);
    assert_eq!(replay.end_tick(), end);

    let mut playback = game(&format!("--replay {}", path.display()));
    remove_dir(&path);
    assert_eq!(factions(&playback), [Faction::Colonists, Faction::Gdo]);
    while !playback.world().resource::<SimControl>().paused {
        playback.update();
    }
    assert_eq!((tick(&playback), hash(&playback)), (end, last));
    assert_eq!(playback.world().resource::<Playback>().desync, None);
    let drones = {
        let sim = playback.world().resource::<SimRes>();
        let drones = sim
            .objects()
            .filter(|o| o.owner == Some(P1) && o.kind == ObjectKind::Drone);
        drones.count()
    };
    assert_eq!(drones, 5, "the Drone trained in the recorded game came out");
    assert_eq!(bar(&mut playback)[4], "Beacon Capacity 5/10");
}

fn remove_dir(path: &Path) {
    std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
}
