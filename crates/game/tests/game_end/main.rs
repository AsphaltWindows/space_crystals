//! The end of the game in the client (slice M2-F2; `core/game_rules.md > End of Game`, design changes 0101, 0102,
//! 0163 and 0164), headless, one tick per update (`ARCHITECTURE.md > Client architecture > Client tests`):
//! - the winner sees Victory and the loser Defeat, the game stops at its last tick, and nobody gives orders any more;
//! - when the last sides fall in the same tick, everyone sees Draw (0163), the players defeated before it too;
//! - the recording is saved at the end, complete, a QA run's later stops come down to the last tick, and a
//!   playback pauses there;
//! - a player defeated while the game goes on, with no ally left (every player of these games: the sim has no
//!   teams), leaves for the results screen and sees nothing of the world, while the others play on (0164); the debug
//!   player switch and a replay show each player's view;
//! - a defeated player whose ally plays on would watch through the union of their own vision and their allies'
//!   (0102): tested through `Sight::watching`, and the rule of who leaves with `hud::results::left_the_game`'s unit
//!   tests.
//!
//! One test binary: `outcome` (the banners, Victory, Defeat and Draw, and the stop), `recording` (the recording
//! saved, a QA run's stop, a playback's pause) and `results` (the results screen, in a game and in a replay, and
//! the union of visions).

mod outcome;
mod recording;
mod results;

use std::path::PathBuf;

use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use sc_sim::testing::ScenarioBuilder;
use sc_sim::vision::FOG_DELAY_TICKS;
use sc_sim::{
    Command, DebugCommand, GameSetup, ObjectId, ObjectKind, OrderKind, PlayerCommand, PlayerId, Replay, Scenario,
    SimConfig, Slot, SpawnAt, Tick, TickInputs, TilePos, VisibilityState,
};
use space_crystals::fog::{EXPLORED_ALPHA, Sight, UNEXPLORED_ALPHA, fog_texels};
use space_crystals::hud::outcome::{BANNER_Z, Banner, BannerText, OutcomeBanner};
use space_crystals::hud::results::{RESULTS_Z, ResultsScreen, ResultsText};
use space_crystals::hud::{HudRoot, TOP_BAR_HEIGHT_VH};
use space_crystals::input::InjectedInput;
use space_crystals::platform::DisplayMode;
use space_crystals::qa::QaConfig;
use space_crystals::replay::{Playback, Recorder, load_replay};
use space_crystals::selection::{ControlStates, Selection};
use space_crystals::sim_driver::{LocalPlayer, PendingCommands, SimControl, SimRes};
use space_crystals::{AppConfig, build_app};

const P1: PlayerId = PlayerId(1);

const P2: PlayerId = PlayerId(2);

const P3: PlayerId = PlayerId(3);

const PK: ObjectKind = ObjectKind::Peacekeeper;

fn tile(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Tile(TilePos::new(x, y))
}

fn centre(x: i32, y: i32) -> Vec2 {
    TilePos::new(x, y).center()
}

/// A flat 24 × 12 map: player 1's Generator 1 and Peacekeeper 3 in the west, player 2's Generator 2 and Peacekeeper
/// 4 in the east, out of each other's sight.
fn duel() -> Scenario {
    ScenarioBuilder::new(&vec![".".repeat(24); 12].join("\n"))
        .spawn(ObjectKind::Generator, Some(P1), tile(1, 1))
        .spawn(ObjectKind::Generator, Some(P2), tile(20, 1))
        .spawn(PK, Some(P1), SpawnAt::Point(centre(4, 7)))
        .spawn(PK, Some(P2), SpawnAt::Point(centre(19, 7)))
        .build()
}

/// A three-player game on a flat 32 × 16 map:
/// 1. to 3. each player's Generator, at (1, 1), (28, 1) and (1, 13);
/// 4. player 1's Peacekeeper, at the centre of (8, 8);
/// 5. player 2's, at (24, 8);
/// 6. player 3's, at (18, 12);
/// 7. a crystal patch at (20, 10), in sight of 5 and 6;
/// 8. one at (15, 13), in sight of 6 only.
fn three_way() -> Scenario {
    let mut scenario = ScenarioBuilder::new(&vec![".".repeat(32); 16].join("\n"))
        .spawn(ObjectKind::Generator, Some(P1), tile(1, 1))
        .spawn(ObjectKind::Generator, Some(P2), tile(28, 1))
        .spawn(ObjectKind::Generator, Some(P3), tile(1, 13))
        .spawn(PK, Some(P1), SpawnAt::Point(centre(8, 8)))
        .spawn(PK, Some(P2), SpawnAt::Point(centre(24, 8)))
        .spawn(PK, Some(P3), SpawnAt::Point(centre(18, 12)))
        .spawn(ObjectKind::CrystalPatch, None, tile(20, 10))
        .spawn(ObjectKind::CrystalPatch, None, tile(15, 13))
        .build();
    scenario.map.slots.push(Slot::new(TilePos::new(16, 4)));
    scenario.setup = GameSetup::default_for(&scenario.map);
    scenario
}

/// A headless game as `player`, with the fog of war, at tick 1.
fn game_as(mut config: AppConfig, player: PlayerId) -> App {
    config.display = DisplayMode::Headless;
    config.player = player;
    let mut app = build_app(config);
    app.insert_resource(TimeUpdateStrategy::FixedTimesteps(1));
    app.finish();
    app.cleanup();
    app.update();
    run_until(&mut app, 1);
    app
}

fn tick(app: &App) -> Tick {
    app.world().resource::<SimRes>().tick()
}

fn run_until(app: &mut App, end: Tick) {
    while tick(app) < end {
        app.update();
    }
}

/// Removes object `id` by player 1's debug command, and runs the tick that applies it.
fn remove(app: &mut App, id: u32) {
    app.world_mut().resource_mut::<PendingCommands>().0.push(PlayerCommand {
        player: P1,
        command: Command::Debug(DebugCommand::Remove { id: ObjectId(id) }),
    });
    let next = tick(app) + 1;
    run_until(app, next);
}

/// The banner's title and line while it shows.
fn banner(app: &mut App) -> Option<(String, String)> {
    let world = app.world_mut();
    let shown = world
        .query_filtered::<&Visibility, With<OutcomeBanner>>()
        .single(world)
        .is_ok_and(|v| *v != Visibility::Hidden);
    let texts: Vec<(BannerText, String)> = world
        .query::<(&BannerText, &Text)>()
        .iter(world)
        .map(|(part, text)| (*part, text.0.clone()))
        .collect();
    let text = |part| texts.iter().find(|(p, _)| *p == part).map(|(_, text)| text.clone());
    let (title, line) = (text(BannerText::Title)?, text(BannerText::Line)?);
    shown.then_some((title, line))
}

fn shown(title: &str, line: &str) -> Option<(String, String)> {
    Some((title.to_string(), line.to_string()))
}

/// The results screen's texts while it shows, top to bottom: the title, the player, their defeat and the game's
/// state.
fn results_screen(app: &mut App) -> Option<[String; 4]> {
    let world = app.world_mut();
    let shown = world
        .query_filtered::<&Visibility, With<ResultsScreen>>()
        .single(world)
        .is_ok_and(|v| *v != Visibility::Hidden);
    let texts: Vec<(ResultsText, String)> = world
        .query::<(&ResultsText, &Text)>()
        .iter(world)
        .map(|(part, text)| (*part, text.0.clone()))
        .collect();
    let text = |part| texts.iter().find(|(p, _)| *p == part).map(|(_, text)| text.clone());
    let parts = [
        ResultsText::Title,
        ResultsText::Player,
        ResultsText::Defeat,
        ResultsText::Status,
    ];
    shown.then(|| parts.map(|part| text(part).unwrap_or_default()))
}

fn lines(lines: [&str; 4]) -> Option<[String; 4]> {
    Some(lines.map(str::to_string))
}

/// Removes objects `ids` by player 1's debug commands, all in the next tick, and runs it.
fn remove_together(app: &mut App, ids: &[u32]) {
    for &id in ids {
        app.world_mut().resource_mut::<PendingCommands>().0.push(PlayerCommand {
            player: P1,
            command: Command::Debug(DebugCommand::Remove { id: ObjectId(id) }),
        });
    }
    let next = tick(app) + 1;
    run_until(app, next);
}

/// Switches the local player, as the debug player switch (F2) does, and updates once.
fn view_as(app: &mut App, player: PlayerId) {
    app.world_mut().resource_mut::<LocalPlayer>().0 = player;
    app.update();
}

/// Selects player 1's Peacekeeper 3 and presses X, the command panel's Stop: whether that frame queued a Stop for
/// the sim.
fn stop_peacekeeper(app: &mut App) -> bool {
    let selection = Selection::new([(ObjectId(3), PK)]);
    app.world_mut().resource_mut::<ControlStates>().of_mut(P1).selection = selection;
    app.world_mut().resource_mut::<PendingCommands>().0.clear();
    app.world_mut().resource_mut::<InjectedInput>().tap_key(KeyCode::KeyX);
    app.update();
    let pending = &app.world().resource::<PendingCommands>().0;
    pending.iter().any(|c| {
        c.player == P1
            && matches!(
                c.command,
                Command::Order {
                    kind: OrderKind::Stop,
                    ..
                }
            )
    })
}
