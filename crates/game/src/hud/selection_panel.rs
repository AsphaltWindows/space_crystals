//! The SelectionPanel (`systems/controls/selection_and_hud.md > SelectionPanel`): a grid of portraits, one per
//! selected object, shown while at least two objects are selected; the ActiveGroup's portraits are highlighted. A
//! click on a portrait follows `> SelectionPanel > Interactions` ([`crate::selection::ControlState::click_portrait`]).
//! Two or more selected objects are always the player's own (design change 0048).
//!
//! The portraits are arranged group by group, in the order of the selection's groups ([`Selection::groups`]), which
//! is the order Tab and Shift-Tab cycle in, and each group's by id: the placeholder of design change 0055 until the
//! HUD pass. Placeholders until the portraits' art exists (Q0001): a cell shows the type's shape in its owner's
//! colour (round Peacekeeper, square Guard, as on the map) over a short label of its type; an unfinished building's
//! (design change 0185: its own group, after the finished ones of its type) is dimmed as the map dims it while it
//! transforms (`crate::object_render::transforming_color`). When more objects are
//! selected than cells fit, the last cell shows how many more there are, highlighted like the ActiveGroup's portraits
//! when it hides some of them, and does nothing when clicked (the design is silent).

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use sc_sim::{ObjectId, ObjectKind};

mod cells;

use cells::{cell_node, spawn_cell_content};

use super::layout::PortraitGrid;
use super::{HudSection, SectionLabel, show_node};
use crate::ClientMode;
use crate::camera::CameraView;
use crate::input::{ActionState, Bindings, InputAction};
use crate::platform::SurfaceSize;
use crate::selection::{ClickRoles, ControlStates, LocalControl, Selection};
use crate::sim_driver::{LocalPlayer, SimRes};

/// A type's short label, each its own: the kind's ([`crate::kinds`]).
pub use crate::kinds::short_label;

/// What a portrait cell shows.
#[derive(Component, Copy, Clone, Debug, PartialEq, Eq)]
pub enum PortraitCell {
    /// A selected object; `active` while it belongs to the ActiveGroup; `unfinished` while it is under construction
    /// or transformation, in its type's unfinished group (design change 0185).
    Object {
        id: ObjectId,
        kind: ObjectKind,
        unfinished: bool,
        active: bool,
    },
    /// The last cell, when the selection holds more objects than the grid fits: how many more, and whether any of
    /// them belongs to the ActiveGroup (then it is highlighted too, so the ActiveGroup always shows).
    More { count: usize, active: bool },
}

impl PortraitCell {
    /// Whether the cell is highlighted as the ActiveGroup's.
    pub fn is_active(self) -> bool {
        match self {
            PortraitCell::Object { active, .. } | PortraitCell::More { active, .. } => active,
        }
    }
}

/// The portraits shown, and where: what a click on the panel hits.
#[derive(Resource, Default, Debug)]
pub struct SelectionPanelView {
    pub cells: Vec<PortraitCell>,
    /// The surface size the cells were placed for.
    surface: Option<Vec2>,
    entities: Vec<Entity>,
}

/// `> SelectionPanel > Visibility`: the cells for `selection` in a grid of `capacity` cells, group by group. None
/// while fewer than two objects are selected (the InfoPanel shows one).
pub fn portraits(selection: &Selection, capacity: usize) -> Vec<PortraitCell> {
    if selection.len() < 2 || capacity == 0 {
        return Vec::new();
    }
    let active = selection.active();
    let mut cells: Vec<PortraitCell> = selection
        .groups()
        .into_iter()
        .flat_map(|group| {
            let active = active == Some(group.key);
            let (kind, unfinished) = (group.kind(), group.is_unfinished());
            group.instances.into_iter().map(move |id| PortraitCell::Object {
                id,
                kind,
                unfinished,
                active,
            })
        })
        .collect();
    if cells.len() > capacity {
        let hidden = cells.split_off(capacity - 1);
        let active = hidden.iter().any(|cell| cell.is_active());
        cells.push(PortraitCell::More {
            count: hidden.len(),
            active,
        });
    }
    cells
}

/// What the SelectionPanel shows: the local player's selection, in a game mode, on a surface of some size.
#[derive(SystemParam)]
pub(super) struct PanelSource<'w> {
    mode: Res<'w, ClientMode>,
    size: Res<'w, SurfaceSize>,
    sim: Res<'w, SimRes>,
    player: Res<'w, LocalPlayer>,
    states: Res<'w, ControlStates>,
}

/// Rebuilds the portraits when the selection, its ActiveGroup or the surface size changes.
pub(super) fn update_selection_panel(
    mut commands: Commands,
    source: PanelSource,
    mut view: ResMut<SelectionPanelView>,
    sections: Query<(Entity, &HudSection)>,
    mut labels: Query<(&SectionLabel, &mut Node)>,
) {
    let surface = source.size.logical();
    let grid = PortraitGrid::on(surface);
    let cells = if source.mode.is_game() {
        portraits(source.states.selection(source.player.0), grid.capacity())
    } else {
        Vec::new()
    };
    if cells == view.cells && view.surface == Some(surface) {
        return;
    }
    let Some((panel, _)) = sections.iter().find(|(_, s)| **s == HudSection::SelectionPanel) else {
        return;
    };
    for entity in view.entities.drain(..) {
        commands.entity(entity).despawn();
    }
    for (label, mut node) in &mut labels {
        if label.0 == HudSection::SelectionPanel {
            show_node(&mut node, cells.is_empty());
        }
    }
    // Absolute children are placed from the section's padding box: inside its border.
    let section = super::layout::section_rect(HudSection::SelectionPanel, surface);
    let origin = section.min + Vec2::splat(super::layout::SECTION_BORDER_PX);
    for (index, cell) in cells.iter().enumerate() {
        let rect = grid.cell_rect(index);
        let entity = commands
            .spawn((
                Name::new("Portrait"),
                *cell,
                cell_node(rect.min - origin, rect.width(), cell.is_active()),
                ChildOf(panel),
            ))
            .id();
        spawn_cell_content(&mut commands, entity, *cell, &source.sim);
        view.entities.push(entity);
    }
    view.cells = cells;
    view.surface = Some(surface);
}

/// `> SelectionPanel > Interactions`: a press of [`InputAction::Select`] on a portrait, with the roles its
/// modifiers hold. The portrait under the cursor comes from the layout ([`PortraitGrid`]), not from UI picking.
pub(super) fn click_portraits(
    actions: Res<ActionState>,
    bindings: Res<Bindings>,
    mode: Res<ClientMode>,
    size: Res<SurfaceSize>,
    view: Res<SelectionPanelView>,
    mut control: LocalControl,
    mut camera: ResMut<CameraView>,
) {
    if !mode.is_game() || !actions.just_pressed(InputAction::Select) || !actions.pointer().over_ui {
        return;
    }
    let Some(cursor) = actions.cursor() else {
        return;
    };
    let grid = PortraitGrid::on(size.logical());
    let Some(PortraitCell::Object { id, .. }) = grid.index_at(cursor).and_then(|i| view.cells.get(i).copied()) else {
        return;
    };
    let roles = ClickRoles::of(&bindings, actions.modifiers());
    let objects = control.objects();
    if let Some(center) = control.update(|state| state.click_portrait(&objects, id, roles)) {
        camera.center = center;
    }
}

#[cfg(test)]
mod tests;
