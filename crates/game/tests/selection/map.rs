//! Selecting on the map: `> MapClickSelection` (design change 0011), `> BoxSelection` and `> Selection >
//! Constraints`.

use std::collections::BTreeMap;

use bevy::prelude::*;
use sc_sim::{CornerPos, ObjectKind, SlotId, Spawn, SpawnAt, scenarios};
use space_crystals::coords::lift;
use space_crystals::selection::markers::DragBoxEdge;

use super::*;
use KeyCode::{ControlLeft, ShiftLeft};

#[test]
fn a_click_selects_one_object_and_shift_toggles_own_units() {
    let mut app = game();
    click(&mut app, at(1), &[]);
    assert_eq!(selected(&app), [1]);
    click(&mut app, at(2), &[]);
    assert_eq!(selected(&app), [2], "a click replaces the selection");
    click(&mut app, at(3), &[ShiftLeft]);
    click(&mut app, at(5), &[ShiftLeft]);
    assert_eq!(selected(&app), [2, 3, 5]);
    click(&mut app, at(2), &[ShiftLeft]);
    assert_eq!(selected(&app), [3, 5], "Shift takes a selected unit out");
    click(&mut app, at(3), &[ShiftLeft]);
    click(&mut app, at(5), &[ShiftLeft]);
    assert!(selected(&app).is_empty());
    // A click just off a unit's drawn box still picks it; a little further, nothing.
    click(&mut app, at(1) + Vec2::new(12.0 + 3.0, 0.0), &[]);
    assert_eq!(selected(&app), [1]);
    click(&mut app, at(1) + Vec2::new(12.0 + 8.0, 0.0), &[]);
    assert!(selected(&app).is_empty());
}

#[test]
fn ctrl_and_double_clicks_select_the_own_units_of_that_type_on_screen() {
    let mut app = game();
    // Every own Peacekeeper in the world view: not player 2's, not the one off screen.
    click(&mut app, at(2), &[ControlLeft]);
    assert_eq!(selected(&app), OWN_PKS);
    assert!(!selected(&app).contains(&FAR_PK) && object_exists(&app, FAR_PK));
    double_click(&mut app, at(4));
    assert_eq!(selected(&app), OWN_GUARDS);
    // With the camera further east, the far Peacekeeper is on screen and the western ones aren't.
    app.world_mut().resource_mut::<CameraView>().center = Vec2::new(2600.0, 900.0);
    app.update();
    click(&mut app, at(FAR_PK), &[ControlLeft]);
    assert_eq!(selected(&app), [FAR_PK]);
    // Ctrl wins over Shift.
    app.world_mut().resource_mut::<CameraView>().center = VIEW_CENTER;
    app.update();
    click(&mut app, at(5), &[ControlLeft, ShiftLeft]);
    assert_eq!(selected(&app), OWN_PKS);
}

/// `> Selection > Constraints`: another player's object or a map resource is only ever selected alone.
#[test]
fn other_players_objects_and_resources_are_selected_alone() {
    let mut app = game();
    click(&mut app, at(1), &[]);
    click(&mut app, at(ENEMY_PK), &[ShiftLeft]);
    assert_eq!(selected(&app), [ENEMY_PK], "Shift doesn't add an enemy");
    click(&mut app, at(ENEMY_GUARD), &[ControlLeft]);
    assert_eq!(selected(&app), [ENEMY_GUARD], "Ctrl selects that enemy alone");
    double_click(&mut app, at(ENEMY_PK));
    assert_eq!(selected(&app), [ENEMY_PK]);
    click(&mut app, at(2), &[ShiftLeft]);
    assert_eq!(selected(&app), [2], "an own unit replaces the enemy");
    click(&mut app, at(PATCH), &[ShiftLeft]);
    assert_eq!(selected(&app), [PATCH]);
    click(&mut app, at(STATION), &[ControlLeft]);
    assert_eq!(selected(&app), [STATION]);
}

/// Placeholder rule (the design is silent): a plain click on empty ground clears the selection, a modified one
/// changes nothing.
#[test]
fn a_click_on_empty_ground_clears_the_selection_unless_modified() {
    let mut app = game();
    let ground = Vec2::new(1500.0, 1200.0);
    click(&mut app, at(1), &[ControlLeft]);
    click(&mut app, ground, &[ShiftLeft]);
    click(&mut app, ground, &[ControlLeft]);
    assert_eq!(selected(&app), OWN_PKS);
    click(&mut app, ground, &[]);
    assert!(selected(&app).is_empty());
}

/// `> BoxSelection > Priority tiers`: own units, all of them; else the one object of the best tier nearest the
/// box's center (no own or enemy buildings exist yet: the tiers are unit tested in `selection::pick`).
#[test]
fn a_box_selects_its_highest_tier() {
    let mut app = game();
    // Own units of both types and player 2's units: all own units, by type.
    drag(&mut app, Vec2::new(850.0, 750.0), Vec2::new(2400.0, 1150.0));
    assert_eq!(selected(&app), ids([1, 2, 3, 4, 5]));
    // Enemy units, the patch and the station: the enemy unit nearest the box's center.
    drag(&mut app, Vec2::new(1500.0, 650.0), Vec2::new(2380.0, 1080.0));
    assert_eq!(selected(&app), [ENEMY_PK]);
    drag(&mut app, Vec2::new(2100.0, 650.0), Vec2::new(2480.0, 1000.0));
    assert_eq!(selected(&app), [ENEMY_GUARD], "nearer the center than 6");
    // Only neutral objects: the nearest one.
    drag(&mut app, Vec2::new(1580.0, 650.0), Vec2::new(1900.0, 1100.0));
    assert_eq!(selected(&app), [STATION]);
    drag(&mut app, Vec2::new(1580.0, 900.0), Vec2::new(1900.0, 1100.0));
    assert_eq!(selected(&app), [PATCH]);
    // A box over a unit's edge catches it; a box over nothing clears the selection (placeholder rule).
    drag(&mut app, at(1) + Vec2::new(8.0, -50.0), at(1) + Vec2::new(60.0, 50.0));
    assert_eq!(selected(&app), [1]);
    drag(&mut app, Vec2::new(1400.0, 1150.0), Vec2::new(1500.0, 1250.0));
    assert!(selected(&app).is_empty());
}

#[test]
fn the_box_shows_while_it_is_dragged() {
    let mut app = game();
    let shown = |app: &mut App| {
        let mut edges = app.world_mut().query::<(&DragBoxEdge, &Visibility)>();
        let visible: Vec<bool> = edges.iter(app.world()).map(|(_, v)| *v != Visibility::Hidden).collect();
        assert_eq!(visible.len(), 4);
        visible.iter().all(|&v| v)
    };
    assert!(!shown(&mut app));
    let (from, to) = (screen_at(&app, at(1)), screen_at(&app, at(5) + Vec2::new(40.0, 0.0)));
    input(&mut app).cursor = Some(from);
    input(&mut app).mouse.press(MouseButton::Left);
    app.update();
    assert!(!shown(&mut app), "not a box before the cursor moves");
    input(&mut app).cursor = Some(to);
    app.update();
    assert!(shown(&mut app));
    input(&mut app).mouse.release(MouseButton::Left);
    app.update();
    assert!(!shown(&mut app));
    assert_eq!(selected(&app), [1, 5]);
}

/// Picking tests where objects are drawn: lifted by the height of the terrain they stand on.
#[test]
fn clicks_pick_objects_where_they_are_drawn() {
    let mut scenario = scenarios::empty(32, 24);
    let plateau: BTreeMap<CornerPos, u8> = (12..=20)
        .flat_map(|x| (9..=15).map(move |y| (CornerPos::new(x, y), 3)))
        .collect();
    scenario.map.terrain.set_corner_heights(&plateau);
    let ground = Vec2::new(16.5, 12.5) * 64.0;
    scenario.map.spawns.push(Spawn::new(
        ObjectKind::Peacekeeper,
        Some(SlotId(1)),
        SpawnAt::Point(ground),
    ));
    assert_eq!(scenario.validate(), vec![]);
    let mut app = game_of(scenario, true);
    app.world_mut().resource_mut::<CameraView>().center = ground;
    app.update();
    let drawn = ground - Vec2::new(0.0, lift(3.0));
    click(&mut app, ground, &[]);
    assert!(selected(&app).is_empty(), "nothing is drawn at its ground point");
    click(&mut app, drawn, &[]);
    assert_eq!(selected(&app), [1]);
}
