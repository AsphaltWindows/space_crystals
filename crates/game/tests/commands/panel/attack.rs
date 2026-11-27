//! A: Attack on every unit panel, issued to the whole selection (`systems/controls/interface.md > CommandPanel`,
//! "Attack goes to everyone", and `systems/controls/interface.md > CommandPanel > Standard Slot Assignments`, "A:
//! Attack, on every unit"; design changes 0397 and 0399): A shows whenever one of the selected units can attack,
//! whichever group is in focus, and none when none can. Its click on an enemy has the units that can attack attack it
//! and the others move to it, a Move on it; its click on the ground gives the whole selection one AttackMove, which
//! the units that can't attack play as a move (`systems/controls/commands.md > AttackMove > Units that can't attack`):
//! "on ground they move to the spot (with the group, as for Attack Move)". On a Colonists field (Drones and Sabre
//! Legionnaires, design change 0399's Verification with the Legionnaires for its Militants, of another faction), a
//! Cults one (Militants, a Prayer-mobile and a Sparrow, 0397's Verification) and a GDO one (Field Engineers and
//! Peacekeepers); and (slice M5-K) with a Barracks in focus, selected with Peacekeepers and Rocketmen (design change
//! 0407's Verification).

use sc_sim::testing::ScenarioBuilder;
use sc_sim::{Faction, mover};

use super::*;

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

/// Attack's slot, A.
fn a() -> CommandSlot {
    slot(1, 0)
}

/// The command of the button the HUD shows in A, and its look.
fn shown_in_a(app: &mut App) -> Option<(PanelButton, ButtonLook)> {
    let shown = buttons(app).into_iter().find(|(slot, ..)| *slot == a());
    shown.map(|(_, button, look)| (button, look))
}

/// What A shows while one of the selected units can attack: Attack, a common command.
const ATTACK: Option<(PanelButton, ButtonLook)> =
    Some((PanelButton::Command(PanelCommand::Attack), ButtonLook::Common));

/// The type of the local player's ActiveGroup.
fn active_type(app: &App) -> Option<ObjectKind> {
    let states = app.world().resource::<ControlStates>();
    states.selection(local_player(app)).active().map(|active| active.kind)
}

/// Tabs until the ActiveGroup is of `kind` (four groups at most).
fn focus(app: &mut App, kind: ObjectKind) {
    for _ in 0..4 {
        if active_type(app) == Some(kind) {
            return;
        }
        tap(app, &[], KeyCode::Tab);
    }
    assert_eq!(active_type(app), Some(kind));
}

/// Drags a box around where the objects of `ids` are now, `margin` past each side, and checks it selected them.
fn select_within(app: &mut App, ids: &[u32], margin: f32) {
    let points: Vec<Vec2> = ids.iter().map(|&id| pos(app, id)).collect();
    let min = points.iter().copied().reduce(Vec2::min).unwrap() - margin;
    let max = points.iter().copied().reduce(Vec2::max).unwrap() + margin;
    drag(app, min, max);
    assert_eq!(selected(app), ids);
}

/// Runs frames until `done`, `frames` at most.
fn run_until(app: &mut App, frames: usize, what: &str, done: impl Fn(&App) -> bool) {
    for _ in 0..frames {
        if done(app) {
            return;
        }
        app.update();
    }
    assert!(done(app), "{what} within {frames} frames");
}

/// The order unit `id` carries out now.
fn current(app: &App, id: u32) -> Option<OrderKind> {
    object_orders(app, id).current.map(|active| active.order.kind)
}

/// Whether unit `id` has done its orders where the ground point `to` is, within the 3 grid units of a crowd's
/// arrival (design change 0183, `sc_sim::mover::OCCUPIED_ARRIVAL_GU`).
fn arrived(app: &App, id: u32, to: Vec2) -> bool {
    current(app, id).is_none() && pos(app, id).distance(to) < sc_sim::gu(mover::OCCUPIED_ARRIVAL_GU)
}

fn flat() -> String {
    vec![".".repeat(64); 48].join("\n")
}

// The objects of [`colonists_game`], by id.
const DRONES: [u32; 2] = [1, 2];
const SABRES: [u32; 2] = [3, 4];
const ENEMY: u32 = 5;

/// Player 1, the Colonists, with two Drones and two Sabre Legionnaires in the west of the world view, and player 2's
/// Peacekeeper in their sight, east of them.
fn colonists_game() -> App {
    let point = |x, y| SpawnAt::Point(Vec2::new(x, y));
    let builder = ScenarioBuilder::new(&flat())
        .faction(P1, Faction::Colonists)
        .faction(P2, Faction::Gdo)
        .spawn(ObjectKind::Drone, Some(P1), point(1000.0, 800.0))
        .spawn(ObjectKind::Drone, Some(P1), point(1060.0, 800.0))
        .spawn(ObjectKind::SabreLegionnaire, Some(P1), point(1000.0, 900.0))
        .spawn(ObjectKind::SabreLegionnaire, Some(P1), point(1060.0, 900.0))
        .spawn(ObjectKind::Peacekeeper, Some(P2), point(1300.0, 850.0));
    game_of(builder.build())
}

/// Design change 0399's Verification: "Select Drones and Militants with the Drones in focus: A is shown. Pressing it
/// makes the Militants attack and the Drones move. Select Drones alone: there is no A." With Sabre Legionnaires for the
/// Militants: the Drones in focus show A, Attack, common to the whole selection, as the Legionnaires' group does. A
/// and a click on the enemy have the Legionnaires attack it and the Drones move to it, a Move on it (design change
/// 0398: "on a target they move to it (as a Move on that object)"), each its order at once; A and a click on the
/// ground give the four one AttackMove, which brings the Drones there. The Drones alone show no A, and its key does
/// nothing.
#[test]
fn drones_in_focus_show_a_which_has_the_legionnaires_attack_and_the_drones_move() {
    let mut app = colonists_game();
    let four = [DRONES, SABRES].concat();
    select_within(&mut app, &four, 30.0);
    focus(&mut app, ObjectKind::Drone);
    assert_eq!(shown_in_a(&mut app), ATTACK, "the Drones in focus");
    focus(&mut app, ObjectKind::SabreLegionnaire);
    assert_eq!(shown_in_a(&mut app), ATTACK);
    focus(&mut app, ObjectKind::Drone);

    tap(&mut app, &[], KeyCode::KeyA);
    assert_eq!(interface(&app), InterfaceState::awaiting(PanelCommand::Attack));
    let enemy_at = pos(&app, ENEMY);
    click(&mut app, enemy_at, &[]);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    let to_enemy = OrderKind::Move(Target::Object(ObjectId(ENEMY)));
    let attack = OrderKind::Attack(ObjectId(ENEMY));
    assert_eq!(
        orders(&app),
        [given(&DRONES, to_enemy, false), given(&SABRES, attack, false)]
    );
    updates(&mut app, 2);
    assert_eq!(DRONES.map(|id| current(&app, id)), [Some(to_enemy); 2]);
    assert_eq!(SABRES.map(|id| current(&app, id)), [Some(attack); 2]);

    let spot = Vec2::new(1100.0, 1150.0);
    tap(&mut app, &[], KeyCode::KeyA);
    click(&mut app, spot, &[]);
    let given_orders = orders(&app);
    assert_eq!(given_orders.len(), 3, "{given_orders:?}");
    assert_eq!(given_orders[2].units, four);
    assert!(matches!(given_orders[2].kind, OrderKind::AttackMove(p) if near(p, spot)));
    run_until(&mut app, 400, "the Drones there", |app| {
        DRONES.iter().all(|&id| arrived(app, id, spot))
    });

    select_within(&mut app, &DRONES, 30.0);
    assert_eq!(shown_in_a(&mut app), None, "the Drones alone");
    tap(&mut app, &[], KeyCode::KeyA);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    assert_eq!(orders(&app).len(), 3, "nothing ordered");
}

// The objects of [`cults_game`], by id.
const MILITANTS: [u32; 2] = [1, 2];
const PRAYER_MOBILE: u32 = 3;
const SPARROW: u32 = 4;

/// Player 1, the Cults, with two Militants, a Prayer-mobile (no weapon: "a pure caster") and a Sparrow (its machine
/// gun), circling in the air between them, in the west of the world view; player 2 has nothing.
fn cults_game() -> App {
    let point = |x, y| SpawnAt::Point(Vec2::new(x, y));
    let builder = ScenarioBuilder::new(&flat())
        .faction(P1, Faction::Cults)
        .faction(P2, Faction::Gdo)
        .spawn(ObjectKind::Militant, Some(P1), point(1000.0, 900.0))
        .spawn(ObjectKind::Militant, Some(P1), point(1300.0, 900.0))
        .spawn(ObjectKind::PrayerMobile, Some(P1), point(1150.0, 1050.0))
        .spawn(ObjectKind::Sparrow, Some(P1), point(1150.0, 900.0));
    game_of(builder.build())
}

/// Design change 0397's Verification: "Select Militants and a Prayer-mobile, press A and click ground: the Militants
/// Attack Move, and the Prayer-mobile moves with them", here with the Sparrow too (slice M5-H1; its panel is
/// BasicCombatUnitInterfaceState's, its A Attack). With the Prayer-mobile in focus, whose panel is the BasicCombatUnit's
/// without its attack commands (slice M5-N11), A is Attack, common, and so it is with the Militants' group and the
/// Sparrow's. A
/// and a click on the ground give the four one AttackMove: the Militants and the Sparrow carry it out, and the
/// Prayer-mobile moves there with them. The Prayer-mobile alone shows no A; the Sparrow alone does.
#[test]
fn militants_attack_move_and_the_prayer_mobile_moves_with_them() {
    let mut app = cults_game();
    select_within(&mut app, &[PRAYER_MOBILE], 40.0);
    assert_eq!(shown_in_a(&mut app), None, "the Prayer-mobile alone");
    tap(&mut app, &[], KeyCode::KeyA);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    // The Sparrow's body, drawn 48 su above its point, and none of the others.
    let sparrow = pos(&app, SPARROW);
    drag(
        &mut app,
        sparrow - Vec2::new(60.0, 100.0),
        sparrow + Vec2::new(60.0, 20.0),
    );
    assert_eq!(selected(&app), [SPARROW]);
    assert_eq!(shown_in_a(&mut app), ATTACK, "the Sparrow alone");

    let four = [MILITANTS.as_slice(), &[PRAYER_MOBILE, SPARROW]].concat();
    // The Sparrow circles 1 grid unit round its point, its body drawn 48 su above it: the box takes it in.
    select_within(&mut app, &four, 140.0);
    for kind in [ObjectKind::Militant, ObjectKind::Sparrow, ObjectKind::PrayerMobile] {
        focus(&mut app, kind);
        assert_eq!(shown_in_a(&mut app), ATTACK, "{kind:?} in focus");
    }

    let spot = Vec2::new(1500.0, 1150.0);
    tap(&mut app, &[], KeyCode::KeyA);
    assert_eq!(interface(&app), InterfaceState::awaiting(PanelCommand::Attack));
    click(&mut app, spot, &[]);
    let given_orders = orders(&app);
    assert_eq!(given_orders.len(), 1, "{given_orders:?}");
    assert_eq!(
        given_orders[0].units, four,
        "one AttackMove to the four, nothing else ordered"
    );
    let attack_move = given_orders[0].kind;
    assert!(matches!(attack_move, OrderKind::AttackMove(p) if near(p, spot)));
    updates(&mut app, 2);
    for id in &four {
        assert_eq!(current(&app, *id), Some(attack_move), "{id}");
    }
    run_until(&mut app, 600, "the Prayer-mobile there", |app| {
        arrived(app, PRAYER_MOBILE, spot)
    });
}

// The objects of [`gdo_game`], by id.
const ENGINEERS: [u32; 2] = [1, 2];
const PEACEKEEPERS: [u32; 2] = [3, 4];
const ENEMY_GUARD: u32 = 5;

/// Player 1, the GDO, with two Field Engineers and two Peacekeepers in the west of the world view, and player 2's
/// Guard in their sight, east of them.
fn gdo_game() -> App {
    let point = |x, y| SpawnAt::Point(Vec2::new(x, y));
    let builder = ScenarioBuilder::new(&flat())
        .faction(P1, Faction::Gdo)
        .faction(P2, Faction::Syndicate)
        .spawn(ObjectKind::FieldEngineer, Some(P1), point(1000.0, 800.0))
        .spawn(ObjectKind::FieldEngineer, Some(P1), point(1060.0, 800.0))
        .spawn(ObjectKind::Peacekeeper, Some(P1), point(1000.0, 900.0))
        .spawn(ObjectKind::Peacekeeper, Some(P1), point(1060.0, 900.0))
        .spawn(ObjectKind::Guard, Some(P2), point(1300.0, 850.0));
    game_of(builder.build())
}

/// The same with the GDO's worker: the Field Engineers in focus, whose own panel has no A, show it while the
/// Peacekeepers are selected with them; its click on the enemy has the Peacekeepers attack it and the Field Engineers
/// move to it.
#[test]
fn field_engineers_in_focus_send_the_peacekeepers_to_attack() {
    let mut app = gdo_game();
    select_within(&mut app, &[ENGINEERS, PEACEKEEPERS].concat(), 30.0);
    focus(&mut app, ObjectKind::FieldEngineer);
    assert_eq!(shown_in_a(&mut app), ATTACK);
    tap(&mut app, &[], KeyCode::KeyA);
    let enemy_at = pos(&app, ENEMY_GUARD);
    click(&mut app, enemy_at, &[]);
    let to_enemy = OrderKind::Move(Target::Object(ObjectId(ENEMY_GUARD)));
    let attack = OrderKind::Attack(ObjectId(ENEMY_GUARD));
    assert_eq!(
        orders(&app),
        [given(&ENGINEERS, to_enemy, false), given(&PEACEKEEPERS, attack, false)]
    );
}

// The objects of [`barracks_game`], by id.
const BARRACKS_ID: u32 = 1;
const BARRACKS_CENTER: Vec2 = Vec2::new(1120.0, 768.0);
const BARRACKS_UNITS: [u32; 4] = [2, 3, 4, 5];
const GUARD: u32 = 6;

/// Player 1, the GDO, with a Barracks on tiles (16, 11) to (18, 12), and two Peacekeepers and two Rocketmen south of
/// it; player 2's Guard in their sight, east of them.
fn barracks_game() -> App {
    let point = |x, y| SpawnAt::Point(Vec2::new(x, y));
    let builder = ScenarioBuilder::new(&flat())
        .faction(P1, Faction::Gdo)
        .faction(P2, Faction::Syndicate)
        .spawn(ObjectKind::Barracks, Some(P1), SpawnAt::Tile(TilePos::new(16, 11)))
        .spawn(ObjectKind::Peacekeeper, Some(P1), point(1040.0, 900.0))
        .spawn(ObjectKind::Peacekeeper, Some(P1), point(1100.0, 900.0))
        .spawn(ObjectKind::Rocketman, Some(P1), point(1160.0, 900.0))
        .spawn(ObjectKind::Rocketman, Some(P1), point(1220.0, 900.0))
        .spawn(ObjectKind::Guard, Some(P2), point(1450.0, 900.0));
    game_of(builder.build())
}

/// Design change 0407's Verification (slice M5-S5's rule, folded into slice M5-K): "Select a Barracks plus some units,
/// Tab to the Barracks: A: Attack is shown, and A + click sends the units to attack." A click selects the Barracks and
/// a Shift + box adds the Peacekeepers and the Rocketmen; with the Barracks in focus, whose own panel has no A, A is
/// Attack, common, as one of the selected units can attack (design change 0399). A and a click on the Guard give the
/// four units one Attack on it, which they carry out, and the Barracks nothing (design change 0397: "an Attack (A) is
/// issued to **every selected unit**").
#[test]
fn a_barracks_in_focus_shows_a_which_sends_the_units_to_attack() {
    let mut app = barracks_game();
    click(&mut app, BARRACKS_CENTER, &[]);
    assert_eq!(selected(&app), [BARRACKS_ID]);
    press_keys(&mut app, &[KeyCode::ShiftLeft]);
    drag(&mut app, Vec2::new(1000.0, 870.0), Vec2::new(1260.0, 930.0));
    release_keys(&mut app, &[KeyCode::ShiftLeft]);
    assert_eq!(selected(&app), [[BARRACKS_ID].as_slice(), &BARRACKS_UNITS].concat());
    focus(&mut app, ObjectKind::Barracks);
    assert_eq!(shown_in_a(&mut app), ATTACK, "the Barracks in focus");

    tap(&mut app, &[], KeyCode::KeyA);
    assert_eq!(interface(&app), InterfaceState::awaiting(PanelCommand::Attack));
    let guard_at = pos(&app, GUARD);
    click(&mut app, guard_at, &[]);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    let attack = OrderKind::Attack(ObjectId(GUARD));
    assert_eq!(orders(&app), [given(&BARRACKS_UNITS, attack, false)]);
    updates(&mut app, 2);
    for id in BARRACKS_UNITS {
        assert_eq!(current(&app, id), Some(attack), "{id}");
    }
}
