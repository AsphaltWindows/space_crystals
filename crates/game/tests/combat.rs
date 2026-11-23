//! Combat feedback in the client (`combat_effects`): a tracer for each shot, HP bars over damaged units and a marker
//! where a unit died, each kept for its number of ticks. Headless, one tick per update, as in `client.rs`.

use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;

use sc_sim::{
    Command, DebugCommand, ObjectId, ObjectKind, OrderKind, PlayerCommand, PlayerId, SpawnAt, Tick, scenarios,
};
use space_crystals::combat_effects::{
    CombatEffects, CombatEffectsMesh, DEATH_MARKER_TICKS, DeathMarker, TRACER_TICKS, hp_bars,
};
use space_crystals::platform::DisplayMode;
use space_crystals::sim_driver::{PendingCommands, SimRes};
use space_crystals::{AppConfig, build_app};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const PEACEKEEPER: ObjectId = ObjectId(1);
const GUARD: ObjectId = ObjectId(2);

fn headless_app() -> App {
    let mut config = AppConfig::new(scenarios::empty(16, 12));
    config.display = DisplayMode::Headless;
    let mut app = build_app(config);
    app.insert_resource(TimeUpdateStrategy::FixedTimesteps(1));
    app.finish();
    app.cleanup();
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

fn spawn(app: &mut App, kind: ObjectKind, owner: PlayerId, at: Vec2) {
    let at = SpawnAt::Point(at);
    let command = DebugCommand::Spawn {
        kind,
        owner: Some(owner),
        at,
    };
    submit(app, P1, Command::Debug(command));
}

fn attack(app: &mut App, player: PlayerId, unit: ObjectId, target: ObjectId) {
    let kind = OrderKind::Attack(target);
    submit(
        app,
        player,
        Command::Order {
            units: vec![unit],
            kind,
            queue: false,
        },
    );
}

fn effects(app: &App) -> &CombatEffects {
    app.world().resource::<CombatEffects>()
}

fn effects_visible(app: &mut App) -> bool {
    let mut query = app.world_mut().query_filtered::<&Visibility, With<CombatEffectsMesh>>();
    *query.single(app.world()).unwrap() != Visibility::Hidden
}

/// A Peacekeeper and a Guard, 160 su apart, attack each other: their first shots land 2 ticks after the orders
/// (Aim 2, Firing 1) and leave tracers for `TRACER_TICKS`; the hits leave HP bars; the Guard's tenth hit kills the
/// Peacekeeper, which leaves a marker for `DEATH_MARKER_TICKS`.
#[test]
fn shots_hits_and_deaths_show_for_their_ticks() {
    let mut app = headless_app();
    app.update();
    spawn(&mut app, ObjectKind::Peacekeeper, P1, Vec2::new(300.0, 300.0));
    spawn(&mut app, ObjectKind::Guard, P2, Vec2::new(460.0, 300.0));
    app.update();
    let spawned = tick(&app);
    assert!(effects(&app).tracers.is_empty() && hp_bars(app.world().resource::<SimRes>(), 1.0).is_empty());
    assert!(!effects_visible(&mut app), "nothing to show");

    attack(&mut app, P1, PEACEKEEPER, GUARD);
    attack(&mut app, P2, GUARD, PEACEKEEPER);
    let first_shots = spawned + 3;
    run_until(&mut app, first_shots);
    let tracers = &effects(&app).tracers;
    assert_eq!(tracers.len(), 2, "{tracers:?}");
    let shot = tracers.iter().find(|t| t.owner == Some(P1)).unwrap();
    assert_eq!(
        (shot.tick, shot.from, shot.to),
        (first_shots, Vec2::new(300.0, 300.0), Vec2::new(460.0, 300.0))
    );
    let mut fractions: Vec<f32> = hp_bars(app.world().resource::<SimRes>(), 1.0)
        .iter()
        .map(|bar| bar.fraction)
        .collect();
    fractions.sort_by(f32::total_cmp);
    assert_eq!(fractions, [71.0 / 80.0, 45.0 / 50.0]);
    assert!(effects_visible(&mut app));

    run_until(&mut app, first_shots + TRACER_TICKS);
    assert!(effects(&app).tracers.is_empty(), "faded");

    // Ten hits of 5, one every 8 ticks.
    let death = first_shots + 9 * 8;
    run_until(&mut app, death);
    assert_eq!(
        effects(&app).deaths,
        [DeathMarker {
            tick: death,
            kind: ObjectKind::Peacekeeper,
            owner: Some(P1),
            pos: Vec2::new(300.0, 300.0),
        }]
    );
    let bars = hp_bars(app.world().resource::<SimRes>(), 1.0);
    assert_eq!(bars.len(), 1, "the Guard's");
    assert_eq!(bars[0].fraction, 35.0 / 80.0);

    run_until(&mut app, death + DEATH_MARKER_TICKS);
    assert!(effects(&app).deaths.is_empty(), "faded");
    assert!(effects_visible(&mut app), "the Guard's HP bar stays");
}
