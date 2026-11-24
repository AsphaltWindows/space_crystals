//! What the client shows of the structures and resources the local player remembers (`core/vision.md > Explored`):
//! Explored tiles show their records, as last seen, never the live objects there; only a GDO player may select a
//! remembered SupplyDeliveryStation (`> Explored Selectability`), whose InfoPanel shows it as last seen; and a unit
//! hit from the fog flashes, with no tracer (Q0017 point 1's assumption). Headless, one tick per update, as in
//! `fog.rs`.

use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;

use sc_sim::testing::ScenarioBuilder;
use sc_sim::{
    Command, DebugCommand, Faction, ObjectId, ObjectKind, OrderKind, PlayerCommand, PlayerId, ResourceState, Spawn,
    SpawnAt, TICKS_PER_SECOND, Tick, TilePos, scenarios,
};
use space_crystals::combat_effects::{CombatEffects, HIT_FLASH_TICKS, ShotView, shot_view, shows_tracer};
use space_crystals::fog::Sight;
use space_crystals::hud::info_panel::InfoText;
use space_crystals::input::InjectedInput;
use space_crystals::object_render::{ObjectVisuals, RememberedVisual, RememberedVisuals};
use space_crystals::platform::DisplayMode;
use space_crystals::selection::{ControlStates, Selectables, Selection};
use space_crystals::sim_driver::{LocalPlayer, PendingCommands, SimRes};
use space_crystals::{AppConfig, build_app};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
/// The objects of [`world`], by id.
const PATCH: u32 = 1;
const STATION: u32 = 2;
const SCOUT: u32 = 3;

fn headless(mut config: AppConfig) -> App {
    config.display = DisplayMode::Headless;
    let mut app = build_app(config);
    app.insert_resource(TimeUpdateStrategy::FixedTimesteps(1));
    app.finish();
    app.cleanup();
    app.update();
    app
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

fn debug(app: &mut App, command: DebugCommand) {
    submit(app, P1, Command::Debug(command));
}

fn centre(x: i32, y: i32) -> Vec2 {
    TilePos::new(x, y).center()
}

fn spawn_unit(app: &mut App, owner: PlayerId, x: i32, y: i32) {
    let at = SpawnAt::Point(centre(x, y));
    debug(
        app,
        DebugCommand::Spawn {
            kind: ObjectKind::Peacekeeper,
            owner: Some(owner),
            at,
        },
    );
}

fn tap(app: &mut App, key: KeyCode) {
    app.world_mut().resource_mut::<InjectedInput>().keys.press(key);
    app.update();
    app.world_mut().resource_mut::<InjectedInput>().keys.release(key);
    app.update();
}

/// A flat 24 × 16 map with a crystal patch on tile (8, 4) and a station on (8, 10)–(9, 11); player 1, of `faction`,
/// has a scout (a Peacekeeper, SightRange 5) on (5, 7), which sees both.
fn world(faction: Faction) -> App {
    let mut scenario = scenarios::empty(24, 16);
    let tile = |kind, x, y| Spawn::new(kind, None, SpawnAt::Tile(TilePos::new(x, y)));
    scenario.map.spawns = vec![
        tile(ObjectKind::CrystalPatch, 8, 4),
        tile(ObjectKind::SupplyDeliveryStation, 8, 10),
    ];
    scenario.setup.players[0].faction = faction;
    let mut app = headless(AppConfig::new(scenario));
    spawn_unit(&mut app, P1, 5, 7);
    app.update();
    app
}

/// Moves player 1's scout to tile (`x`, `y`) in the next tick: (20, 7) sees neither the patch nor the station.
fn scout_to(app: &mut App, x: i32, y: i32) {
    let pos = centre(x, y);
    debug(
        app,
        DebugCommand::SetPosition {
            id: ObjectId(SCOUT),
            pos,
        },
    );
    app.update();
}

/// Whether the visual of live object `id` is drawn; `false` once the object is gone.
fn drawn(app: &App, id: u32) -> bool {
    let world = app.world();
    let Some(&entity) = world.resource::<ObjectVisuals>().by_id.get(&ObjectId(id)) else {
        return false;
    };
    *world.get::<Visibility>(entity).expect("visuals get a Visibility") != Visibility::Hidden
}

/// The objects whose records are drawn, by id, checked against their visuals' entities.
fn remembered(app: &App) -> Vec<u32> {
    let world = app.world();
    let visuals = &world.resource::<RememberedVisuals>().by_id;
    for (id, &entity) in visuals {
        assert_eq!(world.get::<RememberedVisual>(entity), Some(&RememberedVisual(*id)));
        assert_eq!(world.get::<Visibility>(entity), Some(&Visibility::Inherited));
    }
    visuals.keys().map(|id| id.0).collect()
}

fn translation(app: &App, entity: Entity) -> Vec3 {
    app.world().get::<Transform>(entity).unwrap().translation
}

/// What the local player can select, as the client decides it.
fn selectable(app: &App) -> Vec<u32> {
    let world = app.world();
    let sim = world.resource::<SimRes>();
    let sight = Sight::new(sim, world.resource::<LocalPlayer>().0, true);
    Selectables::new(sim, &sight, 1.0).iter().map(|o| o.id.0).collect()
}

fn select(app: &mut App, id: u32) {
    let selection = Selection::new([(ObjectId(id), ObjectKind::SupplyDeliveryStation)]);
    app.world_mut().resource_mut::<ControlStates>().of_mut(P1).selection = selection;
    app.update();
}

fn selected(app: &App) -> Vec<u32> {
    let states = app.world().resource::<ControlStates>();
    states.selection(P1).ids().map(|id| id.0).collect()
}

/// The InfoPanel's title and lines, while it shows something.
fn info(app: &mut App) -> Option<(String, String)> {
    let mut texts = app.world_mut().query::<(&InfoText, &Text, &Node)>();
    let (mut title, mut body) = (None, None);
    for (which, text, node) in texts.iter(app.world()) {
        if node.display == Display::None {
            continue;
        }
        match which {
            InfoText::Title => title = Some(text.0.clone()),
            InfoText::Body => body = Some(text.0.clone()),
        }
    }
    title.zip(body)
}

/// `core/vision.md > Explored`: out of sight, the patch and the station show as remembered, where they were, and
/// their live objects stay hidden. A patch removed out of sight is still drawn; one placed out of sight isn't. The
/// fog's debug toggle shows the live objects instead, the player switch the other player's memory. Back in sight,
/// the removed patch is gone and the live objects show.
#[test]
fn explored_tiles_draw_records_never_live_objects() {
    let mut app = world(Faction::Gdo);
    assert!(drawn(&app, PATCH) && drawn(&app, STATION));
    assert_eq!(remembered(&app), [0_u32; 0]);

    scout_to(&mut app, 20, 7);
    assert!(!drawn(&app, PATCH) && !drawn(&app, STATION));
    assert_eq!(remembered(&app), [PATCH, STATION]);
    let world = app.world();
    let (live, memory) = (
        &world.resource::<ObjectVisuals>().by_id,
        &world.resource::<RememberedVisuals>().by_id,
    );
    for id in [PATCH, STATION].map(ObjectId) {
        assert_eq!(
            translation(&app, memory[&id]),
            translation(&app, live[&id]),
            "where it was"
        );
    }

    debug(&mut app, DebugCommand::Remove { id: ObjectId(PATCH) });
    let at = SpawnAt::Tile(TilePos::new(6, 4));
    let kind = ObjectKind::CrystalPatch;
    debug(&mut app, DebugCommand::Spawn { kind, owner: None, at });
    app.update();
    let placed = 4;
    assert!(app.world().resource::<SimRes>().object(ObjectId(PATCH)).is_none());
    assert_eq!(remembered(&app), [PATCH, STATION], "the removal wasn't seen");
    assert!(!drawn(&app, placed), "placed in the fog: unknown");

    tap(&mut app, KeyCode::F3);
    assert_eq!(remembered(&app), [0_u32; 0], "the fog is off: everything live");
    assert!(drawn(&app, placed) && drawn(&app, STATION));
    tap(&mut app, KeyCode::F3);
    assert_eq!(remembered(&app), [PATCH, STATION]);
    tap(&mut app, KeyCode::F2);
    assert_eq!(app.world().resource::<LocalPlayer>().0, P2);
    assert_eq!(remembered(&app), [0_u32; 0], "player 2 never saw them");
    tap(&mut app, KeyCode::F2);
    assert_eq!(remembered(&app), [PATCH, STATION]);

    scout_to(&mut app, 5, 7);
    assert_eq!(remembered(&app), [0_u32; 0]);
    assert!(drawn(&app, placed) && drawn(&app, STATION));
    let sim = app.world().resource::<SimRes>();
    assert!(
        sim.vision().player(P1).unwrap().record(ObjectId(PATCH)).is_none(),
        "seen gone"
    );
}

/// `core/vision.md > Explored Selectability`: a GDO player may select the station on Explored tiles, by its record,
/// and keeps it selected when it leaves; a click there picks it. Its InfoPanel shows the station as last seen,
/// without CurrentSupplies, and when that was.
#[test]
fn gdo_players_select_a_remembered_station() {
    let mut app = world(Faction::Gdo);
    select(&mut app, STATION);
    let (title, body) = info(&mut app).unwrap();
    assert_eq!(title, "Supply Delivery Station");
    assert!(body.starts_with("Neutral\nSupplies: 0\n"), "{body}");

    scout_to(&mut app, 20, 7);
    let left = tick(&app);
    run_until(&mut app, left + 40);
    assert_eq!(
        selectable(&app),
        [STATION, SCOUT],
        "the patch isn't selectable, the station is"
    );
    assert_eq!(selected(&app), [STATION]);
    let sim = app.world().resource::<SimRes>();
    let objects = Selectables::new(sim, &Sight::new(sim, P1, true), 1.0);
    let station_centre = TilePos::new(9, 11).corner();
    assert_eq!(objects.at(station_centre).map(|o| o.id), Some(ObjectId(STATION)));

    let record = sim.vision().player(P1).unwrap().record(ObjectId(STATION)).unwrap();
    let Some(ResourceState::SupplyStation {
        next_delivery_in: Some(then),
        ..
    }) = record.resource
    else {
        panic!("{record:?}");
    };
    let Some(ResourceState::SupplyStation {
        next_delivery_in: Some(now),
        ..
    }) = sim.object(ObjectId(STATION)).unwrap().resource
    else {
        panic!("the station counts down");
    };
    let seconds = |ticks: u32| ticks.div_ceil(TICKS_PER_SECOND);
    assert!(seconds(then) > seconds(now), "{then} {now}");
    let ago = seconds(sim.tick() - record.seen);
    let (title, body) = info(&mut app).unwrap();
    assert_eq!(title, "Supply Delivery Station");
    assert_eq!(
        body,
        format!(
            "Neutral\nDelivery size: 4\nDelivery interval: 60 s\nNext delivery in {} s\nLast seen {ago} s ago",
            seconds(then)
        )
    );
}

/// Other factions may not select a remembered station: it leaves the selection when its tiles turn Explored.
#[test]
fn other_factions_cannot_select_a_remembered_station() {
    let mut app = world(Faction::Syndicate);
    select(&mut app, STATION);
    assert_eq!(selected(&app), [STATION], "Visible: selectable by anyone");
    scout_to(&mut app, 20, 7);
    assert_eq!(remembered(&app), [PATCH, STATION], "still drawn");
    assert_eq!(selectable(&app), [SCOUT]);
    assert_eq!(selected(&app), [0_u32; 0]);
    assert_eq!(info(&mut app), None);
}

/// 24 × 5 tiles: level 0 in the west, a ramp on column 12, level 1 from column 13 on.
const STEP: &str = "
    ........................
    ........................
    ........................
    ........................
    ........................

    0000000000000111111111111
    0000000000000111111111111
    0000000000000111111111111
    0000000000000111111111111
    0000000000000111111111111
    0000000000000111111111111
";

/// Q0017 point 1's assumption: player 2's Peacekeeper on the high ground (tile (14, 2), level 1) shoots player 1's
/// on (9, 2) from beyond player 1's sight (4 grid units up to level 1) but within its own range (5 grid units down
/// to level 0). Player 1 sees no tracer, but its unit flashes for `HIT_FLASH_TICKS`; with the fog off, and for
/// player 2, the tracer shows instead.
#[test]
fn a_unit_hit_from_the_fog_flashes_without_a_tracer() {
    let mut app = headless(AppConfig::new(ScenarioBuilder::new(STEP).build()));
    spawn_unit(&mut app, P1, 9, 2);
    spawn_unit(&mut app, P2, 14, 2);
    app.update();
    let (target, attacker) = (ObjectId(1), ObjectId(2));
    {
        let sim = app.world().resource::<SimRes>();
        let unit = |id| sim.object(id).unwrap();
        assert!(!sim.is_visible(P1, unit(attacker)) && sim.is_visible(P2, unit(target)));
    }
    let units = vec![attacker];
    let kind = OrderKind::Attack(target);
    let queue = false;
    submit(&mut app, P2, Command::Order { units, kind, queue });
    let ordered = tick(&app);
    while app.world().resource::<CombatEffects>().tracers.is_empty() {
        assert!(tick(&app) < ordered + 20, "player 2 never fired");
        app.update();
    }
    let shot = tick(&app);
    let world = app.world();
    let sim = world.resource::<SimRes>();
    let tracer = &world.resource::<CombatEffects>().tracers[0];
    assert_eq!((tracer.target, tracer.tick), (target, shot));
    let seen_by = |player, fog| shot_view(sim, &Sight::new(sim, player, fog), tracer);
    assert!(!shows_tracer(&Sight::new(sim, P1, true), tracer), "from the fog");
    assert_eq!(seen_by(P1, true), Some(ShotView::Flash(sim.object(target).unwrap())));
    assert_eq!(seen_by(P1, false), Some(ShotView::Tracer), "the fog off");
    assert_eq!(seen_by(P2, true), Some(ShotView::Tracer), "the shooter's own");

    run_until(&mut app, shot + HIT_FLASH_TICKS - 1);
    let world = app.world();
    let sim = world.resource::<SimRes>();
    let tracer = &world.resource::<CombatEffects>().tracers[0];
    assert!(matches!(
        shot_view(sim, &Sight::new(sim, P1, true), tracer),
        Some(ShotView::Flash(_))
    ));
    app.update();
    let world = app.world();
    let sim = world.resource::<SimRes>();
    let flashes = world.resource::<CombatEffects>().tracers.iter();
    assert!(
        flashes
            .filter(|t| t.tick == shot)
            .all(|t| shot_view(sim, &Sight::new(sim, P1, true), t).is_none()),
        "over"
    );
}
