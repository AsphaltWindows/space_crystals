//! The CommandPanel (`systems/controls/selection_and_hud.md > CommandPanel`): the ActiveGroup's commands in the
//! 4 × 3 grid (`crate::command_ui::interface::panel`), shown while everything selected is the local player's.
//!
//! Placeholder buttons until the art exists (Q0001): a short label in the middle and the slot's hotkey, from the
//! bindings, in the top-left corner. Each look has its colours: common commands blue, group commands violet (the
//! design wants the two "visually distinguished"), the PendingCommand lit up in yellow, disabled commands grey and
//! dim, unavailable ones near-black with a dark red border, Back slate. A toggle that is on (the Riot Shields up)
//! has a thick green border, and its label says what pressing it does. A click on a button is a press of its slot,
//! handled with the hotkeys (`crate::command_ui`), which finds the button by the same geometry ([`CommandGrid`]).

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::layout::{CommandGrid, SECTION_BORDER_PX, section_rect};
use super::{HudSection, SectionLabel, show_node, text};
use crate::ClientMode;
use crate::command_ui::interface::{commandable, panel, slot_index};
use crate::command_ui::{ButtonLook, Modes, Panel, PanelSlot};
use crate::input::{Bindings, Button, CommandSlot, InputAction};
use crate::platform::SurfaceSize;
use crate::selection::ControlStates;
use crate::sim_driver::{LocalPlayer, SimRes};

/// Sizes of a button's label and hotkey (placeholders).
const LABEL_VH: f32 = 1.5;
const HOTKEY_VH: f32 = 1.2;
/// The border of a toggle that is on (placeholder).
const ON_BORDER: Color = Color::srgb(0.45, 0.95, 0.55);

/// A button of the panel: its slot and what it shows.
#[derive(Component, Copy, Clone, Debug, PartialEq, Eq)]
pub struct CommandButton {
    pub slot: CommandSlot,
    pub shown: PanelSlot,
}

/// The panel shown, the slots' hotkey labels, and the buttons' entities.
#[derive(Resource, Default, Debug)]
pub struct CommandPanelView {
    pub panel: Panel,
    /// Each slot's hotkey label, row by row from the top left.
    pub hotkeys: Vec<String>,
    /// The surface size the buttons were placed for.
    surface: Option<Vec2>,
    entities: Vec<Entity>,
}

/// The label of `slot`'s hotkey: its first key binding, e.g. "Q" (`> CommandPanel > Grid Layout`, rebindable).
pub fn hotkey_label(bindings: &Bindings, slot: CommandSlot) -> String {
    let chord = bindings
        .of(InputAction::CommandSlot(slot))
        .filter_map(|binding| binding.chord())
        .find(|chord| matches!(chord.button, Button::Key(_)));
    let Some(chord) = chord else {
        return String::new();
    };
    let Button::Key(key) = chord.button else {
        return String::new();
    };
    let name = format!("{key:?}");
    let key = name
        .strip_prefix("Key")
        .or_else(|| name.strip_prefix("Digit"))
        .unwrap_or(&name);
    let held = [
        (chord.modifiers.ctrl, "Ctrl+"),
        (chord.modifiers.alt, "Alt+"),
        (chord.modifiers.shift, "Shift+"),
    ];
    let mut label: String = held.iter().filter(|(on, _)| *on).map(|(_, m)| *m).collect();
    label.push_str(key);
    label
}

/// What the panel shows: the local player's ControlState, in a game mode, on a surface of some size, with the
/// bindings' hotkeys.
#[derive(SystemParam)]
pub(super) struct PanelSource<'w> {
    mode: Res<'w, ClientMode>,
    size: Res<'w, SurfaceSize>,
    sim: Res<'w, SimRes>,
    player: Res<'w, LocalPlayer>,
    states: Res<'w, ControlStates>,
    bindings: Res<'w, Bindings>,
}

impl PanelSource<'_> {
    /// `> CommandPanel`: only displayed while all selected objects are the player's.
    fn panel(&self) -> Panel {
        let state = self.states.of(self.player.0);
        if self.mode.is_game() && commandable(&self.sim, self.player.0, &state.selection) {
            let modes = Modes::of(&self.sim, &state.selection);
            panel(&state.selection, state.interface, &modes)
        } else {
            Panel::default()
        }
    }
}

/// Rebuilds the buttons when the panel, the bindings or the surface size changes.
pub(super) fn update_command_panel(
    mut commands: Commands,
    source: PanelSource,
    mut view: ResMut<CommandPanelView>,
    sections: Query<(Entity, &HudSection)>,
    mut labels: Query<(&SectionLabel, &mut Node)>,
) {
    let surface = source.size.logical();
    let shown = source.panel();
    let rebound = source.bindings.is_changed() || view.hotkeys.is_empty();
    if !rebound && shown == view.panel && view.surface == Some(surface) {
        return;
    }
    let Some((section, _)) = sections.iter().find(|(_, s)| **s == HudSection::CommandPanel) else {
        return;
    };
    if rebound {
        view.hotkeys = CommandSlot::all()
            .map(|slot| hotkey_label(&source.bindings, slot))
            .collect();
    }
    for entity in view.entities.drain(..) {
        commands.entity(entity).despawn();
    }
    for (label, mut node) in &mut labels {
        if label.0 == HudSection::CommandPanel {
            show_node(&mut node, shown.is_empty());
        }
    }
    // Absolute children are placed from the section's padding box: inside its border.
    let grid = CommandGrid::on(surface);
    let origin = section_rect(HudSection::CommandPanel, surface).min + Vec2::splat(SECTION_BORDER_PX);
    let mut entities = Vec::new();
    for (slot, button) in shown.slots() {
        let Some(button) = button else {
            continue;
        };
        let rect = grid.slot_rect(slot);
        let entity = commands
            .spawn((
                Name::new("Command button"),
                CommandButton { slot, shown: button },
                button_node(rect.min - origin, rect.width(), button),
                ChildOf(section),
            ))
            .id();
        let hotkey = view.hotkeys.get(slot_index(slot)).cloned().unwrap_or_default();
        spawn_button_content(&mut commands, entity, button, hotkey);
        entities.push(entity);
    }
    view.entities = entities;
    view.panel = shown;
    view.surface = Some(surface);
}

/// A look's background, border and text colours (placeholders).
fn colors(look: ButtonLook) -> (Color, Color, Color) {
    let text = Color::srgb(0.90, 0.92, 0.95);
    match look {
        ButtonLook::Common => (Color::srgb(0.15, 0.21, 0.30), Color::srgb(0.45, 0.63, 0.88), text),
        ButtonLook::Group => (Color::srgb(0.21, 0.15, 0.30), Color::srgb(0.74, 0.50, 0.96), text),
        ButtonLook::Pending => (
            Color::srgb(0.38, 0.33, 0.08),
            Color::srgb(1.00, 0.88, 0.28),
            Color::srgb(1.00, 0.97, 0.80),
        ),
        ButtonLook::Disabled => (
            Color::srgb(0.13, 0.14, 0.16),
            Color::srgb(0.30, 0.32, 0.36),
            Color::srgb(0.46, 0.48, 0.52),
        ),
        ButtonLook::Unavailable => (
            Color::srgb(0.07, 0.07, 0.08),
            Color::srgb(0.34, 0.16, 0.14),
            Color::srgb(0.30, 0.28, 0.28),
        ),
        ButtonLook::Back => (Color::srgb(0.20, 0.23, 0.27), Color::srgb(0.62, 0.67, 0.74), text),
    }
}

/// A button's node, background and border.
fn button_node(at: Vec2, side: f32, shown: PanelSlot) -> impl Bundle {
    let (background, border, _) = colors(shown.look);
    let border = if shown.on { ON_BORDER } else { border };
    let lit = shown.look == ButtonLook::Pending || shown.on;
    (
        Node {
            position_type: PositionType::Absolute,
            left: px(at.x),
            top: px(at.y),
            width: px(side),
            height: px(side),
            border: UiRect::all(px(if lit { 2.0 } else { 1.0 })),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(background),
        BorderColor::all(border),
    )
}

/// A button's label in the middle and its hotkey in the top-left corner.
fn spawn_button_content(commands: &mut Commands, button: Entity, shown: PanelSlot, hotkey: String) {
    let (_, _, color) = colors(shown.look);
    commands.spawn((text(shown.label(), LABEL_VH, color), ChildOf(button)));
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: vh(0.4),
            top: vh(0.2),
            ..default()
        },
        text(hotkey, HOTKEY_VH, color.with_alpha(0.8)),
        ChildOf(button),
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hotkey_labels_name_the_bound_key() {
        let mut bindings = Bindings::default();
        let slot = |row, column| CommandSlot::new(row, column).unwrap();
        assert_eq!(hotkey_label(&bindings, slot(0, 0)), "Q");
        assert_eq!(hotkey_label(&bindings, slot(2, 3)), "V");
        let action = InputAction::CommandSlot(slot(0, 0));
        bindings.unbind(action);
        assert_eq!(hotkey_label(&bindings, slot(0, 0)), "");
        let chord = crate::input::Chord::key(KeyCode::Digit7).with(crate::input::Modifiers::CTRL);
        bindings.bind(action, crate::input::Binding::Press(chord));
        assert_eq!(hotkey_label(&bindings, slot(0, 0)), "Ctrl+7");
    }
}
