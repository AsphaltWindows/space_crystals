//! The fog delay — `core/vision.md > Visible > Fog delay` (design change 0178, answering question Q0018 point 1): a
//! tile that leaves the sight of all of a player's sources stays Visible for `Vision::fog_delay_ticks` before it turns
//! Explored, and one that comes back into sight meanwhile simply stays Visible. During the delay the tile is
//! genuinely Visible: everything on it shows and counts as seen, for the player's units as for its views. So a target
//! at the very edge of sight no longer blinks out of view for a tick. This is the one fix for that: AttackingObject
//! and Follow have no grace period of their own (design change 0200, Q0051's option A: "the fog delay stays per
//! tile"), and their target lost from vision ends them (`orders::engage`, `orders::travel`) once the delay is over.
//!
//! - **Ticks**: a tile that no source sees any more in phase 5f of tick `t` lingers. It is Visible at the end of ticks
//!   `t` to `t + Vision::fog_delay_ticks − 1`, and phase 5f of tick `t + Vision::fog_delay_ticks` (its end) turns it
//!   Explored, unless a source sees it again first. The last tick it was in sight is `t − 1`, so it stays Visible for
//!   the 8 ticks after it ("Visible if in sight now or within 8 frames", 0178's Implementation impact).
//! - **State**: the tiles that linger, and the tick each one's delay ends. Nothing in the objects says when a tile
//!   left sight, so both are history, like the Explored tiles: hashed, serialized, and kept by a loaded save.
//! - **Cost**: phase 5f touches the tiles that leave sight or come back, as it did without the delay, and those whose
//!   delay ends, never the whole map. Every tile that leaves sight joins a queue, in the order its delay ends
//!   ([`Linger::expiry`]), and 5f pops the ones due from its front. A tile that came back into sight leaves its entry
//!   behind, and the pop skips it. A tile that turns Explored changes the player's revision, so views redraw then.
//!   The memory treats it as a tile that left the sight (`PlayerVision::flips`).

use std::collections::VecDeque;

use super::player::TileBits;
use crate::hash::{StateHash, StateHasher};
use crate::ruleset::rules;
use crate::units::Tick;

/// One player's tiles that linger: still Visible for the fog delay, though none of its sources sees them.
#[derive(Clone, Debug)]
pub(super) struct Linger {
    /// The tiles that linger. State.
    bits: TileBits,
    /// For each tile that lingers, the tick whose phase 5f turns it Explored: its end. State; meaningless where
    /// the tile doesn't linger.
    ends: Vec<Tick>,
    /// `(end, tile)` for each tile that started to linger, by ascending end, those that have since come back into
    /// sight included (stale: [`Linger::expire`] skips them). Derived: rebuilt from the tiles and their ends.
    expiry: VecDeque<(Tick, u32)>,
}

impl Linger {
    pub(super) fn new(tiles: usize) -> Self {
        Self {
            bits: TileBits::new(tiles),
            ends: vec![0; tiles],
            expiry: VecDeque::new(),
        }
    }

    /// Whether tile `i` lingers.
    pub(super) fn get(&self, i: usize) -> bool {
        self.bits.get(i)
    }

    /// The end of tile `i`'s delay, if it lingers.
    pub(super) fn end(&self, i: usize) -> Option<Tick> {
        self.get(i).then(|| self.ends[i])
    }

    /// Phase 5f of `tick`: no source sees tile `i` any more. It lingers until phase 5f of `tick +
    /// Vision::fog_delay_ticks`. The ticks of successive calls never go down, so the queue stays in order.
    pub(super) fn start(&mut self, i: u32, tick: Tick) {
        let tuning = &rules().tuning.vision;
        self.set(i, tick + tuning.fog_delay_ticks);
    }

    fn set(&mut self, i: u32, end: Tick) {
        let t = i as usize;
        self.bits.set(t);
        self.ends[t] = end;
        self.expiry.push_back((end, i));
    }

    /// A source sees tile `i` again, so it no longer lingers. Returns whether it did: if so, it was Visible all along.
    pub(super) fn stop(&mut self, i: u32) -> bool {
        let t = i as usize;
        let lingered = self.bits.get(t);
        if lingered {
            self.bits.clear(t);
        }
        lingered
    }

    /// Phase 5f of `tick`: the tiles whose delay ends now stop lingering, and each goes to `expired`: it turns
    /// Explored. Pops the queue's due entries only.
    pub(super) fn expire(&mut self, tick: Tick, mut expired: impl FnMut(u32)) {
        while let Some(&(end, i)) = self.expiry.front()
            && end <= tick
        {
            self.expiry.pop_front();
            if self.end(i as usize) == Some(end) {
                self.bits.clear(i as usize);
                expired(i);
            }
        }
    }

    /// How many entries the queue holds, stale ones included.
    #[cfg(test)]
    pub(super) fn queued(&self) -> usize {
        self.expiry.len()
    }

    /// The tiles that linger, by ascending index, each with the end of its delay.
    pub(super) fn tiles(&self) -> impl Iterator<Item = (u32, Tick)> + '_ {
        self.bits.ones().map(|i| (i as u32, self.ends[i]))
    }

    /// A new player's tiles start lingering as `tiles` say, `(tile, end)` pairs, each tile once: a save's, or a copy to
    /// check against.
    pub(super) fn restore(&mut self, tiles: impl IntoIterator<Item = (u32, Tick)>) {
        let mut entries: Vec<(Tick, u32)> = tiles.into_iter().map(|(i, end)| (end, i)).collect();
        entries.sort_unstable();
        for (end, i) in entries {
            self.set(i, end);
        }
    }
}

impl PartialEq for Linger {
    /// The same tiles linger, with the same ends; the queue is derived.
    fn eq(&self, other: &Self) -> bool {
        self.bits == other.bits && self.tiles().eq(other.tiles())
    }
}

impl StateHash for Linger {
    /// The tiles that linger, by index, and their ends.
    fn hash(&self, h: &mut StateHasher) {
        let Linger { bits, ends, expiry: _ } = self;
        let tiles = || bits.ones().map(|i| (i as u32, ends[i]));
        h.len(tiles().count());
        for (i, end) in tiles() {
            i.hash(h);
            end.hash(h);
        }
    }
}
