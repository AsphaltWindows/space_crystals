//! The state hash of the mover and its parts (`hash.rs`): every field, destructured, so that a new one doesn't
//! compile until it is hashed.

use super::{
    Locomotion, MoveGoal, MoveStatus, Mover, Orientation, PathState, Side, Sidestep, StepAside, Strafe, Takeoff,
};
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
            exit,
            momentum,
            exit_heading,
            detour_end,
            presses_on,
            step_aside,
            orbit,
            strafe,
            takeoff,
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
        exit.hash(h);
        // (M3-G) Only while the unit has some: the models that stop within the tick never do, so the states of
        // games without a DragMovement unit hash as they did before the field existed (the goldens of M1 and
        // M2 stand). A state with momentum differs from the same state without by the tag and the vector.
        if *momentum != glam::Vec2::ZERO {
            h.tag(1);
            momentum.hash(h);
        }
        // (M3-D2) Likewise only while set, under a tag of its own: games without a unit in a portal's energy doorway
        // hash as before the field existed.
        if let Some(heading) = exit_heading {
            h.tag(2);
            heading.hash(h);
        }
        // (M3-G2) Likewise only while the unit is on a detour round standing units.
        if *detour_end != 0 {
            h.tag(3);
            detour_end.hash(h);
        }
        // (M4-W2, design change 0315) Likewise only on a leg that never counts as arrived short of its goal: an
        // Agent's leg back to a hatch.
        if *presses_on {
            h.tag(4);
        }
        // (M4-W3, design change 0323) Likewise only while the unit is taking a contact step.
        if let Some(StepAside { along_y, to }) = step_aside {
            h.tag(5);
            along_y.hash(h);
            to.hash(h);
        }
        // (M5-H1) Likewise only while a Glider circles a point, and while it is on a strafing run: games without a
        // Glider hash as before the fields existed (`mover::flight`).
        if let Some(orbit) = orbit {
            h.tag(6);
            orbit.hash(h);
        }
        if let Some(Strafe { range, pass }) = strafe {
            h.tag(7);
            range.hash(h);
            // Fieldless enum: the discriminant is the declaration index.
            h.tag(*pass as u8);
        }
        // (M5-N7) Likewise only while a Glider is on its takeoff run: games without a runway hash as before the field
        // existed.
        if let Some(Takeoff { end, along }) = takeoff {
            h.tag(8);
            end.hash(h);
            along.hash(h);
        }
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
