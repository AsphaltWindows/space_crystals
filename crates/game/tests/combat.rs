//! Combat feedback in the client (`combat_effects`): a tracer for each shot, HP bars over damaged and selected units
//! (with shield bars over those that have Shields) and a marker where a unit died, each kept for its number of
//! ticks. Headless, one tick per update, as in
//! `client.rs`. How the fog of war hides them is tested in `fog.rs`.

use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;

use sc_sim::{
    Command, DebugCommand, ObjectId, ObjectKind, OrderKind, PlayerCommand, PlayerId, SpawnAt, Tick, scenarios,
};
use space_crystals::combat_effects::{
    CombatEffects, CombatEffectsMesh, DEATH_MARKER_TICKS, DeathMarker, HpBar, TRACER_TICKS, hp_bars,
};
use space_crystals::fog::{FogOfWar, Sight};
use space_crystals::platform::DisplayMode;
use space_crystals::selection::{ControlStates, Selection};
use space_crystals::sim_driver::{LocalPlayer, PendingCommands, SimRes};
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
        orientation: Default::default(),
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

/// The HP bars drawn at the end of the tick, with the local player's selection and sight.
fn bars(app: &App) -> Vec<HpBar> {
    let world = app.world();
    let player = world.resource::<LocalPlayer>().0;
    let selection = world.resource::<ControlStates>().selection(player);
    let sim = world.resource::<SimRes>();
    let sight = Sight::new(sim, player, world.resource::<FogOfWar>().enabled);
    hp_bars(sim, &sight, 1.0, |id| selection.contains(id))
}

fn effects_visible(app: &mut App) -> bool {
    let mut query = app.world_mut().query_filtered::<&Visibility, With<CombatEffectsMesh>>();
    *query.single(app.world()).unwrap() != Visibility::Hidden
}

/// A Peacekeeper and a Guard, 160 su apart, attack each other: their first shots land 2 ticks after the orders
/// (Aim 2, Firing 1) and leave tracers for `TRACER_TICKS`; the hits leave HP bars; the Guard's tenth hit kills the
/// Peacekeeper, which leaves a marker for `DEATH_MARKER_TICKS`. A selected unit shows its HP bar even at full HP.
/// A second Peacekeeper of player 1 stands by, so that player 1 still sees the Guard once the first is dead; with
/// auto-targeting off (a debug command), it only watches.
#[test]
fn shots_hits_and_deaths_show_for_their_ticks() {
    let mut app = headless_app();
    submit(
        &mut app,
        P1,
        Command::Debug(DebugCommand::SetAutoTargeting { on: false }),
    );
    app.update();
    spawn(&mut app, ObjectKind::Peacekeeper, P1, Vec2::new(300.0, 300.0));
    spawn(&mut app, ObjectKind::Guard, P2, Vec2::new(460.0, 300.0));
    spawn(&mut app, ObjectKind::Peacekeeper, P1, Vec2::new(460.0, 520.0));
    app.update();
    let spawned = tick(&app);
    assert!(effects(&app).tracers.is_empty() && bars(&app).is_empty());
    assert!(!effects_visible(&mut app), "nothing to show");
    app.world_mut().resource_mut::<ControlStates>().of_mut(P1).selection = Selection::new([(GUARD, ObjectKind::Guard)]);
    assert_eq!(bars(&app).iter().map(|bar| bar.fraction).collect::<Vec<_>>(), [1.0]);
    app.world_mut().resource_mut::<ControlStates>().of_mut(P1).selection = Selection::default();

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
    let mut fractions: Vec<f32> = bars(&app).iter().map(|bar| bar.fraction).collect();
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
    let guard_bars = bars(&app);
    assert_eq!(guard_bars.len(), 1, "the Guard's");
    assert_eq!(guard_bars[0].fraction, 35.0 / 80.0);

    run_until(&mut app, death + DEATH_MARKER_TICKS);
    assert!(effects(&app).deaths.is_empty(), "faded");
    assert!(effects_visible(&mut app), "the Guard's HP bar stays");
}

/// Slice M3-B (`systems/combat.md > Shields > Display`, design change 0148: "a shield bar sits directly above the HP
/// bar of every object with Shields"): an object that has Shields gets a shield bar with its HP bar, filled with the
/// share of its MaxShields it has left, and shows them while its Shields alone are short; one without Shields has no
/// shield bar. Player 2's Guard shoots player 1's Sabre Legionnaire, 160 su off: its 6 take 6 of the Sabre's 10 Shields
/// and no HP, so the Sabre's bars show with its HP full, and 0.25 of the Shields back by the end of the tick (`>
/// ShieldRegenRate`). Once the Guard stops, the Shields regenerate to full and the bars go; a selected Guard shows its
/// HP bar alone.
#[test]
fn an_object_with_shields_shows_a_shield_bar_above_its_hp_bar() {
    const SABRE: ObjectId = ObjectId(1);
    let mut app = headless_app();
    submit(
        &mut app,
        P1,
        Command::Debug(DebugCommand::SetAutoTargeting { on: false }),
    );
    app.update();
    spawn(&mut app, ObjectKind::SabreLegionnaire, P1, Vec2::new(300.0, 300.0));
    spawn(&mut app, ObjectKind::Guard, P2, Vec2::new(460.0, 300.0));
    app.update();
    let spawned = tick(&app);
    assert!(bars(&app).is_empty(), "full HP and Shields");
    attack(&mut app, P2, GUARD, SABRE);
    run_until(&mut app, spawned + 3);
    let shown: Vec<(f32, Option<f32>)> = bars(&app).iter().map(|bar| (bar.fraction, bar.shields)).collect();
    assert_eq!(shown, [(1.0, Some(4.25 / 10.0))], "the Sabre's, its HP full");

    let stop = Command::Order {
        units: vec![GUARD],
        kind: OrderKind::Stop,
        queue: false,
    };
    submit(&mut app, P2, stop);
    run_until(&mut app, spawned + 3 + 8 + 24);
    assert!(bars(&app).is_empty(), "the Shields are full again");
    app.world_mut().resource_mut::<ControlStates>().of_mut(P1).selection = Selection::new([(GUARD, ObjectKind::Guard)]);
    let shown: Vec<(f32, Option<f32>)> = bars(&app).iter().map(|bar| (bar.fraction, bar.shields)).collect();
    assert_eq!(shown, [(1.0, None)]);
}
