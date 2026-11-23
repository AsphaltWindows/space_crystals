//! The object edits behind the Place and Select tools, as plain functions on sim types, testable without Bevy:
//! snapping, footprints, placement checks, mirrored copies, hit tests, where validation problems are, and the
//! object palette.

use std::collections::BTreeSet;

use bevy::math::{Rect, Vec2};
use sc_sim::{
    Category, Faction, Footprint, Map, ObjectKind, PlayerId, SU_PER_GU, Scenario, ScenarioProblem, SpawnAt, TilePos,
};

use super::tools::Symmetry;
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

/// The area a spawn covers in sim space units: its tile footprint, or its silhouette around its point.
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

/// The tiles a tile-based spawn covers; none for units.
pub fn footprint_tiles(kind: ObjectKind, at: SpawnAt) -> Vec<TilePos> {
    let (Some((w, h)), SpawnAt::Tile(o)) = (tile_size(kind), at) else {
        return Vec::new();
    };
    (0..h as i32)
        .flat_map(|dy| (0..w as i32).map(move |dx| TilePos::new(o.x + dx, o.y + dy)))
        .collect()
}

/// Why an object can't be placed somewhere.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PlaceProblem {
    /// Not entirely on the map.
    OffMap,
    /// Its footprint overlaps the footprint of this spawn.
    Overlaps(usize),
}

/// Whether an object of `kind` fits at `at`: entirely on the map, and not overlapping the footprint of another
/// tile-based spawn. `moving` is the spawn being moved, which doesn't count.
pub fn check_placement(
    scenario: &Scenario,
    kind: ObjectKind,
    at: SpawnAt,
    moving: Option<usize>,
) -> Result<(), PlaceProblem> {
    let map = &scenario.map;
    match (tile_size(kind), at) {
        (Some((w, h)), SpawnAt::Tile(o)) => {
            let fits = |v: i32, n: u32, size: u32| v >= 0 && i64::from(v) + i64::from(n) <= i64::from(size);
            if !(fits(o.x, w, map.width()) && fits(o.y, h, map.height())) {
                return Err(PlaceProblem::OffMap);
            }
            let mine: BTreeSet<TilePos> = footprint_tiles(kind, at).into_iter().collect();
            let overlap =
                scenario.spawns.iter().enumerate().find(|(i, s)| {
                    Some(*i) != moving && footprint_tiles(s.kind, s.at).iter().any(|t| mine.contains(t))
                });
            match overlap {
                Some((i, _)) => Err(PlaceProblem::Overlaps(i)),
                None => Ok(()),
            }
        }
        (None, SpawnAt::Point(p)) if map.contains_point(p) => Ok(()),
        _ => Err(PlaceProblem::OffMap),
    }
}

/// The mirror image of a spawn location under `symmetry`, unless it is the location itself. A tile footprint is
/// mirrored as a whole; a point is mirrored through the map's center lines.
pub fn mirrored_spawn(map: &Map, kind: ObjectKind, at: SpawnAt, symmetry: Symmetry) -> Option<SpawnAt> {
    let (mirror_x, mirror_y) = match symmetry {
        Symmetry::None => return None,
        Symmetry::MirrorX => (true, false),
        Symmetry::MirrorY => (false, true),
        Symmetry::Point => (true, true),
    };
    let image = match at {
        SpawnAt::Tile(o) => {
            let (w, h) = tile_size(kind).unwrap_or((1, 1));
            let x = if mirror_x {
                map.width() as i32 - o.x - w as i32
            } else {
                o.x
            };
            let y = if mirror_y {
                map.height() as i32 - o.y - h as i32
            } else {
                o.y
            };
            SpawnAt::Tile(TilePos::new(x, y))
        }
        SpawnAt::Point(p) => {
            let size = map_extent_su(map);
            let x = if mirror_x { size.x - p.x } else { p.x };
            let y = if mirror_y { size.y - p.y } else { p.y };
            SpawnAt::Point(Vec2::new(x, y))
        }
    };
    (image != at).then_some(image)
}

/// The owner of a mirrored copy. On a two-player map the copy goes to the other player, so that both sides get
/// the same; otherwise it keeps the owner (placeholder rule, design question Q0004).
pub fn mirrored_owner(owner: Option<PlayerId>, players: usize) -> Option<PlayerId> {
    match owner {
        Some(PlayerId(1)) if players == 2 => Some(PlayerId(2)),
        Some(PlayerId(2)) if players == 2 => Some(PlayerId(1)),
        other => other,
    }
}

/// The spawn drawn at `pos`: of those whose area contains it, the frontmost one (the southernmost, as southern
/// objects draw in front, then the latest).
pub fn spawn_at_point(scenario: &Scenario, pos: Vec2) -> Option<usize> {
    scenario
        .spawns
        .iter()
        .enumerate()
        .map(|(i, s)| (i, spawn_rect(s.kind, s.at)))
        .filter(|(_, r)| r.contains(pos))
        .max_by(|(i, a), (j, b)| a.center().y.total_cmp(&b.center().y).then(i.cmp(j)))
        .map(|(i, _)| i)
}

/// What a click on a problem in the validation list centres the camera on, and the spawn it concerns.
pub fn problem_focus(scenario: &Scenario, problem: &ScenarioProblem) -> (Option<Vec2>, Option<usize>) {
    let map = &scenario.map;
    let slot_spot = |player: PlayerId| {
        let slot = scenario.players.iter().find(|p| p.id == player)?;
        slot.start_location
            .map(TilePos::center)
            .or(slot.camera_start)
            .map(|p| clamp_to_map(map, p))
    };
    match problem {
        ScenarioProblem::DuplicatePlayerId(player) => (slot_spot(*player), None),
        ScenarioProblem::PlayerIdOutOfRange { player, .. } => (slot_spot(*player), None),
        ScenarioProblem::StartLocationOffMap { tile, .. } => (Some(clamp_to_map(map, tile.center())), None),
        ScenarioProblem::CameraStartOffMap { pos, .. } => (Some(clamp_to_map(map, *pos)), None),
        ScenarioProblem::InvalidSpawn { spawn, .. } => {
            let center = scenario
                .spawns
                .get(*spawn)
                .map(|s| clamp_to_map(map, spawn_rect(s.kind, s.at).center()));
            (center, Some(*spawn))
        }
        ScenarioProblem::OverlappingFootprints { second, tile, .. } => {
            (Some(clamp_to_map(map, tile.center())), Some(*second))
        }
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
    use sc_sim::{Spawn, scenarios};

    fn t(x: i32, y: i32) -> TilePos {
        TilePos::new(x, y)
    }

    fn tile_spawn(kind: ObjectKind, x: i32, y: i32) -> Spawn {
        Spawn {
            kind,
            owner: None,
            at: SpawnAt::Tile(t(x, y)),
        }
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
        let mut scenario = scenarios::empty(8, 6);
        scenario.spawns = vec![tile_spawn(ObjectKind::SupplyDeliveryStation, 2, 2)];
        let station = ObjectKind::SupplyDeliveryStation;
        let patch = ObjectKind::CrystalPatch;
        let at = |x, y| SpawnAt::Tile(t(x, y));
        assert_eq!(check_placement(&scenario, patch, at(0, 0), None), Ok(()));
        assert_eq!(
            check_placement(&scenario, patch, at(3, 3), None),
            Err(PlaceProblem::Overlaps(0))
        );
        assert_eq!(
            check_placement(&scenario, station, at(7, 0), None),
            Err(PlaceProblem::OffMap)
        );
        assert_eq!(
            check_placement(&scenario, station, at(-1, 0), None),
            Err(PlaceProblem::OffMap)
        );
        assert_eq!(
            check_placement(&scenario, station, at(3, 3), Some(0)),
            Ok(()),
            "moving onto itself"
        );
        assert_eq!(
            check_placement(&scenario, station, at(4, 2), None),
            Ok(()),
            "touching is fine"
        );
    }

    #[test]
    fn mirrored_spawns_land_on_the_far_side() {
        let map = Map::new(10, 8);
        let station = ObjectKind::SupplyDeliveryStation;
        let at = SpawnAt::Tile(t(1, 2));
        assert_eq!(
            mirrored_spawn(&map, station, at, Symmetry::MirrorX),
            Some(SpawnAt::Tile(t(7, 2)))
        );
        assert_eq!(
            mirrored_spawn(&map, station, at, Symmetry::Point),
            Some(SpawnAt::Tile(t(7, 4)))
        );
        assert_eq!(mirrored_spawn(&map, station, at, Symmetry::None), None);
        let centered = SpawnAt::Tile(t(4, 3));
        assert_eq!(
            mirrored_spawn(&map, station, centered, Symmetry::Point),
            None,
            "its own image"
        );
        let point = SpawnAt::Point(Vec2::new(10.0, 20.0));
        assert_eq!(
            mirrored_spawn(&map, ObjectKind::CrystalPatch, point, Symmetry::MirrorY),
            Some(SpawnAt::Point(Vec2::new(10.0, 512.0 - 20.0)))
        );
        assert_eq!(mirrored_owner(Some(PlayerId(1)), 2), Some(PlayerId(2)));
        assert_eq!(mirrored_owner(Some(PlayerId(1)), 3), Some(PlayerId(1)));
        assert_eq!(mirrored_owner(None, 2), None);
    }

    #[test]
    fn hit_tests_find_the_frontmost_object() {
        let mut scenario = scenarios::empty(8, 6);
        scenario.spawns = vec![
            tile_spawn(ObjectKind::SupplyDeliveryStation, 1, 1),
            tile_spawn(ObjectKind::CrystalPatch, 2, 2),
            tile_spawn(ObjectKind::CrystalPatch, 5, 5),
        ];
        assert_eq!(spawn_at_point(&scenario, Vec2::new(100.0, 100.0)), Some(0));
        assert_eq!(
            spawn_at_point(&scenario, Vec2::new(150.0, 150.0)),
            Some(1),
            "the overlapping patch"
        );
        assert_eq!(spawn_at_point(&scenario, Vec2::new(5.0, 5.0)), None);
    }

    #[test]
    fn problems_point_at_their_spawn() {
        let mut scenario = scenarios::empty(8, 6);
        scenario.spawns = vec![
            tile_spawn(ObjectKind::CrystalPatch, 2, 2),
            tile_spawn(ObjectKind::CrystalPatch, 2, 2),
            tile_spawn(ObjectKind::CrystalPatch, 40, 2),
        ];
        let problems = scenario.validate();
        assert_eq!(problems.len(), 2);
        assert_eq!(
            problem_focus(&scenario, &problems[0]),
            (Some(t(2, 2).center()), Some(1))
        );
        let (center, spawn) = problem_focus(&scenario, &problems[1]);
        assert_eq!(spawn, Some(2));
        assert!(
            scenario.map.contains_point(center.unwrap()),
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
