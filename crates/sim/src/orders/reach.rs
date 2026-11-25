//! AttackMove's reach rule — `systems/controls/behaviors.md > AttackMovingToLocation > Algorithm` 2, design change
//! 0230 (answering question Q0058; it replaces design change 0204's memory of dropped targets): "an attack-moving
//! unit fights what it can reach without leaving its route or losing sight of it."
//!
//! - **The judgement** ([`judge`]). "An enemy out of weapon range is taken only if the path to a firing position
//!   (the path the unit plans when it takes a target anyway) stays within AttackMoveLeashDistance of PathReference
//!   and keeps the target within the unit's SightRange." The path is the one phase 5c would serve the chase
//!   (`World::chase_path`: on what the unit's owner knows, to the point nearest the target that the unit can stand
//!   on); it is walked in steps of [`SAMPLE_SU`] up to the first point the unit's attack reaches the target from,
//!   the firing position. Every point up to there, the unit's own included, must lie within the leash of the
//!   PathReference and have the target within the unit's SightRange, measured as the scan measures it (edge to
//!   edge, with the ElevationModifier where the unit would stand). A path that ends short of any firing position
//!   (the target on a cliff, across water) is a chase the unit can't finish either, and so is one too long to keep
//!   the target in sight ([`JUDGE_PATH_SIGHTS`], which bounds the search).
//! - **No memory** (0230: "the same enemy, or an identical one swapped in unseen, must get the same response"). A
//!   rejection is kept ([`Judged`], `Leg::rejected`) only to save judging again, and only while nothing the
//!   judgement depends on has changed (the Director's decision under 0230's "the exact triggers and caching are the
//!   Director's call"): the enemy stands within [`REJUDGE_MOVE_SU`] of where it was judged, the unit's owner knows
//!   the same obstacles (`PlayerKnowledge::version`), and the PathReference is the same (`Leg::rejected` is cleared
//!   when it is replaced). Where the unit itself stands is left out, as the design leaves it out ("only when
//!   something relevant changes, i.e. the enemy moves a meaningful distance, or the obstacle knowledge the path used
//!   changes"): a unit that walked on past an enemy it couldn't reach doesn't turn back for it.
//! - **The cost bound**. A unit judges at most [`JUDGE_PATHS_PER_TICK`] paths per tick ([`Reach`]), its candidates
//!   in target-priority order (`travel::march`); the rest wait for later ticks. An enemy the PathReference rules out
//!   by itself ([`near_route`]) costs no path at all.

use glam::Vec2;

use crate::combat::within_range;
use crate::defs::AttackAttrs;
use crate::elevation::{self, ELEVATION_MODIFIER, Elevation};
use crate::nav::Search;
use crate::shape::Shape;
use crate::units::{SU_PER_GU, gu};

use super::ATTACK_MOVE_LEASH_GU;
use super::state::{Judged, Leg};
use super::world::{Unit, View, World};

/// Design change 0230 ("the enemy moves a meaningful distance"): an enemy that has moved this far from where it was
/// judged is judged again, space units. A placeholder (the Director's): 1 grid unit.
pub const REJUDGE_MOVE_SU: f32 = SU_PER_GU;

/// The most paths one unit plans in one tick to judge enemies by the reach rule (the Director's decision under design
/// change 0230, "Performance is a priority here"). Technical: measured in `tests/attack_move_perf.rs`.
pub const JUDGE_PATHS_PER_TICK: u32 = 2;

/// The most rejections one leg keeps (`Leg::rejected`). Technical: beyond it, those of enemies out of the unit's
/// sight go first, then the lowest id; an enemy whose rejection went is judged again, with the same outcome.
pub const MAX_REJECTED: usize = 64;

/// A judged path is searched no farther than this many times the unit's SightRange (with the ElevationModifier), in
/// orthogonal steps: a longer one counts as leaving the target's sight. Technical, the cost bound of one judgement
/// (the Director's call under design change 0230, "Performance is a priority here"): every point of a path the rule
/// accepts lies within the unit's SightRange of the target, so such a path is longer than twice that range only
/// where obstacles within sight of the target wind it about; and the search for a path round a long wall, which
/// the rule rejects at its first steps anyway, stops there instead of flooding the map
/// (`tests/attack_move_perf.rs`).
pub const JUDGE_PATH_SIGHTS: f32 = 2.0;

/// The step in which a judged path is walked, space units. Technical: a quarter of a tile.
const SAMPLE_SU: f32 = SU_PER_GU / 4.0;

/// The work of the reach rule in the latest phase 5a: output for tests and timings, never state.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct ReachCounters {
    /// Paths planned to judge enemies.
    pub paths: u32,
    /// The most one unit planned.
    pub most_per_unit: u32,
    /// Nodes their searches expanded (apart from `nav::PATH_BUDGET`, which phase 5c's requests share).
    pub expansions: u32,
}

/// The scratch of the judgements: the search arrays, the unit's budget for the tick and the counters.
#[derive(Default)]
pub(crate) struct Reach {
    search: Search,
    /// Paths the unit whose behaviour runs has planned this tick.
    planned: u32,
    counters: ReachCounters,
}

impl Reach {
    /// A new phase 5a: its work counts from zero.
    pub(crate) fn begin_tick(&mut self) {
        self.counters = ReachCounters::default();
        self.planned = 0;
    }

    /// The next unit's behaviour runs: its budget is whole.
    pub(crate) fn begin_unit(&mut self) {
        self.planned = 0;
    }

    pub(crate) fn counters(&self) -> ReachCounters {
        self.counters
    }
}

/// What [`judge`] makes of a chase.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(super) enum Verdict {
    /// The unit can finish the chase: it takes the target.
    Reachable,
    /// It can't: the target is rejected (`Leg::reject`).
    Unreachable,
    /// Not judged this tick: the unit's budget is spent, or it has no path to judge by (it stands where no path
    /// starts, in a doorway, say). Nothing is kept; it is judged in a later tick.
    Later,
}

impl Judged {
    /// The judgement of `target` as the unit's owner knows the obstacles now (`version`).
    pub(super) fn of(target: &View, version: u32) -> Self {
        Self {
            target: target.id,
            at: target.pos,
            knowledge: version,
        }
    }

    /// Whether the judgement still stands for `target` (module docs).
    pub(super) fn stands(&self, target: &View, version: u32) -> bool {
        self.target == target.id && self.knowledge == version && self.at.distance(target.pos) < REJUDGE_MOVE_SU
    }
}

impl Leg {
    /// Whether a rejection of `target` stands: it is left alone, out of weapon range, without judging it again.
    pub(super) fn rejects(&self, target: &View, version: u32) -> bool {
        self.rejected
            .binary_search_by_key(&target.id, |judged| judged.target)
            .is_ok_and(|i| self.rejected[i].stands(target, version))
    }

    /// The unit can't finish a chase of the enemy of `judged` (module docs). `in_sight` says whether an enemy is in
    /// the unit's sight now, for the cap ([`MAX_REJECTED`]).
    pub(super) fn reject(&mut self, judged: Judged, in_sight: impl Fn(&Judged) -> bool) {
        match self.rejected.binary_search_by_key(&judged.target, |j| j.target) {
            Ok(i) => self.rejected[i] = judged,
            Err(_) => {
                if self.rejected.len() >= MAX_REJECTED {
                    self.rejected.retain(|j| in_sight(j));
                }
                if self.rejected.len() >= MAX_REJECTED {
                    self.rejected.remove(0);
                }
                let i = self.rejected.partition_point(|j| j.target < judged.target);
                self.rejected.insert(i, judged);
            }
        }
    }

    /// The PathReference is `path` from now on. The rejections were judged by the old one: they go, unless the two
    /// are the same route (the path that comes for a leg across open ground is its straight line).
    pub(super) fn set_reference(&mut self, path: &[Vec2]) {
        if self.reference == path {
            return;
        }
        self.reference.clear();
        self.reference.extend_from_slice(path);
        self.rejected.clear();
    }
}

/// The distance from `p` to the nearest point of the route `route`, a chain of legs.
pub(super) fn distance_to_route(p: Vec2, route: &[Vec2]) -> f32 {
    let to_leg = |a: Vec2, b: Vec2| {
        let along = b - a;
        let len2 = along.length_squared();
        let t = if len2 > 0.0 {
            ((p - a).dot(along) / len2).clamp(0.0, 1.0)
        } else {
            0.0
        };
        p.distance(a + along * t)
    };
    match route {
        [] => 0.0,
        [only] => p.distance(*only),
        _ => route
            .windows(2)
            .map(|leg| to_leg(leg[0], leg[1]))
            .fold(f32::MAX, f32::min),
    }
}

/// Whether `p` is within AttackMoveLeashDistance of the PathReference `reference`.
pub(super) fn within_leash(p: Vec2, reference: &[Vec2]) -> bool {
    distance_to_route(p, reference) <= gu(ATTACK_MOVE_LEASH_GU)
}

/// Whether any firing position on `target` can lie within the leash of `reference` at all: a unit that reaches the
/// target stands within its Range (with the ElevationModifier at most) of the target's shape, so its centre is
/// within that, and the two shapes' half diagonals, of the target's centre. Free: no path. `false` rules the
/// target out; `true` leaves it to [`judge`].
pub(super) fn near_route(unit: &Unit, attrs: &AttackAttrs, target: &View, reference: &[Vec2]) -> bool {
    let (lo, hi) = target.shape.bounds();
    let own_half = unit.kind.def().half_extents().unwrap_or(Vec2::ZERO);
    let slack = gu(attrs.range + ELEVATION_MODIFIER) + ((hi - lo) / 2.0).length() + own_half.length();
    distance_to_route(target.pos, reference) <= gu(ATTACK_MOVE_LEASH_GU) + slack
}

/// The reach rule (module docs): whether `unit`, on a leg whose PathReference is `reference`, can finish a chase of
/// `target`, which is out of its weapon range. Plans one path, within the unit's budget for the tick.
pub(super) fn judge(
    unit: &Unit,
    attrs: &AttackAttrs,
    reference: &[Vec2],
    target: &View,
    world: &World,
    reach: &mut Reach,
) -> Verdict {
    // Where the unit stands is the first point of any path.
    if !within_leash(unit.pos, reference) || !near_route(unit, attrs, target, reference) {
        return Verdict::Unreachable;
    }
    if reach.planned >= JUDGE_PATHS_PER_TICK {
        return Verdict::Later;
    }
    reach.planned += 1;
    reach.counters.paths += 1;
    reach.counters.most_per_unit = reach.counters.most_per_unit.max(reach.planned);
    let before = reach.search.counters().expansions;
    let max_tiles = (JUDGE_PATH_SIGHTS * (unit.kind.def().sight_range + ELEVATION_MODIFIER)).ceil() as u32;
    let path = world.chase_path(unit, target, max_tiles, &mut reach.search);
    reach.counters.expansions += reach.search.counters().expansions - before;
    match path {
        Ok(Some(path)) if finishes(unit, attrs, reference, target, &path, world) => Verdict::Reachable,
        Ok(_) => Verdict::Unreachable,
        Err(()) => Verdict::Later,
    }
}

/// Whether `path`, walked from its start, comes to a firing position on `target` with every point up to it within
/// the leash of `reference` and the target within the unit's SightRange from it (module docs).
fn finishes(unit: &Unit, attrs: &AttackAttrs, reference: &[Vec2], target: &View, path: &[Vec2], world: &World) -> bool {
    let sight = unit.kind.def().sight_range;
    // `Some(fires)` for a point the unit may walk through; `None` for one off the leash or out of sight.
    let at = |p: Vec2| {
        if !within_leash(p, reference) {
            return None;
        }
        let source = Elevation::of(world.map, unit.kind, p);
        let gap = Shape::silhouette(unit.kind, p, unit.facing).gap(&target.shape);
        (gap <= gu(elevation::sight_range(sight, source, target.elevation)))
            .then(|| within_range(attrs, gap, source, target.elevation))
    };
    let Some(&start) = path.first() else {
        return false;
    };
    match at(start) {
        Some(true) => return true,
        Some(false) => {}
        None => return false,
    }
    for leg in path.windows(2) {
        let (a, b) = (leg[0], leg[1]);
        let steps = (a.distance(b) / SAMPLE_SU).ceil().max(1.0);
        for i in 1..=steps as u32 {
            match at(a.lerp(b, i as f32 / steps)) {
                Some(true) => return true,
                Some(false) => {}
                None => return false,
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_leash_is_measured_to_the_nearest_point_of_the_route() {
        let route = [Vec2::ZERO, Vec2::new(100.0, 0.0), Vec2::new(100.0, 100.0)];
        assert_eq!(distance_to_route(Vec2::new(50.0, 30.0), &route), 30.0);
        assert_eq!(distance_to_route(Vec2::new(130.0, 50.0), &route), 30.0);
        assert_eq!(
            distance_to_route(Vec2::new(-30.0, -40.0), &route),
            50.0,
            "before its start"
        );
        assert_eq!(distance_to_route(Vec2::new(100.0, 110.0), &route), 10.0, "past its end");
        assert_eq!(distance_to_route(Vec2::new(3.0, 4.0), &[Vec2::ZERO]), 5.0);
        assert_eq!(distance_to_route(Vec2::new(3.0, 4.0), &[Vec2::ZERO, Vec2::ZERO]), 5.0);
        assert_eq!(distance_to_route(Vec2::ONE, &[]), 0.0);
    }
}
