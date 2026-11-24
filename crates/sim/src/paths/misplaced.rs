//! Where a game's starting units may stand (`docs/movement.md > 3`): the check of scenario validation and the
//! editor, which have no `Sim` and so no grids of its objects.

use std::collections::BTreeMap;

use glam::Vec2;

use super::{blocking_footprint, box_overlaps_rect, boxes_overlap};
use crate::defs::{Domain, ObjectKind};
use crate::map::Map;
use crate::nav::{self, NavClass, NavGrid, NavRules, TileRect};
use crate::object::Placement;
use crate::scenario::SpawnProblem;

/// `docs/movement.md > 3` for the objects a game starts with, which skip the command gate: each object of
/// `objects`, in order, against the map and the objects before it. A unit's box must be clear of impassable
/// terrain, the map's edge and the blocking footprints, and overlap no unit of its layer; a blocking footprint must
/// not cover a ground unit. Lists the position and problem of each object that breaks this, so of two objects in
/// conflict, the later one. The same test as the sim's grids (`nav::box_clear`), on grids of the terrain alone,
/// built only for the unit classes present.
pub(crate) fn misplaced<'a>(
    map: &Map,
    objects: impl IntoIterator<Item = (ObjectKind, Vec2, Option<&'a Placement>)>,
) -> Vec<(usize, SpawnProblem)> {
    let rules = NavRules::default();
    let mut grids: BTreeMap<(NavClass, u8), NavGrid> = BTreeMap::new();
    let mut units: Vec<(Vec2, Vec2, Domain)> = Vec::new();
    let mut footprints: Vec<TileRect> = Vec::new();
    let mut problems = Vec::new();
    for (i, (kind, pos, placement)) in objects.into_iter().enumerate() {
        if let Some(rect) = placement.and_then(|p| blocking_footprint(kind, p, rules)) {
            if units
                .iter()
                .any(|&(at, half, domain)| domain == Domain::Ground && box_overlaps_rect(at, half, rect))
            {
                problems.push((i, SpawnProblem::CoversUnit));
            }
            footprints.push(rect);
            continue;
        }
        let def = kind.def();
        let (Some(unit), Some(silhouette)) = (&def.unit, def.silhouette()) else {
            continue;
        };
        let half = silhouette / 2.0;
        let base = unit.base_def();
        let on_impassable = base.nav_class().is_some_and(|class| {
            let size = nav::size_class(silhouette);
            let grid = grids
                .entry((class, size))
                .or_insert_with(|| NavGrid::build(map, class, size, &[], rules));
            !nav::box_clear(grid, half, pos)
        });
        // Surface footprints block ground units only (`nav::NavGrid::build`).
        let on_footprint =
            base.domain == Domain::Ground && footprints.iter().any(|&rect| box_overlaps_rect(pos, half, rect));
        if on_impassable || on_footprint {
            problems.push((i, SpawnProblem::Blocked));
        } else if base.domain.hard_collision()
            && units
                .iter()
                .any(|&(at, other, domain)| domain == base.domain && boxes_overlap(pos, half, at, other))
        {
            problems.push((i, SpawnProblem::OverlapsUnit));
        }
        units.push((pos, half, base.domain));
    }
    problems
}
