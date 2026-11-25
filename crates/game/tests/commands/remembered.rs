//! Right-clicks on what the player remembers and rally points by the units' own default commands (slice M3-R):
//! remembered structures and objects "can be targeted by right-click and by targeted commands (Attack, Gather, Move to
//! object, rally points), but are still not selectable", and the order goes to their last-known position (design
//! change 0139, `core/vision.md > Explored > Targeting remembered objects`); a rally point gives a new unit its own
//! default right-click command on the rally target (design change 0136, `systems/controls/selection_and_hud.md > What
//! a rally point makes a new unit do`); and a unit right-clicked among the selected ones stops while the others follow
//! it (design change 0168, `systems/controls/commands.md > Move`: "Following itself = Stop").
//!
//! The fields of the remembered targets play with the fog of war on: player 1's scout sees them in the first tick and
//! is removed, so that player 1 remembers them, greyed, out of its sight.

use bevy::prelude::*;
use sc_sim::orders::state::{Behaviour, GatherPhase};
use sc_sim::testing::ScenarioBuilder;
use sc_sim::tuning::vision::FOG_DELAY_TICKS;
use sc_sim::{
    Command, DebugCommand, Faction, MoveStatus, ObjectId, ObjectKind, OrderKind, PlayerId, ProductionCommand, Rally,
    Scenario, SpawnAt, Target, TargetView, TilePos,
};
use space_crystals::command_ui::PointerDisplay;
use space_crystals::sim_driver::SimRes;
use space_crystals::{AppConfig, build_app};

use super::*;

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

/// The objects of [`gdo_field`], by id, and where the structures are drawn (the map is flat).
const GDO_PKS: [u32; 2] = [1, 2];
const GDO_BARRACKS: u32 = 3;
const GDO_BARRACKS_AT: Vec2 = Vec2::new(1120.0, 960.0); // tiles (16, 14) to (18, 15)
const GDO_SCOUT: u32 = 4;
/// Player 2's Generator, remembered: tiles (34, 12) to (35, 13).
const GENERATOR: u32 = 5;
const GENERATOR_AT: Vec2 = Vec2::new(2240.0, 832.0);

/// The objects of [`colonists_field`], by id, and where they are drawn.
const DRONES: [u32; 2] = [1, 2];
const PORTAL: u32 = 4;
const PORTAL_AT: Vec2 = Vec2::new(1152.0, 1024.0); // tiles (16, 14) to (19, 17)
/// The field: two patches, remembered, on tiles (34, 12) and (34, 14).
const PATCH_A: u32 = 5;
const PATCH_A_AT: Vec2 = Vec2::new(2208.0, 800.0);
const PATCH_B: u32 = 6;
const COLONISTS_SCOUT: u32 = 7;

fn id(n: u32) -> ObjectId {
    ObjectId(n)
}

fn point(x: f32, y: f32) -> SpawnAt {
    SpawnAt::Point(Vec2::new(x, y))
}

fn tile(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Tile(TilePos::new(x, y))
}

fn flat() -> String {
    vec![".".repeat(64); 48].join("\n")
}

/// A headless game of `scenario` as player 1, as [`game_of`] sets it up but with the fog of war on; then the `scout`
/// is removed, which saw the far field in the first tick, and the game runs on through the fog delay (design change
/// 0178), until player 1 remembers the field out of its sight.
fn fog_game(scenario: Scenario, scout: u32) -> App {
    let mut config = AppConfig::new(scenario);
    config.display = space_crystals::platform::DisplayMode::Headless;
    let mut app = build_app(config);
    app.insert_resource(bevy::time::TimeUpdateStrategy::FixedTimesteps(1));
    app.finish();
    app.cleanup();
    submit(
        &mut app,
        P1,
        Command::Debug(DebugCommand::SetAutoTargeting { on: false }),
    );
    app.update();
    app.world_mut().resource_mut::<CameraView>().center = VIEW_CENTER;
    submit(&mut app, P1, Command::Debug(DebugCommand::Remove { id: id(scout) }));
    updates(&mut app, 2 + FOG_DELAY_TICKS as usize);
    app
}

/// Player 1 (the GDO) with Peacekeepers 1 and 2 in the west and a Barracks (3) with a Deployment Center to power it
/// (7, off screen); its scout (4) by player 2's Generator (5) in the east, removed in the first tick; and player 2's
/// Generator 6 far off, never seen, so that player 2 keeps a structure.
fn gdo_field() -> App {
    let scenario = ScenarioBuilder::new(&flat())
        .spawn(ObjectKind::Peacekeeper, Some(P1), point(900.0, 800.0))
        .spawn(ObjectKind::Peacekeeper, Some(P1), point(900.0, 900.0))
        .spawn(ObjectKind::Barracks, Some(P1), tile(16, 14))
        .spawn(ObjectKind::Peacekeeper, Some(P1), point(2240.0, 640.0))
        .spawn(ObjectKind::Generator, Some(P2), tile(34, 12))
        .spawn(ObjectKind::Generator, Some(P2), tile(60, 44))
        .spawn(ObjectKind::DeploymentCenter, Some(P1), tile(2, 40))
        .build();
    let app = fog_game(scenario, GDO_SCOUT);
    assert!(matches!(view(&app, GENERATOR), TargetView::Remembered { .. }));
    app
}

/// Player 1 (the Colonists) with Drones 1 and 2 in the west, an Egress Node (3) and a Robo Portal (4); a field of two
/// patches in the east (5, 6) that its scout (7) saw in the first tick before it was removed; and player 2's
/// Generator (8) far off, never seen.
fn colonists_field() -> App {
    let scenario = ScenarioBuilder::new(&flat())
        .faction(P1, Faction::Colonists)
        .spawn(ObjectKind::Drone, Some(P1), point(900.0, 800.0))
        .spawn(ObjectKind::Drone, Some(P1), point(900.0, 880.0))
        .spawn(ObjectKind::EgressNode, Some(P1), tile(10, 14))
        .spawn(ObjectKind::RoboPortal, Some(P1), tile(16, 14))
        .spawn(ObjectKind::CrystalPatch, None, tile(34, 12))
        .spawn(ObjectKind::CrystalPatch, None, tile(34, 14))
        .spawn(ObjectKind::Drone, Some(P1), point(2100.0, 870.0))
        .spawn(ObjectKind::Generator, Some(P2), tile(60, 44))
        .build();
    let app = fog_game(scenario, COLONISTS_SCOUT);
    for patch in [PATCH_A, PATCH_B] {
        assert!(matches!(view(&app, patch), TargetView::Remembered { .. }), "greyed");
    }
    app
}

/// Object `n` as player 1 knows it.
fn view(app: &App, n: u32) -> TargetView {
    app.world().resource::<SimRes>().target_view(P1, id(n))
}

fn exists(app: &App, n: u32) -> bool {
    app.world().resource::<SimRes>().object(id(n)).is_some()
}

/// The order unit `n` carries out.
fn current(app: &App, n: u32) -> Option<OrderKind> {
    object_orders(app, n).current.map(|active| active.order.kind)
}

/// Updates until `done` holds, at most `limit` frames (one tick each).
fn until(app: &mut App, limit: usize, what: &str, done: impl Fn(&App) -> bool) {
    for _ in 0..limit {
        if done(app) {
            return;
        }
        app.update();
    }
    panic!("{what}: not within {limit} frames");
}

/// Whether Drone `n` mines patch `patch`.
fn mines(app: &App, n: u32, patch: u32) -> bool {
    let orders = object_orders(app, n);
    orders.current.is_some_and(|active| {
        active.order.kind == OrderKind::Gather(id(patch))
            && matches!(active.state, Behaviour::Gather(gather) if matches!(gather.phase, GatherPhase::Mining { .. }))
    })
}

/// The units player 1's producer `producer` released so far, in id order: those beyond `before`, the last id before
/// production.
fn new_units(app: &App, before: u32) -> Vec<u32> {
    let sim = app.world().resource::<SimRes>();
    (sim.objects())
        .filter(|o| o.id.0 > before && o.owner == Some(P1) && o.orders.is_some())
        .map(|o| o.id.0)
        .collect()
}

fn last_id(app: &App) -> u32 {
    app.world().resource::<SimRes>().objects().last().unwrap().id.0
}

fn train(app: &mut App, producer: u32, kind: ObjectKind) {
    let command = Command::Production {
        objects: vec![id(producer)],
        command: ProductionCommand::Train(kind),
    };
    submit(app, P1, command);
}

fn rally_of(app: &App, producer: u32) -> Option<Rally> {
    production_of(app, producer).rally
}

/// The acceptance of slice M3-R for 0139: with Peacekeepers selected, the pointer over a building player 1 remembers
/// shows Attack, and a right-click gives them an Attack on it, which they carry out by walking to its last-known
/// position; where it stands, they attack it.
#[test]
fn peacekeepers_right_clicked_on_a_remembered_building_attack_it() {
    let mut app = gdo_field();
    select_around(&mut app, &[Vec2::new(900.0, 800.0), Vec2::new(900.0, 900.0)], &GDO_PKS);
    assert_eq!(pointer_over(&mut app, GENERATOR_AT), Some(PointerDisplay::Attack));
    right_click(&mut app, GENERATOR_AT, &[]);
    assert_eq!(
        orders(&app).last(),
        Some(&given(&GDO_PKS, OrderKind::Attack(id(GENERATOR)), false))
    );
    let max_hp = sc_sim::ObjectKind::Generator.def().max_hp.unwrap();
    until(&mut app, 1500, "the Generator takes damage", |app| {
        let sim = app.world().resource::<SimRes>();
        sim.object(id(GENERATOR)).unwrap().hp.is_some_and(|hp| hp < max_hp)
    });
    assert_eq!(current(&app, 1), Some(OrderKind::Attack(id(GENERATOR))));
}

/// Design change 0139: Peacekeepers sent at a remembered building that was destroyed out of their owner's sight walk
/// to its last-known position and stop where they see its place empty, "the way it would for a vanished target".
#[test]
fn peacekeepers_sent_at_a_remembered_building_destroyed_unseen_stop_where_they_see_its_place() {
    let mut app = gdo_field();
    select_around(&mut app, &[Vec2::new(900.0, 800.0), Vec2::new(900.0, 900.0)], &GDO_PKS);
    right_click(&mut app, GENERATOR_AT, &[]);
    submit(&mut app, P2, Command::Debug(DebugCommand::Remove { id: id(GENERATOR) }));
    updates(&mut app, 2);
    assert!(!exists(&app, GENERATOR));
    assert!(matches!(view(&app, GENERATOR), TargetView::Remembered { .. }), "unseen");
    assert_eq!(current(&app, 1), Some(OrderKind::Attack(id(GENERATOR))));
    until(&mut app, 1500, "player 1 sees the place", |app| {
        view(app, GENERATOR) == TargetView::Gone
    });
    updates(&mut app, 1);
    for n in GDO_PKS {
        assert_eq!(current(&app, n), None, "unit {n} stopped");
        assert!(
            pos(&app, n).distance(GENERATOR_AT) > 300.0,
            "where it saw the place, short of it"
        );
    }
}

/// The acceptance for 0136 and 0139: a Barracks right-clicked onto a building player 1 remembers keeps its rally
/// point on it, out of sight (`Rally::Object`), and its new Peacekeeper gets its own default command there: an Attack.
#[test]
fn a_barracks_rallied_on_a_remembered_building_sends_its_units_to_attack_it() {
    let mut app = gdo_field();
    click(&mut app, GDO_BARRACKS_AT, &[]);
    assert_eq!(selected(&app), [GDO_BARRACKS]);
    right_click(&mut app, GENERATOR_AT, &[]);
    updates(&mut app, 2);
    assert!(matches!(rally_of(&app, GDO_BARRACKS), Some(Rally::Object { id, .. }) if id == ObjectId(GENERATOR)));
    set_crystals(&mut app, 100);
    let before = last_id(&app);
    train(&mut app, GDO_BARRACKS, ObjectKind::Peacekeeper);
    until(&mut app, 400, "a Peacekeeper is out", |app| {
        !new_units(app, before).is_empty()
    });
    updates(&mut app, 2);
    let unit = new_units(&app, before)[0];
    assert_eq!(current(&app, unit), Some(OrderKind::Attack(id(GENERATOR))));
    assert!(matches!(rally_of(&app, GDO_BARRACKS), Some(Rally::Object { id, .. }) if id == ObjectId(GENERATOR)));
}

/// The acceptance for 0136: a Barracks right-clicked onto an enemy unit in sight rallies on it, and its new
/// Peacekeeper gets its own default command on it: an Attack.
#[test]
fn a_barracks_rallied_on_an_enemy_sends_its_peacekeepers_to_attack_it() {
    let mut app = gdo_field();
    let enemy_at = Vec2::new(1350.0, 960.0);
    let enemy = spawn(&mut app, ObjectKind::Peacekeeper, Some(P2), enemy_at);
    click(&mut app, GDO_BARRACKS_AT, &[]);
    right_click(&mut app, enemy_at, &[]);
    updates(&mut app, 2);
    assert!(matches!(rally_of(&app, GDO_BARRACKS), Some(Rally::Object { id, .. }) if id == ObjectId(enemy)));
    set_crystals(&mut app, 100);
    let before = last_id(&app);
    train(&mut app, GDO_BARRACKS, ObjectKind::Peacekeeper);
    until(&mut app, 400, "a Peacekeeper is out", |app| {
        !new_units(app, before).is_empty()
    });
    updates(&mut app, 2);
    let unit = new_units(&app, before)[0];
    assert_eq!(current(&app, unit), Some(OrderKind::Attack(id(enemy))));
}

/// The acceptance of slice M3-R for 0139: "scout a field and leave, then right-click a greyed patch with Drones: they
/// walk there and mine". Both get a Gather on the patch, which the pointer previews as GatherResources (slice M3-J2,
/// `> PointerDisplayType > Resolution Rules`: "Cursor over resource (resource gatherer selected)"); the first there
/// mines it, the other finds it taken and goes on to the next patch of the field (design change 0132).
#[test]
fn drones_right_clicked_on_a_greyed_patch_walk_there_and_mine() {
    let mut app = colonists_field();
    select_around(&mut app, &[Vec2::new(900.0, 800.0), Vec2::new(900.0, 880.0)], &DRONES);
    assert_eq!(
        pointer_over(&mut app, PATCH_A_AT),
        Some(PointerDisplay::GatherResources)
    );

    right_click(&mut app, PATCH_A_AT, &[]);
    assert_eq!(
        orders(&app).last(),
        Some(&given(&DRONES, OrderKind::Gather(id(PATCH_A)), false))
    );
    until(&mut app, 1500, "both Drones mine, one patch each", |app| {
        (mines(app, 1, PATCH_A) && mines(app, 2, PATCH_B)) || (mines(app, 2, PATCH_A) && mines(app, 1, PATCH_B))
    });
}

/// The acceptance for 0136: a Robo Portal right-clicked onto a patch rallies there, and its new Drone gets its own
/// default command on it, a Gather, and mines it.
#[test]
fn a_robo_portal_rallied_on_a_patch_sends_its_drones_to_mine() {
    let mut app = colonists_field();
    click(&mut app, PORTAL_AT, &[]);
    assert_eq!(selected(&app), [PORTAL]);
    right_click(&mut app, PATCH_A_AT, &[]);
    updates(&mut app, 2);
    assert!(matches!(rally_of(&app, PORTAL), Some(Rally::Object { id, .. }) if id == ObjectId(PATCH_A)));
    set_crystals(&mut app, 100);
    let before = last_id(&app);
    train(&mut app, PORTAL, ObjectKind::Drone);
    until(&mut app, 400, "a Drone is out", |app| {
        !new_units(app, before).is_empty()
    });
    updates(&mut app, 2);
    let drone = new_units(&app, before)[0];
    assert_eq!(current(&app, drone), Some(OrderKind::Gather(id(PATCH_A))));
    until(&mut app, 1500, "it mines the patch", |app| mines(app, drone, PATCH_A));
}

/// The acceptance for 0136: a Robo Portal right-clicked onto an enemy unit in sight rallies on it, and its new Drone,
/// unarmed, follows it ("Enemy or other object → Move to it: Follow if it is a unit",
/// `factions/colonists/units/drone.md > Right-Click Resolution`), and waits by it, out of the portal. Once the enemy is
/// out of sight, the Drone's Follow ends and it stays where it is (design change 0200), and the rally point is a ground
/// rally where the enemy was last seen (design change 0088).
#[test]
fn a_robo_portal_rallied_on_an_enemy_unit_sends_its_drones_after_it() {
    let mut app = colonists_field();
    let enemy = spawn(&mut app, ObjectKind::Peacekeeper, Some(P2), Vec2::new(1350.0, 1024.0));
    click(&mut app, PORTAL_AT, &[]);
    right_click(&mut app, Vec2::new(1350.0, 1024.0), &[]);
    updates(&mut app, 2);
    assert!(matches!(rally_of(&app, PORTAL), Some(Rally::Object { id, .. }) if id == ObjectId(enemy)));
    set_crystals(&mut app, 100);
    let before = last_id(&app);
    train(&mut app, PORTAL, ObjectKind::Drone);
    until(&mut app, 400, "a Drone is out", |app| {
        !new_units(app, before).is_empty()
    });
    updates(&mut app, 2);
    let drone = new_units(&app, before)[0];
    assert_eq!(current(&app, drone), Some(OrderKind::Move(Target::Object(id(enemy)))));
    // Out through the portal's east side (x 1280), it waits by the enemy, at rest (a Drone brakes off its momentum
    // when it stops, slice M3-G).
    until(&mut app, 200, "the Drone waits by the enemy", |app| {
        let unit = app.world().resource::<SimRes>().object(id(drone)).unwrap();
        let mover = unit.mover.as_ref().unwrap();
        mover.status != MoveStatus::Moving && mover.momentum == Vec2::ZERO && unit.pos.x > 1280.0 + 18.0
    });
    // The enemy leaves at once, far out of sight.
    let last_seen = pos(&app, enemy);
    submit(
        &mut app,
        P2,
        Command::Debug(DebugCommand::SetPosition {
            id: id(enemy),
            pos: Vec2::new(3800.0, 2900.0),
        }),
    );
    until(&mut app, 10, "the Drone's Follow ends", |app| {
        current(app, drone).is_none()
    });
    let stopped = pos(&app, drone);
    updates(&mut app, 30);
    assert_eq!(pos(&app, drone), stopped, "it stays where it stopped");
    assert_eq!(rally_of(&app, PORTAL), Some(Rally::Point(last_seen)));
}

/// Design change 0168's Verification: "Select 6 Peacekeepers (one walking), right-click the walking one: it stops;
/// the other 5 follow it." The right-click gives all six one Move to the walking one; it ends at once for that one,
/// as if it had arrived, and the others follow it.
#[test]
fn a_right_click_on_a_selected_unit_stops_it_and_the_others_follow_it() {
    let mut app = game();
    let more: Vec<u32> = [(900.0, 950.0), (1050.0, 950.0), (1050.0, 1100.0)]
        .into_iter()
        .map(|(x, y)| spawn(&mut app, ObjectKind::Peacekeeper, Some(P1), Vec2::new(x, y)))
        .collect();
    let walking = 1;
    let command = Command::Order {
        units: vec![id(walking)],
        kind: OrderKind::Move(Target::Point(Vec2::new(300.0, 800.0))),
        queue: false,
    };
    submit(&mut app, P1, command);
    updates(&mut app, 3);
    let six: Vec<u32> = [1, 2, 5].into_iter().chain(more).collect();
    let points: Vec<Vec2> = six.iter().map(|&n| pos(&app, n)).collect();
    select_around(&mut app, &points, &six);
    assert_eq!(
        current(&app, walking),
        Some(OrderKind::Move(Target::Point(Vec2::new(300.0, 800.0))))
    );
    let at = pos(&app, walking);
    right_click(&mut app, at, &[]);
    assert_eq!(
        orders(&app).last(),
        Some(&given(&six, OrderKind::Move(Target::Object(id(walking))), false))
    );
    updates(&mut app, 2);
    assert_eq!(current(&app, walking), None, "it stopped");
    let stopped = pos(&app, walking);
    updates(&mut app, 60);
    assert!(pos(&app, walking).distance(stopped) < 4.0, "and stays");
    for &n in &six[1..] {
        assert_eq!(
            current(&app, n),
            Some(OrderKind::Move(Target::Object(id(walking)))),
            "unit {n}"
        );
        assert!(
            pos(&app, n).distance(stopped) < 200.0,
            "unit {n} follows: {}",
            pos(&app, n)
        );
    }
}
