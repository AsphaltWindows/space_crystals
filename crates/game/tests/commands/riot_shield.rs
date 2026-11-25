//! The Peacekeeper's Riot Shield in the command flow (`factions/gdo/units/peacekeeper.md > Riot Shield (toggled
//! ability)`): shield down and shield up are two unit types, the Peacekeeper and the Peacekeeper (Shielded) (design
//! change 0173), which form separate SelectionGroups. The Peacekeeper's button in R raises the shields, the Peacekeeper
//! (Shielded)'s lowers them (design change 0174: R, a placeholder), each going to the ActiveGroup; the units switch
//! type at the press, and their group with it (design change 0198). The Peacekeeper (Shielded)'s panel shows no
//! Attack; a right-click on an enemy has the shielded units follow it (design change 0074) and an Attack leaves them
//! out (0075), while the other units of the selection attack it. A shield that isn't down is drawn in front of its
//! unit. The buttons' labels are short; the InfoPanel names the command under the pointer in full. The two types in a
//! selection, and the groups the units change at the press: `groups`; a press as a target state, and the orders a
//! unit takes while its shield turns (design change 0202): `target`.

mod groups;
mod target;

use bevy::prelude::*;
use sc_sim::defs::RIOT_SHIELD_TOGGLE_TICKS;
use sc_sim::{Ability, ObjectId, OrderKind, PlayerId, ShieldMode, Target};
use space_crystals::command_ui::cursor::PointerLook;
use space_crystals::command_ui::{ButtonLook, InterfaceState, PanelButton, PanelCommand, PanelSlot, PointerDisplay};
use space_crystals::hud::command_panel::{CommandButton, CommandPanelView};
use space_crystals::hud::info_panel::hint::CommandHint;
use space_crystals::object_render::{ObjectVisuals, RiotShieldVisual};

use super::*;
use KeyCode::{KeyA, KeyR, ShiftLeft, Tab};

const RAISE: Ability = Ability::RiotShield { raise: true };
const LOWER: Ability = Ability::RiotShield { raise: false };
const PEACEKEEPER: ObjectKind = ObjectKind::Peacekeeper;
const SHIELDED: ObjectKind = ObjectKind::PeacekeeperShielded;

/// The Riot Shield's slot, R (design change 0174, a placeholder).
fn r() -> CommandSlot {
    slot(0, 3)
}

/// Attack's slot, A.
fn a() -> CommandSlot {
    slot(1, 0)
}

/// The button the HUD shows in `slot`.
fn button(app: &mut App, slot: CommandSlot) -> Option<PanelSlot> {
    let mut query = app.world_mut().query::<&CommandButton>();
    query.iter(app.world()).find(|b| b.slot == slot).map(|b| b.shown)
}

/// The command of the button the HUD shows in `slot`, and its look.
fn command(app: &mut App, slot: CommandSlot) -> Option<(PanelCommand, ButtonLook)> {
    let shown = button(app, slot)?;
    match shown.button {
        PanelButton::Command(command) => Some((command, shown.look)),
        PanelButton::Back => None,
    }
}

fn texts(app: &mut App) -> Vec<String> {
    let mut query = app.world_mut().query::<&Text>();
    query.iter(app.world()).map(|t| t.0.clone()).collect()
}

/// The type of unit `id` and how far up its Riot Shield is, as the sim has them.
fn state(app: &App, id: u32) -> (ObjectKind, ShieldMode) {
    let sim = app.world().resource::<SimRes>();
    let unit = sim.object(ObjectId(id)).expect("in the game");
    (unit.kind, sc_sim::riot_shield::mode(unit).expect("a Peacekeeper"))
}

/// How far up the Riot Shield of unit `id` is.
fn shield(app: &App, id: u32) -> ShieldMode {
    state(app, id).1
}

/// The local player's SelectionGroups, in their order: type, objects, and whether it is the ActiveGroup.
fn groups(app: &App) -> Vec<(ObjectKind, Vec<u32>, bool)> {
    let states = app.world().resource::<ControlStates>();
    let selection = states.selection(local_player(app));
    let active = selection.active();
    selection
        .groups()
        .into_iter()
        .map(|g| {
            (
                g.kind(),
                g.instances.iter().map(|id| id.0).collect(),
                Some(g.key) == active,
            )
        })
        .collect()
}

/// The type of the ActiveGroup.
fn active_type(app: &App) -> Option<ObjectKind> {
    groups(app)
        .into_iter()
        .find(|(.., active)| *active)
        .map(|(kind, ..)| kind)
}

/// Player 1's use of `ability` by `units`.
fn used(units: &[u32], ability: Ability) -> (PlayerId, Vec<u32>, Ability) {
    (PlayerId(1), units.to_vec(), ability)
}

/// Long enough for a toggle to finish: the toggle time, and the ticks the command takes to reach the sim.
fn toggle_time(app: &mut App) {
    updates(app, RIOT_SHIELD_TOGGLE_TICKS as usize + 2);
}

/// Taps R, then runs tick by tick until the toggle it starts is over, and returns what each tick of the toggle leaves:
/// unit `id`'s type and shield, and the ActiveGroup's type. The toggle's first tick is the first in which `id`'s
/// type, as the sim has it, is no longer what it was: the tick that applies the press (design change 0198), the
/// toggle of a unit that isn't mid-shot starting at once.
fn toggle_with_r(app: &mut App, id: u32) -> Vec<(ObjectKind, ShieldMode, Option<ObjectKind>)> {
    let (before, _) = state(app, id);
    input(app).tap_key(KeyR);
    let mut ticks = Vec::new();
    for _ in 0..4 * RIOT_SHIELD_TOGGLE_TICKS {
        app.update();
        let (kind, mode) = state(app, id);
        if kind != before || !ticks.is_empty() {
            ticks.push((kind, mode, active_type(app)));
        }
        if !ticks.is_empty() && matches!(mode, ShieldMode::Raised | ShieldMode::Lowered) {
            return ticks;
        }
    }
    panic!("the toggle never ended: {ticks:?}");
}

/// The drawn shield of unit `id`: its visibility, its scale and the entity it is drawn with.
fn shield_visual(app: &mut App, id: u32) -> Option<(Visibility, f32, Entity)> {
    let mut query = app
        .world_mut()
        .query::<(&RiotShieldVisual, &Visibility, &Transform, &ChildOf)>();
    query
        .iter(app.world())
        .find(|(shield, ..)| shield.0 == ObjectId(id))
        .map(|(_, visibility, transform, parent)| (*visibility, transform.scale.x, parent.parent()))
}

/// The 0174 Verification (`> Riot Shield > Hotkey`): "R on Peacekeepers raises shields in 6 ticks. R on Peacekeeper
/// (Shielded) lowers them." The Peacekeepers' R is Raise Shield, labelled "Raise", next to their Attack; it turns
/// them into Peacekeepers (Shielded) at the press, their group with them (design change 0198: "The type, and so the
/// SelectionGroup, switches at the press"), the sim in the toggle's first tick, and the shield is all the way up in
/// its sixth (0171). Their panel has
/// Lower Shield, "Lower", in R and no Attack, and its R turns them back the same way. By hotkey or by button: an
/// ability, not an order, so no order is given and Shift queues nothing (design change 0151).
#[test]
fn r_raises_the_shields_in_six_ticks_and_lowers_them() {
    let mut app = game();
    select_box(&mut app, &[1, 2]);
    let shown = button(&mut app, r()).expect("a button in R");
    assert_eq!(
        (shown.button, shown.look, shown.label(), shown.name()),
        (
            PanelButton::Command(PanelCommand::RaiseShield),
            ButtonLook::Common,
            "Raise",
            "Raise Shield".to_string()
        )
    );
    assert_eq!(app.world().resource::<CommandPanelView>().hotkeys[3], "R");
    assert_eq!(command(&mut app, a()), Some((PanelCommand::Attack, ButtonLook::Common)));

    let up = toggle_with_r(&mut app, 1);
    assert_eq!(abilities(&app), [used(&[1, 2], RAISE)]);
    let going_up = (1..RIOT_SHIELD_TOGGLE_TICKS)
        .rev()
        .map(|left| ShieldMode::Raising { left });
    let expected: Vec<_> = going_up
        .chain([ShieldMode::Raised])
        .map(|mode| (SHIELDED, mode, Some(SHIELDED)))
        .collect();
    assert_eq!(up, expected);
    assert_eq!(up.len(), 6, "up in 6 ticks");
    assert_eq!(state(&app, 2), (SHIELDED, ShieldMode::Raised));
    assert_eq!(groups(&app), [(SHIELDED, vec![1, 2], true)]);

    let shown = button(&mut app, r()).unwrap();
    assert_eq!(
        (shown.button, shown.look, shown.label(), shown.name()),
        (
            PanelButton::Command(PanelCommand::LowerShield),
            ButtonLook::Common,
            "Lower",
            "Lower Shield".to_string()
        )
    );
    assert!(texts(&mut app).contains(&"Lower".to_string()));
    assert_eq!(button(&mut app, a()), None, "no Attack");

    let down = toggle_with_r(&mut app, 1);
    assert_eq!(abilities(&app)[1..], [used(&[1, 2], LOWER)]);
    let going_down = (1..RIOT_SHIELD_TOGGLE_TICKS)
        .rev()
        .map(|left| ShieldMode::Lowering { left });
    let expected: Vec<_> = going_down
        .chain([ShieldMode::Lowered])
        .map(|mode| (PEACEKEEPER, mode, Some(PEACEKEEPER)))
        .collect();
    assert_eq!(down, expected);
    assert_eq!(command(&mut app, r()).map(|(c, _)| c), Some(PanelCommand::RaiseShield));

    click_button(&mut app, r(), &[]);
    toggle_time(&mut app);
    assert_eq!([state(&app, 1), state(&app, 2)], [(SHIELDED, ShieldMode::Raised); 2]);
    click_button(&mut app, r(), &[]);
    toggle_time(&mut app);
    assert_eq!(
        [state(&app, 1), state(&app, 2)],
        [(PEACEKEEPER, ShieldMode::Lowered); 2]
    );
    assert_eq!(abilities(&app)[2..], [used(&[1, 2], RAISE), used(&[1, 2], LOWER)]);

    tap(&mut app, &[ShiftLeft], KeyR);
    assert_eq!(abilities(&app)[4..], [used(&[1, 2], RAISE)]);
    assert_eq!(orders(&app), []);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
}

/// "Right-click on an enemy unit while the shield is up: the Peacekeeper follows that unit" (design change 0074,
/// which replaced the Move to the clicked location), and "the player must lower the shield before ordering an
/// attack": the pointer previews a Move over an enemy, and the Peacekeeper (Shielded)'s panel has no Attack.
#[test]
fn with_their_shields_up_peacekeepers_follow_the_enemy_they_are_sent_at() {
    let mut app = game();
    select_box(&mut app, &[1, 2]);
    tap(&mut app, &[], KeyR);
    toggle_time(&mut app);
    let screen = screen_at(&app, at(ENEMY_PK));
    input(&mut app).cursor = Some(screen);
    app.update();
    assert_eq!(
        app.world().resource::<PointerLook>().display,
        Some(PointerDisplay::Move)
    );
    assert_eq!(button(&mut app, a()), None);
    tap(&mut app, &[], KeyA);
    assert_eq!(interface(&app), InterfaceState::DefaultState, "no Attack to target");

    right_click(&mut app, at(ENEMY_PK), &[]);
    let follow = OrderKind::Move(Target::Object(ObjectId(ENEMY_PK)));
    assert_eq!(orders(&app), [given(&[1, 2], follow, false)]);
    let current = object_orders(&app, 1).current.expect("on its way").order.kind;
    assert_eq!(current, follow);
}

/// Design change 0150 (`factions/gdo/units/peacekeeper.md > Riot Shield > Orders are kept`): the toggle is a mode
/// switch. With Peacekeepers walking a Shift-queued route, R and Shift + R raise and lower their shields and give no
/// order: each keeps its current order and its queue ("Shift has no effect"), whichever its type.
#[test]
fn toggling_the_shield_keeps_the_orders_shift_or_not() {
    let mut app = game();
    select_box(&mut app, &[1, 2]);
    let (first, second) = (Vec2::new(2000.0, 1200.0), Vec2::new(1300.0, 1200.0));
    right_click(&mut app, first, &[]);
    right_click(&mut app, second, &[ShiftLeft]);
    let given_so_far = orders(&app);
    let route = |app: &App, id: u32| {
        let orders = object_orders(app, id);
        (orders.current.map(|active| active.order.kind), orders.queue.len())
    };
    let before = [route(&app, 1), route(&app, 2)];
    assert!(
        before.iter().all(|&(current, queued)| current.is_some() && queued == 1),
        "{before:?}"
    );
    for modifiers in [&[][..], &[ShiftLeft][..], &[][..], &[ShiftLeft][..]] {
        tap(&mut app, modifiers, KeyR);
        toggle_time(&mut app);
        assert_eq!([route(&app, 1), route(&app, 2)], before, "{modifiers:?}");
    }
    assert_eq!(orders(&app), given_so_far, "no order given");
    assert_eq!(
        abilities(&app),
        [
            used(&[1, 2], RAISE),
            used(&[1, 2], LOWER),
            used(&[1, 2], RAISE),
            used(&[1, 2], LOWER)
        ]
    );
}

/// In a selection whose Guards can attack, Attack stays: with the Guards active (Tab: the shielded Peacekeepers'
/// group, active once they raised their shields, shows none), a click on an enemy in AwaitingTarget[Attack] sends the
/// Guards to attack it and leaves the shielded Peacekeepers out (design change 0075); a right-click on one sends the
/// Guards to attack it and has the Peacekeepers follow it (0074). Each click gives one order per distinct order, in
/// the order of the units' ids: the Peacekeepers' Follow first.
#[test]
fn a_mixed_selection_attacks_with_the_units_that_can() {
    let mut app = game();
    select_mixed(&mut app);
    tap(&mut app, &[], KeyR);
    toggle_time(&mut app);
    assert_eq!(active_type(&app), Some(SHIELDED));
    assert_eq!(button(&mut app, a()), None);
    tap(&mut app, &[], Tab);
    assert_eq!(command(&mut app, a()), Some((PanelCommand::Attack, ButtonLook::Common)));
    tap(&mut app, &[], KeyA);
    assert_eq!(interface(&app), InterfaceState::awaiting(PanelCommand::Attack));
    click(&mut app, at(ENEMY_PK), &[]);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    right_click(&mut app, at(ENEMY_GUARD), &[]);

    let attack = |id: u32| OrderKind::Attack(ObjectId(id));
    let follow = |id: u32| OrderKind::Move(Target::Object(ObjectId(id)));
    assert_eq!(
        orders(&app),
        [
            given(&[3, 4], attack(ENEMY_PK), false),
            given(&[1, 2], follow(ENEMY_GUARD), false),
            given(&[3, 4], attack(ENEMY_GUARD), false),
        ]
    );
}

/// The shield is drawn with its unit's visual while it isn't down, growing to its full size as it goes up, tick by
/// tick (from the toggle's first, when the unit becomes a Peacekeeper (Shielded), to its sixth, when it is all the
/// way up); a Guard has none. It is never `Visible` itself, so the fog that hides its unit hides it too (`fog.rs`).
#[test]
fn a_shield_is_drawn_while_it_is_not_down() {
    let mut app = game();
    let unit = app.world().resource::<ObjectVisuals>().by_id[&ObjectId(1)];
    assert_eq!(
        shield_visual(&mut app, 1).map(|(v, _, parent)| (v, parent)),
        Some((Visibility::Hidden, unit))
    );
    assert_eq!(shield_visual(&mut app, 3), None, "a Guard has no shield");

    click(&mut app, at(1), &[]);
    input(&mut app).tap_key(KeyR);
    let mut scales = Vec::new();
    for _ in 0..20 {
        if shield(&app, 1) == ShieldMode::Raised {
            break;
        }
        app.update();
        let (visibility, scale, _) = shield_visual(&mut app, 1).unwrap();
        if shield(&app, 1) != ShieldMode::Lowered {
            assert_eq!(visibility, Visibility::Inherited);
            scales.push(scale);
        }
    }
    assert_eq!(scales.len(), RIOT_SHIELD_TOGGLE_TICKS as usize);
    assert!(
        scales.is_sorted() && scales[0] < 1.0 && scales.last() == Some(&1.0),
        "{scales:?}"
    );

    tap(&mut app, &[], KeyR);
    toggle_time(&mut app);
    assert_eq!(shield_visual(&mut app, 1).map(|(v, ..)| v), Some(Visibility::Hidden));
}

/// Moves the pointer onto the command panel's button of `slot`, a frame.
fn hover_button(app: &mut App, slot: CommandSlot) {
    let surface = app.world().resource::<SurfaceSize>().logical();
    input(app).cursor = Some(CommandGrid::on(surface).slot_rect(slot).center());
    app.update();
}

/// What the InfoPanel's command hint says, while it is shown.
fn hint(app: &mut App) -> Option<String> {
    let mut query = app.world_mut().query_filtered::<(&Text, &Node), With<CommandHint>>();
    let (text, node) = query.single(app.world()).expect("one hint");
    (node.display != Display::None).then(|| text.0.clone())
}

/// The shield buttons' labels are short enough for their buttons at 720p ("Raise", "Lower"); the InfoPanel names
/// the command of the button under the pointer in full, with its hotkey: "R: Raise Shield" on the Peacekeeper's
/// panel, "R: Lower Shield" on the Peacekeeper (Shielded)'s, nothing over its empty A, nor once the pointer is off
/// the panel.
#[test]
fn the_info_panel_names_the_command_under_the_pointer() {
    let mut app = game();
    select_box(&mut app, &[1, 2]);
    assert_eq!(hint(&mut app), None);
    hover_button(&mut app, r());
    assert_eq!(hint(&mut app).as_deref(), Some("R: Raise Shield"));
    hover_button(&mut app, a());
    assert_eq!(hint(&mut app).as_deref(), Some("A: Attack"));

    tap(&mut app, &[], KeyR);
    toggle_time(&mut app);
    hover_button(&mut app, r());
    assert_eq!(hint(&mut app).as_deref(), Some("R: Lower Shield"));
    hover_button(&mut app, a());
    assert_eq!(hint(&mut app), None, "no Attack");

    input(&mut app).cursor = Some(screen_at(&app, Vec2::new(1000.0, 1000.0)));
    app.update();
    assert_eq!(hint(&mut app), None, "over the map");
}
