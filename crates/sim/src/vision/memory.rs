//! Last-known records (`core/vision.md > Explored`: "Structures and resources are shown in their last-known state
//! (may have been destroyed or changed since)"): what each player remembers of the structures and resources it has
//! seen.
//!
//! - **Records** ([`LastKnown`]): one per tile-based object (a structure or a resource: an object with a footprint,
//!   `Object::placement`) the player has seen and doesn't own, with what the views need to draw it and fill its
//!   InfoPanel: kind, owner, footprint, HP, resource state, and the last tick the player saw it. History: hashed and
//!   serialized with the Explored tiles.
//! - **In sight**: such an object is in a player's sight while a tile it covers is Visible, as
//!   [`PlayerVision::object_state`] counts it. Its record is written at the end of every tick it is in sight
//!   ([`Vision::remember`], after every phase that changes objects, supply deliveries included), so it holds the
//!   state of the last tick the player saw it; out of sight it stays as it was.
//! - **Gone**: a record whose object is gone, or no longer covers the record's footprint, is dropped once a tile
//!   under it is Visible: the player sees the place without it. Until then the player remembers the object there,
//!   destroyed or not (design change 0040: a seen structure in the fog is a wall at its last-known position until
//!   seen otherwise).
//! - **Never seen**: an object placed or built on Explored tiles has no record, so it stays unknown until seen.
//! - **Own objects** are never recorded: a player always knows its own objects as they are.
//!
//! **Update**, driven by vision changes, never a scan of the map or of every object for every player:
//! - Phase 5f's walk over the objects ([`Vision::update`]) merges, in id order, the footprints of the tile-based
//!   objects ([`Footprints`]) with the walk it already makes for the sight sources; only a footprint that came or
//!   went touches the index from tiles to objects.
//! - Each tile whose state flips for a player during the walk (to Visible or back) is noted
//!   (`PlayerVision::see`, `unsee`). After the walk (`PlayerVision::catch_up`), a noted tile under no footprint and
//!   no record costs two bit tests, and most are; on the others, the objects are put in or out of the player's
//!   sight, and if the tile is now Visible, the records on it whose object is gone are dropped. A new footprint
//!   enters the sight if a tile under it is Visible; a vanished one leaves it, and its record goes if a tile under
//!   it is Visible.
//! - At the end of the tick (`PlayerVision::refresh`) only the objects in each player's sight are read, to write
//!   their records.
//!
//! Each player's records are indexed by tile too: which tiles lie under one ([`PlayerVision::record_tiles`]), and a
//! revision that changes whenever a record comes, goes or moves ([`PlayerVision::records_revision`]), for views and
//! for paths planned from what a player knows (design change 0040).
//!
//! [`Vision::remember`]: super::Vision::remember
//! [`Vision::update`]: super::Vision::update

#[cfg(test)]
mod tests;

use std::collections::{BTreeMap, BTreeSet};
use std::iter::Peekable;
use std::vec;

use glam::Vec2;
use serde::{Deserialize, Serialize};

use super::{PlayerVision, TileBits};
use crate::defs::{ObjectDef, ObjectKind};
use crate::hash::{StateHash, StateHasher};
use crate::ids::{ObjectId, PlayerId};
use crate::object::{Object, Placement, ResourceState};
use crate::units::{Tick, TilePos};

/// `core/vision.md > Explored`: a structure or resource as one player last saw it, at the end of tick `seen`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LastKnown {
    pub id: ObjectId,
    pub kind: ObjectKind,
    pub owner: Option<PlayerId>,
    /// Its footprint: only tile-based objects are remembered.
    pub placement: Placement,
    /// Its HP when last seen; `None` for an indestructible type.
    pub hp: Option<f32>,
    /// A map resource's state when last seen (`core/map.md > Resource`).
    pub resource: Option<ResourceState>,
    /// The tick at whose end the player last saw it.
    pub seen: Tick,
}

impl LastKnown {
    /// `object` as a player sees it at the end of `tick`; `None` for an object without a footprint (a unit: Explored
    /// areas show no units, `core/vision.md > Explored`).
    pub fn of(object: &Object, tick: Tick) -> Option<Self> {
        Some(Self {
            id: object.id,
            kind: object.kind,
            owner: object.owner,
            placement: object.placement?,
            hp: object.hp,
            resource: object.resource.clone(),
            seen: tick,
        })
    }

    pub fn def(&self) -> &'static ObjectDef {
        self.kind.def()
    }

    /// Where it stands: the centre of its footprint, which is its object's position (space units).
    pub fn pos(&self) -> Vec2 {
        self.placement.center()
    }
}

impl StateHash for LastKnown {
    fn hash(&self, h: &mut StateHasher) {
        let Self {
            id,
            kind,
            owner,
            placement,
            hp,
            resource,
            seen,
        } = self;
        id.hash(h);
        kind.hash(h);
        owner.hash(h);
        placement.hash(h);
        hp.hash(h);
        resource.hash(h);
        seen.hash(h);
    }
}

/// The tiles under `placement` on a `width` × `height` map, as row-major tile indexes, row by row; tiles off the
/// map are left out.
fn footprint_tiles(placement: Placement, width: u32, height: u32) -> impl Iterator<Item = u32> {
    let span = |start: i32, len: u32, size: u32| {
        let clamp = |v: i64| v.clamp(0, i64::from(size)) as u32;
        clamp(i64::from(start))..clamp(i64::from(start) + i64::from(len))
    };
    let columns = span(placement.origin.x, placement.w, width);
    span(placement.origin.y, placement.h, height).flat_map(move |y| columns.clone().map(move |x| y * width + x))
}

/// The ids an index of `(tile, id)` pairs holds for tile `i`.
fn on(index: &BTreeSet<(u32, ObjectId)>, i: u32) -> impl Iterator<Item = ObjectId> + '_ {
    index
        .range((i, ObjectId(0))..=(i, ObjectId(u32::MAX)))
        .map(|&(_, id)| id)
}

/// The tile-based objects of the sim, by id, and the tiles they cover, so that the memory finds the objects on a
/// tile. Derived from the objects, kept up by phase 5f's walk.
#[derive(Clone, Debug)]
pub(super) struct Footprints {
    width: u32,
    height: u32,
    /// By ascending id.
    list: Vec<(ObjectId, Placement)>,
    /// `(tile, id)` for every tile each one covers.
    tiles: BTreeSet<(u32, ObjectId)>,
    /// The tiles under at least one.
    bits: TileBits,
}

/// What a walk found: the footprints that came and those that went (one that changed is both).
#[derive(Default)]
pub(super) struct Changes {
    added: Vec<(ObjectId, Placement)>,
    removed: Vec<(ObjectId, Placement)>,
}

/// Phase 5f's walk over the objects, merging them in id order with the footprints of the walk before.
pub(super) struct Walk {
    old: Peekable<vec::IntoIter<(ObjectId, Placement)>>,
    kept: Vec<(ObjectId, Placement)>,
    changes: Changes,
}

impl Footprints {
    pub(super) fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            list: Vec::new(),
            tiles: BTreeSet::new(),
            bits: TileBits::new(width as usize * height as usize),
        }
    }

    /// Starts a walk: [`Walk::visit`] every object in id order, then [`Footprints::finish`].
    pub(super) fn walk(&mut self) -> Walk {
        let old = std::mem::take(&mut self.list);
        Walk {
            kept: Vec::with_capacity(old.len()),
            old: old.into_iter().peekable(),
            changes: Changes::default(),
        }
    }

    /// Ends a walk: the footprints that went leave the index and those that came join it.
    pub(super) fn finish(&mut self, walk: Walk) -> Changes {
        let Walk { old, kept, mut changes } = walk;
        changes.removed.extend(old);
        let (width, height) = (self.width, self.height);
        for &(id, placement) in &changes.removed {
            for i in footprint_tiles(placement, width, height) {
                self.tiles.remove(&(i, id));
            }
        }
        for &(_, placement) in &changes.removed {
            for i in footprint_tiles(placement, width, height) {
                if on(&self.tiles, i).next().is_none() {
                    self.bits.clear(i as usize);
                }
            }
        }
        for &(id, placement) in &changes.added {
            for i in footprint_tiles(placement, width, height) {
                self.tiles.insert((i, id));
                self.bits.set(i as usize);
            }
        }
        self.list = kept;
        changes
    }

    /// Where the tile-based object `id` stands, if it exists.
    fn placement(&self, id: ObjectId) -> Option<Placement> {
        let i = self.list.binary_search_by_key(&id, |&(id, _)| id).ok()?;
        Some(self.list[i].1)
    }

    /// The same footprints, indexed the same (V1).
    pub(super) fn matches(&self, other: &Footprints) -> bool {
        self.list == other.list && self.tiles == other.tiles && self.bits == other.bits
    }
}

impl Walk {
    /// The next object of the walk, in ascending id order.
    pub(super) fn visit(&mut self, o: &Object) {
        while let Some(gone) = self.old.next_if(|&(id, _)| id < o.id) {
            self.changes.removed.push(gone);
        }
        let before = self.old.next_if(|&(id, _)| id == o.id);
        match (before, o.placement) {
            (Some(footprint), Some(placement)) if footprint.1 == placement => self.kept.push(footprint),
            (before, now) => {
                self.changes.removed.extend(before);
                if let Some(placement) = now {
                    self.changes.added.push((o.id, placement));
                    self.kept.push((o.id, placement));
                }
            }
        }
    }
}

/// One player's memory (`core/vision.md > Explored`).
#[derive(Clone, Debug)]
pub(super) struct Memory {
    /// The records, by id. History: hashed and serialized.
    records: BTreeMap<ObjectId, LastKnown>,
    /// `(tile, id)` for every tile under a record. Derived.
    tiles: BTreeSet<(u32, ObjectId)>,
    /// The tiles under at least one record. Derived.
    bits: TileBits,
    /// The tile-based objects the player sees now: a tile they cover is Visible. Derived.
    in_sight: BTreeSet<ObjectId>,
    /// Bumped whenever a record comes, goes or moves. Output, not state.
    revision: u32,
}

impl Memory {
    pub(super) fn new(tiles: usize) -> Self {
        Self {
            records: BTreeMap::new(),
            tiles: BTreeSet::new(),
            bits: TileBits::new(tiles),
            in_sight: BTreeSet::new(),
            revision: 0,
        }
    }
}

impl PlayerVision {
    /// The player's records, by ascending id: every structure and resource it has seen, as it last saw it, except
    /// those it has since seen gone (`core/vision.md > Explored`). The record of an object in the player's sight is
    /// that object as it is now.
    pub fn records(&self) -> impl Iterator<Item = &LastKnown> + '_ {
        self.memory.records.values()
    }

    /// The player's record of object `id`, if it has one.
    pub fn record(&self, id: ObjectId) -> Option<&LastKnown> {
        self.memory.records.get(&id)
    }

    /// The records of the objects out of the player's sight now (no tile under them Visible): what its Explored
    /// tiles show in their place (`core/vision.md > Explored`). Their objects may have changed since, or be gone.
    pub fn remembered(&self) -> impl Iterator<Item = &LastKnown> + '_ {
        self.records().filter(|r| !self.sees_footprint(r.placement))
    }

    /// The player's record of object `id` if the object is out of its sight now: what the views show of it.
    pub fn remembered_record(&self, id: ObjectId) -> Option<&LastKnown> {
        self.record(id).filter(|r| !self.sees_footprint(r.placement))
    }

    /// Changes whenever one of the player's records comes, goes or moves (a counter, wrapping), not when one is only
    /// refreshed: a view or a plan built from the records' footprints is stale once it changed (design change 0040:
    /// paths planned from what the player knows). Output, not state, like [`PlayerVision::revision`].
    pub fn records_revision(&self) -> u32 {
        self.memory.revision
    }

    /// Every tile under at least one of the player's records, row by row from the north-west corner: the footprints
    /// of the structures and resources it knows of besides its own.
    pub fn record_tiles(&self) -> impl Iterator<Item = TilePos> + '_ {
        let mut last = None;
        let width = self.width;
        self.memory
            .tiles
            .iter()
            .filter_map(move |&(i, _)| (last.replace(i) != Some(i)).then_some(i))
            .map(move |i| TilePos::new((i % width) as i32, (i / width) as i32))
    }

    /// Whether `tile` lies under one of the player's records.
    pub fn is_record_tile(&self, tile: TilePos) -> bool {
        self.index(tile).is_some_and(|i| self.memory.bits.get(i))
    }

    /// Whether a tile under `placement` is Visible: the player sees the object standing there.
    fn sees_footprint(&self, placement: Placement) -> bool {
        footprint_tiles(placement, self.width, self.height).any(|i| self.seers[i as usize] > 0)
    }

    /// After phase 5f's walk: puts the tile-based objects in or out of the player's sight where that may have
    /// changed (on the tiles that flipped, and the footprints that came or went), and drops the records of objects
    /// the player now sees gone.
    pub(super) fn catch_up(&mut self, footprints: &Footprints, changes: &Changes) {
        let mut flips = std::mem::take(&mut self.flips);
        for &(id, _) in &changes.removed {
            self.memory.in_sight.remove(&id);
        }
        for &(id, placement) in &changes.added {
            self.place(id, placement);
        }
        for &(id, _) in &changes.removed {
            self.drop_if_gone(id, footprints);
        }
        for &i in &flips {
            if footprints.bits.get(i as usize) {
                for id in on(&footprints.tiles, i) {
                    if let Some(placement) = footprints.placement(id) {
                        self.place(id, placement);
                    }
                }
            }
            if self.memory.bits.get(i as usize) && self.seers[i as usize] > 0 {
                let ids: Vec<ObjectId> = on(&self.memory.tiles, i).collect();
                for id in ids {
                    self.drop_if_gone(id, footprints);
                }
            }
        }
        flips.clear();
        self.flips = flips;
    }

    /// The object `id` standing on `placement` is in the player's sight while a tile under it is Visible.
    fn place(&mut self, id: ObjectId, placement: Placement) {
        if self.sees_footprint(placement) {
            self.memory.in_sight.insert(id);
        } else {
            self.memory.in_sight.remove(&id);
        }
    }

    /// `core/vision.md > Explored`: the record of `id` goes once the player sees its place without its object (gone,
    /// or no longer there).
    fn drop_if_gone(&mut self, id: ObjectId, footprints: &Footprints) {
        let Some(record) = self.memory.records.get(&id) else {
            return;
        };
        if footprints.placement(id) != Some(record.placement) && self.sees_footprint(record.placement) {
            self.forget(id);
        }
    }

    /// The end of the tick ([`super::Vision::remember`]): the records of the objects in the player's sight take
    /// their state at the end of `tick`; an object the player owns has none. `object` finds an object by id.
    pub(super) fn refresh<'a>(&mut self, tick: Tick, object: &impl Fn(ObjectId) -> Option<&'a Object>) {
        let in_sight = std::mem::take(&mut self.memory.in_sight);
        for &id in &in_sight {
            match object(id) {
                Some(o) if o.owner == Some(self.id) => self.forget(id),
                Some(o) => {
                    if let Some(record) = LastKnown::of(o, tick) {
                        self.store(record);
                    }
                }
                None => {}
            }
        }
        self.memory.in_sight = in_sight;
    }

    /// Starts the player's memory from `records` (a save's, or a copy to check against).
    pub(super) fn set_records(&mut self, records: impl IntoIterator<Item = LastKnown>) {
        for record in records {
            self.store(record);
        }
    }

    /// Writes `record`, indexing its footprint if it is new or moved.
    fn store(&mut self, record: LastKnown) {
        if let Some(old) = self.memory.records.get_mut(&record.id)
            && old.placement == record.placement
        {
            *old = record;
            return;
        }
        self.forget(record.id);
        for i in footprint_tiles(record.placement, self.width, self.height) {
            self.memory.tiles.insert((i, record.id));
            self.memory.bits.set(i as usize);
        }
        self.memory.records.insert(record.id, record);
        self.memory.revision = self.memory.revision.wrapping_add(1);
    }

    /// Drops the record of `id`, if there is one.
    fn forget(&mut self, id: ObjectId) {
        let Some(record) = self.memory.records.remove(&id) else {
            return;
        };
        for i in footprint_tiles(record.placement, self.width, self.height) {
            self.memory.tiles.remove(&(i, id));
            if on(&self.memory.tiles, i).next().is_none() {
                self.memory.bits.clear(i as usize);
            }
        }
        self.memory.revision = self.memory.revision.wrapping_add(1);
    }
}

/// V1's part for the memory: `have`'s records and what indexes them are those of `want`, a fresh build that started
/// from `have`'s records, so none was due to be dropped or refreshed; and no record is of an object the player owns.
pub(super) fn check(have: &PlayerVision, want: &PlayerVision) -> Result<(), (Vec2, String)> {
    let player = have.id.0;
    if let Some(own) = have.records().find(|r| r.owner == Some(have.id)) {
        return Err((
            own.pos(),
            format!("player {player} remembers its own object #{}", own.id.0),
        ));
    }
    let ids: BTreeSet<ObjectId> = have
        .memory
        .records
        .keys()
        .chain(want.memory.records.keys())
        .copied()
        .collect();
    for id in ids {
        let (a, b) = (have.record(id), want.record(id));
        if a != b {
            let at = a.or(b).map_or(Vec2::ZERO, LastKnown::pos);
            let message = format!(
                "player {player}: the record of #{} is {a:?}, a fresh build's {b:?}",
                id.0
            );
            return Err((at, message));
        }
    }
    if have.memory.in_sight != want.memory.in_sight {
        let message = format!(
            "player {player}: the objects in sight are {:?}, a fresh build's {:?}",
            have.memory.in_sight, want.memory.in_sight
        );
        return Err((Vec2::ZERO, message));
    }
    if have.memory.tiles != want.memory.tiles || have.memory.bits != want.memory.bits {
        let message = format!("player {player}: the tiles under the records differ from a fresh build's");
        return Err((Vec2::ZERO, message));
    }
    Ok(())
}
