//! Progress, stuck detection and recovery — `docs/movement.md > 6.5`, with the repath schedule of `> 5.6`.
//!
//! The rules are `systems/controls/behaviors.md > MovingToLocation`: `> Failure` (design change 0024: stuck after
//! `mover::STUCK_TICKS`, `MAX_REPATHS` attempts at another route after `REPATH_WAITS`, then give up) and `> Occupied
//! target location` (design change 0026, `mover::OCCUPIED_ARRIVAL_GU`). Two moving units that block each other for
//! good both end `Failed` (design change 0022: moving units never yield or swap).

use glam::Vec2;

use super::collide::{Moved, World};
use super::space::Body;
use super::yielding::yields_to;
use crate::ids::PlayerId;
use crate::mover::{
    JAMMED_TICKS, MAX_AVOID_TILES, MAX_REPATHS, MoveGoal, MoveStatus, Mover, OCCUPIED_ARRIVAL_GU, STUCK_GRACE_TICKS,
};
use crate::units::{Tick, TilePos, gu};

/// The length of the path still ahead of a unit at `pos` heading for waypoint `next`.
fn remaining(pos: Vec2, path: &[Vec2], next: usize) -> f32 {
    let Some(ahead) = path.get(next..).filter(|ahead| !ahead.is_empty()) else {
        return 0.0;
    };
    let rest: f32 = ahead.windows(2).map(|leg| leg[0].distance(leg[1])).sum();
    pos.distance(ahead[0]) + rest
}

/// `docs/movement.md > 6.5`, rule 2 — **the hook for slice E**: a unit chasing a target, blocked by stationary
/// friendly units that attack the same target, queues: it keeps waiting and checks again every
/// `mover::QUEUE_RECHECK_TICKS`, with no repath and no failure. Nothing chases yet, so nothing queues.
fn queues_behind(_mover: &Mover, _blocker: Option<&Body>) -> bool {
    // TODO(slice E): `MoveGoal::Object` with an attack order, the blocker friendly, stationary and attacking the
    // same target.
    false
}

/// Notes the tile of the stationary unit in the way as one to avoid (`docs/movement.md > 6.5`, rule 3). A tile is
/// 64 su and a unit far smaller, so the blocker often stands in the unit's own tile, which the path server never
/// avoids (it is the start): then the tile next to it on the blocker's side is noted, the one the way through the
/// blocker leads into. The goal's tile is left out: avoiding it would fail the search outright.
fn avoid_blocker(pos: Vec2, mover: &mut Mover, blocker: &Body) {
    let end = mover.path.last().copied().unwrap_or(pos);
    let (own, goal) = (TilePos::containing(pos), TilePos::containing(end));
    let mut tile = TilePos::containing(blocker.pos);
    if tile == own {
        let to = blocker.pos - pos;
        if to.x.abs() >= to.y.abs() {
            tile.x += if to.x >= 0.0 { 1 } else { -1 };
        } else {
            tile.y += if to.y >= 0.0 { 1 } else { -1 };
        }
    }
    if tile != goal && mover.avoid_tiles.len() < MAX_AVOID_TILES && !mover.avoid_tiles.contains(&tile) {
        mover.avoid_tiles.push(tile);
    }
}

/// `systems/controls/behaviors.md > MovingToLocation > Occupied target location` (design change 0026): whether the
/// unit at `pos`, blocked by `blocker` this tick, counts as arrived. It is within `OCCUPIED_ARRIVAL_GU` of its
/// destination, and the unit in its way isn't moving and won't move for it.
/// - **Yielding comes first** (design changes 0022, 0023): a blocker that yields to this unit has been asked to step
///   aside, and a unit whose blocker steps aside isn't stuck. So the rule is for blockers that stay: units that
///   arrived with its own group order (0027), units on Hold, busy units, enemies, units that can't move, and a
///   yielding unit that found no room.
/// - **Timing** (the design leaves it open; 0026's Verification says "straight away, with no re-route attempts"):
///   when the unit is stuck (`Mover::is_stuck`, 24 ticks), which is 8 ticks before its first attempt at another
///   route; or already when it found no way forward for `JAMMED_TICKS` in a row behind a unit that has no goal of
///   its own, which will never move on. So it never re-routes here.
fn target_is_occupied(pos: Vec2, mover: &Mover, blocker: Option<&Body>, owner: Option<PlayerId>, tick: Tick) -> bool {
    let (MoveGoal::Point(_), Some(blocker)) = (mover.goal, blocker) else {
        return false;
    };
    let end = mover.path.last().copied().unwrap_or(pos);
    if !blocker.stationary || yields_to(blocker, owner, mover.group) || pos.distance(end) > gu(OCCUPIED_ARRIVAL_GU) {
        return false;
    }
    mover.is_stuck(tick) || (blocker.progress.is_none() && mover.blocked_ticks >= JAMMED_TICKS)
}

/// After a unit's move in tick `tick` (`docs/movement.md > 6.5`). Returns whether the move failed.
///
/// **Progress** is the remaining path length: `last_progress` moves on when it beats `best_remaining` by half a
/// step. A new path measures afresh but is no progress by itself. Following a slower unit is progress, and so is
/// **waiting behind a unit that is getting on**: a unit blocked by one that is on its way takes over that unit's
/// `last_progress` if it is later. So a queue at a choke isn't stuck while its head moves, and units that block
/// each other for good share a clock that stands still. (On one path `best_remaining` only falls, and repaths
/// are capped, so the progress of a whole crowd is finite: nothing keeps a deadlock alive.)
///
/// In order:
/// 1. **Occupied target** ([`target_is_occupied`], design change 0026): the unit has `Arrived`.
/// 2. Not **stuck** (`Mover::is_stuck`, design change 0024: `STUCK_TICKS` without getting closer): carry on.
/// 3. **Queueing** ([`queues_behind`]): keep waiting.
/// 4. **Another route**, when its wait is over (`Mover::recovery_due`: 8, 16 and 32 ticks after the unit became
///    stuck or after the attempt before), if an attempt is left. The tile of a stationary unit in its way is added
///    to `avoid_tiles` first ([`avoid_blocker`]): at once if that unit has no goal of its own; if it is on its way
///    itself (jammed for now), only from the second attempt on, the first being a plain retry. If the search finds
///    no way around the avoid tiles, phase 5c drops them and plans again: tiles are too coarse next to units to
///    conclude that there is no way.
/// 5. No attempt left, and still stuck `STUCK_TICKS` after the last one: `Failed`. The behaviour stops and
///    completes (`> Failure`). A unit that never gets on looks for another route 32, 48 and 80 ticks after its
///    last progress and fails at 104 (`mover::GIVE_UP_TICKS`, 6.5 s: 0024's "about 7 s").
///
/// A unit due for 4 or 5 that touches no unit in this tick (it stepped around one) waits up to `STUCK_GRACE_TICKS`
/// to meet its blocker again, so that rules 1 and 4 can see it.
pub(super) fn after_move(pos: Vec2, mover: &mut Mover, step: f32, moved: &Moved, world: &World, tick: Tick) -> bool {
    let blocker = moved.blocker.map(|index| &world.bodies[index]);
    let left = remaining(pos, &mover.path, usize::from(mover.next));
    if mover.best_remaining == f32::MAX {
        // The first measure on a new path: where the unit starts from, no progress yet.
        mover.best_remaining = left;
    } else if left < mover.best_remaining - step * 0.5 {
        mover.best_remaining = left;
        mover.last_progress = tick;
    } else if let Some(ahead) = blocker.and_then(|b| b.progress) {
        mover.last_progress = mover.last_progress.max(ahead.min(tick));
    }
    if target_is_occupied(pos, mover, blocker, world.body().owner, tick) {
        mover.finish(MoveStatus::Arrived);
        return false;
    }
    if !mover.is_stuck(tick) || queues_behind(mover, blocker) {
        return false;
    }
    let due = mover.recovery_due();
    if tick < due || (blocker.is_none() && tick < due.saturating_add(STUCK_GRACE_TICKS)) {
        return false;
    }
    if mover.repaths_left == 0 {
        mover.finish(MoveStatus::Failed);
        return true;
    }
    let first_repath = mover.repaths_left == MAX_REPATHS;
    if let Some(blocker) = blocker.filter(|b| b.stationary && (b.progress.is_none() || !first_repath)) {
        avoid_blocker(pos, mover, blocker);
    }
    mover.counted_repath(tick);
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_remaining_length_runs_from_the_unit_along_the_waypoints_ahead() {
        let path = [Vec2::ZERO, Vec2::new(10.0, 0.0), Vec2::new(10.0, 5.0)];
        assert_eq!(remaining(Vec2::new(4.0, 0.0), &path, 1), 11.0);
        assert_eq!(remaining(Vec2::new(10.0, 1.0), &path, 2), 4.0);
        assert_eq!(remaining(Vec2::ZERO, &path, 3), 0.0);
        assert_eq!(remaining(Vec2::ZERO, &[], 0), 0.0);
    }
}
