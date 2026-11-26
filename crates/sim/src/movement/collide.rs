//! One unit's move against tiles and units — `docs/movement.md > 6.3`: lane seeking ([`World::lane`], in
//! `lanes`), then the rules after the sweep ([`World::resolve`]); of those, going around a standing or oncoming unit
//! is in `contact` ([`World::around`]).
//!
//! Every position this module returns passed [`World::is_free`], the exact predicates of invariants I1 and I2
//! (`docs/movement.md > 8`), against the tiles and against every unit at the position it has right now. Units are
//! committed one after another, so no two ever come to overlap, by construction (`docs/movement.md > 4`), but
//! those `space::blocks` lets through each other (design change 0106). When a candidate
//! fails that test (a rounding corner case), the unit stays where it is, which is free by induction.

mod contact;
mod lanes;
mod straight;

use glam::Vec2;

use super::MoveCounters;
use super::space::{Body, Contact, Layer, blocks, overlaps, resting, sweep};
use crate::mover::{Mover, Side};
use crate::nav::{self, NavGrid};
use crate::production::ExitDoor;
use crate::units::{SU_PER_GU, Tick};

#[cfg(test)]
use contact::turned_45;
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
    /// The exit doorway the moving unit is still in, if it is one a producer released and not yet clear of the
    /// footprint (`Mover::exit`; `core/entities.md > Structure Type > Exit Doorway`, design change 0080; a portal's
    /// energy doorway, design change 0145).
    pub door: Option<ExitDoor>,
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

/// How one unit's tick went.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(super) struct Moved {
    pub pos: Vec2,
    /// The unit got on: it moved freely, slid, followed or stepped diagonally around its blocker. `false` when it
    /// stayed, only closed up to its blocker, or only stepped sideways (`Mover::blocked_ticks` counts these).
    pub forward: bool,
    /// The unit (index into `bodies`) that blocked the way to the steering target, if one did.
    pub blocker: Option<usize>,
    /// (M3-G) The step is one aside: along its blocker's face, or straight across to a lane. It doesn't get the
    /// unit on (`forward` is false), but it is how the unit gets round. A unit with momentum needs many ticks for
    /// it, and isn't blocked while it makes them (`super::follow`).
    pub aside: bool,
    /// (M3-G) The step is a slide along tiles: the way to the steering target ran into a blocked tile or the map's
    /// edge, and the step is what was left of it. A unit with momentum makes the move it asked for itself and
    /// meets the tile (`super::drag`, design change 0035).
    pub slid: bool,
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

    /// **The one collision exception against tiles** (design changes 0080 and 0145): for a unit still in the exit
    /// doorway of the building that released it, whether its box may be at `to`, coming from `from`. That building's
    /// tiles don't block it, every other blocked tile does, and it may only come out of the footprint
    /// (`ExitDoor::allows`). `None` for every other unit: the tiles block it as they are.
    fn door_free(&self, from: Vec2, to: Vec2) -> Option<bool> {
        let door = self.door?;
        let half = self.body().half;
        Some(door.allows(from, to, half) && self.grid.is_none_or(|grid| door.box_clear(grid, half, to)))
    }

    /// Whether the unit's box can travel from `pos` along `d` without entering a blocked tile or leaving the map.
    fn tiles_free(&mut self, pos: Vec2, d: Vec2) -> bool {
        // A step out of a doorway is shorter than the box: where it ends decides.
        if let Some(free) = self.door_free(pos, pos + d) {
            return free;
        }
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
            if other.id == me.id {
                return;
            }
            tests += 1;
            let Some(contact) = sweep(pos, d, other.pos, me.half + other.half) else {
                return;
            };
            // The exceptions last: few of the units near a move are in its way (`blocks` is a box test more).
            if !blocks(&me, other) {
                return;
            }
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
        // The unit in a doorway is measured from where its tick began: never deeper in than that.
        let door = Some(who)
            .filter(|&who| who == self.me)
            .and_then(|_| self.door_free(me.pos, pos));
        if !pos.is_finite() || !door.unwrap_or_else(|| self.grid.is_none_or(clear)) {
            return false;
        }
        let Some(layer) = self.layer else {
            return true;
        };
        let mut free = true;
        layer.near(pos - me.half, pos + me.half, |index| {
            let other = &self.bodies[index as usize];
            free &= !(overlaps(pos, me.half, other) && blocks(me, other));
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
        if let Some(free) = self.door_free(pos, to) {
            // In a doorway the unit takes the step or stays: it doesn't rest against the tiles beside it.
            if !free {
                return pos;
            }
        } else if let Some(grid) = self.grid
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
    #[cfg(test)]
    pub(super) fn resolve(
        &mut self,
        pos: Vec2,
        d: Vec2,
        snap: Option<Vec2>,
        mover: &mut Mover,
        tick: Tick,
        touched: &mut Vec<usize>,
    ) -> Moved {
        self.resolve_for(pos, d, snap, mover, tick, touched, false)
    }

    /// [`World::resolve`] for a unit that goes round a standing unit on the **near side** if `near_side`
    /// ([`World::around`]): the units with momentum (slice M3-G).
    #[expect(
        clippy::too_many_arguments,
        reason = "one unit's move, and how its model goes round a blocker"
    )]
    pub(super) fn resolve_for(
        &mut self,
        pos: Vec2,
        d: Vec2,
        snap: Option<Vec2>,
        mover: &mut Mover,
        tick: Tick,
        touched: &mut Vec<usize>,
        near_side: bool,
    ) -> Moved {
        if !self.tiles_free(pos, d) {
            let first = if d.x.abs() >= d.y.abs() { 0 } else { 1 };
            let mut at = self.along_axis(pos, first, d[first], touched);
            at = self.along_axis(at, 1 - first, d[1 - first], touched);
            return Moved {
                pos: at,
                forward: at != pos,
                blocker: touched.first().copied(),
                aside: false,
                slid: true,
            };
        }
        let Some(hit) = self.first_unit(pos, d) else {
            let to = snap.unwrap_or(pos + d);
            let free = self.is_free(to);
            return Moved {
                pos: if free { to } else { pos },
                forward: free,
                blocker: None,
                aside: false,
                slid: false,
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
                aside: false,
                slid: false,
            };
        }
        if let Some((step, forward)) = self.around(pos, d, hit, mover, tick, touched, near_side) {
            return Moved {
                pos: pos + step,
                forward,
                blocker,
                aside: !forward,
                slid: false,
            };
        }
        Moved {
            pos: self.up_to(pos, d, hit),
            forward: false,
            blocker,
            aside: false,
            slid: false,
        }
    }
}

#[cfg(test)]
mod tests;
