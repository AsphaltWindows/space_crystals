//! What the client shows of the structures and resources the local player remembers (`core/vision.md > Explored`):
//! Explored tiles show their records, as last seen, never the live objects there; only a GDO player may select a
//! remembered SupplyDeliveryStation (`> Explored Selectability`), whose InfoPanel shows it as last seen but with the
//! station's real delivery timer (design change 0057); and a shot from the fog shows from the fog edge, but whole
//! from a shooter it revealed, at the player's own objects (design changes 0177, 0206 and 0244); and the markers on
//! a remembered target stay on its record (design change 0139). Headless, one tick per update, as in `fog.rs`.
//!
//! One test binary: `records` (the records drawn on Explored tiles, an enemy structure's as placed, a Tunnel's at the
//! tier last seen), `station` (the remembered SupplyDeliveryStation: who may select it, its frame and its InfoPanel),
//! `shots` (a shot from the fog) and `markers` (the markers on a remembered target).

mod markers;
mod records;
mod shots;
mod station;

use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;

use sc_sim::vision::FOG_DELAY_TICKS;
use sc_sim::{
    Command, DebugCommand, Faction, ObjectId, ObjectKind, PlayerCommand, PlayerId, Spawn, SpawnAt, SpawnSettings, Tick,
    TilePos, scenarios,
};
use space_crystals::fog::Sight;
use space_crystals::object_render::{ObjectVisuals, RememberedVisual, RememberedVisuals};
use space_crystals::platform::DisplayMode;
use space_crystals::selection::Selectables;
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
            orientation: Default::default(),
        },
    );
}

/// A flat 24 × 16 map with a crystal patch on tile (8, 4) and a station on (8, 10)–(9, 11); player 1, of `faction`,
/// has a scout (a Peacekeeper, SightRange 5) on (5, 7), which sees both. The station has its default values: empty,
/// counting down to its first delivery.
fn world(faction: Faction) -> App {
    let station = SpawnSettings::defaults(ObjectKind::SupplyDeliveryStation).unwrap();
    world_with(faction, station)
}

/// [`world`], with the station's values.
fn world_with(faction: Faction, station: SpawnSettings) -> App {
    let mut scenario = scenarios::empty(24, 16);
    let tile = |kind, x, y| Spawn::new(kind, None, SpawnAt::Tile(TilePos::new(x, y)));
    let mut station_spawn = tile(ObjectKind::SupplyDeliveryStation, 8, 10);
    station_spawn.settings = Some(station);
    scenario.map.spawns = vec![tile(ObjectKind::CrystalPatch, 8, 4), station_spawn];
    scenario.setup.players[0].faction = faction;
    let mut app = headless(AppConfig::new(scenario));
    spawn_unit(&mut app, P1, 5, 7);
    app.update();
    app
}

/// Moves player 1's scout to tile (`x`, `y`) in the next tick, and runs on through the fog delay (design change 0178),
/// until the tiles it left are Explored: (20, 7) sees neither the patch nor the station.
fn scout_to(app: &mut App, x: i32, y: i32) {
    move_to(app, ObjectId(SCOUT), x, y);
}

/// [`scout_to`] for unit `id`.
fn move_to(app: &mut App, id: ObjectId, x: i32, y: i32) {
    let pos = centre(x, y);
    debug(app, DebugCommand::SetPosition { id, pos });
    app.update();
    let moved = tick(app);
    run_until(app, moved + FOG_DELAY_TICKS);
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
