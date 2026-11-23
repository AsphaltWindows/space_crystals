//! The spatial grids and the box geometry of the movement step — `docs/movement.md > 6.1`, `> 3` (collision shape).
//!
//! Everything here is **scratch**: rebuilt at the start of every phase 5d from the objects, so nothing carries over
//! between ticks and nothing is hashed or saved (`docs/movement.md > 3`).
//!
//! Float rules (`docs/movement.md > 7`): the one overlap predicate is [`boxes_overlap`], strict, so touching is
//! free. The sweep works on the same differences (`a − b` against `half_a + half_b`) as the predicate, so a pair the
//! predicate calls apart is never "already inside" for the sweep. Contact positions are assigned
//! ([`resting`]), not accumulated.

use glam::Vec2;

use crate::ids::{ObjectId, PlayerId};
use crate::paths::boxes_overlap;
use crate::units::{SU_PER_GU, Tick};

/// A unit as the collision step sees it: a copy, so the step needs no second borrow of the object map
/// (`docs/movement.md > 4`). Units already moved this tick are listed at their new position, the rest at their old
/// one.
#[derive(Copy, Clone, Debug)]
pub(crate) struct Body {
    pub id: ObjectId,
    /// For the exceptions of [`blocks`] (crushing spares friendly units).
    #[allow(
        dead_code,
        reason = "read by the crushing exception of `blocks`, which a later slice adds"
    )]
    pub owner: Option<PlayerId>,
    pub pos: Vec2,
    pub half: Vec2,
    /// Its latest displacement: this tick's if it already moved, else last tick's.
    pub velocity: Vec2,
    /// `Mover::is_stationary`.
    pub stationary: bool,
    /// For a unit on its way (a goal and a path): the tick of its latest progress, `Mover::last_progress`. Units
    /// waiting behind it share it (`docs/movement.md > 6.5`).
    pub progress: Option<Tick>,
    /// For a unit whose latest move ended `Arrived`: that move's order group (`Mover::group`).
    pub arrived_with: Option<u32>,
}

/// `docs/movement.md > 6.1`: whether `other` is an obstacle for `mover`, both of one layer. **The single place for
/// collision exceptions.** Today every other unit of the layer blocks, friend or enemy.
///
/// Crushing (`systems/units.md > UnitCollision > Crushing`) goes here: a moving crusher isn't blocked by a
/// crushable enemy.
pub(crate) fn blocks(mover: &Body, other: &Body) -> bool {
    mover.id != other.id
}

/// One layer's grid (`docs/movement.md > 6.1`): a cell per tile, each unit listed in every cell its box overlaps or
/// touches.
#[derive(Debug, Default)]
pub(crate) struct Layer {
    width: i32,
    height: i32,
    cells: Vec<Vec<u32>>,
    /// Cells that got an entry since the last reset, to empty only those.
    used: Vec<u32>,
}

impl Layer {
    /// Empties the grid, for a map of `width × height` tiles.
    pub(crate) fn reset(&mut self, width: u32, height: u32) {
        let (width, height) = (width as i32, height as i32);
        if (self.width, self.height) != (width, height) {
            self.width = width;
            self.height = height;
            self.cells = vec![Vec::new(); (width * height) as usize];
            self.used.clear();
            return;
        }
        for cell in self.used.drain(..) {
            self.cells[cell as usize].clear();
        }
    }

    /// The cells the closed box `lo..=hi` reaches, as inclusive tile ranges, clamped to the map.
    fn span(&self, lo: Vec2, hi: Vec2) -> (i32, i32, i32, i32) {
        let tile = |v: f32, max: i32| ((v / SU_PER_GU).floor() as i32).clamp(0, max - 1);
        (
            tile(lo.x, self.width),
            tile(lo.y, self.height),
            tile(hi.x, self.width),
            tile(hi.y, self.height),
        )
    }

    pub(crate) fn insert(&mut self, index: u32, pos: Vec2, half: Vec2) {
        let (x0, y0, x1, y1) = self.span(pos - half, pos + half);
        for y in y0..=y1 {
            for x in x0..=x1 {
                let cell = (y * self.width + x) as usize;
                self.cells[cell].push(index);
                self.used.push(cell as u32);
            }
        }
    }

    pub(crate) fn remove(&mut self, index: u32, pos: Vec2, half: Vec2) {
        let (x0, y0, x1, y1) = self.span(pos - half, pos + half);
        for y in y0..=y1 {
            for x in x0..=x1 {
                let cell = &mut self.cells[(y * self.width + x) as usize];
                if let Some(at) = cell.iter().position(|&i| i == index) {
                    cell.swap_remove(at);
                }
            }
        }
    }

    /// Calls `visit` with every body listed in a cell the closed box `lo..=hi` reaches, in a fixed order. A body
    /// in several of the cells is visited once per cell.
    pub(crate) fn near(&self, lo: Vec2, hi: Vec2, mut visit: impl FnMut(u32)) {
        if self.cells.is_empty() {
            return;
        }
        let (x0, y0, x1, y1) = self.span(lo, hi);
        for y in y0..=y1 {
            for x in x0..=x1 {
                for &index in &self.cells[(y * self.width + x) as usize] {
                    visit(index);
                }
            }
        }
    }
}

/// Where a moving box first meets a standing one.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct Contact {
    /// The part of the displacement travelled before contact, `0..1`.
    pub t: f32,
    /// The axis the boxes meet on: 0 for x, 1 for y.
    pub axis: usize,
}

/// `docs/movement.md > 6.3`: sweeps the box at `pos` along `d` against the standing box at `other`; `reach` is the
/// sum of the two half extents. `None` if they never overlap on the way. Touching is free: a box sliding along
/// another, or leaving it, isn't in contact.
///
/// The slab test of the segment `pos → pos + d` against the box `other ± reach`, on the differences the overlap
/// predicate uses.
pub(crate) fn sweep(pos: Vec2, d: Vec2, other: Vec2, reach: Vec2) -> Option<Contact> {
    let r = pos - other;
    let (mut t_in, mut t_out) = (0.0_f32, 1.0_f32);
    // When each axis is entered; an axis without motion is never "entered".
    let mut entry = [f32::NEG_INFINITY; 2];
    for axis in 0..2 {
        let (r, d, reach) = (r[axis], d[axis], reach[axis]);
        if d == 0.0 {
            if r.abs() >= reach {
                return None;
            }
            continue;
        }
        let (t0, t1) = ((-reach - r) / d, (reach - r) / d);
        let (near, far) = if t0 < t1 { (t0, t1) } else { (t1, t0) };
        t_in = t_in.max(near);
        t_out = t_out.min(far);
        entry[axis] = near;
    }
    if d == Vec2::ZERO || t_in >= t_out {
        return None;
    }
    // The contact axis is the one entered last. When both are entered at once (a corner), it is the one the box
    // moves less along, so that sliding keeps the larger part of the motion.
    let axis = if entry[0] > entry[1] {
        0
    } else if entry[1] > entry[0] {
        1
    } else {
        usize::from(d.y.abs() < d.x.abs())
    };
    Some(Contact { t: t_in, axis })
}

/// The coordinate at which a box rests against another on one axis (`docs/movement.md > 3`: contact positions are
/// assigned): `other ∓ reach`, on the low side of `other` if `low_side`. If rounding leaves that value inside
/// `reach` by the overlap predicate's own arithmetic, it is moved away float by float.
pub(crate) fn resting(other: f32, reach: f32, low_side: bool) -> f32 {
    let mut at = if low_side { other - reach } else { other + reach };
    for _ in 0..4 {
        if (at - other).abs() >= reach {
            break;
        }
        at = if low_side { at.next_down() } else { at.next_up() };
    }
    at
}

/// Whether the box at `pos` overlaps `body`'s (`docs/movement.md > 3`): the predicate of invariant I1.
pub(crate) fn overlaps(pos: Vec2, half: Vec2, body: &Body) -> bool {
    boxes_overlap(pos, half, body.pos, body.half)
}

#[cfg(test)]
mod tests {
    use super::*;

    const REACH: Vec2 = Vec2::splat(24.0);

    fn v(x: f32, y: f32) -> Vec2 {
        Vec2::new(x, y)
    }

    #[test]
    fn a_sweep_finds_the_first_contact_and_its_axis() {
        // 30 su apart in x: 6 su of free travel.
        let hit = sweep(v(0.0, 0.0), v(8.0, 0.0), v(30.0, 0.0), REACH).unwrap();
        assert_eq!((hit.t, hit.axis), (0.75, 0));
        assert_eq!(
            sweep(v(0.0, 0.0), v(6.0, 0.0), v(30.0, 0.0), REACH),
            None,
            "it ends touching"
        );
        // From above, moving down and a little east.
        let hit = sweep(v(0.0, -30.0), v(1.0, 12.0), v(0.0, 0.0), REACH).unwrap();
        assert_eq!((hit.t, hit.axis), (0.5, 1));
    }

    #[test]
    fn touching_boxes_slide_and_part_freely() {
        let (a, b) = (v(0.0, 0.0), v(24.0, 0.0));
        assert_eq!(sweep(a, v(0.0, 5.0), b, REACH), None, "sliding along");
        assert_eq!(sweep(a, v(-5.0, 3.0), b, REACH), None, "leaving");
        assert_eq!(sweep(a, v(5.0, 0.0), b, REACH).map(|c| c.t), Some(0.0), "pushing in");
        assert_eq!(sweep(a, v(0.0, 0.0), b, REACH), None, "standing");
        // Past each other on the other axis.
        assert_eq!(sweep(a, v(50.0, 0.0), v(24.0, 24.0), REACH), None);
    }

    /// A box touching another's corner and moving in on both axes rests on the axis it moves less along.
    #[test]
    fn a_corner_contact_keeps_the_larger_motion() {
        let hit = sweep(v(-24.0, 24.0), v(4.0, -0.5), v(0.0, 0.0), REACH).unwrap();
        assert_eq!((hit.t, hit.axis), (0.0, 1));
        let hit = sweep(v(-24.0, 24.0), v(0.5, -4.0), v(0.0, 0.0), REACH).unwrap();
        assert_eq!((hit.t, hit.axis), (0.0, 0));
    }

    /// The assigned contact never overlaps by the predicate, whatever the rounding.
    #[test]
    fn resting_positions_never_overlap() {
        let mut rng = crate::rng::Rng::from_seed(5);
        for _ in 0..20_000 {
            let other = rng.next_f32() * 16_384.0;
            for reach in [24.0, 30.0, 36.0, 29.7] {
                for low_side in [true, false] {
                    let at = resting(other, reach, low_side);
                    assert!((at - other).abs() >= reach, "{at} rests inside {other} ± {reach}");
                    assert!((at - other).abs() < reach + 0.01);
                }
            }
        }
    }

    #[test]
    fn a_layer_lists_a_body_in_every_cell_it_reaches() {
        let mut layer = Layer::default();
        layer.reset(4, 4);
        let half = Vec2::splat(12.0);
        layer.insert(7, v(64.0, 100.0), half); // straddles tiles (0, 1) and (1, 1)
        let mut seen = Vec::new();
        layer.near(v(0.0, 64.0), v(10.0, 70.0), |i| seen.push(i));
        assert_eq!(seen, [7]);
        seen.clear();
        layer.near(v(130.0, 64.0), v(140.0, 70.0), |i| seen.push(i));
        assert!(seen.is_empty());
        layer.remove(7, v(64.0, 100.0), half);
        layer.insert(7, v(200.0, 200.0), half);
        layer.near(v(0.0, 0.0), v(100.0, 127.0), |i| seen.push(i));
        assert!(seen.is_empty());
        layer.reset(4, 4);
        layer.near(v(0.0, 0.0), v(255.0, 255.0), |i| seen.push(i));
        assert!(seen.is_empty());
    }
}
