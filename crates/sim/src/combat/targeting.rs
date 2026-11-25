//! What an attack can target, and from how far — `systems/combat.md > ValidTarget`, `systems/combat.md >
//! AttackAttributes` (Range, MinRange, TargetDomain; `systems/combat.md > Range Measurement`: edge to edge, design
//! change 0031, measured by `shape`) and `core/vision.md > ElevationModifier`.

use std::collections::BTreeMap;

use glam::Vec2;

use super::damage::pre_armor_multiplier;
use crate::defs::{AttackAttrs, AttackType, Domain, FullyConnectedSubtype, TargetDomain};
use crate::elevation::{self, Elevation};
use crate::ids::ObjectId;
use crate::map::Map;
use crate::object::Object;
use crate::riot_shield;
use crate::shape::Shape;
use crate::units::gu;
use crate::vision::{Hiding, Vision};

/// The corners of the box around `o`'s shape ([`Shape::of`]): the tiles it covers are those vision looks at.
pub fn object_box(o: &Object) -> (Vec2, Vec2) {
    Shape::of(o).bounds()
}

/// The domain an object is targeted in: a unit's base domain (`systems/units.md > UnitBase > Domain`); structures
/// stand on the ground.
pub fn object_domain(o: &Object) -> Domain {
    o.def()
        .unit
        .as_ref()
        .map_or(Domain::Ground, |unit| unit.base_def().domain)
}

/// `systems/combat.md > ValidTarget` 3, `systems/combat.md > SingleTargetDamage`: Ground attacks target Ground units
/// and surfaced Underground ones, Air attacks Air units, and Universal attacks any unit.
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

/// Whether a target `distance` space units away (the gap between the two shapes, [`Shape::gap`]) is within the attack's Range and beyond its
/// MinRange (`systems/controls/behaviors.md > AttackingObject` 2). The ElevationModifier changes the Range only.
/// Both bounds are inclusive: a placeholder reading, the design doesn't say.
pub fn within_range(attrs: &AttackAttrs, distance: f32, source: Elevation, target: Elevation) -> bool {
    !inside_min_range(attrs, distance) && distance <= max_range(attrs, source, target)
}

/// Whether a target `distance` space units away (the gap between the two shapes) is inside the attack's MinRange,
/// too close to hit (`systems/combat.md > AttackAttributes`; design change 0182 says what the behaviours do about
/// it). A target exactly at MinRange isn't: [`within_range`] takes both bounds.
pub fn inside_min_range(attrs: &AttackAttrs, distance: f32) -> bool {
    distance < gu(attrs.min_range)
}

/// `systems/combat.md > AttackAttributes > Range Measurement` (design change 0031): whether an attack with `attrs`
/// reaches from the shape `attacker` to the shape `target`: the gap between their edges is within Range and beyond
/// MinRange, with the ElevationModifier between `source` and `target_elevation`. The one range check: the attack
/// phases, [`can_attack_now`] and the behaviours all ask it.
pub fn reaches(
    attrs: &AttackAttrs,
    attacker: &Shape,
    source: Elevation,
    target: &Shape,
    target_elevation: Elevation,
) -> bool {
    within_range(attrs, attacker.gap(target), source, target_elevation)
}

/// `systems/combat.md > Range Measurement`, ground-location targets: whether the attack reaches from the shape
/// `attacker` to `point`, measured from the attacker's edge. For the location-targeted attacks (Attack Ground,
/// TailDisjointed artillery), which no unit has yet.
pub fn reaches_point(attrs: &AttackAttrs, attacker: &Shape, source: Elevation, point: Vec2, ground: Elevation) -> bool {
    within_range(attrs, attacker.distance_to(point), source, ground)
}

/// Whether `attacker` could attack `target` where both stand now: it has an attack and may use it (its Riot Shield
/// isn't up, `riot_shield::allows_attack`), `target` is destructible and
/// not `attacker` itself, the attacker's owner sees it in `vision` (`systems/combat.md > ValidTarget` 2; a unit
/// without an owner sees nothing), the attack's TargetDomain allows it, and it is within Range and beyond MinRange.
/// The attack is the weapon in hand (`super::weapon`: a Legionnaire's form's, design change 0198).
/// Facing is not required: Aiming turns the unit (`systems/combat.md > Aiming`). The behaviours ask the same of
/// their snapshot, part by part (`orders::engage::close_in`: a unit stops its approach once this holds); this is
/// the check on the objects themselves, for tests and tools.
pub fn can_attack_now(map: &Map, vision: &Vision, attacker: &Object, target: &Object) -> bool {
    let Some(attrs) = super::weapon(attacker) else {
        return false;
    };
    riot_shield::allows_attack(attacker)
        && target.id != attacker.id
        && target.hp.is_some()
        && attacker.owner.is_some_and(|owner| vision.sees(owner, target))
        && domain_allows(attrs.target_domain, object_domain(target))
        && reaches(
            attrs,
            &Shape::of(attacker),
            Elevation::of_object(map, attacker),
            &Shape::of(target),
            Elevation::of_object(map, target),
        )
}

/// A destructible object as every attacker sees it this tick.
#[derive(Clone, Debug)]
pub(super) struct TargetView {
    pub id: ObjectId,
    pub pos: Vec2,
    pub facing: f32,
    pub shape: Shape,
    pub domain: Domain,
    pub elevation: Elevation,
    /// `systems/combat.md > DamageModifierOrder` 1: the product of its pre-armor multipliers where it stands.
    pub pre_armor: f32,
    /// `systems/units.md > Unit > PointArmor`, and a structure's own PointArmor (`ObjectDef::armor`).
    pub point_armor: f32,
    /// `systems/units.md > UnitBase > DirectionalArmor`.
    pub directional_armor: bool,
    /// Whom it is hidden from (`vision::hidden`, plan decision T32): no attacker whose owner it is hidden from sees
    /// it, so none targets it (`systems/combat.md > ValidTarget` 2). The hits on the list land all the same
    /// (`damage::apply` doesn't ask: design changes 0218 and 0262).
    pub hiding: Hiding,
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
                let unit = o.def().unit.as_ref();
                TargetView {
                    id: o.id,
                    pos: o.pos,
                    facing: o.facing,
                    shape: Shape::of(o),
                    domain: object_domain(o),
                    elevation: Elevation::of_object(map, o),
                    pre_armor: pre_armor_multiplier(map, o),
                    point_armor: o.def().armor.point,
                    directional_armor: unit.is_some_and(|u| u.base_def().directional_armor),
                    hiding: Hiding::of(o),
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

    /// Design change 0031: Range and MinRange are gaps between edges, on both sides.
    #[test]
    fn range_is_the_gap_between_the_edges() {
        use crate::defs::ObjectKind;
        let attrs = ObjectKind::Peacekeeper.def().unit.as_ref().unwrap().attack.unwrap();
        let level = Elevation::Level(0);
        let pk = Shape::silhouette(ObjectKind::Peacekeeper, Vec2::new(100.0, 100.0), 0.0);
        let guard = |gap: f32| Shape::silhouette(ObjectKind::Guard, Vec2::new(100.0 + 12.0 + gap + 18.0, 100.0), 0.0);
        assert!(reaches(&attrs, &pk, level, &guard(gu(4.0)), level));
        assert!(!reaches(&attrs, &pk, level, &guard(gu(4.0) + 1.0), level));
        assert!(reaches(&attrs, &pk, Elevation::Level(1), &guard(gu(5.0)), level));
        // A ground location is measured from the attacker's edge.
        assert!(reaches_point(
            &attrs,
            &pk,
            level,
            Vec2::new(112.0 + gu(4.0), 100.0),
            level
        ));
        assert!(!reaches_point(
            &attrs,
            &pk,
            level,
            Vec2::new(113.0 + gu(4.0), 100.0),
            level
        ));
        let mut ranged = attrs;
        ranged.min_range = 1.0;
        assert!(!reaches(&ranged, &pk, level, &guard(gu(1.0) - 1.0), level));
        assert!(reaches(&ranged, &pk, level, &guard(gu(1.0)), level));
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
