//! The fog of war in the client (`fog`): the fog layer's texture follows the local player's vision and is written
//! only when it changes; other players' objects that the local player doesn't see are neither drawn nor
//! selectable, nor their raised Riot Shields, leave its selection, can't be targeted and aren't marked by command
//! indicators; combat effects in the fog don't show; the debug toggle and `--no-fog` show everything; the map editor
//! never shows fog. Headless, one tick per update, as in `client.rs`.

use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;

use sc_sim::defs::RIOT_SHIELD_TOGGLE_TICKS;
use sc_sim::{
    Ability, Command, DebugCommand, ObjectId, ObjectKind, OrderKind, PlayerCommand, PlayerId, RiotShield, Scenario,
    SimConfig, SpawnAt, Target, Tick, TilePos, scenarios,
};
use space_crystals::cli;
use space_crystals::combat_effects::{
    CombatEffects, CombatEffectsMesh, DeathMarker, Tracer, shows_death, shows_tracer,
};
use space_crystals::command_ui::CursorTarget;
use space_crystals::command_ui::indicators::{ATTACK_COLOR, Indicator, selection_indicators};
use space_crystals::editor::EditorConfig;
use space_crystals::fog::{EXPLORED_ALPHA, FogAssets, FogLayer, FogOfWar, FogView, Sight, UNEXPLORED_ALPHA};
use space_crystals::input::InjectedInput;
use space_crystals::object_render::{ObjectVisuals, RiotShieldVisual};
use space_crystals::platform::DisplayMode;
use space_crystals::selection::{ControlStates, Selectables, Selection};
use space_crystals::sim_driver::{PendingCommands, SimRes};
use space_crystals::{AppConfig, ClientMode, build_app};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

fn headless(mut config: AppConfig) -> App {
    config.display = DisplayMode::Headless;
    let mut app = build_app(config);
    app.insert_resource(TimeUpdateStrategy::FixedTimesteps(1));
    app.finish();
    app.cleanup();
    app.update();
    app
}

/// A flat 24 × 16 map, players 1 and 2, as player 1 with debug commands on.
fn game() -> App {
    headless(AppConfig::new(scenarios::empty(24, 16)))
}

fn tick(app: &App) -> Tick {
    app.world().resource::<SimRes>().tick()
}

fn run_until(app: &mut App, until: Tick) {
    while tick(app) < until {
        app.update();
    }
}

fn submit(app: &mut App, player: PlayerId, command: Command) {
    app.world_mut()
        .resource_mut::<PendingCommands>()
        .0
        .push(PlayerCommand { player, command });
}

fn centre(x: i32, y: i32) -> Vec2 {
    TilePos::new(x, y).center()
}

/// Spawns a unit of `owner` on the centre of tile (`x`, `y`) in the next tick.
fn spawn(app: &mut App, kind: ObjectKind, owner: PlayerId, x: i32, y: i32) {
    let command = DebugCommand::Spawn {
        kind,
        owner: Some(owner),
        at: SpawnAt::Point(centre(x, y)),
        orientation: Default::default(),
    };
    submit(app, P1, Command::Debug(command));
}

fn order(app: &mut App, player: PlayerId, unit: u32, kind: OrderKind) {
    let units = vec![ObjectId(unit)];
    submit(
        app,
        player,
        Command::Order {
            units,
            kind,
            queue: false,
        },
    );
}

fn move_to(x: i32, y: i32) -> OrderKind {
    OrderKind::Move(Target::Point(centre(x, y)))
}

/// The fog texture's alpha over tile (`x`, `y`).
fn fog_alpha(app: &App, x: u32, y: u32) -> u8 {
    let world = app.world();
    let image = world
        .resource::<Assets<Image>>()
        .get(&world.resource::<FogAssets>().texture)
        .unwrap();
    image.data.as_ref().unwrap()[((y * image.width() + x) * 4 + 3) as usize]
}

fn byte(alpha: f32) -> u8 {
    (alpha * 255.0).round() as u8
}

fn texture_writes(app: &App) -> u32 {
    app.world().resource::<FogView>().texture_writes
}

/// Whether the visual of object `id` is drawn.
fn drawn(app: &mut App, id: u32) -> bool {
    let entity = app.world().resource::<ObjectVisuals>().by_id[&ObjectId(id)];
    *app.world().get::<Visibility>(entity).expect("visuals get a Visibility") != Visibility::Hidden
}

/// Whether the Riot Shield of unit `id` is drawn: shown, and its unit's visual drawn. A shield is never `Visible`
/// itself, which would show it whatever its unit's visibility.
fn shield_drawn(app: &mut App, id: u32) -> bool {
    let mut query = app.world_mut().query::<(&RiotShieldVisual, &Visibility)>();
    let visibility = query
        .iter(app.world())
        .find(|(shield, _)| shield.0 == ObjectId(id))
        .map(|(_, visibility)| *visibility)
        .expect("a Peacekeeper's shield");
    assert_ne!(visibility, Visibility::Visible, "never shown on its own");
    visibility == Visibility::Inherited && drawn(app, id)
}

fn fog_layer_drawn(app: &mut App) -> bool {
    let mut query = app.world_mut().query_filtered::<&Visibility, With<FogLayer>>();
    *query.single(app.world()).unwrap() != Visibility::Hidden
}

/// The objects player 1 can select, as the client decides it.
fn selectable(app: &App) -> Vec<u32> {
    let world = app.world();
    let sim = world.resource::<SimRes>();
    let fog = world.resource::<FogOfWar>().applies(*world.resource::<ClientMode>());
    let sight = Sight::new(sim, P1, fog);
    Selectables::new(sim, &sight, 1.0).iter().map(|o| o.id.0).collect()
}

fn tap(app: &mut App, key: KeyCode) {
    app.world_mut().resource_mut::<InjectedInput>().keys.press(key);
    app.update();
    app.world_mut().resource_mut::<InjectedInput>().keys.release(key);
    app.update();
}

/// The texture shows the local player's tiles, and is written again only when one of them changes state: black
/// where it has never looked, dimmed where it has, clear where it looks now.
#[test]
fn the_fog_texture_follows_the_local_players_vision() {
    let mut app = game();
    assert_eq!(fog_alpha(&app, 5, 5), byte(UNEXPLORED_ALPHA), "nothing seen yet");
    assert!(fog_layer_drawn(&mut app));
    spawn(&mut app, ObjectKind::Peacekeeper, P1, 5, 5);
    app.update();
    let spawned = tick(&app);
    assert_eq!(fog_alpha(&app, 5, 5), 0);
    assert_eq!(fog_alpha(&app, 10, 5), 0, "5 tiles east");
    assert_eq!(fog_alpha(&app, 11, 5), byte(UNEXPLORED_ALPHA));
    let writes = texture_writes(&app);
    run_until(&mut app, spawned + 10);
    assert_eq!(texture_writes(&app), writes, "nothing changed, nothing written");

    order(&mut app, P1, 1, move_to(15, 5));
    let pos = |app: &App| app.world().resource::<SimRes>().object(ObjectId(1)).unwrap().pos;
    while pos(&app) != centre(15, 5) {
        assert!(tick(&app) < spawned + 400, "the Peacekeeper never arrived");
        app.update();
    }
    assert_eq!(fog_alpha(&app, 5, 5), byte(EXPLORED_ALPHA), "left behind");
    assert_eq!(fog_alpha(&app, 15, 5), 0);
    assert_eq!(fog_alpha(&app, 20, 5), 0);
    assert_eq!(fog_alpha(&app, 21, 5), byte(UNEXPLORED_ALPHA));
    assert!(texture_writes(&app) > writes);
    assert_eq!(
        app.world().resource::<FogView>().mesh_builds,
        1,
        "the map never changed"
    );
}

/// An enemy the local player doesn't see is neither drawn nor selectable; one it sees is both. A selected enemy
/// that walks out of sight leaves the selection.
#[test]
fn hidden_enemies_are_not_drawn_or_selectable() {
    let mut app = game();
    spawn(&mut app, ObjectKind::Peacekeeper, P1, 4, 8);
    spawn(&mut app, ObjectKind::Guard, P2, 20, 8);
    spawn(&mut app, ObjectKind::Guard, P2, 7, 8);
    app.update();
    assert!(drawn(&mut app, 1) && !drawn(&mut app, 2) && drawn(&mut app, 3));
    assert_eq!(selectable(&app), [1, 3]);

    app.world_mut().resource_mut::<ControlStates>().of_mut(P1).selection =
        Selection::new([(ObjectId(3), ObjectKind::Guard)]);
    order(&mut app, P2, 3, move_to(20, 13));
    let start = tick(&app);
    while drawn(&mut app, 3) {
        assert!(tick(&app) < start + 200, "the Guard never left player 1's sight");
        app.update();
    }
    assert!(app.world().resource::<ControlStates>().selection(P1).is_empty());
    assert_eq!(selectable(&app), [1]);
}

/// A raised Riot Shield is drawn with its unit (`object_render::RiotShieldVisual`), so not for an enemy the local
/// player doesn't see.
#[test]
fn a_hidden_enemys_shield_is_not_drawn() {
    let mut app = game();
    spawn(&mut app, ObjectKind::Peacekeeper, P1, 4, 8);
    spawn(&mut app, ObjectKind::Peacekeeper, P2, 20, 8);
    spawn(&mut app, ObjectKind::Peacekeeper, P2, 7, 8);
    app.update();
    let units = vec![ObjectId(2), ObjectId(3)];
    let ability = Ability::RiotShield { raise: true };
    submit(&mut app, P2, Command::Ability { units, ability });
    let raised = tick(&app) + RIOT_SHIELD_TOGGLE_TICKS + 1;
    run_until(&mut app, raised);
    let sim = app.world().resource::<SimRes>();
    assert_eq!(sim.object(ObjectId(2)).unwrap().riot_shield, Some(RiotShield::Raised));
    assert!(!shield_drawn(&mut app, 1), "down");
    assert!(shield_drawn(&mut app, 3), "in sight");
    assert!(!shield_drawn(&mut app, 2), "in the fog");
}

/// The command flow reaches only what the local player sees: the cursor over an enemy hidden by the fog points at
/// the ground (a right-click there moves, not attacks), and an attack's marker on its target goes while the target
/// is hidden, though the order stands.
#[test]
fn hidden_enemies_cannot_be_targeted_or_marked() {
    let mut app = game();
    spawn(&mut app, ObjectKind::Peacekeeper, P1, 4, 8);
    spawn(&mut app, ObjectKind::Guard, P2, 20, 8);
    spawn(&mut app, ObjectKind::Guard, P2, 7, 8);
    app.update();
    // On a flat map a ground point is drawn where it is: the view point is the ground point.
    let under_cursor = |app: &App, x, y| {
        let sim = app.world().resource::<SimRes>();
        let objects = Selectables::new(sim, &Sight::new(sim, P1, true), 1.0);
        CursorTarget::at(&objects, centre(x, y), centre(x, y))
    };
    assert_eq!(under_cursor(&app, 20, 8), CursorTarget::Ground(centre(20, 8)));
    assert_eq!(under_cursor(&app, 7, 8), CursorTarget::EnemyObject(ObjectId(3)));

    app.world_mut().resource_mut::<ControlStates>().of_mut(P1).selection =
        Selection::new([(ObjectId(1), ObjectKind::Peacekeeper)]);
    order(&mut app, P1, 1, OrderKind::Attack(ObjectId(3)));
    order(&mut app, P2, 3, move_to(20, 13));
    app.update();
    let markers = |app: &App| {
        let world = app.world();
        let sim = world.resource::<SimRes>();
        let states = world.resource::<ControlStates>();
        selection_indicators(sim, &Sight::new(sim, P1, true), states, false)
    };
    let attack_marker = Indicator::Object {
        id: ObjectId(3),
        color: ATTACK_COLOR,
    };
    assert_eq!(markers(&app), [attack_marker]);
    let start = tick(&app);
    while drawn(&mut app, 3) {
        assert!(tick(&app) < start + 200, "the Guard never left player 1's sight");
        app.update();
    }
    let sim = app.world().resource::<SimRes>();
    let current = sim
        .object(ObjectId(1))
        .unwrap()
        .orders
        .as_ref()
        .unwrap()
        .current
        .as_ref();
    assert_eq!(current.map(|c| c.order.kind), Some(OrderKind::Attack(ObjectId(3))));
    assert_eq!(markers(&app), []);
}

/// Shots and deaths player 1 can't see don't show: a tracer needs both ends on Visible tiles, a death marker a
/// Visible tile or one of the player's own objects. A fight between two of player 2's units in player 1's fog draws
/// nothing for player 1.
#[test]
fn combat_in_the_fog_shows_nothing() {
    let mut app = game();
    spawn(&mut app, ObjectKind::Peacekeeper, P1, 4, 8);
    spawn(&mut app, ObjectKind::Guard, P2, 18, 8);
    spawn(&mut app, ObjectKind::Peacekeeper, P2, 20, 8);
    app.update();
    {
        let world = app.world();
        let sight = Sight::new(world.resource::<SimRes>(), P1, true);
        let tracer = |from, to| Tracer {
            tick: 1,
            owner: Some(P2),
            target: ObjectId(1),
            from,
            to,
        };
        assert!(shows_tracer(&sight, &tracer(centre(4, 8), centre(6, 8))));
        assert!(
            !shows_tracer(&sight, &tracer(centre(18, 8), centre(4, 8))),
            "from the fog"
        );
        let death = |owner, x, y| DeathMarker {
            tick: 1,
            kind: ObjectKind::Guard,
            owner: Some(owner),
            pos: centre(x, y),
        };
        assert!(shows_death(&sight, &death(P2, 6, 8)));
        assert!(!shows_death(&sight, &death(P2, 18, 8)));
        assert!(shows_death(&sight, &death(P1, 18, 8)), "its own");
    }

    order(&mut app, P2, 2, OrderKind::Attack(ObjectId(3)));
    let ordered = tick(&app);
    run_until(&mut app, ordered + 4);
    assert!(
        !app.world().resource::<CombatEffects>().tracers.is_empty(),
        "they fight"
    );
    let mut effects = app.world_mut().query_filtered::<&Visibility, With<CombatEffectsMesh>>();
    assert_eq!(*effects.single(app.world()).unwrap(), Visibility::Hidden);
}

/// The debug toggle (F3) and `--no-fog` show everything, as if every tile were Visible; the toggle works only with
/// debug commands on, and the sim's vision is the same either way.
#[test]
fn the_fog_can_be_turned_off_for_qa() {
    let mut app = game();
    spawn(&mut app, ObjectKind::Peacekeeper, P1, 4, 8);
    spawn(&mut app, ObjectKind::Guard, P2, 20, 8);
    app.update();
    let hash = |app: &App| app.world().resource::<SimRes>().state_hash();
    assert!(!drawn(&mut app, 2) && fog_layer_drawn(&mut app));
    tap(&mut app, KeyCode::F3);
    assert!(!app.world().resource::<FogOfWar>().enabled);
    assert!(drawn(&mut app, 2) && !fog_layer_drawn(&mut app));
    assert_eq!(selectable(&app), [1, 2]);
    tap(&mut app, KeyCode::F3);
    assert!(app.world().resource::<FogOfWar>().enabled);
    assert!(!drawn(&mut app, 2) && fog_layer_drawn(&mut app));

    // Without debug commands, F3 does nothing.
    let mut config = AppConfig::new(scenarios::empty(24, 16));
    config.scenario.setup.config = SimConfig::default();
    let mut app = headless(config);
    tap(&mut app, KeyCode::F3);
    assert!(app.world().resource::<FogOfWar>().enabled);

    // `--no-fog`: off from the start, the sim seeing what it always sees.
    let mut config = AppConfig::new(scenarios::empty(24, 16));
    config.qa.no_fog = true;
    let mut fogless = headless(config);
    let mut fogged = game();
    for app in [&mut fogless, &mut fogged] {
        spawn(app, ObjectKind::Peacekeeper, P1, 4, 8);
        spawn(app, ObjectKind::Guard, P2, 20, 8);
        app.update();
    }
    assert!(drawn(&mut fogless, 2) && !fog_layer_drawn(&mut fogless));
    assert_eq!(hash(&fogless), hash(&fogged));
}

/// `--no-fog` is a flag of its own, refused in a match without debug commands.
#[test]
fn the_no_fog_flag() {
    let args = |list: &[&str]| list.iter().map(ToString::to_string).collect::<Vec<_>>();
    let config = cli::into_config(cli::parse(&args(&["--no-fog"])).unwrap()).unwrap();
    assert!(config.qa.no_fog && !config.qa.is_active());
    let error = cli::into_config(cli::parse(&args(&["--no-fog", "--no-debug"])).unwrap()).unwrap_err();
    assert!(error.contains("--no-fog"), "{error}");
}

/// The map editor shows the whole map: no fog layer, every object drawn.
#[test]
fn the_editor_shows_no_fog() {
    let mut scenario: Scenario = scenarios::empty(24, 16);
    let spawn = |x, y| sc_sim::Spawn::new(ObjectKind::CrystalPatch, None, SpawnAt::Tile(TilePos::new(x, y)));
    scenario.map.spawns = vec![spawn(3, 3), spawn(20, 12)];
    let dir = std::env::temp_dir().join(format!("sc_fog_test_{}", std::process::id()));
    let mut app = headless(AppConfig::editor(
        scenario,
        EditorConfig {
            path: None,
            maps_dir: dir,
            script: Vec::new(),
        },
    ));
    app.update();
    assert_eq!(*app.world().resource::<ClientMode>(), ClientMode::Editor);
    assert!(!fog_layer_drawn(&mut app));
    assert!(drawn(&mut app, 1) && drawn(&mut app, 2));
}
