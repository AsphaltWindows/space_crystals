//! The shapes distances are measured between — `systems/combat.md > AttackAttributes > Range Measurement` (design
//! change 0031), `systems/controls/command_behaviors.md > MovingToObject > Proximity` (design change 0032) — and,
//! (M5-V1) the shape a unit whose Silhouette turns collides with.
//!
//! Range, MinRange and proximity are measured **edge to edge**: the shortest distance between two objects' shapes,
//! the gap between them. A unit's shape is its Silhouette (`systems/units.md > Unit > Silhouette`), a structure's
//! or map resource's its footprint.
//!
//! **Oriented Silhouettes** (slice M5-V1, plan decision T78; design change 0033,
//! `systems/units.md > UnitCollision > Silhouette Rotation`: "A unit's Silhouette **rotates with its facing** [...]
//! Collision, crushing, range measurement and proximity all use the rotated shape"). A Silhouette that isn't square
//! (`ObjectDef::silhouette_turns`: the Cults' vehicles) turns with its unit's facing: its length (`w`) lies along
//! the facing, its width (`h`) across it, so "facing east it is wide and flat". A square one stays an axis-aligned
//! rectangle, as it always was: turned, it would differ by at most a few space units, while every golden and every
//! measured crowd would move (T78; the approximation is `docs/movement.md` gap G6). Footprints are whole tiles, and
//! axis-aligned. The geometry, with only + − × ÷ and square roots (the facing's sine and cosine come from
//! `crate::math`, so it is the same on every machine):
//! - **Overlap**: the separating-axis test on the four edge directions of the two rectangles ([`Shape::overlaps`]),
//!   strictly, so touching is free (`docs/movement.md > 3`).
//! - **The gap**: 0 if they overlap, else the least distance from a corner of one to the other ([`Shape::gap`]):
//!   between two convex shapes apart, the nearest points are always a corner and an edge.
//! - **Tile work** (vision, the grids, the footprints a unit keeps off) takes the shape's bounding box
//!   ([`Shape::bounds`], [`Shape::bound_half`]): exact against a wall's straight face, wider at a corner. Only the
//!   blocked tiles a unit may not overlap (invariant I2) are tested against the shape itself, by its bounding box
//!   first ([`Shape::clear_on`], [`Shape::tile_contact`]): a path cuts a corner by less than a turned Silhouette's
//!   box, but not by less than the Silhouette.
//! - **The sweep** of a moving shape against a standing one ([`Shape::sweep`]): the separating-axis test along the
//!   move, for collision (`movement::space`).
//!
//! Two axis-aligned rectangles take the arithmetic they always did, so nothing square moves by a rounding. Everything
//! that measures goes through [`Shape`]'s constructors and methods, and [`Shape::silhouette`] takes the facing. So the
//! promise of the axis-aligned days, "oriented boxes change this module only", held for measurement: range,
//! proximity and the occupied target changed here alone. It doesn't hold for collision, which tested boxes of its
//! own: the movement step (`movement::space`), the invariants (`testing::invariants`), the debug teleport and the
//! spots units are put on or kept off (`sim::gate`, `placement`, `sim::popups`, `sim::producers`) now ask a unit's
//! [`Shape`] too (`docs/movement.md > 3`).

use glam::Vec2;

use crate::defs::ObjectKind;
use crate::map::Map;
use crate::math;
use crate::nav::{self, NavGrid};
use crate::object::{Object, Placement};
use crate::units::{SU_PER_GU, TilePos};

/// An object's shape on the map, space units: a rectangle, axis-aligned or (M5-V1) turned with its unit's facing (see
/// the module docs).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Shape {
    centre: Vec2,
    /// Half its width and height; for an oriented shape half its length, along `axis`, and half its width, across it.
    half: Vec2,
    /// (M5-V1) The direction of its length, a unit vector, if it is oriented: a Silhouette that turns, at its unit's
    /// facing. `None` for an axis-aligned rectangle.
    axis: Option<Vec2>,
}

impl Shape {
    /// The axis-aligned rectangle around `centre` with half extents `half`.
    pub fn rect(centre: Vec2, half: Vec2) -> Self {
        Self {
            centre,
            half,
            axis: None,
        }
    }

    /// (M5-V1) The rectangle around `centre` whose length lies along the unit vector `axis`: `half` is half its length
    /// and half its width.
    pub fn oriented(centre: Vec2, half: Vec2, axis: Vec2) -> Self {
        Self {
            centre,
            half,
            axis: Some(axis),
        }
    }

    /// The Silhouette of a unit of `kind` at `pos`, facing `facing` (radians): (M5-V1) turned with the facing if it
    /// isn't square (`ObjectDef::silhouette_turns`), else axis-aligned. A kind without a Silhouette is a point.
    pub fn silhouette(kind: ObjectKind, pos: Vec2, facing: f32) -> Self {
        let def = kind.def();
        let half = def.half_extents().unwrap_or(Vec2::ZERO);
        if def.silhouette_turns() {
            Self::oriented(pos, half, math::facing_vector(facing))
        } else {
            Self::rect(pos, half)
        }
    }

    /// The footprint of a tile-based object: its whole tiles.
    pub fn footprint(placement: &Placement) -> Self {
        let half = Vec2::new(placement.w as f32, placement.h as f32) * (SU_PER_GU / 2.0);
        Self::rect(placement.origin.corner() + half, half)
    }

    /// `o`'s shape: its footprint if it is tile-based, else its Silhouette where it stands.
    pub fn of(o: &Object) -> Self {
        match &o.placement {
            Some(placement) => Self::footprint(placement),
            None => Self::silhouette(o.kind, o.pos, o.facing),
        }
    }

    /// (M5-H1) `o`'s shape where it counts as being on `map` (`Object::map_pos`): [`Shape::of`], but for a Glider past
    /// the map's edge its Silhouette at the point of the map nearest to it, which is where vision, targeting and the
    /// ranges take it to be ("as if it were at the nearest edge tile", `systems/units.md > Air Collision`).
    pub fn on_map(o: &Object, map: &Map) -> Self {
        match &o.placement {
            Some(placement) => Self::footprint(placement),
            None => Self::silhouette(o.kind, o.map_pos(map), o.facing),
        }
    }

    pub fn centre(&self) -> Vec2 {
        self.centre
    }

    /// (M5-V1) The same shape, with its centre at `centre`.
    pub fn at(&self, centre: Vec2) -> Self {
        Self { centre, ..*self }
    }

    /// (M5-V1) The half extents of the axis-aligned box around the shape ([`bounding_half`]): its own for an
    /// axis-aligned rectangle.
    pub fn bound_half(&self) -> Vec2 {
        match self.axis {
            Some(axis) => bounding_half(self.half, axis),
            None => self.half,
        }
    }

    /// The corners `(lo, hi)` of the axis-aligned box around the shape: for what works on tiles (vision).
    pub fn bounds(&self) -> (Vec2, Vec2) {
        let half = self.bound_half();
        (self.centre - half, self.centre + half)
    }

    /// (M5-V1) Whether the two shapes overlap: strictly, so touching is free (`docs/movement.md > 3`). Two
    /// axis-aligned rectangles by the box test (`paths::boxes_overlap`); else by the separating-axis test on the edge
    /// directions of both, along each of which their halves' projections must reach past the distance between their
    /// centres.
    pub fn overlaps(&self, other: &Shape) -> bool {
        if self.axis.is_none() && other.axis.is_none() {
            let apart = (self.centre - other.centre).abs();
            return apart.x < self.half.x + other.half.x && apart.y < self.half.y + other.half.y;
        }
        let between = other.centre - self.centre;
        (self.axes_with(other).into_iter()).all(|n| between.dot(n).abs() < self.radius(n) + other.radius(n))
    }

    /// `systems/combat.md > Range Measurement`: the shortest distance between this shape and `other`, the gap between
    /// their edges. 0 when they touch or overlap. (M5-V1) With an oriented shape: 0 if they overlap
    /// ([`Shape::overlaps`]), else the least distance from a corner of either to the other ([`Shape::distance_to`]).
    pub fn gap(&self, other: &Shape) -> f32 {
        if self.axis.is_none() && other.axis.is_none() {
            let apart = (self.centre - other.centre).abs() - (self.half + other.half);
            return apart.max(Vec2::ZERO).length();
        }
        if self.overlaps(other) {
            return 0.0;
        }
        let from_mine = self.corners().map(|corner| other.distance_to(corner));
        let from_theirs = other.corners().map(|corner| self.distance_to(corner));
        from_mine.into_iter().chain(from_theirs).fold(f32::INFINITY, f32::min)
    }

    /// `systems/combat.md > Range Measurement` (ground-location targets): the shortest distance from this shape to
    /// `point`; 0 inside it. (M5-V1) An oriented shape measures `point` in its own frame, along its length and across.
    pub fn distance_to(&self, point: Vec2) -> f32 {
        let apart = self.local(point).abs() - self.half;
        apart.max(Vec2::ZERO).length()
    }

    /// (M5-V1) The shape moving along `d` against `other`, which stands (collision, `movement::space::contact`): the
    /// part of `d` travelled when the two first overlap (`0..1`), and the edge direction of the two they meet on, the
    /// one along which their projections come to overlap last. `None` if they never overlap on the way. As for
    /// [`Shape::overlaps`], touching is free: a shape sliding along another, or leaving it, isn't in contact; one
    /// already inside meets it at 0. The separating-axis test along the move: on each edge direction the two overlap
    /// for an interval of the move, and they meet where the last of those intervals begins, if it does before the
    /// first ends (the slab test of `movement::space::sweep`, on four axes).
    #[inline]
    pub fn sweep(&self, d: Vec2, other: &Shape) -> Option<(f32, Vec2)> {
        if d == Vec2::ZERO {
            return None;
        }
        let r = self.centre - other.centre;
        let (mut t_in, mut t_out) = (0.0_f32, 1.0_f32);
        // The axis entered last, and when.
        let mut met: Option<(f32, Vec2)> = None;
        for n in self.axes_with(other) {
            let reach = self.radius(n) + other.radius(n);
            let (r, d) = (r.dot(n), d.dot(n));
            if d == 0.0 {
                if r.abs() >= reach {
                    return None;
                }
                continue;
            }
            let (t0, t1) = ((-reach - r) / d, (reach - r) / d);
            let (near, far) = if t0 < t1 { (t0, t1) } else { (t1, t0) };
            if met.is_none_or(|(entered, _)| near > entered) {
                met = Some((near, n));
            }
            t_in = t_in.max(near);
            t_out = t_out.min(far);
        }
        if t_in >= t_out {
            return None;
        }
        met.map(|(_, n)| (t_in, n))
    }

    /// (M5-V1) Whether the shape overlaps no solid tile of `grid` and lies inside the map: invariant I2's test
    /// (`docs/movement.md > 3`). An axis-aligned rectangle by its box (`nav::box_clear`). An oriented one by its
    /// bounding box first, which settles it where that is clear, else by the separating-axis test against each solid
    /// tile the box overlaps, the tiles off the map among them: exact, so a turned Silhouette is clear of a corner
    /// that its edge clears, as on a path that cuts the corner.
    pub fn clear_on(&self, grid: &NavGrid) -> bool {
        let half = self.bound_half();
        if nav::box_clear(grid, half, self.centre) {
            return true;
        }
        if self.axis.is_none() || !self.centre.is_finite() {
            return false;
        }
        let (first, last) = tiles_overlapped(self.centre - half, self.centre + half);
        (first.y..=last.y).all(|y| {
            (first.x..=last.x).all(|x| {
                let tile = TilePos::new(x, y);
                !grid.is_solid(tile) || !self.overlaps(&Shape::tile(tile))
            })
        })
    }

    /// (M5-V1) The part of `d` (`0..1`) the shape travels before it first enters a solid tile of `grid`, the tiles
    /// off the map among them: the earliest [`Shape::sweep`] against each solid tile its bounding box overlaps on the
    /// way. 0 if it is in one already, `None` if it enters none. Exact, for a shape of any orientation.
    pub fn tile_contact(&self, d: Vec2, grid: &NavGrid) -> Option<f32> {
        if !(self.centre.is_finite() && d.is_finite()) {
            return Some(0.0);
        }
        if d == Vec2::ZERO {
            return (!self.clear_on(grid)).then_some(0.0);
        }
        let half = self.bound_half();
        let (lo, hi) = (self.centre - half, self.centre + half);
        let (first, last) = tiles_overlapped(lo.min(lo + d), hi.max(hi + d));
        let mut earliest: Option<f32> = None;
        for y in first.y..=last.y {
            for x in first.x..=last.x {
                let tile = TilePos::new(x, y);
                if !grid.is_solid(tile) {
                    continue;
                }
                if let Some((t, _)) = self.sweep(d, &Shape::tile(tile)) {
                    earliest = Some(earliest.map_or(t, |e| e.min(t)));
                }
            }
        }
        earliest
    }

    /// (M5-V1) Tile `tile`'s square.
    fn tile(tile: TilePos) -> Self {
        let half = Vec2::splat(SU_PER_GU / 2.0);
        Self::rect(tile.corner() + half, half)
    }

    /// Its edge directions, unit vectors: along its length and across it (`x` and `y` for an axis-aligned one).
    fn edges(&self) -> (Vec2, Vec2) {
        match self.axis {
            Some(u) => (u, Vec2::new(-u.y, u.x)),
            None => (Vec2::X, Vec2::Y),
        }
    }

    /// The four axes of the separating-axis test between it and `other`: the edge directions of both.
    fn axes_with(&self, other: &Shape) -> [Vec2; 4] {
        let ((a, b), (c, d)) = (self.edges(), other.edges());
        [a, b, c, d]
    }

    /// Half its extent along the unit vector `n`: how far its projection on `n` reaches from its centre's.
    fn radius(&self, n: Vec2) -> f32 {
        let (u, v) = self.edges();
        self.half.x * u.dot(n).abs() + self.half.y * v.dot(n).abs()
    }

    /// `point` in its own frame, from its centre: along its length and across it.
    fn local(&self, point: Vec2) -> Vec2 {
        let r = point - self.centre;
        match self.axis {
            Some(u) => Vec2::new(r.dot(u), r.y * u.x - r.x * u.y),
            None => r,
        }
    }

    /// Its four corners.
    fn corners(&self) -> [Vec2; 4] {
        let (u, v) = self.edges();
        let (along, across) = (u * self.half.x, v * self.half.y);
        [
            self.centre + along + across,
            self.centre + along - across,
            self.centre - along - across,
            self.centre - along + across,
        ]
    }
}

/// (M5-V1) The first and the last tile (by row and column) that the open box `lo..hi` overlaps: touching a tile's
/// border isn't overlapping it (as `nav::box_clear` counts them).
fn tiles_overlapped(lo: Vec2, hi: Vec2) -> (TilePos, TilePos) {
    let first = TilePos::containing(lo);
    let last = TilePos::new(
        ((hi.x / SU_PER_GU).ceil() as i32 - 1).max(first.x),
        ((hi.y / SU_PER_GU).ceil() as i32 - 1).max(first.y),
    );
    (first, last)
}

/// (M5-V1) The half extents of the axis-aligned box around a rectangle with half extents `half` whose length lies
/// along the unit vector `axis`: `(|cos| hl + |sin| hw, |sin| hl + |cos| hw)`. Facing along an axis it is `half`
/// itself, on its side `half` swapped; at 45°, a 56x40 Silhouette's is 67.9 su square.
pub fn bounding_half(half: Vec2, axis: Vec2) -> Vec2 {
    let (c, s) = (axis.x.abs(), axis.y.abs());
    Vec2::new(c * half.x + s * half.y, s * half.x + c * half.y)
}

/// `systems/combat.md > AttackAttributes > Range Measurement`: the gap between two shapes ([`Shape::gap`]).
pub fn gap(a: &Shape, b: &Shape) -> f32 {
    a.gap(b)
}

#[cfg(test)]
mod tests;
