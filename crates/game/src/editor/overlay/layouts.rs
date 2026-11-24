//! The Starting Layouts in the overlay (slice M2-I, `core/map_editor.md > Starting Layouts`): with the Layout tool,
//! the other factions' layouts of the edited start position dimmed under the edited one (`> Overlay editing`: "all
//! factions' starting layouts at the same start position simultaneously, overlaid on each other"; the sim shows the
//! edited faction's own, `crate::editor::layouts::preview_scenario`), the edited layout's objects outlined in the
//! slot's colour, the build area of its Deployment Center (`> Layout validation`: the Extraction Facility and plates
//! "must lie within the build area of the Deployment Center placed in the same layout"), and the object about to be
//! placed, green where it may go and red on the tiles that refuse it. With every tool, the layout objects that break
//! a rule are outlined in red and a selected one in yellow.

use std::collections::BTreeSet;

use bevy::prelude::*;
use sc_sim::placement::BuildArea;
use sc_sim::{
    Faction, LayoutObject, MapDef, Placement, ScenarioProblem, SlotId, SpawnAt, TilePos, placement::footprint_size,
};

use super::shapes::{Overlay, near_map, rect_corners};
use super::{BAD_EDGE, BAD_FILL, BRUSH_EDGE, GOOD_FILL, IMAGE_EDGE, slot_color};
use crate::coords::drawn_position;
use crate::editor::layouts::check::LayoutGhost;
use crate::editor::layouts::{layout, object_rect};
use crate::editor::objects;
use crate::editor::state::Selection;
use crate::editor::tools::EditorTools;

/// The other factions' layouts at the edited start position (placeholder colours).
const DIM_FILL: Color = Color::srgba(0.85, 0.85, 0.9, 0.16);
const DIM_EDGE: Color = Color::srgba(0.85, 0.85, 0.9, 0.45);
/// The edited layout's build area.
const AREA_EDGE: Color = Color::srgba(0.55, 0.85, 1.0, 0.6);

/// Fills where `object` lies: its footprint's tiles, or a unit's silhouette, and outlines it.
fn object_shape(o: &mut Overlay, object: &LayoutObject, fill: Color, edge: Color) {
    let rect = object_rect(object);
    if !near_map(o.map, rect) {
        return;
    }
    let tiles: BTreeSet<TilePos> = objects::footprint_tiles(object.kind, object.at, object.orientation)
        .into_iter()
        .filter(|t| o.map.in_bounds(*t))
        .collect();
    if tiles.is_empty() {
        let corners = rect_corners(rect).map(|p| drawn_position(o.map, p));
        o.polygon(&corners, 0.0, fill);
    } else {
        for &t in &tiles {
            o.tile_fill(t, fill);
        }
    }
    o.surface_outline(rect, edge);
}

/// Where a structure of a layout stands, as the build area takes it.
fn placement(object: &LayoutObject) -> Option<Placement> {
    let (SpawnAt::Tile(origin), Some((w, h))) = (object.at, footprint_size(object.kind, object.orientation)) else {
        return None;
    };
    let o = object.orientation;
    Some(Placement {
        origin,
        w,
        h,
        rotation: o.rotation,
        flip_h: o.flip_h,
        flip_v: o.flip_v,
    })
}

/// The Layout tool's overlay: the other factions' layouts at the edited slot and its images dimmed, the edited
/// layout outlined (its images fainter), its build area, and the ghost's footprints.
pub(super) fn layout_tool(o: &mut Overlay, map: &MapDef, tools: &EditorTools, ghost: Option<&LayoutGhost>) {
    let (slot, faction) = (tools.slot, tools.faction);
    let set: Vec<SlotId> = map.slot_ids().filter(|s| map.source_slot(*s) == slot).collect();
    for &s in &set {
        for other in Faction::ALL.iter().copied().filter(|f| *f != faction) {
            for object in layout(map, s, other) {
                object_shape(o, object, DIM_FILL, DIM_EDGE);
            }
        }
        let edge = if s == slot { slot_color(slot) } else { IMAGE_EDGE };
        for object in layout(map, s, faction) {
            let rect = object_rect(object);
            if near_map(o.map, rect) {
                o.surface_outline(rect, edge);
            }
        }
    }
    let edited = layout(map, slot, faction);
    let placements: Vec<(sc_sim::ObjectKind, Placement)> =
        edited.iter().filter_map(|x| Some((x.kind, placement(x)?))).collect();
    let area = BuildArea::of_layout(&map.terrain, placements.iter().map(|(k, p)| (*k, p)));
    if !area.is_empty() {
        let tiles: BTreeSet<TilePos> = area.tiles().collect();
        o.tiles_outline(&tiles, AREA_EDGE);
    }
    let Some(ghost) = ghost else {
        return;
    };
    let rect = objects::spawn_rect(ghost.kind, ghost.at, ghost.orientation);
    if !near_map(o.map, rect) {
        return;
    }
    let (marked, whole) = match &ghost.verdict {
        Ok(()) => (BTreeSet::new(), false),
        Err(refusal) => refusal.marked_tiles(),
    };
    let edge = if ghost.verdict.is_ok() { BRUSH_EDGE } else { BAD_EDGE };
    let tiles: BTreeSet<TilePos> = objects::footprint_tiles(ghost.kind, ghost.at, ghost.orientation)
        .into_iter()
        .collect();
    if tiles.is_empty() {
        let fill = if ghost.verdict.is_ok() { GOOD_FILL } else { BAD_FILL };
        let corners = rect_corners(rect).map(|p| drawn_position(o.map, p));
        o.polygon(&corners, 0.0, fill);
    }
    for &t in tiles.iter().filter(|t| o.map.in_bounds(**t)) {
        let bad = whole || marked.contains(&t);
        o.tile_fill(t, if bad { BAD_FILL } else { GOOD_FILL });
    }
    o.surface_outline(rect, edge);
    for &(at, orientation) in &ghost.images {
        let rect = objects::spawn_rect(ghost.kind, at, orientation);
        if near_map(o.map, rect) {
            o.surface_outline(rect, IMAGE_EDGE);
        }
    }
}

/// The layout object a problem is about, if any: its slot, faction and index.
fn problem_object(problem: &ScenarioProblem) -> Option<(SlotId, Faction, usize)> {
    match *problem {
        ScenarioProblem::InvalidLayoutObject {
            slot, faction, index, ..
        }
        | ScenarioProblem::LayoutPlacement {
            slot, faction, index, ..
        }
        | ScenarioProblem::LayoutExtra {
            slot, faction, index, ..
        }
        | ScenarioProblem::LayoutOverlap {
            slot, faction, index, ..
        } => Some((slot, faction, index)),
        _ => None,
    }
}

/// Outlines in red the layout objects that break a rule.
pub(super) fn problems(o: &mut Overlay, map: &MapDef, problems: &[ScenarioProblem]) {
    for (slot, faction, index) in problems.iter().filter_map(problem_object) {
        if let Some(object) = layout(map, slot, faction).get(index) {
            let rect = object_rect(object);
            if near_map(o.map, rect) {
                o.surface_outline(rect, BAD_EDGE);
            }
        }
    }
}

/// The selected layout object's area, if a layout object is selected.
pub(super) fn selected(map: &MapDef, selection: Option<Selection>) -> Option<Rect> {
    let Some(Selection::LayoutObject { slot, faction, index }) = selection else {
        return None;
    };
    layout(map, slot, faction).get(index).map(object_rect)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sc_sim::{BuildingOrientation, ObjectKind};

    /// A layout's structure stands where it is placed, as turned; a unit has no tile placement.
    #[test]
    fn a_layout_structure_stands_as_placed() {
        let dc = LayoutObject {
            kind: ObjectKind::DeploymentCenter,
            at: SpawnAt::Tile(TilePos::new(3, 4)),
            orientation: BuildingOrientation::default(),
        };
        let p = placement(&dc).unwrap();
        assert_eq!((p.origin, p.w, p.h), (TilePos::new(3, 4), 4, 4));
        let unit = LayoutObject {
            kind: ObjectKind::Peacekeeper,
            at: SpawnAt::Point(Vec2::splat(100.0)),
            orientation: BuildingOrientation::default(),
        };
        assert_eq!(placement(&unit), None);
    }
}
