use super::*;
use crate::selection::MAX_SELECTED;
use crate::selection::pick::Relation;
use sc_sim::ObjectKind;

/// Units in a row along y = 100, 50 su apart: object `n` at x = 50·n.
fn row(units: &[(u32, ObjectKind, Relation)]) -> Selectables {
    let unit = |&(n, kind, relation): &(u32, ObjectKind, Relation)| Selectable {
        id: ObjectId(n),
        kind,
        unfinished: false,
        relation,
        drawn: at(n),
        rect: Rect::from_center_half_size(at(n), Vec2::splat(12.0)),
        depth: at(n).y,
    };
    Selectables::from_objects(units.iter().map(unit).collect())
}

const PK: ObjectKind = ObjectKind::Peacekeeper;
const GUARD: ObjectKind = ObjectKind::Guard;

fn at(n: u32) -> Vec2 {
    Vec2::new(50.0 * n as f32, 100.0)
}

fn ids(state: &ControlState) -> Vec<u32> {
    state.selection.ids().map(|i| i.0).collect()
}

const SHIFT: ClickRoles = ClickRoles {
    toggle: true,
    same_type: false,
    center: false,
};
const CTRL: ClickRoles = ClickRoles {
    toggle: false,
    same_type: true,
    center: false,
};
const CTRL_SHIFT: ClickRoles = ClickRoles {
    toggle: true,
    same_type: true,
    center: false,
};
const PLAIN: ClickRoles = ClickRoles {
    toggle: false,
    same_type: false,
    center: false,
};

#[test]
fn clicks_follow_the_map_click_rules_and_the_constraints() {
    use Relation::{Enemy, Neutral, Own};
    let objects = row(&[
        (1, PK, Own),
        (2, PK, Own),
        (3, GUARD, Own),
        (4, PK, Enemy),
        (5, PK, Own),
        (9, PK, Own),
    ]);
    // Unit 9 (x = 450) is off the world view.
    let visible = Rect::new(0.0, 0.0, 400.0, 300.0);
    let mut state = ControlState::default();
    assert!(state.click(&objects, at(1), PLAIN, false, visible));
    assert_eq!(ids(&state), [1]);
    assert!(state.click(&objects, at(3), SHIFT, false, visible));
    assert_eq!(ids(&state), [1, 3]);
    assert!(state.click(&objects, at(1), SHIFT, false, visible));
    assert_eq!(ids(&state), [3], "Shift toggles a selected unit off");
    // Ctrl and a double-click: every own Peacekeeper on screen, not the enemy's nor the one off screen.
    assert!(state.click(&objects, at(2), CTRL, false, visible));
    assert_eq!(ids(&state), [1, 2, 5]);
    state.click(&objects, at(3), PLAIN, false, visible);
    assert!(state.click(&objects, at(5), PLAIN, true, visible));
    assert_eq!(ids(&state), [1, 2, 5]);
    // An enemy is selected alone, also by Shift or Ctrl (0048). Shift then adds no own unit to it (0053), a plain
    // click replaces it.
    assert!(state.click(&objects, at(4), SHIFT, false, visible));
    assert_eq!(ids(&state), [4]);
    assert!(!state.click(&objects, at(4), CTRL, false, visible));
    assert!(!state.click(&objects, at(2), SHIFT, false, visible));
    assert_eq!(ids(&state), [4]);
    assert!(state.click(&objects, at(2), PLAIN, false, visible));
    assert_eq!(ids(&state), [2]);
    // Empty ground: a plain click clears, a modified one keeps the selection (0051, 0052).
    let nowhere = Vec2::new(25.0, 250.0);
    assert!(!state.click(&objects, nowhere, SHIFT, false, visible));
    assert!(!state.click(&objects, nowhere, CTRL, true, visible));
    assert!(!state.click(&objects, nowhere, CTRL_SHIFT, false, visible));
    assert_eq!(ids(&state), [2]);
    assert!(state.click(&objects, nowhere, PLAIN, false, visible));
    assert!(state.selection.is_empty());
    let neutral = row(&[(1, ObjectKind::CrystalPatch, Neutral)]);
    state.click(&neutral, at(1), CTRL, true, visible);
    assert_eq!(ids(&state), [1], "a resource is selected alone");
}

/// `> MapClickSelection > Ctrl+Shift-click own unit` (design change 0052): adds all own units of that type on
/// screen, and removes nothing.
#[test]
fn ctrl_shift_clicks_add_the_own_units_of_that_type_on_screen() {
    use Relation::{Enemy, Own};
    let objects = row(&[
        (1, PK, Own),
        (2, PK, Own),
        (3, GUARD, Own),
        (4, PK, Enemy),
        (5, PK, Own),
        (9, PK, Own),
    ]);
    let visible = Rect::new(0.0, 0.0, 400.0, 300.0);
    let mut state = ControlState::default();
    state.click(&objects, at(3), PLAIN, false, visible);
    assert!(state.click(&objects, at(2), CTRL_SHIFT, false, visible));
    assert_eq!(
        ids(&state),
        [1, 2, 3, 5],
        "not the enemy's nor the one off screen; the Guard stays"
    );
    assert_eq!(
        state.selection.active().map(|k| k.kind),
        Some(GUARD),
        "the ActiveGroup stays"
    );
    assert!(
        !state.click(&objects, at(1), CTRL_SHIFT, false, visible),
        "nothing to add"
    );
    assert_eq!(ids(&state), [1, 2, 3, 5]);
    // A double-click counts as a Ctrl-click, so with Shift held it adds too (placeholder reading).
    state.click(&objects, at(3), PLAIN, false, visible);
    assert!(state.click(&objects, at(5), SHIFT, true, visible));
    assert_eq!(ids(&state), [1, 2, 3, 5]);
    // On an enemy it selects that enemy alone, as any click does (0048), and it adds nothing to it (0053).
    assert!(state.click(&objects, at(4), CTRL_SHIFT, false, visible));
    assert_eq!(ids(&state), [4]);
    assert!(!state.click(&objects, at(1), CTRL_SHIFT, false, visible));
    assert!(!state.click(&objects, at(1), SHIFT, true, visible));
    assert_eq!(ids(&state), [4]);
}

/// `> BoxSelection` (design changes 0049, 0051 and 0053): a plain box replaces the selection, a Shift + box adds to
/// it.
#[test]
fn shift_boxes_add_to_the_selection_and_plain_boxes_replace_it() {
    use Relation::{Enemy, Neutral, Own};
    let objects = row(&[
        (1, PK, Own),
        (2, PK, Own),
        (3, GUARD, Own),
        (4, GUARD, Own),
        (6, PK, Enemy),
        (8, ObjectKind::CrystalPatch, Neutral),
    ]);
    // A box around the objects `first` to `last` of the row, and one around nothing.
    let around = |first: u32, last: u32| Rect::new(at(first).x - 20.0, 50.0, at(last).x + 20.0, 150.0);
    let nothing = Rect::new(0.0, 200.0, 400.0, 300.0);
    let mut state = ControlState::default();
    assert!(state.select_box(&objects, around(1, 2), PLAIN));
    assert_eq!(ids(&state), [1, 2]);
    // Shift adds the box's own units and removes nothing, also where the box covers selected ones.
    assert!(state.select_box(&objects, around(2, 4), SHIFT));
    assert_eq!(ids(&state), [1, 2, 3, 4]);
    assert!(!state.select_box(&objects, around(1, 3), SHIFT));
    assert_eq!(
        state.selection.active().map(|k| k.kind),
        Some(PK),
        "the ActiveGroup stays"
    );
    // What can't join own units isn't added, an enemy or a resource (0048); an empty Shift + box clears nothing.
    assert!(!state.select_box(&objects, around(6, 6), SHIFT));
    assert!(!state.select_box(&objects, around(8, 8), SHIFT));
    assert!(!state.select_box(&objects, nothing, SHIFT));
    assert_eq!(ids(&state), [1, 2, 3, 4]);
    // A box without Shift replaces the selection (Ctrl changes nothing, a placeholder).
    assert!(state.select_box(&objects, around(6, 8), CTRL));
    assert_eq!(ids(&state), [6]);
    // Nothing is added to another player's object, which stays selected (0053).
    assert!(!state.select_box(&objects, around(1, 4), SHIFT));
    assert_eq!(ids(&state), [6]);
    // A plain box over nothing clears the selection (0051); to an empty one, Shift adds what a plain box selects.
    assert!(state.select_box(&objects, nothing, PLAIN));
    assert!(state.selection.is_empty());
    assert!(state.select_box(&objects, around(6, 8), SHIFT));
    assert_eq!(ids(&state), [6]);
}

/// An add removes nothing, not even from a full selection, which takes no more.
#[test]
fn a_full_selection_takes_no_more() {
    let max = MAX_SELECTED as u32;
    let units: Vec<(u32, ObjectKind, Relation)> = (1..=max + 1).map(|n| (n, PK, Relation::Own)).collect();
    let objects = row(&units);
    let mut state = ControlState {
        selection: selection_of((2..=max + 1).map(ObjectId), &objects),
        ..ControlState::default()
    };
    assert_eq!(state.selection.len(), MAX_SELECTED);
    assert!(!state.add_to_selection([ObjectId(1)], &objects));
    assert!(!state.selection.contains(ObjectId(1)) && state.selection.contains(ObjectId(max + 1)));
}

#[test]
fn portrait_clicks_follow_the_panels_rules() {
    use Relation::Own;
    let objects = row(&[(1, PK, Own), (2, PK, Own), (3, GUARD, Own), (4, GUARD, Own)]);
    let all = |state: &mut ControlState| state.selection = selection_of([1, 2, 3, 4].map(ObjectId), &objects);
    let mut state = ControlState::default();
    all(&mut state);
    let alt = ClickRoles { center: true, ..SHIFT };
    assert_eq!(state.click_portrait(&objects, ObjectId(3), alt), Some(at(3)));
    assert_eq!(ids(&state), [1, 2, 3, 4], "Alt only centers");
    assert_eq!(state.click_portrait(&objects, ObjectId(3), SHIFT), None);
    assert_eq!(ids(&state), [1, 2, 4]);
    state.click_portrait(&objects, ObjectId(2), CTRL);
    assert_eq!(ids(&state), [1, 2]);
    all(&mut state);
    state.click_portrait(&objects, ObjectId(4), CTRL_SHIFT);
    assert_eq!(ids(&state), [1, 2]);
    state.click_portrait(&objects, ObjectId(1), PLAIN);
    assert_eq!(ids(&state), [1]);
    state.click_portrait(&objects, ObjectId(1), SHIFT);
    assert!(state.selection.is_empty(), "the last one removed empties the selection");
    assert_eq!(state.click_portrait(&objects, ObjectId(1), alt), None, "not selected");
}

/// `> ObjectInterfaceState` (design change 0078): a control group's recall resets the ObjectInterfaceState, also one
/// that recalls exactly the current selection; the key of an empty group changes nothing (placeholder rule).
#[test]
fn a_recall_resets_the_interface_state() {
    let objects = row(&[(1, PK, Relation::Own), (2, GUARD, Relation::Own)]);
    let (one, two) = (ControlGroup::new(1).unwrap(), ControlGroup::new(2).unwrap());
    let awaiting = InterfaceState::awaiting(crate::command_ui::PanelCommand::Attack);
    let mut state = ControlState {
        selection: selection_of([ObjectId(1), ObjectId(2)], &objects),
        ..ControlState::default()
    };
    assert!(state.assign_group(one, &objects));
    let before = state.selection.clone();
    state.interface = awaiting;
    assert!(state.recall_group(one, &objects));
    assert_eq!(state.selection, before, "the same selection");
    assert_eq!(state.interface, InterfaceState::DefaultState);
    state.interface = awaiting;
    assert!(!state.recall_group(two, &objects), "an empty group");
    assert_eq!(state.interface, awaiting);
}

/// `> ControlGroups > Groups` (design change 0056): only own objects go into control groups. Assign and Add with
/// another player's or a neutral object selected change nothing, not even an empty group.
#[test]
fn control_groups_take_only_own_objects() {
    use Relation::{Enemy, Neutral, Own};
    let objects = row(&[
        (1, PK, Own),
        (2, GUARD, Own),
        (3, PK, Enemy),
        (4, ObjectKind::CrystalPatch, Neutral),
    ]);
    let (one, two) = (ControlGroup::new(1).unwrap(), ControlGroup::new(2).unwrap());
    let mut state = ControlState::default();
    assert!(!state.recall_group(one, &objects), "an empty group recalls nothing");
    state.selection = selection_of([ObjectId(1)], &objects);
    assert!(state.assign_group(one, &objects));
    state.selection = selection_of([ObjectId(2)], &objects);
    assert!(state.add_to_group(one, &objects));
    assert_eq!(state.groups.get(one), [ObjectId(1), ObjectId(2)]);
    for other in [3, 4] {
        state.selection = selection_of([ObjectId(other)], &objects);
        assert!(!state.assign_group(one, &objects) && !state.add_to_group(one, &objects));
        assert!(!state.assign_group(two, &objects) && !state.add_to_group(two, &objects));
        assert_eq!(state.groups.get(one), [ObjectId(1), ObjectId(2)]);
        assert!(state.groups.get(two).is_empty());
    }
    assert!(!state.recall_group(two, &objects));
    assert_eq!(ids(&state), [4]);
    assert!(state.recall_group(one, &objects));
    assert_eq!(ids(&state), [1, 2]);
    // Nothing selected, Assign empties the group.
    state.selection = Selection::EMPTY;
    assert!(state.assign_group(one, &objects));
    assert!(state.groups.get(one).is_empty());
    assert_eq!(
        drawn_centroid([ObjectId(1), ObjectId(2)], &objects),
        Some(Vec2::new(75.0, 100.0))
    );
}
