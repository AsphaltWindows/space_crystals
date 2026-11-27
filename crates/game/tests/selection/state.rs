//! The ControlState against the game: removed objects leave it at once, each player has their own, and the debug
//! player switch changes whose is in use.

use bevy::prelude::*;
use sc_sim::{Command, DebugCommand, PlayerCommand, PlayerId, scenarios};
use space_crystals::sim_driver::{PendingCommands, load_scenario};

use super::*;
use KeyCode::{ControlLeft, Digit1, F2};

fn remove(app: &mut App, id: u32) {
    app.world_mut().resource_mut::<PendingCommands>().0.push(PlayerCommand {
        player: PlayerId(1),
        command: Command::Debug(DebugCommand::Remove { id: ObjectId(id) }),
    });
}

/// `systems/controls/selection.md > ControlState` (validated every tick) and
/// `systems/controls/selection.md > ControlGroups > Operations > Recall` (destroyed objects leave): a removed
/// object (`SimEvent::Removed`, as for a death) leaves the selection and every group in the frame of its tick.
#[test]
fn removed_objects_leave_the_selection_and_the_groups_at_once() {
    let mut app = game();
    click(&mut app, at(1), &[ControlLeft]);
    tap(&mut app, &[ControlLeft], Digit1);
    pause(&mut app);
    assert_eq!(selected(&app), OWN_PKS);
    remove(&mut app, 2);
    app.update();
    assert!(!object_exists(&app, 2));
    assert_eq!(selected(&app), [1, 5]);
    assert_eq!(control_group(&app, 1), [1, 5]);
    // The last ones gone, the selection is empty.
    remove(&mut app, 1);
    remove(&mut app, 5);
    app.update();
    assert!(selected(&app).is_empty() && control_group(&app, 1).is_empty());
}

/// The debug player switch: F2 (placeholder key) makes the local client act as the next player, with that
/// player's own selection and own units, and back. Each player keeps their ControlState.
#[test]
fn the_debug_switch_changes_whose_selection_and_units_are_in_use() {
    let mut app = game();
    let top_bar_shows = |app: &mut App, name: &str| {
        let mut texts = app.world_mut().query::<&Text>();
        texts.iter(app.world()).any(|t| t.0 == name)
    };
    click(&mut app, at(1), &[ControlLeft]);
    assert!(top_bar_shows(&mut app, "Player 1"));
    tap(&mut app, &[], F2);
    assert_eq!(local_player(&app), PlayerId(2));
    assert!(top_bar_shows(&mut app, "Player 2") && !top_bar_shows(&mut app, "Player 1"));
    assert!(selected(&app).is_empty(), "player 2 has selected nothing yet");
    // Player 2 has a camera of their own, at their start (slice M3-J3): back to this test's view.
    app.world_mut().resource_mut::<CameraView>().center = VIEW_CENTER;
    // Player 2's units are the own ones now: Ctrl selects their type, and player 1's are enemies.
    click(&mut app, at(ENEMY_PK), &[ControlLeft]);
    assert_eq!(selected(&app), [ENEMY_PK, ENEMY_PK_2]);
    click(&mut app, at(2), &[KeyCode::ShiftLeft]);
    assert_eq!(selected(&app), [2], "player 1's unit is selected alone");
    pause(&mut app);
    tap(&mut app, &[], F2);
    assert_eq!(local_player(&app), PlayerId(1), "two players: the switch wraps around");
    assert_eq!(selected(&app), OWN_PKS, "player 1's selection is as it was");
}

#[test]
fn without_debug_commands_the_switch_does_nothing() {
    let mut app = game_of(scenario(), false);
    tap(&mut app, &[], F2);
    assert_eq!(local_player(&app), PlayerId(1));
}

/// A new sim (another scenario, a play-test) starts every ControlState over: its ids name other objects.
#[test]
fn a_replaced_sim_starts_the_selections_over() {
    let mut app = game();
    click(&mut app, at(1), &[ControlLeft]);
    tap(&mut app, &[ControlLeft], Digit1);
    pause(&mut app);
    let mut replacement = scenarios::empty(64, 48);
    replacement.map.spawns = scenario().map.spawns;
    load_scenario(app.world_mut(), replacement);
    app.update();
    assert!(object_exists(&app, 1));
    assert!(selected(&app).is_empty() && control_group(&app, 1).is_empty());
}

/// Design change 0199's Verification (`systems/controls/selection.md > Selection > ActiveGroup > When the
/// ActiveGroup disappears`): "Select Peacekeepers (active) and Rocketmen, then have all the Peacekeepers die. The
/// Rocketmen group becomes active." Player 1's Guards, the shared scenario's second group, stand in for the Rocketmen.
/// Peacekeepers 1 and 2 and Guards 3 and 4 selected, the Peacekeepers' group, the first, active and awaiting an
/// Attack's target: a Guard dying ("Completions or deaths in a group that isn't active never change the ActiveGroup")
/// and one Peacekeeper dying (`systems/controls/interface.md > AwaitingTarget > Selection changes while
/// targeting`: automatic removals "do not cancel targeting") change neither; the last Peacekeeper gone (removed as a
/// death removes it, `SimEvent::Removed`), "the next group in Tab order becomes active", the Guards', and the targeting
/// is over.
#[test]
fn when_the_active_groups_last_member_dies_the_next_group_in_tab_order_is_active() {
    use space_crystals::command_ui::{InterfaceState, PanelCommand};
    let mut app = game();
    drag(&mut app, Vec2::new(850.0, 760.0), Vec2::new(1400.0, 840.0));
    assert_eq!(selected(&app), [1, 2, 3, 4]);
    assert_eq!(active_group(&app), [1, 2]);
    tap(&mut app, &[], KeyCode::KeyA);
    let interface = |app: &App| app.world().resource::<ControlStates>().of(local_player(app)).interface;
    let awaiting = InterfaceState::awaiting(PanelCommand::Attack);
    assert_eq!(interface(&app), awaiting);
    remove(&mut app, 3);
    app.update();
    assert_eq!((selected(&app), active_group(&app)), (vec![1, 2, 4], vec![1, 2]));
    remove(&mut app, 1);
    app.update();
    assert_eq!((active_group(&app), interface(&app)), (vec![2], awaiting));
    remove(&mut app, 2);
    app.update();
    assert_eq!(
        (selected(&app), active_group(&app)),
        (vec![4], vec![4]),
        "the Guards' group"
    );
    assert_eq!(interface(&app), InterfaceState::DefaultState);
}
