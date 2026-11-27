//! (M5-V1) The turn of a Silhouette that turns with its facing, and the halving of a blocked step — plan decision T78;
//! `systems/units.md > UnitCollision > Collision Response (Un-sticking)`: "Overlap caused by the rotation itself (a
//! corner sweeping into a wall) follows the same principle: the blocked part is discarded and the unit never ends up
//! overlapping. The exact geometric method is a technical choice."
//!
//! - **A turn** ([`World::turn`]) is tried whole, in parts along which its corners move at most `Movement::turn_try_su`
//!   (4 su), so that a corner can't jump more than the tip of a unit or a tile between two tries. The first part that
//!   isn't free (the predicates of invariants I1 and I2, in the turned shape: `World::fits`) is halved
//!   `Movement::step_halvings` times, and the unit keeps the free part of it: it turns up to contact, within a 32nd of
//!   that part. A turn has no contact a sweep could find, as a move has.
//! - **A move** meets a shape that turns where the sweep of the two shapes says (`space::contact`, exact), and if a
//!   rounding leaves that point inside the other, the move up to it is halved the same way ([`World::free_part`]).
//! - A unit whose Silhouette is square turns freely: its box is the same at every facing.
//!
//! A turn that is cut is a blocked move (design change 0035): the unit's speed is zeroed, its stuck clock runs, and
//! the unit that stopped it is its blocker, asked to yield. It doesn't stand and wait: it takes its step in the shape
//! it has (`super::super::follow::creep_on`), for "blocked units don't freeze; they creep and turn out of the
//! contact". The turn is tried before the move, not with it: the halving of one whole step would test only where it
//! ends, and a long unit's corner passes through another's corner in a turn of 20°; the move is then the sweep of the
//! shape the turn left, with every rule a move has (`docs/movement.md > 6.3`).

use glam::Vec2;

use super::World;
use crate::math;
use crate::movement::space::{Body, Oriented, blocks, overlaps};
use crate::ruleset::rules;

/// What became of a turn ([`World::turn`]).
#[derive(Copy, Clone, Debug, PartialEq)]
pub(in crate::movement) struct Turned {
    /// The facing the unit reached, radians.
    pub facing: f32,
    /// Whether something stopped the turn short of the facing it wanted.
    pub cut: bool,
    /// The unit that stopped it (an index into `bodies`), if one did rather than the tiles.
    pub blocker: Option<usize>,
}

impl World<'_> {
    /// The unit at `pos` turns from `from` to `to` (radians), the shorter way, as far as its turned Silhouette stays
    /// free (module docs). Its body ([`World::mine`]) lies at the facing it reaches from then on, for the rest of its
    /// tick. A unit whose Silhouette doesn't turn (`space::Body::oriented`) reaches `to`.
    pub(in crate::movement) fn turn(&mut self, pos: Vec2, from: f32, to: f32) -> Turned {
        let tuning = &rules().tuning.movement;
        let whole = Turned {
            facing: to,
            cut: false,
            blocker: None,
        };
        let Some(oriented) = self.mine.oriented else {
            return whole;
        };
        let total = math::wrap_angle(to - from);
        let lies = |facing: f32| Oriented::facing(oriented.half, facing);
        if total == 0.0 {
            self.mine = self.mine.turned(lies(to));
            return whole;
        }
        // The facing at the part `f` of the turn; the whole of it lands exactly on `to`.
        let at = |f: f32| {
            if f == 1.0 {
                to
            } else {
                math::wrap_angle(from + total * f)
            }
        };
        // Its corners, the farthest of its points, move `total × |half|` along their arcs.
        let parts = (total.abs() * oriented.half.length() / tuning.turn_try_su)
            .ceil()
            .max(1.0) as u32;
        let mut done = 0.0;
        for part in 1..=parts {
            let f = part as f32 / parts as f32;
            if self.lies_free(pos, lies(at(f))) {
                done = f;
                continue;
            }
            // The blocked part, halved: `lo` is always free (where the part began), `hi` never.
            let (mut lo, mut hi) = (done, f);
            for _ in 0..tuning.step_halvings {
                let mid = (lo + hi) * 0.5;
                if self.lies_free(pos, lies(at(mid))) {
                    lo = mid;
                } else {
                    hi = mid;
                }
            }
            let facing = if lo == 0.0 { from } else { at(lo) };
            let blocker = self.stopped_by(pos, lies(at(hi)));
            self.mine = self.mine.turned(lies(facing));
            return Turned {
                facing,
                cut: true,
                blocker,
            };
        }
        self.mine = self.mine.turned(lies(to));
        whole
    }

    /// (M5-V2) The turn of a unit that can't turn in place, cut ([`World::turn`]), made all the same with its blocked
    /// part dropped (design change 0035, `systems/units.md > UnitCollision > Collision Response (Un-sticking)`: "Overlap
    /// caused by the rotation itself (a corner sweeping into a wall) follows the same principle: the blocked part is
    /// discarded"): the unit at `pos` lies at `facing`, backed off along the unit vector `back`, the way it came, by the
    /// least distance that frees its turned Silhouette, at most `most` (the depth the turn's corners sweep). A vehicle
    /// pressed nose-first against a wall so turns, pivoting about the corner that touches it, where a turn about its
    /// centre is cut to nothing by either front corner. Its body lies at `facing` from then on ([`World::mine`]), and
    /// the place it backed off to is returned; `None` if even `most` doesn't free it. Where it ends decides (the
    /// predicates of invariants I1 and I2, `World::fits`): `most` is a few su at most, which jumps no tile or unit.
    pub(in crate::movement) fn back_off(&mut self, pos: Vec2, facing: f32, back: Vec2, most: f32) -> Option<Vec2> {
        let tuning = &rules().tuning.movement;
        let oriented = self.mine.oriented?;
        let lies = Oriented::facing(oriented.half, facing);
        let free = |d: f32| self.lies_free(pos + back * d, lies);
        if !(most > 0.0 && free(most)) {
            return None;
        }
        // The least of it that frees the unit, within a 32nd: `lo` never frees it (the turn there was cut), `hi` does.
        let (mut lo, mut hi) = (0.0, most);
        for _ in 0..tuning.step_halvings {
            let mid = (lo + hi) * 0.5;
            if free(mid) {
                hi = mid;
            } else {
                lo = mid;
            }
        }
        self.mine = self.mine.turned(lies);
        Some(pos + back * hi)
    }

    /// `pos` moved along `d` as far as `t` of it (`0..=1`), if that move is free (the tiles on the way, and where it
    /// ends: [`World::is_free`]); else as far as is free of it, the move halved `Movement::step_halvings` times and the
    /// free part kept (plan decision T78). `pos` itself if none of it is. The move up to a sweep's contact with a shape
    /// that turns (`World::up_to`): the contact is exact, but a rounding may leave its point inside.
    pub(super) fn free_part(&mut self, pos: Vec2, d: Vec2, t: f32) -> Vec2 {
        let tuning = &rules().tuning.movement;
        if t > 0.0 && self.free_to(pos, pos + d * t) {
            return pos + d * t;
        }
        let (mut lo, mut hi) = (0.0, t);
        for _ in 0..tuning.step_halvings {
            let mid = (lo + hi) * 0.5;
            if self.free_to(pos, pos + d * mid) {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        if lo == 0.0 { pos } else { pos + d * lo }
    }

    /// Whether the unit may move from `pos` to `at`: through free tiles, to a free spot.
    fn free_to(&mut self, pos: Vec2, at: Vec2) -> bool {
        at != pos && self.tiles_free(pos, at - pos) && self.is_free(at)
    }

    /// Whether the unit at `pos`, its Silhouette lying as `oriented`, is free ([`World::is_free`] in that shape).
    fn lies_free(&self, pos: Vec2, oriented: Oriented) -> bool {
        self.fits(&self.mine.turned(oriented), self.me, pos)
    }

    /// The unit with the lowest id that the moving unit at `pos`, lying as `oriented`, would overlap and that blocks
    /// it (`space::blocks`): what stopped its turn, if a unit did.
    fn stopped_by(&self, pos: Vec2, oriented: Oriented) -> Option<usize> {
        let layer = self.layer?;
        let (me, start): (Body, &Body) = (self.mine.turned(oriented), &self.bodies[self.me]);
        let mut first: Option<usize> = None;
        layer.near(pos - me.half, pos + me.half, |index| {
            let index = index as usize;
            let other = &self.bodies[index];
            if overlaps(&me, pos, other) && blocks(start, other) {
                first = Some(first.map_or(index, |found| found.min(index)));
            }
        });
        first
    }
}

#[cfg(test)]
mod tests;
