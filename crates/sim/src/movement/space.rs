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
    /// For friendly yielding (`super::yielding`) and the exceptions of [`blocks`] (crushing spares friendly units).
    pub owner: Option<PlayerId>,
    pub pos: Vec2,
    pub half: Vec2,
    /// Its latest displacement: this tick's if it already moved, else last tick's.
    pub velocity: Vec2,
    /// (G) Where it is heading: the vector from it to its steering waypoint (any length), zero for a unit without
    /// a goal or waiting for its path. Unlike `velocity` it doesn't turn with a step around a blocker, so it tells
    /// an oncoming unit from one that only steps aside (`super::collide::World::lane`, [`Body::heads_against`]).
    pub heading: Vec2,
    /// `Mover::is_stationary`.
    pub stationary: bool,
    /// For a unit on its way (it has a goal): the tick of its latest progress, `Mover::last_progress`. Units
    /// waiting behind it share it (`docs/movement.md > 6.5`).
    pub progress: Option<Tick>,
    /// For a unit whose latest move ended `Arrived`: that move's order group (`Mover::group`).
    pub arrived_with: Option<u32>,
    /// Whether it steps aside for a blocked unit of its owner (`super::yielding::can_yield`, design changes 0022
    /// and 0023).
    pub yields: bool,
    /// Whether it is a Drone on a Gather job (`orders::gather::on_gather_job`, design change 0106): two such
    /// bodies don't collide ([`blocks`]).
    pub gathers: bool,
    /// (M3-H2) Whether those waiting behind it share its `progress` (`super::stuck::leads`): it has got on since
    /// its leg began.
    pub leads: bool,
}

/// `docs/movement.md > 6.1`: whether `other` is an obstacle for `mover`, both of one layer, for the move `mover`
/// makes this tick from where it stands (`mover.pos`). **The single place for collision exceptions.** Every other
/// unit of the layer blocks, friend or enemy, except:
///
/// 1. **Gathering Drones** (`systems/units.md > UnitCollision > Ground Collision > Exception, gathering Drones`,
///    design change 0106): two Drones on a Gather job ([`Body::gathers`]) pass through each other, of any player.
/// 2. **Units that already overlap** (`> Units that already overlap`, design change 0106): two units whose boxes
///    overlap as the mover's turn begins "ignore each other until they have separated; from then on, normal
///    collision applies between them. Nothing is pushed apart." It is read off the two boxes each tick and no pair
///    is remembered: [`overlaps`] is the predicate of invariant I1, so a pair is ignored exactly while I1 would
///    name it. Only rule 1 ever lets two boxes come to overlap, so the pairs this rule covers are those rule 1 left
///    behind (a Drone whose job ended inside another) and they only ever part. While it lasts, either may move
///    freely through the other, deeper in too. An idle one moves off by itself (design change 0126,
///    `super::separate`); nothing moves a unit that has a command.
///
/// A unit ignored for a move is ignored for the whole of it: the sweeps, the lanes and [`overlaps`] at the
/// position it commits all ask here. So a move never brings two units to overlap unless one of the two rules
/// holds between them at its start.
///
/// Crushing (`systems/units.md > UnitCollision > Crushing`) goes here: a moving crusher isn't blocked by a
/// crushable enemy.
pub(crate) fn blocks(mover: &Body, other: &Body) -> bool {
    mover.id != other.id && !(mover.gathers && other.gathers) && !overlaps(mover.pos, mover.half, other)
}

/// One layer's grid (`docs/movement.md > 6.1`): a cell per tile, each unit listed in every cell its box overlaps or
/// touches, and per cell the kinds of unit listed in it.
#[derive(Debug, Default)]
pub(crate) struct Layer {
    width: i32,
    height: i32,
    cells: Vec<Vec<u32>>,
    /// (G) Per cell: what kinds of body were listed in it since the last reset ([`Body::kinds`]). A bit is never
    /// taken back when a body leaves or changes, so it says "maybe", and lets a query skip cells that can't hold
    /// what it looks for ([`Layer::near_any`]).
    kinds: Vec<u16>,
    /// Cells that got an entry since the last reset, to empty only those.
    used: Vec<u32>,
    /// (G) The largest half extent of the bodies listed since the last reset ([`Layer::widest`]).
    widest: f32,
}

/// [`Body::kinds`]: the body steps aside for a blocked unit of its owner (`Body::yields`).
pub(crate) const KIND_YIELDS: u16 = 1 << 8;

/// The middles of the eight 45° sectors of headings, sector `k` from `45k°` to `45k + 45°` (0 is east, clockwise
/// on screen): `(cos, sin)` of `45k + 22.5°`.
const SECTORS: [Vec2; 8] = {
    const C: f32 = 0.923_879_5;
    const S: f32 = 0.382_683_43;
    [
        Vec2::new(C, S),
        Vec2::new(S, C),
        Vec2::new(-S, C),
        Vec2::new(-C, S),
        Vec2::new(-C, -S),
        Vec2::new(-S, -C),
        Vec2::new(S, -C),
        Vec2::new(C, -S),
    ]
};

/// The sector of [`SECTORS`] a heading lies in.
fn sector(heading: Vec2) -> usize {
    let wide = heading.x.abs() > heading.y.abs();
    match (heading.y >= 0.0, heading.x >= 0.0) {
        (true, true) => usize::from(!wide),
        (true, false) => 2 + usize::from(wide),
        (false, false) => 4 + usize::from(!wide),
        (false, true) => 6 + usize::from(wide),
    }
}

/// The kinds ([`Body::kinds`]) of the bodies whose heading may be against `ahead` (a unit vector) by more than the
/// cosine `cos`: every sector that holds such a heading. A heading is within 22.5° of its sector's middle, so the
/// sectors are those whose middle is nearly as much against `ahead`. It errs on the side of "maybe".
pub(crate) fn kinds_against(ahead: Vec2, cos: f32) -> u16 {
    // Two cosines differ by at most the angle between the two directions: 22.5° is 0.393 rad.
    let bound = cos + 0.4;
    let mut kinds = 0;
    for (k, middle) in SECTORS.iter().enumerate() {
        if middle.dot(ahead) < bound {
            kinds |= 1 << k;
        }
    }
    kinds
}

impl Body {
    /// (G) Whether the body's heading is against the unit vector `ahead` by more than the cosine `cos`, which is
    /// negative: the angle between the two is wider than that cosine's. In squares, so without a root.
    pub(crate) fn heads_against(&self, ahead: Vec2, cos: f32) -> bool {
        let along = self.heading.dot(ahead);
        along < 0.0 && along * along > cos * cos * self.heading.length_squared()
    }

    /// (G) What a query may look for in a cell ([`Layer::near_any`]): the sector of the body's heading (bits 0 to 7;
    /// none for a body that isn't heading anywhere) and [`KIND_YIELDS`].
    pub(crate) fn kinds(&self) -> u16 {
        let heading = if self.heading == Vec2::ZERO {
            0
        } else {
            1 << sector(self.heading)
        };
        heading | if self.yields { KIND_YIELDS } else { 0 }
    }
}

impl Layer {
    /// Empties the grid, for a map of `width × height` tiles.
    pub(crate) fn reset(&mut self, width: u32, height: u32) {
        let (width, height) = (width as i32, height as i32);
        self.widest = 0.0;
        if (self.width, self.height) != (width, height) {
            self.width = width;
            self.height = height;
            self.cells = vec![Vec::new(); (width * height) as usize];
            self.kinds = vec![0; (width * height) as usize];
            self.used.clear();
            return;
        }
        for cell in self.used.drain(..) {
            self.cells[cell as usize].clear();
            self.kinds[cell as usize] = 0;
        }
    }

    /// The cells the closed box `lo..=hi` reaches, as inclusive tile ranges, clamped to the map.
    fn span(&self, lo: Vec2, hi: Vec2) -> (i32, i32, i32, i32) {
        // (G) The cast truncates, which is the floor for every tile on the map; left of or above the map both
        // clamp to tile 0. (`floor` is a library call on the baseline x86-64 target, and this is the hot path.)
        let tile = |v: f32, max: i32| ((v / SU_PER_GU) as i32).clamp(0, max - 1);
        (
            tile(lo.x, self.width),
            tile(lo.y, self.height),
            tile(hi.x, self.width),
            tile(hi.y, self.height),
        )
    }

    /// Lists body `index`, of these `kinds` ([`Body::kinds`]), in the cells its box at `pos` reaches.
    pub(crate) fn insert(&mut self, index: u32, pos: Vec2, half: Vec2, kinds: u16) {
        self.widest = self.widest.max(half.max_element());
        let (x0, y0, x1, y1) = self.span(pos - half, pos + half);
        for y in y0..=y1 {
            for x in x0..=x1 {
                let cell = (y * self.width + x) as usize;
                self.cells[cell].push(index);
                self.kinds[cell] |= kinds;
                self.used.push(cell as u32);
            }
        }
    }

    /// (G) The largest half extent, on either axis, of the bodies listed since the last reset: no box reaches
    /// farther than `√2` times this from its centre in any direction.
    pub(crate) fn widest(&self) -> f32 {
        self.widest
    }

    /// (G) Notes that the body listed at `pos` now has these `kinds` ([`Body::kinds`]) too.
    pub(crate) fn mark(&mut self, pos: Vec2, half: Vec2, kinds: u16) {
        if kinds == 0 || self.kinds.is_empty() {
            return;
        }
        let (x0, y0, x1, y1) = self.span(pos - half, pos + half);
        for y in y0..=y1 {
            for x in x0..=x1 {
                self.kinds[(y * self.width + x) as usize] |= kinds;
            }
        }
    }

    /// (G) [`Layer::near`], if a cell the box reaches may list a body of one of `kinds` ([`Body::kinds`]);
    /// otherwise it visits nothing and returns `false`.
    pub(crate) fn near_any(&self, lo: Vec2, hi: Vec2, kinds: u16, mut visit: impl FnMut(u32)) -> bool {
        if self.cells.is_empty() {
            return false;
        }
        let (x0, y0, x1, y1) = self.span(lo, hi);
        if !(y0..=y1).any(|y| (x0..=x1).any(|x| self.kinds[(y * self.width + x) as usize] & kinds != 0)) {
            return false;
        }
        for y in y0..=y1 {
            for x in x0..=x1 {
                for &index in &self.cells[(y * self.width + x) as usize] {
                    visit(index);
                }
            }
        }
        true
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
mod tests;
