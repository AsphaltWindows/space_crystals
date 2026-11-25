//! Vision — `core/vision.md`. Phase 5f (`docs/movement.md > 4`): what each player sees, from the tick's final
//! positions; at the very end of the tick, what it remembers of the structures and resources it has seen.
//!
//! - **States** (`core/vision.md > VisibilityState`): every tile is [`VisibilityState::Unexplored`], `Explored` or
//!   `Visible` for each player. Explored is history: one bit per tile and player, set the first time the tile is
//!   Visible and never cleared, so it is hashed and serialized with the rest of the state. A tile is Visible while at
//!   least one of the player's sight sources sees it, which is a count per tile, kept up to date incrementally, never
//!   hashed and rebuilt from the objects when a save is loaded; and for the fog delay after the last one stops seeing
//!   it ([`linger`], `core/vision.md > Visible > Fog delay`, design change 0178), which is history again. Each player's
//!   are a [`PlayerVision`] (`player`).
//! - **Reveals** ([`reveal`], `core/vision.md > Attacks from outside vision`, design change 0177): an attacker that
//!   shoots one of a player's objects from outside the player's vision is Visible to that player for a while, it alone,
//!   not its tiles. History, like the delay.
//! - **Sources** (`core/entities.md > Object Type > SightRange`: vision goes to the owner): every owned object whose
//!   type has a SightRange above 0. Neutral objects and resources (`core/map.md > Resource`: no vision) see nothing.
//! - **Sight** ([`sight`]): a disc around the object's centre. A tile is seen when its centre lies within the
//!   SightRange, the boundary included, with the ElevationModifier between the object and the ground at the tile's
//!   centre (`core/vision.md > ElevationModifier`): one grid unit more towards lower tiles, one less towards higher
//!   ones; air units take none. No terrain blocks sight: the design has no line of sight.
//! - **Update** (incremental stamps): each source keeps the tiles it stamped and its [`Eye`], everything the stamp
//!   depends on (owner, centre, SightRange, elevation). Phase 5f walks the objects in id order along with the
//!   sources. A source whose eye is unchanged costs nothing; one that moved stamps its disc again, and only the
//!   tiles that differ change their counts; a source that is gone takes its tiles away. Then the tiles whose fog
//!   delay ends turn Explored, and the reveals that end are over, each from a queue or a list of its own, never a
//!   scan of the map. Each player's [`PlayerVision::revision`] changes whenever one of its tiles changed state, so
//!   views redraw only then.
//! - **Timing**: phase 5f sees every object at the end of the tick, those spawned in it included, so Visible is a
//!   function of the end-of-tick state and the history, and a loaded save sees what the running game saw. The next
//!   tick reads it: combat's target checks in 5e (`systems/combat.md > ValidTarget` 2) and, later, the behaviours in
//!   5a see last tick's visibility (`docs/movement.md > 4`). The shots of 5e reveal their attackers ([`reveal`]) on
//!   top of it, for the next tick.
//! - **Objects** ([`PlayerVision::object_state`], [`PlayerVision::object_box_state`]): an object counts as seen while
//!   it is revealed, or when any tile it covers is Visible; it covers the tiles its silhouette box (a unit) or
//!   footprint (a structure or resource) overlaps. Every question whether a player sees an object comes there.
//!   An object hidden from the player ([`hidden`], plan decision T32: an owned underground structure, for every
//!   player but its owner) is never seen, whatever its tiles.
//! - **Memory** ([`memory`], `core/vision.md > Explored`: structures and resources show in their last-known state):
//!   each player's records of the structures and resources it has seen ([`LastKnown`], [`PlayerVision::records`]),
//!   history like the Explored tiles. Phase 5f follows which of them the player sees from the tiles that change state,
//!   and [`Vision::remember`], the last thing a tick does, refreshes the records of those in sight. No record is ever
//!   taken of an object hidden from the player.
//!
//! The map never changes during a game; a sim whose map changed would have to build its vision again
//! ([`Vision::new`]).

pub mod hidden;
pub mod linger;
mod memory;
mod player;
pub mod reveal;
mod save;
mod sight;
#[cfg(test)]
mod tests;

use std::collections::BTreeMap;
use std::fmt;

use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::elevation::Elevation;
use crate::hash::{StateHash, StateHasher};
use crate::ids::{ObjectId, PlayerId};
use crate::map::Map;
use crate::object::Object;
use crate::tuning::vision::{FOG_DELAY_TICKS, REVEAL_TICKS};
use crate::units::{Tick, TilePos};

pub use hidden::{Hiding, hidden_from};
use memory::Footprints;
pub use memory::LastKnown;
pub use player::PlayerVision;
use player::TileBits;
pub(crate) use save::{SavedVision, load};
pub use sight::TILE_SIGHT_POINT;
use sight::stamp;

/// `core/vision.md > VisibilityState`: a tile as one player sees it. Ordered from the least to the most seen, so the
/// best state of a few tiles is their `max`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum VisibilityState {
    /// `core/vision.md > Unexplored`: never within the SightRange of the player's units or structures. Fully black:
    /// terrain, structures and units are hidden.
    Unexplored,
    /// `core/vision.md > Explored`: seen before, not now. Terrain shows, structures and resources in their last-known
    /// state, enemy units not at all.
    Explored,
    /// `core/vision.md > Visible`: within the SightRange of one of the player's units or structures now. Everything
    /// shows.
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
    /// The tile-based objects and the tiles they cover, which the memory looks up ([`memory`]). Derived from the
    /// objects.
    footprints: Footprints,
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
            footprints: Footprints::new(width, height),
        }
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

    /// How `player` sees `object` ([`PlayerVision::object_state`]): Visible while it is revealed, else the best state
    /// among the tiles it covers.
    pub fn object_state(&self, player: PlayerId, object: &Object) -> VisibilityState {
        self.player(player)
            .map_or(VisibilityState::Unexplored, |p| p.object_state(object))
    }

    /// Whether `player` sees `object` now: it is revealed, or a tile it covers is Visible.
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

    /// Phase 5f of `tick`: catches up with `objects`, every object of the sim in ascending id order. New sources
    /// stamp the tiles they see, moved ones stamp again (only the tiles that differ change), and gone ones take their
    /// tiles away; tiles that become Visible become Explored too. A tile none sees any more stays Visible for the fog
    /// delay ([`linger`]); the tiles whose delay ends now turn Explored, and the reveals that end now are over
    /// ([`reveal`]). Each player whose tiles changed state gets a new [`PlayerVision::revision`]. The memory follows
    /// ([`memory`]): which structures and resources each player sees now, from the tiles that changed state and the
    /// footprints that came or went, and the records of those seen gone are dropped. [`Vision::remember`] refreshes
    /// the records at the end of the tick. The ticks of successive updates never go down.
    ///
    /// The sim calls it on its own vision; it is public so that tests can time it and check it on a copy.
    pub fn update<'a>(&mut self, map: &Map, objects: impl IntoIterator<Item = &'a Object>, tick: Tick) {
        let mut old = std::mem::take(&mut self.sources).into_iter().peekable();
        let mut kept = Vec::with_capacity(old.len());
        let mut footprints = self.footprints.walk();
        for o in objects {
            footprints.visit(o);
            while let Some(gone) = old.next_if(|s| s.id < o.id) {
                self.withdraw(&gone, tick);
            }
            let previous = old.next_if(|s| s.id == o.id);
            match (previous, self.eye(map, o)) {
                (Some(source), Some(eye)) if source.eye == eye => kept.push(source),
                (Some(mut source), Some(eye)) => {
                    self.restamp(&mut source, eye, tick);
                    kept.push(source);
                }
                (Some(source), None) => self.withdraw(&source, tick),
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
            self.withdraw(&gone, tick);
        }
        self.sources = kept;
        let changes = self.footprints.finish(footprints);
        for player in &mut self.players {
            player.expire(tick);
            player.catch_up(&self.footprints, &changes);
            if std::mem::take(&mut player.dirty) {
                player.revision = player.revision.wrapping_add(1);
            }
        }
    }

    /// Design change 0159 (`systems/controls/behaviors.md > PathKnowledge`: "Every neutral object placed by the map
    /// maker is known from the start (pathing and memory), like terrain"): at the start of the game, before its first
    /// look, every player gets a record of each of `objects` that has a footprint, as it is at the end of `tick`.
    /// `objects` are the neutral ones the game starts with (`Sim::settle_start`); the players' starting layouts aren't
    /// among them, since they "are never pre-known" (`vision::memory`).
    pub(crate) fn seed<'a>(&mut self, objects: impl Iterator<Item = &'a Object> + Clone, tick: Tick) {
        for player in &mut self.players {
            // Neutral objects are hidden from nobody (`hidden`); the rule holds here too, whatever the caller passes.
            let id = player.id;
            let known = objects.clone().filter(|o| !hidden_from(o, id));
            player.set_records(known.filter_map(|o| LastKnown::of(o, tick)));
        }
    }

    /// The end of the tick, after every phase that changes objects (`Sim::step`): each player's records of the
    /// structures and resources it sees now take their current state, stamped with `tick` (`core/vision.md >
    /// Explored`: a record keeps the last state the player saw). The objects the player owns are never recorded.
    /// `object` finds an object of the sim by id; only those in some player's sight are looked up.
    pub fn remember<'a>(&mut self, tick: Tick, object: impl Fn(ObjectId) -> Option<&'a Object>) {
        for player in &mut self.players {
            player.refresh(tick, &object);
        }
    }

    fn stamp_into(&self, eye: &Eye, out: &mut Vec<u32>) {
        stamp(eye, self.width, self.height, &self.tile_levels, out);
    }

    /// A source is gone, or no longer gives vision, in phase 5f of `tick`: its tiles lose it.
    fn withdraw(&mut self, source: &Source, tick: Tick) {
        let player = &mut self.players[source.eye.player];
        for &i in &source.tiles {
            player.unsee(i, tick);
        }
    }

    /// A source's eye changed, in phase 5f of `tick`: it stamps again, and only the tiles that differ change.
    fn restamp(&mut self, source: &mut Source, eye: Eye, tick: Tick) {
        let mut fresh = std::mem::take(&mut self.scratch);
        self.stamp_into(&eye, &mut fresh);
        if source.eye.player == eye.player {
            if fresh != source.tiles {
                self.players[eye.player].restamp(&source.tiles, &fresh, tick);
            }
        } else {
            self.withdraw(source, tick);
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
    /// order) at the end of `tick`, from the same history (records, tiles that linger, reveals): the same sources,
    /// the same Visible tiles seen by as many of them, and the same tiles lingering until the same ticks, for every
    /// player; every Visible tile Explored; no tile lingering while a source sees it, and no delay or reveal that
    /// ends in the past or beyond its length ([`FOG_DELAY_TICKS`], [`REVEAL_TICKS`]); the same reveals, so none was
    /// due to end; and the memory a fresh build makes of the same records: none dropped or refreshed, none of an
    /// object the player owns, and the same objects in sight. The test harness runs it after every tick
    /// (`testing::invariants`, V1). The first difference, and where on the map (space units).
    pub(crate) fn check<'a>(
        &self,
        map: &Map,
        objects: impl IntoIterator<Item = &'a Object>,
        tick: Tick,
    ) -> Result<(), (Vec2, String)> {
        let objects: BTreeMap<ObjectId, &Object> = objects.into_iter().map(|o| (o.id, o)).collect();
        let mut fresh = Vision::new(map, self.players.iter().map(|p| p.id));
        for (fresh, have) in fresh.players.iter_mut().zip(&self.players) {
            fresh.set_records(have.records().cloned());
            fresh.restore(have.lingering(), have.revealed());
        }
        fresh.update(map, objects.values().copied(), tick);
        fresh.remember(tick, |id| objects.get(&id).copied());
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
                if have.visible_at(i) && !have.explored.get(i) {
                    return fail("is Visible but not Explored".into());
                }
                let end = have.linger.end(i);
                match end {
                    Some(_) if n > 0 => return fail(format!("lingers, though {n} sources see it")),
                    Some(end) if end <= tick || end > tick + FOG_DELAY_TICKS => {
                        return fail(format!("lingers until tick {end}, outside the delay after tick {tick}"));
                    }
                    _ => {}
                }
                if end != want.linger.end(i) {
                    return fail(format!(
                        "lingers until {end:?}, a fresh build's {:?}",
                        want.linger.end(i)
                    ));
                }
            }
            let at = |id: ObjectId| objects.get(&id).map_or(Vec2::ZERO, |o| o.pos);
            let player = have.id.0;
            if let Some((id, end)) = have
                .revealed()
                .find(|&(_, end)| end <= tick || end > tick + REVEAL_TICKS)
            {
                let message = format!(
                    "player {player}: #{} is revealed until tick {end}, outside the reveal after tick {tick}",
                    id.0
                );
                return Err((at(id), message));
            }
            if have.reveals != want.reveals {
                let id = have.revealed().map(|(id, _)| id).next().unwrap_or(ObjectId(0));
                let message = format!(
                    "player {player}: the reveals are {:?}, a fresh build's {:?}",
                    have.reveals, want.reveals
                );
                return Err((at(id), message));
            }
            memory::check(have, want)?;
        }
        if !self.footprints.matches(&fresh.footprints) {
            return Err((
                Vec2::ZERO,
                "the footprints of the tile-based objects differ from a fresh build".into(),
            ));
        }
        Ok(())
    }
}

impl PartialEq for Vision {
    /// The state: each player's history (Explored tiles, the tiles that linger, the reveals and the records). The
    /// rest is derived from the objects, which the sims compare too.
    fn eq(&self, other: &Self) -> bool {
        self.width == other.width
            && self.height == other.height
            && self.players.len() == other.players.len()
            && self.players.iter().zip(&other.players).all(|(a, b)| a.same_history(b))
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
                    "player {}: {} visible ({} lingering), {} explored, {} records, {} revealed, revision {}",
                    p.id.0,
                    tiles(p, VisibilityState::Visible),
                    p.lingering().count(),
                    tiles(p, VisibilityState::Explored),
                    p.records().count(),
                    p.reveals.len(),
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
    /// The history only: each player's ([`PlayerVision`]'s hash). The sight sources and the counts follow from the
    /// objects, which are hashed, and the rest from the map.
    fn hash(&self, h: &mut StateHasher) {
        let Vision {
            width: _,
            height: _,
            tile_levels: _,
            players,
            sources: _,
            scratch: _,
            footprints: _,
        } = self;
        h.len(players.len());
        for p in players {
            p.hash(h);
        }
    }
}
