//! The targeting click allowance (`systems/controls/interface.md > AwaitingTarget > Dragging while targeting`, design
//! change 0073): while a target is awaited, a press and release that moved at most
//! `InputTuning::targeting_click_allowance_px` is a targeting click, at the press point; one that moved further is a
//! drag, which does nothing (0067). Elsewhere the normal drag threshold, `InputTuning::drag_threshold_px`, is
//! unchanged.

use bevy::prelude::*;
use sc_sim::{ObjectId, OrderKind};
use space_crystals::command_ui::{InterfaceState, PanelCommand};

use super::*;
use KeyCode::KeyA;

const AWAIT_ATTACK: InterfaceState = InterfaceState::awaiting(PanelCommand::Attack);

/// The left button goes down at the view point `view`, the cursor moves by each of `moves` (logical pixels, from
/// the press point) a frame each, and the button comes up there. Then a [`pause`]. Whether the selection box was
/// drawn while the button was held.
fn press_moving(app: &mut App, view: Vec2, moves: &[Vec2]) -> bool {
    let pressed = screen_at(app, view);
    input(app).cursor = Some(pressed);
    input(app).mouse.press(MouseButton::Left);
    app.update();
    let mut drawn = false;
    for &by in moves {
        input(app).cursor = Some(pressed + by);
        app.update();
        drawn |= box_drawn(app);
    }
    input(app).mouse.release(MouseButton::Left);
    app.update();
    pause(app);
    drawn
}

/// 0073 Verification 1: Attack (A), then the button goes down on an enemy, the mouse moves about 10 pixels (more
/// than the normal drag threshold) and the button comes up: Attack on that enemy. So it is up to the allowance,
/// and no further.
#[test]
fn a_targeting_click_may_move_up_to_the_allowance() {
    const {
        assert!(
            space_crystals::tuning::input::BUILTIN.drag_threshold_px < 10.0
                && 10.0 < space_crystals::tuning::input::BUILTIN.targeting_click_allowance_px
        )
    };
    let mut app = game();
    click(&mut app, at(1), &[]);
    tap(&mut app, &[], KeyA);
    assert!(!press_moving(&mut app, at(ENEMY_PK), &[Vec2::new(10.0, 0.0)]));
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    assert_eq!(
        orders(&app),
        [given(&[1], OrderKind::Attack(ObjectId(ENEMY_PK)), false)]
    );

    tap(&mut app, &[], KeyA);
    let just_inside = Vec2::new(
        0.0,
        space_crystals::tuning::input::BUILTIN.targeting_click_allowance_px - 1.0,
    );
    press_moving(&mut app, at(ENEMY_GUARD), &[just_inside]);
    assert_eq!(
        orders(&app)[1..],
        [given(&[1], OrderKind::Attack(ObjectId(ENEMY_GUARD)), false)]
    );

    tap(&mut app, &[], KeyA);
    let just_beyond = Vec2::new(
        0.0,
        space_crystals::tuning::input::BUILTIN.targeting_click_allowance_px + 1.0,
    );
    press_moving(&mut app, at(ENEMY_GUARD), &[just_beyond]);
    assert_eq!(interface(&app), AWAIT_ATTACK, "a drag");
    assert_eq!(orders(&app).len(), 2);
}

/// 0073 Verification 2: Attack (A), then the button goes down on an enemy and drags about 40 pixels: nothing is
/// given, no box is drawn, and the target is still awaited. A drag that comes back to the press point before the
/// button comes up is a drag too: the allowance is how far the pointer went, as for the normal threshold.
#[test]
fn a_drag_beyond_the_allowance_does_nothing() {
    let mut app = game();
    click(&mut app, at(1), &[]);
    tap(&mut app, &[], KeyA);
    let drag = [Vec2::new(20.0, 10.0), Vec2::new(40.0, 0.0)];
    assert!(!press_moving(&mut app, at(ENEMY_PK), &drag), "no box");
    assert_eq!(interface(&app), AWAIT_ATTACK);
    assert_eq!(orders(&app), []);
    assert_eq!(selected(&app), [1]);

    let there_and_back = [Vec2::new(40.0, 0.0), Vec2::ZERO];
    assert!(!press_moving(&mut app, at(ENEMY_PK), &there_and_back));
    assert_eq!(interface(&app), AWAIT_ATTACK);
    assert_eq!(orders(&app), []);
}

/// 0073 Verification 3: without targeting, a 10 pixel drag is a drag as before, beyond the normal threshold: from
/// the ground just west of a Guard, where a click picks nothing, it draws the selection box and selects the Guard
/// by it. A move within the threshold draws none: a click.
#[test]
fn without_targeting_the_normal_threshold_holds() {
    let mut app = game();
    click(&mut app, at(1), &[]);
    let west = at(3) - Vec2::new(24.0, 0.0);
    input(&mut app).cursor = Some(screen_at(&app, west));
    app.update();
    assert_eq!(app.world().resource::<PointerLook>().hovered, None, "the ground");
    assert!(
        !press_moving(
            &mut app,
            west,
            &[Vec2::new(
                space_crystals::tuning::input::BUILTIN.drag_threshold_px - 1.0,
                0.0
            )]
        ),
        "a click"
    );
    assert!(press_moving(&mut app, west, &[Vec2::new(10.0, 0.0)]), "a box");
    assert_eq!(selected(&app), [3]);
}
