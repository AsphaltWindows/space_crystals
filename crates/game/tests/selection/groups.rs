//! `systems/controls/selection.md > ControlGroups` (with their default keys) and
//! `systems/controls/selection.md > GroupCycling`.

use bevy::prelude::*;

use super::*;
use KeyCode::{ControlLeft, Digit1, Digit2, Digit3, ShiftLeft, Tab};

#[test]
fn control_groups_assign_add_and_recall() {
    let mut app = game();
    click(&mut app, at(1), &[ControlLeft]);
    tap(&mut app, &[ControlLeft], Digit1);
    assert_eq!(control_group(&app, 1), OWN_PKS, "Ctrl assigns");
    pause(&mut app);
    click(&mut app, at(3), &[]);
    tap(&mut app, &[ShiftLeft], Digit1);
    assert_eq!(control_group(&app, 1), ids([1, 2, 5, 3]), "Shift adds");
    pause(&mut app);
    tap(&mut app, &[ShiftLeft], Digit1);
    assert_eq!(control_group(&app, 1), ids([1, 2, 5, 3]), "without duplicates");
    pause(&mut app);
    assert_eq!(selected(&app), [3], "assigning and adding don't change the selection");
    tap(&mut app, &[ControlLeft], Digit2);
    pause(&mut app);
    click(&mut app, Vec2::new(1500.0, 1200.0), &[]);
    assert!(selected(&app).is_empty());
    tap(&mut app, &[], Digit1);
    assert_eq!(selected(&app), ids([1, 2, 5, 3]), "a number recalls");
    assert_eq!(
        active_group(&app),
        OWN_PKS,
        "the first group of the recalled selection is active"
    );
    pause(&mut app);
    tap(&mut app, &[], Digit2);
    assert_eq!(selected(&app), [3]);
    pause(&mut app);
    // Ctrl + a number with nothing selected empties the group; recalling an empty group changes nothing
    // (placeholder rule).
    click(&mut app, Vec2::new(1500.0, 1200.0), &[]);
    tap(&mut app, &[ControlLeft], Digit2);
    assert!(control_group(&app, 2).is_empty());
    pause(&mut app);
    click(&mut app, at(4), &[]);
    tap(&mut app, &[], Digit2);
    tap(&mut app, &[], Digit3);
    assert_eq!(selected(&app), [4]);
}

#[test]
fn a_double_tap_recalls_the_group_and_centers_the_camera_on_it() {
    let mut app = game();
    drag(&mut app, Vec2::new(850.0, 750.0), Vec2::new(1100.0, 850.0));
    assert_eq!(selected(&app), [1, 2]);
    tap(&mut app, &[ControlLeft], Digit1);
    pause(&mut app);
    click(&mut app, at(ENEMY_PK), &[]);
    // The camera moves away, a single press only recalls, a second press soon after also centers.
    let away = Vec2::new(2800.0, 2000.0);
    app.world_mut().resource_mut::<CameraView>().center = away;
    app.update();
    tap(&mut app, &[], Digit1);
    assert_eq!(selected(&app), [1, 2]);
    assert_eq!(camera_center(&app), away);
    tap(&mut app, &[], Digit1);
    assert_eq!(selected(&app), [1, 2]);
    assert_eq!(camera_center(&app), (at(1) + at(2)) / 2.0);
    // The recalled units are on screen now: a click finds them where they are.
    pause(&mut app);
    click(&mut app, at(2), &[]);
    assert_eq!(selected(&app), [2]);
}

/// `systems/controls/selection.md > ControlGroups > Groups` (design change 0056): only own objects go into
/// control groups. With an enemy or a neutral object selected, Assign and Add change nothing: a group keeps what it
/// holds, an empty one stays empty.
#[test]
fn only_own_objects_go_into_control_groups() {
    let mut app = game();
    click(&mut app, at(1), &[]);
    tap(&mut app, &[ControlLeft], Digit1);
    pause(&mut app);
    for inspected in [ENEMY_PK, PATCH] {
        // Select an enemy (or a neutral object) and press Ctrl + 1: group 1 is unchanged.
        click(&mut app, at(inspected), &[]);
        tap(&mut app, &[ControlLeft], Digit1);
        assert_eq!(control_group(&app, 1), [1], "Ctrl + 1 leaves group 1 as it was");
        pause(&mut app);
        tap(&mut app, &[ShiftLeft], Digit1);
        assert_eq!(control_group(&app, 1), [1], "the enemy doesn't join it");
        pause(&mut app);
        tap(&mut app, &[ControlLeft], Digit2);
        pause(&mut app);
        tap(&mut app, &[ShiftLeft], Digit2);
        assert!(control_group(&app, 2).is_empty(), "nor does it make a group alone");
        pause(&mut app);
        assert_eq!(selected(&app), [inspected]);
    }
    // Own units still go in, and the groups recall them.
    click(&mut app, at(2), &[]);
    tap(&mut app, &[ShiftLeft], Digit2);
    assert_eq!(control_group(&app, 2), [2]);
    pause(&mut app);
    tap(&mut app, &[], Digit1);
    assert_eq!(selected(&app), [1]);
    pause(&mut app);
    tap(&mut app, &[], Digit2);
    assert_eq!(selected(&app), [2]);
}

#[test]
fn tab_and_shift_tab_cycle_the_active_group_and_wrap_around() {
    let mut app = game();
    drag(&mut app, Vec2::new(850.0, 750.0), Vec2::new(1400.0, 1150.0));
    assert_eq!(selected(&app), ids([1, 2, 3, 4, 5]));
    assert_eq!(active_group(&app), OWN_PKS);
    tap(&mut app, &[], Tab);
    assert_eq!(active_group(&app), OWN_GUARDS);
    tap(&mut app, &[], Tab);
    assert_eq!(active_group(&app), OWN_PKS, "past the last group, the first");
    tap(&mut app, &[ShiftLeft], Tab);
    assert_eq!(active_group(&app), OWN_GUARDS, "before the first group, the last");
    tap(&mut app, &[ShiftLeft], Tab);
    assert_eq!(active_group(&app), OWN_PKS);
    assert_eq!(selected(&app), ids([1, 2, 3, 4, 5]), "cycling keeps the selection");
    // The active group stays while it has members; when the last one leaves, the first group is active.
    tap(&mut app, &[], Tab);
    click(&mut app, at(3), &[ShiftLeft]);
    assert_eq!(active_group(&app), [4]);
    click(&mut app, at(4), &[ShiftLeft]);
    assert_eq!(active_group(&app), OWN_PKS);
}
