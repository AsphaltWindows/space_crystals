//! The footprints of the sim's tile-based objects, which phase 5f's walk keeps up (the module docs of
//! `vision::memory`, **Update**): how the memory finds the objects on a tile.

use std::collections::BTreeSet;
use std::iter::Peekable;
use std::vec;

use super::super::TileBits;
use super::{footprint_tiles, on};
use crate::ids::ObjectId;
use crate::object::{Object, Placement};

/// The tile-based objects of the sim, by id, and the tiles they cover, so that the memory finds the objects on a
/// tile. Derived from the objects, kept up by phase 5f's walk.
#[derive(Clone, Debug)]
pub(in crate::vision) struct Footprints {
    width: u32,
    height: u32,
    /// By ascending id.
    list: Vec<(ObjectId, Placement)>,
    /// `(tile, id)` for every tile each one covers.
    pub(super) tiles: BTreeSet<(u32, ObjectId)>,
    /// The tiles under at least one.
    pub(super) bits: TileBits,
}

/// What a walk found: the footprints that came and those that went (one that changed is both).
#[derive(Default)]
pub(in crate::vision) struct Changes {
    pub(super) added: Vec<(ObjectId, Placement)>,
    pub(super) removed: Vec<(ObjectId, Placement)>,
}

/// Phase 5f's walk over the objects, merging them in id order with the footprints of the walk before.
pub(in crate::vision) struct Walk {
    old: Peekable<vec::IntoIter<(ObjectId, Placement)>>,
    kept: Vec<(ObjectId, Placement)>,
    changes: Changes,
}

impl Footprints {
    pub(in crate::vision) fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            list: Vec::new(),
            tiles: BTreeSet::new(),
            bits: TileBits::new(width as usize * height as usize),
        }
    }

    /// Starts a walk: [`Walk::visit`] every object in id order, then [`Footprints::finish`].
    pub(in crate::vision) fn walk(&mut self) -> Walk {
        let old = std::mem::take(&mut self.list);
        Walk {
            kept: Vec::with_capacity(old.len()),
            old: old.into_iter().peekable(),
            changes: Changes::default(),
        }
    }

    /// Ends a walk: the footprints that went leave the index and those that came join it.
    pub(in crate::vision) fn finish(&mut self, walk: Walk) -> Changes {
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
    pub(super) fn placement(&self, id: ObjectId) -> Option<Placement> {
        let i = self.list.binary_search_by_key(&id, |&(id, _)| id).ok()?;
        Some(self.list[i].1)
    }

    /// The same footprints, indexed the same (V1).
    pub(in crate::vision) fn matches(&self, other: &Footprints) -> bool {
        self.list == other.list && self.tiles == other.tiles && self.bits == other.bits
    }
}

impl Walk {
    /// The next object of the walk, in ascending id order.
    pub(in crate::vision) fn visit(&mut self, o: &Object) {
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
