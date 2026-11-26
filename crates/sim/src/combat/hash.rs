//! The state hash of an attack's state (`hash.rs`): [`AttackState`] and the phase of its sequence ([`AttackPhase`]).
//! What slice M4-O added is hashed so that the states of attacks without it hash as they did (the goldens stand): the
//! phases at a location under tags after the others, the location engaged only while set.

use super::{AttackPhase, AttackState};
use crate::hash::{StateHash, StateHasher};

impl StateHash for AttackPhase {
    fn hash(&self, h: &mut StateHasher) {
        match self {
            AttackPhase::Aiming { target } => {
                h.tag(0);
                target.hash(h);
            }
            AttackPhase::Firing { target } => {
                h.tag(1);
                target.hash(h);
            }
            AttackPhase::Cooldown => h.tag(2),
            AttackPhase::Reloading => h.tag(3),
            AttackPhase::AimingAt { at } => {
                h.tag(4);
                at.hash(h);
            }
            AttackPhase::FiringAt { at, target } => {
                h.tag(5);
                at.hash(h);
                target.hash(h);
            }
        }
    }
}

impl StateHash for AttackState {
    fn hash(&self, h: &mut StateHasher) {
        let AttackState {
            target,
            phase,
            elapsed,
            location,
        } = self;
        target.hash(h);
        phase.hash(h);
        elapsed.hash(h);
        // (M4-O) Only while set, under a tag of its own, as `Mover::momentum` (`mover/hash.rs`): an attack that never
        // engaged a location hashes as it did before the field existed, so the goldens stand.
        if let Some(at) = location {
            h.tag(1);
            at.hash(h);
        }
    }
}
