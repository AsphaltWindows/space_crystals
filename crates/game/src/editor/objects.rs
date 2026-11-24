//! The object edits behind the Place and Select tools, as plain functions on sim types, testable without Bevy:
//! snapping, footprints, placement checks, hit tests, where validation problems are, and the object palette.

use std::collections::BTreeSet;

use bevy::math::{Rect, Vec2};
use sc_sim::{
    Category, Faction, Footprint, Map, MapDef, MapType, ObjectKind, SU_PER_GU, ScenarioProblem, SlotId, Spawn, SpawnAt,
    TilePos,
};

use crate::camera::map_extent_su;

/// A tile-based object type's footprint in tiles, or `None` for units.
pub fn tile_size(kind: ObjectKind) -> Option<(u32, u32)> {
    match kind.def().footprint {
        Footprint::Tiles { w, h } => Some((w, h)),
        Footprint::Silhouette { .. } => None,
    }
}

/// Where an object of `kind` goes for the cursor at `pos`: a tile-based object's footprint centres on the grid
/// point nearest to `pos` (for a 1×1 footprint, the tile under it); a unit takes the point itself.
pub fn snap_spawn(kind: ObjectKind, pos: Vec2) -> SpawnAt {
    match tile_size(kind) {
        Some((w, h)) => {
            let origin = |p: f32, n: u32| (p / SU_PER_GU - n as f32 / 2.0 + 0.5).floor() as i32;
            SpawnAt::Tile(TilePos::new(origin(pos.x, w), origin(pos.y, h)))
        }
        None => SpawnAt::Point(pos),
    }
}

/// Where a dragged object goes when its center is dragged to `center`: snapped like a placement and kept
/// entirely on the map.
pub fn dragged_spawn(map: &Map, kind: ObjectKind, center: Vec2) -> SpawnAt {
    match snap_spawn(kind, center) {
        SpawnAt::Tile(o) => {
            let (w, h) = tile_size(kind).unwrap_or((1, 1));
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

/// The area a spawn covers in sim space units: its tile footprint, or its silhouette around its point. (Every
/// tile-based type has a square footprint so far, so building orientations don't change it.)
pub fn spawn_rect(kind: ObjectKind, at: SpawnAt) -> Rect {
    match (kind.def().footprint, at) {
        (Footprint::Tiles { w, h }, SpawnAt::Tile(o)) => {
            Rect::from_corners(o.corner(), o.corner() + Vec2::new(w as f32, h as f32) * SU_PER_GU)
        }
        (Footprint::Silhouette { w, h }, SpawnAt::Point(p)) => Rect::from_center_size(p, Vec2::new(w, h)),
        // The wrong kind of location (an invalid spawn): one tile, or a half-tile square around the point.
        (_, SpawnAt::Tile(o)) => Rect::from_corners(o.corner(), o.corner() + Vec2::splat(SU_PER_GU)),
        (_, SpawnAt::Point(p)) => Rect::from_center_size(p, Vec2::splat(SU_PER_GU / 2.0)),
    }
}

/// The tiles a tile-based object of `kind` at `at` covers, unturned; none for units. A spawn of a damaged file may
/// lie anywhere, even at the ends of the `i32` range: tiles past them are left out instead of overflowing.
pub fn footprint_tiles(kind: ObjectKind, at: SpawnAt) -> Vec<TilePos> {
    let (Some((w, h)), SpawnAt::Tile(o)) = (tile_size(kind), at) else {
        return Vec::new();
    };
    tiles_from(o, w, h)
}

/// The tiles a spawn covers, its building orientation included ([`footprint_tiles`]).
pub fn spawn_tiles(spawn: &Spawn) -> Vec<TilePos> {
    let (Some((w, h)), SpawnAt::Tile(o)) = (tile_size(spawn.kind), spawn.at) else {
        return Vec::new();
    };
    let (w, h) = if spawn.orientation.swaps_axes() { (h, w) } else { (w, h) };
    tiles_from(o, w, h)
}

fn tiles_from(o: TilePos, w: u32, h: u32) -> Vec<TilePos> {
    (0..h as i32)
        .flat_map(|dy| (0..w as i32).map(move |dx| (dx, dy)))
        .filter_map(|(dx, dy)| Some(TilePos::new(o.x.checked_add(dx)?, o.y.checked_add(dy)?)))
        .collect()
}

/// Why an object can't be placed somewhere.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PlaceProblem {
    /// Its center lies in the locked part of a symmetric map, which is generated
    /// (`core/map_editor.md > Symmetry > Editing a symmetric map`).
    Locked,
    /// `core/map_editor.md > Map Types > Skirmish Default`: no owned objects outside the Starting Layouts.
    OwnedOnSkirmishMap,
    /// Not entirely on the map.
    OffMap,
    /// `core/map.md > Elevation > Map resources only on flat terrain`: this footprint tile is sloped.
    NotFlat(TilePos),
    /// Its footprint, or the footprint of one of its symmetric images, overlaps the footprint of this spawn.
    Overlaps(usize),
    /// Its footprint partly overlaps one of its own symmetric images: it lies across an axis or around the center
    /// without being its own image (placeholder rule until the design settles objects on the axes,
    /// `core/map_editor.md > Symmetry`'s TBD).
    OverlapsOwnImage,
}

impl PlaceProblem {
    /// Why `kind` can't go there, for the status line.
    pub fn message(self, kind: ObjectKind) -> String {
        let name = kind.def().name;
        match self {
            PlaceProblem::Locked => {
                format!(
                    "Locked: that part of the map is generated by its symmetry; place the {name} in the bright part"
                )
            }
            PlaceProblem::OwnedOnSkirmishMap => {
                format!("A Skirmish Default map takes no owned {name} outside the Starting Layouts: choose Neutral")
            }
            PlaceProblem::OffMap => format!("{name} doesn't fit on the map there"),
            PlaceProblem::NotFlat(t) => format!("{name} needs flat ground: tile ({}, {}) is sloped", t.x, t.y),
            PlaceProblem::Overlaps(i) => format!("{name} would overlap spawn #{i}"),
            PlaceProblem::OverlapsOwnImage => format!(
                "{name} would overlap its own mirror image: put it on the axis exactly, or away from it (placeholder rule)"
            ),
        }
    }
}

/// Whether `spawn` fits on `map` alone: entirely on the map, and on flat ground if its type needs it.
fn check_fit(map: &Map, spawn: &Spawn) -> Result<(), PlaceProblem> {
    match (tile_size(spawn.kind), spawn.at) {
        (Some((w, h)), SpawnAt::Tile(o)) => {
            let (w, h) = if spawn.orientation.swaps_axes() { (h, w) } else { (w, h) };
            let fits = |v: i32, n: u32, size: u32| v >= 0 && i64::from(v) + i64::from(n) <= i64::from(size);
            if !(fits(o.x, w, map.width()) && fits(o.y, h, map.height())) {
                return Err(PlaceProblem::OffMap);
            }
            // Row by row, like the sim's validation, so both name the same tile.
            if spawn.kind.def().needs_flat_ground
                && let Some(sloped) = spawn_tiles(spawn).into_iter().find(|t| !map.is_flat(*t))
            {
                return Err(PlaceProblem::NotFlat(sloped));
            }
            Ok(())
        }
        (None, SpawnAt::Point(p)) if map.contains_point(p) => Ok(()),
        _ => Err(PlaceProblem::OffMap),
    }
}

/// Whether `spawn` may go where it is on `map`: in the editable part, neutral on a Skirmish Default map, entirely
/// on the map, on flat ground if its type needs it, and, with the images the map's symmetry generates for it, not
/// overlapping the footprint of another tile-based spawn or the box of another unit in its layer, nor partly
/// overlapping an image of itself. `moving` is the spawn being moved, which doesn't count, and neither do its
/// images.
pub fn check_placement(map: &MapDef, spawn: &Spawn, moving: Option<usize>) -> Result<(), PlaceProblem> {
    if !map.is_spawn_editable(spawn) {
        return Err(PlaceProblem::Locked);
    }
    if map.map_type == MapType::SkirmishDefault && spawn.owner.is_some() {
        return Err(PlaceProblem::OwnedOnSkirmishMap);
    }
    check_fit(&map.terrain, spawn)?;
    let images = map.spawn_images(spawn);
    let all: Vec<&Spawn> = std::iter::once(spawn).chain(&images).collect();
    let footprints: Vec<BTreeSet<TilePos>> = all.iter().map(|s| spawn_tiles(s).into_iter().collect()).collect();
    for (i, a) in footprints.iter().enumerate() {
        if footprints[i + 1..].iter().any(|b| !a.is_disjoint(b))
            || all[i + 1..].iter().any(|b| units_overlap(all[i], b))
        {
            return Err(PlaceProblem::OverlapsOwnImage);
        }
    }
    // The spawn being moved, and its images, which move with it.
    let mut skip = BTreeSet::new();
    if let Some(index) = moving
        && let Some(moved) = map.spawns.get(index)
    {
        skip.insert(index);
        for image in map.spawn_images(moved) {
            if let Some(j) = map.spawns.iter().position(|s| *s == image) {
                skip.insert(j);
            }
        }
    }
    let overlap = map.spawns.iter().enumerate().find(|(j, other)| {
        !skip.contains(j)
            && (spawn_tiles(other)
                .iter()
                .any(|t| footprints.iter().any(|f| f.contains(t)))
                || all.iter().any(|s| units_overlap(s, other)))
    });
    match overlap {
        Some((j, _)) => Err(PlaceProblem::Overlaps(j)),
        None => Ok(()),
    }
}

/// `docs/movement.md > 3`: whether two unit spawns' boxes overlap in a layer where units collide (touching is
/// allowed), by the sim's test. Units on blocked ground are left to validation, which lists them.
fn units_overlap(a: &Spawn, b: &Spawn) -> bool {
    let unit_box = |s: &Spawn| {
        let def = s.kind.def();
        let SpawnAt::Point(pos) = s.at else {
            return None;
        };
        let domain = def.unit.as_ref()?.base_def().domain;
        Some((pos, def.half_extents()?, domain))
    };
    match (unit_box(a), unit_box(b)) {
        (Some((pa, ha, la)), Some((pb, hb, lb))) => {
            la == lb && la.hard_collision() && (pa.x - pb.x).abs() < ha.x + hb.x && (pa.y - pb.y).abs() < ha.y + hb.y
        }
        _ => false,
    }
}

/// The spawn drawn at `pos`: of those whose area contains it, the frontmost one (the southernmost, as southern
/// objects draw in front, then the latest).
pub fn spawn_at_point(map: &MapDef, pos: Vec2) -> Option<usize> {
    map.spawns
        .iter()
        .enumerate()
        .map(|(i, s)| (i, spawn_rect(s.kind, s.at)))
        .filter(|(_, r)| r.contains(pos))
        .max_by(|(i, a), (j, b)| a.center().y.total_cmp(&b.center().y).then(i.cmp(j)))
        .map(|(i, _)| i)
}

/// What a click on a problem in the validation list centres the camera on, and the spawn it concerns.
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
                .and_then(|s| clamp(spawn_rect(s.kind, s.at).center()));
            (center, Some(*spawn))
        }
        ScenarioProblem::OverlappingFootprints { second, tile, .. } => (clamp(tile.center()), Some(*second)),
        ScenarioProblem::InvalidLayoutObject { slot, .. }
        | ScenarioProblem::LayoutOverlap { slot, .. }
        | ScenarioProblem::LayoutPlacement { slot, .. }
        | ScenarioProblem::LayoutExtra { slot, .. }
        | ScenarioProblem::LayoutMissing { slot, .. } => (slot_spot(*slot), None),
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

/// A heading of the object palette and its object types.
#[derive(Clone, Debug, PartialEq)]
pub struct PaletteGroup {
    pub title: String,
    pub kinds: Vec<ObjectKind>,
}

/// The object palette: every type of `ObjectKind::ALL`, grouped by faction (neutral first, then
/// `Faction::ALL`) and category, so new types appear without editor changes.
pub fn palette_groups() -> Vec<PaletteGroup> {
    let factions = std::iter::once(None).chain(Faction::ALL.iter().copied().map(Some));
    let mut groups = Vec::new();
    for faction in factions {
        for category in [Category::Resource, Category::Structure, Category::Unit] {
            let kinds: Vec<ObjectKind> = ObjectKind::ALL
                .iter()
                .copied()
                .filter(|k| k.def().faction == faction && k.def().category == category)
                .collect();
            if !kinds.is_empty() {
                groups.push(PaletteGroup {
                    title: group_title(faction, category),
                    kinds,
                });
            }
        }
    }
    groups
}

fn group_title(faction: Option<Faction>, category: Category) -> String {
    let category = match category {
        Category::Resource => "Resources",
        Category::Structure => "Structures",
        Category::Unit => "Units",
    };
    match faction {
        None => category.to_string(),
        Some(f) => format!("{} {}", faction_name(f), category.to_lowercase()),
    }
}

/// A faction's display name.
pub fn faction_name(faction: Faction) -> &'static str {
    match faction {
        Faction::Gdo => "GDO",
        Faction::Syndicate => "Syndicate",
        Faction::Cults => "Cults",
        Faction::Colonists => "Colonists",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sc_sim::{MapSettings, Symmetry, scenarios};

    fn t(x: i32, y: i32) -> TilePos {
        TilePos::new(x, y)
    }

    fn tile_spawn(kind: ObjectKind, x: i32, y: i32) -> Spawn {
        Spawn::new(kind, None, SpawnAt::Tile(t(x, y)))
    }

    #[test]
    fn placements_snap_to_the_grid_and_stay_on_the_map() {
        let patch = ObjectKind::CrystalPatch; // 1×1
        let station = ObjectKind::SupplyDeliveryStation; // 2×2
        assert_eq!(snap_spawn(patch, Vec2::new(100.0, 70.0)), SpawnAt::Tile(t(1, 1)));
        // A 2×2 footprint centres on the nearest grid point: (128, 128) is the corner of tiles 1 and 2.
        assert_eq!(snap_spawn(station, Vec2::new(120.0, 140.0)), SpawnAt::Tile(t(1, 1)));
        let map = Map::new(6, 5);
        assert_eq!(
            dragged_spawn(&map, station, Vec2::new(-500.0, 9000.0)),
            SpawnAt::Tile(t(0, 3))
        );
        assert_eq!(spawn_rect(station, SpawnAt::Tile(t(1, 1))).center(), Vec2::splat(128.0));
        let p = clamp_to_map(&map, Vec2::new(1e9, f32::NAN));
        assert!(map.contains_point(p));
    }

    #[test]
    fn placement_checks_the_map_and_other_footprints() {
        let mut map = scenarios::empty_map(8, 6);
        map.spawns = vec![tile_spawn(ObjectKind::SupplyDeliveryStation, 2, 2)];
        let station = |x, y| tile_spawn(ObjectKind::SupplyDeliveryStation, x, y);
        let patch = |x, y| tile_spawn(ObjectKind::CrystalPatch, x, y);
        assert_eq!(check_placement(&map, &patch(0, 0), None), Ok(()));
        assert_eq!(
            check_placement(&map, &patch(3, 3), None),
            Err(PlaceProblem::Overlaps(0))
        );
        assert_eq!(check_placement(&map, &station(7, 0), None), Err(PlaceProblem::OffMap));
        assert_eq!(check_placement(&map, &station(-1, 0), None), Err(PlaceProblem::OffMap));
        assert_eq!(
            check_placement(&map, &station(3, 3), Some(0)),
            Ok(()),
            "moving onto itself"
        );
        assert_eq!(check_placement(&map, &station(4, 2), None), Ok(()), "touching is fine");
        // `core/map.md > Elevation > Map resources only on flat terrain`: one sloped tile is enough to refuse it.
        map.terrain.set_corner(sc_sim::CornerPos::new(6, 1), 1);
        assert_eq!(
            check_placement(&map, &station(5, 0), None),
            Err(PlaceProblem::NotFlat(t(5, 0)))
        );
        assert_eq!(
            check_placement(&map, &patch(6, 1), None),
            Err(PlaceProblem::NotFlat(t(6, 1)))
        );
        assert_eq!(check_placement(&map, &patch(7, 1), None), Ok(()));
        assert!(
            PlaceProblem::NotFlat(t(5, 0))
                .message(ObjectKind::SupplyDeliveryStation)
                .contains("flat ground")
        );
        // Skirmish Default maps take neutral objects only.
        let mut owned = patch(0, 0);
        owned.owner = Some(SlotId(1));
        map.map_type = MapType::SkirmishDefault;
        assert_eq!(
            check_placement(&map, &owned, None),
            Err(PlaceProblem::OwnedOnSkirmishMap)
        );
    }

    /// On a symmetric map an object goes in the editable part, and neither it nor its images may overlap another
    /// object or partly overlap each other (the placeholder rule for objects on the axes).
    #[test]
    fn placement_on_a_symmetric_map() {
        let map = MapSettings {
            symmetry: Symmetry::MirrorVerticalAxis,
            width: 17,
            height: 16,
            ..MapSettings::default()
        }
        .create()
        .unwrap();
        let station = |x, y| tile_spawn(ObjectKind::SupplyDeliveryStation, x, y);
        assert_eq!(check_placement(&map, &station(2, 2), None), Ok(()));
        assert_eq!(check_placement(&map, &station(12, 2), None), Err(PlaceProblem::Locked));
        // The axis runs through column 8: a 2x2 station at column 7 or 8 straddles it and overlaps its image.
        assert_eq!(
            check_placement(&map, &station(7, 2), None),
            Err(PlaceProblem::OverlapsOwnImage)
        );
        let patch = tile_spawn(ObjectKind::CrystalPatch, 8, 2);
        assert_eq!(
            check_placement(&map, &patch, None),
            Ok(()),
            "on the axis: its own image"
        );
        // An image that would overlap another object.
        let mut map = map;
        map.spawns = vec![station(13, 5)];
        assert_eq!(
            check_placement(&map, &station(2, 5), None),
            Err(PlaceProblem::Overlaps(0))
        );
    }

    /// Review finding: a spawn of a damaged file at the end of the `i32` range must not overflow.
    #[test]
    fn footprints_at_the_ends_of_the_coordinate_range_never_overflow() {
        let station = ObjectKind::SupplyDeliveryStation;
        assert_eq!(footprint_tiles(station, SpawnAt::Tile(t(i32::MAX, i32::MAX))).len(), 1);
        assert_eq!(footprint_tiles(station, SpawnAt::Tile(t(i32::MAX - 1, 0))).len(), 4);
        let mut map = scenarios::empty_map(8, 6);
        map.spawns = vec![
            tile_spawn(station, i32::MAX, i32::MAX),
            tile_spawn(station, i32::MIN, 3),
        ];
        assert_eq!(check_placement(&map, &tile_spawn(station, 1, 1), None), Ok(()));
        let _ = spawn_at_point(&map, Vec2::new(1e30, -1e30));
        map.symmetry = Symmetry::Rotate180;
        let _ = check_placement(&map, &tile_spawn(station, i32::MAX, i32::MIN), None);
    }

    #[test]
    fn hit_tests_find_the_frontmost_object() {
        let mut map = scenarios::empty_map(8, 6);
        map.spawns = vec![
            tile_spawn(ObjectKind::SupplyDeliveryStation, 1, 1),
            tile_spawn(ObjectKind::CrystalPatch, 2, 2),
            tile_spawn(ObjectKind::CrystalPatch, 5, 5),
        ];
        assert_eq!(spawn_at_point(&map, Vec2::new(100.0, 100.0)), Some(0));
        assert_eq!(
            spawn_at_point(&map, Vec2::new(150.0, 150.0)),
            Some(1),
            "the overlapping patch"
        );
        assert_eq!(spawn_at_point(&map, Vec2::new(5.0, 5.0)), None);
    }

    #[test]
    fn problems_point_at_their_spawn() {
        let mut map = scenarios::empty_map(8, 6);
        map.spawns = vec![
            tile_spawn(ObjectKind::CrystalPatch, 2, 2),
            tile_spawn(ObjectKind::CrystalPatch, 2, 2),
            tile_spawn(ObjectKind::CrystalPatch, 40, 2),
        ];
        let problems = map.validate();
        assert_eq!(problems.len(), 2);
        assert_eq!(problem_focus(&map, &problems[0]), (Some(t(2, 2).center()), Some(1)));
        let (center, spawn) = problem_focus(&map, &problems[1]);
        assert_eq!(spawn, Some(2));
        assert!(
            map.terrain.contains_point(center.unwrap()),
            "off-map spots are clamped onto the map"
        );
    }

    #[test]
    fn the_palette_lists_every_object_type_once() {
        let groups = palette_groups();
        let listed: Vec<ObjectKind> = groups.iter().flat_map(|g| g.kinds.iter().copied()).collect();
        let mut sorted = listed.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), listed.len(), "no type twice");
        assert_eq!(sorted.len(), ObjectKind::ALL.len(), "every type");
        assert_eq!(groups[0].title, "Resources");
    }
}
