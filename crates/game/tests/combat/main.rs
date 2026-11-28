//! Combat feedback in the client (`combat_effects`): a tracer for each shot, HP bars over damaged and selected units
//! (with shield bars over those that have Shields) and a marker where a unit died, each kept for its number of
//! ticks. Headless, one tick per update, as in
//! `client.rs`. How the fog of war hides them is tested in `fog.rs`. The Gunner's shots, which can miss, are in
//! [`gunner`]; (M5-N4b) the Shroud clouds and their releases in [`shroud`].

mod gunner;
mod shroud;

use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;

use bevy::mesh::VertexAttributeValues;
use sc_sim::vision::Hiding;
use sc_sim::{
    Command, DebugCommand, ObjectId, ObjectKind, OrderKind, PlayerCommand, PlayerId, SU_PER_GU, SimEvent, SpawnAt,
    Tick, TilePos, scenarios,
};
use space_crystals::combat_effects::blasts::ring_color;
use space_crystals::combat_effects::{
    CombatEffects, CombatEffectsMesh, DeathMarker, HpBar, ShotStyle, hp_bars, shows_flight,
};
use space_crystals::command_ui::feedback::{TARGET_TOO_CLOSE_TEXT, TargetFeedback};
use space_crystals::coords::drawn_at_height;
use space_crystals::fog::{FogOfWar, Sight};
use space_crystals::object_render::{flight_lift, object_height, player_color};
use space_crystals::platform::DisplayMode;
use space_crystals::selection::{ControlStates, Selection};
use space_crystals::sim_driver::{LocalPlayer, PendingCommands, SimEventMessage, SimRes};
use space_crystals::viewer_events::Sighting;
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
    hp_bars(sim, &sight, 1.0, &space_crystals::tuning::ClientTuning::BUILTIN, |id| {
        selection.contains(id)
    })
}

fn effects_visible(app: &mut App) -> bool {
    let mut query = app.world_mut().query_filtered::<&Visibility, With<CombatEffectsMesh>>();
    *query.single(app.world()).unwrap() != Visibility::Hidden
}

/// A Peacekeeper and a Guard, 160 su apart, attack each other: the Peacekeeper's first shot lands 2 ticks after the
/// orders (Aim 2, Firing 1), the Guard's 4 (Aim 4, its laser rifle: design change 0186), and each leaves a tracer for
/// `FeedbackTuning::tracer_ticks`; the hits leave HP bars; the Guard's third hit of 17 kills the Peacekeeper, which
/// leaves a marker for `FeedbackTuning::death_marker_ticks`. A selected unit shows its HP bar even at full HP. A second
/// Peacekeeper of player 1 stands by, so that player 1 still sees the Guard once the first is dead; with auto-targeting
/// off (a debug command), it only watches.
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
    let first_shot = spawned + 3;
    run_until(&mut app, first_shot);
    let tracers = &effects(&app).tracers;
    assert_eq!(tracers.len(), 1, "{tracers:?}");
    let shot = &tracers[0];
    assert_eq!(
        (shot.tick, shot.owner, shot.from, shot.to),
        (first_shot, Some(P1), Vec2::new(300.0, 300.0), Vec2::new(460.0, 300.0))
    );
    let fractions: Vec<f32> = bars(&app).iter().map(|bar| bar.fraction).collect();
    assert_eq!(fractions, [71.0 / 80.0], "the Guard's");
    assert!(effects_visible(&mut app));

    let guard_shot = spawned + 5;
    run_until(&mut app, guard_shot);
    let tracers = &effects(&app).tracers;
    assert_eq!(tracers.len(), 2, "{tracers:?}");
    let shot = tracers.iter().find(|t| t.owner == Some(P2)).unwrap();
    assert_eq!(
        (shot.tick, shot.from, shot.to),
        (guard_shot, Vec2::new(460.0, 300.0), Vec2::new(300.0, 300.0))
    );
    let mut fractions: Vec<f32> = bars(&app).iter().map(|bar| bar.fraction).collect();
    fractions.sort_by(f32::total_cmp);
    assert_eq!(fractions, [33.0 / 50.0, 71.0 / 80.0]);

    run_until(
        &mut app,
        guard_shot + space_crystals::tuning::feedback::BUILTIN.tracer_ticks,
    );
    assert!(effects(&app).tracers.is_empty(), "faded");

    // Three hits of 17, one every 24 ticks; meanwhile the Peacekeeper lands three of 9.
    let death = guard_shot + 2 * 24;
    run_until(&mut app, death);
    assert_eq!(
        effects(&app).deaths,
        [DeathMarker {
            tick: death,
            kind: ObjectKind::Peacekeeper,
            owner: Some(P1),
            pos: Vec2::new(300.0, 300.0),
            seen: Sighting::object_at(Some(P1), Hiding::default(), Vec2::new(300.0, 300.0)),
        }]
    );
    let guard_bars = bars(&app);
    assert_eq!(guard_bars.len(), 1, "the Guard's");
    assert_eq!(guard_bars[0].fraction, 53.0 / 80.0);

    run_until(
        &mut app,
        death + space_crystals::tuning::feedback::BUILTIN.death_marker_ticks,
    );
    assert!(effects(&app).deaths.is_empty(), "faded");
    assert!(effects_visible(&mut app), "the Guard's HP bar stays");
}

/// Slice M3-B (`systems/combat.md > Shields > Display`, design change 0148: "a shield bar sits directly above the HP
/// bar of every object with Shields"): an object that has Shields gets a shield bar with its HP bar, filled with the
/// share of its MaxShields it has left, and shows them while its Shields alone are short; one without Shields has no
/// shield bar. Player 2's Peacekeeper shoots player 1's Sabre Legionnaire, 160 su off: its 10 take the Sabre's 10
/// Shields and no HP, so the Sabre's bars show with its HP full, and 0.25 of the Shields back by the end of the tick
/// (`systems/combat.md > ShieldRegenRate`). Once the Peacekeeper stops, the Shields regenerate to full and the bars go;
/// player 2's Guard, which stands by, selected, shows its HP bar alone. (A Guard's 18, design change 0186, would take
/// HP too.)
#[test]
fn an_object_with_shields_shows_a_shield_bar_above_its_hp_bar() {
    const SABRE: ObjectId = ObjectId(1);
    const SHOOTER: ObjectId = ObjectId(3);
    let mut app = headless_app();
    submit(
        &mut app,
        P1,
        Command::Debug(DebugCommand::SetAutoTargeting { on: false }),
    );
    app.update();
    spawn(&mut app, ObjectKind::SabreLegionnaire, P1, Vec2::new(300.0, 300.0));
    spawn(&mut app, ObjectKind::Guard, P2, Vec2::new(460.0, 300.0));
    spawn(&mut app, ObjectKind::Peacekeeper, P2, Vec2::new(460.0, 380.0));
    app.update();
    let spawned = tick(&app);
    assert!(bars(&app).is_empty(), "full HP and Shields");
    attack(&mut app, P2, SHOOTER, SABRE);
    run_until(&mut app, spawned + 3);
    let shown: Vec<(f32, Option<f32>)> = bars(&app).iter().map(|bar| (bar.fraction, bar.shields)).collect();
    assert_eq!(shown, [(1.0, Some(0.25 / 10.0))], "the Sabre's, its HP full");

    let stop = Command::Order {
        units: vec![SHOOTER],
        kind: OrderKind::Stop,
        queue: false,
    };
    submit(&mut app, P2, stop);
    run_until(&mut app, spawned + 3 + 40);
    assert!(bars(&app).is_empty(), "the Shields are full again");
    app.world_mut().resource_mut::<ControlStates>().of_mut(P1).selection = Selection::new([(GUARD, ObjectKind::Guard)]);
    let shown: Vec<(f32, Option<f32>)> = bars(&app).iter().map(|bar| (bar.fraction, bar.shields)).collect();
    assert_eq!(shown, [(1.0, None)]);
}

/// Slice M5-N4a (`factions/cults/units/disciple.md > Placeholder stats`: "Energy - MaxEnergy 200; starts at 50;
/// regenerates 0.5 per second"): a unit with an energy pool gets an energy bar with its HP bar, under it, filled with
/// the share of its MaxEnergy it has: a Disciple's 50 of 200 as it comes into the game, 50.5 a second later. Its bars
/// show as the HP bar does, while it is selected or damaged: energy short of its maximum isn't damage (Placeholder, not
/// in the design). A unit without an energy pool, a Peacekeeper beside it, selected too, has no energy bar.
#[test]
fn a_disciple_shows_an_energy_bar_under_its_hp_bar() {
    const DISCIPLE: ObjectId = ObjectId(1);
    let mut app = headless_app();
    app.update();
    spawn(&mut app, ObjectKind::Disciple, P1, Vec2::new(300.0, 300.0));
    spawn(&mut app, ObjectKind::Peacekeeper, P1, Vec2::new(400.0, 300.0));
    app.update();
    let spawned = tick(&app);
    assert!(bars(&app).is_empty(), "full HP, and none selected");
    let both = Selection::new([(DISCIPLE, ObjectKind::Disciple), (ObjectId(2), ObjectKind::Peacekeeper)]);
    app.world_mut().resource_mut::<ControlStates>().of_mut(P1).selection = both;
    let shown =
        |app: &App| -> Vec<(f32, Option<f32>)> { bars(app).iter().map(|bar| (bar.fraction, bar.energy)).collect() };
    assert_eq!(shown(&app), [(1.0, Some(50.0 / 200.0)), (1.0, None)]);
    run_until(&mut app, spawned + 16);
    assert_eq!(shown(&app), [(1.0, Some(50.5 / 200.0)), (1.0, None)]);
}

/// (M6-GH1) An aircraft's ammo shows as pips under its HP bar (plan §GH1: "the Hornet's ammo pips"; Placeholder, not
/// in the design's art): a Hornet's missiles left of its 5 (`factions/gdo/units/hornet.md > AttackAttributes`: "Ammo: **5
/// missiles**"), 2 once it has fired three at a Prospector. Its bars show as the HP bar does, while it is selected or
/// damaged; a Peacekeeper beside it, selected too, has no pips.
#[test]
fn a_hornet_shows_its_missiles_left_as_pips_under_its_hp_bar() {
    const HORNET: ObjectId = ObjectId(1);
    let mut app = headless_app();
    submit(
        &mut app,
        P1,
        Command::Debug(DebugCommand::SetAutoTargeting { on: false }),
    );
    app.update();
    spawn(&mut app, ObjectKind::Hornet, P1, Vec2::new(300.0, 300.0));
    spawn(&mut app, ObjectKind::Peacekeeper, P1, Vec2::new(300.0, 420.0));
    spawn(&mut app, ObjectKind::Prospector, P2, Vec2::new(500.0, 300.0));
    app.update();
    let both = Selection::new([(HORNET, ObjectKind::Hornet), (ObjectId(2), ObjectKind::Peacekeeper)]);
    app.world_mut().resource_mut::<ControlStates>().of_mut(P1).selection = both;
    let pips = |app: &App| -> Vec<Option<(u32, u32)>> { bars(app).iter().map(|bar| bar.ammo).collect() };
    assert_eq!(pips(&app), [Some((5, 5)), None]);
    attack(&mut app, P1, HORNET, ObjectId(3));
    let ammo = |app: &App| {
        let sim = app.world().resource::<SimRes>();
        sim.object(HORNET).and_then(|o| o.aircraft).map(|a| a.ammo)
    };
    let until = tick(&app) + 200;
    while ammo(&app) != Some(2) {
        assert!(tick(&app) < until, "three missiles");
        let next = tick(&app) + 1;
        run_until(&mut app, next);
    }
    assert_eq!(pips(&app), [Some((2, 5)), None, None], "and the Prospector, damaged");
}

/// (M6-ST1) A carrier's passengers show as pips over its HP bar, to its owner (plan §ST1: "the passenger pips";
/// Placeholder, not in the design's art): a Subterranean Transport's places taken of its 4
/// (`factions/syndicate/units/subterranean_transport.md > Placeholder stats`: "**Capacity 4** infantry"), none and then
/// 2 once two Guards have boarded it. Its bars show as the HP bar does, while it is selected or damaged; player 2's
/// Transport, which player 1 sees and selects, shows player 1 no pips, nor does a Guard.
#[test]
fn a_transport_shows_its_passengers_as_pips_over_its_hp_bar_to_its_owner() {
    const TRANSPORT: ObjectId = ObjectId(1);
    let mut app = headless_app();
    submit(
        &mut app,
        P1,
        Command::Debug(DebugCommand::SetAutoTargeting { on: false }),
    );
    app.update();
    spawn(&mut app, ObjectKind::SubterraneanTransport, P1, Vec2::new(300.0, 300.0));
    spawn(&mut app, ObjectKind::Guard, P1, Vec2::new(300.0, 420.0));
    spawn(&mut app, ObjectKind::Guard, P1, Vec2::new(380.0, 420.0));
    spawn(&mut app, ObjectKind::SubterraneanTransport, P2, Vec2::new(520.0, 300.0));
    app.update();
    let selected = Selection::new([
        (TRANSPORT, ObjectKind::SubterraneanTransport),
        (ObjectId(2), ObjectKind::Guard),
        (ObjectId(4), ObjectKind::SubterraneanTransport),
    ]);
    app.world_mut().resource_mut::<ControlStates>().of_mut(P1).selection = selected;
    let pips = |app: &App| -> Vec<Option<(u32, u32)>> { bars(app).iter().map(|bar| bar.passengers).collect() };
    assert_eq!(pips(&app), [Some((0, 4)), None, None]);
    let board = Command::Order {
        units: vec![ObjectId(2), ObjectId(3)],
        kind: OrderKind::Enter(TRANSPORT),
        queue: false,
    };
    submit(&mut app, P1, board);
    let room = |app: &App| app.world().resource::<SimRes>().room(TRANSPORT);
    let until = tick(&app) + 200;
    while room(&app) > 2 {
        assert!(tick(&app) < until, "two board");
        let next = tick(&app) + 1;
        run_until(&mut app, next);
    }
    assert_eq!(pips(&app), [Some((2, 4)), None], "the boarded Guard off the map");
}

/// A structure's bars lie along the top edge of its footprint, as wide as it (placeholder look; slice M3-K2's
/// walkthrough found them unit-sized, inside the building). Player 1's Barracks, 3 × 2 tiles, selected, shows its HP
/// bar 192 su wide, centred over it, just above its top edge, clear of its selection frame, which lies inside it.
#[test]
fn a_structure_shows_its_bar_along_the_top_of_its_footprint() {
    const BARRACKS: ObjectId = ObjectId(1);
    let mut app = headless_app();
    app.update();
    let spawn = DebugCommand::Spawn {
        kind: ObjectKind::Barracks,
        owner: Some(P1),
        at: SpawnAt::Tile(TilePos::new(4, 4)),
        orientation: Default::default(),
    };
    submit(&mut app, P1, Command::Debug(spawn));
    app.update();
    app.world_mut().resource_mut::<ControlStates>().of_mut(P1).selection =
        Selection::new([(BARRACKS, ObjectKind::Barracks)]);
    let sim = app.world().resource::<SimRes>();
    let barracks = sim.object(BARRACKS).expect("the Barracks");
    let placement = barracks.placement.expect("a tile-based object");
    assert_eq!((placement.w, placement.h), (3, 2));
    let drawn = drawn_at_height(barracks.pos, object_height(sim.map(), barracks.pos, Some(&placement)));
    let top = drawn.y - 2.0 * SU_PER_GU / 2.0;

    let shown = bars(&app);
    let [bar] = shown.as_slice() else {
        panic!("one bar: {shown:?}");
    };
    assert_eq!((bar.width, bar.center.x, bar.fraction), (3.0 * SU_PER_GU, drawn.x, 1.0));
    assert!(
        (top - 16.0..top).contains(&bar.center.y),
        "just above the top edge, at {}: {}",
        top,
        bar.center.y
    );
}

/// Design change 0207: every Syndicate laser is drawn red, whoever fires it, and other shots as tracers in their
/// shooter's colour. Player 1 (blue) has a Guard and an Agent; both attack player 2's Peacekeeper. The Guard's laser
/// rifle is a red laser, drawn red; the Agent's pistol, its first shot 4 ticks after the order (Aim 3, Fire 1), is a
/// tracer like the Peacekeeper's. The Laser Turret's laser is red too (slice M4-S1).
#[test]
fn a_guards_laser_is_drawn_red() {
    // In the order they are spawned.
    let (shooter, target, agent) = (ObjectId(1), ObjectId(2), ObjectId(3));
    let mut app = headless_app();
    submit(
        &mut app,
        P1,
        Command::Debug(DebugCommand::SetAutoTargeting { on: false }),
    );
    app.update();
    spawn(&mut app, ObjectKind::Guard, P1, Vec2::new(300.0, 300.0));
    spawn(&mut app, ObjectKind::Peacekeeper, P2, Vec2::new(460.0, 300.0));
    spawn(&mut app, ObjectKind::Agent, P1, Vec2::new(460.0, 200.0));
    app.update();
    let spawned = tick(&app);
    attack(&mut app, P1, agent, target);
    run_until(&mut app, spawned + 4);
    let red = |app: &mut App| {
        let mut query = app.world_mut().query_filtered::<&Mesh2d, With<CombatEffectsMesh>>();
        let mesh = query.single(app.world()).unwrap().0.clone();
        let meshes = app.world().resource::<Assets<Mesh>>();
        let Some(VertexAttributeValues::Float32x4(colors)) =
            meshes.get(&mesh).unwrap().attribute(Mesh::ATTRIBUTE_COLOR)
        else {
            panic!("no colours");
        };
        colors.iter().any(|c| c[0] > 0.9 && c[1] < 0.1 && c[3] > 0.0)
    };
    let tracers = &effects(&app).tracers;
    let [shot] = tracers.as_slice() else {
        panic!("the Agent's shot: {tracers:?}");
    };
    assert_eq!((shot.attacker, shot.weapon), (agent, Some(ObjectKind::Agent)));
    assert_eq!(ShotStyle::of(shot.weapon), ShotStyle::Tracer, "a pistol");
    assert!(!red(&mut app), "player 1's blue");

    attack(&mut app, P1, shooter, target);
    run_until(&mut app, spawned + 4 + 5);
    let tracers = &effects(&app).tracers;
    let shot = tracers
        .iter()
        .find(|t| t.attacker == shooter)
        .expect("the Guard's shot");
    assert_eq!((shot.weapon, shot.owner), (Some(ObjectKind::Guard), Some(P1)));
    assert_eq!(ShotStyle::of(shot.weapon), ShotStyle::RedLaser);
    assert!(red(&mut app), "drawn red, not in player 1's blue");
    for kind in [ObjectKind::Peacekeeper, ObjectKind::SabreLegionnaire, ObjectKind::Agent] {
        assert_eq!(ShotStyle::of(Some(kind)), ShotStyle::Tracer, "{kind:?}");
    }
    assert_eq!(ShotStyle::of(Some(ObjectKind::LaserTurret)), ShotStyle::RedLaser);
}

/// Slice M4-U: the Grenadier's grenade is no laser (design change 0207 makes the Syndicate's lasers red) and no shot:
/// a projectile in its thrower's style ([`ShotStyle::Grenade`], plan decision T52: "The styles come from each kind's
/// client module"), drawn from its launch to where it lands, and then a blast there. Player 1's Grenadier attacks
/// player 2's Peacekeeper, 160 su east: its grenade leaves at the end of its Firing (Aim 6, Firing 2) and lands 20
/// ticks later (160 su at 8 su per tick); it leaves no tracer, and the projectile shows to player 1, who sees it fly.
/// Where it lands a blast shows, of the grenade's radius (`factions/syndicate/units/grenadier.md`: "AoE radius: 40
/// space units"), and the Peacekeeper takes its hit.
#[test]
fn a_grenadiers_grenade_flies_as_a_grenade_and_blasts_where_it_lands() {
    let (thrower, target) = (ObjectId(1), ObjectId(2));
    let mut app = headless_app();
    submit(
        &mut app,
        P1,
        Command::Debug(DebugCommand::SetAutoTargeting { on: false }),
    );
    app.update();
    let (from, at) = (Vec2::new(300.0, 300.0), Vec2::new(460.0, 300.0));
    spawn(&mut app, ObjectKind::Grenadier, P1, from);
    spawn(&mut app, ObjectKind::Peacekeeper, P2, at);
    app.update();
    attack(&mut app, P1, thrower, target);
    let ordered = tick(&app);
    while effects(&app).flights.is_empty() {
        assert!(tick(&app) < ordered + 20, "no grenade was thrown");
        app.update();
    }
    let flight = effects(&app).flights[0].clone();
    assert_eq!(
        (flight.shooter, flight.weapon, flight.owner, flight.target),
        (thrower, Some(ObjectKind::Grenadier), Some(P1), Some(target))
    );
    assert_eq!((flight.from, flight.to, flight.lands_at), (from, at, flight.tick + 20));
    assert_eq!(ShotStyle::of(flight.weapon), ShotStyle::Grenade);
    run_until(&mut app, flight.tick + 10);
    {
        let world = app.world();
        let sim = world.resource::<SimRes>();
        let sight = Sight::new(sim, P1, world.resource::<FogOfWar>().enabled);
        let progress = flight.progress(sim, 1.0);
        assert_eq!(progress, 0.5);
        assert!(shows_flight(sim, &sight, &flight, progress), "player 1 sees it fly");
    }
    assert!(effects_visible(&mut app), "the grenade in the air");
    run_until(&mut app, flight.lands_at);
    let effects = effects(&app);
    assert!(effects.flights.is_empty(), "landed");
    assert!(effects.tracers.is_empty(), "no shot");
    let blast = effects.blasts.first().expect("the blast");
    assert_eq!((blast.tick, blast.at, blast.radius), (flight.lands_at, at, 40.0));
    assert_eq!((blast.weapon, blast.owner), (Some(ObjectKind::Grenadier), Some(P1)));
    assert_eq!(
        ring_color(blast),
        Color::from(Srgba::rgb(1.0, 0.55, 0.15)),
        "a fire's orange"
    );
    let hp = app.world().resource::<SimRes>().object(target).and_then(|o| o.hp);
    assert!(hp < ObjectKind::Peacekeeper.def().max_hp, "hit: {hp:?}");
}

/// Whether the effects mesh has a vertex of `color`, up to its alpha.
fn drawn_in(app: &mut App, color: Color) -> bool {
    let mut query = app.world_mut().query_filtered::<&Mesh2d, With<CombatEffectsMesh>>();
    let mesh = query.single(app.world()).unwrap().0.clone();
    let meshes = app.world().resource::<Assets<Mesh>>();
    let Some(VertexAttributeValues::Float32x4(colors)) = meshes.get(&mesh).unwrap().attribute(Mesh::ATTRIBUTE_COLOR)
    else {
        panic!("no colours");
    };
    let rgb = color.to_linear().to_f32_array();
    (colors.iter()).any(|c| c[3] > 0.0 && (0..3).all(|i| (c[i] - rgb[i]).abs() < 1e-4))
}

/// (M5-S8) Design change 0445: the Rocketman's missiles burst as area damage (`factions/gdo/units/rocketman.md`: "an
/// AoERadius of **32 su**"), each burst the grenade's blast at that radius, its ring in its shooter's colour. Player
/// 1's Rocketman attacks player 2's Peacekeeper, 256 su east: its missile flies 16 ticks (16 su a tick) and bursts
/// there, on the ground, a ring in player 1's blue. Then it locks onto player 2's Sparrow, circling far off: its locked
/// missile, whose flight the client keeps none of (it homes), bursts on the Sparrow, in the air at its flight, in the
/// shooter's blue again.
#[test]
fn a_rocketmans_missile_bursts_in_a_ring_of_its_shooters_colour() {
    let (shooter, target, sparrow) = (ObjectId(1), ObjectId(2), ObjectId(3));
    let mut app = headless_app();
    submit(
        &mut app,
        P1,
        Command::Debug(DebugCommand::SetAutoTargeting { on: false }),
    );
    app.update();
    let (from, at) = (Vec2::new(300.0, 300.0), Vec2::new(556.0, 300.0));
    spawn(&mut app, ObjectKind::Rocketman, P1, from);
    spawn(&mut app, ObjectKind::Peacekeeper, P2, at);
    spawn(&mut app, ObjectKind::Sparrow, P2, Vec2::new(300.0, 560.0));
    app.update();
    attack(&mut app, P1, shooter, target);
    let ordered = tick(&app);
    while effects(&app).flights.is_empty() {
        assert!(tick(&app) < ordered + 20, "no missile was fired");
        app.update();
    }
    let flight = effects(&app).flights[0].clone();
    assert_eq!(ShotStyle::of(flight.weapon), ShotStyle::Missile);
    assert_eq!((flight.to, flight.lands_at), (at, flight.tick + 16));
    run_until(&mut app, flight.lands_at);
    let blast = effects(&app).blasts.first().cloned().expect("the burst");
    assert_eq!((blast.tick, blast.at, blast.radius), (flight.lands_at, at, 32.0));
    assert_eq!(
        (blast.weapon, blast.owner, blast.lift),
        (Some(ObjectKind::Rocketman), Some(P1), 0.0),
        "on the ground"
    );
    let blue = player_color(Some(P1));
    assert_eq!(ring_color(&blast), blue);
    assert!(
        drawn_in(&mut app, blue.with_alpha(0.9)),
        "its ring drawn in player 1's blue"
    );

    let lock = Command::Order {
        units: vec![shooter],
        kind: OrderKind::TargetLock(sparrow),
        queue: false,
    };
    submit(&mut app, P1, lock);
    let locked = tick(&app);
    let burst = |app: &App| (effects(app).blasts.iter()).find(|b| b.tick > flight.lands_at).cloned();
    while burst(&app).is_none() {
        assert!(tick(&app) < locked + 120, "no locked missile burst");
        app.update();
    }
    let blast = burst(&app).unwrap();
    assert_eq!(blast.radius, 32.0);
    assert_eq!((blast.weapon, blast.owner), (Some(ObjectKind::Rocketman), Some(P1)));
    assert_eq!(
        blast.lift,
        flight_lift(ObjectKind::Sparrow, &space_crystals::tuning::look::BUILTIN),
        "in the air, at its flight"
    );
    assert!(blast.lift > 0.0);
    assert_eq!(ring_color(&blast), blue);
}

/// The feedback message that shows, if any.
fn feedback(app: &App) -> Option<&'static str> {
    let feedback = app.world().resource::<TargetFeedback>();
    feedback.since.map(|_| feedback.text)
}

/// Gives the client `event` as the sim's of the tick that runs next, as the sim driver forwards them.
fn sim_event(app: &mut App, event: SimEvent) {
    let tick = tick(app) + 1;
    app.world_mut()
        .resource_mut::<Messages<SimEventMessage>>()
        .write(SimEventMessage { tick, event });
}

/// Design change 0182 (`systems/controls/command_behaviors.md > AttackingObject > Target inside MinRange`): an Attack
/// its unit can't carry out, the target inside the unit's MinRange and the unit unable to move (a deployed Artillery
/// Rig), fails "with feedback ("Target too close"; same channel as other refusals)": the feedback band shows it, for
/// `FeedbackTuning::feedback_ticks`, when it is the local player's unit, and not for another player's. No type has a
/// MinRange yet, so the sim's event is handed to the client as the sim driver would.
#[test]
fn an_attack_on_a_target_too_close_shows_its_feedback() {
    let mut app = headless_app();
    submit(
        &mut app,
        P1,
        Command::Debug(DebugCommand::SetAutoTargeting { on: false }),
    );
    app.update();
    spawn(&mut app, ObjectKind::Peacekeeper, P1, Vec2::new(300.0, 300.0));
    spawn(&mut app, ObjectKind::Guard, P2, Vec2::new(340.0, 300.0));
    app.update();
    assert_eq!(app.world().resource::<LocalPlayer>().0, P1);
    assert_eq!(feedback(&app), None);

    sim_event(
        &mut app,
        SimEvent::TargetTooClose {
            unit: GUARD,
            target: PEACEKEEPER,
        },
    );
    app.update();
    assert_eq!(feedback(&app), None, "player 2's Guard: not for player 1");

    sim_event(
        &mut app,
        SimEvent::TargetTooClose {
            unit: PEACEKEEPER,
            target: GUARD,
        },
    );
    app.update();
    let shown = tick(&app);
    assert_eq!(feedback(&app), Some(TARGET_TOO_CLOSE_TEXT));
    assert_eq!(TARGET_TOO_CLOSE_TEXT, "Target too close", "the design's wording");
    run_until(
        &mut app,
        shown + space_crystals::tuning::feedback::BUILTIN.feedback_ticks - 1,
    );
    assert_eq!(feedback(&app), Some(TARGET_TOO_CLOSE_TEXT));
    app.update();
    assert_eq!(feedback(&app), None, "gone after feedback_ticks");
}
