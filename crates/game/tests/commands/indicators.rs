//! Command indicators (`systems/controls/selection_and_hud.md > CommandIndicators`) and the pointer
//! (`systems/controls/selection_and_hud.md > PointerDisplayType`).

use bevy::prelude::*;
use sc_sim::{ObjectId, OrderKind};
use space_crystals::command_ui::PointerDisplay;
use space_crystals::command_ui::cursor::PointerLook;
use space_crystals::command_ui::indicators::{
    ATTACK_COLOR, CommandIndicators, FLASH_TICKS, Indicator, IndicatorMesh, MOVE_COLOR, selection_indicators,
};
use space_crystals::fog::Sight;
use space_crystals::selection::Relation;

use super::*;
use KeyCode::{KeyA, ShiftLeft};

/// The local player's indicators now, with the queued orders' or without (no fog: see `main.rs`).
fn indicators(app: &App, queued: bool) -> Vec<Indicator> {
    let world = app.world();
    let states = world.resource::<ControlStates>();
    let sim = world.resource::<SimRes>();
    selection_indicators(sim, &Sight::new(sim, local_player(app), false), states, queued)
}

fn flashes(app: &App) -> Vec<OrderKind> {
    let flashes = &app.world().resource::<CommandIndicators>().flashes;
    flashes.iter().map(|f| f.kind).collect()
}

/// How many vertices the indicators' mesh has; 0 while it is hidden.
pub(super) fn drawn_vertices(app: &mut App) -> usize {
    let mut query = app
        .world_mut()
        .query_filtered::<(&Mesh2d, &Visibility), With<IndicatorMesh>>();
    let (mesh, visibility) = query.single(app.world()).unwrap();
    if *visibility == Visibility::Hidden {
        return 0;
    }
    let meshes = app.world().resource::<Assets<Mesh>>();
    meshes.get(&mesh.0).unwrap().count_vertices()
}

/// A right-click order flashes at its target for `FLASH_TICKS`. The Move then keeps its green Location indicator
/// while the unit is selected and on its way; a queued order's indicator, and the route through the waypoints, show
/// only while Shift is held. Deselecting the unit removes them.
#[test]
fn orders_flash_and_mark_their_targets_while_selected() {
    let mut app = game();
    click(&mut app, at(1), &[]);
    assert_eq!(drawn_vertices(&mut app), 0, "nothing to show");
    let target = Vec2::new(1000.0, 1150.0);
    let screen = screen_at(&app, target);
    input(&mut app).cursor = Some(screen);
    input(&mut app).mouse.press(MouseButton::Right);
    app.update();
    input(&mut app).mouse.release(MouseButton::Right);
    let flashed = flashes(&app);
    assert!(matches!(flashed[..], [OrderKind::Move(_)]), "{flashed:?}");
    updates(&mut app, FLASH_TICKS as usize + 1);
    assert_eq!(flashes(&app), [], "faded");

    let marked = indicators(&app, false);
    assert!(
        matches!(marked[..], [Indicator::Location { at, color }] if near(at, target) && color == MOVE_COLOR),
        "{marked:?}"
    );
    let plain = drawn_vertices(&mut app);
    assert!(plain > 0);
    let queued_target = Vec2::new(1300.0, 1150.0);
    right_click(&mut app, queued_target, &[ShiftLeft]);
    assert_eq!(indicators(&app, false).len(), 1, "the queued order waits");
    assert_eq!(indicators(&app, true).len(), 2);
    // Shift held: the route and the queued order's marker are drawn too.
    press_keys(&mut app, &[ShiftLeft]);
    app.update();
    assert!(drawn_vertices(&mut app) > plain);
    release_keys(&mut app, &[ShiftLeft]);
    app.update();
    assert_eq!(drawn_vertices(&mut app), plain);

    click(&mut app, at(5), &[]);
    assert_eq!(indicators(&app, true), [], "unit 1 is no longer selected");
    assert_eq!(drawn_vertices(&mut app), 0);
}

/// An Attack's indicator is a red Object indicator around its target.
#[test]
fn an_attack_marks_its_target() {
    let mut app = game();
    click(&mut app, at(1), &[]);
    right_click(&mut app, at(ENEMY_PK), &[]);
    let marked = indicators(&app, false);
    assert_eq!(
        marked,
        [Indicator::Object {
            id: ObjectId(ENEMY_PK),
            color: ATTACK_COLOR
        }]
    );
}

/// `systems/controls/selection_and_hud.md > PointerDisplayType > Resolution Rules`: the pointer previews the
/// right-click from DefaultState, and the pending command's resolution while a target is awaited; over an object it
/// also shows the object's relation. Attack takes an own destructible unit (force-attack, design change 0064), never
/// the indestructible patch.
#[test]
fn the_pointer_previews_what_a_click_would_do() {
    let mut app = game();
    let hover = |app: &mut App, view: Vec2| {
        let screen = screen_at(app, view);
        input(app).cursor = Some(screen);
        app.update();
        *app.world().resource::<PointerLook>()
    };
    let look = |display, targeting, hovered| PointerLook {
        display: Some(display),
        targeting,
        placing: false,
        hovered,
    };
    let ground = Vec2::new(1500.0, 1000.0);
    use PointerDisplay::{Attack, Inactive, Move};
    assert_eq!(hover(&mut app, ground), look(Inactive, false, None), "nothing selected");
    assert_eq!(
        hover(&mut app, at(ENEMY_PK)),
        look(Inactive, false, Some(Relation::Enemy))
    );
    click(&mut app, at(1), &[]);
    assert_eq!(
        hover(&mut app, at(ENEMY_PK)),
        look(Attack, false, Some(Relation::Enemy))
    );
    assert_eq!(hover(&mut app, ground), look(Move, false, None));
    assert_eq!(hover(&mut app, at(3)), look(Move, false, Some(Relation::Own)));
    assert_eq!(hover(&mut app, at(PATCH)), look(Move, false, Some(Relation::Neutral)));
    tap(&mut app, &[], KeyA);
    assert_eq!(hover(&mut app, ground), look(Attack, true, None));
    assert_eq!(hover(&mut app, at(3)), look(Attack, true, Some(Relation::Own)));
    assert_eq!(
        hover(&mut app, at(PATCH)),
        look(Inactive, true, Some(Relation::Neutral))
    );
    // Over the HUD there is nothing to preview.
    input(&mut app).cursor = Some(Vec2::new(640.0, 700.0));
    app.update();
    assert_eq!(*app.world().resource::<PointerLook>(), PointerLook::default());
}
