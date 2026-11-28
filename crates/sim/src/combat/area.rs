//! Area damage — `systems/combat.md > AoEDamage` (plan decision T46): the one path of every area attack in the game.
//! A grenade's blast calls it now (`super::projectiles`), and (M5-S8) the Rocketman's missiles' bursts, normal and
//! locked (design change 0445); the mortar's, the bombs', the shells' and the mines' will. Single-target damage keeps
//! its own path (`super::damage`).
//!
//! - **Who is hit** ([`hits_domain`]): every destructible object whose domain the blast's TargetDomain reaches, the
//!   attack's, or (M5-S8) for the Rocketman's missiles the domain of what they were fired at ([`blast_domain`]), any
//!   owner, the attacker's own objects and the attacker itself included: `systems/combat.md > AoEDamage`, "**Friendly
//!   fire** ... area damage hits **every** object in the circle whose domain matches, **whoever owns it**: own units
//!   and structures, the attacker itself, allies, enemies and neutral objects" (design change 0293, answering Q0083
//!   point 1). Underground objects too, seen or not (design change 0262, keeping 0218: "underground DrillUnits,
//!   Syndicate expansions, a retracted pop-up Laser Turret and GDO mines take their share of any blast over them, even
//!   unseen"): this module never asks whom an object is hidden from (`crate::vision::hidden`).
//! - **How much** ([`strike`]): the exact area of the circle inside each object's shape ([`overlap`]: its footprint,
//!   or its Silhouette, axis-aligned like every [`crate::shape::Shape`]) makes its damage share, its effective
//!   FullArmor and its minimum; the directional zone is taken from the circle's centre. Each object hit is one [`Hit`]
//!   on the tick's list, which resolves it in `> DamageModifierOrder`'s steps like any hit, its Shields first at full
//!   value ("For AoE, the full value is the unit's damage share": `super::resolve`).
//! - **Determinism**: the overlap is exact, in `f32`, with only basic arithmetic and `crate::math`'s `asin` and `sqrt`
//!   (libm): the same bits on every machine (plan decision T46). A unit test pins some of them.

use glam::Vec2;

use super::damage::{Hit, directional_multiplier};
use super::targeting::{TargetView, Targets};
use crate::defs::{AttackAttrs, BlastDomain, Domain, TargetDomain, TargetType};
use crate::ids::ObjectId;
use crate::math::{self, PI};
use crate::ruleset::rules;

/// An area attack's effect where it lands (`systems/combat.md > AoEDamage`): a circle of uniform damage.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Blast {
    /// The object whose attack it is; it may be gone (a projectile lands even if its shooter died: plan decision T45).
    pub attacker: ObjectId,
    /// The circle's centre, space units.
    pub at: Vec2,
    /// The AoERadius, space units.
    pub radius: f32,
    /// The attack's Damage: the total over the circle.
    pub damage: f32,
    /// The attack's TargetDomain ([`hits_domain`]).
    pub domain: TargetDomain,
}

/// `systems/combat.md > AoEDamage`: whether area damage of TargetDomain `attack` hits an object of domain `target`
/// (structures stand on the ground: `super::object_domain`). "Ground AoE hits Ground and surfaced Underground units, Air
/// AoE hits Air units, Universal AoE hits all units regardless of domain", and "Underground objects are hit by area
/// damage too (... Ground and Universal AoE)": a Ground blast hits every Underground unit, surfaced or not (a surfaced
/// one is a Ground unit by its type, M6-ST1's `UnitDef::domain`), where a single-target Ground attack targets only
/// surfaced ones (`super::domain_allows`).
pub fn hits_domain(attack: TargetDomain, target: Domain) -> bool {
    match (attack, target) {
        (TargetDomain::Universal, _)
        | (TargetDomain::Ground, Domain::Ground | Domain::Underground)
        | (TargetDomain::Air, Domain::Air) => true,
        (TargetDomain::Ground, Domain::Air) | (TargetDomain::Air, Domain::Ground | Domain::Underground) => false,
    }
}

/// (M5-S8) The TargetDomain of the blast of attack `attrs` fired at an object of domain `target`, or at the ground with
/// none (an Attack Ground), fixed as it is fired: by its def's [`BlastDomain`] (`TargetType::AoE`).
/// - **The attack's** (`BlastDomain::Attack`, every area attack's but the Rocketman's): its TargetDomain, "AoE only
///   affects units whose domain matches the attack's TargetDomain" (`systems/combat.md > AoEDamage`), a Universal
///   blast every domain. A single-target attack's projectile hits in its TargetDomain too (`super::projectiles`).
/// - **The target's** (`BlastDomain::Target`, the Rocketman's missiles: `factions/gdo/units/rocketman.md >
///   AttackAttributes (normal attack)`, design change 0445): "it keeps the domain of what it was fired at: fired at an
///   **air** unit it hits only air units; fired at a **ground** unit or the ground, only ground (and underground)
///   objects", Air for an air target and Ground for any other, a ground or an underground one, or the ground (the
///   Ground blast hits ground and underground objects alike: [`hits_domain`]).
///
/// Placeholder (not in the design: what a missile fired at an object that left the game while it was being fired, in
/// its Firing, keeps): it is launched at the spot alone (`AttackPhase::FiringAt`'s target is cleared then), as at the
/// ground, a Ground blast.
pub(super) fn blast_domain(attrs: &AttackAttrs, target: Option<Domain>) -> TargetDomain {
    match (attrs.target_type, target) {
        (
            TargetType::AoE {
                blast: BlastDomain::Target,
                ..
            },
            Some(Domain::Air),
        ) => TargetDomain::Air,
        (
            TargetType::AoE {
                blast: BlastDomain::Target,
                ..
            },
            Some(Domain::Ground | Domain::Underground) | None,
        ) => TargetDomain::Ground,
        (
            TargetType::SingleTarget
            | TargetType::AoE {
                blast: BlastDomain::Attack,
                ..
            },
            _,
        ) => attrs.target_domain,
    }
}

/// The area of the part of the circle of `radius` around `centre` that lies inside the axis-aligned rectangle with
/// corners `lo` and `hi` (space units, square space units back): exact, up to `f32` rounding, and the same bits on every
/// machine (module docs). 0 for a rectangle the circle doesn't reach, the rectangle's area for one wholly inside the
/// circle, the circle's for a circle wholly inside the rectangle.
pub fn overlap(centre: Vec2, radius: f32, lo: Vec2, hi: Vec2) -> f32 {
    let (lo, hi) = (lo - centre, hi - centre);
    let r = radius;
    if r <= 0.0 || hi.x <= lo.x || hi.y <= lo.y {
        return 0.0;
    }
    let squared = |v: Vec2| v.x * v.x + v.y * v.y;
    // The rectangle's point nearest to the centre, and its corner farthest from it.
    let nearest = Vec2::ZERO.clamp(lo, hi);
    if squared(nearest) >= r * r {
        return 0.0;
    }
    let farthest = lo.abs().max(hi.abs());
    let rect = (hi.x - lo.x) * (hi.y - lo.y);
    if squared(farthest) <= r * r {
        return rect;
    }
    let circle = PI * r * r;
    if lo.x <= -r && hi.x >= r && lo.y <= -r && hi.y >= r {
        return circle;
    }
    // The circle's halves above and below its centre's line, each folded onto the upper side.
    let area = if lo.y >= 0.0 {
        above(lo.x, hi.x, lo.y, r) - above(lo.x, hi.x, hi.y, r)
    } else if hi.y <= 0.0 {
        above(lo.x, hi.x, -hi.y, r) - above(lo.x, hi.x, -lo.y, r)
    } else {
        2.0 * above(lo.x, hi.x, 0.0, r) - above(lo.x, hi.x, hi.y, r) - above(lo.x, hi.x, -lo.y, r)
    };
    area.clamp(0.0, rect.min(circle))
}

/// The area of the circle of radius `r` around the origin between `x0` and `x1` (`x0 <= x1`) and at or beyond `y`
/// (`y >= 0`), up the y axis: the integral of `√(r² − x²) − y` over the `x` where the circle reaches beyond `y`.
fn above(x0: f32, x1: f32, y: f32, r: f32) -> f32 {
    if y >= r {
        return 0.0;
    }
    // Where the line at `y` crosses the circle.
    let reach = math::sqrt((r * r - y * y).max(0.0));
    let x0 = x0.clamp(-reach, reach);
    let x1 = x1.clamp(-reach, reach);
    // An antiderivative of `√(r² − x²) − y`.
    let integral = |x: f32| {
        let half_chord = math::sqrt((r * r - x * x).max(0.0));
        0.5 * (x * half_chord + r * r * math::asin((x / r).clamp(-1.0, 1.0))) - y * x
    };
    integral(x1) - integral(x0)
}

/// `systems/combat.md > AoEDamage`: the hits of `blast` on `targets` (phase 5e's snapshot: the positions after
/// movement, the HP before this tick's hits), put on the tick's list `hits` in id order, one for each object it
/// reaches ([`hits_domain`]) whose shape the circle overlaps. For an object whose shape has the area `A`, `a` of it
/// inside the circle of area `C`:
/// - its **damage share**, "Attack Damage × (unit overlap area / AoE area)": `Damage × a / C`. Its Shields take that
///   share at full value (`> DamageModifierOrder`: "For AoE, the full value is the unit's damage share"; `super::resolve`);
/// - its **effective armor**, "FullArmor × (unit overlap area / unit total area)": `FullArmor × a / A`, in place of the
///   PointArmor of a single-target hit;
/// - its **directional zone**, from "the angle from the unit's center to the AoE center" (`> DirectionalArmor`, for
///   an object with DirectionalArmor; a blast right on its centre counts as from the front, as a single-target shot
///   from on top of it does: `super::directional_multiplier`);
/// - its **minimum**, "0.5 × overlap fraction": `MinimumDamage × a / A` (`> MinimumDamage > AoE`).
pub(super) fn strike(blast: &Blast, targets: &Targets, hits: &mut Vec<Hit>) {
    let circle = PI * blast.radius * blast.radius;
    if circle <= 0.0 {
        return;
    }
    // Friendly fire (design change 0293, answering Q0083 point 1): every object whose domain the blast hits, whoever
    // owns it, the attacker and its owner's objects included. No owner filter.
    for t in targets.all() {
        if !hits_domain(blast.domain, t.domain) {
            continue;
        }
        let (lo, hi) = t.shape.bounds();
        let total = (hi.x - lo.x) * (hi.y - lo.y);
        let inside = overlap(blast.at, blast.radius, lo, hi);
        if inside <= 0.0 || total <= 0.0 {
            continue;
        }
        let directional = if t.directional_armor {
            directional_multiplier(t.facing, blast.at - t.pos)
        } else {
            1.0
        };
        hits.push(share(
            blast.attacker,
            t,
            blast.damage,
            inside,
            total,
            circle,
            directional,
        ));
    }
}

/// `systems/combat.md > AoEDamage`: the hit on `t` of `attacker`'s area attack of `damage` over an area `area`, `inside`
/// of which lies inside `t`'s shape, whose own area is `total` (`inside` > 0, `total` > 0): its damage share `damage ×
/// inside / area`, its effective FullArmor `FullArmor × inside / total`, its minimum `MinimumDamage × inside / total`,
/// and the DirectionalArmor multiplier `directional`. The one place these rules are written: [`strike`]'s, for a
/// circle, and (M5-N4b) a Shroud's release, over its cloud (`crate::shroud`).
pub(super) fn share(
    attacker: ObjectId,
    t: &TargetView,
    damage: f32,
    inside: f32,
    total: f32,
    area: f32,
    directional: f32,
) -> Hit {
    let fraction = inside / total;
    Hit {
        attacker,
        target: t.id,
        damage: damage * (inside / area),
        pre_armor: t.pre_armor,
        armor: t.full_armor * fraction,
        directional,
        minimum: rules().tuning.combat.minimum_damage * fraction,
    }
}

#[cfg(test)]
mod tests;
