//! Group destinations by StarCraft's **magic box** — `systems/controls/commands.md > Group Destinations (Magic
//! Box)` (design change 0021, the answer to part 1 of question Q0007), `docs/movement.md > 6.4`.
//!
//! When one location-targeted order (Move to a point, AttackMove, Patrol) goes to several units, each unit's own
//! destination is decided here, as the command applies (phase 4):
//! - The **box** is the bounding rectangle of the units' full Silhouettes (design change 0028), their collision
//!   boxes, so a straight line of units has a real box. The group's centre is the box's centre.
//! - A click **inside** the box, its edges included, makes the group **converge**: every unit goes to the clicked
//!   point itself, and those the crowd blocks stop by the occupied target rule (`docs/movement.md > 6.5`, rule 1).
//! - A click **outside** the box **keeps the arrangement**: the group moves by the vector from its centre to the
//!   click, so each unit goes to its position plus that vector and the group arrives in the shape it had.
//! - There is no formation movement: each unit travels to its destination at its own speed.
//!
//! An offset destination the unit can't reach becomes the reachable point nearest to it (design change 0025,
//! `systems/controls/behaviors.md > MovingToLocation > Unreachable target location`): the map's edge clamps it inside
//! the map, and a destination off the unit's ground moves to the nearest point the unit can stand on
//! ([`reachable`]).

use std::collections::BTreeMap;

use glam::Vec2;

use crate::ids::ObjectId;
use crate::map::Map;
use crate::nav::{self, Nav};
use crate::object::Object;
use crate::units::SU_PER_GU;

/// The destination of each of `units` (unique, existing units) for a location order to `click`, in the order of
/// `units`. `None` is the clicked point itself: a single unit goes there, as it always did, and so does every unit
/// of a group clicked inside its box. Outside the box, each unit's own point (the module docs).
pub(crate) fn assign(
    nav: &Nav,
    map: &Map,
    objects: &BTreeMap<ObjectId, Object>,
    units: &[ObjectId],
    click: Vec2,
) -> Vec<Option<Vec2>> {
    let everyone_to_the_click = vec![None; units.len()];
    let members: Vec<&Object> = units.iter().filter_map(|id| objects.get(id)).collect();
    if members.len() < 2 || members.len() != units.len() {
        return everyone_to_the_click;
    }
    // 0028: the box bounds the units' full Silhouettes, not their centres.
    let (lo, hi) = members.iter().fold((Vec2::MAX, Vec2::MIN), |(lo, hi), o| {
        let half = o.def().half_extents().unwrap_or(Vec2::ZERO);
        (lo.min(o.pos - half), hi.max(o.pos + half))
    });
    // 0021: a click inside the box converges. The edges count as inside. A group standing in a line has a box as
    // high as its units, so a click on or between them converges it (0028).
    if click.cmpge(lo).all() && click.cmple(hi).all() {
        return everyone_to_the_click;
    }
    // 0021: outside it, the whole group moves by one vector, from the box's centre to the click.
    let shift = click - (lo + hi) / 2.0;
    let map_size = Vec2::new(map.width() as f32, map.height() as f32) * SU_PER_GU;
    members
        .iter()
        .map(|o| Some(reachable(nav, map_size, o, o.pos + shift)))
        .collect()
}

/// Where unit `o` goes for its offset destination `dest` (design change 0025: the reachable point nearest to it):
/// - **The map's edge** clamps the destination so that the unit's box lies inside the map.
/// - A destination the unit **can't reach** (ground its grid blocks: cliff, water, mountain, a footprint; or
///   another component) moves to the nearest point the unit can stand on ([`nav::reachable_point`]): Q0008's
///   default for unreachable targets (gap G5), which single units get from the path service
///   (`docs/movement.md > 5.3`). The point, not the tile's centre, so that a group meeting a cliff lines up along
///   its edge instead of piling onto one tile.
///
/// Air units have no grid: the map's edge is their only limit.
fn reachable(nav: &Nav, map_size: Vec2, o: &Object, dest: Vec2) -> Vec2 {
    let def = o.def();
    let half = def.half_extents().unwrap_or(Vec2::ZERO);
    let dest = dest.max(half).min((map_size - half).max(half));
    let grid = def
        .unit
        .as_ref()
        .and_then(|unit| unit.base_def().nav_class())
        .zip(def.silhouette())
        .and_then(|(class, silhouette)| nav.grid(class, nav::size_class(silhouette)));
    let Some(grid) = grid else {
        return dest;
    };
    grid.component(grid.tile_of(o.pos))
        .and_then(|component| nav::reachable_point(grid, component, half, dest))
        .unwrap_or(dest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::defs::ObjectKind;
    use crate::ids::PlayerId;
    use crate::scenario::SpawnAt;
    use crate::sim::Sim;
    use crate::testing::ScenarioBuilder;

    const PK: ObjectKind = ObjectKind::Peacekeeper; // 24 × 24 su
    const GUARD: ObjectKind = ObjectKind::Guard; // 36 × 36 su

    fn v(x: f32, y: f32) -> Vec2 {
        Vec2::new(x, y)
    }

    /// The destinations of a group of units of the given kinds, at the given points, ordered to `click`.
    fn destinations(map: &str, units: &[(ObjectKind, Vec2)], click: Vec2) -> Vec<Option<Vec2>> {
        let mut builder = ScenarioBuilder::new(map);
        for &(kind, at) in units {
            builder = builder.spawn(kind, Some(PlayerId(1)), SpawnAt::Point(at));
        }
        let sim = Sim::new(&builder.build());
        let objects: BTreeMap<ObjectId, Object> = sim.objects().map(|o| (o.id, o.clone())).collect();
        let ids: Vec<ObjectId> = objects.keys().copied().collect();
        assign(sim.nav(), sim.map(), &objects, &ids, click)
    }

    /// 8 × 6 open tiles: 512 × 384 su.
    const OPEN: &str = "
        ........
        ........
        ........
        ........
        ........
        ........
    ";

    /// `> Group Destinations (Magic Box)`: outside the box, each unit's offset from the box's centre is kept.
    #[test]
    fn outside_the_box_the_group_moves_by_one_vector() {
        // 0028: the box bounds the silhouettes: x 88..198 (the Guard reaches 18 su east), y 82..132, its centre
        // (143, 107); the click lies east of it.
        let units = [(PK, v(100.0, 100.0)), (GUARD, v(180.0, 100.0)), (PK, v(130.0, 120.0))];
        let shift = v(400.0, 250.0) - v(143.0, 107.0);
        assert_eq!(
            destinations(OPEN, &units, v(400.0, 250.0)),
            units.map(|(_, at)| Some(at + shift))
        );
    }

    /// `> Group Destinations (Magic Box)`: inside the box, its edges included, everyone goes to the click. The box
    /// bounds the units' silhouettes (design change 0028).
    #[test]
    fn inside_the_box_everyone_goes_to_the_click() {
        // The box is x 88..192, y 88..172.
        let units = [(PK, v(100.0, 100.0)), (PK, v(180.0, 100.0)), (PK, v(130.0, 160.0))];
        for click in [
            v(140.0, 130.0),
            v(100.0, 100.0),
            v(180.0, 160.0),
            v(88.0, 88.0),
            v(192.0, 172.0),
        ] {
            assert_eq!(destinations(OPEN, &units, click), [None; 3], "{click}");
        }
        for click in [v(193.0, 130.0), v(140.0, 87.0)] {
            assert!(destinations(OPEN, &units, click).iter().all(Option::is_some), "{click}");
        }
        // 0028: a straight line of units has a real box, as high as the units: x 88..192, y 188..212. A click on a
        // unit or between two of them is inside; one just above or below the line is outside.
        let line = [(PK, v(100.0, 200.0)), (PK, v(140.0, 200.0)), (PK, v(180.0, 200.0))];
        for click in [v(150.0, 200.0), v(120.0, 209.0), v(160.0, 190.0), v(140.0, 212.0)] {
            assert_eq!(destinations(OPEN, &line, click), [None; 3], "{click}");
        }
        assert_eq!(
            destinations(OPEN, &line, v(150.0, 213.0)),
            line.map(|(_, at)| Some(at + v(10.0, 13.0)))
        );
    }

    /// A single unit goes to the clicked point itself, as before slice M.
    #[test]
    fn a_single_unit_goes_to_the_click() {
        assert_eq!(destinations(OPEN, &[(PK, v(100.0, 100.0))], v(400.0, 300.0)), [None]);
    }

    /// Question Q0008 (default): the map's edge clamps a destination inside the map, the unit's box included.
    #[test]
    fn the_maps_edge_clamps_a_destination() {
        // A line 160 su long, its centre (180, 300). Clicked 16 su from the east edge, its east half would leave the
        // map: the Guard (half 18) stops at x 494, the last Peacekeeper (half 12) at x 500.
        let units = [(PK, v(100.0, 300.0)), (GUARD, v(180.0, 300.0)), (PK, v(260.0, 300.0))];
        let ends = destinations(OPEN, &units, v(496.0, 40.0));
        assert_eq!(ends, [Some(v(416.0, 40.0)), Some(v(494.0, 40.0)), Some(v(500.0, 40.0))]);
        // Near the north-west corner.
        let ends = destinations(OPEN, &units, v(20.0, 20.0));
        assert_eq!(ends, [Some(v(12.0, 20.0)), Some(v(20.0, 20.0)), Some(v(100.0, 20.0))]);
    }

    /// Question Q0008 (default): a destination on ground the unit can't stand on, or cut off from it, moves to the
    /// nearest point it can stand on: not a tile's centre, but the edge of the obstacle.
    #[test]
    fn an_unreachable_destination_moves_to_the_nearest_point_the_unit_can_reach() {
        // A cliff on tiles (5..=6, 1..=2), x 320..448, y 64..192; a pond on (5..=7, 3..=5) around an island, the
        // open tile (6, 4).
        let map = "
            ........
            .....cc.
            .....cc.
            .....~~~
            .....~.~
            .....~~~
        ";
        // The box is x 32..224, y 128..288, its centre (128, 208); the click (320, 208) moves the group by (192, 0).
        let units = [(PK, v(160.0, 128.0)), (PK, v(224.0, 288.0)), (PK, v(32.0, 160.0))];
        let ends = destinations(map, &units, v(320.0, 208.0));
        // (352, 128) is on the cliff, 32 su from its west edge: the Peacekeeper stands against that edge.
        assert_eq!(ends[0], Some(v(308.0, 128.0)));
        // (416, 288) is on the island, another component: the nearest shore is west of the pond.
        assert_eq!(ends[1], Some(v(308.0, 288.0)));
        // (224, 160) is open ground: it stays.
        assert_eq!(ends[2], Some(v(224.0, 160.0)));
    }
}
