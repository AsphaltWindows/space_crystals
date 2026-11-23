//! Line of sight for a box, and path smoothing — `docs/movement.md > 5.4`.
//!
//! Boxes are axis-aligned with half extents `half`, centred on the point (`docs/movement.md > 3`, collision
//! shape). Touching is free, as in unit collision: a box blocks a segment only when the segment enters its
//! interior. Tiles outside the map are solid, so a clear box is also inside the map.

use glam::Vec2;

use super::NavGrid;
use crate::units::{SU_PER_GU, TilePos};

/// Whether the segment from `a` along `d` (to `a + d`) enters the open box `(lo, hi)`. A slab test.
fn segment_enters_box(a: Vec2, d: Vec2, lo: Vec2, hi: Vec2) -> bool {
    let (mut t_in, mut t_out) = (0.0_f32, 1.0_f32);
    for (a, d, lo, hi) in [(a.x, d.x, lo.x, hi.x), (a.y, d.y, lo.y, hi.y)] {
        if d == 0.0 {
            if a <= lo || a >= hi {
                return false;
            }
        } else {
            let (t0, t1) = ((lo - a) / d, (hi - a) / d);
            t_in = t_in.max(t0.min(t1));
            t_out = t_out.min(t0.max(t1));
        }
    }
    t_in < t_out
}

/// How many tiles around a point's tile a box with these half extents can reach into: 1 up to a whole tile.
fn reach(half: Vec2) -> i32 {
    (half.max_element() / SU_PER_GU).ceil().clamp(1.0, 4.0) as i32
}

/// Whether the segment enters solid tile `t`, inflated by `half`.
fn blocked_by(grid: &NavGrid, half: Vec2, t: TilePos, a: Vec2, d: Vec2) -> bool {
    grid.is_solid(t) && segment_enters_box(a, d, t.corner() - half, t.corner() + Vec2::splat(SU_PER_GU) + half)
}

/// `docs/movement.md > 5.4`: whether a box with half extents `half` can move in a straight line from `a` to `b`
/// without entering a solid tile or leaving the map.
///
/// It walks the tiles the segment crosses (Amanatides–Woo) and slab-tests the segment against each solid tile
/// around them, inflated by `half`. Exact for boxes up to one tile; larger ones (up to 4 tiles) widen the
/// neighbourhood. `false` for points that aren't finite or lie off the map.
pub fn los(grid: &NavGrid, half: Vec2, a: Vec2, b: Vec2) -> bool {
    if !a.is_finite() || !b.is_finite() {
        return false;
    }
    let (mut tile, end) = (TilePos::containing(a), TilePos::containing(b));
    if !grid.in_bounds(tile) || !grid.in_bounds(end) {
        return false;
    }
    let d = b - a;
    // The neighbourhood of the first tile in full; after each step only the row or column that is new.
    let r = reach(half);
    for y in tile.y - r..=tile.y + r {
        for x in tile.x - r..=tile.x + r {
            if blocked_by(grid, half, TilePos::new(x, y), a, d) {
                return false;
            }
        }
    }
    // Per axis: the step direction, the segment parameter at the next tile border, and the parameter per tile.
    let axis = |a: f32, d: f32, tile: i32| {
        if d > 0.0 {
            (1, ((tile + 1) as f32 * SU_PER_GU - a) / d, SU_PER_GU / d)
        } else if d < 0.0 {
            (-1, (tile as f32 * SU_PER_GU - a) / d, -SU_PER_GU / d)
        } else {
            (0, f32::INFINITY, f32::INFINITY)
        }
    };
    let (step_x, mut t_x, delta_x) = axis(a.x, d.x, tile.x);
    let (step_y, mut t_y, delta_y) = axis(a.y, d.y, tile.y);
    // Counting the steps per axis ends the walk on the end tile whatever the rounding.
    let (mut left_x, mut left_y) = ((end.x - tile.x).abs(), (end.y - tile.y).abs());
    while left_x + left_y > 0 {
        if left_y == 0 || (left_x > 0 && t_x < t_y) {
            tile.x += step_x;
            t_x += delta_x;
            left_x -= 1;
            let x = tile.x + step_x * r;
            if (tile.y - r..=tile.y + r).any(|y| blocked_by(grid, half, TilePos::new(x, y), a, d)) {
                return false;
            }
        } else {
            tile.y += step_y;
            t_y += delta_y;
            left_y -= 1;
            let y = tile.y + step_y * r;
            if (tile.x - r..=tile.x + r).any(|x| blocked_by(grid, half, TilePos::new(x, y), a, d)) {
                return false;
            }
        }
    }
    true
}

/// Whether the segment from `a` to `b` stays out of every tile of `tiles` (touching a border is staying out). The
/// avoid tiles of a repath (`docs/movement.md > 6.5`) are tested with it: they block the search, so the smoothed
/// path and the steering must not cut through them either.
pub fn segment_avoids(a: Vec2, b: Vec2, tiles: &[TilePos]) -> bool {
    tiles
        .iter()
        .all(|t| !segment_enters_box(a, b - a, t.corner(), t.corner() + Vec2::splat(SU_PER_GU)))
}

/// Whether a box with half extents `half` centred on `p` overlaps no solid tile and lies inside the map.
pub fn box_clear(grid: &NavGrid, half: Vec2, p: Vec2) -> bool {
    if !p.is_finite() {
        return false;
    }
    let (lo, hi) = (p - half, p + half);
    // The tiles the open box overlaps: touching a border isn't overlapping.
    let first = TilePos::containing(lo);
    let last = TilePos::new(
        ((hi.x / SU_PER_GU).ceil() as i32 - 1).max(first.x),
        ((hi.y / SU_PER_GU).ceil() as i32 - 1).max(first.y),
    );
    // A box far off the map isn't worth walking.
    if !grid.in_bounds(first) || !grid.in_bounds(last) {
        return false;
    }
    (first.y..=last.y).all(|y| (first.x..=last.x).all(|x| !grid.is_solid(TilePos::new(x, y))))
}

/// The point nearest to `p` inside passable tile `tile` where the box is clear of the tile's solid neighbours,
/// found by pushing out of them. For boxes over one tile wide, or if pushing fails, the centre of the tile's
/// block when `p` isn't clear.
pub(super) fn clamp_to_clear(grid: &NavGrid, half: Vec2, tile: TilePos, p: Vec2) -> Vec2 {
    let limit = SU_PER_GU / 2.0;
    if half.x <= limit && half.y <= limit {
        let lo = tile.corner() + half;
        let hi = tile.corner() + Vec2::splat(SU_PER_GU) - half;
        let solid = |dx: i32, dy: i32| grid.is_solid(TilePos::new(tile.x + dx, tile.y + dy));
        let mut q = p.clamp(tile.corner(), tile.corner() + Vec2::splat(SU_PER_GU));
        if solid(-1, 0) {
            q.x = q.x.max(lo.x);
        }
        if solid(1, 0) {
            q.x = q.x.min(hi.x);
        }
        if solid(0, -1) {
            q.y = q.y.max(lo.y);
        }
        if solid(0, 1) {
            q.y = q.y.min(hi.y);
        }
        // A solid diagonal neighbour cuts a corner rectangle out of the tile: leave it by the shorter way.
        for (dx, dy) in [(-1, -1), (1, -1), (-1, 1), (1, 1)] {
            if !solid(dx, dy) {
                continue;
            }
            let (edge_x, depth_x) = if dx < 0 { (lo.x, lo.x - q.x) } else { (hi.x, q.x - hi.x) };
            let (edge_y, depth_y) = if dy < 0 { (lo.y, lo.y - q.y) } else { (hi.y, q.y - hi.y) };
            if depth_x > 0.0 && depth_y > 0.0 {
                if depth_x <= depth_y {
                    q.x = edge_x;
                } else {
                    q.y = edge_y;
                }
            }
        }
        if box_clear(grid, half, q) {
            return q;
        }
    } else if box_clear(grid, half, p) {
        return p;
    }
    grid.center(tile)
}

/// `docs/movement.md > 5.4`: turns a tile path into waypoints by greedy string-pulling. The candidates are the
/// exact `start`, the centre of every tile of the path, and the exact `goal`; from each kept waypoint the next
/// one is a later candidate with line of sight. The result starts with `start`, ends with `goal`, and has at
/// least two points.
///
/// "Farthest with line of sight" is found by bisection, not by testing every candidate: the last candidate if
/// it is visible, else a visible one whose successor is not. That keeps a long path at `O(log n)` tests per
/// waypoint. The next candidate is always taken as visible: it is, for a box up to one tile wide that starts
/// clear, because neighbouring tile centres see each other and every clear point of a tile sees its centre.
pub fn smooth(grid: &NavGrid, half: Vec2, start: Vec2, tiles: &[TilePos], goal: Vec2) -> Vec<Vec2> {
    smooth_avoiding(grid, half, start, tiles, goal, &[])
}

/// [`smooth`] for a path searched with avoid tiles (`docs/movement.md > 5.3`): no leg of the result cuts through
/// a tile of `avoid` ([`segment_avoids`]), so the detour the search found survives the string-pulling.
pub fn smooth_avoiding(
    grid: &NavGrid,
    half: Vec2,
    start: Vec2,
    tiles: &[TilePos],
    goal: Vec2,
    avoid: &[TilePos],
) -> Vec<Vec2> {
    let sees = |a: Vec2, b: Vec2| los(grid, half, a, b) && segment_avoids(a, b, avoid);
    let mut points = Vec::with_capacity(tiles.len() + 2);
    points.push(start);
    points.extend(tiles.iter().map(|&t| grid.center(t)));
    points.push(goal);
    let last = points.len() - 1;
    let mut out = vec![start];
    let mut at = 0;
    while at < last {
        let next = if sees(points[at], points[last]) {
            last
        } else {
            let (mut visible, mut hidden) = (at + 1, last);
            while hidden - visible > 1 {
                let mid = visible + (hidden - visible) / 2;
                if sees(points[at], points[mid]) {
                    visible = mid;
                } else {
                    hidden = mid;
                }
            }
            visible
        };
        // The start and the goal may sit exactly on a tile centre: keep each point once.
        if next == last || points[next] != out[out.len() - 1] {
            out.push(points[next]);
        }
        at = next;
    }
    out
}
