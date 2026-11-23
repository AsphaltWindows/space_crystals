//! The `Mover` capability: goal in, motion out — `docs/movement.md > 3`.
//!
//! Behaviours (`orders::update`, phase 5a) write a unit's [`Mover::goal`] and read its [`Mover::status`]; nothing
//! else moves a unit. Path serving (`paths::serve`, 5c) fills [`Mover::path`], and the movement step
//! (`movement::step`, 5d) follows it and commits the unit's position, facing, velocity and channels.
//!
//! Paths are **state, not cache** (`docs/movement.md > 3`): every field is serialized and hashed, so a save loaded
//! mid-move continues bit-identically.
//!
//! Several fields belong to steering, collision and stuck recovery (`docs/movement.md > 6`, slice D). This slice
//! initialises and resets them only; D gives them their behaviour.

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
    /// Gave up: no path (`systems/controls/behaviors.md > MovingToLocation > Failure`).
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
    /// Ticks in a row the unit found no way forward (`docs/movement.md > 6.3`, rule 5).
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
        self.clear_path();
        self.path_state = PathState::Wanted { since: tick, group };
        self.repaths_left = MAX_REPATHS;
        self.repath_ready_at = tick;
        self.avoid_tiles.clear();
        self.best_remaining = f32::MAX;
        self.last_progress = tick;
        self.blocked_ticks = 0;
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
    /// `planned_target` on the grids of `nav_version`.
    pub fn follow(&mut self, path: Vec<Vec2>, planned_target: Vec2, nav_version: u32) {
        // Head for the first waypoint after the start.
        self.next = u16::from(path.len() > 1);
        self.path = path;
        self.path_state = PathState::Ready;
        self.nav_version = nav_version;
        self.planned_target = planned_target;
    }

    /// Plans the current goal again from where the unit is now, keeping the leg's repath count.
    pub fn repath(&mut self, tick: Tick) {
        if self.goal != MoveGoal::None {
            self.clear_path();
            self.path_state = PathState::Wanted {
                since: tick,
                group: None,
            };
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
    }
}
