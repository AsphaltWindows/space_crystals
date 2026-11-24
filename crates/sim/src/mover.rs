//! The `Mover` capability: goal in, motion out — `docs/movement.md > 3`.
//!
//! Behaviours (`orders::update`, phase 5a) write a unit's [`Mover::goal`] and read its [`Mover::status`]; nothing
//! else moves a unit. Path serving (`paths::serve`, 5c) fills [`Mover::path`], and the movement step
//! (`movement::step`, 5d) follows it and commits the unit's position, facing, velocity and channels.
//!
//! Paths are **state, not cache** (`docs/movement.md > 3`): every field is serialized and hashed, so a save loaded
//! mid-move continues bit-identically.
//!
//! The replanning fields (`repaths_left`, `last_repath`, `avoid_tiles`, `best_remaining`, `last_progress`) follow
//! `docs/movement.md > 5.6` and `> 6.5`; `blocked_ticks`, `avoid_side` and `avoid_until` follow `> 6.3`; `sidestep`
//! is friendly yielding, `> 6.6`.

use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::hash::{StateHash, StateHasher};
use crate::ids::ObjectId;
use crate::nav::PathGroup;
use crate::units::{Tick, TilePos};

/// `systems/controls/behaviors.md > MovingToLocation > Failure` (design change 0024; a placeholder to tune): a
/// stuck unit looks for another route at most 3 times per order leg (`docs/movement.md > 5.6`).
pub const MAX_REPATHS: u8 = 3;

/// `docs/movement.md > 3`: at most this many `avoid_tiles` (a technical cap).
pub const MAX_AVOID_TILES: usize = 16;

/// `systems/controls/behaviors.md > MovingToLocation > Failure` (design change 0024; a placeholder to tune): a unit
/// is stuck after 1.5 s (24 frames) without getting any closer to its target (`docs/movement.md > 6.5`).
pub const STUCK_TICKS: Tick = 24;

/// `systems/controls/behaviors.md > MovingToLocation > Failure` (design change 0024; placeholders to tune): a stuck
/// unit waits 0.5 s, 1 s and 2 s before its first, second and third attempt at another route
/// (`docs/movement.md > 5.6`).
pub const REPATH_WAITS: [Tick; MAX_REPATHS as usize] = [8, 16, 32];

/// 0024's "about 7 s worst case": a unit that never gets on is stuck after [`STUCK_TICKS`], waits out the three
/// [`REPATH_WAITS`], and gives up once it is still stuck a whole [`STUCK_TICKS`] after its third attempt: 104 ticks,
/// 6.5 s, after its last progress (`docs/movement.md > 5.6`).
pub const GIVE_UP_TICKS: Tick = STUCK_TICKS + REPATH_WAITS[0] + REPATH_WAITS[1] + REPATH_WAITS[2] + STUCK_TICKS;

/// `systems/units.md > UnitCollision > Ground Collision > Friendly yielding` (design change 0022): the longest
/// "one short sidestep" of a yielding unit, in widths of its own silhouette. A placeholder, TBD in the design
/// (0022, 0023: "sidestep distance"). The unit steps only as far as the mover's lane needs, up to this.
pub const YIELD_SIDESTEP_WIDTHS: f32 = 1.0;

/// Friendly yielding (0022): the shortest sidestep, space units, so that a request always moves the unit a little
/// (tuning const).
pub const YIELD_MIN_SIDESTEP_SU: f32 = 1.0;

/// Friendly yielding (0022) in a dense idle group: a sidestep blocked by another idle unit of the owner passes the
/// request on, at most this many units deep; beyond that the first unit stays an ordinary blocker (tuning const,
/// `docs/movement.md > 6.6`).
pub const YIELD_CHAIN_DEPTH: u8 = 4;

/// Friendly yielding (0022): a sidestep ends this many ticks after the time its distance takes, however far the
/// unit got: time for the units it asked in turn to make room (one tick per link of the chain; tuning const).
pub const YIELD_WAIT_TICKS: u8 = YIELD_CHAIN_DEPTH;

/// `docs/movement.md > 6.5`: a stuck unit that isn't in contact with a unit this very tick waits this many ticks
/// more to meet its blocker again before it recovers without knowing it (tuning const).
pub const STUCK_GRACE_TICKS: Tick = 8;

/// From this many ticks without progress on, a unit on its way counts as held up (`Mover::is_held_up`, for
/// overlays; tuning const).
pub const HALF_STUCK_TICKS: Tick = STUCK_TICKS / 2;

/// `docs/movement.md > 5.6`: a unit runs its staggered checks in the ticks where `(tick + id) % STAGGER == 0`.
pub const STAGGER: Tick = 4;

/// `docs/movement.md > 6.3`: a unit that found no way forward for this many ticks in a row counts as stationary
/// for the others (a tuning const of the movement design).
pub const JAMMED_TICKS: u16 = 12;

/// `docs/movement.md > 6.3`, rule 4: a switch of `avoid_side` commits for this many ticks (tuning const).
pub const AVOID_COMMIT_TICKS: Tick = 16;

/// `systems/controls/behaviors.md > MovingToLocation > Occupied target location` (design change 0026; a placeholder
/// to tune): a unit stuck within 2 grid units of its destination, blocked by units that aren't moving, counts as
/// arrived (`docs/movement.md > 6.5`, rule 1).
pub const OCCUPIED_ARRIVAL_GU: f32 = 2.0;

/// `docs/movement.md > 6.5`, rule 2: a unit queueing behind friendly attackers checks again after this many ticks.
/// Slice E uses it (chasing); tuning const.
pub const QUEUE_RECHECK_TICKS: Tick = 16;

/// What a unit is asked to reach (`docs/movement.md > 3`).
#[derive(Copy, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum MoveGoal {
    /// No goal: the unit stands still, unless it sidesteps for a blocked unit of its owner ([`Sidestep`]).
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
    /// Failure`; the numbers are design change 0024). Two units head-on in a passage too narrow to pass both end here
    /// (design change 0022: moving units never yield or swap).
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

/// The one short sidestep of a yielding unit (`systems/units.md > UnitCollision > Ground Collision > Friendly
/// yielding`, design changes 0022 and 0023; `docs/movement.md > 6.6`): a real move through the collision step, by
/// an idle unit that a blocked unit of its owner asked to make way.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Sidestep {
    /// The way it steps, a unit vector.
    pub dir: Vec2,
    /// How far it still has to go along `dir`, space units.
    pub left: f32,
    /// It ends after this many more ticks, wherever the unit got to.
    pub ticks_left: u8,
    /// How many units the request was passed through: 0 when a moving unit asked ([`YIELD_CHAIN_DEPTH`]).
    pub depth: u8,
    /// The order group of the moving unit that asked. A unit that arrived with that group doesn't yield to it
    /// (design change 0027), so a request passed on keeps it.
    pub for_group: Option<u32>,
}

impl Sidestep {
    /// A sidestep of `dist` space units along `dir` for a unit that moves `speed` space units per tick.
    pub fn new(dir: Vec2, dist: f32, speed: f32, depth: u8, for_group: Option<u32>) -> Self {
        let ticks = (dist / speed).ceil().clamp(1.0, 200.0) as u8;
        Self {
            dir,
            left: dist,
            ticks_left: ticks.saturating_add(YIELD_WAIT_TICKS),
            depth,
            for_group,
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
    /// (`orders::OrderGroup`). A unit that arrived doesn't yield to the members of its group that are still
    /// arriving (design change 0027, `docs/movement.md > 6.6`).
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
    /// The tick of this leg's latest counted repath; before the first, the tick the leg began
    /// (`docs/movement.md > 5.6`).
    pub last_repath: Tick,
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
    /// Friendly yielding (design change 0022): the sidestep the unit is taking for a blocked unit of its owner.
    /// Only a unit without a goal and without an order has one; an order ends it.
    pub sidestep: Option<Sidestep>,
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
            last_repath: 0,
            avoid_tiles: Vec::new(),
            // Finite, like every float in the state (`docs/movement.md > 7`).
            best_remaining: f32::MAX,
            last_progress: 0,
            blocked_ticks: 0,
            avoid_side: Side::Right,
            avoid_until: 0,
            sidestep: None,
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
        self.last_repath = tick;
        self.avoid_tiles.clear();
        self.best_remaining = f32::MAX;
        self.last_progress = tick;
        self.blocked_ticks = 0;
        self.avoid_side = Side::Right;
        self.avoid_until = tick;
        self.sidestep = None;
    }

    /// No goal: the unit stops where it is (instantly, with infinite deceleration) and stands still. `status`
    /// becomes `Idle`. A sidestep ends too: every order stops or sends the unit, and a unit with an order doesn't
    /// yield (design change 0023).
    pub fn stop(&mut self) {
        self.sidestep = None;
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

    /// The wait before the leg's next counted repath (design change 0024: 0.5 s, 1 s, 2 s), [`REPATH_WAITS`].
    fn next_wait(&self) -> Tick {
        let used = usize::from(MAX_REPATHS.saturating_sub(self.repaths_left));
        REPATH_WAITS[used.min(REPATH_WAITS.len() - 1)]
    }

    /// Whether a repath that counts against the cap may happen in `tick`: one is left, and the wait since the
    /// last one (or since the leg began) is over (`docs/movement.md > 5.6`).
    pub fn may_repath(&self, tick: Tick) -> bool {
        self.repaths_left > 0 && tick >= self.last_repath.saturating_add(self.next_wait())
    }

    /// A repath that counts against the leg's cap (`docs/movement.md > 5.6`): takes one of `repaths_left` and notes
    /// its tick. Returns `false`, changing nothing, unless [`Mover::may_repath`].
    pub fn counted_repath(&mut self, tick: Tick) -> bool {
        if !self.may_repath(tick) || self.goal == MoveGoal::None {
            return false;
        }
        self.repaths_left -= 1;
        self.last_repath = tick;
        self.repath(tick);
        true
    }

    /// `systems/controls/behaviors.md > MovingToLocation > Failure` (design change 0024): whether the unit is
    /// **stuck** in `tick`: [`STUCK_TICKS`] have passed without it getting any closer.
    pub fn is_stuck(&self, tick: Tick) -> bool {
        tick.saturating_sub(self.last_progress) >= STUCK_TICKS
    }

    /// Design change 0024, for a unit that stays stuck: the tick it next looks for another route, the wait of
    /// [`REPATH_WAITS`] after it became stuck or after its previous attempt, whichever is later; with no attempt
    /// left, the tick it gives up, [`STUCK_TICKS`] after the last one. Without any progress that is 32, 48 and 80
    /// ticks after the last progress, and 104 ([`GIVE_UP_TICKS`]).
    pub fn recovery_due(&self) -> Tick {
        let stuck_at = self.last_progress.saturating_add(STUCK_TICKS);
        if self.repaths_left == 0 {
            return stuck_at.max(self.last_repath.saturating_add(STUCK_TICKS));
        }
        stuck_at.max(self.last_repath).saturating_add(self.next_wait())
    }

    /// The tick since which the unit's current path has shown no progress: its latest progress, or its latest
    /// counted repath if that was later (`docs/movement.md > 6.5`).
    pub fn stuck_clock(&self) -> Tick {
        self.last_progress.max(self.last_repath)
    }

    /// `systems/units.md > UnitCollision > Ground Collision > Friendly yielding` (design change 0023): whether the
    /// unit can move at all in its current mode, at `max_speed` space units per tick. Modes that only slow it
    /// (a raised Riot Shield, `speed_factor` 0.5) leave it able to, and it sidesteps at that speed. **The hook for
    /// units that can't move**: a deployed or deploying Artillery Rig will answer `false` here (its mode stops it),
    /// and then it never yields.
    pub fn can_move(&self, max_speed: f32) -> bool {
        max_speed * self.speed_factor > 0.0
    }

    /// Whether the unit is on its way but hasn't got on for `HALF_STUCK_TICKS`: on the way to being stuck, or
    /// stuck and recovering (`docs/movement.md > 6.5`). For overlays.
    pub fn is_held_up(&self, tick: Tick) -> bool {
        self.status == MoveStatus::Moving
            && self.path_state == PathState::Ready
            && tick.saturating_sub(self.stuck_clock()) >= HALF_STUCK_TICKS
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
            last_repath,
            avoid_tiles,
            best_remaining,
            last_progress,
            blocked_ticks,
            avoid_side,
            avoid_until,
            sidestep,
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
        last_repath.hash(h);
        avoid_tiles.hash(h);
        best_remaining.hash(h);
        last_progress.hash(h);
        blocked_ticks.hash(h);
        avoid_side.hash(h);
        avoid_until.hash(h);
        sidestep.hash(h);
    }
}

impl StateHash for Sidestep {
    fn hash(&self, h: &mut StateHasher) {
        let Sidestep {
            dir,
            left,
            ticks_left,
            depth,
            for_group,
        } = self;
        dir.hash(h);
        left.hash(h);
        ticks_left.hash(h);
        depth.hash(h);
        for_group.hash(h);
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

    /// Design change 0024, `docs/movement.md > 5.6`: three repaths per leg, after waits of 8, 16 and 32 ticks.
    #[test]
    fn counted_repaths_are_capped_and_wait() {
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
        assert_eq!(
            at,
            [18, 34, 66],
            "8, 16 and 32 ticks after the leg began or the repath before"
        );
        assert_eq!((mover.last_repath, mover.stuck_clock()), (66, 66));
        mover.follow(vec![Vec2::ZERO, Vec2::ONE], Vec2::ONE, 0);
        assert!(!mover.is_held_up(66 + HALF_STUCK_TICKS - 1) && mover.is_held_up(66 + HALF_STUCK_TICKS));
        mover.last_progress = 70;
        assert_eq!(mover.stuck_clock(), 70);
        assert_eq!(mover.repaths_left, 0);
        // A new leg starts afresh.
        mover.go_to(MoveGoal::Point(Vec2::ONE), None, 200);
        assert_eq!((mover.repaths_left, mover.may_repath(208)), (MAX_REPATHS, true));
        assert_eq!(mover.stuck_clock(), 200);
    }

    /// Design change 0024: stuck after 24 ticks without getting closer; another route after waits of 8, 16 and 32
    /// ticks; then it gives up, 104 ticks (6.5 s, "about 7 s") after its last progress.
    #[test]
    fn a_unit_that_never_gets_on_follows_the_schedule_of_0024() {
        let mut mover = Mover::default();
        mover.go_to(MoveGoal::Point(Vec2::ONE), None, 100);
        assert!(!mover.is_stuck(123) && mover.is_stuck(124));
        let mut due = Vec::new();
        for _ in 0..MAX_REPATHS {
            let tick = mover.recovery_due();
            assert!(mover.counted_repath(tick));
            due.push(tick - 100);
        }
        assert_eq!(due, [32, 48, 80]);
        assert_eq!(mover.recovery_due() - 100, GIVE_UP_TICKS);
        assert_eq!(GIVE_UP_TICKS, 104);
        // Progress in between: stuck again 24 ticks later, and the next attempt waits from then.
        let mut mover = Mover::default();
        mover.go_to(MoveGoal::Point(Vec2::ONE), None, 100);
        assert!(mover.counted_repath(132));
        mover.last_progress = 150;
        assert_eq!(mover.recovery_due(), 150 + 24 + 16);
    }

    /// Design change 0023: modes that slow a unit leave it able to move; a unit that can't move is the hook's
    /// `false`.
    #[test]
    fn a_unit_can_move_unless_its_mode_stops_it() {
        let mut mover = Mover::default();
        assert!(mover.can_move(4.0));
        mover.speed_factor = 0.5;
        assert!(mover.can_move(4.0), "a raised Riot Shield only slows it");
        mover.speed_factor = 0.0;
        assert!(!mover.can_move(4.0));
        assert_eq!(
            Sidestep::new(Vec2::X, 24.0, 2.0, 0, None).ticks_left,
            12 + YIELD_WAIT_TICKS
        );
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
