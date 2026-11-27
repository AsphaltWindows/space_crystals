//! (M5-K) Homing projectiles — `systems/combat.md > HeadDisjointed`: "The Firing phase is short and spawns a projectile
//! which tracks the target independently of the attacking unit's further actions. The unit enters Cooldown and
//! Reloading while the projectile is in flight. The effect of the attack is delayed until the projectile reaches the
//! target. The projectile cannot miss." The Rocketman's locked missiles are the first
//! (`factions/gdo/units/rocketman.md > Target Lock`: "every missile it fires at that target is **homing**
//! (HeadDisjointed behavior: tracks the target, cannot miss)").
//!
//! - **The launch** (`super::Projectile::launch`), at the end of a HeadDisjointed attack's Firing, from the shooter's
//!   muzzle at its target (`crate::combat::phases`), the target's centre then its first `Projectile::to`. A homing
//!   projectile is a projectile of the store with its own flight ([`Homing`]): it lands as the others do, in launch
//!   order (`super::land`).
//! - **The flight** ([`steer`]): in each tick after its launch's, in phase 5e before that tick's launches join the
//!   store, it flies its ProjectileSpeed straight at where its target stands then (the phase's snapshot, after
//!   movement), `Projectile::to` following the target. It reaches the target in the tick its distance to the target's
//!   centre is at most its speed, and lands then: on its target, wherever the target is, with its single-target hit
//!   (`super::land`; an area attack's blast there), or with its blast at once for an attack without a speed. Against a
//!   target that stands still it takes a DoublyDisjointed projectile's time ([`super::flight_ticks`]); a target that
//!   runs from it puts its arrival off (`Projectile::lands_at`, the tick it would land in if its target stood where it
//!   is now). Placeholder (not in the design: what it flies at, and when it has reached its target): the target's
//!   centre, where a DoublyDisjointed projectile flies to, and a step at most short of it.
//! - **Nothing but its target** stops it, and vision plays no part: it follows its target whoever sees it, and its
//!   shooter may die meanwhile ("independently of the attacking unit's further actions"). A target faster than the
//!   missile would outrun it for as long as it ran: none is (the Sparrow, the fastest, flies at most 10 su a frame, the
//!   Rocketman's missile 16).
//! - **A target that leaves the game in flight**, killed by another attack or taken off the map (`super::forget`).
//!   Placeholder (not in the design, which says only "The projectile cannot miss"): the missile flies on to where its
//!   target last stood and lands there, hitting nothing, as a FullyConnected shot at a target destroyed before it lands
//!   "hits nothing" (`systems/combat.md > FullyConnected > Hit timing and overkill`: "Overkill is intentionally
//!   wasteful"). (M5-S8) The Rocketman's locked missile, area damage since design change 0445 ("bursting on the
//!   target"), bursts there all the same, in the domain of what it was fired at, fixed as it left: harmlessly, or on
//!   whatever of that domain is there.

use glam::Vec2;
use serde::{Deserialize, Serialize};

use super::{Projectile, flight_ticks};
use crate::combat::targeting::Targets;
use crate::hash::{StateHash, StateHasher};
use crate::units::Tick;

/// A HeadDisjointed projectile's flight (module docs), `Projectile::homing`.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Homing {
    /// Where it is, space units: at the shooter's muzzle as it leaves, then a step of `speed` nearer its target each
    /// tick.
    pub pos: Vec2,
    /// Where it was a tick before, space units, as `Object::prev_pos` is: the client draws it between the two.
    pub prev: Vec2,
    /// Its attack's ProjectileSpeed, space units per tick; 0 for an attack without one, which reaches its target at
    /// once.
    pub speed: f32,
}

impl Homing {
    /// The flight of a projectile leaving `from` at `speed` space units per tick.
    pub(super) fn leaving(from: Vec2, speed: Option<f32>) -> Self {
        Self {
            pos: from,
            prev: from,
            speed: speed.filter(|&speed| speed > 0.0).unwrap_or(0.0),
        }
    }
}

/// Phase 5e of `tick`, before the tick's launches join `store`: each homing projectile of `store` flies one step at
/// its target, where `targets`, the phase's snapshot, has it, or on at the point it flew at last for one whose target
/// left the game (module docs). One that reaches it is due now (`Projectile::lands_at`), at the target's centre, and
/// lands in this phase (`super::land`); the others are due when they would reach it if it stood still.
pub(in crate::combat) fn steer(store: &mut [Projectile], tick: Tick, targets: &Targets) {
    for p in store {
        let Some(homing) = &mut p.homing else {
            continue;
        };
        if let Some(t) = p.target.and_then(|id| targets.get(id)) {
            p.to = t.pos;
        }
        homing.prev = homing.pos;
        let left = homing.pos.distance(p.to);
        if homing.speed <= 0.0 || left <= homing.speed {
            homing.pos = p.to;
            p.lands_at = tick;
        } else {
            homing.pos += (p.to - homing.pos) * (homing.speed / left);
            p.lands_at = tick + flight_ticks(homing.pos, p.to, Some(homing.speed));
        }
    }
}

impl StateHash for Homing {
    fn hash(&self, h: &mut StateHasher) {
        let Homing { pos, prev, speed } = self;
        pos.hash(h);
        prev.hash(h);
        speed.hash(h);
    }
}

#[cfg(test)]
mod tests;
