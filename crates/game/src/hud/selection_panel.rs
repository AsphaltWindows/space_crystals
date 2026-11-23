//! The SelectionPanel (`systems/controls/selection_and_hud.md > SelectionPanel`): a grid of portraits, one per
//! selected object, shown while at least two objects are selected; the ActiveGroup's portraits are highlighted. A
//! click on a portrait follows `> SelectionPanel > Interactions` ([`crate::selection::ControlState::click_portrait`]).
//!
//! Placeholders until the portraits' art exists (Q0001): a cell shows the type's shape in its owner's colour (round
//! Peacekeeper, square Guard, as on the map) over a short label of its type. The portraits follow the selection's
//! groups in order, each group's by id (the design gives no order). When more objects are selected than cells fit,
//! the last cell shows how many more there are and does nothing when clicked (the design is silent).

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use sc_sim::{ObjectId, ObjectKind};

use super::layout::PortraitGrid;
use super::{HudSection, SectionLabel, show_node, text};
use crate::ClientMode;
use crate::camera::CameraView;
use crate::input::{ActionState, Bindings, InputAction};
use crate::object_render::player_color;
use crate::platform::SurfaceSize;
use crate::selection::{ClickRoles, ControlStates, LocalControl, Selection};
use crate::sim_driver::{LocalPlayer, SimRes};

const CELL_BACKGROUND: Color = Color::srgb(0.16, 0.18, 0.22);
const CELL_BORDER: Color = Color::srgb(0.30, 0.33, 0.38);
/// The ActiveGroup's "sheer highlight" (placeholder colours).
const ACTIVE_BACKGROUND: Color = Color::srgb(0.22, 0.30, 0.26);
const ACTIVE_BORDER: Color = Color::srgb(0.55, 0.95, 0.60);
const CELL_TEXT: Color = Color::srgb(0.88, 0.90, 0.93);
/// A cell's label size, and its shape's share of the cell (placeholders).
const CELL_LABEL_VH: f32 = 1.3;
const SHAPE_PERCENT: f32 = 48.0;

/// What a portrait cell shows.
#[derive(Component, Copy, Clone, Debug, PartialEq, Eq)]
pub enum PortraitCell {
    /// A selected object; `active` while it belongs to the ActiveGroup.
    Object {
        id: ObjectId,
        kind: ObjectKind,
        active: bool,
    },
    /// The last cell, when the selection holds more objects than the grid fits: how many more.
    More(usize),
}

/// The portraits shown, and where: what a click on the panel hits.
#[derive(Resource, Default, Debug)]
pub struct SelectionPanelView {
    pub cells: Vec<PortraitCell>,
    /// The surface size the cells were placed for.
    surface: Option<Vec2>,
    entities: Vec<Entity>,
}

/// `> SelectionPanel > Visibility`: the cells for `selection` in a grid of `capacity` cells. None while fewer than
/// two objects are selected (the InfoPanel shows one).
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
            let kind = group.kind();
            group
                .instances
                .into_iter()
                .map(move |id| PortraitCell::Object { id, kind, active })
        })
        .collect();
    if cells.len() > capacity {
        let shown = capacity - 1;
        let more = cells.len() - shown;
        cells.truncate(shown);
        cells.push(PortraitCell::More(more));
    }
    cells
}

/// The short label of a type on its portrait (placeholder).
pub fn short_label(kind: ObjectKind) -> &'static str {
    match kind {
        ObjectKind::CrystalPatch => "SC",
        ObjectKind::SupplyDeliveryStation => "SDS",
        ObjectKind::Peacekeeper => "PK",
        ObjectKind::Guard => "GD",
    }
}

/// Whether a type's portrait shape is round (as its placeholder on the map is).
fn round(kind: ObjectKind) -> bool {
    matches!(kind, ObjectKind::Peacekeeper)
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
                cell_node(
                    rect.min - origin,
                    rect.width(),
                    matches!(cell, PortraitCell::Object { active: true, .. }),
                ),
                ChildOf(panel),
            ))
            .id();
        spawn_cell_content(&mut commands, entity, *cell, &source.sim);
        view.entities.push(entity);
    }
    view.cells = cells;
    view.surface = Some(surface);
}

/// A cell's node, background and border.
fn cell_node(at: Vec2, side: f32, active: bool) -> impl Bundle {
    (
        Node {
            position_type: PositionType::Absolute,
            left: px(at.x),
            top: px(at.y),
            width: px(side),
            height: px(side),
            border: UiRect::all(px(if active { 2.0 } else { 1.0 })),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            row_gap: px(side * 0.04),
            ..default()
        },
        BackgroundColor(if active { ACTIVE_BACKGROUND } else { CELL_BACKGROUND }),
        BorderColor::all(if active { ACTIVE_BORDER } else { CELL_BORDER }),
    )
}

/// A cell's shape and label.
fn spawn_cell_content(commands: &mut Commands, cell: Entity, shown: PortraitCell, sim: &SimRes) {
    match shown {
        PortraitCell::Object { id, kind, .. } => {
            let owner = sim.object(id).and_then(|o| o.owner);
            commands.spawn((
                Node {
                    width: percent(SHAPE_PERCENT),
                    height: percent(SHAPE_PERCENT),
                    border_radius: if round(kind) {
                        BorderRadius::MAX
                    } else {
                        BorderRadius::ZERO
                    },
                    ..default()
                },
                BackgroundColor(player_color(owner)),
                ChildOf(cell),
            ));
            commands.spawn((text(short_label(kind), CELL_LABEL_VH, CELL_TEXT), ChildOf(cell)));
        }
        PortraitCell::More(more) => {
            commands.spawn((text(format!("+{more}"), CELL_LABEL_VH * 1.4, CELL_TEXT), ChildOf(cell)));
        }
    }
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
mod tests {
    use super::*;

    #[test]
    fn portraits_need_two_objects_and_end_in_a_count_when_full() {
        let selection = |n: u32| Selection::new((1..=n).map(|i| (ObjectId(i), ObjectKind::Peacekeeper)));
        assert!(portraits(&selection(1), 10).is_empty());
        let two = portraits(&selection(2), 10);
        assert_eq!(
            two,
            [1, 2].map(|i| PortraitCell::Object {
                id: ObjectId(i),
                kind: ObjectKind::Peacekeeper,
                active: true
            })
        );
        let full = portraits(&selection(12), 10);
        assert_eq!(full.len(), 10);
        assert_eq!(full[9], PortraitCell::More(3));
        assert_eq!(portraits(&selection(10), 10).len(), 10, "exactly full shows them all");
        assert!(!portraits(&selection(10), 10).contains(&PortraitCell::More(1)));
    }

    #[test]
    fn only_the_active_groups_portraits_are_highlighted() {
        let mut selection = Selection::new([
            (ObjectId(1), ObjectKind::Peacekeeper),
            (ObjectId(2), ObjectKind::Guard),
            (ObjectId(3), ObjectKind::Peacekeeper),
        ]);
        selection.cycle(true);
        let active: Vec<(ObjectId, bool)> = portraits(&selection, 10)
            .into_iter()
            .map(|c| match c {
                PortraitCell::Object { id, active, .. } => (id, active),
                PortraitCell::More(_) => unreachable!(),
            })
            .collect();
        assert_eq!(
            active,
            [(ObjectId(1), false), (ObjectId(3), false), (ObjectId(2), true)]
        );
    }
}
