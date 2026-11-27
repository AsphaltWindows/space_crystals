//! Progress, stuck detection and recovery — `docs/movement.md > 6.5`, with the repath schedule of `docs/movement.md >
//! 5.6`.
//!
//! The rules are `systems/controls/command_behaviors.md > MovingToLocation`: `systems/controls/command_behaviors.md >
//! Failure` (design change 0024: stuck after `tuning::movement::STUCK_TICKS`, `MAX_REPATHS` attempts at another route
//! after `REPATH_WAITS`, then give up) and `systems/controls/command_behaviors.md > Occupied target location` (design
//! change 0026, `tuning::movement::OCCUPIED_ARRIVAL_GU`). Two moving units that block each other for good both end
//! `Failed` (design change 0022: moving units never yield or swap).

use glam::Vec2;

use super::Chased;
use super::collide::{Moved, World};
use super::round;
use super::space::Body;
use super::yielding::yields_to;
use crate::ids::{ObjectId, PlayerId};
use crate::mover::{MAX_AVOID_TILES, MoveGoal, MoveStatus, Mover};
use crate::shape::Shape;
use crate::tuning::movement::{JAMMED_TICKS, MAX_REPATHS, OCCUPIED_ARRIVAL_GU, STUCK_GRACE_TICKS};
use crate::units::{Tick, TilePos, gu};

/// The length of the path still ahead of a unit at `pos` heading for waypoint `next`.
pub(super) fn remaining(pos: Vec2, path: &[Vec2], next: usize) -> f32 {
    let Some(ahead) = path.get(next..).filter(|ahead| !ahead.is_empty()) else {
        return 0.0;
    };
    let rest: f32 = ahead.windows(2).map(|leg| leg[0].distance(leg[1])).sum();
    pos.distance(ahead[0]) + rest
}

/// (M3-H2) Whether the units waiting behind a unit on its way share its progress clock ([`after_move`]): it **has
/// got on since its leg began**. A new leg sets the unit's own clock to the tick it begins (`Mover::go_to`), which
/// is right for the unit (it is stuck only `STUCK_TICKS` after it was sent) but is no progress: an order isn't.
/// Until this rule, those behind it took that clock over all the same, so a unit whose legs kept beginning again
/// without it ever moving on (a worker trying its patch again, a blocked unit the player keeps clicking on)
/// renewed their clocks with every leg, and none of them ever looked for a way round it.
///
/// Read off the state, with no field of its own: on a leg without a counted repath, `last_repath` is the tick the
/// leg began, and `last_progress` is later only if the unit got on, took over the clock of a unit ahead that did,
/// or stood waiting for its path or its attack (`super::wait` runs the clock on; a path normally comes in the
/// tick it is asked for, so that is rare, and it then counts as before this rule). After a counted repath the unit
/// was stuck on this leg and its clock is old: it is shared as before.
pub(super) fn leads(mover: &Mover) -> bool {
    mover.repaths_left < MAX_REPATHS || mover.last_progress > mover.last_repath
}

/// The progress clock `blocker` shares with a unit waiting behind it: `Body::progress`, if it [`leads`].
fn shared_progress(blocker: &Body) -> Option<Tick> {
    blocker.progress.filter(|_| blocker.leads)
}

/// `docs/movement.md > 6.5`, rule 2: a unit chasing the target of its attack (`chased`, which its attack engages),
/// blocked by a stationary unit of its owner that attacks the same target, **queues**: its attempts at another
/// route are detours round the units in its way, never a path request. `engaged` lists what each body's attack
/// engages; the unit's own index and `blocker` index it. (The check is a few comparisons, so it runs in every tick
/// the unit is blocked.)
///
/// **(M3-G2) A queueing unit goes round** (`systems/units.md > UnitCollision > Ground Collision`: "Moving units must
/// pathfind around occupied space"; `systems/controls/command_behaviors.md > AttackingObject > Blocked by its own
/// side`, "Farther out"). A unit behind its owner's firing line stood out of range for as long as the target lived,
/// with open ground on both sides (`docs/playtest/m3.md`: the last of four Peacekeepers on a building). The units ahead
/// of it won't move on while the target stands, so there is nothing to wait for. It takes a detour round the row
/// they stand in ([`round::detour`]; a path request would bring back the path it has, the row is in no grid):
/// - **When**: on the schedule of the re-routes (stuck, and the wait of `Mover::recovery_due` over), or already
///   after `JAMMED_TICKS` without a way forward behind a unit that has no goal of its own (it stands in range and
///   will not move on), as the occupied-target rule has it.
/// - **How often**: an attempt counts against the leg's `MAX_REPATHS` and waits like a repath (`Mover::go_round`).
///   The first goes by the nearer end of the row's front, the second by the other, the third by the nearer again.
/// - **A lost race is free**: if the unit now in its way moved after this unit's last attempt (`Body::moved`), it
///   took the place the unit went for. That says nothing about room, so the next attempt doesn't count, and goes
///   by the nearer end, round the newcomer. Each lost race is a unit that found its place, so they are at most as
///   many as the units: eight in a column all get into range (`tests/attack_behind_friends/`).
/// - **No way round, or no attempt left** (design change 0223, answering question Q0057 point 2: "If there isn't,
///   the normal Failure rule applies ... it does not wait behind the blockers indefinitely"): an attempt that
///   finds no way round counts all the same, and once none is left the unit gives up as any stuck unit does, still
///   stuck `STUCK_TICKS` after its last attempt ([`after_move`], rule 5). The behaviour then gives the target up
///   (`systems/controls/command_behaviors.md > AttackingObject > Failure`). Until 0223 such a unit stood behind the row
///   for as long as the target lived.
fn queues_behind(chased: Option<&Chased>, engaged: &[Option<ObjectId>], world: &World, blocker: Option<usize>) -> bool {
    let (Some(chased), Some(blocker)) = (chased, blocker) else {
        return false;
    };
    let (me, other) = (world.body(), &world.bodies[blocker]);
    let attacks = |index: usize| engaged.get(index).copied().flatten() == Some(chased.id);
    attacks(world.me) && attacks(blocker) && me.owner.is_some() && other.owner == me.owner && other.stationary
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

/// (G) Notes the tiles of the units that arrived with the order group `group` as ones to avoid, the nearest to
/// `pos` first (ties by id), as many as `MAX_AVOID_TILES` allow ([`avoid_blocker`] for each). For a unit shut out
/// of its place by its own group: those units won't make way for it (design change 0027), so its new route goes
/// round all of them, not round the one in its way into the next, and comes at its place from the open side. Two
/// blocks that crossed head-on arrive out of order, and a Guard can't pass between Guards that stand 16 su apart:
/// the late ones stopped behind the group farther than `OCCUPIED_ARRIVAL_GU` from their places and gave up
/// ([`after_move`], rule 1). A place that the group closes in on every side stays out of reach, and the unit gives up
/// as before.
fn avoid_group(pos: Vec2, mover: &mut Mover, bodies: &[Body], group: u32) {
    let mut arrived: Vec<&Body> = bodies.iter().filter(|b| b.arrived_with == Some(group)).collect();
    arrived.sort_by(|a, b| {
        let (da, db) = (a.pos.distance_squared(pos), b.pos.distance_squared(pos));
        da.total_cmp(&db).then(a.id.cmp(&b.id))
    });
    for body in arrived {
        avoid_blocker(pos, mover, body);
    }
}

/// `systems/controls/command_behaviors.md > MovingToLocation > Occupied target location` (design changes 0026 and
/// 0169): whether the unit at `pos` (its shape `shape` there), blocked by `blocker` this tick, counts as arrived. It
/// is within `OCCUPIED_ARRIVAL_GU` (3 grid units) of its destination, and the unit in its way isn't moving and won't
/// move for it.
/// - **An object goal** (`systems/controls/command_behaviors.md > MovingToObject > Completion`, "Blocked near the
///   target"; design changes 0032, 0033 and 0169): the same rule and radius, measured as the gap between the unit's
///   shape and the object's (`chased`). What reaching it means is the behaviour's: a followed unit is waited for, a
///   static target completes the order.
/// - **Yielding comes first** (design changes 0022, 0023): a blocker that yields to this unit has been asked to step
///   aside, and a unit whose blocker steps aside isn't stuck. So the rule is for blockers that stay: units that
///   arrived with its own group order (0027), units on Hold, busy units, enemies, units that can't move, and a
///   yielding unit that found no room.
/// - **Timing** (the design leaves it open; 0026's Verification says "straight away, with no re-route attempts"):
///   when the unit is stuck (`Mover::is_stuck`, 24 ticks), which is 8 ticks before its first attempt at another
///   route; or already when it found no way forward for `JAMMED_TICKS` in a row behind a unit that has no goal of
///   its own, which will never move on. So it never re-routes here.
/// - **Not on a leg that presses on** (`Mover::presses_on`; design change 0315,
///   `factions/syndicate/units/agent.md > Gathering > Gathering rules`, "Waiting at the hatch": until an Agent is
///   within reach of the hatch "it is never 'arrived' [...] and is exempt from MovingToLocation's 'blocked near the
///   target by standing units counts as arrived' rule"). Such a unit goes on by rules 2 to 5: it steers, looks for
///   other routes, and fails when none is left; its behaviour then begins the leg again
///   (`orders::gather::legs::return_leg`).
fn target_is_occupied(
    pos: Vec2,
    shape: Shape,
    mover: &Mover,
    chased: Option<&Chased>,
    blocker: Option<&Body>,
    owner: Option<PlayerId>,
    tick: Tick,
) -> bool {
    let Some(blocker) = blocker.filter(|_| !mover.presses_on) else {
        return false;
    };
    let distance = match (mover.goal, chased) {
        (MoveGoal::Point(_), _) => pos.distance(mover.path.last().copied().unwrap_or(pos)),
        (MoveGoal::Object { .. }, Some(chased)) => shape.gap(&chased.shape),
        (MoveGoal::Object { .. }, None) | (MoveGoal::None, _) => return false,
    };
    if !blocker.stationary || yields_to(blocker, owner, mover.group) || distance > gu(OCCUPIED_ARRIVAL_GU) {
        return false;
    }
    mover.is_stuck(tick) || (blocker.progress.is_none() && mover.blocked_ticks >= JAMMED_TICKS)
}

/// After a unit's move in tick `tick` (`docs/movement.md > 6.5`). Returns whether the move failed.
///
/// **Progress towards a moving object** (`chased`, a `MoveGoal::Object` whose object is a unit that moved this
/// tick or the last) is getting on at all: the object may move as fast as the unit, and the path is re-aimed at it
/// as it moves, so its remaining length says nothing. Such a unit makes progress in every tick it moves forward
/// (`Moved::forward`), and is stuck when units or terrain have kept it from that for `STUCK_TICKS`. Towards an
/// object at rest, progress is measured as to a point, afresh from when it came to rest.
///
/// **Progress** to a point is the remaining path length: `last_progress` moves on when it beats `best_remaining` by
/// half a step. A new path measures afresh but is no progress by itself. Following a slower unit is progress, and so is
/// **waiting behind a unit that is getting on**: a unit blocked by one that is on its way and has got on since its
/// leg began ([`leads`]) takes over that unit's `last_progress` if it is later. So a queue at a choke isn't stuck while
/// its head moves, and units that block each other for good share a clock that stands still. (On one path
/// `best_remaining` only falls, and repaths are capped, so the progress of a whole crowd is finite: nothing keeps a
/// deadlock alive.)
///
/// In order:
/// 1. **Occupied target** ([`target_is_occupied`], design change 0026): the unit has `Arrived`.
/// 2. Not **stuck** (`Mover::is_stuck`, design change 0024: `STUCK_TICKS` without getting closer): carry on.
/// 3. **Queueing** ([`queues_behind`]): a unit chasing its attack's target behind its owner's attackers asks for
///    no other route. **(M3-G2)** When an attempt is due it goes round the row they stand in instead
///    ([`round::detour`]). **(0223)** With no attempt left it gives up by rule 5, like any unit.
/// 4. **Another route**, when its wait is over (`Mover::recovery_due`: 8, 16 and 32 ticks after the unit became
///    stuck or after the attempt before), if an attempt is left. The tile of a stationary unit in its way is added
///    to `avoid_tiles` first ([`avoid_blocker`]): at once if that unit has no goal of its own; if it is on its way
///    itself (jammed for now), only from the second attempt on, the first being a plain retry. **(G)** A unit that
///    arrived with this unit's own group order won't make way for it (design change 0027), and nor will the others
///    that did: then the tiles of all of them are added ([`avoid_group`]), and the new route goes round the group.
///    If the search finds no way around the avoid tiles, phase 5c drops them and plans again: tiles are too coarse
///    next to units to conclude that there is no way.
/// 5. No attempt left, and still stuck `STUCK_TICKS` after the last one: `Failed`. The behaviour stops and
///    completes (`systems/controls/command_behaviors.md > Failure`). A unit that never gets on looks for another route
///    32, 48 and 80 ticks after its last progress and fails at 104 (`tuning::movement::GIVE_UP_TICKS`, 6.5 s: 0024's
///    "about 7 s").
///
/// A unit due for 4 or 5 that touches no unit in this tick (it stepped around one) waits up to `STUCK_GRACE_TICKS`
/// to meet its blocker again, so that rules 1 and 4 can see it.
#[expect(
    clippy::too_many_arguments,
    reason = "one unit's move and what the step knows of the object it heads for"
)]
pub(super) fn after_move(
    pos: Vec2,
    mover: &mut Mover,
    step: f32,
    moved: &Moved,
    world: &World,
    chased: Option<&Chased>,
    engaged: &[Option<ObjectId>],
    tick: Tick,
) -> bool {
    let blocker = moved.blocker.map(|index| &world.bodies[index]);
    let left = remaining(pos, &mover.path, usize::from(mover.next));
    // Bodies are in id order.
    let target_moves = chased.is_some_and(|chased| {
        let found = world.bodies.binary_search_by_key(&chased.id, |body| body.id);
        found.is_ok_and(|index| world.bodies[index].velocity != Vec2::ZERO)
    });
    if target_moves {
        mover.best_remaining = f32::MAX;
        if moved.forward {
            mover.last_progress = tick;
        } else if let Some(ahead) = blocker.and_then(shared_progress) {
            mover.last_progress = mover.last_progress.max(ahead.min(tick));
        }
    } else if mover.best_remaining == f32::MAX {
        // The first measure on a new path: where the unit starts from, no progress yet.
        mover.best_remaining = left;
    } else if left < mover.best_remaining - step * 0.5 {
        mover.best_remaining = left;
        mover.last_progress = tick;
    } else if let Some(ahead) = blocker.and_then(shared_progress) {
        mover.last_progress = mover.last_progress.max(ahead.min(tick));
    }
    let me = world.body();
    if target_is_occupied(pos, me.shape_at(pos), mover, chased, blocker, me.owner, tick) {
        mover.finish(MoveStatus::Arrived);
        return false;
    }
    let queueing = moved
        .blocker
        .filter(|_| queues_behind(chased, engaged, world, moved.blocker));
    if let Some(index) = queueing {
        // Rule 3: round the row while an attempt is left or a race was lost. Otherwise it carries on, and fails
        // by rule 5 as any unit does (design change 0223).
        let ahead = &world.bodies[index];
        let jammed = ahead.progress.is_none() && mover.blocked_ticks >= JAMMED_TICKS;
        let lost_race = ahead.moved > mover.last_repath;
        if (jammed || (mover.is_stuck(tick) && tick >= mover.recovery_due())) && mover.may_go_round(tick, lost_race) {
            let other_end = !lost_race && mover.repaths_left == MAX_REPATHS - 1;
            let target = chased.map_or(pos, |chased| chased.pos);
            let detour = round::detour(world, index, pos, target, other_end);
            mover.go_round(pos, &detour, tick, lost_race);
            return false;
        }
    }
    if !mover.is_stuck(tick) {
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
    if queueing.is_some() {
        // Never reached: with an attempt left and its wait over, rule 3 went round. A queueing unit is never
        // given a path request for the row, which no grid shows (M3-G2).
        return false;
    }
    let first_repath = mover.repaths_left == MAX_REPATHS;
    if let Some(blocker) = blocker.filter(|b| b.stationary && (b.progress.is_none() || !first_repath)) {
        avoid_blocker(pos, mover, blocker);
        if let Some(group) = mover.group.filter(|&group| blocker.arrived_with == Some(group)) {
            avoid_group(pos, mover, world.bodies, group);
        }
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

    /// (M3-H2) A unit's progress clock counts for those behind it once it has got on since its leg began: not on
    /// the tick it was sent, nor while it only stands blocked; from its first progress on, and for good after a
    /// counted repath (it was stuck on this leg, and its clock is old).
    #[test]
    fn a_unit_leads_once_it_has_got_on_since_its_leg_began() {
        let goal = Vec2::new(500.0, 0.0);
        let mut mover = Mover::default();
        mover.go_to(MoveGoal::Point(goal), None, 40);
        assert!(!leads(&mover), "a new leg is no progress");
        mover.follow(vec![Vec2::ZERO, goal], goal, 0);
        assert!(!leads(&mover), "nor is its path");
        mover.last_progress = 41;
        assert!(leads(&mover));
        // Sent again: the clock is its own again.
        mover.go_to(MoveGoal::Point(goal), None, 60);
        assert!(!leads(&mover));
        // Stuck, it looks for another route at tick 92: that clock is shared, old as it is.
        assert!(mover.counted_repath(92));
        assert_eq!((mover.last_progress, mover.last_repath), (60, 92));
        assert!(leads(&mover));
    }

    /// `systems/controls/command_behaviors.md > MovingToLocation > Occupied target location`, and design change 0315's
    /// exemption from it (`factions/syndicate/units/agent.md > Gathering > Gathering rules`, "Waiting at the hatch"):
    /// stuck behind a standing unit within 3 grid units of its goal, a unit has arrived, unless its leg presses on.
    #[test]
    fn a_leg_that_presses_on_never_arrives_behind_a_standing_unit() {
        let blocker = Body {
            id: ObjectId(2),
            owner: Some(PlayerId(1)),
            pos: Vec2::new(140.0, 96.0),
            half: Vec2::splat(18.0),
            oriented: None,
            velocity: Vec2::ZERO,
            heading: Vec2::ZERO,
            stationary: true,
            progress: None,
            arrived_with: None,
            yields: false,
            gathers: false,
            leads: false,
            moved: 0,
        };
        let (pos, goal) = (Vec2::new(100.0, 96.0), Vec2::new(180.0, 96.0));
        let mut mover = Mover::default();
        mover.go_to(MoveGoal::Point(goal), None, 0);
        mover.follow(vec![pos, goal], goal, 0);
        mover.blocked_ticks = JAMMED_TICKS;
        let arrived = |mover: &Mover| {
            let (shape, owner) = (Shape::rect(pos, Vec2::splat(18.0)), Some(PlayerId(1)));
            target_is_occupied(pos, shape, mover, None, Some(&blocker), owner, 40)
        };
        assert!(arrived(&mover));
        mover.press_on();
        assert!(!arrived(&mover));
    }

    /// (G) A unit shut out by its own group avoids the tiles of all the units that arrived with that order, the
    /// nearest first, up to the cap; never its goal's tile, nor units of another order.
    #[test]
    fn a_unit_shut_out_by_its_group_avoids_the_tiles_of_all_that_arrived_with_it() {
        let body = |id: u32, x: f32, y: f32, arrived_with: Option<u32>| Body {
            id: ObjectId(id),
            owner: Some(PlayerId(1)),
            pos: Vec2::new(x, y),
            half: Vec2::splat(18.0),
            oriented: None,
            velocity: Vec2::ZERO,
            heading: Vec2::ZERO,
            stationary: true,
            progress: None,
            arrived_with,
            yields: true,
            gathers: false,
            leads: true,
            moved: 0,
        };
        // The unit stands in tile (1, 1) and goes to tile (7, 1).
        let pos = Vec2::new(100.0, 96.0);
        let mut mover = Mover {
            path: vec![pos, Vec2::new(480.0, 96.0)],
            ..Mover::default()
        };
        let bodies = [
            body(1, 264.0, 96.0, Some(7)),
            body(2, 200.0, 160.0, Some(7)),
            // In the unit's own tile: the tile next to it on that side, (2, 1).
            body(3, 120.0, 100.0, Some(7)),
            body(4, 200.0, 96.0, Some(7)),
            // Another order's, and one on the goal's tile.
            body(5, 150.0, 40.0, Some(8)),
            body(6, 480.0, 100.0, Some(7)),
        ];
        avoid_group(pos, &mut mover, &bodies, 7);
        let tile = TilePos::new;
        assert_eq!(mover.avoid_tiles, [tile(2, 1), tile(3, 1), tile(3, 2), tile(4, 1)]);

        // A long row of them, in tiles (2, 5) to (25, 5), listed from the far end: the nearest MAX_AVOID_TILES.
        let row: Vec<Body> = (0..24)
            .map(|i| body(i, 1600.0 - 64.0 * i as f32, 352.0, Some(7)))
            .collect();
        let mut mover = Mover {
            path: vec![pos, Vec2::new(480.0, 96.0)],
            ..Mover::default()
        };
        avoid_group(pos, &mut mover, &row, 7);
        let nearest: Vec<TilePos> = (2..2 + MAX_AVOID_TILES as i32).map(|x| tile(x, 5)).collect();
        assert_eq!(mover.avoid_tiles, nearest);
    }
}
