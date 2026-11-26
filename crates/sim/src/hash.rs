//! Canonical state hashing for [`crate::Sim::state_hash`].
//!
//! Each state type writes its fields to a [`StateHasher`] in a fixed little-endian encoding:
//! integers as their bytes, `f32` as its bits (`to_bits`), `bool` as one byte, strings and
//! sequences with their length first, `Option` and enums as a tag byte plus their fields.
//! The hasher is FNV-1a 64, which is fully specified, so the same state gives the same hash
//! on every machine and Rust version. `std`'s `DefaultHasher` gives no such guarantee.
//!
//! Every impl destructures its type without `..`, so adding a field fails to compile until
//! the field is hashed. Types with private fields implement [`StateHash`] in their own module.

use std::collections::{BTreeSet, VecDeque};

use glam::Vec2;

use crate::defs::ObjectKind;
use crate::ids::{ObjectId, PlayerId};
use crate::map::{Tile, TileLook, TilePreset, TilePresetId};
use crate::object::{Object, Placement, ResourceState, Rotation};
use crate::player::{Faction, Player};
use crate::sim::SimConfig;
use crate::units::TilePos;

/// FNV-1a, 64-bit (Fowler, Noll, Vo).
pub(crate) struct StateHasher(u64);

impl StateHasher {
    const OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;

    pub(crate) fn new() -> Self {
        Self(Self::OFFSET_BASIS)
    }

    pub(crate) fn finish(&self) -> u64 {
        self.0
    }

    pub(crate) fn bytes(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.0 ^= u64::from(b);
            self.0 = self.0.wrapping_mul(Self::PRIME);
        }
    }

    /// Lengths and counts, as `u64` so the encoding doesn't depend on the platform.
    pub(crate) fn len(&mut self, len: usize) {
        (len as u64).hash(self);
    }

    pub(crate) fn tag(&mut self, tag: u8) {
        self.bytes(&[tag]);
    }
}

pub(crate) trait StateHash {
    fn hash(&self, h: &mut StateHasher);
}

macro_rules! hash_le_bytes {
    ($($t:ty),*) => {
        $(impl StateHash for $t {
            fn hash(&self, h: &mut StateHasher) {
                h.bytes(&self.to_le_bytes());
            }
        })*
    };
}
hash_le_bytes!(u8, u16, u32, u64, i32);

impl StateHash for f32 {
    fn hash(&self, h: &mut StateHasher) {
        self.to_bits().hash(h);
    }
}

impl StateHash for bool {
    fn hash(&self, h: &mut StateHasher) {
        h.tag(u8::from(*self));
    }
}

impl StateHash for char {
    fn hash(&self, h: &mut StateHasher) {
        u32::from(*self).hash(h);
    }
}

impl StateHash for String {
    fn hash(&self, h: &mut StateHasher) {
        h.len(self.len());
        h.bytes(self.as_bytes());
    }
}

impl<T: StateHash> StateHash for Option<T> {
    fn hash(&self, h: &mut StateHasher) {
        match self {
            None => h.tag(0),
            Some(v) => {
                h.tag(1);
                v.hash(h);
            }
        }
    }
}

impl<T: StateHash> StateHash for Vec<T> {
    fn hash(&self, h: &mut StateHasher) {
        h.len(self.len());
        for v in self {
            v.hash(h);
        }
    }
}

/// Like a `Vec`: front to back, length first.
impl<T: StateHash> StateHash for VecDeque<T> {
    fn hash(&self, h: &mut StateHasher) {
        h.len(self.len());
        for v in self {
            v.hash(h);
        }
    }
}

/// Like a `Vec`: in order, length first.
impl<T: StateHash> StateHash for BTreeSet<T> {
    fn hash(&self, h: &mut StateHasher) {
        h.len(self.len());
        for v in self {
            v.hash(h);
        }
    }
}

impl StateHash for Vec2 {
    fn hash(&self, h: &mut StateHasher) {
        self.x.hash(h);
        self.y.hash(h);
    }
}

impl StateHash for TilePos {
    fn hash(&self, h: &mut StateHasher) {
        let TilePos { x, y } = self;
        x.hash(h);
        y.hash(h);
    }
}

impl StateHash for ObjectId {
    fn hash(&self, h: &mut StateHasher) {
        self.0.hash(h);
    }
}

impl StateHash for PlayerId {
    fn hash(&self, h: &mut StateHasher) {
        self.0.hash(h);
    }
}

impl StateHash for TilePresetId {
    fn hash(&self, h: &mut StateHasher) {
        self.0.hash(h);
    }
}

impl StateHash for ObjectKind {
    fn hash(&self, h: &mut StateHasher) {
        // Fieldless enum: the discriminant is the declaration index.
        (*self as u32).hash(h);
    }
}

impl StateHash for Faction {
    fn hash(&self, h: &mut StateHasher) {
        (*self as u32).hash(h);
    }
}

impl StateHash for Rotation {
    fn hash(&self, h: &mut StateHasher) {
        (*self as u32).hash(h);
    }
}

impl StateHash for Tile {
    fn hash(&self, h: &mut StateHasher) {
        let Tile { preset } = self;
        preset.hash(h);
    }
}

impl StateHash for TilePreset {
    fn hash(&self, h: &mut StateHasher) {
        let TilePreset {
            name,
            buildable,
            traversible,
            rugged,
            drillable,
            recruitable,
            look,
        } = self;
        name.hash(h);
        buildable.hash(h);
        traversible.hash(h);
        rugged.hash(h);
        drillable.hash(h);
        recruitable.hash(h);
        look.hash(h);
    }
}

impl StateHash for TileLook {
    fn hash(&self, h: &mut StateHasher) {
        (*self as u32).hash(h);
    }
}

impl StateHash for Player {
    fn hash(&self, h: &mut StateHasher) {
        let Player {
            id,
            name,
            faction,
            crystals,
            supplies,
            alloys,
            essence,
            flux,
            defeated_at,
            techs,
        } = self;
        id.hash(h);
        name.hash(h);
        faction.hash(h);
        crystals.hash(h);
        supplies.hash(h);
        alloys.hash(h);
        essence.hash(h);
        flux.hash(h);
        defeated_at.hash(h);
        // (M5-N3) Only while the player has completed some, so that a game without research hashes as it did.
        if !techs.is_empty() {
            h.tag(1);
            techs.hash(h);
        }
    }
}

impl StateHash for Placement {
    fn hash(&self, h: &mut StateHasher) {
        let Placement {
            origin,
            w,
            h: height,
            rotation,
            flip_h,
            flip_v,
        } = self;
        origin.hash(h);
        w.hash(h);
        height.hash(h);
        rotation.hash(h);
        flip_h.hash(h);
        flip_v.hash(h);
    }
}

impl StateHash for ResourceState {
    fn hash(&self, h: &mut StateHasher) {
        match self {
            ResourceState::CrystalPatch { remaining } => {
                h.tag(0);
                remaining.hash(h);
            }
            ResourceState::SupplyStation {
                delivery_size,
                delivery_interval,
                current_supplies,
                next_delivery_in,
            } => {
                h.tag(1);
                delivery_size.hash(h);
                delivery_interval.hash(h);
                current_supplies.hash(h);
                next_delivery_in.hash(h);
            }
        }
    }
}

impl StateHash for SimConfig {
    fn hash(&self, h: &mut StateHasher) {
        let SimConfig { debug } = self;
        debug.hash(h);
    }
}

impl StateHash for Object {
    fn hash(&self, h: &mut StateHasher) {
        let Object {
            id,
            kind,
            owner,
            pos,
            prev_pos,
            facing,
            prev_facing,
            hp,
            shields,
            placement,
            resource,
            orders,
            mover,
            attack,
            riot_shield,
            extractor,
            production,
            constructor,
            worker,
            transform,
            conversion,
            tunnel,
            construction,
            decay,
            popup,
            lineage,
            augmentation,
            energy,
        } = self;
        id.hash(h);
        kind.hash(h);
        owner.hash(h);
        pos.hash(h);
        prev_pos.hash(h);
        facing.hash(h);
        prev_facing.hash(h);
        hp.hash(h);
        shields.hash(h);
        placement.hash(h);
        resource.hash(h);
        // Their impls live with their types (`orders.rs`, `mover.rs`, `combat.rs`, `riot_shield.rs`,
        // `economy/mining.rs`, `production.rs`, `construction.rs`, `economy/gathering.rs`, `transform.rs`,
        // `conversion.rs`).
        orders.hash(h);
        mover.hash(h);
        attack.hash(h);
        riot_shield.hash(h);
        extractor.hash(h);
        production.hash(h);
        constructor.hash(h);
        worker.hash(h);
        transform.hash(h);
        conversion.hash(h);
        // (M4-A) Only while set (`tunnel.rs`), under a tag of its own: the states of games without a Tunnel hash as
        // they did before the field existed, so the goldens stand.
        if let Some(tunnel) = tunnel {
            h.tag(1);
            tunnel.hash(h);
        }
        // (M4-E) The same for a site (`construction/site.rs`), under a tag of its own.
        if let Some(construction) = construction {
            h.tag(2);
            construction.hash(h);
        }
        // (M4-H) The same for a neutral expansion's decay (`sim/tunnels.rs`), under a tag of its own.
        if let Some(decay) = decay {
            h.tag(3);
            decay.hash(h);
        }
        // (M4-S2) The same for a turret's pop-up state (`popup.rs`), under a tag of its own: only a turret has one.
        if let Some(popup) = popup {
            h.tag(4);
            popup.hash(h);
        }
        // (M5-A) The same for a Cults unit's lineage, the Hall that produced each Recruit it embodies (plan decision
        // T56, one entry per Recruit since slice M5-C), under a tag of its own: only while it has an entry.
        if !lineage.is_empty() {
            h.tag(5);
            lineage.hash(h);
        }
        // (M5-F2) The same for a Cults infantry unit's Augmentations (`crate::augmentation`), under a tag of its own:
        // only while it has one.
        if !augmentation.is_none() {
            h.tag(6);
            augmentation.hash(h);
        }
        // (M5-N4a) The same for an object's energy (`crate::energy`), under a tag of its own: only while it has an
        // energy pool.
        if let Some(energy) = energy {
            h.tag(7);
            energy.hash(h);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fnv(bytes: &[u8]) -> u64 {
        let mut h = StateHasher::new();
        h.bytes(bytes);
        h.finish()
    }

    fn hash_of(v: &impl StateHash) -> u64 {
        let mut h = StateHasher::new();
        v.hash(&mut h);
        h.finish()
    }

    /// Published FNV-1a 64 test vectors.
    #[test]
    fn fnv1a_64_test_vectors() {
        assert_eq!(fnv(b""), 0xcbf29ce484222325);
        assert_eq!(fnv(b"a"), 0xaf63dc4c8601ec8c);
        assert_eq!(fnv(b"foobar"), 0x85944171f73967e8);
    }

    #[test]
    fn encoding_separates_values() {
        // Length prefixes: ["ab", "c"] and ["a", "bc"] must differ.
        let a = vec!["ab".to_string(), "c".to_string()];
        let b = vec!["a".to_string(), "bc".to_string()];
        assert_ne!(hash_of(&a), hash_of(&b));
        // Option tags: None and Some(0) differ.
        assert_ne!(hash_of(&None::<u32>), hash_of(&Some(0u32)));
        // Floats hash their bits: 0.0 and -0.0 differ.
        assert_ne!(hash_of(&0.0f32), hash_of(&-0.0f32));
    }
}
