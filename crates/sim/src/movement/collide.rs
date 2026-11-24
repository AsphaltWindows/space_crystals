//! One unit's move against tiles and units — `docs/movement.md > 6.3`: lane seeking ([`World::lane`], in
//! `lanes`), then the rules after the sweep ([`World::resolve`]).
//!
//! Every position this module returns passed [`World::is_free`], the exact predicates of invariants I1 and I2
//! (`docs/movement.md > 8`), against the tiles and against every unit at the position it has right now. Units are
//! committed one after another, so no two ever overlap, by construction (`docs/movement.md > 4`). When a candidate
//! fails that test (a rounding corner case), the unit stays where it is, which is free by induction.

mod lanes;

use core::f32::consts::FRAC_1_SQRT_2;

use glam::Vec2;

use super::MoveCounters;
use super::space::{Body, Contact, Layer, blocks, overlaps, resting, sweep};
use crate::mover::{AVOID_COMMIT_TICKS, Mover, Side};
use crate::nav::{self, NavGrid};
use crate::units::{SU_PER_GU, Tick};

pub(crate) use lanes::Lane;

/// What one unit's move sees: its tiles, its layer's units, itself.
pub(super) struct World<'a> {
    /// The grid of the unit's nav class; `None` for air units, which cross every tile.
    pub grid: Option<&'a NavGrid>,
    /// The grid of the unit's collision layer; `None` for air units, which collide with nothing.
    pub layer: Option<&'a Layer>,
    pub bodies: &'a [Body],
    /// The moving unit's index in `bodies`.
    pub me: usize,
    /// Scratch of [`World::lane`].
    pub lanes: &'a mut Vec<Lane>,
    pub counters: &'a mut MoveCounters,
}

/// The unit that stops a sweep first.
#[derive(Copy, Clone, Debug)]
struct Hit {
    /// Index into `bodies`.
    index: usize,
    contact: Contact,
}

/// `docs/movement.md > 6.3`, rule 4: from this share of its move along the blocker's face on, a unit goes around on
/// the side it leans to; below it the contact is head-on and `Mover::avoid_side` decides (tuning const).
const LEAN: f32 = 0.3;

/// How one unit's tick went.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(super) struct Moved {
    pub pos: Vec2,
    /// The unit got on: it moved freely, slid, followed or stepped diagonally around its blocker. `false` when it
    /// stayed, only closed up to its blocker, or only stepped sideways (`Mover::blocked_ticks` counts these).
    pub forward: bool,
    /// The unit (index into `bodies`) that blocked the way to the steering target, if one did.
    pub blocker: Option<usize>,
}

/// `(x, y)` turned 45° towards `side`, same length. Constants, no trigonometry (`docs/movement.md > 6.3`).
fn turned_45(d: Vec2, side: Side) -> Vec2 {
    match side {
        Side::Right => Vec2::new(d.x - d.y, d.x + d.y) * FRAC_1_SQRT_2,
        Side::Left => Vec2::new(d.x + d.y, d.y - d.x) * FRAC_1_SQRT_2,
    }
}

/// `(x, y)` turned 90° towards `side`. Right is clockwise on screen (+y is south), so units heading east step south.
fn turned_90(d: Vec2, side: Side) -> Vec2 {
    match side {
        Side::Right => Vec2::new(-d.y, d.x),
        Side::Left => Vec2::new(d.y, -d.x),
    }
}

impl World<'_> {
    pub(super) fn body(&self) -> &Body {
        &self.bodies[self.me]
    }

    /// Whether the unit's box can travel from `pos` along `d` without entering a blocked tile or leaving the map.
    fn tiles_free(&mut self, pos: Vec2, d: Vec2) -> bool {
        let Some(grid) = self.grid else {
            return true;
        };
        self.counters.tile_sweeps += 1;
        // (G) In the open, a short move of a small box can't reach a solid tile.
        let reach = self.body().half.max_element() + d.abs().max_element();
        (reach <= SU_PER_GU && grid.open_around(pos)) || nav::los(grid, self.body().half, pos, pos + d)
    }

    /// The first unit the box meets travelling from `pos` along `d`: the earliest contact, then the lowest id
    /// (`docs/movement.md > 6.3`).
    fn first_unit(&mut self, pos: Vec2, d: Vec2) -> Option<Hit> {
        let layer = self.layer?;
        if d == Vec2::ZERO {
            return None;
        }
        self.counters.unit_sweeps += 1;
        let me = *self.body();
        let (lo, hi) = (pos.min(pos + d) - me.half, pos.max(pos + d) + me.half);
        let mut best: Option<Hit> = None;
        let mut tests = 0;
        layer.near(lo, hi, |index| {
            let other = &self.bodies[index as usize];
            if !blocks(&me, other) {
                return;
            }
            tests += 1;
            let Some(contact) = sweep(pos, d, other.pos, me.half + other.half) else {
                return;
            };
            let earlier = best.is_none_or(|b| {
                contact.t < b.contact.t || (contact.t == b.contact.t && other.id < self.bodies[b.index].id)
            });
            if earlier {
                best = Some(Hit {
                    index: index as usize,
                    contact,
                });
            }
        });
        self.counters.pair_tests += tests;
        best
    }

    /// Whether the unit's box at `pos` overlaps no blocked tile, stays on the map and overlaps no unit: the
    /// predicates of invariants I1 and I2.
    pub(super) fn is_free(&self, pos: Vec2) -> bool {
        self.is_free_for(self.me, pos)
    }

    /// [`World::is_free`] for another unit of the layer, `who` (an index into `bodies`), on the same tiles.
    pub(super) fn is_free_for(&self, who: usize, pos: Vec2) -> bool {
        let me = &self.bodies[who];
        // (G) In the open, a small box is clear of the tiles without a look at them.
        let clear = |grid: &NavGrid| {
            (me.half.max_element() <= SU_PER_GU && grid.open_around(pos)) || nav::box_clear(grid, me.half, pos)
        };
        if !pos.is_finite() || self.grid.is_some_and(|grid| !clear(grid)) {
            return false;
        }
        let Some(layer) = self.layer else {
            return true;
        };
        let mut free = true;
        layer.near(pos - me.half, pos + me.half, |index| {
            let other = &self.bodies[index as usize];
            free &= !(blocks(me, other) && overlaps(pos, me.half, other));
        });
        free
    }

    /// Whether the whole move from `pos` along `d` is free of tiles and units. A unit in the way is noted in
    /// `touched`.
    pub(super) fn clear(&mut self, pos: Vec2, d: Vec2, touched: &mut Vec<usize>) -> bool {
        if !self.tiles_free(pos, d) {
            return false;
        }
        if let Some(hit) = self.first_unit(pos, d) {
            touched.push(hit.index);
            return false;
        }
        self.is_free(pos + d)
    }

    /// `pos` moved along `d` up to the contact with `hit` (`docs/movement.md > 6.3`: advance to contact). On the
    /// contact axis the position is assigned from the blocker's edge. `pos` itself if that spot isn't free.
    fn up_to(&mut self, pos: Vec2, d: Vec2, hit: Hit) -> Vec2 {
        let other = self.bodies[hit.index];
        let axis = hit.contact.axis;
        let mut at = pos + d * hit.contact.t;
        let reach = self.body().half[axis] + other.half[axis];
        let edge = resting(other.pos[axis], reach, d[axis] > 0.0);
        // Never backwards, never beyond the move.
        at[axis] = if d[axis] > 0.0 {
            edge.clamp(pos[axis], pos[axis] + d[axis])
        } else {
            edge.clamp(pos[axis] + d[axis], pos[axis])
        };
        if at != pos && self.tiles_free(pos, at - pos) && self.is_free(at) {
            at
        } else {
            pos
        }
    }

    /// `pos` moved by `amount` along one axis, as far as tiles and units allow (`docs/movement.md > 6.3`, rule 2).
    /// A unit that stops it is noted in `touched`.
    fn along_axis(&mut self, pos: Vec2, axis: usize, amount: f32, touched: &mut Vec<usize>) -> Vec2 {
        if amount == 0.0 {
            return pos;
        }
        let half = self.body().half;
        let mut to = pos;
        to[axis] += amount;
        if let Some(grid) = self.grid
            && !nav::box_clear(grid, half, to)
        {
            // The step is shorter than a tile, so the box runs into the next row or column of tiles: it rests on
            // the border between them. Borders and half extents are whole numbers, so this is exact.
            let border = if amount > 0.0 {
                ((pos[axis] + half[axis]) / SU_PER_GU).ceil() * SU_PER_GU - half[axis]
            } else {
                ((pos[axis] - half[axis]) / SU_PER_GU).floor() * SU_PER_GU + half[axis]
            };
            to[axis] = if amount > 0.0 {
                border.clamp(pos[axis], to[axis])
            } else {
                border.clamp(to[axis], pos[axis])
            };
        }
        if to == pos {
            return pos;
        }
        if let Some(hit) = self.first_unit(pos, to - pos) {
            touched.push(hit.index);
            return self.up_to(pos, to - pos, hit);
        }
        if self.is_free(to) { to } else { pos }
    }

    /// The sidestep of a yielding unit (`docs/movement.md > 6.6`): `pos` moved by `d` if the whole move is free,
    /// else by its two axis components one after the other, the larger first, each as far as tiles and units
    /// allow. The units in the way are noted in `touched`.
    pub(super) fn shuffle(&mut self, pos: Vec2, d: Vec2, touched: &mut Vec<usize>) -> Vec2 {
        if d == Vec2::ZERO {
            return pos;
        }
        if self.clear(pos, d, touched) {
            return pos + d;
        }
        let first = if d.x.abs() >= d.y.abs() { 0 } else { 1 };
        let at = self.along_axis(pos, first, d[first], touched);
        self.along_axis(at, 1 - first, d[1 - first], touched)
    }

    /// `docs/movement.md > 6.3`: one unit's move of `d` from `pos` towards its steering target. `snap` is the
    /// goal when it is within this step: the unit then lands exactly on it. `touched` collects the units that
    /// were in the way (indices into `bodies`).
    ///
    /// 1. Nothing in the way: the whole move.
    /// 2. A tile first: slide, the two axis components one after the other, the larger first, each as far as it
    ///    goes.
    /// 3. A unit moving the same way (`velocity · d > 0`): follow, up to contact, no detour.
    /// 4. A unit standing or oncoming: go around it ([`World::around`]).
    /// 5. All fail: up to contact, or stay.
    pub(super) fn resolve(
        &mut self,
        pos: Vec2,
        d: Vec2,
        snap: Option<Vec2>,
        mover: &mut Mover,
        tick: Tick,
        touched: &mut Vec<usize>,
    ) -> Moved {
        if !self.tiles_free(pos, d) {
            let first = if d.x.abs() >= d.y.abs() { 0 } else { 1 };
            let mut at = self.along_axis(pos, first, d[first], touched);
            at = self.along_axis(at, 1 - first, d[1 - first], touched);
            return Moved {
                pos: at,
                forward: at != pos,
                blocker: touched.first().copied(),
            };
        }
        let Some(hit) = self.first_unit(pos, d) else {
            let to = snap.unwrap_or(pos + d);
            let free = self.is_free(to);
            return Moved {
                pos: if free { to } else { pos },
                forward: free,
                blocker: None,
            };
        };
        touched.push(hit.index);
        let blocker = Some(hit.index);
        if self.bodies[hit.index].velocity.dot(d) > 0.0 {
            let at = self.up_to(pos, d, hit);
            return Moved {
                pos: at,
                forward: at != pos,
                blocker,
            };
        }
        if let Some((step, forward)) = self.around(pos, d, hit, mover, tick, touched) {
            return Moved {
                pos: pos + step,
                forward,
                blocker,
            };
        }
        Moved {
            pos: self.up_to(pos, d, hit),
            forward: false,
            blocker,
        }
    }

    /// `docs/movement.md > 6.3`, rule 4: the step that takes the unit around the standing or oncoming unit of
    /// `hit`, and whether it gets the unit on (`Moved::forward`). `None` if there is none. Every step is taken only
    /// if it is wholly free. Boxes are axis-aligned, so "sideways" is along the blocker's face: a step that leans
    /// into the blocker would never be free.
    ///
    /// The move `d` splits into a part straight at the face and a part along it, its **lean**.
    /// - **Glance** (the lean is at least half of `d`, squared): the unit only clips the blocker and slides along
    ///   it, the lean alone.
    /// - **Leaning** (the lean is at least [`LEAN`] of `d`): the unit goes around on the side it leans to: `d`
    ///   turned 45° that way, else a whole step along the face that way. It never steps back along the face: the
    ///   side it took becomes its `Mover::avoid_side`, so when its own step along the face turns the contact
    ///   head-on, it carries on the same way.
    /// - **Head-on**: the side is `Mover::avoid_side`, Right for everyone at first, so head-on units pass like
    ///   traffic: `d` turned 45°, else a whole step along the face. If both fail and `avoid_until` has passed, the
    ///   side switches for at least `AVOID_COMMIT_TICKS` and its two steps are tried. A blocker that itself moves
    ///   along its face is passed **behind**: once `avoid_until` has passed, the side switches (and commits) to
    ///   the one the blocker comes from; stepping the way it goes would keep the two side by side.
    fn around(
        &mut self,
        pos: Vec2,
        d: Vec2,
        hit: Hit,
        mover: &mut Mover,
        tick: Tick,
        touched: &mut Vec<usize>,
    ) -> Option<(Vec2, bool)> {
        let axis = hit.contact.axis;
        let lean = d[1 - axis];
        let mut glance = Vec2::ZERO;
        glance[1 - axis] = lean;
        if lean * lean >= 0.5 * d.length_squared() && self.clear(pos, glance, touched) {
            return Some((glance, true));
        }
        // Straight at the face, as long as `d`: its quarter turn is a whole step along the face.
        let mut at_face = Vec2::ZERO;
        at_face[axis] = d.length().copysign(d[axis]);
        let towards = |side: Side| turned_90(at_face, side)[1 - axis];
        if lean.abs() >= LEAN * d.length() {
            let side = if towards(Side::Right) * lean > 0.0 {
                Side::Right
            } else {
                Side::Left
            };
            let step = self.sidestep(pos, d, at_face, side, touched);
            if step.is_some() {
                mover.avoid_side = side;
            }
            return step;
        }
        let across = self.bodies[hit.index].velocity[1 - axis];
        if across != 0.0 && tick >= mover.avoid_until {
            let behind = if towards(Side::Right) * across < 0.0 {
                Side::Right
            } else {
                Side::Left
            };
            if behind != mover.avoid_side {
                mover.avoid_side = behind;
                mover.avoid_until = tick.saturating_add(AVOID_COMMIT_TICKS);
            }
        }
        let step = self.sidestep(pos, d, at_face, mover.avoid_side, touched);
        if step.is_some() || tick < mover.avoid_until {
            return step;
        }
        mover.avoid_side = mover.avoid_side.other();
        mover.avoid_until = tick.saturating_add(AVOID_COMMIT_TICKS);
        self.sidestep(pos, d, at_face, mover.avoid_side, touched)
    }

    /// The sidestep to `side`: `d` turned 45°, which gets the unit on, else a whole step along the blocker's face.
    fn sidestep(
        &mut self,
        pos: Vec2,
        d: Vec2,
        at_face: Vec2,
        side: Side,
        touched: &mut Vec<usize>,
    ) -> Option<(Vec2, bool)> {
        [(turned_45(d, side), true), (turned_90(at_face, side), false)]
            .into_iter()
            .find(|&(step, _)| self.clear(pos, step, touched))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn right_is_clockwise_on_screen() {
        let east = Vec2::new(4.0, 0.0);
        assert_eq!(turned_90(east, Side::Right), Vec2::new(0.0, 4.0), "south");
        assert_eq!(turned_90(east, Side::Left), Vec2::new(0.0, -4.0), "north");
        let south_east = turned_45(east, Side::Right);
        assert!(south_east.x > 0.0 && south_east.y > 0.0 && (south_east.length() - 4.0).abs() < 1e-5);
        let north_east = turned_45(east, Side::Left);
        assert!(north_east.x > 0.0 && north_east.y < 0.0 && (north_east.length() - 4.0).abs() < 1e-5);
        // Twice 45° is 90°.
        let twice = turned_45(south_east, Side::Right);
        assert!((twice - turned_90(east, Side::Right)).length() < 1e-5);
    }

    /// `docs/movement.md > 6.3`, rule 4: a unit going round a blocker on the side it leans to keeps that side when
    /// its own step along the face turns the contact head-on, so it never steps back. Pressed against an idle Guard,
    /// a unit whose lean was about [`LEAN`] stepped north as leaning, then south as head-on with `avoid_side` Left,
    /// for good (two groups crossing in the lanes of the magic box found it, `> 6.4`).
    #[test]
    fn a_unit_going_round_on_its_lean_side_never_steps_back() {
        let body = |id: u32, pos: Vec2, half: f32| Body {
            id: crate::ids::ObjectId(id),
            owner: None,
            pos,
            half: Vec2::splat(half),
            velocity: Vec2::ZERO,
            heading: Vec2::ZERO,
            stationary: id == 2,
            progress: None,
            arrived_with: None,
            yields: false,
        };
        // An idle Guard, x 502..538 and y 372..408. A Peacekeeper against its east face heads for a point north-west
        // of it; a head-on contact would make it step south.
        let guard = body(2, Vec2::new(520.0, 390.0), 18.0);
        let target = Vec2::new(416.0, 352.0);
        let mut pos = Vec2::new(550.0, 395.9);
        let mut mover = Mover {
            avoid_side: Side::Left,
            ..Mover::default()
        };
        let (mut layer, mut counters, mut lanes) = (Layer::default(), MoveCounters::default(), Vec::new());
        for tick in 100..130 {
            let bodies = [body(1, pos, 12.0), guard];
            layer.reset(16, 12);
            for (index, b) in bodies.iter().enumerate() {
                layer.insert(index as u32, b.pos, b.half, b.kinds());
            }
            let mut world = World {
                grid: None,
                layer: Some(&layer),
                bodies: &bodies,
                me: 0,
                lanes: &mut lanes,
                counters: &mut counters,
            };
            let d = (target - pos).clamp_length_max(4.0);
            let moved = world.resolve(pos, d, None, &mut mover, tick, &mut Vec::new());
            assert!(moved.pos.y <= pos.y, "tick {tick}: from {pos} back to {}", moved.pos);
            pos = moved.pos;
        }
        // Round the Guard's north-east corner and on west.
        assert!(pos.x < 500.0 && pos.y < 360.0, "{pos}");
    }
}
