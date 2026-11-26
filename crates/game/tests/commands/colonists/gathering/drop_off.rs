//! A Drone's S, Set Drop-off (`factions/colonists/units/drone.md > Set Drop-off Location`, `> Drone
//! ObjectInterfaceState`; design change 0124: "Drone: Q Move, A Deploy, S Set Drop-off, X Stop", W Deploy since
//! design change 0398): S enters
//! AwaitingTarget\[SetDropOffLocation\], and a left-click on an own Egress Node makes it the AssignedDropOff of the
//! selected Drones (`sc_sim::Ability::SetDropOff`, to every selected Drone: question Q0040 point 9), back in
//! DefaultState. "Does not interrupt current mining/movement — only updates AssignedDropOff."

use sc_sim::Ability;

use super::*;
use KeyCode::{Escape, KeyS};

/// The Set Drop-off abilities used so far, as the sim received them: the Drones and the node.
fn set_drop_offs(app: &App) -> Vec<(Vec<u32>, u32)> {
    (abilities(app).into_iter())
        .map(|(player, units, ability)| match ability {
            Ability::SetDropOff(node) if player == P1 => (units, node.0),
            other => panic!("{player:?} used {other:?}"),
        })
        .collect()
}

/// How far Drone `id` has mined its patch, while it mines.
fn mining(app: &App, id: u32) -> Option<u32> {
    match gather(app, id)?.phase {
        GatherPhase::Mining { done } => Some(done),
        _ => None,
    }
}

/// S, then a click on node 2 while Drones A and B gather patch 3 for node 1, one mining and the other walking there
/// or waiting at it: the pointer over the node shows ReturnResources (placeholder, `PointerDisplay::of`), the click
/// sets both Drones' drop-off, as one ability, and returns to DefaultState. Their Gathers go on: the one mining mines
/// on and takes its load to node 2, the other goes on with its own.
#[test]
fn s_then_a_click_on_an_own_node_makes_it_the_drop_off() {
    let mut app = gathering_game();
    select_around(&mut app, &[home(A), home(B)], &[A, B]);
    right_click(&mut app, home(PATCH), &[]);
    run_until(&mut app, 300, "a Drone mines", |app| {
        mining(app, A).or(mining(app, B)).is_some()
    });
    let (miner, other) = if mining(&app, A).is_some() { (A, B) } else { (B, A) };
    let begun = mining(&app, miner).expect("mining");
    assert_eq!([drop_off(&app, A), drop_off(&app, B)], [Some(NODE); 2]);

    tap(&mut app, &[], KeyS);
    assert_eq!(interface(&app), InterfaceState::awaiting(PanelCommand::SetDropOff));
    assert_eq!(
        pointer_over(&mut app, home(NODE_2)),
        Some(PointerDisplay::ReturnResources)
    );
    click(&mut app, home(NODE_2), &[]);
    assert_eq!(set_drop_offs(&app), [(vec![A, B], NODE_2)]);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    assert_eq!([drop_off(&app, A), drop_off(&app, B)], [Some(NODE_2); 2]);
    assert!(
        mining(&app, miner).is_some_and(|done| done > begun),
        "the miner mines on"
    );
    let walking_or_waiting = |phase: GatherPhase| matches!(phase, GatherPhase::ToPatch | GatherPhase::Waiting { .. });
    assert!(in_phase(&app, other, walking_or_waiting), "{:?}", gather(&app, other));
    assert_eq!(orders(&app).len(), 1, "no order but the Gather");

    run_until(&mut app, 300, "the load goes to node 2", |app| {
        unloading_at(app, miner).is_some()
    });
    assert_eq!(unloading_at(&app, miner), Some(NODE_2));
}

/// A Drone on its way back with its load when its drop-off changes takes the load to the new one: the return leg reads
/// the AssignedDropOff (`> Assigned Drop-off`). It never unloads at node 1, which it was heading for.
#[test]
fn a_drone_on_its_way_back_takes_its_load_to_the_new_drop_off() {
    let mut app = gathering_game();
    click(&mut app, home(A), &[]);
    right_click(&mut app, home(PATCH), &[]);
    // The leg back names its node from its first tick on.
    let heading_for =
        |app: &App| (gather(app, A)).and_then(|g| g.node.filter(|_| matches!(g.phase, GatherPhase::ToDropOff)));
    run_until(&mut app, 300, "A carries its load back", |app| {
        heading_for(app).is_some()
    });
    assert_eq!(heading_for(&app), Some(ObjectId(NODE)));
    assert_eq!(load(&app, A), 8);

    tap(&mut app, &[], KeyS);
    click(&mut app, home(NODE_2), &[]);
    assert_eq!(set_drop_offs(&app), [(vec![A], NODE_2)]);
    run_until(&mut app, 300, "A unloads", |app| unloading_at(app, A).is_some());
    assert_eq!(unloading_at(&app, A), Some(NODE_2));
    assert_eq!(orders(&app).len(), 1, "no order but the Gather");
}

/// `systems/controls/interface.md > AwaitingTarget[CommandType]` with Set Drop-off pending, its S lit up: a
/// click on anything but an own Egress Node (the patch, the ground, an own portal, an enemy structure), under the
/// Inactive pointer, is ignored, the target still awaited, and the feedback line says so (design change 0065). Escape
/// cancels, and so does a right-click, which orders nothing ("Escape or right-click cancels").
#[test]
fn set_drop_off_ignores_an_invalid_target_and_escape_or_a_right_click_cancels_it() {
    let mut app = gathering_game();
    select_around(&mut app, &[home(A), home(B)], &[A, B]);
    let awaiting = InterfaceState::awaiting(PanelCommand::SetDropOff);
    tap(&mut app, &[], KeyS);
    assert_eq!(interface(&app), awaiting);
    let s = (slot(1, 1), PanelButton::Command(PanelCommand::SetDropOff));
    let lit = buttons(&mut app).into_iter().find(|b| (b.0, b.1) == s).map(|b| b.2);
    assert_eq!(lit, Some(ButtonLook::Pending));
    for at in [home(PATCH), GROUND, home(ROBO_PORTAL), home(ENEMY_PORTAL)] {
        assert_eq!(pointer_over(&mut app, at), Some(PointerDisplay::Inactive), "{at}");
        assert_eq!(says(&mut app, |app| click(app, at, &[])), Some("Invalid target"));
        assert_eq!(interface(&app), awaiting, "still awaited after a click at {at}");
    }
    assert_eq!(set_drop_offs(&app), []);

    tap(&mut app, &[], Escape);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    tap(&mut app, &[], KeyS);
    assert_eq!(interface(&app), awaiting);
    right_click(&mut app, GROUND, &[]);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    assert_eq!((orders(&app), set_drop_offs(&app)), (vec![], vec![]));
    assert_eq!([drop_off(&app, A), drop_off(&app, B)], [None; 2]);
}
