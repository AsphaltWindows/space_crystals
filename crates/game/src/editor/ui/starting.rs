//! The palette's Starting layouts section (slice M2-I, `core/map_editor.md > Starting Layouts`): the slot and
//! faction pickers, what the edited layout still lacks or whether it is complete (`> Skirmish Default restriction`:
//! "The editor shows what is still missing"), the layouts flagged as outdated (`> Outdated layouts`), the Layout
//! tool's palette with how many of each object the layout may still take, and the turn buttons. Rebuilt only when
//! what it shows changed. It comes last in the palette, below the fold at any window size (the UI is sized in `vh`),
//! so choosing the Layout tool scrolls the palette to it ([`reveal_section`]).

use bevy::prelude::*;
use sc_sim::{Faction, SlotId};

use super::lists::{list_node, rebuild};
use super::{DIM_TEXT, OK_TEXT, SMALL_VH, WARN_TEXT, header, label, row, small_button};
use crate::editor::actions::EditorAction;
use crate::editor::doc::EditorDoc;
use crate::editor::layouts::{PaletteEntry, layout_state, outdated_text, palette};
use crate::editor::objects::faction_name;
use crate::editor::orientation::Turn;
use crate::editor::state::EditorProblems;
use crate::editor::tools::{EditorTools, Tool};

/// The section's part that follows the document and the Layout tool's slot and faction.
#[derive(Component)]
pub struct LayoutPanel;

/// The section's header, which [`reveal_section`] scrolls to.
#[derive(Component)]
pub struct LayoutSectionHeader;

/// What [`LayoutPanel`] shows.
#[derive(Clone, Debug, PartialEq)]
pub struct LayoutView {
    /// The editable slots, each with the generated slots that follow it.
    slots: Vec<(SlotId, Vec<SlotId>)>,
    /// What the edited layout lacks, and whether it is fine.
    state: (String, bool),
    outdated: Option<String>,
    palette: Vec<PaletteEntry>,
}

/// A row of the turn buttons (R, Shift+R, F, Shift+F), for the structure the Place or the Layout tool places.
pub fn turn_row() -> impl Bundle {
    (
        row(),
        children![
            small_button(Turn::Clockwise.label(), EditorAction::Turn(Turn::Clockwise)),
            small_button(
                Turn::CounterClockwise.label(),
                EditorAction::Turn(Turn::CounterClockwise)
            ),
            small_button(Turn::FlipHorizontal.label(), EditorAction::Turn(Turn::FlipHorizontal)),
            small_button(Turn::FlipVertical.label(), EditorAction::Turn(Turn::FlipVertical)),
        ],
    )
}

/// Spawns the section into the palette.
pub fn spawn_section(panel: &mut ChildSpawnerCommands) {
    panel.spawn((LayoutSectionHeader, header("Starting layouts")));
    panel.spawn(label(
        "Each start position has a layout per faction; a game uses the one of the faction its player picks. The \
         Layout tool (T) places the chosen object, drags one, Delete removes it. Other factions show dimmed.",
        SMALL_VH,
        DIM_TEXT,
    ));
    panel.spawn((LayoutPanel, list_node()));
    panel.spawn(turn_row());
    panel.spawn(row()).with_children(|r| {
        r.spawn(small_button("Layout tool (T)", EditorAction::SetTool(Tool::Layout)));
    });
}

/// When the Layout tool is chosen, scrolls the palette so that the Starting layouts section starts at its top, once
/// the palette is laid out (never in a headless run, which lays out no UI). Leaves the palette alone otherwise.
pub fn reveal_section(
    tools: Res<EditorTools>,
    mut chosen: Local<Option<Tool>>,
    mut due: Local<bool>,
    headers: Query<(&ComputedNode, &UiGlobalTransform, &ChildOf), With<LayoutSectionHeader>>,
    mut panels: Query<(&ComputedNode, &UiGlobalTransform, &mut ScrollPosition)>,
) {
    if *chosen != Some(tools.tool) {
        *due |= tools.tool == Tool::Layout;
        *chosen = Some(tools.tool);
    }
    if !*due {
        return;
    }
    let Ok((header, header_at, parent)) = headers.single() else {
        return;
    };
    let Ok((panel, panel_at, mut scroll)) = panels.get_mut(parent.parent()) else {
        return;
    };
    if panel.size.y <= 0.0 || header.size.y <= 0.0 {
        return; // Not laid out yet.
    }
    // In physical pixels: where the header's top lies in the scrolled content, less the panel's padding.
    let top = |node: &ComputedNode, at: &UiGlobalTransform| at.translation.y - node.size.y / 2.0;
    let offset = top(header, header_at) - top(panel, panel_at) + panel.scroll_position.y - panel.padding.min_inset.y;
    let wanted = (offset * panel.inverse_scale_factor).max(0.0);
    if scroll.0.y != wanted {
        scroll.0.y = wanted;
    }
    *due = false;
}

pub fn rebuild_layout_panel(
    mut commands: Commands,
    doc: Res<EditorDoc>,
    tools: Res<EditorTools>,
    problems: Res<EditorProblems>,
    list: Query<Entity, With<LayoutPanel>>,
    mut shown: Local<Option<LayoutView>>,
) {
    if shown.is_some() && !doc.is_changed() && !tools.is_changed() && !problems.is_changed() {
        return;
    }
    let map = doc.map();
    let (slot, faction) = (tools.slot, tools.faction);
    let slots = map
        .slot_ids()
        .filter(|s| map.is_slot_editable(*s))
        .map(|s| {
            let images = map.slot_ids().filter(|i| *i != s && map.source_slot(*i) == s).collect();
            (s, images)
        })
        .collect();
    let wanted = LayoutView {
        slots,
        state: layout_state(map, &problems.0, slot, faction),
        outdated: outdated_text(map),
        palette: palette(map, slot, faction),
    };
    rebuild(&mut commands, &list, &mut shown, wanted, |list, view| {
        list.spawn(row()).with_children(|r| {
            r.spawn(label("Slot", SMALL_VH, DIM_TEXT));
            for (slot, _) in &view.slots {
                r.spawn(small_button(format!("S{}", slot.0), EditorAction::SelectSlot(*slot)));
            }
        });
        let follow: Vec<String> = view
            .slots
            .iter()
            .filter(|(_, images)| !images.is_empty())
            .map(|(slot, images)| {
                let names: Vec<String> = images.iter().map(|i| format!("S{}", i.0)).collect();
                let verb = if names.len() == 1 { "follows" } else { "follow" };
                format!("{} {verb} S{}", names.join(", "), slot.0)
            })
            .collect();
        if !follow.is_empty() {
            list.spawn(label(format!("Generated: {}", follow.join("; ")), SMALL_VH, DIM_TEXT));
        }
        list.spawn(row()).with_children(|r| {
            for &f in Faction::ALL {
                r.spawn(small_button(faction_name(f), EditorAction::SetLayoutFaction(f)));
            }
        });
        let (state, fine) = &view.state;
        list.spawn(label(state.clone(), SMALL_VH, if *fine { OK_TEXT } else { WARN_TEXT }));
        if let Some(outdated) = &view.outdated {
            list.spawn(label(outdated.clone(), SMALL_VH, WARN_TEXT));
        }
        list.spawn(row()).with_children(|r| {
            for entry in &view.palette {
                r.spawn(small_button(entry.label(), EditorAction::SetLayoutKind(entry.kind)));
            }
        });
    });
}
