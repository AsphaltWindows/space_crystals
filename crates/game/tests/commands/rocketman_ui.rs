//! The Rocketman's UI in the client (slice M5-K, design change 0356; `factions/gdo/units/rocketman.md > Placeholder
//! stats`: "Panel: BasicCombatUnitInterfaceState, plus **R: Target Lock** (targeted: left-click an enemy unit or
//! structure)"): R awaits an enemy unit or structure, under the Attack pointer over one, and gives every selected
//! Rocketman a Target Lock on it; anything else is an invalid target. Shift+R after a Move queues the lock
//! (`systems/controls/commands.md > Unit Command`: a Shift-queued command is appended to the queue; plan decision T76,
//! design change 0438), which starts once the Rocketmen have arrived and fires its homing missiles into an enemy
//! structure. A Shift+right-click on the ground queues a Move behind a lock on a building, which goes on until the
//! building dies (design change 0438's Verification). The panel as plain data is
//! `command_ui::interface::tests::rocketman`'s.

use sc_sim::Faction;
use sc_sim::combat::TargetLock;
use sc_sim::testing::ScenarioBuilder;
use space_crystals::command_ui::feedback::{INVALID_TARGET_TEXT, TargetFeedback};
use space_crystals::command_ui::{ButtonLook, PanelButton, PanelCommand};
use space_crystals::hud::command_panel::CommandButton;

use super::*;
use KeyCode::{Escape, KeyR, ShiftLeft};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

// The objects of [`rocketman_game`], by id.
/// Player 1's two Rocketmen.
const ROCKETMEN: [u32; 2] = [1, 2];
/// Player 2's Peacekeeper, east of them, and Generator, south-east of them, both in their sight.
const ENEMY_PK: u32 = 3;
const GENERATOR: u32 = 4;
/// A point of the ground in the world view, with nothing on it.
const GROUND: Vec2 = Vec2::new(1500.0, 700.0);

/// A game of the GDO against the GDO on a flat 64 × 48 map, as [`game`] sets it up (no fog, no auto-targeting, the
/// camera at [`VIEW_CENTER`]): player 1's two Rocketmen, player 2's Peacekeeper 4 GU east of them and its Generator on
/// tiles (17, 15) to (18, 16), south-east of them.
fn rocketman_game() -> App {
    game_of(rocketman_scenario().build())
}

/// The scenario of [`rocketman_game`], for a test to add to.
fn rocketman_scenario() -> ScenarioBuilder {
    let point = |x, y| SpawnAt::Point(Vec2::new(x, y));
    ScenarioBuilder::new(&vec![".".repeat(64); 48].join("\n"))
        .faction(P1, Faction::Gdo)
        .faction(P2, Faction::Gdo)
        .spawn(ObjectKind::Rocketman, Some(P1), point(1000.0, 900.0))
        .spawn(ObjectKind::Rocketman, Some(P1), point(1060.0, 900.0))
        .spawn(ObjectKind::Peacekeeper, Some(P2), point(1300.0, 900.0))
        .spawn(ObjectKind::Generator, Some(P2), SpawnAt::Tile(TilePos::new(17, 15)))
}

/// The button the HUD shows in `slot`: its command, its look and its label.
fn shown_in(app: &mut App, slot: CommandSlot) -> Option<(PanelButton, ButtonLook, &'static str)> {
    let mut query = app.world_mut().query::<&CommandButton>();
    let mut shown = query.iter(app.world()).filter(|b| b.slot == slot);
    shown.next().map(|b| (b.shown.button, b.shown.look, b.shown.label()))
}

/// The feedback message that shows.
fn said(app: &App) -> Option<&'static str> {
    let feedback = app.world().resource::<TargetFeedback>();
    feedback.since.map(|_| feedback.text)
}

/// Unit `id`'s Target Lock under way (`sc_sim::AttackState::target_lock`).
fn target_lock(app: &App, id: u32) -> Option<TargetLock> {
    let sim = app.world().resource::<SimRes>();
    sim.object(ObjectId(id))?.attack.as_ref()?.target_lock
}

/// The order unit `id` carries out now.
fn current(app: &App, id: u32) -> Option<OrderKind> {
    object_orders(app, id).current.map(|active| active.order.kind)
}

/// R, Target Lock, labelled "Lock", is common to the two selected Rocketmen. It awaits its target: over the enemy
/// Peacekeeper the pointer is Attack (placeholder), over an own Rocketman and over the ground Inactive, and a click on
/// the ground is an invalid target, which keeps the target awaited and gives no order. A click on the Peacekeeper gives
/// both Rocketmen one Target Lock order on it, and the targeting ends; in the sim, both begin their locks on it.
#[test]
fn r_locks_every_selected_rocketman_on_the_enemy_clicked() {
    let mut app = rocketman_game();
    select_units(&mut app, &ROCKETMEN);
    let lock = PanelButton::Command(PanelCommand::TargetLock);
    assert_eq!(shown_in(&mut app, slot(0, 3)), Some((lock, ButtonLook::Common, "Lock")));
    tap(&mut app, &[], KeyR);
    let awaiting = InterfaceState::awaiting(PanelCommand::TargetLock);
    assert_eq!(interface(&app), awaiting);
    let enemy_at = pos(&app, ENEMY_PK);
    assert_eq!(pointer_over(&mut app, enemy_at), Some(PointerDisplay::Attack));
    let own_at = pos(&app, ROCKETMEN[1]);
    assert_eq!(pointer_over(&mut app, own_at), Some(PointerDisplay::Inactive), "own");
    assert_eq!(pointer_over(&mut app, GROUND), Some(PointerDisplay::Inactive), "ground");
    click(&mut app, GROUND, &[]);
    assert_eq!(interface(&app), awaiting);
    assert_eq!(orders(&app), []);
    assert_eq!(said(&app), Some(INVALID_TARGET_TEXT));

    click(&mut app, enemy_at, &[]);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    let order = OrderKind::TargetLock(ObjectId(ENEMY_PK));
    assert_eq!(orders(&app), [given(&ROCKETMEN, order, false)]);
    updates(&mut app, 2);
    for id in ROCKETMEN {
        assert_eq!(current(&app, id), Some(order), "{id}");
        let lock = target_lock(&app, id).expect("locking");
        assert_eq!((lock.target, lock.locked), (ObjectId(ENEMY_PK), false), "{id}");
    }
}

/// A right-click on the ground moves the Rocketmen there; Shift+R and a Shift-click on player 2's Generator queue a
/// Target Lock on it behind the Move (a structure: "Target Lock can target structures (user, 2026-11-26)"). The
/// Rocketmen walk there first, then lock, and their homing missiles hit the Generator.
#[test]
fn shift_r_after_a_move_queues_a_lock_on_a_structure() {
    let mut app = rocketman_game();
    select_units(&mut app, &ROCKETMEN);
    let spot = Vec2::new(1000.0, 1040.0);
    right_click(&mut app, spot, &[]);
    tap(&mut app, &[ShiftLeft], KeyR);
    let generator_at = pos(&app, GENERATOR);
    click(&mut app, generator_at, &[ShiftLeft]);
    tap(&mut app, &[], Escape);
    let given_orders = orders(&app);
    assert_eq!(given_orders.len(), 2, "{given_orders:?}");
    assert!(matches!(given_orders[0].kind, OrderKind::Move(Target::Point(p)) if near(p, spot)));
    let lock = OrderKind::TargetLock(ObjectId(GENERATOR));
    assert_eq!(given_orders[1], given(&ROCKETMEN, lock, true));
    updates(&mut app, 2);
    assert!(matches!(current(&app, ROCKETMEN[0]), Some(OrderKind::Move(_))));
    assert_eq!(target_lock(&app, ROCKETMEN[0]), None, "not before the Move is done");

    let full = hp(&app, GENERATOR).expect("a Generator has HP");
    let mut locked = false;
    for _ in 0..400 {
        app.update();
        locked |= ROCKETMEN
            .iter()
            .all(|&id| target_lock(&app, id).is_some_and(|lock| lock.locked));
        if hp(&app, GENERATOR).is_some_and(|hp| hp < full) {
            break;
        }
    }
    assert!(locked, "both locked on the Generator");
    assert!(hp(&app, GENERATOR).is_some_and(|hp| hp < full), "hit");
    for id in ROCKETMEN {
        assert_eq!(current(&app, id), Some(lock), "{id}");
    }
}

/// Design change 0438's Verification (answering Q0171): "A Rocketman locking a building, Shift+right-click ground: the
/// lock continues; when the building dies, it moves there." Player 2's Generator has 40 HP left, a few locked
/// missiles' worth, and player 2 a second Generator far off, which keeps it in the game (losing its last structure
/// would defeat it and end this one). Rocketman 1 locks onto the Generator, and a Shift+right-click on the ground
/// queues a Move behind the lock, which goes on: the Rocketman stands, locks and hits until the Generator dies, and
/// then walks to the point clicked.
#[test]
fn a_shift_right_click_on_the_ground_waits_behind_a_lock_on_a_building() {
    let far = SpawnAt::Tile(TilePos::new(56, 40));
    let mut app = game_of(rocketman_scenario().spawn(ObjectKind::Generator, Some(P2), far).build());
    set_hp(&mut app, GENERATOR, 40.0);
    let rocketman = ROCKETMEN[0];
    select_units(&mut app, &[rocketman]);
    tap(&mut app, &[], KeyR);
    let generator_at = pos(&app, GENERATOR);
    click(&mut app, generator_at, &[]);
    updates(&mut app, 2);
    let lock = OrderKind::TargetLock(ObjectId(GENERATOR));
    assert_eq!(current(&app, rocketman), Some(lock));
    let spot = Vec2::new(1000.0, 1040.0);
    right_click(&mut app, spot, &[ShiftLeft]);
    let given_orders = orders(&app);
    assert_eq!(given_orders.len(), 2, "{given_orders:?}");
    assert_eq!(given_orders[0], given(&[rocketman], lock, false));
    let walk = given_orders[1].kind;
    assert!(
        matches!(walk, OrderKind::Move(Target::Point(p)) if near(p, spot)),
        "{walk:?}"
    );
    assert!(given_orders[1].queue, "Shift queues the Move");

    let stands = pos(&app, rocketman);
    let (mut locked, mut hits, mut died) = (false, 0, false);
    for _ in 0..400 {
        let before = hp(&app, GENERATOR);
        app.update();
        let Some(now) = hp(&app, GENERATOR) else {
            died = true;
            break;
        };
        hits += usize::from(before.is_some_and(|hp| now < hp));
        assert_eq!(current(&app, rocketman), Some(lock), "the lock continues");
        assert_eq!(pos(&app, rocketman), stands, "the Rocketman stands");
        locked |= target_lock(&app, rocketman).is_some_and(|lock| lock.locked);
    }
    assert!(died, "the Generator dies");
    assert!(locked && hits > 0, "locked: {locked}; hits before the last: {hits}");

    updates(&mut app, 2);
    assert_eq!(
        current(&app, rocketman),
        Some(walk),
        "the queued Move runs once the lock has ended"
    );
    let mut arrived = false;
    for _ in 0..200 {
        app.update();
        if current(&app, rocketman).is_none() {
            arrived = true;
            break;
        }
    }
    assert!(arrived, "the Move ends");
    let at = pos(&app, rocketman);
    assert!(at.distance(spot) < 8.0, "it moved there: {at:?}");
}
