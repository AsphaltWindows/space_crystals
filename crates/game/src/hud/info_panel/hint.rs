//! The full name of the command button under the pointer, at the foot of the InfoPanel (slice M3-S): a button's label
//! is short enough for its button at 720p, and may shorten its command's name ("Raise" for Raise Shield), which this
//! keeps in sight with its hotkey, "R: Raise Shield" (`crate::command_ui::PanelCommand::name`). The client has no
//! tooltips; the design names no place for a command's name (placeholder until the HUD pass).

use bevy::prelude::*;

use super::super::command_panel::CommandPanelView;
use super::super::layout::CommandGrid;
use super::super::{LABEL_VH, set_text, show_node, text};
use crate::ClientMode;
use crate::command_ui::Panel;
use crate::command_ui::interface::slot_index;
use crate::input::{ActionState, CommandSlot};
use crate::platform::SurfaceSize;

/// The colour of the hint, set apart from the InfoPanel's lines (placeholder).
const HINT_COLOR: Color = Color::srgb(0.98, 0.90, 0.55);

/// The InfoPanel's line naming the command button under the pointer; hidden while there is none.
#[derive(Component, Copy, Clone, Debug, PartialEq, Eq)]
pub struct CommandHint;

/// The hint's text node, hidden, for the InfoPanel's content (`super::spawn_content`).
pub(in crate::hud) fn spawn(panel: &mut ChildSpawnerCommands) {
    let hidden = Node {
        display: Display::None,
        ..default()
    };
    panel.spawn((CommandHint, text("", LABEL_VH, HINT_COLOR), hidden));
}

/// What the hint says of `slot` of `panel`, whose hotkeys are `hotkeys` (row by row from the top left): its hotkey
/// and its full name, or the name alone for an unbound slot; `None` for an empty slot.
pub fn hint(panel: &Panel, hotkeys: &[String], slot: CommandSlot) -> Option<String> {
    let name = panel.get(slot)?.name();
    Some(match hotkeys.get(slot_index(slot)).filter(|key| !key.is_empty()) {
        Some(key) => format!("{key}: {name}"),
        None => name,
    })
}

/// Names the button under the pointer, in a game mode: the slot the cursor is in (the geometry the clicks use,
/// [`CommandGrid`]) of the panel shown.
pub(in crate::hud) fn update_command_hint(
    actions: Res<ActionState>,
    mode: Res<ClientMode>,
    size: Res<SurfaceSize>,
    view: Res<CommandPanelView>,
    mut hints: Query<(&mut Text, &mut Node), With<CommandHint>>,
) {
    let slot = (mode.is_game() && actions.pointer().over_ui)
        .then(|| actions.cursor())
        .flatten()
        .and_then(|cursor| CommandGrid::on(size.logical()).slot_at(cursor));
    let shown = slot.and_then(|slot| hint(&view.panel, &view.hotkeys, slot));
    for (mut text, mut node) in &mut hints {
        set_text(&mut text, shown.as_deref().unwrap_or(""));
        show_node(&mut node, shown.is_some());
    }
}
