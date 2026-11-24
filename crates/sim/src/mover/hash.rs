//! The state hash of the mover and its parts (`hash.rs`): every field, destructured, so that a new one doesn't
//! compile until it is hashed.

use super::{Locomotion, MoveGoal, MoveStatus, Mover, Orientation, PathState, Side, Sidestep};
use crate::hash::{StateHash, StateHasher};
use crate::nav::PathGroup;

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
