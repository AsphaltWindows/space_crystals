//! What the tool under the cursor would change, drawn before the click: the brush's tiles and their symmetric
//! images, the corners a height dab moves, the regions a fill covers, the object about to be placed with its
//! images, and red where an edit would be refused.

use std::collections::BTreeSet;

use bevy::prelude::*;
use sc_sim::{Category, MapDef, ObjectKind, SU_PER_GU, Spawn, SpawnAt, Symmetry, TilePos};

use super::shapes::{Overlay, near_map, rect_corners};
use super::{BAD_EDGE, BAD_FILL, BRUSH_EDGE, BRUSH_FILL, CASCADE_DOT, GOOD_FILL, IMAGE_EDGE, TARGET_DOT};
use crate::camera::map_extent_su;
use crate::coords::{drawn_at_height, drawn_position};
use crate::editor::objects::{self, PlaceProblem};
use crate::editor::terrain;
use crate::editor::tools::EditorTools;

const AXIS_COLOR: Color = Color::srgba(1.0, 1.0, 1.0, 0.45);

/// The symmetry's mirror axes, following the surface, or its center of rotation (`core/map_editor.md >
/// Symmetry`). Nothing for a symmetry that doesn't fit the map.
pub(super) fn symmetry_axes(o: &mut Overlay, map: &MapDef) {
    let symmetry = map.symmetry;
    if map.image_count() < 2 {
        return;
    }
    let extent = map_extent_su(o.map);
    let center = extent / 2.0;
    let vertical = matches!(symmetry, Symmetry::MirrorVerticalAxis | Symmetry::MirrorBothAxes);
    let horizontal = matches!(symmetry, Symmetry::MirrorHorizontalAxis | Symmetry::MirrorBothAxes);
    if vertical {
        o.surface_line(
            Vec2::new(center.x, 0.0),
            Vec2::new(center.x, extent.y),
            0.002,
            AXIS_COLOR,
        );
    }
    if horizontal {
        o.surface_line(
            Vec2::new(0.0, center.y),
            Vec2::new(extent.x, center.y),
            0.002,
            AXIS_COLOR,
        );
    }
    let drawn_center = drawn_position(o.map, center);
    match symmetry {
        Symmetry::Rotate180 => o.cross(drawn_center, SU_PER_GU, AXIS_COLOR),
        Symmetry::Rotate90 => {
            o.cross(drawn_center, SU_PER_GU, AXIS_COLOR);
            o.view_outline(Rect::from_center_size(drawn_center, Vec2::splat(SU_PER_GU)), AXIS_COLOR);
        }
        _ => {}
    }
}

/// Marks the tile under the cursor as locked: the tools refuse to edit there.
pub(super) fn locked_cursor(o: &mut Overlay, tile: TilePos) {
    if o.map.in_bounds(tile) {
        o.tile_fill(tile, BAD_FILL);
        o.tiles_outline(&BTreeSet::from([tile]), BAD_EDGE);
    }
}

/// Outlines what an edit of `tiles` changes in the locked part: their images.
fn image_outline(o: &mut Overlay, map: &MapDef, tiles: &BTreeSet<TilePos>) {
    let images: BTreeSet<TilePos> = terrain::tiles_with_images(map, tiles)
        .difference(tiles)
        .copied()
        .collect();
    if !images.is_empty() {
        o.tiles_outline(&images, IMAGE_EDGE);
    }
}

/// The Paint tool's dab under the cursor: its tiles in the editable part, red where the preset would be refused as
/// too steep ([`terrain::refuses`]), and the outline of their images.
pub(super) fn paint_preview(o: &mut Overlay, map: &MapDef, tools: &EditorTools, tile: TilePos) {
    let tiles = terrain::dab_tiles(map, tile, tools.brush);
    for &t in &tiles {
        let changes = o.map.tile(t).is_some_and(|current| current.preset != tools.preset);
        let refused = changes && terrain::refuses(o.map, t, tools.preset);
        o.tile_fill(t, if refused { BAD_FILL } else { BRUSH_FILL });
    }
    o.tiles_outline(&tiles, BRUSH_EDGE);
    image_outline(o, map, &tiles);
}

/// The Height tool's dab under the cursor: its tiles, the outline of their images, and every corner it would
/// move, from where it is drawn now to where it would be: white for the corners the brush sets, amber for the ones
/// the slopes around them move ([`terrain::raise_or_set`] on a copy of the map).
pub(super) fn height_preview(o: &mut Overlay, map: &MapDef, tools: &EditorTools, tile: TilePos) {
    let tiles = terrain::dab_tiles(map, tile, tools.brush);
    for &t in &tiles {
        o.tile_fill(t, BRUSH_FILL);
    }
    o.tiles_outline(&tiles, BRUSH_EDGE);
    image_outline(o, map, &tiles);
    let corners = terrain::dab_corners(map, tile, tools.brush);
    let targets = terrain::corners_with_images(map, &corners);
    let mut after = map.clone();
    let size = o.line * 2.5;
    for c in terrain::raise_or_set(&mut after, &corners, tools.elevation_mode, tools.elevation) {
        let (Some(from), Some(to)) = (map.terrain.corner(c), after.terrain.corner(c)) else {
            continue;
        };
        let (a, b) = (
            drawn_at_height(c.pos(), f32::from(from)),
            drawn_at_height(c.pos(), f32::from(to)),
        );
        let color = if targets.contains(&c) { TARGET_DOT } else { CASCADE_DOT };
        o.segment(a, b, 0.003, color.with_alpha(0.6));
        o.dot(b, size, color);
    }
}

/// The Fill tool at the cursor: the start tile and its images.
pub(super) fn fill_preview(o: &mut Overlay, map: &MapDef, tile: TilePos) {
    for t in terrain::image_tiles(map, tile)
        .into_iter()
        .filter(|t| o.map.in_bounds(*t))
    {
        o.tiles_outline(&BTreeSet::from([t]), BRUSH_EDGE);
    }
}

/// One placement of the Place tool: its footprint on the surface, green where it fits, red where it doesn't, with
/// its sloped tiles marked when it needs flat ground.
pub(super) fn placement_preview(o: &mut Overlay, kind: ObjectKind, at: SpawnAt, problem: Option<PlaceProblem>) {
    let map = o.map;
    let rect = objects::spawn_rect(kind, at);
    if !near_map(map, rect) {
        return;
    }
    let (fill, edge) = match problem {
        None => (GOOD_FILL, BRUSH_EDGE),
        Some(_) => (BAD_FILL, BAD_EDGE),
    };
    let tiles: BTreeSet<TilePos> = objects::footprint_tiles(kind, at).into_iter().collect();
    if tiles.is_empty() {
        // A unit: its silhouette's rectangle.
        let corners = rect_corners(rect).map(|p| drawn_position(map, p));
        o.polygon(&corners, 0.0, fill);
        o.surface_outline(rect, edge);
        return;
    }
    for &t in &tiles {
        o.tile_fill(t, fill);
        if matches!(problem, Some(PlaceProblem::NotFlat(_))) && !map.is_flat(t) {
            o.tile_fill(t, BAD_FILL);
        }
    }
    o.tiles_outline(&tiles, edge);
}

/// Where the Place tool would put the object for the cursor at `pos`, then the images the editor would generate
/// for it, each with why the placement is refused (the same for all of them).
pub(super) fn placements(map: &MapDef, tools: &EditorTools, pos: Vec2) -> Vec<(SpawnAt, Option<PlaceProblem>)> {
    let kind = tools.kind;
    let owner = match kind.def().category {
        Category::Resource => None,
        _ => tools.owner,
    };
    let spawn = Spawn::new(kind, owner, objects::snap_spawn(kind, pos));
    let problem = objects::check_placement(map, &spawn, None).err();
    let mut found = vec![(spawn.at, problem)];
    // A locked spot has no images: the placement is refused.
    if problem != Some(PlaceProblem::Locked) {
        found.extend(map.spawn_images(&spawn).into_iter().map(|s| (s.at, problem)));
    }
    found
}
