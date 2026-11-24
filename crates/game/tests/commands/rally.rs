//! Rally points from the command flow (slice M2-H1): a right-click with a Barracks selected sets its rally point
//! (`> Production Building Default Right-Click`), and so does C, Set Rally Point, then a left-click
//! (`factions/gdo/structures.md > Barracks > ObjectInterfaceState[Barracks]`); on the Barracks itself either clears it
//! (design change 0086). In a mixed selection each object does its own (design changes 0062 and 0063). While the
//! Barracks is selected its rally point shows: a marker and a line from the building (design change 0087), red on an
//! enemy until the sim turns it into a ground rally point where the enemy was last seen (design change 0088).

use bevy::prelude::*;
use sc_sim::{Command, ObjectId, OrderKind, PlayerId, ProductionCommand, Rally, Target};
use space_crystals::command_ui::indicators::{ATTACK_COLOR, Indicator, MOVE_COLOR, RallyIndicator, rally_indicators};
use space_crystals::command_ui::{InterfaceState, PanelCommand, PointerDisplay};
use space_crystals::fog::Sight;

use super::indicators::drawn_vertices;
use super::*;
use KeyCode::{Escape, KeyC, ShiftLeft};

const P1: PlayerId = PlayerId(1);

/// Bare ground in the world view, away from every object.
const GROUND: Vec2 = Vec2::new(1600.0, 650.0);

/// Barracks `id`'s rally point, as the sim has it.
fn rally(app: &App, id: u32) -> Option<Rally> {
    production_of(app, id).rally
}

/// The ground point of a rally point at `point`, give or take the cursor's rounding.
fn rallies_at(rally: Option<Rally>, point: Vec2) -> bool {
    matches!(rally, Some(Rally::Point(p)) if near(p, point))
}

/// The SetRally commands given since the first `from` production commands: producers and target.
fn rallies_since(app: &App, from: usize) -> Vec<(Vec<u32>, Target)> {
    productions(app)[from..]
        .iter()
        .map(|(player, producers, command)| {
            assert_eq!(*player, P1);
            match command {
                ProductionCommand::SetRally(target) => (producers.clone(), *target),
                other => panic!("{other:?} sets no rally point"),
            }
        })
        .collect()
}

/// The rally indicators the local player sees now (no fog: see `main.rs`).
fn shown(app: &App) -> Vec<RallyIndicator> {
    let world = app.world();
    let sim = world.resource::<SimRes>();
    let states = world.resource::<ControlStates>();
    rally_indicators(sim, &Sight::new(sim, local_player(app), false), states)
}

/// `> Production Building Default Right-Click` (design change 0086): with a Barracks selected, a right-click on the
/// ground, on an own unit, on an enemy or on a neutral object sets its rally point there, and a right-click on the
/// Barracks itself clears it. The pointer shows Move over all of them ("Selection is a production building: Move
/// (rally point)"). A rally point is set at once, Shift or not, and no unit gets an order.
#[test]
fn a_right_click_sets_the_rally_point_and_on_the_barracks_clears_it() {
    let mut app = production_game(0, 0);
    click(&mut app, barracks_at(15), &[]);
    for over in [GROUND, at(2), at(ENEMY_PK), at(PATCH)] {
        assert_eq!(pointer_over(&mut app, over), Some(PointerDisplay::Move), "{over}");
    }
    right_click(&mut app, GROUND, &[]);
    let given = rallies_since(&app, 0);
    assert!(
        matches!(given[..], [(ref producers, Target::Point(p))] if producers == &[15] && near(p, GROUND)),
        "{given:?}"
    );
    assert!(rallies_at(rally(&app, 15), GROUND), "{:?}", rally(&app, 15));

    for (n, id) in [2, ENEMY_PK, PATCH].into_iter().enumerate() {
        right_click(&mut app, at(id), &[]);
        assert_eq!(rallies_since(&app, n + 1), [(vec![15], Target::Object(ObjectId(id)))]);
        assert!(matches!(rally(&app, 15), Some(Rally::Object { id: on, .. }) if on == ObjectId(id)));
    }
    right_click(&mut app, barracks_at(15), &[]);
    assert_eq!(rallies_since(&app, 4), [(vec![15], Target::Object(ObjectId(15)))]);
    assert_eq!(rally(&app, 15), None, "cleared");

    right_click(&mut app, GROUND, &[ShiftLeft]);
    assert!(rallies_at(rally(&app, 15), GROUND));
    assert_eq!(orders(&app), []);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
}

/// `> Standard Slot Assignments > C`: Set Rally Point awaits its target, which the panel marks; a left-click on the
/// ground or on an object sets the rally point there and returns to DefaultState, and one on the Barracks itself
/// clears it. The pointer shows Move ("AwaitingTarget[SetRallyPoint] over ground or object: Move"). A Shift-click
/// sets it and ends the targeting too (a placeholder reading: a rally point isn't queued). Esc, a right-click and
/// Back cancel, and set nothing.
#[test]
fn c_then_a_click_sets_the_rally_point() {
    let mut app = production_game(0, 0);
    click(&mut app, barracks_at(15), &[]);
    let awaiting = InterfaceState::awaiting(PanelCommand::SetRallyPoint);
    tap(&mut app, &[], KeyC);
    assert_eq!(interface(&app), awaiting);
    assert_eq!(pointer_over(&mut app, GROUND), Some(PointerDisplay::Move));
    assert_eq!(pointer_over(&mut app, at(ENEMY_PK)), Some(PointerDisplay::Move));
    click(&mut app, GROUND, &[]);
    assert!(rallies_at(rally(&app, 15), GROUND));
    assert_eq!(interface(&app), InterfaceState::DefaultState);

    click_button(&mut app, CommandSlot::RALLY_POINT, &[]);
    assert_eq!(interface(&app), awaiting);
    click(&mut app, barracks_at(15), &[]);
    assert_eq!(rally(&app, 15), None, "cleared");
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    assert_eq!(selected(&app), [15]);

    tap(&mut app, &[], KeyC);
    click(&mut app, at(ENEMY_PK), &[ShiftLeft]);
    assert!(matches!(rally(&app, 15), Some(Rally::Object { id, .. }) if id == ObjectId(ENEMY_PK)));
    assert_eq!(
        interface(&app),
        InterfaceState::DefaultState,
        "a Shift-click ends it too"
    );

    let sent = productions(&app).len();
    tap(&mut app, &[], KeyC);
    tap(&mut app, &[], Escape);
    tap(&mut app, &[], KeyC);
    right_click(&mut app, GROUND, &[]);
    tap(&mut app, &[], KeyC);
    tap(&mut app, &[], KeyCode::KeyZ);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    assert_eq!(productions(&app).len(), sent, "cancelled: nothing set");
    assert!(matches!(rally(&app, 15), Some(Rally::Object { .. })));
}

/// Design change 0087's verifications: while the Barracks is selected, its rally point shows a green marker on the
/// ground, a red one around an enemy and a green one around an own unit or a neutral object, with the line from the
/// building; deselected, nothing shows, and a cleared rally point shows nothing either.
#[test]
fn the_rally_point_shows_while_the_barracks_is_selected() {
    let mut app = production_game(0, 0);
    click(&mut app, barracks_at(15), &[]);
    assert_eq!((shown(&app), drawn_vertices(&mut app)), (vec![], 0), "no rally point");
    right_click(&mut app, GROUND, &[]);
    let markers = shown(&app);
    assert!(
        matches!(markers[..], [RallyIndicator { producer, marker: Indicator::Location { at, color } }]
            if producer == ObjectId(15) && near(at, GROUND) && color == MOVE_COLOR),
        "{markers:?}"
    );
    assert!(drawn_vertices(&mut app) > 0);
    let object = |id, color| RallyIndicator {
        producer: ObjectId(15),
        marker: Indicator::Object {
            id: ObjectId(id),
            color,
        },
    };
    right_click(&mut app, at(ENEMY_PK), &[]);
    assert_eq!(shown(&app), [object(ENEMY_PK, ATTACK_COLOR)]);
    right_click(&mut app, at(2), &[]);
    assert_eq!(shown(&app), [object(2, MOVE_COLOR)]);
    right_click(&mut app, at(PATCH), &[]);
    assert_eq!(shown(&app), [object(PATCH, MOVE_COLOR)]);

    click(&mut app, GROUND, &[]);
    assert_eq!(selected(&app), Vec::<u32>::new());
    assert_eq!((shown(&app), drawn_vertices(&mut app)), (vec![], 0), "deselected");
    click(&mut app, barracks_at(15), &[]);
    assert_eq!(shown(&app), [object(PATCH, MOVE_COLOR)]);
    right_click(&mut app, barracks_at(15), &[]);
    assert_eq!((shown(&app), drawn_vertices(&mut app)), (vec![], 0), "cleared");
}

/// Design change 0088's second verification: a rally point on an enemy shows red around it while player 1 sees it;
/// once the enemy is out of sight the sim makes it a ground rally point where the enemy was last seen, and the
/// marker turns green there.
#[test]
fn an_enemy_rally_point_turns_green_where_the_enemy_was_last_seen() {
    let mut app = production_game(0, 0);
    click(&mut app, barracks_at(15), &[]);
    right_click(&mut app, at(ENEMY_PK), &[]);
    let marked = shown(&app);
    assert!(
        matches!(marked[..], [RallyIndicator { marker: Indicator::Object { color, .. }, .. }] if color == ATTACK_COLOR)
    );
    // Far from every unit of player 1's.
    let away = DebugCommand::SetPosition {
        id: ObjectId(ENEMY_PK),
        pos: Vec2::new(3900.0, 2900.0),
    };
    submit(&mut app, P1, Command::Debug(away));
    updates(&mut app, 2);
    assert_eq!(rally(&app, 15), Some(Rally::Point(at(ENEMY_PK))));
    assert_eq!(
        shown(&app),
        [RallyIndicator {
            producer: ObjectId(15),
            marker: Indicator::Location {
                at: at(ENEMY_PK),
                color: MOVE_COLOR
            },
        }]
    );
}

/// Design changes 0062 and 0063 with a Barracks: Peacekeepers and a Barracks selected, a right-click on the ground
/// moves the units and sets the Barracks' rally point (the plan's acceptance), and Shift queues the units' order
/// only. Over an enemy the units would attack and the Barracks rally, so the pointer is Mixed.
#[test]
fn units_move_and_the_barracks_rallies_on_one_right_click() {
    let mut app = production_game(0, 0);
    select_box(&mut app, &[1, 2]);
    add_box(&mut app, barracks_at(15));
    assert_eq!(selected(&app), [1, 2, 15]);
    assert_eq!(pointer_over(&mut app, GROUND), Some(PointerDisplay::Move));
    assert_eq!(pointer_over(&mut app, at(ENEMY_PK)), Some(PointerDisplay::Mixed));
    right_click(&mut app, GROUND, &[]);
    let moved = orders(&app);
    assert!(
        matches!(moved[..], [ref order] if order.units == [1, 2] && near(point_of(order.kind), GROUND) && !order.queue),
        "{moved:?}"
    );
    assert!(rallies_at(rally(&app, 15), GROUND));
    right_click(&mut app, at(ENEMY_PK), &[ShiftLeft]);
    assert_eq!(
        orders(&app)[1..],
        [given(&[1, 2], OrderKind::Attack(ObjectId(ENEMY_PK)), true)]
    );
    assert_eq!(rallies_since(&app, 1), [(vec![15], Target::Object(ObjectId(ENEMY_PK)))]);
}

/// No smart-casting with rally points: three Barracks selected, a right-click and C with a click each set all of
/// their rally points with one command. On one of them it clears that one's, and the others rally on it (the sim's
/// reading of design change 0086 for a group).
#[test]
fn every_selected_barracks_takes_the_rally_point() {
    let mut app = production_game(0, 0);
    select_barracks(&mut app, &BARRACKS);
    right_click(&mut app, GROUND, &[]);
    let given = rallies_since(&app, 0);
    assert!(
        matches!(given[..], [(ref producers, _)] if producers == &BARRACKS),
        "{given:?}"
    );
    assert!(BARRACKS.iter().all(|&id| rallies_at(rally(&app, id), GROUND)));
    assert_eq!(shown(&app).len(), 3, "a marker and a line each");

    tap(&mut app, &[], KeyC);
    click(&mut app, barracks_at(16), &[]);
    assert_eq!(
        rallies_since(&app, 1),
        [(BARRACKS.to_vec(), Target::Object(ObjectId(16)))]
    );
    assert_eq!(rally(&app, 16), None);
    for id in [15, 17] {
        assert!(matches!(rally(&app, id), Some(Rally::Object { id: on, .. }) if on == ObjectId(16)));
    }
}
