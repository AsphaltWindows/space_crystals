//! (M3-G2) The way round a row of standing units — `docs/movement.md > 6.5`, rule 2 (queueing).
//!
//! `systems/units.md > UnitCollision > Ground Collision`: "Moving units must pathfind around occupied space." A
//! unit chasing the target of its attack behind its owner's units that stand attacking that target (`stuck::
//! queues_behind`) has nothing to wait for: they stay where they are until the target is destroyed. The path
//! server can't take it round them. They are in no grid, and its avoid tiles are a tile wide, where the units in
//! the way stand a fraction of a tile from it, often in its own tile: the route round the tile is the route it
//! has. The steering rules (`collide::World::around`) take a unit round one blocker; they keep no memory, so
//! behind a row whose units stand a little out of line (each stopped in the tick its attack reached the target)
//! the unit steps into the pocket between two of them and back, for good.
//!
//! So the unit's path gets a **detour**, straight from the units themselves ([`detour`]): out of the row's box,
//! and round it to an end of the side that faces the target. From there the path goes on as it was planned.

use glam::Vec2;

use super::collide::World;
use super::space::sweep;
use crate::nav;
use crate::tuning::movement::ROW_CLEARANCE_SU;

/// A row is followed through at most this many units (a technical bound on [`detour`]'s walk; a larger crowd is
/// gone round as far as it was followed).
const ROW_MAX: usize = 32;

/// The **row** of `blocker` (an index into `world.bodies`): it, and every unit that stands for good where it is
/// (stationary, and with no goal of its own: in range of its target, arrived, idle) so close to one of the row that
/// the moving unit doesn't fit between the two, on either axis. Units that are only held up, as the unit itself
/// is, aren't of it: they are looking for their own way. In the order found, which is the grid's, so the same on
/// every machine.
fn row(world: &World, blocker: usize) -> Vec<usize> {
    let mut row = vec![blocker];
    let Some(layer) = world.layer else {
        return row;
    };
    let width = world.body().half * 2.0;
    let mut at = 0;
    while at < row.len() && row.len() < ROW_MAX {
        let member = world.bodies[row[at]];
        let reach = member.half + width;
        layer.near(member.pos - reach, member.pos + reach, |index| {
            let index = index as usize;
            let other = &world.bodies[index];
            let stands = other.stationary && other.progress.is_none();
            if index == world.me || !stands || row.len() >= ROW_MAX || row.contains(&index) {
                return;
            }
            let gap = (other.pos - member.pos).abs() - (member.half + other.half);
            if gap.x < width.x && gap.y < width.y {
                row.push(index);
            }
        });
        at += 1;
    }
    row
}

/// One way round the row's box: out of it by a side, then from corner to corner.
struct Route {
    points: Vec<Vec2>,
    length: f32,
    /// The corner it ends at: an index into the box's corners.
    end: usize,
}

/// The detour of the unit at `pos` round the row of `blocker`, the unit in its way, towards `target` (the centre of
/// the object it chases): the waypoints to put in front of its path (`Mover::go_round`), none if there is no way
/// round.
///
/// The row's **box** is the bounding box of its units, grown by the unit's own half extents and
/// [`ROW_CLEARANCE_SU`]: outside it the unit's box is clear of every unit of the row. The unit stands in it, against
/// its blocker. The box's **front** is the side that faces the target, and the detour leads to one of the front's
/// two ends: there the unit is beside the row's end and as near the target as the row is, which is in range if
/// the row is and there is room (the units of the row stopped as soon as they were). It goes
/// 1. straight **out** of the box by a side, and
/// 2. **round** the box from corner to corner, either way, to that end.
///
/// Of the ways that are clear the shortest is taken (among equals: out by the west, east, north, south side, in
/// that order; the front's first end clockwise; clockwise), or, if `other_end`, the shortest to the front's other
/// end: for an attempt after one that didn't get the unit on. A leg is clear if the unit's box, moved along it,
/// stays off blocked tiles (`nav::los`, on the grid the unit collides on: it stands there) and off the row's own
/// units ([`sweep`]): a row bent round the unit shuts the sides it covers. Other units on the way are met as any
/// unit is.
pub(super) fn detour(world: &World, blocker: usize, pos: Vec2, target: Vec2, other_end: bool) -> Vec<Vec2> {
    let me = *world.body();
    let row = row(world, blocker);
    let (mut lo, mut hi) = (Vec2::MAX, Vec2::MIN);
    for &index in &row {
        let body = &world.bodies[index];
        lo = lo.min(body.pos - body.half);
        hi = hi.max(body.pos + body.half);
    }
    let grown = me.half + Vec2::splat(ROW_CLEARANCE_SU);
    let (lo, hi) = (lo - grown, hi + grown);
    // Clockwise on screen from the north-west corner. Side `i` runs from corner `i` to corner `i + 1`: north,
    // east, south, west.
    let corners = [lo, Vec2::new(hi.x, lo.y), hi, Vec2::new(lo.x, hi.y)];
    let leg_clear = |from: Vec2, to: Vec2| {
        from == to
            || (world.grid.is_none_or(|grid| nav::los(grid, me.half, from, to))
                && row.iter().all(|&index| {
                    let body = &world.bodies[index];
                    sweep(from, to - from, body.pos, me.half + body.half).is_none()
                }))
    };
    // The front: the side towards the target, by the larger part of the way to it from the box's middle.
    let to_target = target - (lo + hi) * 0.5;
    let front = match (
        to_target.x.abs() >= to_target.y.abs(),
        to_target.x >= 0.0,
        to_target.y >= 0.0,
    ) {
        (true, true, _) => 1,
        (true, false, _) => 3,
        (false, _, true) => 2,
        (false, _, false) => 0,
    };
    // Where the unit is out of the box by a side: where it stands, if it already is.
    let out = |side: usize| match side {
        0 => Vec2::new(pos.x, pos.y.min(lo.y)),
        1 => Vec2::new(pos.x.max(hi.x), pos.y),
        2 => Vec2::new(pos.x, pos.y.max(hi.y)),
        _ => Vec2::new(pos.x.min(lo.x), pos.y),
    };
    let mut routes: Vec<Route> = Vec::with_capacity(16);
    for side in [3, 1, 0, 2] {
        let out = out(side);
        if !leg_clear(pos, out) {
            continue;
        }
        for end in [front, (front + 1) % 4] {
            for clockwise in [true, false] {
                let mut points = vec![out];
                let mut length = pos.distance(out);
                // Clockwise the next corner is the side's second one, the other way its first.
                let mut corner = if clockwise { (side + 1) % 4 } else { side };
                loop {
                    length += points[points.len() - 1].distance(corners[corner]);
                    points.push(corners[corner]);
                    if corner == end {
                        break;
                    }
                    corner = if clockwise { (corner + 1) % 4 } else { (corner + 3) % 4 };
                }
                routes.push(Route { points, length, end });
            }
        }
    }
    // The shortest first; the sort is stable, so equals stay in the order made.
    routes.sort_by(|a, b| a.length.total_cmp(&b.length));
    let nearest_end = routes.first().map(|route| route.end);
    let wanted = |route: &&Route| (Some(route.end) == nearest_end) != other_end;
    let clear = |route: &&Route| {
        let mut from = pos;
        route.points.iter().all(|&to| {
            let clear = leg_clear(from, to);
            from = to;
            clear
        })
    };
    let found = routes
        .iter()
        .filter(wanted)
        .find(clear)
        .or_else(|| routes.iter().filter(|route| !wanted(route)).find(clear));
    let mut detour = Vec::new();
    let mut from = pos;
    for &point in found.map_or(&[][..], |route| &route.points) {
        if point != from {
            detour.push(point);
            from = point;
        }
    }
    detour
}

#[cfg(test)]
mod tests;
