//! The contact step — `docs/movement.md > 6.3`, rule 4: how a unit goes around the standing or oncoming unit its
//! sweep ran into ([`World::around`]): which side it takes, and the step to that side.

use core::f32::consts::FRAC_1_SQRT_2;

use glam::Vec2;

use super::{Hit, World, turned_90};
use crate::mover::{Mover, Side};
use crate::tuning::movement::{AVOID_COMMIT_TICKS, LEAN};
use crate::units::Tick;

/// `(x, y)` turned 45° towards `side`, same length. Constants, no trigonometry (`docs/movement.md > 6.3`).
pub(super) fn turned_45(d: Vec2, side: Side) -> Vec2 {
    match side {
        Side::Right => Vec2::new(d.x - d.y, d.x + d.y) * FRAC_1_SQRT_2,
        Side::Left => Vec2::new(d.x + d.y, d.y - d.x) * FRAC_1_SQRT_2,
    }
}

impl World<'_> {
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
    /// - **(M3-G) The near side**, for a unit with momentum (`near_side`) at a blocker that stands: it goes round
    ///   on the side of the blocker's middle it is on, the shorter way, whatever its lean, and `avoid_side` only
    ///   decides when it is dead in the middle. Seen from that side it never changes: a step aside doesn't turn
    ///   it back, as the lean does when the target is close behind the blocker (the target then lies back across
    ///   the face, and the unit stepped to and fro for good). A unit that stops within the tick loses a few steps
    ///   that way; one with momentum loses all its speed at each turn, and never got round. If that side is shut,
    ///   it takes the other and keeps to it for `AVOID_COMMIT_TICKS`, like a head-on unit.
    #[expect(
        clippy::too_many_arguments,
        reason = "one unit's move, and how its model goes round a blocker"
    )]
    pub(super) fn around(
        &mut self,
        pos: Vec2,
        d: Vec2,
        hit: Hit,
        mover: &mut Mover,
        tick: Tick,
        touched: &mut Vec<usize>,
        near_side: bool,
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
        let standing = self.bodies[hit.index].velocity == Vec2::ZERO;
        if near_side && standing {
            let offset = pos[1 - axis] - self.bodies[hit.index].pos[1 - axis];
            if offset != 0.0 && tick >= mover.avoid_until {
                mover.avoid_side = if towards(Side::Right) * offset > 0.0 {
                    Side::Right
                } else {
                    Side::Left
                };
            }
        } else if lean.abs() >= LEAN * d.length() {
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
