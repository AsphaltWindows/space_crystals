//! When a unit on its way counts as stuck, when it looks for another route and when it gives up — design change
//! 0024 (`systems/controls/command_behaviors.md > MovingToLocation > Failure`), `docs/movement.md > 5.6`,
//! `docs/movement.md > 6.5`.

use glam::Vec2;

use super::{MoveGoal, MoveStatus, Mover, PathState};
use crate::ruleset::rules;
use crate::units::Tick;

impl Mover {
    /// The wait before the leg's next counted repath (design change 0024: 0.5 s, 1 s, 2 s), `Movement::repath_waits`.
    fn next_wait(&self) -> Tick {
        let tuning = &rules().tuning.movement;
        let used = usize::from(tuning.max_repaths.saturating_sub(self.repaths_left));
        tuning.repath_waits[used.min(tuning.repath_waits.len() - 1)]
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

    /// (M3-G2) Whether an attempt at a way round a row of standing units may happen in `tick`
    /// ([`Mover::go_round`]): the wait since the last attempt (or since the leg began) is over, and one is left,
    /// or this one is `free`.
    pub fn may_go_round(&self, tick: Tick, free: bool) -> bool {
        (free || self.repaths_left > 0) && tick >= self.last_repath.saturating_add(self.next_wait())
    }

    /// (M3-G2) An attempt at a way round a row of standing units (`movement::round`), which counts against the
    /// leg's cap like a repath and waits like one, but asks for no path: `detour` (waypoints from `from`, where the
    /// unit stands, round the row) goes in front of what is left of its path; with none, only the attempt is
    /// counted. Progress is measured afresh, as on a new path. Returns `false`, changing nothing, unless
    /// [`Mover::may_repath`].
    pub fn go_round(&mut self, from: Vec2, detour: &[Vec2], tick: Tick, free: bool) -> bool {
        if !self.may_go_round(tick, free) || self.goal == MoveGoal::None {
            return false;
        }
        if !free {
            self.repaths_left -= 1;
        }
        self.last_repath = tick;
        if !detour.is_empty() {
            // What is left of an earlier detour goes: this one starts from where the unit is.
            let ahead = self
                .path
                .split_off(usize::from(self.next.max(self.detour_end)).min(self.path.len()));
            self.path.clear();
            self.path.push(from);
            self.path.extend_from_slice(detour);
            self.detour_end = self.path.len() as u16;
            self.path.extend(ahead);
            self.next = 1;
            self.best_remaining = f32::MAX;
            self.blocked_ticks = 0;
            // (0323) The detour is the unit's way round now: a contact step it was taking ends.
            self.step_aside = None;
        }
        true
    }

    /// `systems/controls/command_behaviors.md > MovingToLocation > Failure` (design change 0024): whether the unit is
    /// **stuck** in `tick`: `Movement::stuck_ticks` have passed without it getting any closer.
    pub fn is_stuck(&self, tick: Tick) -> bool {
        let tuning = &rules().tuning.movement;
        tick.saturating_sub(self.last_progress) >= tuning.stuck_ticks
    }

    /// Design change 0024, for a unit that stays stuck: the tick it next looks for another route, the wait of
    /// `Movement::repath_waits` after it became stuck or after its previous attempt, whichever is later; with no
    /// attempt left, the tick it gives up, `Movement::stuck_ticks` after the last one. Without any progress that is 32,
    /// 48 and 80 ticks after the last progress, and 104 (`tuning::movement::Movement::give_up_ticks`).
    pub fn recovery_due(&self) -> Tick {
        let tuning = &rules().tuning.movement;
        let stuck_at = self.last_progress.saturating_add(tuning.stuck_ticks);
        if self.repaths_left == 0 {
            return stuck_at.max(self.last_repath.saturating_add(tuning.stuck_ticks));
        }
        stuck_at.max(self.last_repath).saturating_add(self.next_wait())
    }

    /// The tick since which the unit's current path has shown no progress: its latest progress, or its latest
    /// counted repath if that was later (`docs/movement.md > 6.5`).
    pub fn stuck_clock(&self) -> Tick {
        self.last_progress.max(self.last_repath)
    }

    /// Whether the unit is on its way but hasn't got on for `Movement::half_stuck_ticks`: on the way to being stuck, or
    /// stuck and recovering (`docs/movement.md > 6.5`). For overlays.
    pub fn is_held_up(&self, tick: Tick) -> bool {
        let tuning = &rules().tuning.movement;
        self.status == MoveStatus::Moving
            && self.path_state == PathState::Ready
            && tick.saturating_sub(self.stuck_clock()) >= tuning.half_stuck_ticks()
    }
}
