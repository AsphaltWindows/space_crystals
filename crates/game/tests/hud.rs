//! The HUD of slice M2-G, headless (`ARCHITECTURE.md > Client architecture > Client tests`): the resource bar in the
//! top bar (`factions/gdo/overview.md > DisplayHud`; design changes 0077, 0090 and 0143), its note while the fog of
//! war is off (slice M5-S6) and the structures' InfoPanels (design changes 0099, 0105 and 0155), with the fog of war
//! on. Each `App::update` is one tick; clicks and keys come through the virtual mouse and keyboard (`InjectedInput`).

use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use sc_sim::testing::ScenarioBuilder;
use sc_sim::{Command, DebugCommand, ObjectKind, PlayerCommand, PlayerId, Scenario, SpawnAt, Tick, TilePos};
use space_crystals::camera::{CameraView, ViewportInsets, viewport_rect, visible_extent_su};
use space_crystals::fog::FogOfWar;
use space_crystals::hud::TopBarNote;
use space_crystals::hud::info_panel::InfoText;
use space_crystals::hud::resources::{ALERT_COLOR, ResourceFigure};
use space_crystals::input::InjectedInput;
use space_crystals::platform::{DisplayMode, SurfaceSize};
use space_crystals::selection::ControlStates;
use space_crystals::sim_driver::{LocalPlayer, PendingCommands, SimRes};
use space_crystals::{AppConfig, build_app};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

// The objects of [`scenario`], by id.
/// Player 1's Deployment Center and plates (on patches 1 and 2).
const DC: u32 = 4;
const PLATE: u32 = 6;
/// Player 2's Generator and plate (on patch 3), in the sight of player 1's Deployment Center.
const ENEMY_GENERATOR: u32 = 8;
const ENEMY_PLATE: u32 = 9;

fn tile(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Tile(TilePos::new(x, y))
}

/// The centre of tile `(x, y)` (the map is flat, so it is drawn there too).
fn tile_center(x: i32, y: i32) -> Vec2 {
    TilePos::new(x, y).center()
}

/// A flat 24 × 12 map: patches 1–3; player 1's starting base (`factions/gdo/economy.md > Starting Conditions`): the
/// Deployment Center 4, the Extraction Facility 5 and plates 6 and 7 on patches 1 and 2; player 2's Generator 8 and
/// plate 9 on patch 3, close enough for the Deployment Center to see them.
fn scenario() -> Scenario {
    use ObjectKind::{CrystalPatch, DeploymentCenter, ExtractionFacility, ExtractionPlate, Generator};
    ScenarioBuilder::new(&vec![".".repeat(24); 12].join("\n"))
        .spawn(CrystalPatch, None, tile(1, 8))
        .spawn(CrystalPatch, None, tile(3, 8))
        .spawn(CrystalPatch, None, tile(5, 7))
        .spawn(DeploymentCenter, Some(P1), tile(1, 1)) // (1, 1) to (4, 4)
        .spawn(ExtractionFacility, Some(P1), tile(6, 1)) // (6, 1) to (8, 3)
        .spawn(ExtractionPlate, Some(P1), tile(1, 8))
        .spawn(ExtractionPlate, Some(P1), tile(3, 8))
        .spawn(Generator, Some(P2), tile(6, 5)) // (6, 5) to (7, 6)
        .spawn(ExtractionPlate, Some(P2), tile(5, 7))
        .build()
}

/// A headless game of `scenario` as `player`, with the fog of war, the camera over the middle of the map, at tick 1.
fn game_as(scenario: Scenario, player: PlayerId) -> App {
    game_of(scenario, player, false)
}

/// [`game_as`], with the fog of war off from the start if `no_fog` (`--no-fog`).
fn game_of(scenario: Scenario, player: PlayerId, no_fog: bool) -> App {
    let mut config = AppConfig::new(scenario);
    config.display = DisplayMode::Headless;
    config.player = player;
    config.qa.no_fog = no_fog;
    let mut app = build_app(config);
    app.insert_resource(TimeUpdateStrategy::FixedTimesteps(1));
    app.finish();
    app.cleanup();
    app.update();
    app.world_mut().resource_mut::<CameraView>().center = Vec2::new(768.0, 384.0);
    run_until(&mut app, 1);
    app
}

fn game() -> App {
    game_as(scenario(), P1)
}

fn tick(app: &App) -> Tick {
    app.world().resource::<SimRes>().tick()
}

fn run_until(app: &mut App, end: Tick) {
    while tick(app) < end {
        app.update();
    }
}

/// `command` from player 1, applied by the next tick.
fn debug(app: &mut App, command: DebugCommand) {
    app.world_mut().resource_mut::<PendingCommands>().0.push(PlayerCommand {
        player: P1,
        command: Command::Debug(command),
    });
    app.update();
}

fn spawn(app: &mut App, kind: ObjectKind, owner: PlayerId, at: SpawnAt) {
    debug(
        app,
        DebugCommand::Spawn {
            kind,
            owner: Some(owner),
            at,
            orientation: Default::default(),
        },
    );
}

/// The figures the resource bar shows, left to right, each with its text, a warning marked with `!`.
fn bar(app: &mut App) -> Vec<String> {
    let mut query = app.world_mut().query::<(&ResourceFigure, &Text, &TextColor, &Node)>();
    let mut shown: Vec<(ResourceFigure, String)> = query
        .iter(app.world())
        .filter(|(.., node)| node.display != Display::None)
        .map(|(figure, text, color, _)| {
            let alert = if color.0 == ALERT_COLOR { "!" } else { "" };
            (*figure, format!("{}{alert}", text.0))
        })
        .collect();
    shown.sort();
    shown.into_iter().map(|(_, text)| text).collect()
}

/// What the top bar's notes say, those it shows ([`TopBarNote`]).
pub(crate) fn notes(app: &mut App) -> Vec<String> {
    let mut notes = app.world_mut().query::<(&TopBarNote, &Node, &Children)>();
    let shown: Vec<Entity> = (notes.iter(app.world()))
        .filter(|(_, node, _)| node.display != Display::None)
        .flat_map(|(.., children)| children.iter())
        .collect();
    (shown.into_iter())
        .filter_map(|text| app.world().get::<Text>(text).map(|text| text.0.clone()))
        .collect()
}

/// A tap of `key` on the virtual keyboard: down in one frame, up in the next.
fn tap(app: &mut App, key: KeyCode) {
    app.world_mut().resource_mut::<InjectedInput>().tap_key(key);
    app.update();
    app.update();
}

/// The InfoPanel's title, its lines and its activity area, those it shows.
fn info(app: &mut App) -> (Option<String>, Option<String>, Option<String>) {
    let mut texts = app.world_mut().query::<(&InfoText, &Text, &Node)>();
    let (mut title, mut body, mut activity) = (None, None, None);
    for (which, text, node) in texts.iter(app.world()) {
        if node.display == Display::None {
            continue;
        }
        let slot = match which {
            InfoText::Title => &mut title,
            InfoText::Body => &mut body,
            InfoText::Activity => &mut activity,
        };
        *slot = Some(text.0.clone());
    }
    (title, body, activity)
}

/// The panel of one object: its title and lines, and no activity area.
fn panel(title: &str, body: &str) -> (Option<String>, Option<String>, Option<String>) {
    (Some(title.into()), Some(body.into()), None)
}

/// A left click at the view point `view`: the button goes down in one frame and up in the next, then half a second
/// passes, so that no later click makes a double-click.
fn click(app: &mut App, view: Vec2) {
    let size = *app.world().resource::<SurfaceSize>();
    let insets = *app.world().resource::<ViewportInsets>();
    let (position, viewport) = viewport_rect(size, insets);
    let su_per_pixel = visible_extent_su(viewport, size.physical.x, 1.0).x / viewport.x as f32;
    let center = app.world().resource::<CameraView>().center;
    let screen = ((view - center) / su_per_pixel + position.as_vec2() + viewport.as_vec2() / 2.0) / size.scale_factor;
    let mut input = app.world_mut().resource_mut::<InjectedInput>();
    input.cursor = Some(screen);
    input.mouse.press(MouseButton::Left);
    app.update();
    app.world_mut()
        .resource_mut::<InjectedInput>()
        .mouse
        .release(MouseButton::Left);
    for _ in 0..9 {
        app.update();
    }
}

fn selected(app: &App) -> Vec<u32> {
    let player = app.world().resource::<LocalPlayer>().0;
    let states = app.world().resource::<ControlStates>();
    states.selection(player).ids().map(|id| id.0).collect()
}

/// `factions/gdo/overview.md > DisplayHud` at the start of a game (design change 0090's first verification: "Power
/// 21/30"; Supplies 1, `factions/gdo/economy.md > Starting Conditions`), and the crystals
/// after the plates' first payout, at tick 48 (the M2 plan's acceptance for slice G), starting from the Starting
/// Conditions' 100 Space Crystals (slice M2-F1).
#[test]
fn the_bar_shows_the_gdo_resources_and_the_first_payout() {
    let mut app = game();
    assert_eq!(
        bar(&mut app),
        ["Space Crystals 100", "Supplies 1", "Power 21/30", "Unit Control 0/200"]
    );
    run_until(&mut app, 47);
    assert_eq!(bar(&mut app)[0], "Space Crystals 100");
    run_until(&mut app, 48);
    assert_eq!(
        bar(&mut app),
        ["Space Crystals 120", "Supplies 1", "Power 21/30", "Unit Control 0/200"],
        "two plates paid 10 each"
    );
}

/// Design change 0090's second verification: a Barracks before any Generator makes it 51/30, in red. One Generator
/// isn't enough (51/50), two take it out of the red, and losing one puts it back.
#[test]
fn power_is_consumed_over_generated_and_red_while_short() {
    let mut app = game();
    spawn(&mut app, ObjectKind::Barracks, P1, tile(10, 1));
    assert_eq!(bar(&mut app)[2], "Power 51/30!");
    spawn(&mut app, ObjectKind::Generator, P1, tile(14, 1));
    assert_eq!(bar(&mut app)[2], "Power 51/50!");
    spawn(&mut app, ObjectKind::Generator, P1, tile(17, 1));
    assert_eq!(bar(&mut app)[2], "Power 51/70");
    let generator = app
        .world()
        .resource::<SimRes>()
        .objects()
        .find(|o| o.kind == ObjectKind::Generator && o.owner == Some(P1))
        .map(|o| o.id)
        .unwrap();
    debug(&mut app, DebugCommand::Remove { id: generator });
    assert_eq!(bar(&mut app)[2], "Power 51/50!");
}

/// `> Unit Control: current used / 200`, through the sim's one accessor (which counts the units in production too,
/// design change 0143; the HUD's waiting Barracks: `tests/commands/production.rs`): the player's units, and a unit
/// gone gives its Unit Control back.
#[test]
fn unit_control_counts_the_players_units() {
    let mut app = game();
    for x in [1100.0, 1150.0] {
        spawn(
            &mut app,
            ObjectKind::Peacekeeper,
            P1,
            SpawnAt::Point(Vec2::new(x, 650.0)),
        );
    }
    spawn(
        &mut app,
        ObjectKind::Peacekeeper,
        P2,
        SpawnAt::Point(Vec2::new(1450.0, 100.0)),
    );
    assert_eq!(bar(&mut app)[3], "Unit Control 2/200");
    let first = app
        .world()
        .resource::<SimRes>()
        .objects()
        .find(|o| o.kind == ObjectKind::Peacekeeper);
    let id = first.map(|o| o.id).unwrap();
    debug(&mut app, DebugCommand::Remove { id });
    assert_eq!(bar(&mut app)[3], "Unit Control 1/200");
}

/// The bar is the local player's: the debug player switch (F2) shows player 2's, and a local player who isn't in the
/// game has none.
#[test]
fn the_bar_follows_the_local_player() {
    let mut app = game();
    app.world_mut().resource_mut::<InjectedInput>().tap_key(KeyCode::F2);
    app.update();
    assert_eq!(app.world().resource::<LocalPlayer>().0, P2);
    assert_eq!(
        bar(&mut app),
        ["Space Crystals 100", "Supplies 1", "Power 3/20", "Unit Control 0/200"]
    );
    let mut nobody = game_as(scenario(), PlayerId(3));
    assert!(bar(&mut nobody).is_empty());
}

/// Design change 0105: the local player's own structure shows its name and HP, and no activity while it has none to
/// show (the Deployment Center's construction lines come with slice M2-H2); another player's its name, owner and HP.
/// Ids by clicks, as a player picks them.
#[test]
fn own_and_other_players_structures_have_their_info_panels() {
    let mut app = game();
    assert_eq!(info(&mut app), (None, None, None));
    click(&mut app, tile_center(2, 2));
    assert_eq!(selected(&app), [DC]);
    assert_eq!(info(&mut app), panel("Deployment Center", "HP 1000 / 1000"));
    click(&mut app, tile_center(6, 5));
    assert_eq!(selected(&app), [ENEMY_GENERATOR]);
    assert_eq!(info(&mut app), panel("Generator", "Player 2\nHP 350 / 350"));
    // To player 2, the Generator is its own.
    app.world_mut().resource_mut::<InjectedInput>().tap_key(KeyCode::F2);
    app.update();
    click(&mut app, tile_center(6, 5));
    assert_eq!(selected(&app), [ENEMY_GENERATOR]);
    assert_eq!(info(&mut app), panel("Generator", "HP 350 / 350"));
}

/// Design change 0105's verifications through the panel, with the fog of war: player 1's Barracks, selected by a click
/// while it produces, shows its unit in production with the progress, the queue length and the rally point in the
/// activity area; to player 2, who sees it from its Generator, it shows its owner and HP only ("no production or
/// queue information"). The lines themselves are `activity`'s unit test.
#[test]
fn an_own_producing_barracks_shows_its_activity_to_its_owner_only() {
    use sc_sim::{ObjectId, ProductionCommand, Target};
    let mut app = game();
    spawn(&mut app, ObjectKind::Barracks, P1, tile(9, 4)); // (9, 4) to (11, 5)
    spawn(&mut app, ObjectKind::Generator, P1, tile(14, 1));
    spawn(&mut app, ObjectKind::Generator, P1, tile(17, 1));
    assert_eq!(bar(&mut app)[2], "Power 51/70", "full speed");
    debug(
        &mut app,
        DebugCommand::SetResources {
            player: P1,
            crystals: 1000,
            supplies: 0,
            alloys: 0,
            essence: 0,
            flux: 0,
        },
    );
    let barracks = ObjectId(10);
    let production = |command| PlayerCommand {
        player: P1,
        command: Command::Production {
            objects: vec![barracks],
            command,
        },
    };
    let rally = ProductionCommand::SetRally(Target::Point(tile_center(15, 5)));
    let train = ProductionCommand::Train(ObjectKind::Peacekeeper);
    app.world_mut().resource_mut::<PendingCommands>().0.extend([
        production(train),
        production(train),
        production(rally),
    ]);
    click(&mut app, tile_center(10, 4));
    assert_eq!(selected(&app), [barracks.0]);
    let (title, body, activity) = info(&mut app);
    assert_eq!(
        (title.as_deref(), body.as_deref()),
        (Some("Barracks"), Some("HP 300 / 300"))
    );
    let activity = activity.expect("an activity area");
    let lines: Vec<&str> = activity.lines().collect();
    assert!(lines[0].starts_with("Training Peacekeeper: "), "{lines:?}");
    assert_eq!(lines[1..], ["Queue: 2 / 20", "Rally point: ground"]);

    app.world_mut().resource_mut::<InjectedInput>().tap_key(KeyCode::F2);
    app.update();
    click(&mut app, tile_center(10, 4));
    assert_eq!(selected(&app), [barracks.0]);
    assert_eq!(info(&mut app), panel("Barracks", "Player 1\nHP 300 / 300"));
}

/// Design change 0099's third verification, as 0155 limits it (slice M3-J2): a click on a plate's tile selects the
/// plate, and its panel shows its owner the crystals left in the patch under it, which go down with every load
/// (`factions/gdo/economy.md > ExtractionPlate > InfoPanel`); another player's plate shows its name, owner and HP
/// only, the reduced panel of another player's structure (0105). Player 2 (F2) sees its own plate's crystals and
/// not player 1's, which a Generator of its own (10) beside them lets it see.
#[test]
fn only_its_owner_sees_the_crystals_left_under_a_plate() {
    let mut app = game();
    spawn(&mut app, ObjectKind::Generator, P2, tile(1, 10));
    click(&mut app, tile_center(1, 8));
    assert_eq!(selected(&app), [PLATE]);
    assert_eq!(
        info(&mut app),
        panel("Extraction Plate", "HP 85 / 85\nSpace Crystals: 1500")
    );
    run_until(&mut app, 48);
    assert_eq!(
        info(&mut app),
        panel("Extraction Plate", "HP 85 / 85\nSpace Crystals: 1490")
    );
    click(&mut app, tile_center(5, 7));
    assert_eq!(selected(&app), [ENEMY_PLATE]);
    assert_eq!(info(&mut app), panel("Extraction Plate", "Player 2\nHP 85 / 85"));

    app.world_mut().resource_mut::<InjectedInput>().tap_key(KeyCode::F2);
    app.update();
    click(&mut app, tile_center(5, 7));
    assert_eq!(selected(&app), [ENEMY_PLATE]);
    let (_, body, _) = info(&mut app);
    let body = body.expect("lines");
    assert!(
        body.starts_with("HP 85 / 85\nSpace Crystals: "),
        "its own plate: {body:?}"
    );
    click(&mut app, tile_center(1, 8));
    assert_eq!(selected(&app), [PLATE]);
    assert_eq!(info(&mut app), panel("Extraction Plate", "Player 1\nHP 85 / 85"));
}

/// (M5-S6) While the fog of war is off, the top bar says so, "Fog off (F3)", after the player's name: F3, the debug
/// key next to F2's player switch, turns it off and the note shows; F3 again turns it on and the note goes. A game
/// started with `--no-fog` shows it from the start. (A replay's playback too: `replay.rs`.)
#[test]
fn the_top_bar_notes_while_the_fog_is_off() {
    let fog = |app: &App| app.world().resource::<FogOfWar>().enabled;
    let mut app = game();
    assert!(fog(&app));
    assert_eq!(notes(&mut app), [""; 0]);
    tap(&mut app, KeyCode::F3);
    assert!(!fog(&app));
    assert_eq!(notes(&mut app), ["Fog off (F3)"]);
    assert_eq!(TopBarNote::FogOff.text(), "Fog off (F3)");
    tap(&mut app, KeyCode::F3);
    assert!(fog(&app));
    assert_eq!(notes(&mut app), [""; 0]);

    let mut app = game_of(scenario(), P1, true);
    assert!(!fog(&app));
    assert_eq!(notes(&mut app), ["Fog off (F3)"]);
    tap(&mut app, KeyCode::F3);
    assert_eq!(notes(&mut app), [""; 0]);
}
