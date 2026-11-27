//! One player's vision (`core/vision.md > Vision`): the state of each of its tiles, the counts of its sources that
//! see each one, the tiles still Visible for the fog delay ([`super::linger`]), and the attackers revealed to it
//! ([`super::reveal`]). [`super::Vision`] keeps one per player and updates them in phase 5f.

use std::collections::BTreeMap;

use glam::Vec2;

use super::VisibilityState;
use super::hidden::Hiding;
use super::linger::Linger;
use super::memory::Memory;
use crate::combat::object_box;
use crate::hash::{StateHash, StateHasher};
use crate::ids::{ObjectId, PlayerId};
use crate::object::Object;
use crate::units::{SU_PER_GU, Tick, TilePos, clamp_to_map};

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

    /// Sets the bits of the first `tiles` tiles, those of the words' padding beyond them left clear (as
    /// [`TileBits::new`] has them, and a save loads them).
    pub(super) fn fill(&mut self, tiles: usize) {
        for (w, word) in self.words.iter_mut().enumerate() {
            let bits = tiles.saturating_sub(w * 64).min(64);
            *word = if bits == 64 { u64::MAX } else { (1u64 << bits) - 1 };
        }
    }

    pub(super) fn clear(&mut self, i: usize) {
        if let Some(w) = self.words.get_mut(i / 64) {
            *w &= !(1 << (i % 64));
        }
    }

    /// The tiles whose bit is set, ascending: a look at each word, and at each set bit.
    pub(super) fn ones(&self) -> impl Iterator<Item = usize> + '_ {
        self.words.iter().enumerate().flat_map(|(w, &word)| {
            let mut rest = word;
            std::iter::from_fn(move || {
                (rest != 0).then(|| {
                    let bit = rest.trailing_zeros() as usize;
                    rest &= rest - 1;
                    w * 64 + bit
                })
            })
        })
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
    /// `> Visible > Fog delay`: the tiles no source sees that are still Visible, and when each turns Explored
    /// ([`super::linger`]). History: hashed and serialized.
    pub(super) linger: Linger,
    /// `> Attacks from outside vision`: the attackers revealed to the player, each with the tick whose phase 5f ends
    /// its reveal ([`super::reveal`]). History: hashed and serialized.
    pub(super) reveals: BTreeMap<ObjectId, Tick>,
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
            linger: Linger::new(tiles),
            reveals: BTreeMap::new(),
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

    /// Whether tile `i` is Visible: a source sees it, or it lingers for the fog delay ([`super::linger`]).
    pub(super) fn visible_at(&self, i: usize) -> bool {
        self.seers.get(i).is_some_and(|&n| n > 0) || self.linger.get(i)
    }

    pub(super) fn state_at(&self, i: usize) -> VisibilityState {
        if self.visible_at(i) {
            VisibilityState::Visible
        } else if self.explored.get(i) {
            VisibilityState::Explored
        } else {
            VisibilityState::Unexplored
        }
    }

    /// `core/vision.md > VisibilityState` of `tile`; Unexplored off the map. A tile is Visible while a source sees
    /// it, and for the fog delay after ([`super::linger`]).
    pub fn state(&self, tile: TilePos) -> VisibilityState {
        self.index(tile)
            .map_or(VisibilityState::Unexplored, |i| self.state_at(i))
    }

    /// Whether `tile` is Visible only for the fog delay, none of the player's sources seeing it any more
    /// ([`super::linger`]). For views and tests; everything else asks [`PlayerVision::state`].
    pub fn lingers(&self, tile: TilePos) -> bool {
        self.index(tile).is_some_and(|i| self.linger.get(i))
    }

    /// Whether the object `id` is revealed to the player now: it shot one of the player's objects from outside the
    /// player's vision lately (`core/vision.md > Attacks from outside vision`, [`super::reveal`]).
    pub fn is_revealed(&self, id: ObjectId) -> bool {
        self.reveals.contains_key(&id)
    }

    /// Every tile's state, row by row from the north-west corner.
    pub fn states(&self) -> impl Iterator<Item = VisibilityState> + '_ {
        (0..self.seers.len()).map(|i| self.state_at(i))
    }

    /// The best state among the tiles the box `lo..hi` (space units) overlaps; Unexplored if it lies off the map.
    /// A box only touching a tile's edge doesn't overlap it. Tiles only: whether the player sees an object is
    /// [`PlayerVision::object_box_state`]'s question.
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

    /// How the player sees `object` ([`PlayerVision::object_box_state`] of the box its silhouette (a unit) or its
    /// footprint (a structure or resource) makes, `combat::object_box`, and of whom it is hidden from now). (M5-H1)
    /// The box is where the object counts as being ([`PlayerVision::map_box`]).
    pub fn object_state(&self, object: &Object) -> VisibilityState {
        let (lo, hi) = self.map_box(object);
        self.object_box_state(object.id, lo, hi, Hiding::of(object))
    }

    /// (M5-H1) The box of `object` (`combat::object_box`) where it counts as being (`Object::map_pos`): a Glider past
    /// the map's edge "stays visible [...] as if it were at the nearest edge tile" (`systems/units.md > Air
    /// Collision`), its box moved with its centre to the point of the map nearest to it.
    pub(super) fn map_box(&self, object: &Object) -> (Vec2, Vec2) {
        let (lo, hi) = object_box(object);
        let shift = clamp_to_map(object.pos, self.width, self.height) - object.pos;
        (lo + shift, hi + shift)
    }

    /// How the player sees the object `id` whose box is `lo..hi` (space units, `combat::object_box`), hidden from the
    /// players `hiding` says (its [`Hiding`]). Never Visible while it is hidden from the player (plan decision T32,
    /// [`super::hidden`]: an owned underground structure, for every player but its owner), whatever its tiles: at
    /// most Explored then, the state of the place. Otherwise Visible while it is revealed to the player
    /// (`core/vision.md > Attacks from outside vision`, [`super::reveal`]), else the best state among the tiles the box
    /// overlaps ([`PlayerVision::box_state`]), the fog delay included. The one question whether the player sees an
    /// object: the behaviours, combat, the command gate, the records and the client's sight all ask it, through
    /// [`PlayerVision::object_state`] or with the box and the [`Hiding`] they hold.
    pub fn object_box_state(&self, id: ObjectId, lo: Vec2, hi: Vec2, hiding: Hiding) -> VisibilityState {
        if hiding.hides_from(self.id) {
            self.box_state(lo, hi).min(VisibilityState::Explored)
        } else if self.is_revealed(id) {
            VisibilityState::Visible
        } else {
            self.box_state(lo, hi)
        }
    }

    /// Whether the player sees `object` now ([`PlayerVision::object_state`]).
    pub fn sees(&self, object: &Object) -> bool {
        self.object_state(object) == VisibilityState::Visible
    }

    /// Whether the player sees the object `id` whose box is `lo..hi` and which `hiding` hides from some players, now
    /// ([`PlayerVision::object_box_state`]).
    pub fn sees_object_box(&self, id: ObjectId, lo: Vec2, hi: Vec2, hiding: Hiding) -> bool {
        self.object_box_state(id, lo, hi, hiding) == VisibilityState::Visible
    }

    /// Whether a tile the box `lo..hi` overlaps is Visible now. Tiles only, for a place such as a remembered
    /// footprint; an object's question is [`PlayerVision::sees_object_box`].
    pub fn sees_box(&self, lo: Vec2, hi: Vec2) -> bool {
        self.box_state(lo, hi) == VisibilityState::Visible
    }

    /// Whether one of the player's sources sees a tile the box `lo..hi` overlaps: [`PlayerVision::sees_box`] without
    /// the fog delay.
    pub(super) fn sources_see_box(&self, lo: Vec2, hi: Vec2) -> bool {
        let (x0, x1) = tile_span(lo.x, hi.x, self.width);
        let (y0, y1) = tile_span(lo.y, hi.y, self.height);
        (y0..y1).any(|y| (x0..x1).any(|x| self.seers[y as usize * self.width as usize + x as usize] > 0))
    }

    /// One more source sees tile `i`: Visible from the first one on, and Explored from then on. A tile still Visible
    /// for the fog delay just stays Visible.
    pub(super) fn see(&mut self, i: u32) {
        let t = i as usize;
        self.seers[t] += 1;
        if self.seers[t] == 1 && !self.linger.stop(i) {
            self.explored.set(t);
            self.dirty = true;
            self.flips.push(i);
        }
    }

    /// One source fewer sees tile `i`, in phase 5f of `tick`. Once none does, the tile stays Visible for the fog
    /// delay ([`super::linger`]), and [`PlayerVision::expire`] turns it Explored when the delay ends.
    pub(super) fn unsee(&mut self, i: u32, tick: Tick) {
        let t = i as usize;
        self.seers[t] -= 1;
        if self.seers[t] == 0 {
            self.linger.start(i, tick);
        }
    }

    /// Phase 5f of `tick`, after the sources: the tiles whose fog delay ends turn Explored, and the reveals that end
    /// are over.
    pub(super) fn expire(&mut self, tick: Tick) {
        let Self {
            linger, dirty, flips, ..
        } = self;
        linger.expire(tick, |i| {
            *dirty = true;
            flips.push(i);
        });
        self.reveals.retain(|_, &mut end| end > tick);
    }

    /// (M5-S7) `core/vision.md > Starting exploration`: every tile Explored, as a game that starts explored has
    /// them before its first look ([`super::Vision::explore_all`]).
    pub(super) fn explore_all(&mut self) {
        self.explored.fill(self.width as usize * self.height as usize);
        self.dirty = true;
    }

    /// A new player's history besides the Explored tiles and the records (a save's, or a copy to check against):
    /// the tiles that linger, as `(tile, end)` pairs, and the attackers revealed, as `(id, end)` pairs. The tiles
    /// that linger are Visible, so Explored too, and the memory looks at them as at any tile that became Visible.
    pub(super) fn restore(
        &mut self,
        lingering: impl IntoIterator<Item = (u32, Tick)>,
        revealed: impl IntoIterator<Item = (ObjectId, Tick)>,
    ) {
        self.linger.restore(lingering);
        let tiles: Vec<u32> = self.linger.tiles().map(|(i, _)| i).collect();
        for i in tiles {
            self.explored.set(i as usize);
            self.dirty = true;
            self.flips.push(i);
        }
        self.reveals.extend(revealed);
    }

    /// The tiles that linger, by index, each with the end of its delay ([`super::linger`]).
    pub(super) fn lingering(&self) -> impl Iterator<Item = (u32, Tick)> + '_ {
        self.linger.tiles()
    }

    /// The attackers revealed to the player, by id, each with the end of its reveal ([`super::reveal`]).
    pub(super) fn revealed(&self) -> impl Iterator<Item = (ObjectId, Tick)> + '_ {
        self.reveals.iter().map(|(&id, &end)| (id, end))
    }

    /// Whether `other` has the same history: the Explored tiles, the tiles that linger, the reveals and the records.
    pub(super) fn same_history(&self, other: &PlayerVision) -> bool {
        self.id == other.id
            && self.explored == other.explored
            && self.linger == other.linger
            && self.reveals == other.reveals
            && self.records().eq(other.records())
    }

    /// A source's stamp went from `old` to `new` (both ascending), in phase 5f of `tick`: only the tiles in one of
    /// them change.
    pub(super) fn restamp(&mut self, old: &[u32], new: &[u32], tick: Tick) {
        let (mut a, mut b) = (old.iter().peekable(), new.iter().peekable());
        loop {
            match (a.peek(), b.peek()) {
                (Some(&&x), Some(&&y)) if x == y => {
                    a.next();
                    b.next();
                }
                (Some(&&x), Some(&&y)) if x < y => {
                    self.unsee(x, tick);
                    a.next();
                }
                (_, Some(&&y)) => {
                    self.see(y);
                    b.next();
                }
                (Some(&&x), None) => {
                    self.unsee(x, tick);
                    a.next();
                }
                (None, None) => break,
            }
        }
    }
}

impl StateHash for PlayerVision {
    /// The history: the Explored tiles, the records (the memory's only state), the tiles that linger and the
    /// reveals. The rest is derived from the objects, or output.
    fn hash(&self, h: &mut StateHasher) {
        let PlayerVision {
            id,
            width: _,
            height: _,
            explored,
            seers: _,
            linger,
            reveals,
            revision: _,
            dirty: _,
            flips: _,
            memory: _,
        } = self;
        id.hash(h);
        h.len(explored.words.len());
        for word in &explored.words {
            word.hash(h);
        }
        h.len(self.records().count());
        for record in self.records() {
            record.hash(h);
        }
        linger.hash(h);
        h.len(reveals.len());
        for (id, end) in reveals {
            id.hash(h);
            end.hash(h);
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
