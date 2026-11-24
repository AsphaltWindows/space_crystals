//! Vision — `core/vision.md`. Phase 5f (`docs/movement.md > 4`): what each player sees, from the tick's final
//! positions.
//!
//! - **States** (`> VisibilityState`): every tile is [`VisibilityState::Unexplored`], `Explored` or `Visible` for
//!   each player. Explored is history: one bit per tile and player, set the first time the tile is Visible and never
//!   cleared, so it is hashed and serialized with the rest of the state. Visible is derived: a tile is Visible while
//!   at least one of the player's sight sources sees it. That is a count per tile, kept up to date incrementally,
//!   never hashed and rebuilt from the objects when a save is loaded.
//! - **Sources** (`core/entities.md > Object Type > SightRange`: vision goes to the owner): every owned object whose
//!   type has a SightRange above 0. Neutral objects and resources (`core/map.md > Resource`: no vision) see nothing.
//! - **Sight** ([`sight`]): a disc around the object's centre. A tile is seen when its centre lies within the
//!   SightRange, the boundary included, with the ElevationModifier between the object and the ground at the tile's
//!   centre (`core/vision.md > ElevationModifier`): one grid unit more towards lower tiles, one less towards higher
//!   ones; air units take none. No terrain blocks sight: the design has no line of sight.
//! - **Update** (incremental stamps): each source keeps the tiles it stamped and its [`Eye`], everything the stamp
//!   depends on (owner, centre, SightRange, elevation). Phase 5f walks the objects in id order along with the
//!   sources. A source whose eye is unchanged costs nothing; one that moved stamps its disc again, and only the
//!   tiles that differ change their counts; a source that is gone takes its tiles away. Each player's
//!   [`PlayerVision::revision`] changes whenever one of its tiles changed state, so views redraw only then.
//! - **Timing**: phase 5f sees every object at the end of the tick, those spawned in it included, so Visible is a
//!   function of the end-of-tick state and a loaded save sees what the running game saw. The next tick reads it:
//!   combat's target checks in 5e (`systems/combat.md > ValidTarget` 2) and, later, the behaviours in 5a see last
//!   tick's visibility (`docs/movement.md > 4`).
//! - **Objects** ([`PlayerVision::object_state`]): an object counts as seen when any tile it covers is Visible; it
//!   covers the tiles its silhouette box (a unit) or footprint (a structure or resource) overlaps.
//!
//! The map never changes during a game; a sim whose map changed would have to build its vision again
//! ([`Vision::new`]).

mod sight;
#[cfg(test)]
mod tests;

use std::fmt;

use glam::Vec2;
use serde::{Deserialize, Serialize, Serializer};

use crate::combat::object_box;
use crate::elevation::Elevation;
use crate::hash::{StateHash, StateHasher};
use crate::ids::{ObjectId, PlayerId};
use crate::map::Map;
use crate::object::Object;
use crate::units::{SU_PER_GU, TilePos};

pub use sight::TILE_SIGHT_POINT;
use sight::stamp;

/// `core/vision.md > VisibilityState`: a tile as one player sees it. Ordered from the least to the most seen, so the
/// best state of a few tiles is their `max`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum VisibilityState {
    /// `> Unexplored`: never within the SightRange of the player's units or structures. Fully black: terrain,
    /// structures and units are hidden.
    Unexplored,
    /// `> Explored`: seen before, not now. Terrain shows, structures and resources in their last-known state, enemy
    /// units not at all.
    Explored,
    /// `> Visible`: within the SightRange of one of the player's units or structures now. Everything shows.
    Visible,
}

/// Everything a sight source's stamp depends on, besides the map. A source whose eye didn't change sees the same
/// tiles.
#[derive(Copy, Clone, Debug, PartialEq)]
struct Eye {
    /// Index of the owner in [`Vision`]'s players.
    player: usize,
    /// The object's centre, space units.
    at: Vec2,
    /// Its type's SightRange, grid units; above 0.
    sight: f32,
    /// Its side of the ElevationModifier: its elevation level, or exempt (air units).
    elevation: Elevation,
}

/// A sight source: an object that gives its owner vision, and the tiles it stamped.
#[derive(Clone, Debug)]
struct Source {
    id: ObjectId,
    eye: Eye,
    /// The tiles it sees, as tile indexes (row-major), ascending.
    tiles: Vec<u32>,
}

/// One tile per bit, row-major.
#[derive(Clone, Debug, PartialEq, Eq)]
struct TileBits {
    words: Vec<u64>,
}

impl TileBits {
    fn new(tiles: usize) -> Self {
        Self {
            words: vec![0; tiles.div_ceil(64)],
        }
    }

    fn get(&self, i: usize) -> bool {
        self.words.get(i / 64).is_some_and(|w| (w >> (i % 64)) & 1 == 1)
    }

    fn set(&mut self, i: usize) {
        if let Some(w) = self.words.get_mut(i / 64) {
            *w |= 1 << (i % 64);
        }
    }
}

/// One player's vision (`core/vision.md > Vision`).
#[derive(Clone, Debug)]
pub struct PlayerVision {
    id: PlayerId,
    width: u32,
    height: u32,
    /// `> Explored`: every tile this player has seen. History: hashed and serialized.
    explored: TileBits,
    /// How many of the player's sources see each tile; Visible where it isn't 0. Derived.
    seers: Vec<u32>,
    /// Bumped by each update that changed the state of one of the player's tiles. Output for views, not state.
    revision: u32,
    /// A tile changed state in the update under way.
    dirty: bool,
}

impl PlayerVision {
    fn new(id: PlayerId, width: u32, height: u32) -> Self {
        let tiles = width as usize * height as usize;
        Self {
            id,
            width,
            height,
            explored: TileBits::new(tiles),
            seers: vec![0; tiles],
            revision: 0,
            dirty: false,
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

    fn index(&self, tile: TilePos) -> Option<usize> {
        let inside = tile.x >= 0 && tile.y >= 0 && (tile.x as u32) < self.width && (tile.y as u32) < self.height;
        inside.then(|| tile.y as usize * self.width as usize + tile.x as usize)
    }

    fn state_at(&self, i: usize) -> VisibilityState {
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
    fn see(&mut self, i: u32) {
        let i = i as usize;
        self.seers[i] += 1;
        if self.seers[i] == 1 {
            self.explored.set(i);
            self.dirty = true;
        }
    }

    /// One source fewer sees tile `i`: Explored once none does.
    fn unsee(&mut self, i: u32) {
        let i = i as usize;
        self.seers[i] -= 1;
        if self.seers[i] == 0 {
            self.dirty = true;
        }
    }

    /// A source's stamp went from `old` to `new` (both ascending): only the tiles in one of them change.
    fn restamp(&mut self, old: &[u32], new: &[u32]) {
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
fn tile_span(lo: f32, hi: f32, len: u32) -> (i64, i64) {
    let first = (lo / SU_PER_GU).floor();
    let last = ((hi / SU_PER_GU).ceil() - 1.0).max(first);
    let clamp = |v: f32| (v as i64).clamp(0, i64::from(len));
    (clamp(first), clamp(last + 1.0))
}

/// Every player's vision (`core/vision.md`), at the end of the latest tick.
#[derive(Clone)]
pub struct Vision {
    width: u32,
    height: u32,
    /// The elevation level of the ground at each tile's sight point ([`TILE_SIGHT_POINT`]), row-major: the target
    /// side of the ElevationModifier. Derived from the map.
    tile_levels: Vec<u8>,
    /// In the sim's player order.
    players: Vec<PlayerVision>,
    /// The sight sources, by ascending object id. Derived from the objects.
    sources: Vec<Source>,
    /// Scratch for a new stamp.
    scratch: Vec<u32>,
}

impl Vision {
    /// No vision yet: every tile Unexplored for each of `players`, on `map`. [`Vision::update`] adds the sources.
    pub fn new(map: &Map, players: impl IntoIterator<Item = PlayerId>) -> Self {
        let (width, height) = (map.width(), map.height());
        let mut tile_levels = Vec::with_capacity(width as usize * height as usize);
        for y in 0..height as i32 {
            for x in 0..width as i32 {
                tile_levels.push(map.elevation_level(TilePos::new(x, y).corner() + TILE_SIGHT_POINT));
            }
        }
        Self {
            width,
            height,
            tile_levels,
            players: players
                .into_iter()
                .map(|id| PlayerVision::new(id, width, height))
                .collect(),
            sources: Vec::new(),
            scratch: Vec::new(),
        }
    }

    /// The vision of a loaded save: `explored` per player as saved, the rest derived from `objects` (all of them,
    /// in id order). Tiles Visible now count as Explored even if the save forgot them.
    fn load<'a>(
        map: &Map,
        players: impl IntoIterator<Item = PlayerId>,
        explored: Vec<TileBits>,
        objects: impl IntoIterator<Item = &'a Object>,
    ) -> Self {
        let mut vision = Self::new(map, players);
        for (player, bits) in vision.players.iter_mut().zip(explored) {
            player.explored = bits;
        }
        vision.update(map, objects);
        vision
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    /// `player`'s vision; `None` for a player who isn't in the game.
    pub fn player(&self, player: PlayerId) -> Option<&PlayerVision> {
        self.players.iter().find(|p| p.id == player)
    }

    /// Every player's vision, in the sim's player order.
    pub fn players(&self) -> &[PlayerVision] {
        &self.players
    }

    /// `core/vision.md > VisibilityState`: how `player` sees `tile`; Unexplored off the map and for a player who
    /// isn't in the game.
    pub fn state(&self, player: PlayerId, tile: TilePos) -> VisibilityState {
        self.player(player)
            .map_or(VisibilityState::Unexplored, |p| p.state(tile))
    }

    /// The best state among the tiles `object` covers, for `player` ([`PlayerVision::object_state`]).
    pub fn object_state(&self, player: PlayerId, object: &Object) -> VisibilityState {
        self.player(player)
            .map_or(VisibilityState::Unexplored, |p| p.object_state(object))
    }

    /// Whether `player` sees `object` now: a tile it covers is Visible.
    pub fn sees(&self, player: PlayerId, object: &Object) -> bool {
        self.player(player).is_some_and(|p| p.sees(object))
    }

    /// The eye of `o`, if it is a sight source: owned by a player of the game, of a type with a SightRange.
    fn eye(&self, map: &Map, o: &Object) -> Option<Eye> {
        let owner = o.owner?;
        let sight = o.def().sight_range;
        if sight <= 0.0 {
            return None;
        }
        Some(Eye {
            player: self.players.iter().position(|p| p.id == owner)?,
            at: o.pos,
            sight,
            elevation: Elevation::of_object(map, o),
        })
    }

    /// Phase 5f: catches up with `objects`, every object of the sim in ascending id order. New sources stamp the
    /// tiles they see, moved ones stamp again (only the tiles that differ change), and gone ones take their tiles
    /// away; tiles that become Visible become Explored too. Each player whose tiles changed state gets a new
    /// [`PlayerVision::revision`].
    ///
    /// The sim calls it on its own vision; it is public so that tests can time it and check it on a copy.
    pub fn update<'a>(&mut self, map: &Map, objects: impl IntoIterator<Item = &'a Object>) {
        let mut old = std::mem::take(&mut self.sources).into_iter().peekable();
        let mut kept = Vec::with_capacity(old.len());
        for o in objects {
            while let Some(gone) = old.next_if(|s| s.id < o.id) {
                self.withdraw(&gone);
            }
            let previous = old.next_if(|s| s.id == o.id);
            match (previous, self.eye(map, o)) {
                (Some(source), Some(eye)) if source.eye == eye => kept.push(source),
                (Some(mut source), Some(eye)) => {
                    self.restamp(&mut source, eye);
                    kept.push(source);
                }
                (Some(source), None) => self.withdraw(&source),
                (None, Some(eye)) => {
                    let mut tiles = Vec::new();
                    self.stamp_into(&eye, &mut tiles);
                    let player = &mut self.players[eye.player];
                    for &i in &tiles {
                        player.see(i);
                    }
                    kept.push(Source { id: o.id, eye, tiles });
                }
                (None, None) => {}
            }
        }
        for gone in old {
            self.withdraw(&gone);
        }
        self.sources = kept;
        for player in &mut self.players {
            if std::mem::take(&mut player.dirty) {
                player.revision = player.revision.wrapping_add(1);
            }
        }
    }

    fn stamp_into(&self, eye: &Eye, out: &mut Vec<u32>) {
        stamp(eye, self.width, self.height, &self.tile_levels, out);
    }

    /// A source is gone, or no longer gives vision: its tiles lose it.
    fn withdraw(&mut self, source: &Source) {
        let player = &mut self.players[source.eye.player];
        for &i in &source.tiles {
            player.unsee(i);
        }
    }

    /// A source's eye changed: it stamps again, and only the tiles that differ change.
    fn restamp(&mut self, source: &mut Source, eye: Eye) {
        let mut fresh = std::mem::take(&mut self.scratch);
        self.stamp_into(&eye, &mut fresh);
        if source.eye.player == eye.player {
            if fresh != source.tiles {
                self.players[eye.player].restamp(&source.tiles, &fresh);
            }
        } else {
            self.withdraw(source);
            let player = &mut self.players[eye.player];
            for &i in &fresh {
                player.see(i);
            }
        }
        std::mem::swap(&mut source.tiles, &mut fresh);
        source.eye = eye;
        self.scratch = fresh;
    }

    /// Checks the incremental state against a vision built from scratch out of `objects` (all of the sim's, in id
    /// order): the same sources, and the same Visible tiles seen by as many of them, for every player; and every
    /// Visible tile Explored. The test harness runs it after every tick (`testing::invariants`, V1). The first
    /// difference, and where on the map (space units).
    pub(crate) fn check<'a>(
        &self,
        map: &Map,
        objects: impl IntoIterator<Item = &'a Object>,
    ) -> Result<(), (Vec2, String)> {
        let mut fresh = Vision::new(map, self.players.iter().map(|p| p.id));
        fresh.update(map, objects);
        let eyes = |v: &Vision| v.sources.iter().map(|s| (s.id, s.eye)).collect::<Vec<_>>();
        if eyes(self) != eyes(&fresh) {
            let at = self.sources.first().map_or(Vec2::ZERO, |s| s.eye.at);
            return Err((at, "the sight sources differ from a fresh build".into()));
        }
        for (have, want) in self.players.iter().zip(&fresh.players) {
            for (i, (&n, &m)) in have.seers.iter().zip(&want.seers).enumerate() {
                let tile = TilePos::new((i % self.width as usize) as i32, (i / self.width as usize) as i32);
                let fail =
                    |message: String| Err((tile.center(), format!("player {}: tile {tile:?} {message}", have.id.0)));
                if n != m {
                    return fail(format!("is seen by {n} sources, a fresh count says {m}"));
                }
                if n > 0 && !have.explored.get(i) {
                    return fail("is Visible but not Explored".into());
                }
            }
        }
        Ok(())
    }

    /// The serde form of each player's Explored tiles ([`ExploredRows`]).
    fn explored_rows(&self) -> Vec<ExploredRows> {
        self.players
            .iter()
            .map(|p| ExploredRows {
                player: p.id,
                explored: (0..self.height as usize)
                    .map(|y| {
                        (0..self.width as usize)
                            .map(|x| {
                                if p.explored.get(y * self.width as usize + x) {
                                    EXPLORED_SYMBOL
                                } else {
                                    UNEXPLORED_SYMBOL
                                }
                            })
                            .collect()
                    })
                    .collect(),
            })
            .collect()
    }
}

/// Tile symbols of the Explored rows in a save: Explored (Visible tiles are Explored too), and not.
const EXPLORED_SYMBOL: char = '#';
const UNEXPLORED_SYMBOL: char = '.';

/// One player's Explored tiles in a save (the serde form of [`Vision`]): one string per tile row, north to south,
/// [`EXPLORED_SYMBOL`] for a tile the player has seen and [`UNEXPLORED_SYMBOL`] for one it hasn't.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ExploredRows {
    player: PlayerId,
    explored: Vec<String>,
}

/// The vision of a loaded save ([`crate::Sim`]'s serde form): `saved` must hold the Explored rows of exactly
/// `players`, in order, each of `map`'s size; the rest is derived from `objects`.
pub(crate) fn load<'a>(
    map: &Map,
    players: &[PlayerId],
    saved: Vec<ExploredRows>,
    objects: impl IntoIterator<Item = &'a Object>,
) -> Result<Vision, String> {
    let saved_ids: Vec<PlayerId> = saved.iter().map(|rows| rows.player).collect();
    if saved_ids != players {
        return Err(format!(
            "the vision is saved for players {saved_ids:?}, the game has {players:?}"
        ));
    }
    let (width, height) = (map.width() as usize, map.height() as usize);
    let mut explored = Vec::with_capacity(saved.len());
    for rows in saved {
        let wrong_size = rows.explored.len() != height || rows.explored.iter().any(|r| r.chars().count() != width);
        if wrong_size {
            return Err(format!(
                "player {}'s explored tiles don't match the {width} x {height} map",
                rows.player.0
            ));
        }
        let mut bits = TileBits::new(width * height);
        for (y, row) in rows.explored.iter().enumerate() {
            for (x, symbol) in row.chars().enumerate() {
                match symbol {
                    EXPLORED_SYMBOL => bits.set(y * width + x),
                    UNEXPLORED_SYMBOL => {}
                    other => {
                        return Err(format!(
                            "player {}'s explored tiles: {other:?} is neither {EXPLORED_SYMBOL:?} nor \
                             {UNEXPLORED_SYMBOL:?}",
                            rows.player.0
                        ));
                    }
                }
            }
        }
        explored.push(bits);
    }
    Ok(Vision::load(map, players.iter().copied(), explored, objects))
}

impl Serialize for Vision {
    /// Only the history: each player's Explored rows ([`ExploredRows`]). The rest comes from the objects on load.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.explored_rows().serialize(serializer)
    }
}

impl PartialEq for Vision {
    /// The state: each player's Explored tiles. The rest is derived from the objects, which the sims compare too.
    fn eq(&self, other: &Self) -> bool {
        self.width == other.width
            && self.height == other.height
            && self.players.len() == other.players.len()
            && self
                .players
                .iter()
                .zip(&other.players)
                .all(|(a, b)| a.id == b.id && a.explored == b.explored)
    }
}

impl fmt::Debug for Vision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let tiles = |p: &PlayerVision, state| p.states().filter(|&s| s == state).count();
        let players: Vec<String> = self
            .players
            .iter()
            .map(|p| {
                format!(
                    "player {}: {} visible, {} explored, revision {}",
                    p.id.0,
                    tiles(p, VisibilityState::Visible),
                    tiles(p, VisibilityState::Explored),
                    p.revision
                )
            })
            .collect();
        f.debug_struct("Vision")
            .field("size", &(self.width, self.height))
            .field("players", &players)
            .field("sources", &self.sources.len())
            .finish_non_exhaustive()
    }
}

impl StateHash for Vision {
    /// The history only: each player's Explored tiles. Visible follows from the objects, which are hashed.
    fn hash(&self, h: &mut StateHasher) {
        h.len(self.players.len());
        for p in &self.players {
            p.id.hash(h);
            h.len(p.explored.words.len());
            for word in &p.explored.words {
                word.hash(h);
            }
        }
    }
}
