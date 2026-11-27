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
//!   (`super::area`), whose hits go on the tick's list ahead of the tick's new hits. (M5-K) A single-target projectile
//!   (the Rocketman's missile, `factions/gdo/units/rocketman.md`) hits the object at the spot where it lands
//!   (`Targets::at_the_spot`), or nothing: "The effect applies to whatever units are at the target location when the
//!   projectile arrives". Placeholder (not in the design: which object, of those at the spot, a single-target
//!   projectile hits, and whether a structure counts): TailDisjointed's rule (`systems/combat.md > TailDisjointed`,
//!   Q0126: "whatever object (unit or structure) covers the locked point is hit; if several do, the one the shot was
//!   aimed at"), whose words for the spot are DoublyDisjointed's.
//! - **(M5-K) HeadDisjointed projectiles** track their target, and land on it wherever it is ([`homing`]): the
//!   projectiles of the store with a flight of their own ([`Projectile::homing`]). A TailDisjointed attack has no
//!   projectile: it strikes the spot it locked at the end of its Firing (`super::phases`, slice M5-O3).

use glam::Vec2;
use serde::{Deserialize, Serialize};

use super::area::{self, Blast};
use super::damage::{Hit, Origins, directional_multiplier};
use super::targeting::{TargetView, Targets, domain_allows};
use crate::defs::{AttackAttrs, AttackType, TargetDomain, TargetType};
use crate::events::SimEvent;
use crate::hash::{StateHash, StateHasher};
use crate::ids::{ObjectId, PlayerId};
use crate::units::{Tick, gu};

mod homing;

pub use homing::Homing;
pub(super) use homing::steer;

/// A projectile in flight (module docs).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Projectile {
    /// The object that launched it. It may have died since: the projectile lands all the same, and reveals it to no
    /// one ([`land`]).
    pub shooter: ObjectId,
    /// The shooter's owner as it launched.
    pub owner: Option<PlayerId>,
    /// The object the shot was aimed at, while it is in the game; none for a shot at the ground (Attack Ground). A
    /// DoublyDisjointed projectile lands where it was locked, whatever became of its target; (M5-K) a HeadDisjointed
    /// one tracks it ([`homing`]). Left out of files while unset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<ObjectId>,
    /// Where it was launched from: the shooter's centre, space units.
    pub from: Vec2,
    /// Where it lands: the location locked at the end of the Aiming phase, space units; (M5-K) for a homing one, where
    /// its target stands as of its last step, or last stood.
    pub to: Vec2,
    /// The tick in whose phase 5e it lands; (M5-K) for a homing one, the tick it would land in if its target stood
    /// still from now on.
    pub lands_at: Tick,
    /// The attack's Damage.
    pub damage: f32,
    /// The attack's AoERadius, space units; 0 for a single-target attack (module docs).
    pub radius: f32,
    /// The attack's TargetDomain.
    pub domain: TargetDomain,
    /// (M5-K) A HeadDisjointed projectile's flight ([`homing`]); none for a DoublyDisjointed one, which flies straight
    /// to `to`. Left out of files, and of the hash, while unset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub homing: Option<Homing>,
}

/// `systems/combat.md > DoublyDisjointed`: the ticks a projectile flies from `from` to `to` at `speed` space units per
/// tick: the straight distance over the speed, rounded up to whole ticks (plan decision T45). 0 for a point right at
/// `from`, and for an attack without a ProjectileSpeed (`systems/combat.md > DoublyDisjointed >
/// RequiresProjectileSpeed - true`: the defs give every one of them a speed).
pub fn flight_ticks(from: Vec2, to: Vec2, speed: Option<f32>) -> Tick {
    match speed {
        Some(speed) if speed > 0.0 => (from.distance(to) / speed).ceil() as Tick,
        _ => 0,
    }
}

impl Projectile {
    /// The projectile `shooter` (of `owner`) launches in tick `tick` from `from` to `to` with its attack `attrs`, aimed
    /// at `target`, if at an object (module docs); (M5-K) a HeadDisjointed attack's homes in on `target` ([`homing`]),
    /// `to` its centre as it leaves.
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
            homing: matches!(attrs.attack_type, AttackType::HeadDisjointed)
                .then(|| Homing::leaving(from, attrs.projectile_speed)),
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
            homing: self.homing.is_some(),
        }
    }
}

/// Phase 5e of `tick`: every projectile of `store` due by then lands, in launch order, and leaves the store. Each is a
/// blast where it lands (`SimEvent::Blast`), whose hits on `targets`, the phase's snapshot, go on `hits`
/// ([`area::strike`]); (M5-K) a single-target one's, of radius 0, the one hit of [`struck`], if any. Whom a blast
/// reveals is phase 5e's to say, from the hits (`super::step`). (M5-N4b) Its hits leave from where it was launched,
/// `origins` (`super::damage::Origins`: "judged by its position at launch", plan decision T77).
pub(super) fn land(
    store: &mut Vec<Projectile>,
    tick: Tick,
    targets: &Targets,
    hits: &mut Vec<Hit>,
    origins: &mut Origins,
    events: &mut Vec<SimEvent>,
) {
    store.retain(|p| {
        if p.lands_at > tick {
            return true;
        }
        events.push(SimEvent::Blast {
            at: p.to,
            radius: p.radius,
        });
        origins.mark(hits.len(), p.from);
        if p.radius > 0.0 {
            let blast = Blast {
                attacker: p.shooter,
                at: p.to,
                radius: p.radius,
                damage: p.damage,
                domain: p.domain,
            };
            area::strike(&blast, targets, hits);
        } else if let Some(t) = struck(p, targets) {
            hits.push(single_hit(p, t));
        }
        false
    });
}

/// (M5-K) What single-target projectile `p` hits as it lands (module docs), of what its TargetDomain allows, other than
/// its shooter: the object at the spot it lands on ([`Targets::at_the_spot`]); a homing one, its target, which it has
/// reached ("The projectile cannot miss"), or nothing if its target left the game ([`homing`]).
fn struck<'t>(p: &Projectile, targets: &'t Targets) -> Option<&'t TargetView> {
    let hits = |t: &TargetView| t.id != p.shooter && domain_allows(p.domain, t.domain);
    if p.homing.is_some() {
        return p.target.and_then(|id| targets.get(id)).filter(|t| hits(t));
    }
    targets.at_the_spot(p.to, p.target, hits)
}

/// (M5-K) `p`'s single-target hit on `t` (`systems/combat.md > SingleTargetDamage`): its Damage against `t`'s
/// PointArmor, from the side it was launched from (`systems/units.md > UnitBase > DirectionalArmor`; plan decision
/// T77: a projectile's hits are "judged by its position at launch").
fn single_hit(p: &Projectile, t: &TargetView) -> Hit {
    let directional = if t.directional_armor {
        directional_multiplier(t.facing, p.from - t.pos)
    } else {
        1.0
    };
    Hit::single(p.shooter, t.id, p.damage, t.pre_armor, t.point_armor, directional)
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
            homing,
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
        // (M5-K) Only while set: a DoublyDisjointed projectile hashes as it did before homing ones existed.
        if let Some(homing) = homing {
            h.tag(1);
            homing.hash(h);
        }
    }
}

#[cfg(test)]
mod golden;
#[cfg(test)]
mod perf;
#[cfg(test)]
mod tests;
