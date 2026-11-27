//! The minimap's alerts (slice M5-MM2, design change 0443, `systems/controls/hud.md > Minimap > Alerts`), in a
//! headless game, one tick an update, as player 1: a hit on one of their buildings off screen pings at its spot and
//! one on screen doesn't; a fight goes on pinging once every 10 s, and two fights far apart ping each; the ring shrinks
//! and fades and is gone after 2 s; Space centres the camera on the latest alert, and does nothing before the first;
//! and a restart (the playtest menu's) leaves no alert of the old game.

use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use sc_sim::testing::ScenarioBuilder;
use sc_sim::{Command, DebugCommand, ObjectId, ObjectKind, OrderKind, PlayerCommand, PlayerId, SpawnAt, Tick, TilePos};
use space_crystals::camera::{
    CameraView, ViewportInsets, clamp_center, map_extent_su, viewport_rect, visible_extent_su,
};
use space_crystals::coords::{drawn_bounds, drawn_position};
use space_crystals::hud::minimap::{MinimapAlerts, MinimapFrame, Ping, PingRing};
use space_crystals::input::InjectedInput;
use space_crystals::platform::{DisplayMode, SurfaceSize};
use space_crystals::playtest_menu::{MenuButton, MenuLayout, PlaytestMenu};
use space_crystals::sim_driver::{PendingCommands, SimRes};
use space_crystals::tuning::feedback::{PING_THROTTLE_TICKS, PING_TICKS};
use space_crystals::{AppConfig, build_app};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
/// Player 1's Generator in the north-east, and player 2's Peacekeeper 6 grid units west of it.
const EAST_GENERATOR: ObjectId = ObjectId(1);
const EAST_ATTACKER: ObjectId = ObjectId(2);
/// Player 1's Generator in the south-west, and player 2's Peacekeeper 6 grid units east of it.
const WEST_GENERATOR: ObjectId = ObjectId(3);
const WEST_ATTACKER: ObjectId = ObjectId(4);
/// The map's middle, where the camera looks unless a test moves it: both Generators are off screen.
const MIDDLE: Vec2 = Vec2::new(2048.0, 1536.0);

/// A flat 64 × 48 map, its four objects as above, each attacker beyond its sight of the Generator, so that only an
/// order starts a fight, even with auto-targeting on (a restarted game's); and a Field Engineer of player 2's beside
/// each Generator, with no weapon, so that player 2 sees the Generators to order an attack on them. Player 1's view,
/// debug commands on, the fog on.
fn game() -> App {
    let tile = |x, y| SpawnAt::Tile(TilePos::new(x, y));
    let point = |x: i32, y: i32| SpawnAt::Point(TilePos::new(x, y).center());
    let scenario = ScenarioBuilder::new(&vec![".".repeat(64); 48].join("\n"))
        .spawn(ObjectKind::Generator, Some(P1), tile(52, 6))
        .spawn(ObjectKind::Peacekeeper, Some(P2), point(45, 7))
        .spawn(ObjectKind::Generator, Some(P1), tile(8, 38))
        .spawn(ObjectKind::Peacekeeper, Some(P2), point(16, 39))
        .spawn(ObjectKind::FieldEngineer, Some(P2), point(52, 10))
        .spawn(ObjectKind::FieldEngineer, Some(P2), point(8, 36))
        .build();
    let mut config = AppConfig::new(scenario);
    config.display = DisplayMode::Headless;
    let mut app = build_app(config);
    app.insert_resource(TimeUpdateStrategy::FixedTimesteps(1));
    app.finish();
    app.cleanup();
    submit(
        &mut app,
        P1,
        Command::Debug(DebugCommand::SetAutoTargeting { on: false }),
    );
    app.update();
    look_at(&mut app, MIDDLE);
    app.update();
    app
}

fn submit(app: &mut App, player: PlayerId, command: Command) {
    let command = PlayerCommand { player, command };
    app.world_mut().resource_mut::<PendingCommands>().0.push(command);
}

/// Player 2's `attacker` attacks `target`, from the next tick.
fn attack(app: &mut App, attacker: ObjectId, target: ObjectId) {
    let kind = OrderKind::Attack(target);
    let order = Command::Order {
        units: vec![attacker],
        kind,
        queue: false,
    };
    submit(app, P2, order);
}

fn tick(app: &App) -> Tick {
    app.world().resource::<SimRes>().tick()
}

fn hp(app: &App, id: ObjectId) -> f32 {
    let sim = app.world().resource::<SimRes>();
    sim.object(id).and_then(|o| o.hp).expect("standing")
}

fn pos(app: &App, id: ObjectId) -> Vec2 {
    app.world().resource::<SimRes>().object(id).expect("standing").pos
}

fn alerts(app: &App) -> &MinimapAlerts {
    app.world().resource::<MinimapAlerts>()
}

/// Updates until `done`, at most `ticks` more ticks; panics if it never is.
fn run_until(app: &mut App, ticks: Tick, mut done: impl FnMut(&App) -> bool) {
    let until = tick(app) + ticks;
    while !done(app) {
        assert!(tick(app) < until, "not by tick {until}");
        app.update();
    }
}

/// Centres the camera on the view point `view`, kept on the map from the next frame on.
fn look_at(app: &mut App, view: Vec2) {
    app.world_mut().resource_mut::<CameraView>().center = view;
}

fn camera(app: &App) -> Vec2 {
    app.world().resource::<CameraView>().center
}

/// Where the camera looks once centred on the ground point `ground`: at its drawn position, kept on the map
/// (`clamp_center`) as the camera keeps every view.
fn centred_on(app: &App, ground: Vec2) -> Vec2 {
    let size = *app.world().resource::<SurfaceSize>();
    let insets = *app.world().resource::<ViewportInsets>();
    let map = app.world().resource::<SimRes>().map();
    let (_, viewport) = viewport_rect(size, insets);
    let visible = visible_extent_su(viewport, size.physical.x, 1.0);
    clamp_center(drawn_position(map, ground), visible, drawn_bounds(map))
}

fn frame(app: &App) -> MinimapFrame {
    let size = *app.world().resource::<SurfaceSize>();
    MinimapFrame::on(size, map_extent_su(app.world().resource::<SimRes>().map()))
}

/// The rings shown: each one's rectangle on the surface (logical pixels) and its alpha.
fn rings(app: &mut App) -> Vec<(Rect, f32)> {
    let origin = frame(app).area.min;
    let px = |v: Val| match v {
        Val::Px(px) => px,
        other => panic!("{other:?}"),
    };
    let world = app.world_mut();
    let mut rings = world.query_filtered::<(&Node, &BorderColor), With<PingRing>>();
    (rings.iter(world))
        .filter(|(node, _)| node.display != Display::None)
        .map(|(node, border)| {
            let at = origin + Vec2::new(px(node.left), px(node.top));
            let rect = Rect::from_corners(at, at + Vec2::new(px(node.width), px(node.height)));
            (rect, border.top.alpha())
        })
        .collect()
}

/// Taps Space, for a frame.
fn space(app: &mut App) {
    app.world_mut().resource_mut::<InjectedInput>().tap_key(KeyCode::Space);
    app.update();
}

/// Updates until the next ping, at most 200 ticks on, and returns it.
fn next_ping(app: &mut App) -> Ping {
    let made = alerts(app).made;
    run_until(app, 200, |app| alerts(app).made > made);
    *alerts(app).pings().last().unwrap()
}

/// Change 0443: "takes damage while it is off screen (outside the camera's view)" pings "on the minimap at that
/// spot": the Generator's first hit, with the camera on the map's middle, pings at its spot in the tick of the hit,
/// and its ring shows there on the minimap. With the camera on the Generator, the same fight pings never.
#[test]
fn a_hit_off_screen_pings_there_and_one_on_screen_doesnt() {
    let mut app = game();
    attack(&mut app, EAST_ATTACKER, EAST_GENERATOR);
    let full = hp(&app, EAST_GENERATOR);
    run_until(&mut app, 200, |app| hp(app, EAST_GENERATOR) < full);
    let at = pos(&app, EAST_GENERATOR);
    let expected = Ping {
        player: P1,
        at,
        tick: tick(&app),
    };
    assert_eq!(alerts(&app).pings(), [expected], "the hit's tick, at the Generator");
    let shown = rings(&mut app);
    assert_eq!(shown.len(), 1);
    let on_minimap = frame(&app).screen_at(at);
    assert!(
        shown[0].0.center().distance(on_minimap) < 1.0,
        "{shown:?} at {on_minimap}"
    );

    let mut app = game();
    let view = centred_on(&app, pos(&app, EAST_GENERATOR));
    look_at(&mut app, view);
    attack(&mut app, EAST_ATTACKER, EAST_GENERATOR);
    run_until(&mut app, 200, |app| hp(app, EAST_GENERATOR) < full);
    for _ in 0..60 {
        app.update();
    }
    assert!(hp(&app, EAST_GENERATOR) < full - 10.0, "hit again since");
    assert_eq!(alerts(&app).made, 0, "on screen: no ping");
    assert!(rings(&mut app).is_empty());
}

/// "At most one ping per area every 10 s": the Generator, hit about once a second for 26 s off screen, pings at its
/// first hit, then at the first hit 10 s on, and so on; never sooner. The other Generator, 50 grid units away, hit
/// meanwhile, pings too, within those 10 s.
#[test]
fn a_fight_pings_once_every_ten_seconds_and_two_fights_far_apart_each_ping() {
    let mut app = game();
    attack(&mut app, EAST_ATTACKER, EAST_GENERATOR);
    attack(&mut app, WEST_ATTACKER, WEST_GENERATOR);
    let (east, west) = (pos(&app, EAST_GENERATOR), pos(&app, WEST_GENERATOR));
    let mut made: Vec<Ping> = Vec::new();
    let mut hits = Vec::new();
    let mut east_hp = hp(&app, EAST_GENERATOR);
    for _ in 0..(2 * PING_THROTTLE_TICKS + 100) {
        app.update();
        if hp(&app, EAST_GENERATOR) < east_hp {
            hits.push(tick(&app));
            east_hp = hp(&app, EAST_GENERATOR);
        }
        for ping in alerts(&app).pings() {
            if !made.contains(ping) {
                made.push(*ping);
            }
        }
    }
    assert!(hits.len() > 15, "hit about once a second: {hits:?}");
    let east_pings: Vec<Tick> = made.iter().filter(|p| p.at == east).map(|p| p.tick).collect();
    let west_pings: Vec<Tick> = made.iter().filter(|p| p.at == west).map(|p| p.tick).collect();
    assert_eq!(made.len(), east_pings.len() + west_pings.len(), "{made:?}");
    // The first hit, then each first hit 10 s or more after the ping before.
    let mut expected = vec![hits[0]];
    for &hit in &hits {
        if hit - expected.last().unwrap() >= PING_THROTTLE_TICKS {
            expected.push(hit);
        }
    }
    assert_eq!(east_pings, expected, "the hits: {hits:?}");
    assert_eq!(east_pings.len(), 3);
    assert!(
        west_pings.len() == 3 && west_pings[0].abs_diff(east_pings[0]) < PING_THROTTLE_TICKS,
        "{west_pings:?}"
    );
}

/// "A ring that shrinks and fades over about 2 s": from the frame of its hit, the ring is smaller or as large, and
/// clearer, every frame, then gone, [`PING_TICKS`] ticks on (a frame later as the frames are drawn here: between the
/// tick before and the tick, `RenderAlpha` 0).
#[test]
fn the_ring_shrinks_and_fades_and_is_gone_after_two_seconds() {
    let mut app = game();
    attack(&mut app, EAST_ATTACKER, EAST_GENERATOR);
    let ping = next_ping(&mut app);
    let mut drawn = Vec::new();
    loop {
        let shown = rings(&mut app);
        assert!(shown.len() <= 1);
        let Some(&(rect, alpha)) = shown.first() else {
            break;
        };
        drawn.push((tick(&app) - ping.tick, rect.width(), alpha));
        app.update();
        assert!(tick(&app) <= ping.tick + PING_TICKS + 1, "gone by then: {drawn:?}");
    }
    let (_, first_size, first_alpha) = drawn[0];
    assert!(first_size > 20.0 && first_alpha == 1.0, "{drawn:?}");
    for pair in drawn.windows(2) {
        let [(_, size, alpha), (age, next_size, next_alpha)] = pair else {
            unreachable!()
        };
        assert!(next_size <= size, "shrinks: {pair:?}");
        assert!(next_alpha < alpha || *age <= 1, "fades: {pair:?}");
    }
    let (last_age, last_size, last_alpha) = *drawn.last().unwrap();
    assert!(
        last_age >= PING_TICKS - 1 && last_size < 10.0 && last_alpha < 0.05,
        "{drawn:?}"
    );
    assert!(alerts(&app).pings().contains(&ping), "kept for its 10 s");
}

/// "Jump to the latest alert: Space ... centres the camera on the latest alert's spot (snap, as Centering)": nothing
/// before the first ping; then the camera on the latest ping's spot, the other Generator's, hit later, as near as the
/// camera goes (on the map's edge); and again once panned away.
#[test]
fn space_centres_the_camera_on_the_latest_alert() {
    let mut app = game();
    space(&mut app);
    assert_eq!(camera(&app), MIDDLE, "no alert yet");
    attack(&mut app, EAST_ATTACKER, EAST_GENERATOR);
    let first = next_ping(&mut app);
    assert_eq!(first.at, pos(&app, EAST_GENERATOR));
    attack(&mut app, WEST_ATTACKER, WEST_GENERATOR);
    let latest = next_ping(&mut app);
    assert_eq!(latest.at, pos(&app, WEST_GENERATOR));
    assert_eq!(camera(&app), MIDDLE);
    space(&mut app);
    let west = centred_on(&app, latest.at);
    assert_eq!(camera(&app), west, "snapped onto the latest alert");
    look_at(&mut app, MIDDLE);
    app.update();
    space(&mut app);
    assert_eq!(camera(&app), west, "again");
}

/// The playtest menu's Restart starts the game over with no alerts: no ping, none shown, and Space does nothing.
#[test]
fn a_restart_leaves_no_alerts() {
    let mut app = game();
    attack(&mut app, EAST_ATTACKER, EAST_GENERATOR);
    next_ping(&mut app);
    assert_eq!(rings(&mut app).len(), 1);
    app.world_mut().resource_mut::<InjectedInput>().tap_key(KeyCode::F10);
    app.update();
    let size = app.world().resource::<SurfaceSize>().logical();
    let menu = *app.world().resource::<PlaytestMenu>();
    let layout = MenuLayout::of(menu, size).expect("the menu is open");
    let restart = layout.button(MenuButton::Restart).expect("Restart").center();
    let mut input = app.world_mut().resource_mut::<InjectedInput>();
    input.cursor = Some(restart);
    input.mouse.press(MouseButton::Left);
    input.mouse.release(MouseButton::Left);
    app.update();
    app.update();
    assert!(tick(&app) < 5, "started over");
    let alerts = alerts(&app);
    assert_eq!((alerts.pings(), alerts.latest(P1), alerts.made), (&[][..], None, 0));
    assert!(rings(&mut app).is_empty());
    let view = camera(&app);
    space(&mut app);
    assert_eq!(camera(&app), view, "no alert to jump to");
}
