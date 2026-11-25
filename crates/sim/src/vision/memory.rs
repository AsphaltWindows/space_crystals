//! Last-known records (`core/vision.md > Explored`: "Structures and resources are shown in their last-known state
//! (may have been destroyed or changed since)"): what each player remembers of the structures and resources it has
//! seen.
//!
//! - **Records** ([`LastKnown`]): one per tile-based object (a structure or a resource: an object with a footprint,
//!   `Object::placement`) the player has seen and doesn't own, with what the views need to draw it and fill its
//!   InfoPanel: kind, owner, footprint, HP, resource state, a Tunnel's tier, a site's progress, and the last tick the
//!   player saw it. History: hashed and serialized with the Explored tiles.
//! - **In sight**: such an object is in a player's sight while a tile it covers is Visible, the fog delay included,
//!   as [`PlayerVision::box_state`] counts it (only units attack, so none is ever revealed: `super::reveal`). Its
//!   record is written at the end of every tick it is in sight
//!   ([`Vision::remember`], after every phase that changes objects, supply deliveries included), so it holds the
//!   state of the last tick the player saw it; out of sight it stays as it was.
//! - **Gone**: a record whose object is gone, or no longer covers the record's footprint, is dropped once a tile
//!   under it is Visible: the player sees the place without it. Until then the player remembers the object there,
//!   destroyed or not (design change 0040: a seen structure in the fog is a wall at its last-known position until
//!   seen otherwise).
//! - **Never seen**: an object placed or built on Explored tiles has no record, so it stays unknown until seen.
//! - **Known from the start** (design change 0159, `systems/controls/behaviors.md > PathKnowledge`: "every neutral
//!   object placed by the map maker" is "known from the start (pathing and memory), like terrain"): every player
//!   starts with a record of each neutral object the game starts with, as the map placed it
//!   ([`super::Vision::seed`]), seen or not; from then on it is a record like any other. A player's starting layout
//!   has an owner, and nothing of it is known until seen ("it would otherwise reveal their start position and
//!   faction"); a neutral object that comes into the game later isn't either.
//! - **Own objects** are never recorded: a player always knows its own objects as they are.
//! - **Hidden objects** are never recorded either (plan decision T32, `super::hidden`: an owned underground structure,
//!   for every player but its owner): the player sees their tiles, not them, so it knows nothing of them there.
//!
//! **Update**, driven by vision changes, never a scan of the map or of every object for every player:
//! - Phase 5f's walk over the objects ([`Vision::update`]) merges, in id order, the footprints of the tile-based
//!   objects ([`Footprints`]) with the walk it already makes for the sight sources; only a footprint that came or
//!   went touches the index from tiles to objects.
//! - Each tile whose state flips for a player in the update (to Visible, or back once its fog delay ends) is noted
//!   (`PlayerVision::see`, `expire`). After the walk (`PlayerVision::catch_up`), a noted tile under no footprint and
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

mod footprints;
#[cfg(test)]
mod tests;

use std::collections::{BTreeMap, BTreeSet};

use glam::Vec2;
use serde::{Deserialize, Serialize};

use super::hidden::hidden_from;
use super::{PlayerVision, TileBits};
use crate::construction::site::{max_hp_at, site_def};
use crate::defs::{ObjectDef, ObjectKind};
use crate::hash::{StateHash, StateHasher};
use crate::ids::{ObjectId, PlayerId};
use crate::object::{Object, Placement, ResourceState};
use crate::progress::Progress;
use crate::tunnel::TunnelState;
use crate::units::{Tick, TilePos};

pub(super) use footprints::{Changes, Footprints};

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
    /// A Tunnel's tier when last seen (`Object::tunnel`, plan decision T39; design change 0258: "Enemies see a
    /// Tunnel's tier"), which its look, its MaxHP and, from slice M4-E on, the area a ghost judges by follow (design
    /// change 0237: "at their remembered position and last-seen tier"). `None` for every other type. Optional in files
    /// and left out of them while `None`, so that the records saved before it existed still load.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tunnel: Option<TunnelState>,
    /// Whether it was a site, a structure under construction, when last seen, and how far its construction had come
    /// then (`Object::construction`'s progress, `crate::construction::site`): a site in the fog is remembered as one,
    /// as last seen, though it may have been finished since (`core/vision.md > Explored`), and its look and its MaxHP
    /// follow (slice M4-I2). Not its builder, held inside it (`factions/syndicate/units/agent.md > While the Tunnel is
    /// under construction`: enemies see "the site, like any surface structure, but not the Agent inside it"). `None`
    /// for a finished structure and a resource. Optional in files and left out of them while `None`, as `tunnel`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub site: Option<Progress>,
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
            // The tier alone: a Tunnel's eject queue is its owner's business (`crate::network`).
            tunnel: object.tunnel.as_ref().map(|tunnel| TunnelState::at(tunnel.tier)),
            // The progress alone: the builder is its owner's business.
            site: object.construction.map(|site| site.progress),
        })
    }

    pub fn def(&self) -> &'static ObjectDef {
        self.kind.def()
    }

    /// The MaxHP of the object as last seen: its type's, a Tunnel's its tier's, a site's the ConstructionHP Rule's at
    /// its progress (`Object::max_hp`). `None` for an indestructible type.
    pub fn max_hp(&self) -> Option<f32> {
        let tier = (crate::tunnel::def_of(self.kind)).and_then(|def| def.tier(self.tunnel.as_ref()?.tier));
        let full = tier.map_or(self.def().max_hp, |tier| Some(tier.max_hp));
        match (&self.site, site_def(self.kind)) {
            (Some(progress), Some(def)) => full.map(|full| max_hp_at(def, progress, full)),
            _ => full,
        }
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
            tunnel,
            site,
        } = self;
        id.hash(h);
        kind.hash(h);
        owner.hash(h);
        placement.hash(h);
        hp.hash(h);
        resource.hash(h);
        seen.hash(h);
        // (M4-A) Only while set, as `Object::tunnel`: the records of games without a Tunnel hash as before.
        if let Some(tunnel) = tunnel {
            h.tag(1);
            tunnel.hash(h);
        }
        // (M4-I2) Only while set too: the records of games in which no one saw a site hash as before.
        if let Some(site) = site {
            h.tag(2);
            site.hash(h);
        }
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
    /// The records the latest update dropped because the player saw their place without their object
    /// ([`PlayerVision::seen_gone`]), by ascending id. Scratch: the sim reads it in the same tick.
    seen_gone: Vec<ObjectId>,
}

impl Memory {
    pub(super) fn new(tiles: usize) -> Self {
        Self {
            records: BTreeMap::new(),
            tiles: BTreeSet::new(),
            bits: TileBits::new(tiles),
            in_sight: BTreeSet::new(),
            revision: 0,
            seen_gone: Vec::new(),
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

    /// The player's records whose footprints share a tile with `placement`, by ascending id: what it knows to stand
    /// there.
    pub fn records_on(&self, placement: Placement) -> impl Iterator<Item = &LastKnown> + '_ {
        let ids: BTreeSet<ObjectId> = footprint_tiles(placement, self.width, self.height)
            .flat_map(|i| on(&self.memory.tiles, i))
            .collect();
        ids.into_iter().filter_map(|id| self.record(id))
    }

    /// The objects whose records the latest update ([`super::Vision::update`], phase 5f) dropped because the player
    /// saw their place without them (`core/vision.md > Explored`: gone, or no longer there), by ascending id. Until
    /// then the player remembered them there, and the orders and rally points that name one went on as if it stood
    /// (design change 0139): the sim ends them in the same tick (`Sim::follow_targets`). Scratch, not state: a
    /// loaded save starts with none, so read it only within the tick of the update.
    pub fn seen_gone(&self) -> &[ObjectId] {
        &self.memory.seen_gone
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

    /// Whether a tile under `placement` is Visible, the fog delay included: the player sees the object standing there.
    fn sees_footprint(&self, placement: Placement) -> bool {
        footprint_tiles(placement, self.width, self.height).any(|i| self.visible_at(i as usize))
    }

    /// After phase 5f's walk: puts the tile-based objects in or out of the player's sight where that may have
    /// changed (on the tiles that flipped, and the footprints that came or went), and drops the records of objects
    /// the player now sees gone.
    pub(super) fn catch_up(&mut self, footprints: &Footprints, changes: &Changes) {
        self.memory.seen_gone.clear();
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
            if self.memory.bits.get(i as usize) && self.visible_at(i as usize) {
                let ids: Vec<ObjectId> = on(&self.memory.tiles, i).collect();
                for id in ids {
                    self.drop_if_gone(id, footprints);
                }
            }
        }
        flips.clear();
        self.flips = flips;
        self.memory.seen_gone.sort_unstable();
        self.memory.seen_gone.dedup();
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
            self.memory.seen_gone.push(id);
        }
    }

    /// The end of the tick ([`super::Vision::remember`]): the records of the objects in the player's sight take
    /// their state at the end of `tick`; an object the player owns has none, and nor has one hidden from the player
    /// (plan decision T32, `super::hidden`: the player sees its place, not it). `object` finds an object by id.
    pub(super) fn refresh<'a>(&mut self, tick: Tick, object: &impl Fn(ObjectId) -> Option<&'a Object>) {
        let in_sight = std::mem::take(&mut self.memory.in_sight);
        for &id in &in_sight {
            match object(id) {
                Some(o) if o.owner == Some(self.id) || hidden_from(o, self.id) => self.forget(id),
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

    /// Object `id` became the player's (`super::Vision::forget_own`): its record, if any, goes.
    pub(super) fn forget_own(&mut self, id: ObjectId) {
        self.forget(id);
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
