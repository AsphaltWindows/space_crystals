//! Progress (plan decision T54, a mechanic implemented once): ticks done of a total, in the fixed point of plan
//! decision T6, so that a tick at part speed counts its share of a tick (a GDO building's power ratio,
//! `economy::PowerSummary::ratio`). [`Progress::step`] takes it one tick on, [`Progress::done`] tells whether every
//! tick is done.
//!
//! A transformation counts its frames with one (`crate::transform`: a Warp In, a Beacon Deployment). The other timers
//! of the sim keep their own state, because a `Progress` would change their hashed bytes: the producer's and the
//! constructor's progress (`crate::production`, `crate::construction`) and the Extraction Facility's cycle
//! (`economy::mining`) store no total, which their recipes and defs give; a Weapon Conversion counts the frames left
//! (`crate::conversion`), a Riot Shield how far up it is, both ways (`crate::riot_shield`); a worker's mining and
//! unloading count whole frames done of its def's durations (`orders::state::GatherPhase`), and the Tunnel Network the
//! tick a unit went in (`network::Held`). Each can take a `Progress` at the next re-bless of the goldens that touches
//! it; a new timer takes one from the start.

use serde::{Deserialize, Serialize};

use crate::economy::RATIO_ONE;
use crate::hash::{StateHash, StateHasher};

/// Ticks done of a total (module docs).
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Progress {
    /// How far it has come: [`RATIO_ONE`] for each whole tick done, a share of it for a tick at part speed. Every
    /// tick is done at `total × RATIO_ONE`. `progress` in files, the name a transformation's files give it.
    #[serde(rename = "progress")]
    pub elapsed: u32,
    /// How many ticks it takes at full speed.
    pub total: u32,
}

impl Progress {
    /// No tick done yet of `total`.
    pub fn new(total: u32) -> Self {
        Self { elapsed: 0, total }
    }

    /// One tick at `speed`, in [`RATIO_ONE`]s of a tick (`RATIO_ONE` at full speed): whether every tick is done now.
    /// Saturating: the totals of the defs are short enough for their progress to fit (`ObjectDef` consistency tests),
    /// a hand-edited save's needn't be.
    pub fn step(&mut self, speed: u32) -> bool {
        self.elapsed = self.elapsed.saturating_add(speed);
        self.done()
    }

    /// Whether every tick is done.
    pub fn done(&self) -> bool {
        self.elapsed >= self.total.saturating_mul(RATIO_ONE)
    }

    /// The whole ticks done, at most [`Progress::total`].
    pub fn ticks_done(&self) -> u32 {
        (self.elapsed / RATIO_ONE).min(self.total)
    }

    /// The whole ticks done of the total, from 0 to 1 (1 for a total of none).
    pub fn ratio(&self) -> f32 {
        match self.total {
            0 => 1.0,
            total => self.ticks_done() as f32 / total as f32,
        }
    }

    /// The whole percent done, rounded down: 100 only once every tick is done.
    pub fn percent(&self) -> u32 {
        match self.total {
            0 => 100,
            total => (u64::from(self.ticks_done()) * 100 / u64::from(total)) as u32,
        }
    }
}

impl StateHash for Progress {
    /// The ticks done, then the total: the bytes a transformation hashed before it kept a `Progress`.
    fn hash(&self, h: &mut StateHasher) {
        let Progress { elapsed, total } = self;
        elapsed.hash(h);
        total.hash(h);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Half-speed ticks count half a tick each; the whole ticks done stop at the total, and a hand-edited progress
    /// saturates.
    #[test]
    fn ticks_count_their_share_and_saturate() {
        let mut p = Progress::new(4);
        assert_eq!((p.ticks_done(), p.percent(), p.ratio(), p.done()), (0, 0, 0.0, false));
        assert!(!p.step(RATIO_ONE / 2));
        assert_eq!(p.ticks_done(), 0, "half a tick isn't one");
        assert!(!p.step(RATIO_ONE / 2));
        assert_eq!((p.ticks_done(), p.percent(), p.ratio()), (1, 25, 0.25));
        assert!(!p.step(2 * RATIO_ONE));
        assert_eq!(p.percent(), 75, "100 only once every tick is done");
        assert!(p.step(RATIO_ONE));
        assert_eq!((p.ticks_done(), p.percent(), p.ratio(), p.done()), (4, 100, 1.0, true));
        p.elapsed = u32::MAX - 1;
        assert!(p.step(RATIO_ONE) && p.elapsed == u32::MAX && p.ticks_done() == 4);
        let none = Progress::new(0);
        assert_eq!((none.done(), none.percent(), none.ratio()), (true, 100, 1.0));
    }

    /// The ticks done, then the total, as two `u32`s: the bytes a transformation hashed before it kept a `Progress`.
    #[test]
    fn hashes_as_its_two_numbers() {
        let p = Progress {
            elapsed: 10 * RATIO_ONE + 3,
            total: 48,
        };
        let (mut one, mut two) = (StateHasher::new(), StateHasher::new());
        p.hash(&mut one);
        p.elapsed.hash(&mut two);
        p.total.hash(&mut two);
        assert_eq!(one.finish(), two.finish());
    }
}
