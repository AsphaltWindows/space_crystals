//! Targeting (`> AwaitingTarget`, `> BasicCombatUnitInterfaceState > AwaitingTarget[..] resolution`): a target
//! command's hotkey or button awaits a left-click on the map, which issues the command and returns to DefaultState;
//! Esc, a right-click and Back (Z) cancel. The rules of design changes 0064 to 0069: force-attack on own and
//! neutral destructible objects, invalid target clicks, no other commands and no box while targeting, the press
//! point, and which selection changes cancel. Shift while targeting (0070) is `shift_targeting.rs`, the click
//! allowance (0073) `click_allowance.rs`.

use bevy::prelude::*;
use sc_sim::{Command, ObjectId, OrderKind, Target};
use space_crystals::command_ui::feedback::{FEEDBACK_TICKS, INVALID_TARGET_TEXT, TargetFeedback, TargetFeedbackView};
use space_crystals::command_ui::{ButtonLook, InterfaceState, UnitCommand};
use space_crystals::hud::command_panel::CommandButton;
use space_crystals::selection::Relation;

use super::*;
use KeyCode::{ControlLeft, Digit1, Digit2, Escape, KeyA, KeyE, KeyQ, KeyR, KeyS, KeyX, KeyZ, ShiftLeft, Tab};

const AWAIT_ATTACK: InterfaceState = InterfaceState::awaiting(UnitCommand::Attack);
const AWAIT_PATROL: InterfaceState = InterfaceState::awaiting(UnitCommand::Patrol);

fn attack(id: u32) -> OrderKind {
    OrderKind::Attack(ObjectId(id))
}

fn object(id: u32) -> OrderKind {
    OrderKind::Move(Target::Object(ObjectId(id)))
}

/// Removes object `id` from the game, as a death would (`SimEvent::Removed`), by a debug command; it is gone two
/// frames later.
fn remove(app: &mut App, id: u32) {
    let command = Command::Debug(DebugCommand::Remove { id: ObjectId(id) });
    submit(app, PlayerId(1), command);
    updates(app, 2);
    assert!(app.world().resource::<SimRes>().object(ObjectId(id)).is_none());
}

/// Whether the invalid-target message shows.
fn feedback_shown(app: &mut App) -> bool {
    let mut query = app
        .world_mut()
        .query_filtered::<&Visibility, With<TargetFeedbackView>>();
    let shown = *query.single(app.world()).unwrap() != Visibility::Hidden;
    assert_eq!(shown, app.world().resource::<TargetFeedback>().since.is_some());
    shown
}

/// Drags the left button from the view point `from` to `to` (down, moved, up, a frame each, then a [`pause`]), and
/// says whether the selection box was drawn while the button was held.
fn drag_drawing(app: &mut App, from: Vec2, to: Vec2) -> bool {
    let (from, to) = (screen_at(app, from), screen_at(app, to));
    input(app).cursor = Some(from);
    input(app).mouse.press(MouseButton::Left);
    app.update();
    input(app).cursor = Some(to);
    app.update();
    let drawn = box_drawn(app);
    input(app).mouse.release(MouseButton::Left);
    app.update();
    pause(app);
    drawn
}

/// Whether the pointer, with the cursor at `screen` (logical pixels), is over an object, a frame later.
fn over_object(app: &mut App, screen: Vec2) -> bool {
    input(app).cursor = Some(screen);
    app.update();
    app.world().resource::<PointerLook>().hovered.is_some()
}

/// Attack (A): a click on the ground gives AttackMove there, a click on an enemy Attack on it; the left-click
/// selects nothing. A click on an own unit attacks it too (force-attack, design change 0064). A click on the crystal
/// patch, which can't be destroyed, is no valid target: nothing happens and the target is still awaited.
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
    assert_eq!(orders(&app)[1], given(&OWN_PKS, attack(ENEMY_PK), false));
    assert_eq!(selected(&app), OWN_PKS);

    tap(&mut app, &[], KeyA);
    click(&mut app, at(OWN_GUARDS[0]), &[]);
    assert_eq!(orders(&app)[2], given(&OWN_PKS, attack(OWN_GUARDS[0]), false));
    assert_eq!(interface(&app), InterfaceState::DefaultState);

    tap(&mut app, &[], KeyA);
    click(&mut app, at(PATCH), &[]);
    assert_eq!(interface(&app), AWAIT_ATTACK, "still awaiting a valid target");
    assert_eq!(orders(&app).len(), 3);
    assert_eq!(selected(&app), OWN_PKS);
}

/// Design change 0064 (force-attack, 0059): while Attack awaits its target, an own or a neutral destructible object
/// is one: the pointer shows Attack over it, the click gives Attack on it, and the sim takes it. The crystal patch,
/// indestructible, stays Inactive. A right-click never attacks them: it moves to them (0059).
#[test]
fn attack_targeting_takes_own_and_neutral_destructible_objects() {
    let mut app = game();
    // A neutral Peacekeeper, in sight of player 1's units.
    let neutral_at = Vec2::new(1100.0, 1000.0);
    let spawn = DebugCommand::Spawn {
        kind: ObjectKind::Peacekeeper,
        owner: None,
        at: SpawnAt::Point(neutral_at),
        orientation: Default::default(),
    };
    submit(&mut app, PlayerId(1), Command::Debug(spawn));
    updates(&mut app, 2);
    let sim = app.world().resource::<SimRes>();
    let neutral = sim
        .objects()
        .find(|o| o.owner.is_none() && o.kind == ObjectKind::Peacekeeper)
        .expect("spawned")
        .id
        .0;

    click(&mut app, at(1), &[]);
    tap(&mut app, &[], KeyA);
    assert_eq!(pointer_over(&mut app, at(OWN_GUARDS[0])), Some(PointerDisplay::Attack));
    assert_eq!(pointer_over(&mut app, neutral_at), Some(PointerDisplay::Attack));
    assert_eq!(app.world().resource::<PointerLook>().hovered, Some(Relation::Neutral));
    assert_eq!(pointer_over(&mut app, at(PATCH)), Some(PointerDisplay::Inactive));
    click(&mut app, at(OWN_GUARDS[0]), &[]);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    tap(&mut app, &[], KeyA);
    click(&mut app, neutral_at, &[]);
    assert_eq!(
        orders(&app),
        [
            given(&[1], attack(OWN_GUARDS[0]), false),
            given(&[1], attack(neutral), false)
        ]
    );
    let current = object_orders(&app, 1).current.expect("attacking").order.kind;
    assert_eq!(current, attack(neutral), "the sim took it");

    // DefaultState: the right-click moves to them.
    assert_eq!(pointer_over(&mut app, neutral_at), Some(PointerDisplay::Move));
    right_click(&mut app, at(OWN_GUARDS[0]), &[]);
    right_click(&mut app, neutral_at, &[]);
    assert_eq!(
        orders(&app)[2..],
        [
            given(&[1], object(OWN_GUARDS[0]), false),
            given(&[1], object(neutral), false)
        ]
    );
}

/// Design change 0065: Patrol, then a click on an enemy unit, which Patrol can't use (the pointer shows Inactive):
/// the click is ignored and Patrol still awaits its target, with short feedback, the "invalid target" message, for
/// `FEEDBACK_TICKS`. A click on the ground then gives the patrol.
#[test]
fn an_invalid_target_click_is_ignored_with_feedback() {
    let mut app = game();
    click(&mut app, at(1), &[]);
    tap(&mut app, &[], KeyS);
    assert!(!feedback_shown(&mut app));
    assert_eq!(pointer_over(&mut app, at(ENEMY_PK)), Some(PointerDisplay::Inactive));
    click(&mut app, at(ENEMY_PK), &[]);
    assert_eq!(interface(&app), AWAIT_PATROL);
    assert_eq!(orders(&app), []);
    assert!(feedback_shown(&mut app));
    let mut texts = app.world_mut().query::<&Text>();
    assert!(texts.iter(app.world()).any(|t| t.0 == INVALID_TARGET_TEXT));

    let ground = Vec2::new(1200.0, 1150.0);
    click(&mut app, ground, &[]);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    let given_orders = orders(&app);
    assert_eq!(given_orders.len(), 1);
    assert!(matches!(given_orders[0].kind, OrderKind::Patrol(p) if near(p, ground)));
    assert!(feedback_shown(&mut app), "still showing");
    updates(&mut app, FEEDBACK_TICKS as usize);
    assert!(!feedback_shown(&mut app), "faded");
}

/// Design change 0066: while Attack awaits its target the other commands are unavailable: their hotkeys and
/// buttons do nothing, and the panel shows them disabled. After Esc, Patrol's hotkey starts Patrol targeting.
#[test]
fn other_commands_are_unavailable_while_targeting() {
    let mut app = game();
    select_mixed(&mut app);
    tap(&mut app, &[], KeyA);
    let mut buttons = app.world_mut().query::<&CommandButton>();
    let patrol = buttons
        .iter(app.world())
        .find(|b| b.slot == slot(1, 1))
        .map(|b| b.shown.look);
    assert_eq!(patrol, Some(ButtonLook::Disabled));
    for key in [KeyS, KeyQ, KeyE, KeyX, KeyR] {
        tap(&mut app, &[], key);
        assert_eq!(interface(&app), AWAIT_ATTACK, "{key:?}");
    }
    click_button(&mut app, slot(1, 1), &[]);
    click_button(&mut app, slot(2, 1), &[]);
    assert_eq!(interface(&app), AWAIT_ATTACK);
    assert_eq!(orders(&app), []);
    assert_eq!(abilities(&app), []);

    tap(&mut app, &[], Escape);
    tap(&mut app, &[], KeyS);
    assert_eq!(interface(&app), AWAIT_PATROL);
}

/// Design change 0067: a drag across own units while Attack awaits its target draws no box, selects nothing,
/// orders nothing, and the target is still awaited. The same drag in DefaultState draws its box and selects them.
#[test]
fn a_drag_while_targeting_does_nothing() {
    let mut app = game();
    click(&mut app, at(5), &[]);
    tap(&mut app, &[], KeyA);
    let (from, to) = (Vec2::new(850.0, 760.0), Vec2::new(1400.0, 840.0));
    assert!(!drag_drawing(&mut app, from, to), "no box");
    assert_eq!(selected(&app), [5]);
    assert_eq!(interface(&app), AWAIT_ATTACK);
    assert_eq!(orders(&app), []);

    tap(&mut app, &[], Escape);
    assert!(drag_drawing(&mut app, from, to));
    assert_eq!(selected(&app), [1, 2, 3, 4]);
}

/// Design change 0068: a targeting click counts at the press point. The button goes down on the enemy, at its
/// edge, and comes up 3 pixels further, over the ground (a click, not a drag: `TARGETING_CLICK_ALLOWANCE_PX`):
/// Attack on the enemy, not AttackMove. And the button goes down on a walking enemy and comes up once it has walked away from
/// under the cursor: Attack on it, as the press found it.
#[test]
fn a_targeting_click_counts_at_the_press_point() {
    let mut app = game();
    click(&mut app, at(1), &[]);
    tap(&mut app, &[], KeyA);
    let mut edge = screen_at(&app, at(ENEMY_PK));
    assert!(over_object(&mut app, edge));
    while over_object(&mut app, edge + Vec2::X) {
        edge += Vec2::X;
    }
    let released = edge + Vec2::new(3.0, 0.0);
    assert!(!over_object(&mut app, released), "the ground");
    input(&mut app).cursor = Some(edge);
    input(&mut app).mouse.press(MouseButton::Left);
    app.update();
    input(&mut app).cursor = Some(released);
    app.update();
    input(&mut app).mouse.release(MouseButton::Left);
    app.update();
    pause(&mut app);
    assert_eq!(orders(&app), [given(&[1], attack(ENEMY_PK), false)]);
    assert_eq!(interface(&app), InterfaceState::DefaultState);

    // Player 2's Peacekeeper walks north, in sight of player 1's scouts.
    let walk = Command::Order {
        units: vec![ObjectId(ENEMY_PK)],
        kind: OrderKind::Move(Target::Point(Vec2::new(2200.0, 600.0))),
        queue: false,
    };
    submit(&mut app, PlayerId(2), walk);
    updates(&mut app, 2);
    tap(&mut app, &[], KeyA);
    let pos = app.world().resource::<SimRes>().object(ObjectId(ENEMY_PK)).unwrap().pos;
    let pressed = screen_at(&app, pos);
    input(&mut app).cursor = Some(pressed);
    input(&mut app).mouse.press(MouseButton::Left);
    app.update();
    assert_eq!(app.world().resource::<PointerLook>().hovered, Some(Relation::Enemy));
    let mut held = 0;
    while app.world().resource::<PointerLook>().hovered.is_some() {
        app.update();
        held += 1;
        assert!(held < 60, "the enemy walks away");
    }
    input(&mut app).mouse.release(MouseButton::Left);
    app.update();
    pause(&mut app);
    assert_eq!(
        orders(&app)[2..],
        [given(&[1], attack(ENEMY_PK), false)],
        "player 2's walk is [1]"
    );
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
    assert_eq!(interface(&app), AWAIT_PATROL);
    let ground = Vec2::new(1200.0, 1150.0);
    click(&mut app, ground, &[]);
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
    assert_eq!(interface(&app), AWAIT_PATROL);
}

/// `commands.md > Unit Command`: Shift queues a targeted command too. A Shift-click keeps the target awaited while
/// Shift is held (design change 0070, which replaced the return to DefaultState after every issuing click); the
/// click helper lets go of Shift after the click, which ends targeting. Shift pressed for the hotkey and held
/// through the click is the same.
#[test]
fn a_shift_click_queues_the_targeted_order() {
    let mut app = game();
    click(&mut app, at(1), &[]);
    tap(&mut app, &[], KeyA);
    click(&mut app, Vec2::new(1000.0, 1000.0), &[ShiftLeft]);
    assert_eq!(interface(&app), InterfaceState::DefaultState, "Shift let go");
    press_keys(&mut app, &[ShiftLeft]);
    input(&mut app).tap_key(KeyQ);
    app.update();
    click(&mut app, Vec2::new(1200.0, 1000.0), &[]);
    let queued = InterfaceState::AwaitingTarget {
        pending: UnitCommand::Move,
        queued: true,
    };
    assert_eq!(interface(&app), queued, "Shift still held");
    release_keys(&mut app, &[ShiftLeft]);
    app.update();
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    tap(&mut app, &[], KeyA);
    click(&mut app, Vec2::new(1000.0, 1100.0), &[]);
    let queued: Vec<bool> = orders(&app).iter().map(|o| o.queue).collect();
    assert_eq!(queued, [true, true, false]);
    assert_eq!(selected(&app), [1], "Shift-clicks while targeting toggle no selection");
}

/// `> ObjectInterfaceState[ObjectEnum]`: the player's change of the Selection or of its ActiveGroup resets the
/// state (design change 0069: Tab, a control group's recall), and so does every control group recall, also one
/// that recalls exactly the current selection (0078; its Verification 1): the panel shows the default commands
/// again. Tab, with nothing to cycle to, changes nothing and leaves the target awaited.
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

    pause(&mut app);
    tap(&mut app, &[], KeyA);
    tap(&mut app, &[], Tab);
    assert_eq!(interface(&app), AWAIT_ATTACK, "one group: Tab changes nothing");
    pause(&mut app);
    tap(&mut app, &[], Digit1);
    assert_eq!(selected(&app), [5], "the same selection");
    assert_eq!(interface(&app), InterfaceState::DefaultState, "any recall resets");
    let mut buttons = app.world_mut().query::<&CommandButton>();
    let looks: Vec<(CommandSlot, ButtonLook)> = buttons.iter(app.world()).map(|b| (b.slot, b.shown.look)).collect();
    assert!(looks.contains(&(slot(1, 0), ButtonLook::Common)), "Attack: {looks:?}");
    assert!(
        !looks.iter().any(|&(_, look)| look == ButtonLook::Back),
        "no Back: {looks:?}"
    );
}

/// Design change 0069: a selected unit that dies (removed here, as a death removes it) leaves the selection, but
/// targeting stays and the click gives the order to the others. A recall of control group 2 cancels it and selects
/// the group. Targeting ends with the whole ActiveGroup: the single unit selected, or the Peacekeepers of a mixed
/// selection, whose Guards are left.
#[test]
fn deaths_end_targeting_only_with_the_whole_active_group() {
    let mut app = game();
    select_box(&mut app, &[3, 4]);
    tap(&mut app, &[ControlLeft], Digit2);
    pause(&mut app);
    click(&mut app, at(OWN_PKS[0]), &[ControlLeft]);
    assert_eq!(selected(&app), OWN_PKS);
    tap(&mut app, &[], KeyA);
    remove(&mut app, 2);
    assert_eq!(selected(&app), [1, 5]);
    assert_eq!(interface(&app), AWAIT_ATTACK);
    click(&mut app, at(ENEMY_PK), &[]);
    assert_eq!(orders(&app), [given(&[1, 5], attack(ENEMY_PK), false)]);

    tap(&mut app, &[], KeyA);
    tap(&mut app, &[], Digit2);
    assert_eq!(selected(&app), [3, 4], "control group 2 recalled");
    assert_eq!(interface(&app), InterfaceState::DefaultState);

    click(&mut app, at(5), &[]);
    tap(&mut app, &[], KeyA);
    remove(&mut app, 5);
    assert!(selected(&app).is_empty());
    assert_eq!(interface(&app), InterfaceState::DefaultState);

    drag(&mut app, Vec2::new(850.0, 760.0), Vec2::new(1400.0, 840.0));
    assert_eq!(selected(&app), [1, 3, 4]);
    tap(&mut app, &[], KeyA);
    remove(&mut app, 1);
    assert_eq!(selected(&app), [3, 4]);
    assert_eq!(
        interface(&app),
        InterfaceState::DefaultState,
        "the Peacekeepers are gone"
    );
    assert_eq!(orders(&app).len(), 1);
}
