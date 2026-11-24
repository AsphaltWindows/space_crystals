//! The Peacekeeper's Riot Shield in the command flow (`factions/gdo/units.md > Peacekeeper > Riot Shield (toggled
//! ability)`): its button in R and its hotkey raise and lower the shields of the Peacekeepers it goes to, and the
//! button shows whether they are up. With the shields up, Attack is disabled; a right-click on an enemy has the
//! shielded units follow it (design change 0074) and an Attack leaves them out (0075), while the other units of the
//! selection attack it. A shield that isn't down is drawn in front of its unit.

use bevy::prelude::*;
use sc_sim::defs::RIOT_SHIELD_TOGGLE_TICKS;
use sc_sim::{Ability, ObjectId, OrderKind, PlayerId, RiotShield, Target};
use space_crystals::command_ui::cursor::PointerLook;
use space_crystals::command_ui::{ButtonLook, InterfaceState, PanelButton, PanelSlot, PointerDisplay, UnitCommand};
use space_crystals::hud::command_panel::{CommandButton, CommandPanelView};
use space_crystals::object_render::{ObjectVisuals, RiotShieldVisual};

use super::*;
use KeyCode::{KeyA, KeyR, ShiftLeft, Tab};

const RAISE: Ability = Ability::RiotShield { raise: true };
const LOWER: Ability = Ability::RiotShield { raise: false };

/// The Riot Shield's slot, R (a placeholder: the design's hotkey is TBD).
fn r() -> CommandSlot {
    slot(0, 3)
}

/// The button the HUD shows in `slot`.
fn button(app: &mut App, slot: CommandSlot) -> Option<PanelSlot> {
    let mut query = app.world_mut().query::<&CommandButton>();
    query.iter(app.world()).find(|b| b.slot == slot).map(|b| b.shown)
}

fn texts(app: &mut App) -> Vec<String> {
    let mut query = app.world_mut().query::<&Text>();
    query.iter(app.world()).map(|t| t.0.clone()).collect()
}

/// The Riot Shield of unit `id`, as the sim has it.
fn shield(app: &App, id: u32) -> RiotShield {
    let sim = app.world().resource::<SimRes>();
    sim.object(ObjectId(id)).unwrap().riot_shield.expect("a Peacekeeper")
}

/// Player 1's use of `ability` by `units`.
fn used(units: &[u32], ability: Ability) -> (PlayerId, Vec<u32>, Ability) {
    (PlayerId(1), units.to_vec(), ability)
}

/// Long enough for a toggle to finish: the toggle time, and the ticks the command takes to reach the sim.
fn toggle_time(app: &mut App) {
    updates(app, RIOT_SHIELD_TOGGLE_TICKS as usize + 2);
}

/// The drawn shield of unit `id`: its visibility, its scale and the entity it is drawn with.
fn shield_visual(app: &mut App, id: u32) -> Option<(Visibility, f32, Entity)> {
    let mut query = app
        .world_mut()
        .query::<(&RiotShieldVisual, &Visibility, &Transform, &ChildOf)>();
    query
        .iter(app.world())
        .find(|(shield, ..)| shield.0 == ObjectId(id))
        .map(|(_, visibility, transform, parent)| (*visibility, transform.scale.x, parent.parent()))
}

/// R, by hotkey or by a click on its button, raises the shields of the selected Peacekeepers, and lowers them once
/// they are up; the button says which it does. An ability, not an order: no order is given, and Shift queues
/// nothing.
#[test]
fn the_hotkey_and_the_button_raise_and_lower_the_shields() {
    let mut app = game();
    select_box(&mut app, &[1, 2]);
    let shown = button(&mut app, r()).expect("a button in R");
    assert_eq!(shown.button, PanelButton::Command(UnitCommand::RiotShield));
    assert_eq!(
        (shown.look, shown.on, shown.label()),
        (ButtonLook::Common, false, "Raise Shield")
    );
    assert_eq!(app.world().resource::<CommandPanelView>().hotkeys[3], "R");

    tap(&mut app, &[], KeyR);
    assert_eq!(abilities(&app), [used(&[1, 2], RAISE)]);
    toggle_time(&mut app);
    assert_eq!([shield(&app, 1), shield(&app, 2)], [RiotShield::Raised; 2]);
    let shown = button(&mut app, r()).unwrap();
    assert_eq!((shown.on, shown.label()), (true, "Lower Shield"));
    assert!(texts(&mut app).contains(&"Lower Shield".to_string()));

    click_button(&mut app, r(), &[]);
    assert_eq!(abilities(&app)[1..], [used(&[1, 2], LOWER)]);
    toggle_time(&mut app);
    assert_eq!([shield(&app, 1), shield(&app, 2)], [RiotShield::Lowered; 2]);
    assert!(!button(&mut app, r()).unwrap().on);

    tap(&mut app, &[ShiftLeft], KeyR);
    assert_eq!(abilities(&app)[2..], [used(&[1, 2], RAISE)]);
    assert_eq!(orders(&app), []);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
}

/// `> CommandPanel`: the Riot Shield is the Peacekeepers' own, a group command: with them the ActiveGroup it goes
/// to them alone, and with the Guards active R is empty. A group whose shields differ raises them all.
#[test]
fn in_a_mixed_selection_r_goes_to_the_peacekeepers() {
    let mut app = game();
    select_mixed(&mut app);
    assert_eq!(button(&mut app, r()).map(|b| b.look), Some(ButtonLook::Group));
    tap(&mut app, &[], KeyR);
    assert_eq!(abilities(&app), [used(&[1, 2], RAISE)]);
    tap(&mut app, &[], Tab);
    assert_eq!(button(&mut app, r()), None);
    tap(&mut app, &[], KeyR);
    assert_eq!(abilities(&app).len(), 1);
    toggle_time(&mut app);
    assert_eq!(shield(&app, 1), RiotShield::Raised);

    // Peacekeeper 5 joins with its shield down: R raises the group's shields rather than lowering them.
    select_box(&mut app, &[1, 5]);
    assert!(!button(&mut app, r()).unwrap().on);
    tap(&mut app, &[], KeyR);
    assert_eq!(abilities(&app)[1..], [used(&[1, 5], RAISE)]);
    toggle_time(&mut app);
    assert_eq!([shield(&app, 1), shield(&app, 5)], [RiotShield::Raised; 2]);
}

/// "Right-click on an enemy unit while the shield is up: the Peacekeeper follows that unit" (design change 0074,
/// which replaced the Move to the clicked location), and "the player must lower the shield before ordering an
/// attack": the pointer previews a Move over an enemy, and Attack is disabled.
#[test]
fn with_their_shields_up_peacekeepers_follow_the_enemy_they_are_sent_at() {
    let mut app = game();
    select_box(&mut app, &[1, 2]);
    tap(&mut app, &[], KeyR);
    toggle_time(&mut app);
    let screen = screen_at(&app, at(ENEMY_PK));
    input(&mut app).cursor = Some(screen);
    app.update();
    assert_eq!(
        app.world().resource::<PointerLook>().display,
        Some(PointerDisplay::Move)
    );
    assert_eq!(button(&mut app, slot(1, 0)).map(|b| b.look), Some(ButtonLook::Disabled));
    tap(&mut app, &[], KeyA);
    assert_eq!(interface(&app), InterfaceState::DefaultState, "no Attack to target");

    right_click(&mut app, at(ENEMY_PK), &[]);
    let follow = OrderKind::Move(Target::Object(ObjectId(ENEMY_PK)));
    assert_eq!(orders(&app), [given(&[1, 2], follow, false)]);
    let current = object_orders(&app, 1).current.expect("on its way").order.kind;
    assert_eq!(current, follow);
}

/// In a selection whose Guards can attack, Attack stays: a click on an enemy in AwaitingTarget[Attack] sends the
/// Guards to attack it and leaves the shielded Peacekeepers out (design change 0075); a right-click on one sends
/// the Guards to attack it and has the Peacekeepers follow it (0074). Each click gives one order per distinct order,
/// in the order of the units' ids: the Peacekeepers' Follow first.
#[test]
fn a_mixed_selection_attacks_with_the_units_that_can() {
    let mut app = game();
    select_mixed(&mut app);
    tap(&mut app, &[], KeyR);
    toggle_time(&mut app);
    assert_eq!(button(&mut app, slot(1, 0)).map(|b| b.look), Some(ButtonLook::Common));
    tap(&mut app, &[], KeyA);
    assert_eq!(interface(&app), InterfaceState::awaiting(UnitCommand::Attack));
    click(&mut app, at(ENEMY_PK), &[]);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    right_click(&mut app, at(ENEMY_GUARD), &[]);

    let attack = |id: u32| OrderKind::Attack(ObjectId(id));
    let follow = |id: u32| OrderKind::Move(Target::Object(ObjectId(id)));
    assert_eq!(
        orders(&app),
        [
            given(&[3, 4], attack(ENEMY_PK), false),
            given(&[1, 2], follow(ENEMY_GUARD), false),
            given(&[3, 4], attack(ENEMY_GUARD), false),
        ]
    );
}

/// The shield is drawn with its unit's visual while it isn't down, growing to its full size as it goes up; a
/// Guard has none. It is never `Visible` itself, so the fog that hides its unit hides it too (`fog.rs`).
#[test]
fn a_shield_is_drawn_while_it_is_not_down() {
    let mut app = game();
    let unit = app.world().resource::<ObjectVisuals>().by_id[&ObjectId(1)];
    assert_eq!(
        shield_visual(&mut app, 1).map(|(v, _, parent)| (v, parent)),
        Some((Visibility::Hidden, unit))
    );
    assert_eq!(shield_visual(&mut app, 3), None, "a Guard has no shield");

    click(&mut app, at(1), &[]);
    input(&mut app).tap_key(KeyR);
    let mut scales = Vec::new();
    for _ in 0..20 {
        if shield(&app, 1) == RiotShield::Raised {
            break;
        }
        app.update();
        let (visibility, scale, _) = shield_visual(&mut app, 1).unwrap();
        if shield(&app, 1) != RiotShield::Lowered {
            assert_eq!(visibility, Visibility::Inherited);
            scales.push(scale);
        }
    }
    assert_eq!(scales.len(), RIOT_SHIELD_TOGGLE_TICKS as usize);
    assert!(
        scales.is_sorted() && scales[0] < 1.0 && scales.last() == Some(&1.0),
        "{scales:?}"
    );

    tap(&mut app, &[], KeyR);
    toggle_time(&mut app);
    assert_eq!(shield_visual(&mut app, 1).map(|(v, ..)| v), Some(Visibility::Hidden));
}
