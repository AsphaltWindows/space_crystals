//! Shift while targeting (`systems/controls/selection_and_hud.md > AwaitingTarget > Shift while targeting`, design
//! change 0070): a Shift-click on a valid target queues the PendingCommand and the target is still awaited while Shift
//! is held; letting go of Shift after a queued click ends targeting. A click without Shift issues and ends it at once.
//! Shift is pressed and let go explicitly, apart from the clicks.

use bevy::prelude::*;
use sc_sim::{ObjectId, OrderKind, SimEvent};
use space_crystals::command_ui::feedback::{FEEDBACK_TICKS, TargetFeedback};
use space_crystals::command_ui::{InterfaceState, PanelCommand};

use super::*;
use KeyCode::{Escape, KeyA, KeyQ, ShiftLeft};

const AWAIT_ATTACK: InterfaceState = InterfaceState::awaiting(PanelCommand::Attack);
/// Attack awaiting its target after a Shift-click queued it.
const QUEUED_ATTACK: InterfaceState = InterfaceState::AwaitingTarget {
    pending: PanelCommand::Attack,
    queued: true,
};

fn attack(id: u32) -> OrderKind {
    OrderKind::Attack(ObjectId(id))
}

/// Whether the invalid-target feedback shows.
fn feedback_shown(app: &App) -> bool {
    app.world().resource::<TargetFeedback>().since.is_some()
}

/// A left click at the view point `view` that holds and lets go of no key itself: the button goes down in one
/// frame and up in the next, a frame each.
fn bare_click(app: &mut App, view: Vec2) {
    let screen = screen_at(app, view);
    input(app).cursor = Some(screen);
    input(app).mouse.press(MouseButton::Left);
    app.update();
    input(app).mouse.release(MouseButton::Left);
    app.update();
}

/// 0070 Verification 1, with enemy units for the buildings, which M1 doesn't have: Attack (A), then Shift held
/// over three clicks on enemies. Each click queues one Attack for both Peacekeepers and the target is still
/// awaited; it ends as Shift is let go. The units attack the three in the order of the clicks.
#[test]
fn shift_clicks_queue_attacks_until_shift_is_let_go() {
    let mut app = game();
    // A third enemy, in sight of player 1's scout 13.
    let third = spawn(&mut app, ObjectKind::Guard, Some(PlayerId(2)), Vec2::new(2000.0, 650.0));
    select_box(&mut app, &[1, 2]);
    tap(&mut app, &[], KeyA);
    press_keys(&mut app, &[ShiftLeft]);
    app.update();
    let targets = [ENEMY_PK, ENEMY_GUARD, third];
    for (n, &target) in targets.iter().enumerate() {
        let target_at = pos(&app, target);
        bare_click(&mut app, target_at);
        pause(&mut app);
        assert_eq!(interface(&app), QUEUED_ATTACK, "after click {n}");
        assert_eq!(orders(&app).len(), n + 1);
    }
    release_keys(&mut app, &[ShiftLeft]);
    app.update();
    assert_eq!(
        interface(&app),
        InterfaceState::DefaultState,
        "ended as Shift was let go"
    );
    let queued: Vec<Given> = targets.iter().map(|&t| given(&[1, 2], attack(t), true)).collect();
    assert_eq!(orders(&app), queued);
    let unit = object_orders(&app, 1);
    assert_eq!(unit.current.expect("attacking").order.kind, attack(ENEMY_PK));
    let queue: Vec<OrderKind> = unit.queue.iter().map(|o| o.kind).collect();
    assert_eq!(queue, [attack(ENEMY_GUARD), attack(third)]);

    // Nobody fights back (auto-targeting is off): the first shot of the Peacekeepers at each target, in order.
    let mut first_shots = Vec::new();
    for event in run_collecting(&mut app, 900) {
        if let SimEvent::Attacked { attacker, target, .. } = event
            && [1, 2].contains(&attacker.0)
            && !first_shots.contains(&target.0)
        {
            first_shots.push(target.0);
        }
    }
    assert_eq!(first_shots, targets);
}

/// 0070 Verification 2: Move (Q), then Shift held over three clicks on the ground: three waypoints, each one Move
/// queued for both units (one group, its magic box built from where the group's earlier orders leave it). The
/// units walk them in order.
#[test]
fn shift_clicks_queue_move_waypoints() {
    let mut app = game();
    select_box(&mut app, &[1, 2]);
    tap(&mut app, &[], KeyQ);
    press_keys(&mut app, &[ShiftLeft]);
    app.update();
    let waypoints = [
        Vec2::new(1000.0, 1000.0),
        Vec2::new(1250.0, 1000.0),
        Vec2::new(1250.0, 1180.0),
    ];
    for waypoint in waypoints {
        bare_click(&mut app, waypoint);
        let queued = InterfaceState::AwaitingTarget {
            pending: PanelCommand::Move,
            queued: true,
        };
        assert_eq!(interface(&app), queued);
    }
    release_keys(&mut app, &[ShiftLeft]);
    app.update();
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    let given_orders = orders(&app);
    assert_eq!(given_orders.len(), 3, "{given_orders:?}");
    for (order, waypoint) in given_orders.iter().zip(waypoints) {
        assert_eq!((order.units.as_slice(), order.queue), ([1, 2].as_slice(), true));
        assert!(near(point_of(order.kind), waypoint), "{order:?}");
    }

    // The waypoints the units' current orders go to, in turn, until they have walked them all.
    let mut walked: [Vec<Vec2>; 2] = Default::default();
    for tick in 0.. {
        let units = [object_orders(&app, 1), object_orders(&app, 2)];
        if units.iter().all(|u| u.current.is_none() && u.queue.is_empty()) {
            break;
        }
        for (unit, walked) in units.iter().zip(&mut walked) {
            let point = unit.current.as_ref().map(|c| point_of(c.order.kind));
            if let Some(point) = point
                && walked.last() != Some(&point)
            {
                walked.push(point);
            }
        }
        assert!(tick < 400, "still walking: {walked:?}");
        app.update();
    }
    for walked in walked {
        assert_eq!(walked.len(), 3, "{walked:?}");
        assert!(walked.iter().zip(waypoints).all(|(w, p)| near(*w, p)), "{walked:?}");
    }
}

/// 0070 Verification 3: Attack (A) and a click on an enemy without Shift: the Attack is given, not queued, and
/// targeting ends in the frame of the click.
#[test]
fn a_click_without_shift_issues_and_ends_targeting() {
    let mut app = game();
    click(&mut app, at(1), &[]);
    tap(&mut app, &[], KeyA);
    bare_click(&mut app, at(ENEMY_PK));
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    assert_eq!(orders(&app), [given(&[1], attack(ENEMY_PK), false)]);
}

/// 0070 Verification 4: with Shift held, a click on a target the command can't use (the crystal patch, which
/// can't be destroyed: the Inactive pointer) is ignored with feedback, and the target is still awaited, before a
/// queued click and after one; Shift let go after the queued click ends targeting.
#[test]
fn an_invalid_target_with_shift_held_keeps_targeting() {
    let mut app = game();
    click(&mut app, at(1), &[]);
    tap(&mut app, &[], KeyA);
    press_keys(&mut app, &[ShiftLeft]);
    assert_eq!(pointer_over(&mut app, at(PATCH)), Some(PointerDisplay::Inactive));
    bare_click(&mut app, at(PATCH));
    assert!(feedback_shown(&app));
    assert_eq!(interface(&app), AWAIT_ATTACK);
    assert_eq!(orders(&app), []);
    pause(&mut app);

    bare_click(&mut app, at(ENEMY_PK));
    assert_eq!(interface(&app), QUEUED_ATTACK);
    updates(&mut app, FEEDBACK_TICKS as usize);
    assert!(!feedback_shown(&app), "faded");
    bare_click(&mut app, at(PATCH));
    assert!(feedback_shown(&app));
    assert_eq!(interface(&app), QUEUED_ATTACK, "still awaited");
    assert_eq!(orders(&app), [given(&[1], attack(ENEMY_PK), true)]);

    release_keys(&mut app, &[ShiftLeft]);
    app.update();
    assert_eq!(interface(&app), InterfaceState::DefaultState);
}

/// The design's TBD, built as its proposed default: Shift pressed and let go before any target was clicked
/// changes nothing; targeting simply continues, and a click without Shift then issues and ends it.
#[test]
fn shift_let_go_before_a_click_keeps_targeting() {
    let mut app = game();
    click(&mut app, at(1), &[]);
    tap(&mut app, &[], KeyA);
    press_keys(&mut app, &[ShiftLeft]);
    updates(&mut app, 2);
    release_keys(&mut app, &[ShiftLeft]);
    updates(&mut app, 2);
    assert_eq!(interface(&app), AWAIT_ATTACK);
    bare_click(&mut app, at(ENEMY_PK));
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    assert_eq!(orders(&app), [given(&[1], attack(ENEMY_PK), false)]);
}

/// Shift counts as the button goes down, where the click does (design change 0068): Shift let go while the button
/// is still held, after a queued click, still queues that click's order, and targeting ends after it.
#[test]
fn shift_counts_as_the_button_goes_down() {
    let mut app = game();
    click(&mut app, at(1), &[]);
    tap(&mut app, &[], KeyA);
    press_keys(&mut app, &[ShiftLeft]);
    bare_click(&mut app, at(ENEMY_PK));
    input(&mut app).cursor = Some(screen_at(&app, at(ENEMY_GUARD)));
    input(&mut app).mouse.press(MouseButton::Left);
    app.update();
    release_keys(&mut app, &[ShiftLeft]);
    app.update();
    assert_eq!(interface(&app), QUEUED_ATTACK, "the press goes first");
    input(&mut app).mouse.release(MouseButton::Left);
    app.update();
    app.update();
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    assert_eq!(
        orders(&app),
        [
            given(&[1], attack(ENEMY_PK), true),
            given(&[1], attack(ENEMY_GUARD), true)
        ]
    );
}

/// The cancel rules are unchanged (`systems/controls/selection_and_hud.md > AwaitingTarget`): Esc and a right-click
/// cancel queued targeting while Shift is still held, and the right-click gives no order. What was queued stays queued.
#[test]
fn esc_and_right_click_cancel_queued_targeting() {
    let mut app = game();
    click(&mut app, at(1), &[]);
    let cancels: [&dyn Fn(&mut App); 2] = [&|app| tap(app, &[], Escape), &|app| {
        right_click(app, Vec2::new(1000.0, 1000.0), &[])
    }];
    for cancel in cancels {
        tap(&mut app, &[], KeyA);
        press_keys(&mut app, &[ShiftLeft]);
        bare_click(&mut app, at(ENEMY_PK));
        assert_eq!(interface(&app), QUEUED_ATTACK);
        cancel(&mut app);
        assert_eq!(interface(&app), InterfaceState::DefaultState);
        release_keys(&mut app, &[ShiftLeft]);
        pause(&mut app);
        assert_eq!(interface(&app), InterfaceState::DefaultState);
    }
    assert_eq!(
        orders(&app),
        [given(&[1], attack(ENEMY_PK), true), given(&[1], attack(ENEMY_PK), true)],
        "the right-click moved nobody"
    );
}
