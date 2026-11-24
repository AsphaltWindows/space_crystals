//! When a unit on its way counts as stuck, when it looks for another route and when it gives up — design change
//! 0024 (`systems/controls/behaviors.md > MovingToLocation > Failure`), `docs/movement.md > 5.6`, `> 6.5`.

use super::{MoveGoal, MoveStatus, Mover, PathState};
use crate::units::Tick;

/// `systems/controls/behaviors.md > MovingToLocation > Failure` (design change 0024; a placeholder to tune): a
/// stuck unit looks for another route at most 3 times per order leg (`docs/movement.md > 5.6`).
pub const MAX_REPATHS: u8 = 3;

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

/// `docs/movement.md > 6.5`: a stuck unit that isn't in contact with a unit this very tick waits this many ticks
/// more to meet its blocker again before it recovers without knowing it (tuning const).
pub const STUCK_GRACE_TICKS: Tick = 8;

/// From this many ticks without progress on, a unit on its way counts as held up (`Mover::is_held_up`, for
/// overlays; tuning const).
pub const HALF_STUCK_TICKS: Tick = STUCK_TICKS / 2;

impl Mover {
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

    /// Whether the unit is on its way but hasn't got on for `HALF_STUCK_TICKS`: on the way to being stuck, or
    /// stuck and recovering (`docs/movement.md > 6.5`). For overlays.
    pub fn is_held_up(&self, tick: Tick) -> bool {
        self.status == MoveStatus::Moving
            && self.path_state == PathState::Ready
            && tick.saturating_sub(self.stuck_clock()) >= HALF_STUCK_TICKS
    }
}
