//! Right-clicks (`systems/controls/selection_and_hud.md > RightClickResolution`,
//! `systems/controls/selection_and_hud.md > BasicCombatUnitInterfaceState > RightClickResolution`) and the
//! Shift-queue (`systems/controls/commands.md > Unit Command`).

use bevy::prelude::*;
use sc_sim::defs::RIOT_SHIELD_TOGGLE_TICKS;
use sc_sim::{ObjectId, OrderKind, Target};

use super::*;
use KeyCode::{ControlLeft, KeyR, ShiftLeft, Tab};

/// Ground: Move to the point. An enemy: Attack it. An own unit, the neutral patch and the neutral station: Move to
/// that object; a right-click never attacks an own or a neutral object (design change 0059). Each to the whole
/// selection, replacing the units' orders.
#[test]
fn a_right_click_resolves_by_what_is_under_the_cursor() {
    let mut app = game();
    select_box(&mut app, &[1, 2]);
    let ground = Vec2::new(1000.0, 1000.0);
    right_click(&mut app, ground, &[]);
    right_click(&mut app, at(ENEMY_PK), &[]);
    right_click(&mut app, at(ENEMY_GUARD), &[]);
    right_click(&mut app, at(OWN_GUARDS[0]), &[]);
    right_click(&mut app, at(PATCH), &[]);
    right_click(&mut app, at(STATION), &[]);
    let given_orders = orders(&app);
    assert_eq!(given_orders.len(), 6, "{given_orders:?}");
    assert!(near(point_of(given_orders[0].kind), ground), "{:?}", given_orders[0]);
    let object = |id: u32| OrderKind::Move(Target::Object(ObjectId(id)));
    let attack = |id: u32| OrderKind::Attack(ObjectId(id));
    assert_eq!(
        given_orders[1..],
        [
            given(&[1, 2], attack(ENEMY_PK), false),
            given(&[1, 2], attack(ENEMY_GUARD), false),
            given(&[1, 2], object(OWN_GUARDS[0]), false),
            given(&[1, 2], object(PATCH), false),
            given(&[1, 2], object(STATION), false),
        ]
    );
    assert!(given_orders.iter().all(|o| o.units == [1, 2] && !o.queue));
    assert_eq!(selected(&app), [1, 2], "a right-click selects nothing");
}

/// `systems/controls/selection_and_hud.md > CommandPanel`: no right-click resolution without a selection, nor for
/// another player's or a neutral one.
#[test]
fn a_right_click_orders_nothing_without_an_own_selection() {
    let mut app = game();
    right_click(&mut app, Vec2::new(1000.0, 1000.0), &[]);
    click(&mut app, at(ENEMY_PK), &[]);
    right_click(&mut app, Vec2::new(1000.0, 1000.0), &[]);
    click(&mut app, at(STATION), &[]);
    right_click(&mut app, at(ENEMY_PK), &[]);
    assert_eq!(orders(&app), []);
}

/// `systems/controls/commands.md > Unit Command`: Shift appends an order to the units' queues; without Shift an order
/// replaces the whole queue and the current order. The sim's queues show it.
#[test]
fn shift_right_clicks_queue_and_plain_ones_replace() {
    let mut app = game();
    click(&mut app, at(1), &[]);
    let waypoints = [
        Vec2::new(900.0, 1000.0),
        Vec2::new(1100.0, 1000.0),
        Vec2::new(1100.0, 1200.0),
    ];
    right_click(&mut app, waypoints[0], &[]);
    right_click(&mut app, waypoints[1], &[ShiftLeft]);
    right_click(&mut app, waypoints[2], &[ShiftLeft]);
    let given_orders = orders(&app);
    let queued: Vec<bool> = given_orders.iter().map(|o| o.queue).collect();
    assert_eq!(queued, [false, true, true]);
    for (order, waypoint) in given_orders.iter().zip(waypoints) {
        assert!(near(point_of(order.kind), waypoint));
    }
    // The first move is on its way; the other two wait in the queue.
    let unit = object_orders(&app, 1);
    let current = unit.current.expect("moving").order.kind;
    assert!(near(point_of(current), waypoints[0]));
    let queue: Vec<Vec2> = unit.queue.iter().map(|o| point_of(o.kind)).collect();
    assert!(
        near(queue[0], waypoints[1]) && near(queue[1], waypoints[2]),
        "{queue:?}"
    );

    let elsewhere = Vec2::new(1300.0, 1150.0);
    right_click(&mut app, elsewhere, &[]);
    assert!(!orders(&app)[3].queue);
    let unit = object_orders(&app, 1);
    assert!(unit.queue.is_empty(), "replaced");
    assert!(near(point_of(unit.current.expect("moving").order.kind), elsewhere));

    // Ctrl is no queue modifier: a Ctrl right-click replaces too.
    right_click(&mut app, waypoints[0], &[ControlLeft]);
    assert!(!orders(&app)[4].queue);
}

/// Design changes 0062 and 0063: each selected unit resolves a right-click for itself, whichever group is active,
/// and the orders go out grouped, one per distinct order. With the Peacekeepers' Riot Shields up, a right-click on
/// an enemy sends the Guards to attack it and the Peacekeepers, which can't attack now, to follow it (0062's
/// `move_to`, design change 0074: it replaced the Move to the clicked spot of question Q0020 point 1), with either
/// group active. The pointer previews it: Attack while all of them would attack, Mixed once they would do
/// different things, Move over the ground. A right-click on the ground is one order for all four: one group for the
/// magic box.
#[test]
fn each_selected_unit_resolves_a_right_click_for_itself() {
    let mut app = game();
    select_mixed(&mut app);
    let enemy = at(ENEMY_PK);
    let ground = Vec2::new(1500.0, 1000.0);
    assert_eq!(pointer_over(&mut app, enemy), Some(PointerDisplay::Attack));
    tap(&mut app, &[], KeyR);
    updates(&mut app, RIOT_SHIELD_TOGGLE_TICKS as usize + 2);
    assert_eq!(pointer_over(&mut app, enemy), Some(PointerDisplay::Mixed));
    assert_eq!(pointer_over(&mut app, ground), Some(PointerDisplay::Move));
    assert_eq!(pointer_over(&mut app, at(OWN_PKS[2])), Some(PointerDisplay::Move));

    right_click(&mut app, enemy, &[]);
    tap(&mut app, &[], Tab);
    assert_eq!(
        pointer_over(&mut app, enemy),
        Some(PointerDisplay::Mixed),
        "the Guards active"
    );
    right_click(&mut app, enemy, &[]);
    let given_orders = orders(&app);
    assert_eq!(given_orders.len(), 4, "{given_orders:?}");
    for pair in given_orders.chunks(2) {
        assert_eq!(
            pair,
            [
                given(&[1, 2], OrderKind::Move(Target::Object(ObjectId(ENEMY_PK))), false),
                given(&[3, 4], OrderKind::Attack(ObjectId(ENEMY_PK)), false),
            ]
        );
    }

    right_click(&mut app, ground, &[]);
    let given_orders = orders(&app);
    assert_eq!(given_orders.len(), 5, "{given_orders:?}");
    assert_eq!(given_orders[4].units, [1, 2, 3, 4]);
    assert!(near(point_of(given_orders[4].kind), ground));
    assert_eq!(selected(&app), [1, 2, 3, 4]);
}
