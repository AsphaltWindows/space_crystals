//! One player's vision (`core/vision.md > Vision`): the state of each of its tiles, and the counts of its
//! sources that see each one. [`super::Vision`] keeps one per player and updates them in phase 5f.

use glam::Vec2;

use super::VisibilityState;
use super::memory::Memory;
use crate::combat::object_box;
use crate::ids::PlayerId;
use crate::object::Object;
use crate::units::{SU_PER_GU, TilePos};

/// One tile per bit, row-major.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct TileBits {
    pub(super) words: Vec<u64>,
}

impl TileBits {
    pub(super) fn new(tiles: usize) -> Self {
        Self {
            words: vec![0; tiles.div_ceil(64)],
        }
    }

    pub(super) fn get(&self, i: usize) -> bool {
        self.words.get(i / 64).is_some_and(|w| (w >> (i % 64)) & 1 == 1)
    }

    pub(super) fn set(&mut self, i: usize) {
        if let Some(w) = self.words.get_mut(i / 64) {
            *w |= 1 << (i % 64);
        }
    }

    pub(super) fn clear(&mut self, i: usize) {
        if let Some(w) = self.words.get_mut(i / 64) {
            *w &= !(1 << (i % 64));
        }
    }
}

/// One player's vision (`core/vision.md > Vision`).
#[derive(Clone, Debug)]
pub struct PlayerVision {
    pub(super) id: PlayerId,
    pub(super) width: u32,
    pub(super) height: u32,
    /// `> Explored`: every tile this player has seen. History: hashed and serialized.
    pub(super) explored: TileBits,
    /// How many of the player's sources see each tile; Visible where it isn't 0. Derived.
    pub(super) seers: Vec<u32>,
    /// Bumped by each update that changed the state of one of the player's tiles. Output for views, not state.
    pub(super) revision: u32,
    /// A tile changed state in the update under way.
    pub(super) dirty: bool,
    /// The tiles that became Visible, or stopped being Visible, in the update under way (a tile may come twice):
    /// what the memory looks at. Scratch.
    pub(super) flips: Vec<u32>,
    /// `> Explored`: the last-known records of the structures and resources the player has seen (history), and which
    /// of those objects it sees now (derived).
    pub(super) memory: Memory,
}

impl PlayerVision {
    pub(super) fn new(id: PlayerId, width: u32, height: u32) -> Self {
        let tiles = width as usize * height as usize;
        Self {
            id,
            width,
            height,
            explored: TileBits::new(tiles),
            seers: vec![0; tiles],
            revision: 0,
            dirty: false,
            flips: Vec::new(),
            memory: Memory::new(tiles),
        }
    }

    pub fn player(&self) -> PlayerId {
        self.id
    }

    /// Changes whenever the state of one of this player's tiles changes (a counter, wrapping). Output, not state: a
    /// loaded save starts it again, so compare it only within one sim.
    pub fn revision(&self) -> u32 {
        self.revision
    }

    pub(super) fn index(&self, tile: TilePos) -> Option<usize> {
        let inside = tile.x >= 0 && tile.y >= 0 && (tile.x as u32) < self.width && (tile.y as u32) < self.height;
        inside.then(|| tile.y as usize * self.width as usize + tile.x as usize)
    }

    pub(super) fn state_at(&self, i: usize) -> VisibilityState {
        if self.seers.get(i).is_some_and(|&n| n > 0) {
            VisibilityState::Visible
        } else if self.explored.get(i) {
            VisibilityState::Explored
        } else {
            VisibilityState::Unexplored
        }
    }

    /// `core/vision.md > VisibilityState` of `tile`; Unexplored off the map.
    pub fn state(&self, tile: TilePos) -> VisibilityState {
        self.index(tile)
            .map_or(VisibilityState::Unexplored, |i| self.state_at(i))
    }

    /// Every tile's state, row by row from the north-west corner.
    pub fn states(&self) -> impl Iterator<Item = VisibilityState> + '_ {
        (0..self.seers.len()).map(|i| self.state_at(i))
    }

    /// The best state among the tiles the box `lo..hi` (space units) overlaps; Unexplored if it lies off the map.
    /// A box only touching a tile's edge doesn't overlap it.
    pub fn box_state(&self, lo: Vec2, hi: Vec2) -> VisibilityState {
        let (x0, x1) = tile_span(lo.x, hi.x, self.width);
        let (y0, y1) = tile_span(lo.y, hi.y, self.height);
        let mut best = VisibilityState::Unexplored;
        for y in y0..y1 {
            for x in x0..x1 {
                best = best.max(self.state_at(y as usize * self.width as usize + x as usize));
                if best == VisibilityState::Visible {
                    return best;
                }
            }
        }
        best
    }

    /// The best state among the tiles `object` covers: those its silhouette box (a unit) or its footprint (a
    /// structure or resource) overlaps.
    pub fn object_state(&self, object: &Object) -> VisibilityState {
        let (lo, hi) = object_box(object);
        self.box_state(lo, hi)
    }

    /// Whether the player sees `object` now: a tile it covers is Visible.
    pub fn sees(&self, object: &Object) -> bool {
        self.object_state(object) == VisibilityState::Visible
    }

    /// Whether the player sees something whose box is `lo..hi` now.
    pub fn sees_box(&self, lo: Vec2, hi: Vec2) -> bool {
        self.box_state(lo, hi) == VisibilityState::Visible
    }

    /// One more source sees tile `i`: Visible from the first one on, and Explored from then on.
    pub(super) fn see(&mut self, i: u32) {
        let t = i as usize;
        self.seers[t] += 1;
        if self.seers[t] == 1 {
            self.explored.set(t);
            self.dirty = true;
            self.flips.push(i);
        }
    }

    /// One source fewer sees tile `i`: Explored once none does.
    pub(super) fn unsee(&mut self, i: u32) {
        let t = i as usize;
        self.seers[t] -= 1;
        if self.seers[t] == 0 {
            self.dirty = true;
            self.flips.push(i);
        }
    }

    /// A source's stamp went from `old` to `new` (both ascending): only the tiles in one of them change.
    pub(super) fn restamp(&mut self, old: &[u32], new: &[u32]) {
        let (mut a, mut b) = (old.iter().peekable(), new.iter().peekable());
        loop {
            match (a.peek(), b.peek()) {
                (Some(&&x), Some(&&y)) if x == y => {
                    a.next();
                    b.next();
                }
                (Some(&&x), Some(&&y)) if x < y => {
                    self.unsee(x);
                    a.next();
                }
                (_, Some(&&y)) => {
                    self.see(y);
                    b.next();
                }
                (Some(&&x), None) => {
                    self.unsee(x);
                    a.next();
                }
                (None, None) => break,
            }
        }
    }
}

/// The tiles `lo..hi` overlaps along one axis, as a half-open range of tile coordinates clamped to `0..len`: from
/// the tile holding `lo` to the last one that starts before `hi`, so a box only touching a tile's edge doesn't
/// overlap it, but a box of no width still covers the tile it lies in.
pub(super) fn tile_span(lo: f32, hi: f32, len: u32) -> (i64, i64) {
    let first = (lo / SU_PER_GU).floor();
    let last = ((hi / SU_PER_GU).ceil() - 1.0).max(first);
    let clamp = |v: f32| (v as i64).clamp(0, i64::from(len));
    (clamp(first), clamp(last + 1.0))
}
