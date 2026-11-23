//! One unit's move against tiles and units — `docs/movement.md > 6.3`, the rules after the sweep.
//!
//! Every position this module returns passed [`World::is_free`], the exact predicates of invariants I1 and I2
//! (`docs/movement.md > 8`), against the tiles and against every unit at the position it has right now. Units are
//! committed one after another, so no two ever overlap, by construction (`docs/movement.md > 4`). When a candidate
//! fails that test (a rounding corner case), the unit stays where it is, which is free by induction.

use core::f32::consts::FRAC_1_SQRT_2;

use glam::Vec2;

use super::MoveCounters;
use super::space::{Body, Contact, Layer, blocks, overlaps, resting, sweep};
use crate::mover::{AVOID_COMMIT_TICKS, Mover, Side};
use crate::nav::{self, NavGrid};
use crate::units::{SU_PER_GU, Tick};

/// What one unit's move sees: its tiles, its layer's units, itself.
pub(super) struct World<'a> {
    /// The grid of the unit's nav class; `None` for air units, which cross every tile.
    pub grid: Option<&'a NavGrid>,
    /// The grid of the unit's collision layer; `None` for air units, which collide with nothing.
    pub layer: Option<&'a Layer>,
    pub bodies: &'a [Body],
    /// The moving unit's index in `bodies`.
    pub me: usize,
    pub counters: &'a mut MoveCounters,
}

/// The unit that stops a sweep first.
#[derive(Copy, Clone, Debug)]
struct Hit {
    /// Index into `bodies`.
    index: usize,
    contact: Contact,
}

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
    fn body(&self) -> &Body {
        &self.bodies[self.me]
    }

    /// Whether the unit's box can travel from `pos` along `d` without entering a blocked tile or leaving the map.
    fn tiles_free(&mut self, pos: Vec2, d: Vec2) -> bool {
        let Some(grid) = self.grid else {
            return true;
        };
        self.counters.tile_sweeps += 1;
        nav::los(grid, self.body().half, pos, pos + d)
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
        let me = self.body();
        if !pos.is_finite() || self.grid.is_some_and(|grid| !nav::box_clear(grid, me.half, pos)) {
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
    fn clear(&mut self, pos: Vec2, d: Vec2, touched: &mut Vec<usize>) -> bool {
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

    /// `docs/movement.md > 6.3`: one unit's move of `d` from `pos` towards its steering target. `snap` is the
    /// goal when it is within this step: the unit then lands exactly on it. `touched` collects the units that
    /// were in the way (indices into `bodies`).
    ///
    /// 1. Nothing in the way: the whole move.
    /// 2. A tile first: slide, the two axis components one after the other, the larger first, each as far as it
    ///    goes.
    /// 3. A unit moving the same way (`velocity · d > 0`): follow, up to contact, no detour.
    /// 4. A unit standing or oncoming: go around. First a **glance**: if the unit only clips the blocker (the move
    ///    without its contact-axis component keeps at least half of its squared length), it slides along it. Else
    ///    sidestep towards `Mover::avoid_side`: `d` turned 45°, then a step of the same length along the blocker's
    ///    face (`d` turned 90° when it meets the face squarely; boxes are axis-aligned, so "sideways" is along the
    ///    face, and a step that leans into the blocker would never be free). Each is taken only if wholly free. If
    ///    both fail and `avoid_until` has passed, the side switches for at least `AVOID_COMMIT_TICKS` and its two
    ///    are tried. A blocker that itself moves along its face is passed **behind**: once `avoid_until` has
    ///    passed, the side switches (and commits) to the one the blocker comes from.
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
        let stay = |blocker| Moved {
            pos,
            forward: false,
            blocker,
        };
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
            return if self.is_free(to) {
                Moved {
                    pos: to,
                    forward: true,
                    blocker: None,
                }
            } else {
                stay(None)
            };
        };
        touched.push(hit.index);
        let blocker = Some(hit.index);
        let other = self.bodies[hit.index];
        if other.velocity.dot(d) > 0.0 {
            let at = self.up_to(pos, d, hit);
            return Moved {
                pos: at,
                forward: at != pos,
                blocker,
            };
        }
        let axis = hit.contact.axis;
        let mut glance = d;
        glance[axis] = 0.0;
        if glance.length_squared() >= 0.5 * d.length_squared() && self.clear(pos, glance, touched) {
            return Moved {
                pos: pos + glance,
                forward: true,
                blocker,
            };
        }
        // Straight at the blocker's face, as long as `d`: its quarter turn runs along the face.
        let mut at_face = Vec2::ZERO;
        at_face[axis] = d.length().copysign(d[axis]);
        // A blocker moving along its face (crossing traffic, or an oncoming unit already stepping aside) is passed
        // behind: on the side it comes from. Stepping the way it goes would keep the two side by side.
        let across = other.velocity[1 - axis];
        if across != 0.0 && tick >= mover.avoid_until {
            let behind = if turned_90(at_face, Side::Right)[1 - axis] * across < 0.0 {
                Side::Right
            } else {
                Side::Left
            };
            if behind != mover.avoid_side {
                mover.avoid_side = behind;
                mover.avoid_until = tick.saturating_add(AVOID_COMMIT_TICKS);
            }
        }
        let mut switched = false;
        loop {
            let side = mover.avoid_side;
            if self.clear(pos, turned_45(d, side), touched) {
                return Moved {
                    pos: pos + turned_45(d, side),
                    forward: true,
                    blocker,
                };
            }
            if self.clear(pos, turned_90(at_face, side), touched) {
                return Moved {
                    pos: pos + turned_90(at_face, side),
                    forward: false,
                    blocker,
                };
            }
            if switched || tick < mover.avoid_until {
                break;
            }
            mover.avoid_side = side.other();
            mover.avoid_until = tick.saturating_add(AVOID_COMMIT_TICKS);
            switched = true;
        }
        Moved {
            pos: self.up_to(pos, d, hit),
            forward: false,
            blocker,
        }
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
}
