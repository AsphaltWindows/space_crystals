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
use sc_sim::tunnel::TunnelState;
use sc_sim::{ObjectId, ObjectKind};

use super::layout::PortraitGrid;
use super::{HudSection, SectionLabel, show_node, text};
use crate::ClientMode;
use crate::camera::CameraView;
use crate::input::{ActionState, Bindings, InputAction};
use crate::object_render::{player_color, transforming_color};
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

/// The short label of a type (placeholders until the art, Q0001): on its portrait, on the map (`object_render`'s
/// tags) and on the button that makes it. Each its own (slice M3-J3): none is another type's, nor a planned unit's or
/// building's button label (`Planned::label`: the Scout's had been the patch's "SC"). The Peacekeeper's two types
/// have two letters each, "PK" and "PS", short enough for the Peacekeeper's 24-su body.
pub fn short_label(kind: ObjectKind) -> &'static str {
    match kind {
        ObjectKind::CrystalPatch => "SC",
        ObjectKind::SupplyDeliveryStation => "SDS",
        ObjectKind::Peacekeeper => "PK",
        ObjectKind::Guard => "GD",
        ObjectKind::DeploymentCenter => "DC",
        ObjectKind::Generator => "GN",
        ObjectKind::Barracks => "BR",
        ObjectKind::ExtractionFacility => "EF",
        ObjectKind::ExtractionPlate => "EP",
        ObjectKind::Drone => "DR",
        ObjectKind::Beacon => "BN",
        ObjectKind::EgressNode => "EN",
        ObjectKind::RoboPortal => "RP",
        ObjectKind::BioPortal => "BP",
        ObjectKind::SabreLegionnaire => "SL",
        ObjectKind::RifleLegionnaire => "RL",
        ObjectKind::PeacekeeperShielded => "PS",
        ObjectKind::Agent => "AG",
        // A Tunnel shows its tier ([`object_label`]): its type's label is Tier 1's, the tier it starts at.
        ObjectKind::Tunnel => TIER_LABELS[0],
        ObjectKind::Hideout => "HI",
    }
}

/// A Tunnel's label by tier (slice M4-A, `factions/syndicate/tunnels.md > Tunnel > Tiers`): "T1" to "T3".
const TIER_LABELS: [&str; 3] = ["T1", "T2", "T3"];

/// The label of a Tunnel at `tier` ([`TIER_LABELS`]); the type's own for a tier the table hasn't.
pub fn tier_label(tier: u8) -> &'static str {
    let index = usize::from(tier).checked_sub(1);
    index
        .and_then(|i| TIER_LABELS.get(i))
        .copied()
        .unwrap_or(TIER_LABELS[0])
}

/// The label an object shows, on the map (a record's too) and on its portrait: its type's short label
/// ([`short_label`]), a Tunnel's its tier's ([`tier_label`], from `Object::tunnel` or the record's), to every player
/// who sees it (design change 0258: "Enemies see a Tunnel's tier").
pub fn object_label(kind: ObjectKind, tunnel: Option<TunnelState>) -> &'static str {
    match tunnel {
        Some(state) => tier_label(state.tier),
        None => short_label(kind),
    }
}

/// Whether a type's portrait shape is round (as its placeholder on the map is).
fn round(kind: ObjectKind) -> bool {
    matches!(kind, ObjectKind::Peacekeeper | ObjectKind::PeacekeeperShielded)
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
        PortraitCell::Object {
            id, kind, unfinished, ..
        } => {
            let object = sim.object(id);
            let owner = player_color(object.and_then(|o| o.owner));
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
                BackgroundColor(if unfinished { transforming_color(owner) } else { owner }),
                ChildOf(cell),
            ));
            let label = object_label(kind, object.and_then(|o| o.tunnel.clone()));
            commands.spawn((text(label, CELL_LABEL_VH, CELL_TEXT), ChildOf(cell)));
        }
        PortraitCell::More { count, .. } => {
            commands.spawn((text(format!("+{count}"), CELL_LABEL_VH * 1.4, CELL_TEXT), ChildOf(cell)));
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

    const PK: ObjectKind = ObjectKind::Peacekeeper;
    const GUARD: ObjectKind = ObjectKind::Guard;

    /// The objects of the highlighted portraits, in the panel's order.
    fn highlighted(selection: &Selection) -> Vec<u32> {
        portraits(selection, 10)
            .into_iter()
            .filter_map(|cell| match cell {
                PortraitCell::Object { id, active: true, .. } => Some(id.0),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn portraits_need_two_objects_and_end_in_a_count_when_full() {
        let selection = |n: u32| Selection::new((1..=n).map(|i| (ObjectId(i), PK)));
        assert!(portraits(&selection(1), 10).is_empty());
        let two = portraits(&selection(2), 10);
        assert_eq!(
            two,
            [1, 2].map(|i| PortraitCell::Object {
                id: ObjectId(i),
                kind: PK,
                unfinished: false,
                active: true
            })
        );
        let full = portraits(&selection(12), 10);
        assert_eq!(full.len(), 10);
        assert_eq!(full[9], PortraitCell::More { count: 3, active: true });
        let exactly = portraits(&selection(10), 10);
        assert_eq!(exactly.len(), 10, "exactly full shows them all");
        assert!(exactly.iter().all(|cell| matches!(cell, PortraitCell::Object { .. })));
    }

    #[test]
    fn only_the_active_groups_portraits_are_highlighted() {
        let mut selection = Selection::new([(ObjectId(1), PK), (ObjectId(2), GUARD), (ObjectId(3), PK)]);
        selection.cycle(true);
        let active: Vec<(ObjectId, bool)> = portraits(&selection, 10)
            .into_iter()
            .map(|c| match c {
                PortraitCell::Object { id, active, .. } => (id, active),
                PortraitCell::More { .. } => unreachable!(),
            })
            .collect();
        assert_eq!(
            active,
            [(ObjectId(1), false), (ObjectId(3), false), (ObjectId(2), true)]
        );
    }

    /// `> SelectionPanel` (design change 0055): the portraits come group by group, and Tab and Shift-Tab make the
    /// groups active in that order, wrapping around. Patches stand in for the Ungroupable own structures that don't
    /// exist yet (a resource is never selected with other objects, 0048).
    #[test]
    fn tab_and_shift_tab_follow_the_order_of_the_portraits() {
        let patch = ObjectKind::CrystalPatch;
        let mut selection = Selection::new([
            (ObjectId(1), GUARD),
            (ObjectId(2), PK),
            (ObjectId(3), patch),
            (ObjectId(4), PK),
            (ObjectId(5), patch),
        ]);
        let order: Vec<u32> = portraits(&selection, 10)
            .into_iter()
            .map(|cell| match cell {
                PortraitCell::Object { id, .. } => id.0,
                PortraitCell::More { .. } => unreachable!(),
            })
            .collect();
        assert_eq!(
            order,
            [3, 5, 2, 4, 1],
            "each patch alone, then the Peacekeepers, then the Guard"
        );
        assert_eq!(highlighted(&selection), [3], "the first group is active");
        let mut tab = |forward: bool| {
            selection.cycle(forward);
            highlighted(&selection)
        };
        let forward: Vec<Vec<u32>> = (0..4).map(|_| tab(true)).collect();
        assert_eq!(forward, [vec![5], vec![2, 4], vec![1], vec![3]]);
        let backward: Vec<Vec<u32>> = (0..4).map(|_| tab(false)).collect();
        assert_eq!(backward, [vec![1], vec![2, 4], vec![5], vec![3]]);
    }

    /// The count of the portraits that don't fit is highlighted when it hides some of the ActiveGroup's.
    #[test]
    fn the_count_of_hidden_portraits_is_highlighted_when_it_hides_the_active_group() {
        let pks = (1..=9).map(|i| (ObjectId(i), PK));
        let mut selection = Selection::new(pks.chain((10..=12).map(|i| (ObjectId(i), GUARD))));
        let cells = portraits(&selection, 10);
        assert!(cells[..9].iter().all(|cell| cell.is_active()));
        assert_eq!(
            cells[9],
            PortraitCell::More {
                count: 3,
                active: false
            }
        );
        selection.cycle(true);
        let cells = portraits(&selection, 10);
        assert!(cells[..9].iter().all(|cell| !cell.is_active()));
        assert_eq!(
            cells[9],
            PortraitCell::More { count: 3, active: true },
            "the Guards are all hidden"
        );
    }

    /// Design change 0185: 2 finished Bio Portals and 1 warping in, two groups; the unfinished one's portrait comes
    /// after the finished ones' and is marked, and Tab highlights it alone.
    #[test]
    fn an_unfinished_buildings_portrait_comes_in_a_group_of_its_own() {
        let bio = ObjectKind::BioPortal;
        let unfinished = crate::selection::GroupType::unfinished(bio);
        let mut selection = Selection::new([
            (ObjectId(3), unfinished),
            (ObjectId(4), bio.into()),
            (ObjectId(5), bio.into()),
        ]);
        let cells = |selection: &Selection| -> Vec<(u32, bool, bool)> {
            portraits(selection, 10)
                .into_iter()
                .map(|cell| match cell {
                    PortraitCell::Object {
                        id, unfinished, active, ..
                    } => (id.0, unfinished, active),
                    PortraitCell::More { .. } => unreachable!(),
                })
                .collect()
        };
        assert_eq!(
            cells(&selection),
            [(4, false, true), (5, false, true), (3, true, false)]
        );
        selection.cycle(true);
        assert_eq!(
            cells(&selection),
            [(4, false, false), (5, false, false), (3, true, true)]
        );
    }

    /// Slice M3-J3 (M3-K2's walkthrough): every type's short label is its own, and none is a planned unit's or
    /// building's button label ("SC" was both the crystal patch's and the Scout button's); the Peacekeeper's two
    /// types have two-letter labels of their own. Slice M4-A: the Agent's AG, the Hideout's HI, and a Tunnel's by
    /// tier, T1 to T3, T1 its type's.
    #[test]
    fn every_label_is_its_own() {
        use crate::command_ui::interface::templates::Planned;
        let labels: Vec<&str> = (ObjectKind::ALL.iter().map(|&kind| short_label(kind)))
            .chain([2, 3].map(tier_label))
            .chain(Planned::ALL.map(Planned::label))
            .collect();
        let syndicate = [ObjectKind::Agent, ObjectKind::Tunnel, ObjectKind::Hideout].map(short_label);
        assert_eq!(syndicate, ["AG", "T1", "HI"]);
        assert_eq!([0, 1, 2, 3, 4].map(tier_label), ["T1", "T1", "T2", "T3", "T1"]);
        let tier = |tier| object_label(ObjectKind::Tunnel, Some(TunnelState::at(tier)));
        assert_eq!((tier(3), object_label(ObjectKind::Agent, None)), ("T3", "AG"));
        let distinct: std::collections::BTreeSet<&str> = labels.iter().copied().collect();
        assert_eq!(distinct.len(), labels.len(), "{labels:?}");
        assert_eq!([PK, ObjectKind::PeacekeeperShielded].map(short_label), ["PK", "PS"]);
        assert_eq!(
            (short_label(ObjectKind::CrystalPatch), Planned::Scout.label()),
            ("SC", "ST")
        );
    }
}
