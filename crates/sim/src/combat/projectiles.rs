//! Projectiles — `systems/combat.md > DoublyDisjointed` (plan decision T45): "The Firing phase is short and spawns a
//! projectile which travels to the location the target was at at the end of the Aiming phase. The unit is free after
//! the projectile spawns. The effect applies to whatever units are at the target location when the projectile
//! arrives. Targets can dodge by moving out of the locked location."
//!
//! - **The store**: `Sim::projectiles`, every projectile in flight in launch order. A projectile is not an object: it
//!   can't be seen, selected or targeted, and it is hashed and serialized only while the store holds any
//!   (`Sim::state_hash`), so that the states of games without one are as they were before it existed.
//! - **A launch** ([`Projectile::launch`]), at the end of a DoublyDisjointed attack's Firing (`super::phases`): from the
//!   shooter's centre to the locked point, with its attack's parameters copied, so that a projectile whose shooter has
//!   died lands with them all the same, and its arrival tick ([`flight_ticks`]). There is no path and no collision in
//!   flight: the design has none.
//! - **A landing** ([`land`]), in phase 5e of its arrival tick, in launch order: an area attack's blast where it lands
//!   (`super::area`), whose hits go on the tick's list ahead of the tick's new hits. A single-target projectile isn't in
//!   the design (`// TODO(attack types)`): it lands as a blast of radius 0, which hits nothing.
//! - **Not built**: HeadDisjointed projectiles, which track their target. The store keeps a slot for a unit target
//!   ([`Projectile::target`]), so that they come cheaply. A TailDisjointed attack has no projectile: it strikes the
//!   spot it locked at the end of its Firing (`super::phases`, slice M5-O3).

use glam::Vec2;
use serde::{Deserialize, Serialize};

use super::area::{self, Blast};
use super::damage::Hit;
use super::targeting::Targets;
use crate::defs::{AttackAttrs, TargetDomain, TargetType};
use crate::events::SimEvent;
use crate::hash::{StateHash, StateHasher};
use crate::ids::{ObjectId, PlayerId};
use crate::units::{Tick, gu};

/// A projectile in flight (module docs).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Projectile {
    /// The object that launched it. It may have died since: the projectile lands all the same, and reveals it to no
    /// one ([`land`]).
    pub shooter: ObjectId,
    /// The shooter's owner as it launched.
    pub owner: Option<PlayerId>,
    /// The object the shot was aimed at, while it is in the game; none for a shot at the ground (Attack Ground). A
    /// DoublyDisjointed projectile lands where it was locked, whatever became of its target; this is the slot a
    /// HeadDisjointed one would track (module docs). Left out of files while unset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<ObjectId>,
    /// Where it was launched from: the shooter's centre, space units.
    pub from: Vec2,
    /// Where it lands: the location locked at the end of the Aiming phase, space units.
    pub to: Vec2,
    /// The tick in whose phase 5e it lands.
    pub lands_at: Tick,
    /// The attack's Damage.
    pub damage: f32,
    /// The attack's AoERadius, space units; 0 for a single-target attack (module docs).
    pub radius: f32,
    /// The attack's TargetDomain.
    pub domain: TargetDomain,
}

/// `systems/combat.md > DoublyDisjointed`: the ticks a projectile flies from `from` to `to` at `speed` space units per
/// tick: the straight distance over the speed, rounded up to whole ticks (plan decision T45). 0 for a point right at
/// `from`, and for an attack without a ProjectileSpeed (`> DoublyDisjointed > RequiresProjectileSpeed - true`: the
/// defs give every one of them a speed).
pub fn flight_ticks(from: Vec2, to: Vec2, speed: Option<f32>) -> Tick {
    match speed {
        Some(speed) if speed > 0.0 => (from.distance(to) / speed).ceil() as Tick,
        _ => 0,
    }
}

impl Projectile {
    /// The projectile `shooter` (of `owner`) launches in tick `tick` from `from` to `to` with its attack `attrs`, aimed
    /// at `target`, if at an object (module docs).
    pub(super) fn launch(
        shooter: ObjectId,
        owner: Option<PlayerId>,
        attrs: &AttackAttrs,
        from: Vec2,
        to: Vec2,
        target: Option<ObjectId>,
        tick: Tick,
    ) -> Self {
        let radius = match attrs.target_type {
            TargetType::AoE { radius } => gu(radius),
            TargetType::SingleTarget => 0.0,
        };
        Self {
            shooter,
            owner,
            target,
            from,
            to,
            lands_at: tick + flight_ticks(from, to, attrs.projectile_speed),
            damage: attrs.damage,
            radius,
            domain: attrs.target_domain,
        }
    }

    /// The event of its launch.
    pub(super) fn launched(&self) -> SimEvent {
        SimEvent::ProjectileLaunched {
            shooter: self.shooter,
            target: self.target,
            from: self.from,
            to: self.to,
            lands_at: self.lands_at,
        }
    }
}

/// Phase 5e of `tick`: every projectile of `store` due by then lands, in launch order, and leaves the store. Each is a
/// blast where it lands (`SimEvent::Blast`), whose hits on `targets`, the phase's snapshot, go on `hits`
/// ([`area::strike`]). Whom a blast reveals is phase 5e's to say, from the hits (`super::step`).
pub(super) fn land(
    store: &mut Vec<Projectile>,
    tick: Tick,
    targets: &Targets,
    hits: &mut Vec<Hit>,
    events: &mut Vec<SimEvent>,
) {
    store.retain(|p| {
        if p.lands_at > tick {
            return true;
        }
        let blast = Blast {
            attacker: p.shooter,
            at: p.to,
            radius: p.radius,
            damage: p.damage,
            domain: p.domain,
        };
        events.push(SimEvent::Blast {
            at: p.to,
            radius: p.radius,
        });
        area::strike(&blast, targets, hits);
        false
    });
}

/// The removal path (`Sim::remove_objects`): the objects `gone` (sorted by id) left the game. A projectile aimed at one
/// no longer names it, and flies on to its point; one whose shooter is gone keeps its id, which no object takes again.
pub(crate) fn forget(store: &mut [Projectile], gone: &[ObjectId]) {
    for p in store {
        if p.target.is_some_and(|id| gone.binary_search(&id).is_ok()) {
            p.target = None;
        }
    }
}

impl StateHash for TargetDomain {
    fn hash(&self, h: &mut StateHasher) {
        h.tag(match self {
            TargetDomain::Ground => 0,
            TargetDomain::Air => 1,
            TargetDomain::Universal => 2,
        });
    }
}

impl StateHash for Projectile {
    fn hash(&self, h: &mut StateHasher) {
        let Projectile {
            shooter,
            owner,
            target,
            from,
            to,
            lands_at,
            damage,
            radius,
            domain,
        } = self;
        shooter.hash(h);
        owner.hash(h);
        target.hash(h);
        from.hash(h);
        to.hash(h);
        lands_at.hash(h);
        damage.hash(h);
        radius.hash(h);
        domain.hash(h);
    }
}

#[cfg(test)]
mod golden;
#[cfg(test)]
mod perf;
#[cfg(test)]
mod tests;
