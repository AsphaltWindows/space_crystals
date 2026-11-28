//! What an attack can target, and from how far — `systems/combat.md > ValidTarget`, `systems/combat.md >
//! AttackAttributes` (Range, MinRange, TargetDomain; `systems/combat.md > Range Measurement`: edge to edge, design
//! change 0031, measured by `shape`) and `core/vision.md > ElevationModifier`.

use std::collections::BTreeMap;

use glam::Vec2;

use super::damage::pre_armor_multiplier;
use crate::defs::{AttackAttrs, AttackType, Domain, FullyConnectedSubtype, ObjectKind, TargetDomain};
use crate::elevation::{self, Elevation};
use crate::ids::{ObjectId, PlayerId};
use crate::map::Map;
use crate::object::Object;
use crate::popup;
use crate::riot_shield;
use crate::shape::Shape;
use crate::units::gu;
use crate::vision::{Hiding, Vision};

/// The corners of the box around `o`'s shape ([`Shape::of`]): the tiles it covers are those vision looks at.
pub fn object_box(o: &Object) -> (Vec2, Vec2) {
    Shape::of(o).bounds()
}

/// The domain an object is targeted in: its type's ([`kind_domain`]), but (M6-GH2) Ground for an aircraft down on an
/// Airfield's slot (`crate::landing::down`; `systems/mechanics/aircraft_landing.md > Assigned base`: "**Landed
/// aircraft are ground targets** [...] Air-only weapons (TargetDomain Air) can't hit it. Once it takes off it is an air
/// target again. **During landing and take-off** [...] it is an **air target**", design change 0428 point 1). Every
/// reader of an object's domain asks it: the scans and ValidTarget ([`can_attack_now`], `orders::world`), the blasts
/// and the projectiles (`super::Targets`), the gate's `Reject::CannotHit` (`Sim::known_domain`) and the right-click
/// (`orders::resolution`). Collision and separation go by the type's domain (`UnitDef::domain`, M6-ST1): a landed
/// aircraft stays on the air layer (`movement::air`).
pub fn object_domain(o: &Object) -> Domain {
    if crate::landing::down(o) {
        return Domain::Ground;
    }
    kind_domain(o.kind)
}

/// (M6-GH2) The domain an object of type `kind` is targeted in, as its type has it: a unit's domain
/// (`systems/units.md > UnitBase > Domain`), (M6-ST1) its type's own over its base's (`UnitDef::domain`, plan decision
/// T94: the Subterranean Transport's above-ground type, a DrillUnit, is Ground); structures stand on the ground. An
/// object's own is [`object_domain`]; a remembered one's is its recorded type's (`Sim::known_domain`).
pub fn kind_domain(kind: ObjectKind) -> Domain {
    kind.def().unit.as_ref().map_or(Domain::Ground, |unit| unit.domain())
}

/// (M6-S6a) Whether a weapon of TargetDomain `attack` (`None`: no weapon) can hit a target in the domain `target`
/// ([`domain_allows`]), the target's domain as the attacker's owner knows it: [`object_domain`]'s of an object it sees,
/// Ground for a structure it remembers (`Sim::known_domain`, `orders::resolution::Aimed::domain`). The one check of
/// design change 0461 (`systems/controls/interface.md > CommandPanel`, "Attack goes to everyone": "a unit whose weapon
/// can't hit the clicked target's domain [...] is treated like a unit that can't attack"): the gate's, which leaves
/// such a unit out of an Attack and refuses one none of its units can carry out (`Reject::CannotHit`), the
/// right-click's (`orders::resolution`), and the client's targeting click, which sends such a unit after the target
/// instead and finds an Attack none of the selection can carry out an invalid target. An object targeted in another
/// domain than its base's (a landed aircraft's) is [`object_domain`]'s to say.
pub fn can_hit(attack: Option<TargetDomain>, target: Domain) -> bool {
    attack.is_some_and(|attack| domain_allows(attack, target))
}

/// `systems/combat.md > ValidTarget` 3, `systems/combat.md > SingleTargetDamage`: Ground attacks target Ground units
/// and surfaced Underground ones, Air attacks Air units, and Universal attacks any unit.
pub fn domain_allows(attack: TargetDomain, target: Domain) -> bool {
    match (attack, target) {
        (TargetDomain::Universal, _) | (TargetDomain::Ground, Domain::Ground) | (TargetDomain::Air, Domain::Air) => {
            true
        }
        // (M6-ST1) A surfaced Underground unit (`systems/units.md > DrillUnit`, its above-ground mode) is a Ground
        // target by its type's own Domain (`UnitDef::domain`, plan decision T94: the surfaced Subterranean Transport),
        // so an Underground one is underground, and no single-target attack targets it.
        (TargetDomain::Ground, Domain::Underground) => false,
        (TargetDomain::Ground, Domain::Air) | (TargetDomain::Air, Domain::Ground | Domain::Underground) => false,
    }
}

/// `systems/combat.md > AttackType > CanTargetGround`, "derived directly from the AttackType": true for TailDisjointed
/// and DoublyDisjointed attacks, which may target a location (`systems/combat.md > AttackTarget`,
/// `systems/combat.md > LocationTarget`). Only units whose
/// attack can are given an Attack Ground (`systems/controls/commands.md > AttackGround`; the gate, and the client's
/// panel).
pub fn can_target_ground(attack_type: AttackType) -> bool {
    match attack_type {
        AttackType::TailDisjointed | AttackType::DoublyDisjointed => true,
        AttackType::FullyConnected(_) | AttackType::HeadDisjointed => false,
    }
}

/// Whether a unit of type `kind` takes Attack orders (`systems/controls/commands.md > Attack`): a unit with a weapon to
/// attack with (`UnitDef::attack`, which no Augmentation adds), unless its Riot Shield is up
/// (`riot_shield::takes_attack_orders`, design change 0202: the Peacekeeper (Shielded)). (M5-N4a) A unit without a
/// weapon, the Disciple ("AttackAttributes - None (placeholder: no weapon [...])", `factions/cults/units/disciple.md >
/// Placeholder stats`), ignores an Attack as a unit whose attack can't target the ground ignores an Attack Ground
/// ([`can_target_ground`]). The gate (`Sim::validate`), the default command (`orders::resolution::Abilities`) and the
/// client's panel and right-click ask it.
pub fn takes_attack_orders(kind: ObjectKind) -> bool {
    let armed = kind.def().unit.as_ref().is_some_and(|unit| unit.attack.is_some());
    armed && riot_shield::takes_attack_orders(kind)
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
/// MinRange (`systems/controls/command_behaviors.md > AttackingObject` 2). The ElevationModifier changes the Range
/// only. Both bounds are inclusive: a placeholder reading, the design doesn't say.
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
/// `attacker` to `point`, measured from the attacker's edge ("Ground-location targets (Attack Ground, TailDisjointed
/// artillery): measured from the attacker's shape to the target point"), with the ElevationModifier between `source`
/// and the ground there.
pub fn reaches_point(attrs: &AttackAttrs, attacker: &Shape, source: Elevation, point: Vec2, ground: Elevation) -> bool {
    within_range(attrs, attacker.distance_to(point), source, ground)
}

/// Whether `attacker` could attack `target` where both stand now: it has an attack and may use it (its Riot Shield
/// isn't up, `riot_shield::allows_attack`; a structure is complete, `Object::is_operational`), `target` is destructible
/// and not `attacker` itself, the attacker's owner sees it in `vision` (`systems/combat.md > ValidTarget` 2; a unit
/// without an owner sees nothing), the attack's TargetDomain allows it, and it is within Range and beyond MinRange.
/// The attack is the weapon in hand (`super::weapon`: a Legionnaire's form's, design change 0198), whose domain and
/// ranges its Augmentations leave as they are (`super::weapon_in_hand`).
/// Facing is not required: Aiming turns the unit (`systems/combat.md > Aiming`). The behaviours ask the same of
/// their snapshot, part by part (`orders::engage::close_in`: a unit stops its approach once this holds); this is
/// the check on the objects themselves, for tests and tools.
pub fn can_attack_now(map: &Map, vision: &Vision, attacker: &Object, target: &Object) -> bool {
    let Some(attrs) = super::weapon_in_hand(attacker) else {
        return false;
    };
    riot_shield::allows_attack(attacker)
        && attacker.is_operational()
        && popup::may_fire(attacker)
        && target.id != attacker.id
        && target.hp.is_some()
        && attacker.owner.is_some_and(|owner| vision.sees(owner, target))
        && domain_allows(attrs.target_domain, object_domain(target))
        && reaches(
            attrs,
            &Shape::on_map(attacker, map),
            Elevation::of_object(map, attacker),
            &Shape::on_map(target, map),
            Elevation::of_object(map, target),
        )
}

/// A destructible object as every attacker sees it this tick.
#[derive(Clone, Debug)]
pub(super) struct TargetView {
    pub id: ObjectId,
    /// (M4-S1) Its owner: a structure's own pick takes only its enemies (`super::structures`).
    pub owner: Option<PlayerId>,
    /// (M4-S1) The domain its own attack threatens (`super::threat`), for a structure's own pick: threatening targets
    /// first (`systems/controls/behaviors.md > BaseAutoTargeting > Target Selection Priority`). (M5-S2b) A building's
    /// units inside add theirs ([`Targets::with_occupants`]).
    pub threat: Option<TargetDomain>,
    pub pos: Vec2,
    pub facing: f32,
    pub shape: Shape,
    pub domain: Domain,
    pub elevation: Elevation,
    /// `systems/combat.md > DamageModifierOrder` 1: the product of its pre-armor multipliers where it stands.
    pub pre_armor: f32,
    /// `systems/units.md > Unit > PointArmor`, and a structure's own PointArmor (`ObjectDef::armor`).
    pub point_armor: f32,
    /// Its FullArmor (`ObjectDef::armor`), which area damage meets (`systems/combat.md > AoEDamage`, `super::area`).
    pub full_armor: f32,
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
                    owner: o.owner,
                    threat: super::threat(o),
                    // (M5-H1) Where it counts as being: a Glider past the map's edge "stays [...] attackable as if
                    // it were at the nearest edge tile" (`Object::map_pos`).
                    pos: o.map_pos(map),
                    facing: o.facing,
                    shape: Shape::on_map(o, map),
                    domain: object_domain(o),
                    elevation: Elevation::of_object(map, o),
                    pre_armor: pre_armor_multiplier(map, o),
                    // Its armour now: a lowered pop-up turret's is its underground one (`Object::armor`, slice M4-S2).
                    point_armor: o.armor().point,
                    full_armor: o.armor().full,
                    directional_armor: unit.is_some_and(|u| u.base_def().directional_armor),
                    hiding: Hiding::of(o),
                }
            })
            .collect();
        Self(views)
    }

    /// (M5-S2b) The snapshot with what the units inside the buildings threaten (`super::Threats`, design change 0377,
    /// answering Q0139): a building that fires through its occupants is a threatening target to a structure's own
    /// pick, its occupants' among them, while one of them could attack the ground (`super::structures::pick`).
    pub(super) fn with_occupants(mut self, inside: &super::Threats) -> Self {
        inside.add_to(&mut self.0, |t| t.id, |t| &mut t.threat);
        self
    }

    pub(super) fn get(&self, id: ObjectId) -> Option<&TargetView> {
        let i = self.0.binary_search_by_key(&id, |t| t.id).ok()?;
        self.0.get(i)
    }

    /// Every destructible object, in id order: what an area attack may hit (`super::area`).
    pub(super) fn all(&self) -> &[TargetView] {
        &self.0
    }

    /// (M5-O3, M5-K) What a single-target shot whose effect lands at the spot `at` hits: an object whose shape covers
    /// the point (`Shape::distance_to` is 0 inside it and on its edge), a unit's Silhouette or a structure's footprint,
    /// that the attack may hit (`hits`: its TargetDomain allows it, and it isn't the shooter), whoever owns it. None:
    /// it misses. A TailDisjointed strike's (`super::phases`; `systems/combat.md > TailDisjointed`: "**Structures
    /// count too** (Q0126): whatever object (unit or structure) covers the locked point is hit; if several do, the
    /// one the shot was aimed at, if it still covers the point, otherwise any one of them (Director's choice)"), and a
    /// single-target DoublyDisjointed projectile's as it lands (`super::projectiles`; `systems/combat.md >
    /// DoublyDisjointed`: "The effect applies to whatever units are at the target location when the projectile
    /// arrives", in the same words). Of several, the one it was aimed at, `aimed`, while it covers the spot, so that a
    /// target that stands still is always hit; else the first in id order.
    pub(super) fn at_the_spot(
        &self,
        at: Vec2,
        aimed: Option<ObjectId>,
        hits: impl Fn(&TargetView) -> bool,
    ) -> Option<&TargetView> {
        let covers = |t: &&TargetView| hits(t) && t.shape.distance_to(at) == 0.0;
        let aimed = aimed.and_then(|id| self.get(id)).filter(covers);
        aimed.or_else(|| self.0.iter().find(covers))
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

    /// `systems/combat.md > AttackType`: "CanTargetGround - derived (false for FullyConnected/HeadDisjointed, true
    /// for TailDisjointed/DoublyDisjointed)".
    #[test]
    fn only_the_attacks_that_can_miss_can_target_the_ground() {
        assert!(!can_target_ground(AttackType::FullyConnected(
            FullyConnectedSubtype::Ranged
        )));
        assert!(!can_target_ground(AttackType::FullyConnected(
            FullyConnectedSubtype::Melee
        )));
        assert!(!can_target_ground(AttackType::HeadDisjointed));
        assert!(can_target_ground(AttackType::TailDisjointed));
        assert!(can_target_ground(AttackType::DoublyDisjointed));
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
