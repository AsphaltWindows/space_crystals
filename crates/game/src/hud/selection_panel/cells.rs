//! A portrait cell's look (placeholders until the portraits' art exists, Q0001): the type's shape in its owner's
//! colour, round as its placeholder on the map is or square, over its label; dimmed while it transforms.

use bevy::prelude::*;
use sc_sim::ObjectKind;

use super::PortraitCell;
use crate::hud::text;
use crate::kinds::object_label;
use crate::object_render::{Look, UnitShape, player_color, transforming_color};
use crate::sim_driver::SimRes;

const CELL_BACKGROUND: Color = Color::srgb(0.16, 0.18, 0.22);
const CELL_BORDER: Color = Color::srgb(0.30, 0.33, 0.38);
/// The ActiveGroup's "sheer highlight" (placeholder colours).
const ACTIVE_BACKGROUND: Color = Color::srgb(0.22, 0.30, 0.26);
const ACTIVE_BORDER: Color = Color::srgb(0.55, 0.95, 0.60);
const CELL_TEXT: Color = Color::srgb(0.88, 0.90, 0.93);
/// A cell's label size, and its shape's share of the cell (placeholders).
const CELL_LABEL_VH: f32 = 1.3;
const SHAPE_PERCENT: f32 = 48.0;

/// Whether a type's portrait shape is round, as its placeholder on the map is ([`Look`]).
fn round(kind: ObjectKind) -> bool {
    Look::of(kind).unit_shape() == Some(UnitShape::Round)
}

/// A cell's node, background and border.
pub(super) fn cell_node(at: Vec2, side: f32, active: bool) -> impl Bundle {
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
pub(super) fn spawn_cell_content(commands: &mut Commands, cell: Entity, shown: PortraitCell, sim: &SimRes) {
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
