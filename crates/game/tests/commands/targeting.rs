//! Targeting (`> AwaitingTarget`, `> BasicCombatUnitInterfaceState > AwaitingTarget[..] resolution`): a target
//! command's hotkey or button awaits a left-click on the map, which issues the command and returns to DefaultState;
//! Esc, a right-click and Back (Z) cancel.

use bevy::prelude::*;
use sc_sim::{ObjectId, OrderKind, Target};
use space_crystals::command_ui::{InterfaceState, UnitCommand};

use super::*;
use KeyCode::{ControlLeft, Digit1, Escape, KeyA, KeyQ, KeyS, KeyZ, ShiftLeft, Tab};

const AWAIT_ATTACK: InterfaceState = InterfaceState::AwaitingTarget(UnitCommand::Attack);

/// Attack (A): a click on the ground gives AttackMove there, a click on an enemy Attack on it; the left-click
/// selects nothing. A click on an own unit is no valid target: nothing happens and the target is still awaited.
#[test]
fn attack_awaits_a_target_and_resolves_by_it() {
    let mut app = game();
    click(&mut app, at(OWN_PKS[0]), &[ControlLeft]);
    assert_eq!(selected(&app), OWN_PKS);
    tap(&mut app, &[], KeyA);
    assert_eq!(interface(&app), AWAIT_ATTACK);
    assert_eq!(orders(&app), [], "a StateOnlyTransition");
    let ground = Vec2::new(1500.0, 1000.0);
    click(&mut app, ground, &[]);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    let given_orders = orders(&app);
    assert_eq!(given_orders.len(), 1);
    assert!(matches!(given_orders[0].kind, OrderKind::AttackMove(p) if near(p, ground)));
    assert_eq!(
        (given_orders[0].units.clone(), given_orders[0].queue),
        (OWN_PKS.to_vec(), false)
    );
    assert_eq!(selected(&app), OWN_PKS, "the click selected nothing");

    tap(&mut app, &[], KeyA);
    click(&mut app, at(ENEMY_PK), &[]);
    assert_eq!(
        orders(&app)[1],
        given(&OWN_PKS, OrderKind::Attack(ObjectId(ENEMY_PK)), false)
    );
    assert_eq!(selected(&app), OWN_PKS);

    tap(&mut app, &[], KeyA);
    click(&mut app, at(OWN_GUARDS[0]), &[]);
    click(&mut app, at(PATCH), &[]);
    assert_eq!(interface(&app), AWAIT_ATTACK, "still awaiting a valid target");
    assert_eq!(orders(&app).len(), 2);
    assert_eq!(selected(&app), OWN_PKS);
}

/// Move (Q) takes the ground or any object; Patrol (S) only the ground.
#[test]
fn move_takes_any_target_and_patrol_only_the_ground() {
    let mut app = game();
    click(&mut app, at(1), &[]);
    tap(&mut app, &[], KeyQ);
    click(&mut app, at(PATCH), &[]);
    tap(&mut app, &[], KeyQ);
    click(&mut app, at(ENEMY_PK), &[]);
    tap(&mut app, &[], KeyS);
    click(&mut app, at(ENEMY_PK), &[]);
    assert_eq!(interface(&app), InterfaceState::AwaitingTarget(UnitCommand::Patrol));
    let ground = Vec2::new(1200.0, 1150.0);
    click(&mut app, ground, &[]);
    let object = |id: u32| OrderKind::Move(Target::Object(ObjectId(id)));
    let given_orders = orders(&app);
    assert_eq!(
        given_orders[..2],
        [given(&[1], object(PATCH), false), given(&[1], object(ENEMY_PK), false)]
    );
    assert!(matches!(given_orders[2].kind, OrderKind::Patrol(p) if near(p, ground)));
    assert_eq!(given_orders.len(), 3);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
}

/// `> AwaitingTarget`: Escape or right-click cancels (a StateOnlyTransition back to DefaultState); so does Back,
/// by its hotkey (Z) or its button (`> Standard Slot Assignments`). None of them gives an order: the right-click
/// that cancels moves nobody.
#[test]
fn esc_right_click_and_back_cancel_targeting() {
    let mut app = game();
    select_mixed(&mut app);
    let cancels: [&dyn Fn(&mut App); 4] = [
        &|app| tap(app, &[], Escape),
        &|app| right_click(app, Vec2::new(1000.0, 1000.0), &[]),
        &|app| tap(app, &[], KeyZ),
        &|app| click_button(app, CommandSlot::BACK, &[]),
    ];
    for cancel in cancels {
        tap(&mut app, &[], KeyA);
        assert_eq!(interface(&app), AWAIT_ATTACK);
        cancel(&mut app);
        assert_eq!(interface(&app), InterfaceState::DefaultState);
    }
    assert_eq!(orders(&app), []);
    assert_eq!(selected(&app), [1, 2, 3, 4]);
    // The button enters targeting too.
    click_button(&mut app, slot(1, 1), &[]);
    assert_eq!(interface(&app), InterfaceState::AwaitingTarget(UnitCommand::Patrol));
}

/// `commands.md > Unit Command`: Shift queues a targeted command too. The design returns to DefaultState after
/// every issuing click (`> AwaitingTarget`), Shift or not.
#[test]
fn a_shift_click_queues_the_targeted_order() {
    let mut app = game();
    click(&mut app, at(1), &[]);
    tap(&mut app, &[], KeyA);
    click(&mut app, Vec2::new(1000.0, 1000.0), &[ShiftLeft]);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    tap(&mut app, &[ShiftLeft], KeyQ);
    click(&mut app, Vec2::new(1200.0, 1000.0), &[ShiftLeft]);
    tap(&mut app, &[], KeyA);
    click(&mut app, Vec2::new(1000.0, 1100.0), &[]);
    let queued: Vec<bool> = orders(&app).iter().map(|o| o.queue).collect();
    assert_eq!(queued, [true, true, false]);
    assert_eq!(selected(&app), [1], "Shift-clicks while targeting toggle no selection");
}

/// `> ObjectInterfaceState[ObjectEnum]`: a change of the Selection or of its ActiveGroup resets the state.
#[test]
fn a_new_selection_or_active_group_ends_targeting() {
    let mut app = game();
    click(&mut app, at(5), &[]);
    tap(&mut app, &[ControlLeft], Digit1);
    pause(&mut app);
    select_mixed(&mut app);
    tap(&mut app, &[], KeyA);
    tap(&mut app, &[], Tab);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    tap(&mut app, &[], KeyA);
    tap(&mut app, &[], Digit1);
    assert_eq!(selected(&app), [5], "control group 1 recalled");
    assert_eq!(interface(&app), InterfaceState::DefaultState);
}
