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
//!
//! - `schedule`: when a unit counts as stuck, when it looks for another route and when it gives up (design change
//!   0024, `docs/movement.md > 5.6`).
//! - `sidestep`: the one short sidestep of a yielding unit (`docs/movement.md > 6.6`).
//! - `hash`: the state hash of the mover and its parts.

mod hash;
mod schedule;
mod sidestep;
#[cfg(test)]
mod tests;

use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::ids::ObjectId;
use crate::nav::PathGroup;
use crate::units::{Tick, TilePos};

pub use schedule::{GIVE_UP_TICKS, HALF_STUCK_TICKS, MAX_REPATHS, REPATH_WAITS, STUCK_GRACE_TICKS, STUCK_TICKS};
pub use sidestep::{Sidestep, YIELD_CHAIN_DEPTH, YIELD_MIN_SIDESTEP_SU, YIELD_SIDESTEP_WIDTHS, YIELD_WAIT_TICKS};

/// `docs/movement.md > 3`: at most this many `avoid_tiles` (a technical cap).
pub const MAX_AVOID_TILES: usize = 16;

/// `docs/movement.md > 5.6`: a unit runs its staggered checks in the ticks where `(tick + id) % STAGGER == 0`.
pub const STAGGER: Tick = 4;

/// `docs/movement.md > 6.3`: a unit that found no way forward for this many ticks in a row counts as stationary
/// for the others (a tuning const of the movement design).
pub const JAMMED_TICKS: u16 = 12;

/// `docs/movement.md > 6.3`, rule 4: a switch of `avoid_side` commits for this many ticks (tuning const).
pub const AVOID_COMMIT_TICKS: Tick = 16;

/// `systems/controls/behaviors.md > MovingToLocation > Occupied target location` (design changes 0026 and 0169; a
/// placeholder to tune): a unit stuck within 3 grid units of its destination, blocked by units that aren't moving,
/// counts as arrived (`docs/movement.md > 6.5`, rule 1). The one radius of both behaviours: `> MovingToObject`'s
/// "Blocked near the target" is the same rule (design changes 0033 and 0169). It was 2 until design change 0169.
pub const OCCUPIED_ARRIVAL_GU: f32 = 3.0;

/// What a unit is asked to reach (`docs/movement.md > 3`).
#[derive(Copy, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum MoveGoal {
    /// No goal: the unit stands still, unless it sidesteps for a blocked unit of its owner ([`Sidestep`]).
    #[default]
    None,
    /// A point, space units.
    Point(Vec2),
    /// An object, until the gap between the unit's shape and the object's is at most `reach` space units, edge to
    /// edge (`systems/controls/behaviors.md > MovingToObject > Proximity`; `shape`). The path is kept aimed at the
    /// object as it moves (`paths::chase_check`). The behaviours' chases and follows (`orders::engage`,
    /// `orders::travel`).
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
    /// Reached the goal, or the reachable point nearest to it (design change 0025), or stopped within
    /// [`OCCUPIED_ARRIVAL_GU`] of it behind units that won't move (design changes 0026, 0032, 0033 and 0169).
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
    /// Slowing down to a stop. With infinite deceleration (every M1 unit) it never lasts beyond the tick. A
    /// DragMovement unit without a goal is Stopping while it brakes off its [`Mover::momentum`] (slice M3-G).
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
    /// MaxSpeed is multiplied by this. The unit's modes set it when they change: 0.5 from the moment its Riot Shield
    /// starts going up until it is all the way down again (`riot_shield`, `factions/gdo/units.md > Peacekeeper`).
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
    /// The version of its owner's path knowledge when the path was planned (`PlayerKnowledge::version`; the sim's
    /// `nav_version` for a unit without an owner): a different one means the obstacles its owner knows of changed,
    /// and the unit checks its path again (`systems/controls/behaviors.md > PathKnowledge`).
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
    /// The producing structure whose exit doorway the unit is still in: it was released there and its box still
    /// overlaps the footprint (`core/entities.md > Structure Type > Exit Doorway`, design change 0080;
    /// `production::Doorway`). While it is set, that footprint's tiles don't block the unit, which may only come
    /// out; phase 5d clears it once the unit is clear of the footprint, for good. Optional in files, so that
    /// states saved before it existed still load.
    #[serde(default)]
    pub exit: Option<ObjectId>,
    /// The velocity a unit with momentum carries into its next tick, space units per tick (slice M3-G:
    /// `systems/units.md > DragMovement`, `movement::drag`): its latest displacement, with whatever an obstacle
    /// blocked of it dropped (design change 0035). Always zero for the models that stop within the tick. Unlike
    /// `velocity` it outlives the goal: [`Mover::stop`] and [`Mover::finish`] leave it, and the movement step
    /// brakes the unit to a stop from it (`Locomotion::Stopping`). Left out of files while zero, so that states
    /// saved before it existed still load, and of the hash while zero (`mover/hash.rs`).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub momentum: Vec2,
}

fn is_zero(v: &Vec2) -> bool {
    *v == Vec2::ZERO
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
            exit: None,
            momentum: Vec2::ZERO,
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
    /// yield (design change 0023). A unit with [`Mover::momentum`] keeps it: from its next turn in phase 5d it
    /// brakes to a stop (`movement::drag`), and only then is at rest.
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
    /// `planned_target` on its owner's grids of knowledge version `nav_version`. Progress is measured afresh on the new path, but the stuck
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

    /// `systems/units.md > UnitCollision > Ground Collision > Friendly yielding` (design change 0023): whether the
    /// unit can move at all in its current mode, at `max_speed` space units per tick. Modes that only slow it
    /// (a raised Riot Shield, `speed_factor` 0.5) leave it able to, and it sidesteps at that speed. **The hook for
    /// units that can't move**: a deployed or deploying Artillery Rig will answer `false` here (its mode stops it),
    /// and then it never yields.
    pub fn can_move(&self, max_speed: f32) -> bool {
        max_speed * self.speed_factor > 0.0
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
