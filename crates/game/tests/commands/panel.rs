//! The CommandPanel (`> CommandPanel`): its layout for the M1 units, its hotkeys and buttons, who receives its
//! commands (no smart-casting: all of them, design change 0071), and that only the local player's units do.

use bevy::prelude::*;
use sc_sim::defs::RIOT_SHIELD_TOGGLE_TICKS;
use sc_sim::{Ability, ObjectId, OrderKind, PlayerId, ShieldMode};
use space_crystals::command_ui::{ButtonLook, InterfaceState, PanelButton, PanelCommand};
use space_crystals::hud::command_panel::{CommandButton, CommandPanelView};
use space_crystals::hud::{HudSection, SectionLabel};

use super::*;
use KeyCode::{F2, KeyA, KeyE, KeyR, KeyX, ShiftLeft, Tab};

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
/// add their Raise Shield in R, a group command (`factions/gdo/units/peacekeeper.md > Riot Shield`), labelled
/// "Raise" to fit its button at 720p. Each button is labelled with its command and its hotkey. While a target is
/// awaited Z is Back, and disabled and unavailable buttons look apart.
#[test]
fn a_mixed_selection_shows_the_basic_combat_unit_grid() {
    use ButtonLook::{Common, Group, Unavailable};
    use PanelCommand::*;
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
            (slot(0, 3), c(RaiseShield), Group),
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
        "Move", "Hold", "Raise", "Attack", "Patrol", "Stop", "Q", "E", "R", "A", "S", "X",
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

/// `factions/colonists/units/sabre_legionnaire.md > ObjectInterfaceState`: "BasicCombatUnitInterfaceState, plus" R,
/// Convert (slice M3-B, `convert`): both Legionnaires show the Guard's grid with their Convert in R, "R: Convert to
/// Rifle Legionnaire" on the Sabre's, "R: Convert to Sabre Legionnaire" on the Rifle's (each form's own, design change
/// 0198). The portals show the production panel (slice M3-D, `portals`), the Beacon and the Drone their own (slice
/// M3-J1, `warp_in`, `deploy`); the Egress Node its F (slice M3-J2, `colonists::gathering`).
#[test]
fn the_legionnaires_show_the_basic_combat_unit_grid() {
    use ObjectKind::{RifleLegionnaire, SabreLegionnaire};
    use space_crystals::command_ui::interface::templates::{InterfaceTemplate, grid};
    for (kind, other) in [
        (SabreLegionnaire, RifleLegionnaire),
        (RifleLegionnaire, SabreLegionnaire),
    ] {
        let mut legionnaire = grid(ObjectKind::Guard);
        legionnaire[3] = Some(PanelCommand::Convert(other));
        assert_eq!(InterfaceTemplate::of(kind), Some(InterfaceTemplate::BasicCombatUnit));
        assert_eq!(grid(kind), legionnaire, "{kind:?}");
    }
    let f = grid(ObjectKind::EgressNode);
    assert_eq!(f[7], Some(PanelCommand::SelectReturningDrones));
    assert_eq!(f.iter().flatten().count(), 1);
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
/// the whole selection (common commands); Shift queues them (`systems/controls/commands.md > Unit Command`).
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
/// to the ActiveGroup alone: the Peacekeepers' Raise Shield, `riot_shield.rs`.)
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

/// A point of the ground in the world view, clear of every object, so that a click there is on the ground.
fn bare_ground(app: &App) -> Vec2 {
    let sim = app.world().resource::<SimRes>();
    let mut candidates =
        (0..8).flat_map(|i| (0..4).map(move |j| Vec2::new(800.0 + 200.0 * i as f32, 650.0 + 180.0 * j as f32)));
    candidates
        .find(|&p| sim.objects().all(|o| o.pos.distance(p) > 150.0))
        .expect("bare ground")
}

/// `> SelectionGroup > No smart-casting` (design change 0071) with `> CommandPanel`: every command the panel shows,
/// by its hotkey and by its button, reaches every unit it is issued to, never some of them: a common command the
/// whole selection, a group command the whole ActiveGroup. For each ActiveGroup of a selection of three
/// Peacekeepers and two Guards (Tab), each command shown is used: an immediate one is issued, one that awaits a
/// target gets a click on the ground. The orders and abilities given must go to exactly those units, all at once.
/// A shown command this does not know how to give, or that gives nothing, fails it. (The unavailable Reverse and
/// AttackGround go to nobody: no M1 unit has them.) Raise Shield's hotkey turns the three Peacekeepers into
/// Peacekeepers (Shielded), whose group becomes the ActiveGroup (design change 0173), so its button, in R, is then
/// Lower Shield, which goes to the same three and turns them back.
#[test]
fn every_panel_command_reaches_all_its_recipients() {
    use PanelCommand::*;
    let mut app = game();
    drag(&mut app, Vec2::new(850.0, 760.0), Vec2::new(1400.0, 1140.0));
    let all = [1, 2, 3, 4, 5];
    assert_eq!(selected(&app), all);
    let mut used = Vec::new();
    for _ in 0..2 {
        let states = app.world().resource::<ControlStates>();
        let group = states.selection(PlayerId(1)).active_group().expect("a selection");
        let active: Vec<u32> = group.instances.iter().map(|id| id.0).collect();
        for (slot, button, look) in buttons(&mut app) {
            let PanelButton::Command(command) = button else {
                continue;
            };
            let expected = match look {
                ButtonLook::Common => all.to_vec(),
                ButtonLook::Group => active.clone(),
                ButtonLook::Unavailable => continue,
                other => panic!("{command:?} looks {other:?}"),
            };
            for by_button in [false, true] {
                let before = (orders(&app).len(), abilities(&app).len());
                if by_button {
                    click_button(&mut app, slot, &[]);
                } else {
                    tap(&mut app, &[], slot.default_key());
                    pause(&mut app);
                }
                if interface(&app) == InterfaceState::awaiting(command) {
                    let ground = bare_ground(&app);
                    click(&mut app, ground, &[]);
                }
                assert_eq!(interface(&app), InterfaceState::DefaultState, "{command:?}");
                let new_orders = orders(&app).split_off(before.0).into_iter().map(|o| o.units);
                let new_abilities = abilities(&app)
                    .split_off(before.1)
                    .into_iter()
                    .map(|(_, units, _)| units);
                let reached: Vec<Vec<u32>> = new_orders.chain(new_abilities).collect();
                assert_eq!(
                    reached,
                    std::slice::from_ref(&expected),
                    "{command:?}, by button: {by_button}"
                );
            }
            used.push(command);
        }
        tap(&mut app, &[], Tab);
    }
    let pks = [Move, HoldPosition, RaiseShield, Attack, Patrol, Stop];
    let guards = [Move, HoldPosition, Attack, Patrol, Stop];
    assert_eq!(used, [pks.as_slice(), guards.as_slice()].concat());
}

/// 0071 Verification 3: four Peacekeepers selected and the Riot Shield key (R): all four raise their shields, and
/// are Peacekeepers (Shielded) from the toggle's start (design change 0173).
#[test]
fn the_riot_shield_key_raises_every_selected_peacekeeper() {
    let mut app = game();
    let fourth = spawn(
        &mut app,
        ObjectKind::Peacekeeper,
        Some(PlayerId(1)),
        Vec2::new(1050.0, 1100.0),
    );
    let four = [1, 2, 5, fourth];
    select_units(&mut app, &four);
    tap(&mut app, &[], KeyR);
    let raise = Ability::RiotShield { raise: true };
    assert_eq!(abilities(&app), [(PlayerId(1), four.to_vec(), raise)]);
    updates(&mut app, RIOT_SHIELD_TOGGLE_TICKS as usize + 2);
    let sim = app.world().resource::<SimRes>();
    let shields = four.map(|id| {
        let unit = sim.object(ObjectId(id)).unwrap();
        (unit.kind, sc_sim::riot_shield::mode(unit))
    });
    let shielded = (ObjectKind::PeacekeeperShielded, Some(ShieldMode::Raised));
    assert_eq!(shields, [shielded; 4]);
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
    // Player 2 has a camera of their own, at their start (slice M3-J3): back to this test's view.
    app.world_mut().resource_mut::<CameraView>().center = VIEW_CENTER;
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
