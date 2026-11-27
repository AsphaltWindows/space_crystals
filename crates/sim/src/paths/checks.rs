//! The staggered path checks of phase 5c (`docs/movement.md > 5.6`, `> 6.2`): what keeps a unit's path good while
//! it walks. They run on the grid the unit plans on, its owner's knowledge (`paths::knowledge`).

use glam::Vec2;

use crate::mover::Mover;
use crate::nav::{self, NavGrid};
use crate::ruleset::rules;
use crate::units::{Tick, gu};

/// At most this many waypoints are passed in one staggered check (a technical bound on its line-of-sight tests).
const MAX_WAYPOINTS_PER_CHECK: usize = 8;

/// The staggered path checks of one unit with a path (`docs/movement.md > 5.6`, `> 6.2`), run in the ticks where
/// `(tick + id) % STAGGER == 0`, on the grid the unit plans on (its owner's knowledge):
///
/// 1. **What its owner knows of the ground changed** (`version`, `PlayerKnowledge::version`; for a unit without an
///    owner the true map's `nav_version`): line of sight along the remaining path. If every leg is still clear the
///    path stays, else the unit plans again. Not counted against the repath cap. This is the re-route of
///    `systems/controls/behaviors.md > PathKnowledge`: a unit that walks towards an obstacle it didn't know about
///    plans again once its owner has seen it.
/// 2. **Steering target**: the unit heads for the farthest waypoint it sees, one after another from the current
///    one, so it need not touch any of them.
/// 3. **Lost sight** of the steering waypoint (pushed off the path by sidesteps) **and stuck**: another route, a
///    repath that counts against the cap, on the schedule of design change 0024 (`Mover::recovery_due`: never
///    before the unit is stuck and has waited). A unit that is still getting on finds its own way back by sliding,
///    which saves the leg's few repaths for when it needs them.
pub(super) fn staggered_check(grid: &NavGrid, pos: Vec2, half: Vec2, mover: &mut Mover, version: u32, tick: Tick) {
    let sees = |from: Vec2, to: Vec2| nav::los(grid, half, from, to);
    let next = usize::from(mover.next);
    let Some(&target) = mover.path.get(next) else {
        return;
    };
    if mover.nav_version != version {
        let ahead = &mover.path[next..];
        if sees(pos, target) && ahead.windows(2).all(|leg| sees(leg[0], leg[1])) {
            mover.nav_version = version;
        } else {
            mover.repath(tick);
            return;
        }
    }
    // The steering also stays out of the unit's avoid tiles, as its path does.
    let mut at = next;
    // (M3-G2) Nor does it skip the waypoints of a detour round standing units: no grid shows what they lead round.
    while at + 1 < mover.path.len()
        && at >= usize::from(mover.detour_end)
        && at - next < MAX_WAYPOINTS_PER_CHECK
        && sees(pos, mover.path[at + 1])
        && nav::segment_avoids(pos, mover.path[at + 1], &mover.avoid_tiles)
    {
        at += 1;
    }
    if at != next {
        mover.next = at as u16;
    } else if mover.is_stuck(tick) && tick >= mover.recovery_due() && !sees(pos, target) {
        mover.counted_repath(tick);
    }
}

/// `docs/movement.md > 5.6`, "Chase target moved" — the staggered check of a unit heading for an object
/// (`MoveGoal::Object`) that is now at `target`, run before [`staggered_check`]:
///
/// - **In sight** (a straight line the unit's box can walk, clear of its avoid tiles): the unit steers straight at
///   the object, with no path search: its path becomes the one leg from where it is to the object.
/// - **Out of sight**: it keeps the path it has while the object stays near where the path leads (within
///   `Paths::chase_replan_gu` or `Paths::chase_replan_share` of the distance), and plans again once it has moved
///   farther, at most every `Paths::chase_replan_ticks`. These repaths don't count against the cap: the rate limit
///   bounds them.
///
/// `grid` is `None` for air units, which always fly straight.
pub(super) fn chase_check(
    grid: Option<&NavGrid>,
    pos: Vec2,
    half: Vec2,
    mover: &mut Mover,
    target: Vec2,
    due: bool,
    tick: Tick,
) {
    let tuning = &rules().tuning.paths;
    if mover.planned_target == target {
        return;
    }
    let in_sight = grid.is_none_or(|grid| nav::los(grid, half, pos, target))
        && nav::segment_avoids(pos, target, &mover.avoid_tiles);
    if in_sight {
        mover.path.clear();
        mover.path.extend([pos, target]);
        mover.next = 1;
        mover.detour_end = 0;
        mover.planned_target = target;
        return;
    }
    let moved = target.distance(mover.planned_target);
    if due && moved > gu(tuning.chase_replan_gu).max(tuning.chase_replan_share * pos.distance(target)) {
        mover.repath(tick);
    }
}
