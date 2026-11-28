//! The spatial grids and the box geometry of the movement step — `docs/movement.md > 6.1`, `docs/movement.md > 3`
//! (collision shape). What it holds of a unit is [`Body`] (`space/body.rs`: the two types alone, split out by plan
//! decision T28; their code is here). (M5-V4) Keep the grid's code and the geometry's together here: with the grid
//! split out into a module of its own, the movement step took 8 % longer in `vehicles_perf` (the code as the release
//! build's codegen units place it).
//!
//! Everything here is **scratch**: rebuilt at the start of every phase 5d from the objects, so nothing carries over
//! between ticks and nothing is hashed or saved (`docs/movement.md > 3`).
//!
//! Float rules (`docs/movement.md > 7`): the one overlap predicate is [`overlaps`], strict, so touching is free. The
//! sweep works on the same differences (`a − b` against `half_a + half_b`) as the predicate, so a pair the predicate
//! calls apart is never "already inside" for the sweep. Contact positions are assigned ([`resting`]), not
//! accumulated.
//!
//! **(M5-V1) Oriented Silhouettes** (plan decision T78; `systems/units.md > UnitCollision > Silhouette Rotation`): a
//! body whose Silhouette turns with its facing ([`Body::oriented`]) collides with its turned shape
//! (`shape::Shape`): [`overlaps`] and [`contact`] take the separating-axis test for any pair with such a body in it,
//! "a square box against an oriented one" too, and the box test of two boxes as before. Its [`Body::half`] is the
//! bounding box of its turned shape: what the grids list it under, what the tiles are tested against first (where
//! that box isn't clear of them, the shape itself: `shape::Shape::clear_on`), and what the heuristics that read a box
//! (lanes, the contact step, yielding, a way round a row) see. Its contacts are found by
//! the sweep of the two shapes ([`contact`]); where the sweep's contact rests is the halving of `World::up_to`, not
//! [`resting`], whose exact values are the boxes'.

use glam::Vec2;

use super::crush;
use crate::math;
use crate::paths::boxes_overlap;
use crate::shape::{self, Shape};
use crate::units::SU_PER_GU;

mod body;

pub(crate) use body::{Body, Oriented};

impl Oriented {
    /// A Silhouette with half extents `half` turned to `facing` (radians).
    pub(crate) fn facing(half: Vec2, facing: f32) -> Self {
        Self {
            half,
            facing,
            axis: math::facing_vector(facing),
        }
    }

    /// The half extents of its bounding box (`shape::bounding_half`): [`Body::half`].
    pub(crate) fn bound_half(&self) -> Vec2 {
        shape::bounding_half(self.half, self.axis)
    }
}

/// `docs/movement.md > 6.1`: whether `other` is an obstacle for `mover`, both of one layer, for the move `mover`
/// makes this tick from where it stands (`mover.pos`). **The single place for collision exceptions.** Every other
/// unit of the layer blocks, friend or enemy, except:
///
/// 1. **Gathering Drones** (`systems/units.md > UnitCollision > Ground Collision > Exception, gathering Drones`,
///    design change 0106): two Drones on a Gather job ([`Body::gathers`]) pass through each other, of any player.
/// 2. **Units that already overlap** (`systems/units.md > Units that already overlap`, design change 0106): two units
///    whose boxes overlap as the mover's turn begins "ignore each other until they have separated; from then on, normal
///    collision applies between them. Nothing is pushed apart." It is read off the two boxes each tick and no pair
///    is remembered: [`overlaps`] is the predicate of invariant I1, so a pair is ignored exactly while I1 would
///    name it. Only rule 1 ever lets two boxes come to overlap, so the pairs this rule covers are those rule 1 left
///    behind (a Drone whose job ended inside another) and they only ever part. While it lasts, either may move
///    freely through the other, deeper in too. An idle one moves off by itself (design change 0126,
///    `super::separate`); nothing moves a unit that has a command.
///
/// 3. **(M5-V4) Crushing** (`systems/units.md > UnitCollision > Crushing`, plan decision T86 with design change
///    0418): a crusher whose move this tick crushes ([`Body::crushes`]: Moving, Reversing or Stopping, design change
///    0453, or turning in place)
///    passes through the crushable units of its enemies ([`Body::crushable`]): "crushers may path and drive through
///    crushable enemies — those enemies are not obstacles to them", and a spin's crushable enemies "don't block the
///    spin". Those it overlaps or sweeps die in the same tick (`super::crush`). Every other pair collides as before: a
///    crusher that stands and doesn't turn ("a normal hard obstacle"), own and allied crushables ("No friendly
///    crushing"), and a crushable moving into a crusher ("Crushable units avoid crushers"): only the mover's flag
///    counts.
///
/// A unit ignored for a move is ignored for the whole of it: the sweeps, the lanes and [`overlaps`] at the
/// position it commits all ask here, and (M5-V4) so does a turn's every part (`super::collide::World::turn`). So a
/// move never brings two units to overlap unless one of the rules holds between them at its start, and under rule 3
/// the crushable doesn't outlive the tick.
pub(crate) fn blocks(mover: &Body, other: &Body) -> bool {
    mover.id != other.id
        && !(mover.gathers && other.gathers)
        && !crush::crushes(mover, other)
        && !overlaps(mover, mover.pos, other)
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

/// (M5-V4) [`Body::kinds`]: the body is crushable (`Body::crushable`), for the crush step's query
/// (`super::crush`).
pub(crate) const KIND_CRUSHABLE: u16 = 1 << 9;

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
    /// (M5-V1) Its shape with its centre at `pos`: its Silhouette turned as it lies ([`Body::oriented`]), else its box.
    pub(crate) fn shape_at(&self, pos: Vec2) -> Shape {
        match self.oriented {
            Some(oriented) => Shape::oriented(pos, oriented.half, oriented.axis),
            None => Shape::rect(pos, self.half),
        }
    }

    /// (M5-V1) The body turned to lie as `oriented` (a Silhouette that turns with its facing): its box follows.
    pub(crate) fn turned(&self, oriented: Oriented) -> Self {
        Self {
            half: oriented.bound_half(),
            oriented: Some(oriented),
            ..*self
        }
    }

    /// (G) Whether the body's heading is against the unit vector `ahead` by more than the cosine `cos`, which is
    /// negative: the angle between the two is wider than that cosine's. In squares, so without a root.
    pub(crate) fn heads_against(&self, ahead: Vec2, cos: f32) -> bool {
        let along = self.heading.dot(ahead);
        along < 0.0 && along * along > cos * cos * self.heading.length_squared()
    }

    /// (G) What a query may look for in a cell ([`Layer::near_any`]): the sector of the body's heading (bits 0 to 7;
    /// none for a body that isn't heading anywhere), [`KIND_YIELDS`] and (M5-V4) [`KIND_CRUSHABLE`].
    pub(crate) fn kinds(&self) -> u16 {
        let heading = if self.heading == Vec2::ZERO {
            0
        } else {
            1 << sector(self.heading)
        };
        heading | if self.yields { KIND_YIELDS } else { 0 } | if self.crushable { KIND_CRUSHABLE } else { 0 }
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
    /// The axis the boxes meet on: 0 for x, 1 for y. (M5-V1) For two shapes of which one turns with its facing, the
    /// one nearest to the edge direction they meet on ([`contact`]).
    pub axis: usize,
    /// (M5-V1) The edge direction two shapes meet on, a unit vector, if one of them turns with its facing: the face
    /// they meet on is then no axis. `None` for two boxes.
    pub normal: Option<Vec2>,
}

impl Contact {
    /// The face the two meet on ([`Face`]).
    pub(crate) fn face(&self) -> Face {
        Face {
            axis: self.axis,
            normal: self.normal,
        }
    }
}

/// (M5-V1) The face a moving unit met (`Contact::face`), for the rules that go round it
/// (`super::collide::World::around`): the parts of a vector straight at it and along it. For two boxes the face lies
/// along an axis, and the parts are a vector's coordinates, exactly as those rules always took them; for a shape that
/// turns with its facing, the projections on the edge direction they meet on and on the direction across it.
#[derive(Copy, Clone, Debug)]
pub(crate) struct Face {
    axis: usize,
    normal: Option<Vec2>,
}

impl Face {
    /// The part of `v` straight at the face.
    pub(crate) fn at(&self, v: Vec2) -> f32 {
        match self.normal {
            None => v[self.axis],
            Some(n) => v.dot(n),
        }
    }

    /// The part of `v` along the face.
    pub(crate) fn along(&self, v: Vec2) -> f32 {
        match self.normal {
            None => v[1 - self.axis],
            Some(n) => v.dot(Vec2::new(-n.y, n.x)),
        }
    }

    /// The vector of length `s` straight at the face (signed).
    pub(crate) fn at_by(&self, s: f32) -> Vec2 {
        match self.normal {
            None => {
                let mut v = Vec2::ZERO;
                v[self.axis] = s;
                v
            }
            Some(n) => n * s,
        }
    }

    /// The vector of length `s` along the face (signed).
    pub(crate) fn along_by(&self, s: f32) -> Vec2 {
        match self.normal {
            None => {
                let mut v = Vec2::ZERO;
                v[1 - self.axis] = s;
                v
            }
            Some(n) => Vec2::new(-n.y, n.x) * s,
        }
    }
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
    Some(Contact {
        t: t_in,
        axis,
        normal: None,
    })
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

/// Whether `me`'s box at `pos` overlaps `other`'s where it is (`docs/movement.md > 3`): the predicate of invariant I1.
/// (M5-V1) Their shapes', if either turns with its facing (`Shape::overlaps`, the separating-axis test).
pub(crate) fn overlaps(me: &Body, pos: Vec2, other: &Body) -> bool {
    if me.oriented.is_none() && other.oriented.is_none() {
        return boxes_overlap(pos, me.half, other.pos, other.half);
    }
    me.shape_at(pos).overlaps(&other.shape_at(other.pos))
}

/// Where `me`, moving from `pos` along `d`, first meets `other`, which stands: [`sweep`] for two boxes; (M5-V1) the
/// sweep of their shapes if either turns with its facing (`Shape::sweep`, the separating-axis test along the move),
/// with the edge direction they meet on, the face that the rules going round it read (`Contact::face`), and the one of
/// `x` and `y` nearest to it. `None` if they never overlap on the way; touching is free.
pub(crate) fn contact(me: &Body, pos: Vec2, d: Vec2, other: &Body) -> Option<Contact> {
    if me.oriented.is_none() && other.oriented.is_none() {
        return sweep(pos, d, other.pos, me.half + other.half);
    }
    let (t, normal) = me.shape_at(pos).sweep(d, &other.shape_at(other.pos))?;
    Some(Contact {
        t,
        axis: usize::from(normal.y.abs() > normal.x.abs()),
        normal: Some(normal),
    })
}

#[cfg(test)]
mod tests;
