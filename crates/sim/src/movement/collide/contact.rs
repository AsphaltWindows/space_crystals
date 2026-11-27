//! Going around a blocker, and the contact step — `docs/movement.md > 6.3`, rule 4; `systems/units.md >
//! UnitCollision > Ground Collision`, "Moving units steer around blockers themselves": how a unit goes around the
//! standing or oncoming unit its sweep ran into ([`World::around`]): which side it takes, and the step to that side.
//!
//! **The contact step** (design change 0323): "for **infantry (LightInfantry, HeavyInfantry), hover vehicles
//! (HoverVehicle, HoverCraft) and TrackedVehicles**, the step is at **90°** to its heading (straight sideways), and
//! its length is **the unit's own width** (the size of its Silhouette across its heading), so it clears the lane it
//! was in." Which step a base takes is data on its def (`defs::ContactStep`, [`World::contact`]). The technical
//! decisions, each the simplest predictable rule (design change 0223):
//! - **Straight sideways is along the blocker's face.** Boxes are axis-aligned: for a unit that meets a face
//!   squarely that is 90° to its heading; for one that leans, the quarter turn of its heading leans into the blocker
//!   on one side and is never free there. The width is the box's extent along that face. (M5-V1) Where a Silhouette
//!   that turns with its facing meets another shape, the face is the edge direction the two meet on, no axis
//!   (`space::Face`): the glance, the lean and the step along the face follow it. The contact step itself stays
//!   along the axis nearest to it, by the extent of the unit's bounding box (`StepAside` keeps an axis): the Hand of
//!   Heaven's, a TrackedVehicle's, for a later slice to turn.
//! - **Over several ticks** (`Mover::step_aside`, `mover::StepAside`): the step is to a coordinate fixed when it
//!   begins, the unit's own width along the face from where it stands. Each tick the unit moves its speed towards
//!   it, the last part exactly onto it, and nothing else: no step forward, no lane seeking, even if its way ahead
//!   comes free meanwhile. It "steps fully sideways by its own width" (0323's Verification).
//! - **It begins only if its first tick's part is wholly free**, as every step of rule 4; else that side is shut,
//!   and the sides are tried as they were: Right first, the other side once `avoid_until` has passed.
//! - **Partly blocked, it ends there**: a tick's part that isn't wholly free is taken as far as tiles and units
//!   allow, and the step is over. From the next tick the unit steers as ever, and a contact begins a new step.
//! - **It is a step aside** (`Moved::aside`), like the single step along the face it replaces: it doesn't get the
//!   unit on, so `Mover::blocked_ticks` counts its ticks (not for a unit with momentum that moves), and the stuck
//!   clock runs. The unit ahead of it, if one is still in its way, stays its blocker for the stuck rules and is
//!   asked to yield.
//! - **A step commits its side** for `AVOID_COMMIT_TICKS` (1 s), as a switch of sides always did: it becomes the
//!   unit's `avoid_side`, and until `avoid_until` the unit takes its next step to the same side, whatever its lean.
//!   A step of a whole width can leave the unit just short of clear of a blocker that stands a little to that side,
//!   leaning back the way it came: by its lean it stepped straight back, to and fro for good (a Guard between two of
//!   its firing line, `tests/attack_behind_friends/`). So the unit goes round on the side it began on, "to its right
//!   first, switching sides after 1 s". A step that began is finished whatever `avoid_until` says.
//! - **At a blocker that stands, the near side**: the side of the blocker's middle the unit is on, `avoid_side`
//!   (Right at first) dead in the middle; the rule slice M3-G made for the units with momentum, for the reason it
//!   gave: seen from the blocker's middle the side never changes as the unit steps, where the lean turns back once
//!   the unit has stepped past its line to the target. At a blocker that moves, the sides are chosen as before:
//!   the lean, `avoid_side` head-on (Right first, like traffic), the pass behind.
//! - **A new path ends it** (`Mover::follow`; a detour round a row, `Mover::go_round`), and so does a new leg or a
//!   stop: the unit has another way to go.
//!
//! A base with `ContactStep::Diagonal` steps as before 0323: `d` turned 45°, else one tick's step along the face.
//!
//! **(M5-V2) Turn and graze** (`ContactStep::Graze`; 0323: "**WheeledVehicles** instead **turn and graze** around the
//! obstacle"): a unit that can't turn in place takes no step aside. Its step is a whole step along the blocker's face,
//! to the side the rules above choose (the near side at a standing blocker), if that is free; the unit's model turns it
//! to head along that step and drives on (`super::super::wheeled`): its side scrapes past the blocker, and a move that
//! meets it is blocked as any (design change 0035), so the unit creeps and keeps turning until it is past. It commits
//! its side for `AVOID_COMMIT_TICKS` from each such step, as a contact step does: a unit that turns by its speed would
//! not get round a blocker by turning one way and then the other.

use core::f32::consts::FRAC_1_SQRT_2;

use glam::Vec2;

use super::{Hit, Moved, World, turned_90};
use crate::defs::ContactStep;
use crate::mover::{Mover, Side, StepAside};
use crate::tuning::movement::{AVOID_COMMIT_TICKS, LEAN};
use crate::units::Tick;

/// `(x, y)` turned 45° towards `side`, same length. Constants, no trigonometry (`docs/movement.md > 6.3`).
pub(super) fn turned_45(d: Vec2, side: Side) -> Vec2 {
    match side {
        Side::Right => Vec2::new(d.x - d.y, d.x + d.y) * FRAC_1_SQRT_2,
        Side::Left => Vec2::new(d.x + d.y, d.y - d.x) * FRAC_1_SQRT_2,
    }
}

/// Where this tick's part of the contact step `aside` takes a unit at `pos` that moves `speed` space units a tick:
/// `speed` nearer to the step's end along its axis, or exactly onto it.
fn part_of(aside: StepAside, pos: Vec2, speed: f32) -> Vec2 {
    let axis = aside.axis();
    let left = aside.to - pos[axis];
    let mut at = pos;
    at[axis] = if left.abs() <= speed {
        aside.to
    } else {
        pos[axis] + speed.copysign(left)
    };
    at
}

impl World<'_> {
    /// `docs/movement.md > 6.3`, rule 4: where the step that takes the unit around the standing or oncoming unit
    /// of `hit` leads, and whether it gets the unit on (`Moved::forward`). `None` if there is none. Every step is
    /// taken only if it is wholly free. Boxes are axis-aligned, so "sideways" is along the blocker's face: a step
    /// that leans into the blocker would never be free. `speed` is how far the unit moves in a tick.
    ///
    /// **The sidestep** below is the base's contact step ([`World::sidestep`]): straight sideways by the unit's own
    /// width, over several ticks (design change 0323, the module docs); for a `ContactStep::Diagonal` base, `d`
    /// turned 45°, else a whole step along the face.
    ///
    /// The move `d` splits into a part straight at the face and a part along it, its **lean**.
    /// - **Glance** (the lean is at least half of `d`, squared): the unit only clips the blocker and slides along
    ///   it, the lean alone.
    /// - **Leaning** (the lean is at least [`LEAN`] of `d`): the unit goes around on the side it leans to, by the
    ///   sidestep that way. It never steps back along the face: the side it took becomes its `Mover::avoid_side`,
    ///   so when its own step along the face turns the contact head-on, it carries on the same way.
    /// - **Head-on**: the side is `Mover::avoid_side`, Right for everyone at first, so head-on units pass like
    ///   traffic: the sidestep that way. If it fails and `avoid_until` has passed, the side switches for at least
    ///   `AVOID_COMMIT_TICKS` and its sidestep is tried. A blocker that itself moves along its face is passed
    ///   **behind**: once `avoid_until` has passed, the side switches (and commits) to the one the blocker comes
    ///   from; stepping the way it goes would keep the two side by side.
    /// - **(M3-G) The near side**, for a unit with momentum (`near_side`), and (M4-W3) for every unit whose base
    ///   takes the contact step (`ContactStep::Sideways`), at a blocker that stands: it goes round
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
        speed: f32,
        hit: Hit,
        mover: &mut Mover,
        tick: Tick,
        touched: &mut Vec<usize>,
        near_side: bool,
    ) -> Option<(Vec2, bool)> {
        // (M5-V1) The face met: an axis for two boxes, else the edge direction of the shapes (`space::Face`).
        let face = hit.contact.face();
        let lean = face.along(d);
        let glance = face.along_by(lean);
        if lean * lean >= 0.5 * d.length_squared() && self.clear(pos, glance, touched) {
            return Some((pos + glance, true));
        }
        // Straight at the face, as long as `d`: its quarter turn is a whole step along the face.
        let at_face = face.at_by(d.length().copysign(face.at(d)));
        let towards = |side: Side| face.along(turned_90(at_face, side));
        let standing = self.bodies[hit.index].velocity == Vec2::ZERO;
        // (0323) A contact step takes as many ticks as a unit with momentum takes over a step aside, and (M5-V2) a
        // grazing unit turns as slowly.
        let near_side = near_side || matches!(self.contact, ContactStep::Sideways | ContactStep::Graze);
        if near_side && standing {
            let offset = face.along(pos) - face.along(self.bodies[hit.index].pos);
            if offset != 0.0 && tick >= mover.avoid_until {
                mover.avoid_side = if towards(Side::Right) * offset > 0.0 {
                    Side::Right
                } else {
                    Side::Left
                };
            }
        } else if lean.abs() >= LEAN * d.length() && !self.keeps_its_side(mover, tick) {
            let side = if towards(Side::Right) * lean > 0.0 {
                Side::Right
            } else {
                Side::Left
            };
            let step = self.sidestep(pos, d, at_face, side, speed, mover, tick, touched);
            if step.is_some() {
                mover.avoid_side = side;
            }
            return step;
        }
        let across = face.along(self.bodies[hit.index].velocity);
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
        let step = self.sidestep(pos, d, at_face, mover.avoid_side, speed, mover, tick, touched);
        if step.is_some() || tick < mover.avoid_until {
            return step;
        }
        mover.avoid_side = mover.avoid_side.other();
        mover.avoid_until = tick.saturating_add(AVOID_COMMIT_TICKS);
        self.sidestep(pos, d, at_face, mover.avoid_side, speed, mover, tick, touched)
    }

    /// Whether the unit is going round on the side it last took a contact step to, whatever its lean (design change
    /// 0323, the module docs): a contact step commits its side for `AVOID_COMMIT_TICKS`, and (M5-V2) so does a graze.
    /// A `ContactStep::Diagonal` base never does: its lean chooses in every tick, as before.
    fn keeps_its_side(&self, mover: &Mover, tick: Tick) -> bool {
        self.contact != ContactStep::Diagonal && tick < mover.avoid_until
    }

    /// The sidestep to `side` of a unit at `pos` whose move `d` met a blocker's face (`at_face`: straight at the
    /// face, as long as `d`), by its base's rule ([`World::contact`]): where it leads this tick, and whether it
    /// gets the unit on. `None` if it isn't free.
    /// - `ContactStep::Sideways` (design change 0323, the module docs): the contact step, straight sideways by
    ///   the unit's own width. This tick's part of it, `speed` long, if that is wholly free; the rest is the
    ///   mover's from now on (`Mover::step_aside`, [`World::step_aside`]).
    /// - `ContactStep::Diagonal`: `d` turned 45°, which gets the unit on, else a whole step along the face.
    /// - (M5-V2) `ContactStep::Graze`: a whole step along the face, its side committed (the module docs).
    #[expect(clippy::too_many_arguments, reason = "one unit's move, and the blocker's face")]
    fn sidestep(
        &mut self,
        pos: Vec2,
        d: Vec2,
        at_face: Vec2,
        side: Side,
        speed: f32,
        mover: &mut Mover,
        tick: Tick,
        touched: &mut Vec<usize>,
    ) -> Option<(Vec2, bool)> {
        let along = turned_90(at_face, side);
        match self.contact {
            ContactStep::Diagonal => [(turned_45(d, side), true), (along, false)]
                .into_iter()
                .find(|&(step, _)| self.clear(pos, step, touched))
                .map(|(step, forward)| (pos + step, forward)),
            ContactStep::Graze => {
                if !self.clear(pos, along, touched) {
                    return None;
                }
                mover.avoid_side = side;
                mover.avoid_until = tick.saturating_add(AVOID_COMMIT_TICKS);
                Some((pos + along, false))
            }
            ContactStep::Sideways => {
                // (M5-V1) Along the axis nearest to the face of a turned shape: a step aside is along an axis.
                let along_y = along.y.abs() > along.x.abs();
                let axis = usize::from(along_y);
                let width = 2.0 * self.body().half[axis];
                let aside = StepAside {
                    along_y,
                    to: pos[axis] + width.copysign(along[axis]),
                };
                let at = part_of(aside, pos, speed);
                if !self.clear_to(pos, at, touched) {
                    return None;
                }
                mover.step_aside = Some(aside);
                mover.avoid_side = side;
                mover.avoid_until = tick.saturating_add(AVOID_COMMIT_TICKS);
                Some((at, false))
            }
        }
    }

    /// Whether the whole move from `pos` to `at` is free ([`World::clear`]), `at` itself included: the sum the
    /// sweep ends on may differ from it by a rounding.
    fn clear_to(&mut self, pos: Vec2, at: Vec2, touched: &mut Vec<usize>) -> bool {
        let d = at - pos;
        self.clear(pos, d, touched) && (pos + d == at || self.is_free(at))
    }

    /// The next part of the contact step the unit at `pos` is taking (`Mover::step_aside`; design change 0323, the
    /// module docs), instead of the move `d` it would make towards its steering target; `None` if it is taking
    /// none. `speed` is how far it moves in a tick.
    /// - This tick's part wholly free: the unit takes it, the last part exactly onto the step's end.
    /// - Not free: it goes as far as tiles and units allow, and the step is over.
    ///
    /// It is a step aside and doesn't get the unit on. The unit still in the way of `d`, if there is one, is its
    /// blocker (else the unit that stopped the step), and is noted in `touched` first. Whether the step is made is
    /// for the caller to note once the unit has moved (`StepAside::is_done`): a unit with momentum doesn't move by
    /// the part it asks for.
    pub(in crate::movement) fn step_aside(
        &mut self,
        pos: Vec2,
        d: Vec2,
        speed: f32,
        mover: &mut Mover,
        touched: &mut Vec<usize>,
    ) -> Option<Moved> {
        let aside = mover.step_aside?;
        let axis = aside.axis();
        if aside.to == pos[axis] || speed <= 0.0 {
            mover.step_aside = None;
            return None;
        }
        if let Some(hit) = self.first_unit(pos, d) {
            touched.push(hit.index);
        }
        let mut at = part_of(aside, pos, speed);
        if !self.clear_to(pos, at, touched) {
            mover.step_aside = None;
            at = self.along_axis(pos, axis, at[axis] - pos[axis], touched);
        }
        Some(Moved {
            pos: at,
            forward: false,
            blocker: touched.first().copied(),
            aside: true,
            slid: false,
        })
    }
}

#[cfg(test)]
mod tests;
