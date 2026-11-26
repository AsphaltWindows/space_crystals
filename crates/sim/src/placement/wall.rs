//! The wall round a footprint (design change 0339, slice M5-E): where a unit that works at a footprint from outside
//! stands, and the way it walks there round the footprint — `factions/cults/economy.md > Cults Building Mechanics >
//! Group construction rules`: "each Recruit walks to the **part of the footprint closest to it**; if that part is
//! taken (blocked), it **wall-hugs** along the footprint until it reaches a part where it can work, and builds from
//! there (within its 8 su reach of the edge, as for mining)". Geometry alone: what takes a unit, and where it stands,
//! is the caller's (`Sim::step_spots`).
//!
//! - **The wall** ([`Wall`]): the loop a unit's centre follows round a footprint with its box a margin off the
//!   footprint's edge: a rectangle, the footprint grown on each side by half the unit's box and the margin, whose
//!   corners are diagonally off the footprint's. Its points are counted along it from the middle of its east side,
//!   clockwise on screen (+y is south), as slice M5-F1's spots round a footprint were. [`Wall::out`] is the same loop
//!   further out.
//! - **The search** ([`Wall::hug`]): the point of the wall nearest the unit (the part of the footprint closest to it),
//!   then further and further along the wall from there, both ways a step at a time, of two as far the clockwise one
//!   first, until the two ways meet halfway round: the nearest part along the wall either way, the shorter way first.
//! - **The way round** ([`Wall::next_corner`], [`Wall::row_of`]): a unit walks to its point along a wall further out,
//!   by its corners, where its way along the wall turns, the shorter way round, and in from there; never over the
//!   footprint, nor through the units standing at the wall. [`crosses`] tells whether a straight way goes over a
//!   footprint.

use glam::Vec2;

use crate::object::Placement;
use crate::units::SU_PER_GU;

/// The loop round a footprint for a unit of one size (module docs).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Wall {
    /// The footprint's centre.
    centre: Vec2,
    /// Half the loop's rectangle along each axis: half the footprint, half the unit's box and the margin.
    half: Vec2,
}

impl Wall {
    /// The wall round the footprint of `placement` for a unit whose box has half extents `half`, its box `margin` off
    /// the footprint's edge.
    pub fn around(placement: &Placement, half: Vec2, margin: f32) -> Wall {
        let footprint = Vec2::new(placement.w as f32, placement.h as f32) * (SU_PER_GU / 2.0);
        Wall {
            centre: placement.center(),
            half: footprint + half + Vec2::splat(margin),
        }
    }

    /// The same wall `by` further out along each axis.
    pub fn out(self, by: Vec2) -> Wall {
        Wall {
            half: self.half + by,
            ..self
        }
    }

    /// The length of the loop.
    pub fn length(&self) -> f32 {
        4.0 * (self.half.x + self.half.y)
    }

    /// How far along the loop its corners are, clockwise from the south-east one.
    fn corners(&self) -> [f32; 4] {
        let Vec2 { x, y } = self.half;
        [y, y + 2.0 * x, 3.0 * y + 2.0 * x, 3.0 * y + 4.0 * x]
    }

    /// The point `along` along the loop (module docs), taken round it as many times as needed.
    pub fn point(&self, along: f32) -> Vec2 {
        let Vec2 { x, y } = self.half;
        let t = along.rem_euclid(self.length());
        let [south_east, south_west, north_west, north_east] = self.corners();
        let offset = if t < south_east {
            Vec2::new(x, t)
        } else if t < south_west {
            Vec2::new(x - (t - south_east), y)
        } else if t < north_west {
            Vec2::new(-x, y - (t - south_west))
        } else if t < north_east {
            Vec2::new(-x + (t - north_west), -y)
        } else {
            Vec2::new(x, -y + (t - north_east))
        };
        self.centre + offset
    }

    /// How far along the loop ([`Wall::point`]) its point nearest `p` is: outside the loop, `p` brought onto it along
    /// each axis; inside it, onto its nearest side, of two as near the first clockwise from the east.
    pub fn along(&self, p: Vec2) -> f32 {
        let Vec2 { x, y } = self.half;
        let d = p - self.centre;
        let on = if d.x.abs() >= x || d.y.abs() >= y {
            d.clamp(-self.half, self.half)
        } else {
            let gaps = [x - d.x, y - d.y, x + d.x, y + d.y];
            let nearest = (0..4).min_by(|&a, &b| gaps[a].total_cmp(&gaps[b])).unwrap_or(0);
            match nearest {
                0 => Vec2::new(x, d.y),
                1 => Vec2::new(d.x, y),
                2 => Vec2::new(-x, d.y),
                _ => Vec2::new(d.x, -y),
            }
        };
        let [south_east, south_west, north_west, _] = self.corners();
        if on.x == x {
            if on.y >= 0.0 { on.y } else { self.length() + on.y }
        } else if on.y == y {
            south_east + (x - on.x)
        } else if on.x == -x {
            south_west + (y - on.y)
        } else {
            north_west + (on.x + x)
        }
    }

    /// The points of the wall a unit at `from` tries in turn (module docs): the one nearest `from`, then those `step`,
    /// twice `step`, ... further along the loop, clockwise first, then the other way, up to halfway round, the point
    /// halfway round once.
    pub fn hug(self, from: Vec2, step: f32) -> impl Iterator<Item = Vec2> {
        let (start, halfway) = (self.along(from), self.length() / 2.0);
        let steps = if step > 0.0 { (halfway / step) as u32 } else { 0 };
        let ways = (1..=steps).flat_map(move |k| {
            let d = k as f32 * step;
            [Some(d), (d < halfway).then_some(-d)].into_iter().flatten()
        });
        std::iter::once(0.0).chain(ways).map(move |d| self.point(start + d))
    }

    /// The corner of the loop where the way along it from the point nearest `from` to the point nearest `to` turns
    /// first, going the shorter way round, of two as long the clockwise one; `None` if it doesn't turn, both on one
    /// side.
    pub fn next_corner(&self, from: Vec2, to: Vec2) -> Option<Vec2> {
        let length = self.length();
        let (a, b) = (self.along(from), self.along(to));
        let clockwise = (b - a).rem_euclid(length);
        let (way, far) = if clockwise <= length - clockwise {
            (1.0, clockwise)
        } else {
            (-1.0, length - clockwise)
        };
        (self.corners().into_iter())
            .map(|corner| ((way * (corner - a)).rem_euclid(length), corner))
            .filter(|&(d, _)| d > 0.0 && d < far)
            .min_by(|x, y| x.0.total_cmp(&y.0))
            .map(|(_, corner)| self.point(corner))
    }

    /// How many times `step` further out than this wall ([`Wall::out`]) the wall through `p` is, to the nearest; 0 on
    /// this one, or inside it.
    pub fn row_of(&self, p: Vec2, step: Vec2) -> u32 {
        let rows = (((p - self.centre).abs() - self.half) / step).max_element().round();
        if rows.is_finite() { rows.max(0.0) as u32 } else { 0 }
    }

    /// The first `n` such that the wall `n` times `step` further out ([`Wall::out`]) is all off a map `size` space
    /// units wide and high, and so is every one further out: its rectangle reaches the map's farthest edge from the
    /// centre along both axes.
    pub fn off_map(&self, step: Vec2, size: Vec2) -> u32 {
        let farthest = self.centre.max(size - self.centre);
        let n = ((farthest - self.half) / step).ceil().max_element();
        if n.is_finite() { n.max(0.0) as u32 } else { 0 }
    }
}

/// Whether the way straight from `from` to `to` passes through the inside of the rectangle centred on `centre` with
/// half extents `half`: going along its edge, or touching it, is clear.
pub fn crosses(from: Vec2, to: Vec2, centre: Vec2, half: Vec2) -> bool {
    let (low, high) = (centre - half, centre + half);
    let way = to - from;
    let (mut enter, mut leave) = (0.0_f32, 1.0_f32);
    for axis in 0..2 {
        let (p, d) = (from[axis], way[axis]);
        if d == 0.0 {
            if p <= low[axis] || p >= high[axis] {
                return false;
            }
            continue;
        }
        let (a, b) = ((low[axis] - p) / d, (high[axis] - p) / d);
        enter = enter.max(a.min(b));
        leave = leave.min(a.max(b));
        if enter >= leave {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests;
