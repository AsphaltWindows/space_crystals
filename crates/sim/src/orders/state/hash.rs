//! The state hash of the behaviours' working state (`hash.rs`): every field, destructured, so that a new one doesn't
//! compile until it is hashed. The fields added after M2 are hashed only while set, so that a state without them
//! hashes as it did before they existed (the goldens stand).

use super::*;
use crate::hash::{StateHash, StateHasher};

impl StateHash for Pursuit {
    fn hash(&self, h: &mut StateHasher) {
        let Pursuit { target, chasing } = self;
        target.hash(h);
        chasing.hash(h);
    }
}

fn hash_ignored(ignored: &Ignored, h: &mut StateHasher) {
    match ignored {
        None => h.tag(0),
        Some((id, until)) => {
            h.tag(1);
            id.hash(h);
            until.hash(h);
        }
    }
}

impl StateHash for Auto {
    fn hash(&self, h: &mut StateHasher) {
        match self {
            Auto::Watching { ignored } => {
                h.tag(0);
                hash_ignored(ignored, h);
            }
            Auto::Engaging { pursuit, origin } => {
                h.tag(1);
                pursuit.hash(h);
                origin.hash(h);
            }
            Auto::Returning => h.tag(2),
        }
    }
}

impl StateHash for AttackOrder {
    fn hash(&self, h: &mut StateHasher) {
        let AttackOrder {
            chasing,
            last_seen,
            lost,
            remembered,
            on_hold,
            backing_off,
            too_close,
        } = self;
        chasing.hash(h);
        last_seen.hash(h);
        lost.hash(h);
        // (M3-R, M3-L2) Only while set, as `Mover::momentum` (`mover/hash.rs`): an Attack that never went for a
        // remembered target, waited behind a Riot Shield nor had its target inside its MinRange hashes as it did
        // before the fields existed (the goldens stand). A state with one differs from the same state without by its
        // tag and value.
        if *remembered {
            h.tag(1);
        }
        if let Some(follow) = on_hold {
            h.tag(2);
            follow.hash(h);
        }
        if let Some(from) = backing_off {
            h.tag(3);
            from.hash(h);
        }
        if *too_close {
            h.tag(4);
        }
    }
}

impl StateHash for Approach {
    fn hash(&self, h: &mut StateHasher) {
        let Approach {
            moving,
            parked,
            last_seen,
            lost,
            remembered,
        } = self;
        moving.hash(h);
        parked.hash(h);
        last_seen.hash(h);
        lost.hash(h);
        // (M3-R) Only while set, as `AttackOrder`'s.
        if *remembered {
            h.tag(1);
        }
    }
}

impl StateHash for Leg {
    fn hash(&self, h: &mut StateHasher) {
        let Leg {
            to,
            reference,
            planned,
            marching,
            engaged,
            ignored,
            walking,
            resumed,
            dropped,
        } = self;
        to.hash(h);
        reference.hash(h);
        planned.hash(h);
        marching.hash(h);
        engaged.hash(h);
        hash_ignored(ignored, h);
        // (M3-L2) Only while set, as `AttackOrder`'s.
        if !walking.is_empty() {
            h.tag(1);
            walking.hash(h);
        }
        if *resumed {
            h.tag(2);
        }
        if !dropped.is_empty() {
            h.tag(3);
            h.len(dropped.len());
            for Dropped { target, seen } in dropped {
                target.hash(h);
                seen.hash(h);
            }
        }
    }
}

impl StateHash for Patrol {
    fn hash(&self, h: &mut StateHasher) {
        let Patrol {
            origin,
            destination,
            outbound,
            failed_legs,
            leg,
        } = self;
        origin.hash(h);
        destination.hash(h);
        outbound.hash(h);
        failed_legs.hash(h);
        leg.hash(h);
    }
}

impl StateHash for GatherPhase {
    fn hash(&self, h: &mut StateHasher) {
        match self {
            GatherPhase::ToPatch => h.tag(0),
            GatherPhase::Waiting { since } => {
                h.tag(1);
                since.hash(h);
            }
            GatherPhase::Mining { done } => {
                h.tag(2);
                done.hash(h);
            }
            GatherPhase::ToDropOff => h.tag(3),
            GatherPhase::DroppingOff { done } => {
                h.tag(4);
                done.hash(h);
            }
            GatherPhase::Delivered => h.tag(5),
        }
    }
}

impl StateHash for Gather {
    fn hash(&self, h: &mut StateHasher) {
        let Gather {
            phase,
            node,
            moving,
            retry_at,
            spot,
            ending,
        } = self;
        phase.hash(h);
        node.hash(h);
        moving.hash(h);
        retry_at.hash(h);
        spot.hash(h);
        ending.hash(h);
    }
}

impl StateHash for Behaviour {
    fn hash(&self, h: &mut StateHasher) {
        match self {
            Behaviour::Plain => h.tag(0),
            Behaviour::Attack(state) => {
                h.tag(1);
                state.hash(h);
            }
            Behaviour::Approach(state) => {
                h.tag(2);
                state.hash(h);
            }
            Behaviour::AttackMove(leg) => {
                h.tag(3);
                leg.hash(h);
            }
            Behaviour::Patrol(patrol) => {
                h.tag(4);
                patrol.hash(h);
            }
            Behaviour::Hold { target } => {
                h.tag(5);
                target.hash(h);
            }
            Behaviour::Gather(gather) => {
                h.tag(6);
                gather.hash(h);
            }
            Behaviour::Deploy(Deploy { arrived }) => {
                h.tag(7);
                arrived.hash(h);
            }
        }
    }
}
