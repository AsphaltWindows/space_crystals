//! The CommandPanel (`> CommandPanel`): its layout for the M1 units, its hotkeys and buttons, who receives its
//! commands, and that only the local player's units do.

use bevy::prelude::*;
use sc_sim::{OrderKind, PlayerId};
use space_crystals::command_ui::{ButtonLook, InterfaceState, PanelButton, UnitCommand};
use space_crystals::hud::command_panel::{CommandButton, CommandPanelView};
use space_crystals::hud::{HudSection, SectionLabel};

use super::*;
use KeyCode::{F2, KeyA, KeyE, KeyX, ShiftLeft, Tab};

/// The buttons the HUD shows: slot, button, look.
fn buttons(app: &mut App) -> Vec<(CommandSlot, PanelButton, ButtonLook)> {
    let mut query = app.world_mut().query::<&CommandButton>();
    let mut shown: Vec<_> = query
        .iter(app.world())
        .map(|b| (b.slot, b.shown.button, b.shown.look))
        .collect();
    shown.sort_by_key(|(slot, ..)| *slot);
    shown
}

/// Whether the CommandPanel section shows its name, as it does while it has no buttons.
fn name_shown(app: &mut App) -> bool {
    let mut query = app.world_mut().query::<(&SectionLabel, &Node)>();
    query
        .iter(app.world())
        .any(|(label, node)| label.0 == HudSection::CommandPanel && node.display != Display::None)
}

fn texts(app: &mut App) -> Vec<String> {
    let mut query = app.world_mut().query::<&Text>();
    query.iter(app.world()).map(|t| t.0.clone()).collect()
}

/// `> BasicCombatUnitInterfaceState > DefaultState commands` and `> Standard Slot Assignments` (0002): a selection
/// of Peacekeepers and Guards shows Move (Q), Hold (E), Attack (A), Patrol (S) and Stop (X), all common to both
/// types; Reverse (W) and AttackGround (D) unavailable to both; Z, C and V empty. The Peacekeepers, the ActiveGroup,
/// add their Riot Shield in R, a group command (`factions/gdo/units.md > Peacekeeper > Riot Shield`). Each button is
/// labelled with its command and its hotkey. While a target is awaited Z is Back, and disabled and unavailable
/// buttons look apart.
#[test]
fn a_mixed_selection_shows_the_basic_combat_unit_grid() {
    use ButtonLook::{Common, Group, Unavailable};
    use UnitCommand::*;
    let mut app = game();
    assert!(buttons(&mut app).is_empty(), "nothing selected");
    assert!(name_shown(&mut app));
    select_mixed(&mut app);
    let c = PanelButton::Command;
    assert_eq!(
        buttons(&mut app),
        [
            (slot(0, 0), c(Move), Common),
            (slot(0, 1), c(Reverse), Unavailable),
            (slot(0, 2), c(HoldPosition), Common),
            (slot(0, 3), c(RiotShield), Group),
            (slot(1, 0), c(Attack), Common),
            (slot(1, 1), c(Patrol), Common),
            (slot(1, 2), c(AttackGround), Unavailable),
            (slot(2, 1), c(Stop), Common),
        ]
    );
    let view = app.world().resource::<CommandPanelView>();
    assert_eq!(view.hotkeys[..4], ["Q", "W", "E", "R"]);
    assert_eq!(view.hotkeys[8..], ["Z", "X", "C", "V"]);
    let shown = texts(&mut app);
    for label in [
        "Move",
        "Hold",
        "Raise Shield",
        "Attack",
        "Patrol",
        "Stop",
        "Q",
        "E",
        "R",
        "A",
        "S",
        "X",
    ] {
        assert!(shown.contains(&label.to_string()), "{label} in {shown:?}");
    }
    assert!(!name_shown(&mut app), "the section's name gives way");

    tap(&mut app, &[], KeyA);
    let looks = buttons(&mut app);
    assert!(looks.contains(&(CommandSlot::BACK, PanelButton::Back, ButtonLook::Back)));
    assert!(looks.contains(&(slot(1, 0), c(Attack), ButtonLook::Pending)));
    assert!(looks.contains(&(slot(0, 0), c(Move), ButtonLook::Disabled)));
    let mut query = app.world_mut().query::<(&CommandButton, &BackgroundColor)>();
    let backgrounds: Vec<(ButtonLook, Color)> = query.iter(app.world()).map(|(b, bg)| (b.shown.look, bg.0)).collect();
    let background = |look| backgrounds.iter().find(|(l, _)| *l == look).map(|(_, bg)| *bg).unwrap();
    assert_ne!(background(ButtonLook::Disabled), background(ButtonLook::Unavailable));
}

/// `> CommandPanel`: the panel shows only while every selected object is the local player's.
#[test]
fn enemy_and_neutral_selections_show_no_commands() {
    let mut app = game();
    click(&mut app, at(ENEMY_PK), &[]);
    assert_eq!(selected(&app), [ENEMY_PK]);
    assert!(buttons(&mut app).is_empty());
    click(&mut app, at(PATCH), &[]);
    assert_eq!(selected(&app), [PATCH]);
    assert!(buttons(&mut app).is_empty());
    assert!(name_shown(&mut app));
}

/// `> DefaultState`: HoldPosition (E) and Stop (X) are issued at once, by hotkey or by a click on their button, to
/// the whole selection (common commands); Shift queues them (`commands.md > Unit Command`).
#[test]
fn hotkeys_and_buttons_issue_immediate_commands() {
    let mut app = game();
    select_mixed(&mut app);
    let all = [1, 2, 3, 4];
    tap(&mut app, &[], KeyE);
    tap(&mut app, &[ShiftLeft], KeyX);
    click_button(&mut app, slot(0, 2), &[]);
    click_button(&mut app, slot(2, 1), &[ShiftLeft]);
    assert_eq!(
        orders(&app),
        [
            given(&all, OrderKind::Hold, false),
            given(&all, OrderKind::Stop, true),
            given(&all, OrderKind::Hold, false),
            given(&all, OrderKind::Stop, true),
        ]
    );
    assert_eq!(selected(&app), all, "a click on the panel selects nothing");
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    // An unavailable or empty slot does nothing.
    tap(&mut app, &[], KeyCode::KeyW);
    click_button(&mut app, slot(2, 3), &[]);
    assert_eq!(orders(&app).len(), 4);
}

/// `> CommandPanel`: a common command goes to the whole selection, whichever group is active. (A group command goes
/// to the ActiveGroup alone: the Peacekeepers' Riot Shield, `riot_shield.rs`.)
#[test]
fn common_commands_go_to_the_whole_selection_from_any_active_group() {
    let mut app = game();
    select_mixed(&mut app);
    tap(&mut app, &[], Tab);
    let states = app.world().resource::<ControlStates>();
    let active = states.selection(PlayerId(1)).active().unwrap();
    assert_eq!(active.kind, sc_sim::ObjectKind::Guard);
    tap(&mut app, &[], KeyE);
    tap(&mut app, &[], KeyA);
    click(&mut app, Vec2::new(1000.0, 1000.0), &[]);
    let given_orders = orders(&app);
    assert_eq!(given_orders.len(), 2);
    assert!(given_orders.iter().all(|o| o.units == [1, 2, 3, 4]), "{given_orders:?}");
}

/// `> CommandPanel`: "Commands can only be issued to objects owned by the player." Another player's selected unit
/// takes no command, by hotkey or right-click; after the debug switch (F2) the same unit is the local player's and
/// does, in that player's name.
#[test]
fn only_the_local_players_units_take_commands() {
    let mut app = game();
    click(&mut app, at(ENEMY_PK), &[]);
    right_click(&mut app, Vec2::new(1000.0, 1000.0), &[]);
    tap(&mut app, &[], KeyE);
    tap(&mut app, &[], KeyA);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    assert_eq!(selected(&app), [ENEMY_PK]);
    assert_eq!(orders(&app), []);

    tap(&mut app, &[], F2);
    assert_eq!(local_player(&app), PlayerId(2));
    click(&mut app, at(ENEMY_PK), &[]);
    tap(&mut app, &[], KeyE);
    assert_eq!(
        orders(&app),
        [Given {
            player: PlayerId(2),
            units: vec![ENEMY_PK],
            kind: OrderKind::Hold,
            queue: false
        }]
    );
}
