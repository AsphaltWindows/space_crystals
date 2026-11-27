//! (G) Lane seeking — `docs/movement.md > 6.3`: before anything else, a unit looks ahead for the units it has to
//! pass (oncoming ones, and idle groups that make way) and steps towards a lane between them, the way pedestrians
//! do ([`World::lane`], [`World::lane_step`]). Deterministic by sorted lanes and fixed cell order; it keeps no
//! state but `Mover::avoid_side`, and writes no position except through [`World::clear`].

use core::f32::consts::FRAC_1_SQRT_2;

use glam::Vec2;

use super::{Moved, World, turned_90};
use crate::ids::ObjectId;
use crate::movement::space::{KIND_YIELDS, blocks, kinds_against};
use crate::movement::yielding::yields_to;
use crate::mover::{Mover, Side};
use crate::ruleset::rules;

/// (G) The lane of a unit ahead, as the unit being moved sees it ([`World::lane`]): the sideways shifts of its
/// own centre, positive to its Right, at which the two would meet, widened by `Movement::lane_margin_su`.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct Lane {
    pub(in crate::movement) lo: f32,
    pub(in crate::movement) hi: f32,
    /// Index into `bodies`.
    pub(in crate::movement) index: u32,
    /// An idle unit that yields to the mover, not an oncoming one.
    pub(in crate::movement) standing: bool,
    /// How far the mover can go straight on before the two would touch; negative beside it.
    pub(in crate::movement) free: f32,
}

impl Lane {
    /// The shift at which the mover would walk straight at the unit's centre, and how far to either side of it
    /// the two boxes would still meet.
    pub(in crate::movement) fn centre_and_reach(&self) -> (f32, f32) {
        let tuning = &rules().tuning.movement;
        (
            (self.lo + self.hi) * 0.5,
            (self.hi - self.lo) * 0.5 - tuning.lane_margin_su,
        )
    }
}

impl World<'_> {
    /// (G) Lane seeking, `docs/movement.md > 6.3`: the sideways shift (positive to the Right of `ahead`, a unit
    /// vector) that takes the unit at `pos` out of the way of the units ahead that it has to pass, the way
    /// pedestrians form lanes. `None` if none is in its way (or only one idle unit, below). Afterwards
    /// `self.lanes` holds the lanes it found.
    ///
    /// It looks `Movement::lane_look` of its lengths ahead (and beside itself), no farther than `to_goal`, the end of
    /// its way: it never has to pass a unit beyond that, so units converging on one point from all sides aren't in each
    /// other's way. `aim` is the object it heads for, which isn't in its way either. Two kinds of unit count:
    /// - **Oncoming units**: their heading (`Body::heading`: where they are going, not their latest step) is
    ///   against `ahead` (`Movement::lane_oncoming_cos`), whether they move or are jammed.
    /// - **Idle units that yield to it** (`yielding::yields_to`, for its order group `group`): a group of them
    ///   parts for it, so it walks between them instead of at one of them.
    ///
    /// Each one's **lane** is the range of shifts at which the two boxes would meet, widened by
    /// `Movement::lane_margin_su`. With the lanes that hang together around the unit:
    /// - **One idle unit alone**: `None`. The unit walks on and asks it to step aside, as before
    ///   (`docs/movement.md > 6.6`).
    /// - **A clear lane within reach** (`Movement::lane_reach` of its widths; around an idle group only
    ///   `Movement::lane_around_idle`): the nearest shift outside every lane. Left and right about equally far
    ///   (`Movement::lane_tie_su`): the side `prefer`, the unit's `Mover::avoid_side`.
    /// - **None within reach** (a dense group ahead): the middle of the widest gap between two neighbouring
    ///   lanes within reach, a nearer gap before a slightly wider one (`Movement::lane_near`), then the side `prefer`.
    ///   The units of two oncoming groups all do this, so the groups mesh like combs; the units on the outside
    ///   find a clear lane and move out, the gaps next to them widen, and the comb spreads until every lane is
    ///   clear.
    ///
    /// Deterministic: the lanes are sorted by their bounds, then by body index (bodies are in id order), and
    /// every decision compares `f32`s made with `+ - *` only.
    pub(in crate::movement) fn lane(
        &mut self,
        pos: Vec2,
        ahead: Vec2,
        to_goal: f32,
        aim: Option<ObjectId>,
        group: Option<u32>,
        prefer: Side,
    ) -> Option<f32> {
        let tuning = &rules().tuning.movement;
        let layer = self.layer?;
        let me = *self.body();
        let right = turned_90(ahead, Side::Right);
        // Half the box's extent along a direction (the box is axis-aligned).
        let extent = |half: Vec2, dir: Vec2| dir.x.abs() * half.x + dir.y.abs() * half.y;
        let (my_along, my_across) = (extent(me.half, ahead), extent(me.half, right));
        // The distances of lane seeking are in the unit's own lengths and widths.
        let (look, reach) = (tuning.lane_look * 2.0 * my_along, tuning.lane_reach * 2.0 * my_across);
        // The corridor from `back` behind the unit to `front` ahead of it, `aside` wide to either side: the box
        // around it.
        let corridor = |back: f32, front: f32, aside: f32| {
            let middle = pos + ahead * ((front - back) * 0.5);
            let half = ahead.abs() * ((front + back) * 0.5) + right.abs() * aside;
            (middle - half, middle + half)
        };
        let bodies = self.bodies;
        let lane_of = |index: u32| {
            let other = &bodies[index as usize];
            let oncoming = other.heads_against(ahead, tuning.lane_oncoming_cos);
            let standing = !oncoming && yields_to(other, me.owner, group);
            if !(oncoming || standing) || aim == Some(other.id) || !blocks(&me, other) {
                return None;
            }
            let to = other.pos - pos;
            let (along, across) = (to.dot(ahead), to.dot(right));
            let reach_along = my_along + extent(other.half, ahead);
            let reach_across = my_across + extent(other.half, right);
            let ahead_of_it = along > -reach_along && along - reach_along < look && along < to_goal;
            (ahead_of_it && across.abs() - reach_across < reach).then_some(Lane {
                lo: across - reach_across - tuning.lane_margin_su,
                hi: across + reach_across + tuning.lane_margin_su,
                index,
                standing,
                free: along - reach_along,
            })
        };
        self.lanes.clear();
        // Most units have nobody to pass. First, the cells' marks say whether a unit of the kinds in question
        // is anywhere near, without a look at the bodies; then the unit's own lane alone is searched for one in
        // its way; only a unit that has one looks at the lanes to either side. A unit in its way has its centre
        // within the reach of the two boxes of its lane, and no box reaches farther than `widest` from its centre
        // in any direction, so the first search misses none that the second would find.
        let kinds = kinds_against(ahead, tuning.lane_oncoming_cos) | KIND_YIELDS;
        let widest = layer.widest() * core::f32::consts::SQRT_2;
        let (lo, hi) = corridor(
            my_along + widest,
            look + my_along + widest,
            my_across + widest + tuning.lane_margin_su,
        );
        let (mut tests, mut in_its_way) = (0, false);
        layer.near_any(lo, hi, kinds, |index| {
            if !in_its_way {
                tests += 1;
                in_its_way = lane_of(index).is_some_and(|lane| lane.lo < 0.0 && lane.hi > 0.0);
            }
        });
        if !in_its_way {
            self.counters.lane_tests += tests;
            return None;
        }
        let lanes = &mut *self.lanes;
        let (lo, hi) = corridor(my_along, look, reach + my_across + my_across);
        layer.near(lo, hi, |index| {
            tests += 1;
            lanes.extend(lane_of(index));
        });
        self.counters.lane_tests += tests;
        if lanes.iter().all(|lane| lane.lo >= 0.0 || lane.hi <= 0.0) {
            return None;
        }
        // A body is listed once per cell it reaches.
        lanes.sort_by(|a, b| a.lo.total_cmp(&b.lo).then(a.index.cmp(&b.index)));
        lanes.dedup_by_key(|lane| lane.index);
        // The lanes that hang together around the unit: where they end on either side.
        let (mut left, mut right_end) = (0.0_f32, 0.0_f32);
        for lane in lanes.iter() {
            if lane.lo < right_end && lane.hi > right_end {
                right_end = lane.hi;
            }
        }
        for _ in 0..lanes.len() {
            let before = left;
            for lane in lanes.iter() {
                if lane.hi > left && lane.lo < left {
                    left = lane.lo;
                }
            }
            if left == before {
                break;
            }
        }
        // One idle unit alone in the way is asked to step aside, as ever (`docs/movement.md > 6.6`).
        let together = || lanes.iter().filter(|lane| lane.hi > left && lane.lo < right_end);
        let all_standing = together().all(|lane| lane.standing);
        if all_standing && together().count() == 1 {
            return None;
        }
        // An idle group makes way, so the unit walks around it only when it is barely in its way.
        let around = if all_standing {
            tuning.lane_around_idle * 2.0 * my_across
        } else {
            reach
        };
        let prefers_right = prefer == Side::Right;
        let (to_right, to_left) = (right_end, -left);
        if to_right.min(to_left) <= around {
            let tie = (to_right - to_left).abs() < tuning.lane_tie_su;
            let go_right = if tie { prefers_right } else { to_right < to_left };
            return Some(if go_right { to_right } else { -to_left });
        }
        // The best gap between neighbouring lanes, without their margins: (score, middle).
        let mut best: Option<(f32, f32)> = None;
        let mut covered = f32::NEG_INFINITY;
        for lane in lanes.iter() {
            let (lo, hi) = (lane.lo + tuning.lane_margin_su, lane.hi - tuning.lane_margin_su);
            if covered != f32::NEG_INFINITY {
                let (width, middle) = (lo - covered, (lo + covered) * 0.5);
                let score = width - tuning.lane_near * middle.abs();
                let better = best.is_none_or(|(best_score, _)| {
                    score > best_score || (score == best_score && (middle > 0.0) == prefers_right)
                });
                if middle.abs() <= reach && better {
                    best = Some((score, middle));
                }
            }
            covered = covered.max(hi);
        }
        best.map(|(_, middle)| middle)
    }

    /// (G) `docs/movement.md > 6.3`, lane seeking: the step of a unit at `pos` that wants to move `d` and has
    /// units ahead to pass ([`World::lane`]; `to_goal` and `aim` as there). Returns the shift to its lane, if it
    /// is among such units, and its step, if one towards that lane is free; without one the rules of
    /// [`World::resolve`] apply.
    /// - Forward and sideways at once, towards its lane: at most 45° off its way, the whole step.
    /// - If that isn't free: sideways alone, towards its lane. Like a step along a blocker's face it doesn't
    ///   get the unit on (`Moved::forward`), and the unit in the way is its blocker.
    ///
    /// The side it steps to becomes its `Mover::avoid_side`, so that a contact right after is taken the same way.
    pub(in crate::movement) fn lane_step(
        &mut self,
        pos: Vec2,
        d: Vec2,
        to_goal: f32,
        aim: Option<ObjectId>,
        mover: &mut Mover,
        touched: &mut Vec<usize>,
    ) -> (Option<f32>, Option<Moved>) {
        let step = d.length();
        if step <= 0.0 {
            return (None, None);
        }
        let ahead = d / step;
        let Some(shift) = self.lane(pos, ahead, to_goal, aim, mover.group, mover.avoid_side) else {
            return (None, None);
        };
        if shift == 0.0 {
            return (Some(shift), None);
        }
        mover.avoid_side = if shift > 0.0 { Side::Right } else { Side::Left };
        let right = turned_90(ahead, Side::Right);
        let across = shift.clamp(-step * FRAC_1_SQRT_2, step * FRAC_1_SQRT_2);
        let along = (step * step - across * across).sqrt();
        let both = ahead * along + right * across;
        if self.clear(pos, both, touched) {
            let moved = Moved {
                pos: pos + both,
                forward: true,
                blocker: None,
                aside: false,
                slid: false,
            };
            return (Some(shift), Some(moved));
        }
        let blocker = touched.first().copied();
        let aside = right * shift.clamp(-step, step);
        let moved = self.clear(pos, aside, touched).then_some(Moved {
            pos: pos + aside,
            forward: false,
            blocker,
            aside: true,
            slid: false,
        });
        (Some(shift), moved)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::movement::MoveCounters;
    use crate::movement::space::{Body, Layer};

    /// A unit as the lane tests need it: a Peacekeeper's box, heading `heading`; `yields` for an idle unit of
    /// player 1.
    fn lane_body(id: u32, pos: Vec2, heading: Vec2, yields: bool) -> Body {
        Body {
            id: crate::ids::ObjectId(id),
            owner: Some(crate::ids::PlayerId(1)),
            pos,
            half: Vec2::splat(12.0),
            oriented: None,
            velocity: heading * 4.0,
            heading,
            stationary: heading == Vec2::ZERO,
            progress: None,
            arrived_with: None,
            yields,
            gathers: false,
            leads: true,
            moved: 0,
            crushes: false,
            crushable: false,
        }
    }

    /// The lane of body 0, which heads east, among `bodies` (`World::lane`), preferring `prefer`.
    fn lane_of(bodies: &[Body], prefer: Side) -> Option<f32> {
        let (mut layer, mut counters, mut lanes) = (Layer::default(), MoveCounters::default(), Vec::new());
        layer.reset(16, 12);
        for (index, b) in bodies.iter().enumerate() {
            layer.insert(index as u32, b.pos, b.half, b.kinds());
        }
        let mut world = World {
            grid: None,
            door: None,
            layer: Some(&layer),
            bodies,
            me: 0,
            contact: crate::defs::ContactStep::Sideways,
            lanes: &mut lanes,
            counters: &mut counters,
            mine: bodies[0],
        };
        world.lane(bodies[0].pos, Vec2::X, f32::INFINITY, None, None, prefer)
    }

    /// (G) `docs/movement.md > 6.3`, lane seeking. East is the mover's way, so its Right is south (+y).
    #[test]
    fn a_unit_seeks_a_lane_among_the_units_it_has_to_pass() {
        let tuning = &rules().tuning.movement;
        let (east, west) = (Vec2::X, -Vec2::X);
        let me = lane_body(1, Vec2::new(300.0, 300.0), east, false);
        let oncoming = |id: u32, dy: f32| lane_body(id, Vec2::new(360.0, 300.0 + dy), west, false);
        let idle = |id: u32, dy: f32| lane_body(id, Vec2::new(360.0, 300.0 + dy), Vec2::ZERO, true);
        // Nobody ahead, or nobody in its way: no lane to seek.
        assert_eq!(lane_of(&[me], Side::Right), None);
        assert_eq!(lane_of(&[me, oncoming(2, 30.0)], Side::Right), None);
        // Units going its way, standing units that won't yield and units behind it don't count.
        let stranger = Body {
            owner: Some(crate::ids::PlayerId(2)),
            ..idle(2, 0.0)
        };
        let behind = lane_body(3, Vec2::new(240.0, 300.0), west, false);
        assert_eq!(
            lane_of(&[me, lane_body(2, Vec2::new(360.0, 300.0), east, false)], Side::Right),
            None
        );
        assert_eq!(lane_of(&[me, stranger, behind], Side::Right), None);
        // Head-on: a clear lane is 24 su and the margin to either side; the side it prefers decides.
        let clear = 24.0 + tuning.lane_margin_su;
        assert_eq!(lane_of(&[me, oncoming(2, 0.0)], Side::Right), Some(clear));
        assert_eq!(lane_of(&[me, oncoming(2, 0.0)], Side::Left), Some(-clear));
        // Off centre: the short way round, whatever it prefers.
        assert_eq!(lane_of(&[me, oncoming(2, 10.0)], Side::Right), Some(10.0 - clear));
        assert_eq!(lane_of(&[me, oncoming(2, -10.0)], Side::Left), Some(clear - 10.0));
        // The object it heads for isn't in its way.
        let (mut layer, mut counters, mut lanes) = (Layer::default(), MoveCounters::default(), Vec::new());
        layer.reset(16, 12);
        let bodies = [me, oncoming(2, 0.0)];
        for (index, b) in bodies.iter().enumerate() {
            layer.insert(index as u32, b.pos, b.half, b.kinds());
        }
        let mut world = World {
            grid: None,
            door: None,
            layer: Some(&layer),
            bodies: &bodies,
            me: 0,
            contact: crate::defs::ContactStep::Sideways,
            lanes: &mut lanes,
            counters: &mut counters,
            mine: bodies[0],
        };
        let pos = me.pos;
        assert_eq!(
            world.lane(pos, east, f32::INFINITY, Some(bodies[1].id), None, Side::Right),
            None
        );
        // Nor is a unit beyond the end of its way.
        assert_eq!(world.lane(pos, east, 50.0, None, None, Side::Right), None);
        assert_eq!(world.lane(pos, east, 70.0, None, None, Side::Right), Some(clear));

        // A row of oncoming units 40 su apart, wider than its reach: no clear lane, so the middle of a gap between
        // two of them; of two equal gaps, the one on the side it prefers.
        let row: Vec<Body> = std::iter::once(me)
            .chain((0..7).map(|i| oncoming(2 + i, 40.0 * (i as f32 - 3.0))))
            .collect();
        assert_eq!(lane_of(&row, Side::Right), Some(20.0));
        assert_eq!(lane_of(&row, Side::Left), Some(-20.0));
        // A wider gap a little farther off wins; a much farther one doesn't (`Movement::lane_near`).
        let mut wider = row.clone();
        for body in &mut wider[1..4] {
            body.pos.y -= 12.0;
        }
        assert_eq!(
            lane_of(&wider, Side::Right),
            Some(-26.0),
            "the gap of 28 su to the north"
        );

        // One idle unit of its owner alone: it walks on and asks it to step aside. A group of them: it walks
        // between them, or around the group if that is near.
        assert_eq!(lane_of(&[me, idle(2, 0.0)], Side::Right), None);
        let group: Vec<Body> = std::iter::once(me)
            .chain((0..7).map(|i| idle(2 + i, 40.0 * (i as f32 - 3.0))))
            .collect();
        assert_eq!(lane_of(&group, Side::Right), Some(20.0));
        let edge: Vec<Body> = [me, idle(2, 0.0), idle(3, -40.0), idle(4, -80.0)].into();
        assert_eq!(lane_of(&edge, Side::Right), Some(clear), "around the end of the group");
    }
}
