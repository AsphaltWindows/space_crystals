//! The `Mover` capability: goal in, motion out — `docs/movement.md > 3`.
//!
//! Behaviours (`orders::update`, phase 5a) write a unit's [`Mover::goal`] and read its [`Mover::status`]; nothing
//! else moves a unit. Path serving (`paths::serve`, 5c) fills [`Mover::path`], and the movement step
//! (`movement::step`, 5d) follows it and commits the unit's position, facing, velocity and channels.
//!
//! Paths are **state, not cache** (`docs/movement.md > 3`): every field is serialized and hashed, so a save loaded
//! mid-move continues bit-identically.
//!
//! The replanning fields (`repaths_left`, `repath_ready_at`, `avoid_tiles`, `best_remaining`, `last_progress`) follow
//! `docs/movement.md > 5.6` and `> 6.5`; `blocked_ticks`, `avoid_side` and `avoid_until` follow `> 6.3`.

use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::hash::{StateHash, StateHasher};
use crate::ids::ObjectId;
use crate::nav::PathGroup;
use crate::units::{Tick, TilePos};

/// `docs/movement.md > 5.6`: repaths per order leg. Default until question Q0008 (gap G4) is answered: 3.
pub const MAX_REPATHS: u8 = 3;

/// `docs/movement.md > 3`: at most this many `avoid_tiles` (a technical cap).
pub const MAX_AVOID_TILES: usize = 16;

/// `docs/movement.md > 6.5`: a unit is stuck after this many ticks without progress. Default until question Q0008
/// (gap G4) is answered: 1.5 s.
pub const STUCK_TICKS: Tick = 24;

/// `docs/movement.md > 5.6`: after its n-th repath a unit plans no other for this many ticks (the backoff doubles).
/// Default until question Q0008 (gap G4) is answered: 0.5 s, 1 s, 2 s.
pub const REPATH_BACKOFF: [Tick; MAX_REPATHS as usize] = [8, 16, 32];

/// `docs/movement.md > 6.5`: a stuck unit that isn't in contact with a unit this very tick waits this many ticks
/// more to meet its blocker again before it recovers without knowing it (tuning const).
pub const STUCK_GRACE_TICKS: Tick = 8;

/// `docs/movement.md > 6.5`, `> 5.6`: from this many ticks without progress on, a unit notes the stationary units
/// in its way for its next repath, and a unit that lost sight of its steering waypoint repaths (tuning const).
pub const HALF_STUCK_TICKS: Tick = STUCK_TICKS / 2;

/// `docs/movement.md > 5.6`: a unit runs its staggered checks in the ticks where `(tick + id) % STAGGER == 0`.
pub const STAGGER: Tick = 4;

/// `docs/movement.md > 6.3`: a unit that found no way forward for this many ticks in a row counts as stationary
/// for the others (a tuning const of the movement design).
pub const JAMMED_TICKS: u16 = 12;

/// `docs/movement.md > 6.3`, rule 4: a switch of `avoid_side` commits for this many ticks (tuning const).
pub const AVOID_COMMIT_TICKS: Tick = 16;

/// `docs/movement.md > 6.5`, rule 1: a unit stuck behind stationary units within this many grid units of its goal
/// point counts as arrived. Default until question Q0008 (gap G5) is answered: 1.5 GU.
pub const CROWDED_ARRIVAL_GU: f32 = 1.5;

/// `docs/movement.md > 6.5`, rule 2: a unit queueing behind friendly attackers checks again after this many ticks.
/// Slice E uses it (chasing); tuning const.
pub const QUEUE_RECHECK_TICKS: Tick = 16;

/// What a unit is asked to reach (`docs/movement.md > 3`).
#[derive(Copy, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum MoveGoal {
    /// No goal: the unit stands still. Idle units are never displaced (G2, question Q0007).
    #[default]
    None,
    /// A point, space units.
    Point(Vec2),
    /// An object, until the unit is within `reach` space units of it (`systems/controls/behaviors.md >
    /// MovingToObject`; measured as in G7, question Q0009). Behaviours use it from slice E on.
    Object { id: ObjectId, reach: f32 },
}

/// How the current goal is going (`docs/movement.md > 3`). Behaviours read it in phase 5a.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum MoveStatus {
    /// No goal was given since the last outcome was read, or the unit was stopped.
    #[default]
    Idle,
    /// On the way, or waiting for its path.
    Moving,
    /// Reached the goal, or the reachable point nearest to it (G5, question Q0008).
    Arrived,
    /// Gave up: no path, or stuck with no repath left (`systems/controls/behaviors.md > MovingToLocation >
    /// Failure`; the numbers are question Q0008). Two units head-on in a passage too narrow to pass both end here
    /// (question Q0007, gap G3).
    Failed,
    /// The goal object no longer exists.
    TargetLost,
}

/// `systems/controls/behaviors.md > LocomotionChannel`.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Locomotion {
    /// Moving along the path.
    Moving,
    /// Moving backwards along the path (bases with CanReverse only; none in M1).
    Reversing,
    /// Slowing down to a stop. With infinite deceleration (every M1 unit) it never lasts beyond the tick.
    Stopping,
    /// Not moving.
    #[default]
    Stationary,
}

/// `systems/controls/behaviors.md > OrientationChannel`. Not [`crate::Orientation`], the map symmetry type.
#[derive(Copy, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum Orientation {
    /// Rotating towards a point, at most the base's turn rate per tick.
    Turning(Vec2),
    /// Holding the current facing.
    #[default]
    Maintaining,
}

/// Where the unit's path stands (`docs/movement.md > 3`, `> 5.5`).
#[derive(Copy, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum PathState {
    /// No path, none wanted.
    #[default]
    None,
    /// A path is wanted since tick `since`; phase 5c serves requests in `(since, id)` order. `group` is the order
    /// group it was ordered with, for a shared field (`docs/movement.md > 5.5`).
    Wanted { since: Tick, group: Option<PathGroup> },
    /// `path` holds the path.
    Ready,
}

/// Which way a unit sidesteps an oncoming one (`docs/movement.md > 6.3`, rule 4).
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Side {
    /// Everyone starts with Right, so head-on units pass like traffic.
    #[default]
    Right,
    Left,
}

impl Side {
    pub fn other(self) -> Side {
        match self {
            Side::Right => Side::Left,
            Side::Left => Side::Right,
        }
    }
}

/// A unit's movement state (`docs/movement.md > 3`). Only units have one.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Mover {
    pub goal: MoveGoal,
    pub status: MoveStatus,
    /// The design's channels (`systems/controls/behaviors.md > BaseActionChannels`); combat and crushing read them.
    pub locomotion: Locomotion,
    pub orientation: Orientation,
    /// This tick's displacement, space units per tick; zero when the unit stood still or arrived.
    pub velocity: Vec2,
    /// MaxSpeed is multiplied by this: 0.5 under a raised Riot Shield (`factions/gdo/units.md > Peacekeeper`).
    pub speed_factor: f32,
    /// The order group of the current goal, or of the last one: the units of one command share it
    /// (`orders::OrderGroup`). A unit stuck behind a stationary unit that arrived with its own group has arrived
    /// too (`docs/movement.md > 6.5`, rule 1).
    pub group: Option<u32>,
    /// The waypoints of the current path, from its start to its goal; empty without a path.
    pub path: Vec<Vec2>,
    /// Index into `path` of the waypoint the unit is heading for.
    pub next: u16,
    pub path_state: PathState,
    /// The sim's `nav_version` when the path was planned; a different one means the obstacles changed.
    pub nav_version: u32,
    /// The point the path was planned to.
    pub planned_target: Vec2,
    /// Repaths left for this leg (`docs/movement.md > 5.6`).
    pub repaths_left: u8,
    /// No repath before this tick (the backoff, `docs/movement.md > 5.6`).
    pub repath_ready_at: Tick,
    /// Tiles a repath treats as blocked, at most [`MAX_AVOID_TILES`] (`docs/movement.md > 6.5`).
    pub avoid_tiles: Vec<TilePos>,
    /// Shortest remaining path length seen on this leg, space units (`docs/movement.md > 6.5`).
    pub best_remaining: f32,
    /// The tick `best_remaining` last improved.
    pub last_progress: Tick,
    /// Ticks in a row the unit found no way forward (`docs/movement.md > 6.3`, rule 5): it stayed, only closed
    /// up to its blocker, or only stepped sideways (the 90° sidestep).
    pub blocked_ticks: u16,
    /// The side the unit sidesteps to (`docs/movement.md > 6.3`, rule 4).
    pub avoid_side: Side,
    /// `avoid_side` doesn't switch before this tick.
    pub avoid_until: Tick,
}

impl Default for Mover {
    /// An idle unit.
    fn default() -> Self {
        Self {
            goal: MoveGoal::None,
            status: MoveStatus::Idle,
            locomotion: Locomotion::Stationary,
            orientation: Orientation::Maintaining,
            velocity: Vec2::ZERO,
            speed_factor: 1.0,
            group: None,
            path: Vec::new(),
            next: 0,
            path_state: PathState::None,
            nav_version: 0,
            planned_target: Vec2::ZERO,
            repaths_left: MAX_REPATHS,
            repath_ready_at: 0,
            avoid_tiles: Vec::new(),
            // Finite, like every float in the state (`docs/movement.md > 7`).
            best_remaining: f32::MAX,
            last_progress: 0,
            blocked_ticks: 0,
            avoid_side: Side::Right,
            avoid_until: 0,
        }
    }
}

impl Mover {
    /// A behaviour sends the unit to `goal` (a new leg, `docs/movement.md > 5.6`): it wants a path from tick `tick`
    /// on and stands still until it has one. `group` is the order group, for a shared path field.
    pub fn go_to(&mut self, goal: MoveGoal, group: Option<PathGroup>, tick: Tick) {
        self.goal = goal;
        self.status = MoveStatus::Moving;
        self.group = group.map(|g| g.id);
        self.clear_path();
        self.path_state = PathState::Wanted { since: tick, group };
        self.repaths_left = MAX_REPATHS;
        self.repath_ready_at = tick;
        self.avoid_tiles.clear();
        self.best_remaining = f32::MAX;
        self.last_progress = tick;
        self.blocked_ticks = 0;
        self.avoid_side = Side::Right;
        self.avoid_until = tick;
    }

    /// No goal: the unit stops where it is (instantly, with infinite deceleration) and stands still. `status`
    /// becomes `Idle`.
    pub fn stop(&mut self) {
        self.goal = MoveGoal::None;
        self.status = MoveStatus::Idle;
        self.clear_path();
        self.velocity = Vec2::ZERO;
        self.locomotion = Locomotion::Stationary;
        self.orientation = Orientation::Maintaining;
    }

    /// The current goal ends with `status` (`Arrived`, `Failed` or `TargetLost`): no goal, no path, standing still.
    pub fn finish(&mut self, status: MoveStatus) {
        self.stop();
        self.status = status;
    }

    /// Phase 5c hands the unit its path: waypoints from the unit's position to the goal, planned to
    /// `planned_target` on the grids of `nav_version`. Progress is measured afresh on the new path, but the stuck
    /// clock (`last_progress`) runs on: a new path alone is no progress (`docs/movement.md > 6.5`).
    pub fn follow(&mut self, path: Vec<Vec2>, planned_target: Vec2, nav_version: u32) {
        // Head for the first waypoint after the start.
        self.next = u16::from(path.len() > 1);
        self.path = path;
        self.path_state = PathState::Ready;
        self.nav_version = nav_version;
        self.planned_target = planned_target;
        self.best_remaining = f32::MAX;
        self.blocked_ticks = 0;
    }

    /// Plans the current goal again from where the unit is now, keeping the leg's repath count: the repaths that
    /// don't count against the cap (the ground changed, a teleport; `docs/movement.md > 5.6`).
    pub fn repath(&mut self, tick: Tick) {
        if self.goal != MoveGoal::None {
            self.clear_path();
            self.path_state = PathState::Wanted {
                since: tick,
                group: None,
            };
        }
    }

    /// Whether a repath that counts against the cap may happen in `tick`: one is left, and the backoff of the
    /// last one is over (`docs/movement.md > 5.6`).
    pub fn may_repath(&self, tick: Tick) -> bool {
        self.repaths_left > 0 && tick >= self.repath_ready_at
    }

    /// A repath that counts against the leg's cap (`docs/movement.md > 5.6`): takes one of `repaths_left` and
    /// starts the backoff, [`REPATH_BACKOFF`]. Returns `false`, changing nothing, unless [`Mover::may_repath`].
    pub fn counted_repath(&mut self, tick: Tick) -> bool {
        if !self.may_repath(tick) || self.goal == MoveGoal::None {
            return false;
        }
        let used = usize::from(MAX_REPATHS - self.repaths_left);
        self.repaths_left -= 1;
        self.repath_ready_at = tick.saturating_add(REPATH_BACKOFF[used.min(REPATH_BACKOFF.len() - 1)]);
        self.repath(tick);
        true
    }

    /// The tick the unit's stuck clock runs from (`docs/movement.md > 6.5`): its latest progress, or its latest
    /// counted repath if that was later, so that every new path gets `STUCK_TICKS` to show progress. The repath's
    /// tick is `repath_ready_at` less the backoff it started.
    pub fn stuck_clock(&self) -> Tick {
        let used = usize::from(MAX_REPATHS.saturating_sub(self.repaths_left));
        let last_repath = match used.checked_sub(1).and_then(|i| REPATH_BACKOFF.get(i)) {
            Some(backoff) => self.repath_ready_at.saturating_sub(*backoff),
            None => 0,
        };
        self.last_progress.max(last_repath)
    }

    /// `docs/movement.md > 6.3`: whether the unit counts as stationary for the others: its locomotion is
    /// Stationary, or it found no way forward for [`JAMMED_TICKS`].
    pub fn is_stationary(&self) -> bool {
        self.locomotion == Locomotion::Stationary || self.blocked_ticks >= JAMMED_TICKS
    }

    /// The point the current path ends on: the goal, or the reachable point nearest to it.
    pub fn path_end(&self) -> Option<Vec2> {
        match self.path_state {
            PathState::Ready => self.path.last().copied(),
            PathState::None | PathState::Wanted { .. } => None,
        }
    }

    /// The waypoints still ahead, from the one the unit is heading for; empty without a path.
    pub fn remaining_path(&self) -> &[Vec2] {
        match self.path_state {
            PathState::Ready => self.path.get(usize::from(self.next)..).unwrap_or(&[]),
            PathState::None | PathState::Wanted { .. } => &[],
        }
    }

    fn clear_path(&mut self) {
        self.path.clear();
        self.next = 0;
        self.path_state = PathState::None;
    }
}

impl StateHash for MoveGoal {
    fn hash(&self, h: &mut StateHasher) {
        match self {
            MoveGoal::None => h.tag(0),
            MoveGoal::Point(p) => {
                h.tag(1);
                p.hash(h);
            }
            MoveGoal::Object { id, reach } => {
                h.tag(2);
                id.hash(h);
                reach.hash(h);
            }
        }
    }
}

impl StateHash for MoveStatus {
    fn hash(&self, h: &mut StateHasher) {
        // Fieldless enum: the discriminant is the declaration index.
        h.tag(*self as u8);
    }
}

impl StateHash for Locomotion {
    fn hash(&self, h: &mut StateHasher) {
        h.tag(*self as u8);
    }
}

impl StateHash for Orientation {
    fn hash(&self, h: &mut StateHasher) {
        match self {
            Orientation::Turning(toward) => {
                h.tag(0);
                toward.hash(h);
            }
            Orientation::Maintaining => h.tag(1),
        }
    }
}

impl StateHash for PathState {
    fn hash(&self, h: &mut StateHasher) {
        match self {
            PathState::None => h.tag(0),
            PathState::Wanted { since, group } => {
                h.tag(1);
                since.hash(h);
                match group {
                    None => h.tag(0),
                    Some(PathGroup { id, anchor }) => {
                        h.tag(1);
                        id.hash(h);
                        anchor.hash(h);
                    }
                }
            }
            PathState::Ready => h.tag(2),
        }
    }
}

impl StateHash for Side {
    fn hash(&self, h: &mut StateHasher) {
        h.tag(*self as u8);
    }
}

impl StateHash for Mover {
    fn hash(&self, h: &mut StateHasher) {
        let Mover {
            goal,
            status,
            locomotion,
            orientation,
            velocity,
            speed_factor,
            group,
            path,
            next,
            path_state,
            nav_version,
            planned_target,
            repaths_left,
            repath_ready_at,
            avoid_tiles,
            best_remaining,
            last_progress,
            blocked_ticks,
            avoid_side,
            avoid_until,
        } = self;
        goal.hash(h);
        status.hash(h);
        locomotion.hash(h);
        orientation.hash(h);
        velocity.hash(h);
        speed_factor.hash(h);
        group.hash(h);
        path.hash(h);
        next.hash(h);
        path_state.hash(h);
        nav_version.hash(h);
        planned_target.hash(h);
        repaths_left.hash(h);
        repath_ready_at.hash(h);
        avoid_tiles.hash(h);
        best_remaining.hash(h);
        last_progress.hash(h);
        blocked_ticks.hash(h);
        avoid_side.hash(h);
        avoid_until.hash(h);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn go_to_wants_a_path_and_stop_drops_it() {
        let mut mover = Mover {
            path: vec![Vec2::ZERO, Vec2::ONE],
            next: 1,
            path_state: PathState::Ready,
            velocity: Vec2::X,
            ..Mover::default()
        };
        assert_eq!(mover.remaining_path(), [Vec2::ONE]);
        let goal = MoveGoal::Point(Vec2::new(5.0, 6.0));
        mover.go_to(goal, None, 7);
        assert_eq!((mover.goal, mover.status), (goal, MoveStatus::Moving));
        assert_eq!(mover.path_state, PathState::Wanted { since: 7, group: None });
        assert!(mover.path.is_empty() && mover.remaining_path().is_empty());
        mover.stop();
        assert_eq!(
            (mover.goal, mover.status, mover.path_state),
            (MoveGoal::None, MoveStatus::Idle, PathState::None)
        );
        assert_eq!(mover.velocity, Vec2::ZERO);
        mover.finish(MoveStatus::Arrived);
        assert_eq!(mover.status, MoveStatus::Arrived);
        // Without a goal there is nothing to plan.
        mover.repath(9);
        assert_eq!(mover.path_state, PathState::None);
        assert!(!mover.counted_repath(9));
    }

    /// `docs/movement.md > 5.6`: three repaths per leg, 8, 16 and 32 ticks apart at least.
    #[test]
    fn counted_repaths_are_capped_and_back_off() {
        let mut mover = Mover::default();
        mover.go_to(MoveGoal::Point(Vec2::ONE), None, 10);
        let mut at = Vec::new();
        for tick in 10..200 {
            if mover.counted_repath(tick) {
                at.push(tick);
                assert_eq!(
                    mover.path_state,
                    PathState::Wanted {
                        since: tick,
                        group: None
                    }
                );
            }
        }
        assert_eq!(at, [10, 18, 34], "8 and 16 ticks apart");
        assert_eq!(mover.repath_ready_at, 34 + 32);
        assert_eq!(mover.stuck_clock(), 34, "the latest repath restarts the stuck clock");
        mover.last_progress = 40;
        assert_eq!(mover.stuck_clock(), 40);
        assert_eq!(mover.repaths_left, 0);
        // A new leg starts afresh.
        mover.go_to(MoveGoal::Point(Vec2::ONE), None, 200);
        assert_eq!((mover.repaths_left, mover.may_repath(200)), (MAX_REPATHS, true));
        assert_eq!(mover.stuck_clock(), 200);
    }

    #[test]
    fn a_new_path_measures_afresh_but_the_stuck_clock_runs_on() {
        let mut mover = Mover::default();
        mover.go_to(MoveGoal::Point(Vec2::X), None, 1);
        mover.best_remaining = 3.0;
        mover.blocked_ticks = JAMMED_TICKS;
        mover.locomotion = Locomotion::Moving;
        assert!(mover.is_stationary());
        mover.follow(vec![Vec2::ZERO, Vec2::X], Vec2::X, 2);
        assert_eq!((mover.best_remaining, mover.last_progress), (f32::MAX, 1));
        assert!(!mover.is_stationary());
        assert_eq!(mover.path_end(), Some(Vec2::X));
    }
}
