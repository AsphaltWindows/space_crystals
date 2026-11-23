//! What an attack can target, and from how far — `systems/combat.md > ValidTarget`, `> AttackAttributes` (Range,
//! MinRange, TargetDomain), `core/vision.md > ElevationModifier`, and `docs/movement.md > 2`, gap G7 (question
//! Q0009).

use std::collections::BTreeMap;

use glam::Vec2;

use super::damage::pre_armor_multiplier;
use crate::defs::{AttackAttrs, AttackType, Domain, FullyConnectedSubtype, TargetDomain};
use crate::elevation::{self, Elevation};
use crate::ids::ObjectId;
use crate::map::Map;
use crate::object::Object;
use crate::units::{SU_PER_GU, gu};

/// How Range and MinRange are measured (question Q0009, `docs/movement.md > 2`, gap G7): the options the question
/// offers.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RangeMeasure {
    /// Option A: from the attacker's centre to the nearest point of the target's silhouette or footprint, so a unit
    /// can fire at a building's edge.
    CentreToNearestPoint,
    /// Option B: from centre to centre.
    CentreToCentre,
    /// Option C: from edge to edge, between the nearest points of the attacker's silhouette and the target's shape.
    EdgeToEdge,
}

/// The measure in force: option A, the default of question Q0009, until the Designer answers it.
pub const RANGE_MEASURE: RangeMeasure = RangeMeasure::CentreToNearestPoint;

/// The distance from `from` to the nearest point of the axis-aligned box `lo..hi`; 0 inside it.
pub fn distance_to_box(from: Vec2, lo: Vec2, hi: Vec2) -> f32 {
    (lo - from).max(from - hi).max(Vec2::ZERO).length()
}

/// The distance an attack's Range is measured over ([`RANGE_MEASURE`]), from an attacker at `pos` whose silhouette
/// has half extents `half` to a target whose shape is the box `lo..hi` ([`object_box`]).
pub fn range_distance(pos: Vec2, half: Vec2, lo: Vec2, hi: Vec2) -> f32 {
    measure(RANGE_MEASURE, pos, half, lo, hi)
}

fn measure(how: RangeMeasure, pos: Vec2, half: Vec2, lo: Vec2, hi: Vec2) -> f32 {
    match how {
        RangeMeasure::CentreToNearestPoint => distance_to_box(pos, lo, hi),
        RangeMeasure::CentreToCentre => pos.distance((lo + hi) / 2.0),
        // The attacker's box against the target's: its centre against the target's box grown by its half extents.
        RangeMeasure::EdgeToEdge => distance_to_box(pos, lo - half, hi + half),
    }
}

/// The shape Range is measured to (gap G7): a unit's silhouette, an axis-aligned box around its position that never
/// rotates (question Q0009, gap G6), or a tile-based object's footprint.
pub fn object_box(o: &Object) -> (Vec2, Vec2) {
    if let Some(p) = &o.placement {
        let lo = p.origin.corner();
        return (lo, lo + Vec2::new(p.w as f32, p.h as f32) * SU_PER_GU);
    }
    let half = o.def().half_extents().unwrap_or(Vec2::ZERO);
    (o.pos - half, o.pos + half)
}

/// The domain an object is targeted in: a unit's base domain (`systems/units.md > UnitBase > Domain`); structures
/// stand on the ground.
pub fn object_domain(o: &Object) -> Domain {
    o.def()
        .unit
        .as_ref()
        .map_or(Domain::Ground, |unit| unit.base_def().domain)
}

/// `systems/combat.md > ValidTarget` 3, `> SingleTargetDamage`: Ground attacks target Ground units and surfaced
/// Underground ones, Air attacks Air units, and Universal attacks any unit.
pub fn domain_allows(attack: TargetDomain, target: Domain) -> bool {
    match (attack, target) {
        (TargetDomain::Universal, _) | (TargetDomain::Ground, Domain::Ground) | (TargetDomain::Air, Domain::Air) => {
            true
        }
        // TODO(underground units): a surfaced Underground unit (`systems/units.md > DrillUnit`, its above-ground mode)
        // is a Ground target. No unit surfaces yet, so every Underground unit counts as submerged.
        (TargetDomain::Ground, Domain::Underground) => false,
        (TargetDomain::Ground, Domain::Air) | (TargetDomain::Air, Domain::Ground | Domain::Underground) => false,
    }
}

/// The farthest an attack reaches, space units: its Range with the ElevationModifier from `source` against
/// `target` (`core/vision.md > ElevationModifier`), which melee attacks don't take (`systems/combat.md >
/// FullyConnected > Melee`).
pub fn max_range(attrs: &AttackAttrs, source: Elevation, target: Elevation) -> f32 {
    let range = match attrs.attack_type {
        AttackType::FullyConnected(FullyConnectedSubtype::Melee) => attrs.range,
        AttackType::FullyConnected(FullyConnectedSubtype::Ranged)
        | AttackType::HeadDisjointed
        | AttackType::TailDisjointed
        | AttackType::DoublyDisjointed => elevation::attack_range(attrs.range, source, target),
    };
    gu(range)
}

/// Whether a target `distance` space units away ([`range_distance`]) is within the attack's Range and beyond its
/// MinRange (`systems/controls/behaviors.md > AttackingObject` 2). The ElevationModifier changes the Range only.
/// Both bounds are inclusive: a placeholder reading, the design doesn't say.
pub fn within_range(attrs: &AttackAttrs, distance: f32, source: Elevation, target: Elevation) -> bool {
    distance >= gu(attrs.min_range) && distance <= max_range(attrs, source, target)
}

/// Whether `attacker` could attack `target` where both stand now: it has an attack, `target` is destructible and
/// not `attacker` itself, the attack's TargetDomain allows it, and it is within Range and beyond MinRange. Facing is
/// not required: Aiming turns the unit (`systems/combat.md > Aiming`). Neither is vision yet (TODO(slice V):
/// `> ValidTarget` 2). For the behaviours (slice E: chase until this holds) and tests.
pub fn can_attack_now(map: &Map, attacker: &Object, target: &Object) -> bool {
    let Some(attrs) = attacker.def().unit.as_ref().and_then(|unit| unit.attack.as_ref()) else {
        return false;
    };
    let (lo, hi) = object_box(target);
    let half = attacker.def().half_extents().unwrap_or(Vec2::ZERO);
    target.id != attacker.id
        && target.hp.is_some()
        && domain_allows(attrs.target_domain, object_domain(target))
        && within_range(
            attrs,
            range_distance(attacker.pos, half, lo, hi),
            Elevation::of_object(map, attacker),
            Elevation::of_object(map, target),
        )
}

/// A destructible object as every attacker sees it this tick.
#[derive(Clone, Debug)]
pub(super) struct TargetView {
    pub id: ObjectId,
    pub pos: Vec2,
    pub facing: f32,
    /// Its shape ([`object_box`]).
    pub lo: Vec2,
    pub hi: Vec2,
    pub domain: Domain,
    pub elevation: Elevation,
    /// `systems/combat.md > DamageModifierOrder` 1: the product of its pre-armor multipliers where it stands.
    pub pre_armor: f32,
    /// `systems/units.md > Unit > PointArmor`; 0 for objects without unit attributes.
    pub point_armor: f32,
    /// `systems/units.md > UnitBase > DirectionalArmor`.
    pub directional_armor: bool,
}

/// Every destructible object, in id order: phase 5e's snapshot, taken before any unit acts, so that every attacker
/// sees the same tick (positions after movement, HP before this tick's hits).
pub(super) struct Targets(Vec<TargetView>);

impl Targets {
    pub(super) fn snapshot(map: &Map, objects: &BTreeMap<ObjectId, Object>) -> Self {
        let views = objects
            .values()
            .filter(|o| o.hp.is_some())
            .map(|o| {
                let (lo, hi) = object_box(o);
                let unit = o.def().unit.as_ref();
                TargetView {
                    id: o.id,
                    pos: o.pos,
                    facing: o.facing,
                    lo,
                    hi,
                    domain: object_domain(o),
                    elevation: Elevation::of_object(map, o),
                    pre_armor: pre_armor_multiplier(map, o),
                    point_armor: unit.map_or(0.0, |u| u.point_armor),
                    directional_armor: unit.is_some_and(|u| u.base_def().directional_armor),
                }
            })
            .collect();
        Self(views)
    }

    pub(super) fn get(&self, id: ObjectId) -> Option<&TargetView> {
        let i = self.0.binary_search_by_key(&id, |t| t.id).ok()?;
        self.0.get(i)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_distance_to_a_box_is_to_its_nearest_point() {
        let (lo, hi) = (Vec2::new(100.0, 100.0), Vec2::new(164.0, 132.0));
        assert_eq!(distance_to_box(Vec2::new(130.0, 120.0), lo, hi), 0.0, "inside");
        assert_eq!(distance_to_box(Vec2::new(40.0, 110.0), lo, hi), 60.0, "west of it");
        assert_eq!(distance_to_box(Vec2::new(130.0, 172.0), lo, hi), 40.0, "south of it");
        assert_eq!(distance_to_box(Vec2::new(167.0, 136.0), lo, hi), 5.0, "past a corner");
    }

    /// Question Q0009's three options, for a 24 su attacker 100 su west of a 64 su footprint's centre.
    #[test]
    fn the_three_measures_of_question_q0009() {
        let (pos, half) = (Vec2::new(0.0, 32.0), Vec2::splat(12.0));
        let (lo, hi) = (Vec2::new(68.0, 0.0), Vec2::new(132.0, 64.0));
        assert_eq!(measure(RangeMeasure::CentreToNearestPoint, pos, half, lo, hi), 68.0);
        assert_eq!(measure(RangeMeasure::CentreToCentre, pos, half, lo, hi), 100.0);
        assert_eq!(measure(RangeMeasure::EdgeToEdge, pos, half, lo, hi), 56.0);
        assert_eq!(
            RANGE_MEASURE,
            RangeMeasure::CentreToNearestPoint,
            "the default until Q0009 is answered"
        );
        assert_eq!(range_distance(pos, half, lo, hi), 68.0);
    }

    #[test]
    fn target_domains() {
        use Domain::{Air, Ground, Underground};
        let allowed = |attack| {
            [Ground, Air, Underground]
                .into_iter()
                .filter(|&d| domain_allows(attack, d))
                .collect::<Vec<_>>()
        };
        assert_eq!(allowed(TargetDomain::Ground), [Ground]);
        assert_eq!(allowed(TargetDomain::Air), [Air]);
        assert_eq!(allowed(TargetDomain::Universal), [Ground, Air, Underground]);
    }
}
