//! Progress, stuck detection and recovery — `docs/movement.md > 6.5`, with the repath cap of `> 5.6`.
//!
//! The numbers are the defaults of question Q0008 (`mover::STUCK_TICKS`, `MAX_REPATHS`, `REPATH_BACKOFF`,
//! `CROWDED_ARRIVAL_GU`). What two units blocking each other for good do (both end `Failed`) is the default of
//! question Q0007.

use glam::Vec2;

use super::collide::{Moved, World};
use super::space::Body;
use crate::mover::{
    CROWDED_ARRIVAL_GU, MAX_AVOID_TILES, MAX_REPATHS, MoveGoal, MoveStatus, Mover, STUCK_GRACE_TICKS, STUCK_TICKS,
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

/// After a unit's move in tick `tick` (`docs/movement.md > 6.5`). Returns whether the move failed.
///
/// **Progress** is the remaining path length: `last_progress` moves on when it beats `best_remaining` by half a
/// step. A new path measures afresh but is no progress by itself. Following a slower unit is progress, and so is
/// **waiting behind a unit that is getting on**: a unit blocked by one that is on its way takes over that unit's
/// `last_progress` if it is later. So a queue at a choke isn't stuck while its head moves, and units that block
/// each other for good share a clock that stands still. (On one path `best_remaining` only falls, and repaths
/// are capped, so the progress of a whole crowd is finite: nothing keeps a deadlock alive.)
///
/// A unit is **stuck** after `STUCK_TICKS` without progress, counted from its latest counted repath if that was
/// later (`Mover::stuck_clock`): every new path gets its time. Once stuck, in order:
/// 1. **Crowded arrival**: the goal is a point and the unit in its way is stationary, and either the goal is
///    within `CROWDED_ARRIVAL_GU`, or that unit arrived on a move of the same order group: the unit has
///    `Arrived` (G5). The second case lets a group fill up outwards from the slots it reached: slots are 8 su
///    apart, so a unit whose slot got walled in by earlier arrivals can't reach it, however large the group is.
/// 2. **Queueing** ([`queues_behind`]): keep waiting.
/// 3. **Repath**, once the backoff of the last repath is over (`docs/movement.md > 5.6`), if a repath is left.
///    The tile of a stationary unit in its way is added to `avoid_tiles` first ([`avoid_blocker`]): at once if
///    that unit has no goal of its own; if it is on its way itself (jammed for now), only from the second
///    repath on, the first being a plain retry. If the search finds no way around the avoid tiles, phase 5c
///    drops them and plans again: tiles are too coarse next to units to conclude that there is no way.
/// 4. No repath left, and the last backoff over: `Failed`. The behaviour stops and completes
///    (`systems/controls/behaviors.md > MovingToLocation > Failure`). A unit that never gets on repaths 24, 48
///    and 72 ticks after its last progress and fails at 104 (6.5 s), the default of question Q0008.
///
/// A stuck unit that touches no unit in this tick (it stepped around one) waits up to `STUCK_GRACE_TICKS` to meet
/// its blocker again, so that rules 1 and 3 can see it.
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
    let waited = tick.saturating_sub(mover.stuck_clock());
    if waited < STUCK_TICKS || (blocker.is_none() && waited < STUCK_TICKS + STUCK_GRACE_TICKS) {
        return false;
    }
    let end = mover.path.last().copied().unwrap_or(pos);
    let crowded = blocker.is_some_and(|b| {
        let with_its_group = mover.group.is_some() && b.arrived_with == mover.group;
        b.stationary && (with_its_group || pos.distance(end) <= gu(CROWDED_ARRIVAL_GU))
    });
    if matches!(mover.goal, MoveGoal::Point(_)) && crowded {
        mover.finish(MoveStatus::Arrived);
        return false;
    }
    if queues_behind(mover, blocker) || tick < mover.repath_ready_at {
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
