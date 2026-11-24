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

use std::path::PathBuf;

use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use sc_sim::testing::ScenarioBuilder;
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

/// Design change 0101's verification 3, the client's half: when player 2 loses their last building, player 1 sees
/// Victory and player 2 Defeat (the debug player switch shows each player's banner), the game stops at that tick, and
/// from then on the command panel gives no orders.
#[test]
fn the_winner_sees_victory_the_loser_defeat_and_the_game_stops() {
    let mut app = game_as(AppConfig::new(duel()), P1);
    run_until(&mut app, 3);
    assert_eq!(banner(&mut app), None);
    assert!(
        stop_peacekeeper(&mut app),
        "while the game goes on, X stops the Peacekeeper"
    );

    remove(&mut app, 2);
    let end = tick(&app);
    let sim = app.world().resource::<SimRes>();
    assert_eq!(
        sim.outcome().map(|o| (o.tick, o.winners.clone())),
        Some((end, vec![P1]))
    );
    assert_eq!(banner(&mut app), shown("VICTORY", "The game is over"));
    for _ in 0..20 {
        app.update();
    }
    assert_eq!(tick(&app), end, "the game stopped");
    assert!(!stop_peacekeeper(&mut app), "no more orders");

    app.world_mut().resource_mut::<LocalPlayer>().0 = P2;
    app.update();
    assert_eq!(banner(&mut app), shown("DEFEAT", "The game is over"));
    // The loser owns nothing: nothing shows Visible to them.
    let sim = app.world().resource::<SimRes>();
    let sight = Sight::new(sim, P2, true);
    assert!(sim.objects().all(|o| sight.object(o) != VisibilityState::Visible));
    assert_eq!(Banner::of(sim, P1), Some(Banner::Victory));
    assert_eq!(Banner::of(sim, PlayerId(9)), None, "not in the game");
}

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

/// Design change 0163's Verification: "Two players destroy each other's last building in the same tick: both see
/// 'Draw'; replay saved." The game stops at that tick, and nobody gives orders any more.
#[test]
fn players_defeated_in_the_same_tick_both_see_a_draw() {
    let path = std::env::temp_dir()
        .join(format!("sc_game_end_draw_test_{}", std::process::id()))
        .join("draw.json");
    let mut config = AppConfig::new(duel());
    config.record = Some(path.clone());
    let mut app = game_as(config, P1);
    run_until(&mut app, 3);
    remove_together(&mut app, &[1, 2]);
    let end = tick(&app);
    let outcome = app.world().resource::<SimRes>().outcome().cloned().unwrap();
    assert!(outcome.is_draw() && outcome.tick == end);
    assert_eq!(banner(&mut app), shown("DRAW", "The game is over"));
    assert_eq!(
        results_screen(&mut app),
        None,
        "defeated in the last tick: no results screen"
    );
    let recorder = app.world().resource::<Recorder>();
    assert!(
        recorder.is_finished() && recorder.saved_at() == Some(end),
        "replay saved"
    );
    let replay = load_replay(&path).unwrap();
    std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    assert_eq!(replay.run().outcome(), Some(&outcome));
    for _ in 0..20 {
        app.update();
    }
    assert_eq!(tick(&app), end, "the game stopped");
    assert!(!stop_peacekeeper(&mut app), "no more orders");

    view_as(&mut app, P2);
    assert_eq!(banner(&mut app), shown("DRAW", "The game is over"));
    let sim = app.world().resource::<SimRes>();
    assert_eq!([Banner::of(sim, P1), Banner::of(sim, P2)], [Some(Banner::Draw); 2]);
}

/// Design change 0164's Verification: "3-player FFA: the first defeated player is moved to a results screen; the
/// other two keep playing." Player 3 has no allies (no teams), so their defeat takes them to the results screen at
/// once: DEFEATED, their name and faction, when it happened, and that the game goes on. It covers the world view and
/// the HUD Bar, below the top bar, so they see nothing of the world, and no banner says Spectating. The game goes
/// on, and players 1 and 2 play as before: no screen, no banner, their commands given.
#[test]
fn a_player_defeated_with_no_ally_left_leaves_for_the_results_screen() {
    let mut app = game_as(AppConfig::new(three_way()), P3);
    run_until(&mut app, 3);
    assert_eq!(results_screen(&mut app), None);
    remove(&mut app, 3);
    let defeat = tick(&app);
    let sim = app.world().resource::<SimRes>();
    assert!(sim.player(P3).unwrap().is_defeated() && sim.allies(P3).is_empty());
    assert_eq!(sim.outcome(), None);
    run_until(&mut app, defeat + 20);
    assert_eq!(
        results_screen(&mut app),
        lines([
            "DEFEATED",
            "Player 3 (GDO)",
            &format!("Defeated at 0:00 (tick {defeat})"),
            "The game goes on for the other players",
        ])
    );
    assert_eq!(banner(&mut app), None, "not spectating");
    assert_eq!(Banner::of(app.world().resource::<SimRes>(), P3), None);
    // Everything below the top bar, opaque, over the HUD.
    let world = app.world_mut();
    let (node, background, z) = world
        .query_filtered::<(&Node, &BackgroundColor, &GlobalZIndex), With<ResultsScreen>>()
        .single(world)
        .unwrap();
    assert_eq!(
        (node.top, node.bottom, node.left, node.width),
        (vh(TOP_BAR_HEIGHT_VH), px(0), px(0), percent(100))
    );
    assert_eq!((background.0.alpha(), z.0), (1.0, RESULTS_Z));
    assert!(RESULTS_Z > 0 && BANNER_Z > RESULTS_Z);
    let hud_z: Vec<i32> = world
        .query_filtered::<Option<&GlobalZIndex>, With<HudRoot>>()
        .iter(world)
        .map(|z| z.map_or(0, |z| z.0))
        .collect();
    assert_eq!(hud_z, [0, 0], "the top bar and the HUD Bar lie under it");

    // The game goes on: players 1 and 2 still play, and see nothing of this.
    assert_eq!(tick(&app), defeat + 20);
    for player in [P1, P2] {
        view_as(&mut app, player);
        assert_eq!((results_screen(&mut app), banner(&mut app)), (None, None));
    }
    view_as(&mut app, P1);
    let selection = Selection::new([(ObjectId(4), PK)]);
    app.world_mut().resource_mut::<ControlStates>().of_mut(P1).selection = selection;
    app.world_mut().resource_mut::<PendingCommands>().0.clear();
    app.world_mut().resource_mut::<InjectedInput>().tap_key(KeyCode::KeyX);
    app.update();
    let pending = &app.world().resource::<PendingCommands>().0;
    assert!(
        pending
            .iter()
            .any(|c| c.player == P1 && matches!(c.command, Command::Order { .. })),
        "player 1 still gives orders"
    );
    run_until(&mut app, defeat + 40);
    assert_eq!(app.world().resource::<SimRes>().outcome(), None);
    // The debug player switch back to player 3 shows their screen again.
    view_as(&mut app, P3);
    assert!(results_screen(&mut app).is_some());
}

/// Design change 0163 with a player defeated before the end: player 3 leaves for the results screen; then players
/// 1 and 2 lose their last buildings in the same tick. The game is a draw, and all three see Draw: players 1 and 2
/// over the world, player 3 over their results screen, which now says the game is over.
#[test]
fn a_player_defeated_earlier_sees_the_draw_on_the_results_screen() {
    let mut app = game_as(AppConfig::new(three_way()), P3);
    run_until(&mut app, 3);
    remove(&mut app, 3);
    let defeat = tick(&app);
    run_until(&mut app, 10);
    assert_eq!(banner(&mut app), None);
    remove_together(&mut app, &[1, 2]);
    let end = tick(&app);
    let sim = app.world().resource::<SimRes>();
    assert_eq!(sim.outcome().map(|o| (o.tick, o.is_draw())), Some((end, true)));
    assert_eq!(banner(&mut app), shown("DRAW", "The game is over"));
    assert_eq!(
        results_screen(&mut app),
        lines([
            "DEFEATED",
            "Player 3 (GDO)",
            &format!("Defeated at 0:00 (tick {defeat})"),
            "The game is over",
        ])
    );
    for player in [P1, P2] {
        view_as(&mut app, player);
        assert_eq!(banner(&mut app), shown("DRAW", "The game is over"));
        assert_eq!(results_screen(&mut app), None, "player {player:?} was there at the end");
    }
}

/// Design change 0164 in a replay: played back as player 3, the playback shows their results screen from their
/// defeat on, as the game did; the debug player switch shows the others' views, the game going on in them.
#[test]
fn a_replay_shows_the_results_screen_of_a_player_who_left() {
    let mut scenario = three_way();
    scenario.setup.config = SimConfig::DEBUG;
    let mut replay = Replay::new(scenario);
    let mut sim = replay.start();
    replay.record_checked(None, &sim);
    while sim.tick() < 40 {
        let mut inputs = TickInputs::default();
        if sim.tick() + 1 == 10 {
            inputs.commands.push(PlayerCommand {
                player: P1,
                command: Command::Debug(DebugCommand::Remove { id: ObjectId(3) }),
            });
        }
        sim.step(&inputs);
        replay.record_checked(Some(inputs), &sim);
    }
    replay.checkpoint(&sim);

    let mut app = game_as(AppConfig::playback(replay, PathBuf::from("left.json")), P3);
    run_until(&mut app, 9);
    assert_eq!(results_screen(&mut app), None);
    run_until(&mut app, 12);
    assert_eq!(
        results_screen(&mut app),
        lines([
            "DEFEATED",
            "Player 3 (GDO)",
            "Defeated at 0:00 (tick 10)",
            "The game goes on for the other players",
        ])
    );
    view_as(&mut app, P1);
    assert_eq!((results_screen(&mut app), banner(&mut app)), (None, None));
}

/// Design change 0102's mechanism, through `Sight::watching` (the sim has no teams to make player 2 an ally of player
/// 3): the union of player 3's vision and player 2's shows what player 2 sees, live, and what either remembers; the
/// rest stays hidden.
#[test]
fn a_spectator_watches_through_the_union_of_the_visions() {
    let mut app = game_as(AppConfig::new(three_way()), P3);
    run_until(&mut app, 3);
    remove(&mut app, 3);
    run_until(&mut app, 8);
    let sim = app.world().resource::<SimRes>();
    let alone = Sight::new(sim, P3, true);
    let union = Sight::watching(sim, P3, &[P3, P2], true);
    let object = |id: u32| sim.object(ObjectId(id)).unwrap();

    // Player 2's Peacekeeper and the patch beside it: live in the union; unseen, or remembered, alone.
    assert!(union.sees(object(5)) && !alone.sees(object(5)));
    assert!(union.sees(object(7)) && union.remembered_record(ObjectId(7)).is_none());
    assert!(!alone.sees(object(7)) && alone.remembered_record(ObjectId(7)).is_some());
    // The patch only player 3 saw: remembered either way.
    assert!(!union.sees(object(8)) && union.remembered_record(ObjectId(8)).is_some());
    let remembered: Vec<u32> = union.remembered().map(|r| r.id.0).collect();
    assert_eq!(remembered, [8]);
    // Player 1's Peacekeeper, seen by neither: hidden.
    assert!(!union.sees(object(4)) && !alone.sees(object(4)));

    // The tiles: the best state of the two visions.
    assert_eq!(union.tile(TilePos::new(24, 8)), VisibilityState::Visible);
    assert_eq!(union.tile(TilePos::new(18, 12)), VisibilityState::Explored);
    assert_eq!(union.tile(TilePos::new(8, 8)), VisibilityState::Unexplored);
    let (w, h) = (sim.map().width(), sim.map().height());
    let alpha = |sight: &Sight, x: u32, y: u32| fog_texels(sight, w, h)[((y * w + x) * 4 + 3) as usize];
    let expected = |a: f32| (a * 255.0).round() as u8;
    assert_eq!(alpha(&union, 24, 8), 0);
    assert_eq!(alpha(&union, 18, 12), expected(EXPLORED_ALPHA));
    assert_eq!(alpha(&union, 8, 8), expected(UNEXPLORED_ALPHA));
    assert_eq!(alpha(&alone, 24, 8), expected(UNEXPLORED_ALPHA));
    // Whom a sight watches is part of its revision, so the fog layer redraws when it changes.
    assert_ne!(union.revision(), alone.revision());
}
