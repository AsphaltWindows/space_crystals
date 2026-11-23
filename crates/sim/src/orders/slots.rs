//! Group slots — `docs/movement.md > 6.4`, gap G1 (question Q0007).
//!
//! Several units ordered to one point can't all stand on it. Each gets its own slot on a lattice around the point,
//! nearest free slots first, and goes there at its own speed.

use std::collections::BTreeMap;

use glam::Vec2;

use crate::ids::ObjectId;
use crate::nav::{self, Nav, NavClass};
use crate::object::Object;

/// G1 (question Q0007): space between neighbouring slots, on top of the largest silhouette in the group.
pub const SLOT_GAP: f32 = 8.0;

/// A member of the group, as the slots see it.
struct Member {
    id: ObjectId,
    pos: Vec2,
    half: Vec2,
    /// The unit's grid, `(class, size class)`; `None` for air units.
    grid: Option<(NavClass, u8)>,
}

impl Member {
    fn of(o: &Object) -> Option<Member> {
        let def = o.def();
        let silhouette = def.silhouette()?;
        let grid = def
            .unit
            .as_ref()?
            .base_def()
            .nav_class()
            .map(|class| (class, nav::size_class(silhouette)));
        Some(Member {
            id: o.id,
            pos: o.pos,
            half: silhouette / 2.0,
            grid,
        })
    }
}

/// `docs/movement.md > 6.4`: the point each of `units` (unique, existing units) goes to for a move to `click`, in
/// the order of `units`. A single unit goes to the clicked point itself (`None`).
///
/// 1. The **lead** is the member with the largest silhouette (the lowest id among equals); its grid and component
///    are the group's. Members of another class or component path to the nearest point they can reach.
/// 2. The **anchor** is the click's tile, retargeted into the lead's component as in 5.3: the clicked point if its
///    tile is in the component, else the centre of the component's nearest tile.
/// 3. **Candidates** lie on a square lattice around the anchor with pitch = largest silhouette + [`SLOT_GAP`],
///    nearest first (ties north to south, then west to east). A candidate is kept if the largest member's box
///    there is clear of blocked tiles and its tile is in the anchor's component. The first `n` kept are the slots;
///    at most `4n + 64` candidates are examined.
/// 4. **Assignment**: the slot farthest from the members' centroid takes its nearest unassigned member (ties by
///    id), and so on. Members left without a slot go to the anchor.
pub(crate) fn assign(
    nav: &Nav,
    objects: &BTreeMap<ObjectId, Object>,
    units: &[ObjectId],
    click: Vec2,
) -> Vec<Option<Vec2>> {
    let everyone_to_the_click = vec![None; units.len()];
    let members: Vec<Member> = units.iter().filter_map(|id| Member::of(objects.get(id)?)).collect();
    if members.len() < 2 || members.len() != units.len() {
        return everyone_to_the_click;
    }
    // The lead: the largest silhouette, the lowest id among equals.
    let extent = |m: &Member| m.half.max_element();
    let lead = members
        .iter()
        .min_by(|a, b| extent(b).total_cmp(&extent(a)).then(a.id.cmp(&b.id)))
        .expect("at least two members");
    let half = members.iter().fold(Vec2::ZERO, |h, m| h.max(m.half));
    let pitch = 2.0 * half.max_element() + SLOT_GAP;
    // Air units (no grid) aren't in M1: such a group goes to the click.
    let Some(grid) = lead.grid.and_then(|(class, size)| nav.grid(class, size)) else {
        return everyone_to_the_click;
    };
    let Some(component) = grid.component(grid.tile_of(lead.pos)) else {
        return everyone_to_the_click;
    };
    let click_tile = grid.tile_of(click);
    let anchor = if grid.component(click_tile) == Some(component) {
        click
    } else {
        let tile = grid
            .nearest_in_component(component, click_tile)
            .expect("the component exists");
        grid.center(tile)
    };

    let n = members.len();
    let slots: Vec<Vec2> = lattice(4 * n + 64)
        .map(|(i, j)| anchor + Vec2::new(i as f32, j as f32) * pitch)
        .filter(|&p| nav::box_clear(grid, half, p) && grid.component(grid.tile_of(p)) == Some(component))
        .take(n)
        .collect();

    // Greedy assignment, farthest slot first (ties in lattice order).
    let centroid = members.iter().fold(Vec2::ZERO, |s, m| s + m.pos) / n as f32;
    let mut by_distance: Vec<usize> = (0..slots.len()).collect();
    by_distance.sort_by(|&a, &b| {
        let (da, db) = (slots[a].distance_squared(centroid), slots[b].distance_squared(centroid));
        db.total_cmp(&da).then(a.cmp(&b))
    });
    let mut assigned: Vec<Option<Vec2>> = vec![None; n];
    for s in by_distance {
        let slot = slots[s];
        let nearest = (0..n).filter(|&m| assigned[m].is_none()).min_by(|&a, &b| {
            let (da, db) = (
                members[a].pos.distance_squared(slot),
                members[b].pos.distance_squared(slot),
            );
            da.total_cmp(&db).then(members[a].id.cmp(&members[b].id))
        });
        if let Some(m) = nearest {
            assigned[m] = Some(slot);
        }
    }
    // Members come in the order of `units`; leftovers go to the anchor.
    assigned.into_iter().map(|slot| Some(slot.unwrap_or(anchor))).collect()
}

/// The first `count` points of the integer lattice around `(0, 0)`, nearest first, ties by row (north to south)
/// then column (west to east). Integer keys only, so the order is the same everywhere.
fn lattice(count: usize) -> impl Iterator<Item = (i32, i32)> {
    // Every point within distance `r` lies in the square of radius `r`, and the disc of radius `r` holds at least
    // `count` points once `r * r >= count`; the rest of the square lies farther out.
    let mut r: i32 = 0;
    while ((r * r) as usize) < count {
        r += 1;
    }
    let mut points: Vec<(i32, i32)> = (-r..=r).flat_map(|j| (-r..=r).map(move |i| (i, j))).collect();
    points.sort_by_key(|&(i, j)| (i * i + j * j, j, i));
    points.truncate(count);
    points.into_iter()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_lattice_runs_nearest_first() {
        let points: Vec<(i32, i32)> = lattice(9).collect();
        assert_eq!(
            points,
            [
                (0, 0),
                (0, -1),
                (-1, 0),
                (1, 0),
                (0, 1),
                (-1, -1),
                (1, -1),
                (-1, 1),
                (1, 1)
            ]
        );
        // Far enough out, the square's corners come after the ring beyond it.
        let many: Vec<(i32, i32)> = lattice(200).collect();
        assert_eq!(many.len(), 200);
        let d2 = |(i, j): (i32, i32)| i * i + j * j;
        assert!(many.windows(2).all(|w| d2(w[0]) <= d2(w[1])));
        assert!(many.contains(&(8, 0)) && !many.contains(&(7, 7)));
    }
}
