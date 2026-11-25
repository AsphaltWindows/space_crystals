//! The object edits behind the Place and Select tools, as plain functions on sim types, testable without Bevy:
//! snapping, footprints, placement checks, hit tests, where validation problems are, and the object palette.

mod palette;
mod placement;
#[cfg(test)]
mod tests;

use bevy::math::{Rect, Vec2};
use sc_sim::{
    BuildingOrientation, Category, Footprint, Map, MapDef, ObjectKind, SU_PER_GU, ScenarioProblem, SlotId, Spawn,
    SpawnAt, TilePos,
};

use crate::camera::map_extent_su;

pub use palette::{PaletteGroup, faction_name, palette_groups};
pub use placement::{PlaceProblem, check_placement, overlaps_own_image};

/// A tile-based object type's footprint in tiles, unturned, or `None` for units.
pub fn tile_size(kind: ObjectKind) -> Option<(u32, u32)> {
    match kind.def().footprint {
        Footprint::Tiles { w, h } => Some((w, h)),
        Footprint::Silhouette { .. } => None,
    }
}

/// A tile-based object type's footprint in tiles turned by `orientation` (`sc_sim::placement::footprint_size`: a
/// quarter turn swaps its sides), or `None` for units.
pub fn oriented_size(kind: ObjectKind, orientation: BuildingOrientation) -> Option<(u32, u32)> {
    sc_sim::placement::footprint_size(kind, orientation)
}

/// Where an object of `kind` turned by `orientation` goes for the cursor at `pos`: a tile-based object's footprint
/// centres on the grid point nearest to `pos` (for a 1×1 footprint, the tile under it); a unit takes the point
/// itself.
pub fn snap_spawn(kind: ObjectKind, orientation: BuildingOrientation, pos: Vec2) -> SpawnAt {
    match oriented_size(kind, orientation) {
        Some((w, h)) => {
            let origin = |p: f32, n: u32| (p / SU_PER_GU - n as f32 / 2.0 + 0.5).floor() as i32;
            SpawnAt::Tile(TilePos::new(origin(pos.x, w), origin(pos.y, h)))
        }
        None => SpawnAt::Point(pos),
    }
}

/// Where a dragged object turned by `orientation` goes when its center is dragged to `center`: snapped like a
/// placement and kept entirely on the map.
pub fn dragged_spawn(map: &Map, kind: ObjectKind, orientation: BuildingOrientation, center: Vec2) -> SpawnAt {
    match snap_spawn(kind, orientation, center) {
        SpawnAt::Tile(o) => {
            let (w, h) = oriented_size(kind, orientation).unwrap_or((1, 1));
            let keep = |v: i32, n: u32, size: u32| v.min(size as i32 - n as i32).max(0);
            SpawnAt::Tile(TilePos::new(keep(o.x, w, map.width()), keep(o.y, h, map.height())))
        }
        SpawnAt::Point(p) => SpawnAt::Point(clamp_to_map(map, p)),
    }
}

/// The nearest point on the map: inside its last row and column, not on their far edges.
pub fn clamp_to_map(map: &Map, p: Vec2) -> Vec2 {
    let p = if p.is_finite() { p } else { Vec2::ZERO };
    p.clamp(Vec2::ZERO, map_extent_su(map) - Vec2::splat(0.01))
}

/// The area an object of `kind` at `at`, turned by `orientation`, covers in sim space units: its tile footprint, or
/// its silhouette around its point.
pub fn spawn_rect(kind: ObjectKind, at: SpawnAt, orientation: BuildingOrientation) -> Rect {
    match (oriented_size(kind, orientation), kind.def().footprint, at) {
        (Some((w, h)), _, SpawnAt::Tile(o)) => {
            Rect::from_corners(o.corner(), o.corner() + Vec2::new(w as f32, h as f32) * SU_PER_GU)
        }
        (None, Footprint::Silhouette { w, h }, SpawnAt::Point(p)) => Rect::from_center_size(p, Vec2::new(w, h)),
        // The wrong kind of location (an invalid spawn): one tile, or a half-tile square around the point.
        (_, _, SpawnAt::Tile(o)) => Rect::from_corners(o.corner(), o.corner() + Vec2::splat(SU_PER_GU)),
        (_, _, SpawnAt::Point(p)) => Rect::from_center_size(p, Vec2::splat(SU_PER_GU / 2.0)),
    }
}

/// The tiles a tile-based object of `kind` at `at`, turned by `orientation`, covers; none for units. A spawn of a
/// damaged file may lie anywhere, even at the ends of the `i32` range: tiles past them are left out instead of
/// overflowing.
pub fn footprint_tiles(kind: ObjectKind, at: SpawnAt, orientation: BuildingOrientation) -> Vec<TilePos> {
    let (Some((w, h)), SpawnAt::Tile(o)) = (oriented_size(kind, orientation), at) else {
        return Vec::new();
    };
    tiles_from(o, w, h)
}

/// The tiles a spawn covers, its building orientation included ([`footprint_tiles`]).
pub fn spawn_tiles(spawn: &Spawn) -> Vec<TilePos> {
    footprint_tiles(spawn.kind, spawn.at, spawn.orientation)
}

fn tiles_from(o: TilePos, w: u32, h: u32) -> Vec<TilePos> {
    (0..h as i32)
        .flat_map(|dy| (0..w as i32).map(move |dx| (dx, dy)))
        .filter_map(|(dx, dy)| Some(TilePos::new(o.x.checked_add(dx)?, o.y.checked_add(dy)?)))
        .collect()
}

/// The spawn drawn at `pos`: of those whose area contains it, the frontmost one (the southernmost, as southern
/// objects draw in front, then one that isn't a map resource, as a plate covers its patch, then the latest).
pub fn spawn_at_point(map: &MapDef, pos: Vec2) -> Option<usize> {
    let on_top = |s: &Spawn| s.kind.def().category != Category::Resource;
    map.spawns
        .iter()
        .enumerate()
        .map(|(i, s)| (i, spawn_rect(s.kind, s.at, s.orientation), on_top(s)))
        .filter(|(_, r, _)| r.contains(pos))
        .max_by(|(i, a, a_top), (j, b, b_top)| {
            a.center()
                .y
                .total_cmp(&b.center().y)
                .then(a_top.cmp(b_top))
                .then(i.cmp(j))
        })
        .map(|(i, ..)| i)
}

/// What a click on a problem in the validation list centres the camera on, and the spawn it concerns: a layout
/// object's problem centres on the object, a layout that lacks objects on its slot's start.
pub fn problem_focus(map: &MapDef, problem: &ScenarioProblem) -> (Option<Vec2>, Option<usize>) {
    use sc_sim::scenario::Asymmetry;
    let terrain = &map.terrain;
    let clamp = |p: Vec2| Some(clamp_to_map(terrain, p));
    let slot_spot = |slot: SlotId| map.slot(slot).and_then(|s| clamp(s.start.center()));
    match problem {
        ScenarioProblem::TooSteep { tile, .. } | ScenarioProblem::StartOffMap { tile, .. } => {
            (clamp(tile.center()), None)
        }
        ScenarioProblem::CameraStartOffMap { pos, .. } => (clamp(*pos), None),
        ScenarioProblem::NotSymmetric(Asymmetry::Tile(tile)) => (clamp(tile.center()), None),
        ScenarioProblem::NotSymmetric(Asymmetry::Corner(corner)) => (clamp(corner.pos()), None),
        ScenarioProblem::NotSymmetric(Asymmetry::Slot(slot)) => (slot_spot(*slot), None),
        ScenarioProblem::InvalidSpawn { spawn, .. } => {
            let center = map
                .spawns
                .get(*spawn)
                .and_then(|s| clamp(spawn_rect(s.kind, s.at, s.orientation).center()));
            (center, Some(*spawn))
        }
        ScenarioProblem::OverlappingFootprints { second, tile, .. } => (clamp(tile.center()), Some(*second)),
        ScenarioProblem::InvalidLayoutObject {
            slot, faction, index, ..
        }
        | ScenarioProblem::LayoutOverlap {
            slot, faction, index, ..
        }
        | ScenarioProblem::LayoutPlacement {
            slot, faction, index, ..
        }
        | ScenarioProblem::LayoutExtra {
            slot, faction, index, ..
        } => {
            let object = map.slot(*slot).and_then(|s| s.layouts.get(*faction).get(*index));
            let center = object.and_then(|o| clamp(spawn_rect(o.kind, o.at, o.orientation).center()));
            (center.or_else(|| slot_spot(*slot)), None)
        }
        ScenarioProblem::LayoutMissing { slot, .. } => (slot_spot(*slot), None),
        ScenarioProblem::UnknownSlot { slot, .. } | ScenarioProblem::SlotTaken { slot, .. } => (slot_spot(*slot), None),
        ScenarioProblem::MapTooBig { .. }
        | ScenarioProblem::SkirmishWithoutSymmetry
        | ScenarioProblem::SymmetryNeedsSquare { .. }
        | ScenarioProblem::NotSymmetric(_)
        | ScenarioProblem::NoSlots
        | ScenarioProblem::TooManySlots(_)
        | ScenarioProblem::SlotsNotInImageSets { .. }
        | ScenarioProblem::NoPlayers => (None, None),
    }
}
