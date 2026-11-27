//! The state hash of an attack's state (`hash.rs`): [`AttackState`] and the phase of its sequence ([`AttackPhase`]).
//! What slice M4-O added is hashed so that the states of attacks without it hash as they did (the goldens stand): the
//! phases at a location under tags after the others, the location engaged only while set. So is a unit's turret
//! (slice M5-U1, [`TurretState`]): only while it has one; (M5-K) a Target Lock ([`TargetLock`]), only while one is
//! under way; and (M6-GR) a share of a tick banked at low Power, only while there is one.

use super::{AttackPhase, AttackState, TargetLock, TurretState};
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
            turret,
            target_lock,
            power_carry,
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
        // (M5-U1) The same for a turret (plan decision T65): an attack without one hashes as it did.
        if let Some(turret) = turret {
            h.tag(2);
            turret.hash(h);
        }
        // (M5-K) The same for a Target Lock: an attack without one hashes as it did.
        if let Some(lock) = target_lock {
            h.tag(3);
            lock.hash(h);
        }
        // (M6-GR) The same for a share of a tick banked at low Power (`super::power`): an attack at full Power hashes
        // as it did.
        if *power_carry != 0 {
            h.tag(4);
            power_carry.hash(h);
        }
    }
}

impl StateHash for TargetLock {
    fn hash(&self, h: &mut StateHasher) {
        let TargetLock {
            target,
            elapsed,
            locked,
        } = self;
        target.hash(h);
        elapsed.hash(h);
        locked.hash(h);
    }
}

impl StateHash for TurretState {
    fn hash(&self, h: &mut StateHasher) {
        let TurretState { facing, locked } = self;
        facing.hash(h);
        locked.hash(h);
    }
}
