//! The CommandPanel (`systems/controls/selection_and_hud.md > CommandPanel`): the ActiveGroup's commands in the
//! 4 × 3 grid (`crate::command_ui::interface::panel`), shown while everything selected is the local player's.
//!
//! Placeholder buttons until the art exists (Q0001): a short label in the middle and the slot's hotkey, from the
//! bindings, in the top-left corner. Every label keeps the label size at 720p (slice M3-K2a: six characters at most,
//! `crate::command_ui::PanelCommand::label`); a label never wraps, and one too long for its button at the label size
//! (on a surface too small for it) is drawn smaller, to fit on one line ([`label_px`]). Each look has its colours:
//! common commands blue, group commands violet (the design wants the two "visually distinguished"), the
//! PendingCommand lit up in yellow, disabled commands grey and dim (a Train or a Cancel Production the command gate
//! would refuse too: `crate::command_ui::Conditions`), unavailable ones near-black with a dark red border, Back
//! slate. A click on a button is a press of its slot, handled with the hotkeys (`crate::command_ui`), which finds the
//! button by the same geometry ([`CommandGrid`]); the button under the pointer has its full name in the InfoPanel
//! (`super::info_panel::CommandHint`), which its short label shortens.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::layout::{CommandGrid, SECTION_BORDER_PX, section_rect};
use super::{HudSection, SectionLabel, show_node, text};
use crate::ClientMode;
use crate::command_ui::interface::{commandable, panel, slot_index};
use crate::command_ui::{ButtonLook, Conditions, Panel, PanelSlot};
use crate::input::{Bindings, Button, CommandSlot, InputAction};
use crate::platform::SurfaceSize;
use crate::selection::ControlStates;
use crate::sim_driver::{LocalPlayer, SimRes};

/// Sizes of a button's label and hotkey (placeholders).
const LABEL_VH: f32 = 1.5;
const HOTKEY_VH: f32 = 1.2;
/// The room a label leaves on each side, inside the button's border (placeholder).
const LABEL_MARGIN_VH: f32 = 0.2;
/// The advance of every glyph of the HUD's font, in ems: Bevy's default font, Fira Mono, is monospaced, 600 units to
/// its em of 1000.
const GLYPH_ADVANCE_EM: f32 = 0.6;
/// The borders of a button, and of a lit one: the PendingCommand.
const BORDER_PX: f32 = 1.0;
const LIT_BORDER_PX: f32 = 2.0;

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
            let conditions = Conditions::of(&self.sim, self.player.0, &state.selection);
            panel(&state.selection, state.interface, &conditions)
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
        let label_size = label_px(button.label(), rect.width(), surface);
        spawn_button_content(&mut commands, entity, button, label_size, hotkey);
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
    let lit = shown.look == ButtonLook::Pending;
    (
        Node {
            position_type: PositionType::Absolute,
            left: px(at.x),
            top: px(at.y),
            width: px(side),
            height: px(side),
            border: UiRect::all(px(if lit { LIT_BORDER_PX } else { BORDER_PX })),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(background),
        BorderColor::all(border),
    )
}

/// The font size, in logical pixels, of `label` on a button of side `side` on a surface of `surface` logical pixels:
/// the label size, or less for a label too long for one line at that size, which then fills the room inside the
/// thicker border of a lit button and the margins, so that a label keeps its size when its button lights up. Any
/// label fits, its glyphs all as wide (the font is monospaced: [`GLYPH_ADVANCE_EM`]).
fn label_px(label: &str, side: f32, surface: Vec2) -> f32 {
    let vh = surface.y / 100.0;
    let room = side - 2.0 * (LIT_BORDER_PX + LABEL_MARGIN_VH * vh);
    let ems = GLYPH_ADVANCE_EM * label.chars().count().max(1) as f32;
    (LABEL_VH * vh).min(room / ems).max(0.0)
}

/// A button's label in the middle, of `label_size` logical pixels ([`label_px`]), and its hotkey in the top-left
/// corner. Neither wraps.
fn spawn_button_content(commands: &mut Commands, button: Entity, shown: PanelSlot, label_size: f32, hotkey: String) {
    let (_, _, color) = colors(shown.look);
    commands.spawn((
        Text::new(shown.label()),
        TextFont::from_font_size(FontSize::Px(label_size)),
        TextColor(color),
        TextLayout::no_wrap(),
        ChildOf(button),
    ));
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: vh(0.4),
            top: vh(0.2),
            ..default()
        },
        text(hotkey, HOTKEY_VH, color.with_alpha(0.8)),
        TextLayout::no_wrap(),
        ChildOf(button),
    ));
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use sc_sim::ObjectKind;

    use super::*;
    use crate::command_ui::PanelButton;
    use crate::command_ui::interface::{BuildMenu, TRANSFORMING_GRID, commands, grid, menu_grid};

    /// Every label a command button can show: of each command of each type's panels in any state (its DefaultState
    /// grid, its build or Warp In menus, what it constructs, places or warps in and the Cancels: [`commands`]; an
    /// unfinished building's Cancel), and Back.
    fn every_label() -> BTreeSet<&'static str> {
        let label = |button| {
            let look = ButtonLook::Common;
            PanelSlot {
                button,
                look,
                why: None,
            }
            .label()
        };
        let mut labels = BTreeSet::from([label(PanelButton::Back)]);
        for &kind in ObjectKind::ALL {
            let menus = BuildMenu::ALL.map(|menu| menu_grid(kind, menu));
            let grids = [grid(kind), TRANSFORMING_GRID].into_iter().chain(menus);
            for command in grids.flatten().flatten().chain(commands(kind)) {
                labels.insert(label(PanelButton::Command(command)));
            }
        }
        labels
    }

    /// Slice M3-K2a: every label of every panel keeps the label size on its button at 720p, lit or not, on one line
    /// ("Cancel Ready" and the other long labels came out at about 6 px there, slice M3-U having made a label too
    /// long for its button shrink to fit), and so on every surface of these, of any shape, down to 800 × 600. That is
    /// six characters at most at 720p: a seventh would not fit. The full names are the hint's
    /// (`super::super::info_panel::hint`).
    #[test]
    fn every_label_keeps_the_label_size_at_720p() {
        let labels = every_label();
        for label in [
            "PK", "RP", "SK", "Adv.", "Place", "Cancel", "Drop", "Drones", "Raise", "Back",
        ] {
            assert!(labels.contains(label), "{label} is shown");
        }
        let surfaces = [
            Vec2::new(1280.0, 720.0),
            Vec2::new(800.0, 600.0),
            Vec2::new(1366.0, 768.0),
            Vec2::new(1920.0, 1080.0),
            Vec2::new(2560.0, 1080.0),
            Vec2::new(3840.0, 2160.0),
            Vec2::new(1080.0, 1920.0),
        ];
        for surface in surfaces {
            let side = CommandGrid::on(surface).cell;
            let full = LABEL_VH * (surface.y / 100.0);
            for label in &labels {
                assert_eq!(label_px(label, side, surface), full, "{label} on {surface}");
                let width = GLYPH_ADVANCE_EM * full * label.chars().count() as f32;
                assert!(
                    width <= side - 2.0 * LIT_BORDER_PX,
                    "{label} on {surface}: {width} in {side}"
                );
            }
        }
        let surface = Vec2::new(1280.0, 720.0);
        let side = CommandGrid::on(surface).cell;
        assert!(label_px("Cancel", side, surface) == LABEL_VH * 7.2);
        assert!(
            label_px("Reverse", side, surface) < LABEL_VH * 7.2,
            "seven characters don't fit"
        );
    }

    /// A label too long for its button, as none is now, would be drawn smaller rather than wrap (slice M3-U: the
    /// highlighted "Warp RP" of a Beacon's Warp In menu wrapped onto two lines): it fits on one line, lit or not, and
    /// keeps its size when its button lights up.
    #[test]
    fn a_label_too_long_for_its_button_is_drawn_smaller_on_one_line() {
        let surface = Vec2::new(1280.0, 720.0);
        let side = CommandGrid::on(surface).cell;
        for label in ["Warp RP", "Cancel Ready Building"] {
            let size = label_px(label, side, surface);
            let width = GLYPH_ADVANCE_EM * size * label.chars().count() as f32;
            assert!(size > 0.0 && size < LABEL_VH * 7.2, "{label}: {size}");
            assert!(
                width <= side - 2.0 * (LIT_BORDER_PX + LABEL_MARGIN_VH * 7.2) + 1e-3,
                "{label}: {width} in {side}"
            );
        }
    }

    /// The font the labels are measured for: Bevy's default, every glyph [`GLYPH_ADVANCE_EM`] wide (its `hmtx`
    /// table's advances, of its `head` table's units to the em, but the null glyph's).
    #[test]
    fn the_hud_font_is_monospaced_at_the_advance_the_labels_are_measured_with() {
        let font = bevy::text::DEFAULT_FONT_DATA;
        let u16_at = |at: usize| u16::from_be_bytes([font[at], font[at + 1]]);
        let u32_at = |at: usize| u32::from_be_bytes([font[at], font[at + 1], font[at + 2], font[at + 3]]);
        let table = |tag: &[u8; 4]| {
            (0..usize::from(u16_at(4)))
                .map(|i| 12 + 16 * i)
                .find(|&record| &font[record..record + 4] == tag)
                .map(|record| u32_at(record + 8) as usize)
                .expect("the font has the table")
        };
        let units_per_em = f32::from(u16_at(table(b"head") + 18));
        let metrics = usize::from(u16_at(table(b"hhea") + 34));
        let advances: BTreeSet<u16> = (0..metrics).map(|i| u16_at(table(b"hmtx") + 4 * i)).collect();
        assert_eq!(
            advances,
            BTreeSet::from([0, 600]),
            "the null glyph's and all the others'"
        );
        assert_eq!(600.0 / units_per_em, GLYPH_ADVANCE_EM);
    }

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
