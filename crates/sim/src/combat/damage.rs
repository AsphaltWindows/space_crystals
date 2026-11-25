//! What a hit does — `systems/combat.md > SingleTargetDamage`, `> DamageModifierOrder`, `> DirectionalArmor`,
//! `> MinimumDamage`, `> Shields > Shield Damage Resolution` — and the tick's list of hits, applied together.
//!
//! **Shields** (plan decision T18; `crate::shields`) are resolved when a hit is applied, not when it is fired: the
//! shot puts the attack's Damage on the list with what the target's armour and position make of it ([`Hit`]), and
//! [`apply`] takes the target's Shields as the hits before it in the list left them ([`resolve`]). So the second of
//! two hits in a tick meets the Shields the first one left.

use std::collections::BTreeMap;

use glam::Vec2;

use crate::events::SimEvent;
use crate::ids::ObjectId;
use crate::map::Map;
use crate::math::{self, PI};
use crate::object::Object;
use crate::riot_shield;
use crate::tuning::combat::{
    DIRECTIONAL_FRONT, DIRECTIONAL_HALF_ARC, DIRECTIONAL_REAR, DIRECTIONAL_SIDE, MINIMUM_DAMAGE,
};
use crate::units::TilePos;
use crate::vision::Hiding;

/// One attack's effect on its target, waiting on the tick's list until every unit has acted: the attack's Damage, and
/// the target's modifiers as the shot found them (`systems/combat.md > DamageModifierOrder`), for [`resolve`]. A
/// single-target hit ([`Hit::single`]), or one object's share of an area attack (`super::area::strike`).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Hit {
    pub attacker: ObjectId,
    pub target: ObjectId,
    /// The attack's Damage, before any modifier: what the target's Shields take, "at full value"; of an area attack,
    /// the target's damage share (`systems/combat.md > AoEDamage`: "For AoE, the full value is the unit's damage
    /// share").
    pub damage: f32,
    /// The product of the pre-armor multipliers on hits against the target (step 1, [`pre_armor_multiplier`]).
    pub pre_armor: f32,
    /// The armour step 2 subtracts: the target's PointArmor, or its effective FullArmor for an area attack's share
    /// (`systems/combat.md > AoEDamage`).
    pub armor: f32,
    /// The DirectionalArmor multiplier, from where the attacker stands, or from an area attack's centre (step 3,
    /// [`directional_multiplier`]); 1 for a target without directional armour.
    pub directional: f32,
    /// Step 4's floor: MinimumDamage, or MinimumDamage × the overlap fraction for an area attack's share
    /// (`systems/combat.md > MinimumDamage > AoE`).
    pub minimum: f32,
}

impl Hit {
    /// A single-target hit (`systems/combat.md > SingleTargetDamage`): the whole Damage, against the target's
    /// PointArmor, with the MinimumDamage.
    pub fn single(
        attacker: ObjectId,
        target: ObjectId,
        damage: f32,
        pre_armor: f32,
        point_armor: f32,
        directional: f32,
    ) -> Self {
        Self {
            attacker,
            target,
            damage,
            pre_armor,
            armor: point_armor,
            directional,
            minimum: MINIMUM_DAMAGE,
        }
    }
}

/// What a hit takes from its target ([`resolve`]).
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Taken {
    /// Shields lost.
    pub shields: f32,
    /// HP lost.
    pub hp: f32,
}

/// `systems/combat.md > SingleTargetDamage`: `max((Damage × pre-armor multipliers − PointArmor) × directional
/// multiplier, MinimumDamage)`, in the order of `> DamageModifierOrder`. HP is fractional (`> MinimumDamage`), so
/// nothing is rounded. The HP a hit takes from a target without Shields ([`resolve`]).
pub fn hit_damage(damage: f32, pre_armor: f32, point_armor: f32, directional: f32) -> f32 {
    hp_damage(damage, pre_armor, point_armor, directional, MINIMUM_DAMAGE)
}

/// Steps 1 to 4 of `> DamageModifierOrder` on `damage`, with `armor` as step 2's and `minimum` as step 4's floor.
fn hp_damage(damage: f32, pre_armor: f32, armor: f32, directional: f32, minimum: f32) -> f32 {
    ((damage * pre_armor - armor) * directional).max(minimum)
}

/// What `hit` takes from a target with `shields` (its current Shields; `None` for an object without them, which takes
/// [`hit_damage`]). `> Shields > Shield Damage Resolution`: "Incoming damage is applied to Shields first, at full value
/// — PointArmor and FullArmor do NOT reduce damage against Shields. If damage exceeds current Shields, Shields drop to
/// 0 and the remaining (overflow) damage is applied to HP using the normal SingleTargetDamage [...] armor calculations
/// above [...], with the MinimumDamage capped at the overflow amount [...]. If a unit's current Shields fully absorb
/// the incoming damage, no HP is lost and armor is never consulted." So the overflow goes through the steps of `>
/// DamageModifierOrder` ("Colonists Shields are resolved before all of this [...] only the overflow enters step 1"),
/// with the minimum `min(MinimumDamage, overflow)` (`> MinimumDamage > Shield overflow`: "HP damage = max(overflow -
/// armor, min(minimum, overflow))"). The pre-armor multipliers and the directional one touch the overflow only, never
/// the Shields (`> DamageModifierOrder`: "no modifier of any step touches Shields", design change 0147).
///
/// An area attack's share goes the same way, its share in place of the Damage, its effective FullArmor in place of
/// the PointArmor and its own minimum (`systems/combat.md > AoEDamage`, `super::area`): "the remaining (overflow)
/// damage is applied to HP using the normal SingleTargetDamage / AoEDamage armor calculations above".
pub fn resolve(hit: &Hit, shields: Option<f32>) -> Taken {
    let Some(shields) = shields else {
        return Taken {
            shields: 0.0,
            hp: hp_damage(hit.damage, hit.pre_armor, hit.armor, hit.directional, hit.minimum),
        };
    };
    let absorbed = hit.damage.min(shields).max(0.0);
    let overflow = hit.damage - absorbed;
    let hp = if overflow > 0.0 {
        let minimum = hit.minimum.min(overflow);
        hp_damage(overflow, hit.pre_armor, hit.armor, hit.directional, minimum)
    } else {
        0.0
    };
    Taken { shields: absorbed, hp }
}

/// `systems/combat.md > DamageModifierOrder` 1: the product of the pre-armor multipliers on hits against `o` where
/// it stands, from every direction. Built so far:
/// - the RuggedTerrainDefenseBonus (`systems/units.md > LightInfantry`): the bonus's fraction of the damage is taken
///   off while the unit's centre is on a Rugged tile (a placeholder reading: the design doesn't say which tile counts
///   for a unit standing on several);
/// - the Peacekeeper's raised Riot Shield (`factions/gdo/units/peacekeeper.md > Riot Shield`): "a 50% pre-armor
///   reduction ... multiplying with other pre-armor reductions" (`riot_shield::damage_factor`).
///
/// TODO: the Null Chamber exit buff (`factions/syndicate/structures/null_chamber.md`)
/// multiplies in here.
pub fn pre_armor_multiplier(map: &Map, o: &Object) -> f32 {
    let bonus = o
        .def()
        .unit
        .as_ref()
        .and_then(|unit| unit.rugged_terrain_defense_bonus)
        .filter(|_| {
            map.tile_preset(TilePos::containing(o.pos))
                .is_some_and(|preset| preset.rugged)
        });
    bonus.map_or(1.0, |bonus| 1.0 - bonus) * riot_shield::damage_factor(o)
}

/// `systems/combat.md > DirectionalArmor`: the multiplier on a hit against a unit facing `facing`, from an attack
/// source in direction `to_source` from it. Front within ±45° of the facing, rear within ±45° of directly behind,
/// sides between; a source exactly on a boundary counts as front or rear (a placeholder reading: the design doesn't
/// say), and one right on top of the unit as front.
pub fn directional_multiplier(facing: f32, to_source: Vec2) -> f32 {
    if to_source == Vec2::ZERO {
        return DIRECTIONAL_FRONT;
    }
    let off = math::wrap_angle(math::vector_angle(to_source) - facing).abs();
    if off <= DIRECTIONAL_HALF_ARC {
        DIRECTIONAL_FRONT
    } else if off >= PI - DIRECTIONAL_HALF_ARC {
        DIRECTIONAL_REAR
    } else {
        DIRECTIONAL_SIDE
    }
}

/// Applies the tick's `hits`, in their order (attacker id order), each to the Shields and the HP its target has
/// then ([`resolve`]), then finds the dead: every object a hit brought to 0 HP or less. All hits count, including
/// those on a target an earlier hit in the list has already killed. Returns the dead in id order, for the removal
/// path.
pub(super) fn apply(
    objects: &mut BTreeMap<ObjectId, Object>,
    hits: &[Hit],
    events: &mut Vec<SimEvent>,
) -> Vec<ObjectId> {
    let mut struck = Vec::new();
    for hit in hits {
        let Some(Object {
            hp: Some(hp), shields, ..
        }) = objects.get_mut(&hit.target)
        else {
            continue;
        };
        let taken = resolve(hit, *shields);
        if let Some(shields) = shields {
            *shields -= taken.shields;
        }
        *hp -= taken.hp;
        struck.push(hit.target);
        events.push(SimEvent::Hit {
            attacker: hit.attacker,
            target: hit.target,
            damage: taken.hp,
            shield_damage: taken.shields,
        });
    }
    struck.sort_unstable();
    struck.dedup();
    struck.retain(|id| objects.get(id).and_then(|o| o.hp).is_some_and(|hp| hp <= 0.0));
    for id in &struck {
        if let Some(o) = objects.get(id) {
            events.push(SimEvent::Died {
                id: o.id,
                kind: o.kind,
                owner: o.owner,
                pos: o.pos,
                hiding: Hiding::of(o),
            });
        }
    }
    struck
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_target_damage_follows_the_modifier_order() {
        // Peacekeeper (10) on a Guard (PointArmor 1): 9.
        assert_eq!(hit_damage(10.0, 1.0, 1.0, 1.0), 9.0);
        // Guard (6) on a Peacekeeper (PointArmor 1) on Rugged ground (50% off before armor): 6 × 0.5 − 1 = 2.
        assert_eq!(hit_damage(6.0, 0.5, 1.0, 1.0), 2.0);
        // Armor before the directional multiplier: (10 − 2) × 0.75 = 6, not 10 × 0.75 − 2 = 5.5.
        assert_eq!(hit_damage(10.0, 1.0, 2.0, DIRECTIONAL_FRONT), 6.0);
        assert_eq!(hit_damage(10.0, 1.0, 2.0, DIRECTIONAL_REAR), 10.0);
        // The minimum applies last, whatever the armor.
        assert_eq!(hit_damage(3.0, 1.0, 5.0, DIRECTIONAL_REAR), MINIMUM_DAMAGE);
        assert_eq!(hit_damage(1.0, 0.5, 0.0, DIRECTIONAL_FRONT), MINIMUM_DAMAGE);
        // Fractions stay fractions.
        assert_eq!(hit_damage(5.0, 0.5, 1.0, 1.0), 1.5);
    }

    #[test]
    fn directional_zones() {
        let facing = 0.0; // east
        let deg = |d: f32| math::facing_vector(math::degrees_to_radians(d));
        assert_eq!(directional_multiplier(facing, deg(0.0)), DIRECTIONAL_FRONT);
        assert_eq!(directional_multiplier(facing, deg(44.0)), DIRECTIONAL_FRONT);
        assert_eq!(directional_multiplier(facing, deg(-44.0)), DIRECTIONAL_FRONT);
        assert_eq!(directional_multiplier(facing, deg(46.0)), DIRECTIONAL_SIDE);
        assert_eq!(directional_multiplier(facing, deg(-90.0)), DIRECTIONAL_SIDE);
        assert_eq!(directional_multiplier(facing, deg(134.0)), DIRECTIONAL_SIDE);
        assert_eq!(directional_multiplier(facing, deg(136.0)), DIRECTIONAL_REAR);
        assert_eq!(directional_multiplier(facing, deg(180.0)), DIRECTIONAL_REAR);
        assert_eq!(directional_multiplier(facing, deg(-170.0)), DIRECTIONAL_REAR);
        // Facing south: a source to the north is behind.
        assert_eq!(directional_multiplier(PI / 2.0, Vec2::new(0.0, -5.0)), DIRECTIONAL_REAR);
        assert_eq!(directional_multiplier(PI / 2.0, Vec2::new(0.0, 5.0)), DIRECTIONAL_FRONT);
        assert_eq!(directional_multiplier(facing, Vec2::ZERO), DIRECTIONAL_FRONT);
    }

    /// A hit of `damage` against PointArmor `point_armor`, from no particular direction.
    fn hit(damage: f32, point_armor: f32) -> Hit {
        Hit::single(ObjectId(1), ObjectId(2), damage, 1.0, point_armor, 1.0)
    }

    fn taken(shields: f32, hp: f32) -> Taken {
        Taken { shields, hp }
    }

    /// The plan's acceptance, "The design's cases" (`systems/combat.md > Shields > Shield Damage Resolution`, `>
    /// MinimumDamage > Shield overflow`), exactly.
    #[test]
    fn shields_take_a_hit_first_and_the_overflow_meets_the_armour() {
        // 15 on 10 Shields, armour 0: the Shields reach 0, the HP drops by 5.
        assert_eq!(resolve(&hit(15.0, 0.0), Some(10.0)), taken(10.0, 5.0));
        // 25 on a Robo Portal's 20 Shields, PointArmor 1: 4.
        assert_eq!(resolve(&hit(25.0, 1.0), Some(20.0)), taken(20.0, 4.0));
        // 10.25 on 10 Shields, PointArmor 1: the minimum is capped at the overflow, 0.25.
        assert_eq!(resolve(&hit(10.25, 1.0), Some(10.0)), taken(10.0, 0.25));
        // 8 on 10 Shields: no HP lost, whatever the armour ("armor is never consulted").
        for armour in [0.0, 1.0, 100.0] {
            assert_eq!(resolve(&hit(8.0, armour), Some(10.0)), taken(8.0, 0.0));
        }
        // Exactly the Shields: absorbed ("If damage exceeds current Shields" it overflows).
        assert_eq!(resolve(&hit(10.0, 0.0), Some(10.0)), taken(10.0, 0.0));
        // No Shields left: the usual damage, the minimum whole.
        assert_eq!(resolve(&hit(10.0, 1.0), Some(0.0)), taken(0.0, 9.0));
        assert_eq!(resolve(&hit(3.0, 5.0), Some(0.0)), taken(0.0, MINIMUM_DAMAGE));
        // An object without Shields: `hit_damage`, as ever.
        assert_eq!(resolve(&hit(10.0, 1.0), None), taken(0.0, 9.0));
        assert_eq!(resolve(&hit(3.0, 5.0), None), taken(0.0, MINIMUM_DAMAGE));
    }

    /// `> DamageModifierOrder` (design change 0147): "no modifier of any step touches Shields"; the pre-armor
    /// multipliers and the DirectionalArmor touch the overflow only.
    #[test]
    fn the_multipliers_touch_the_overflow_only() {
        let rear = Hit {
            directional: DIRECTIONAL_REAR,
            ..hit(15.0, 0.0)
        };
        assert_eq!(resolve(&rear, Some(10.0)), taken(10.0, 6.25));
        let halved = Hit {
            pre_armor: 0.5,
            ..hit(15.0, 1.0)
        };
        assert_eq!(resolve(&halved, Some(10.0)), taken(10.0, 1.5));
        // A front hit whose overflow the armour stops: the minimum, capped at the overflow, after the multiplier.
        let front = Hit {
            directional: DIRECTIONAL_FRONT,
            ..hit(10.25, 1.0)
        };
        assert_eq!(resolve(&front, Some(10.0)), taken(10.0, 0.25));
    }

    /// A target of `kind` with `shields`, for [`apply`].
    fn target(kind: crate::defs::ObjectKind, shields: f32) -> Object {
        Object {
            id: ObjectId(2),
            kind,
            owner: None,
            pos: Vec2::ZERO,
            prev_pos: Vec2::ZERO,
            facing: 0.0,
            prev_facing: 0.0,
            hp: kind.def().max_hp,
            shields: Some(shields),
            placement: None,
            resource: None,
            orders: None,
            mover: None,
            attack: None,
            riot_shield: None,
            extractor: None,
            production: None,
            constructor: None,
            worker: None,
            transform: None,
            conversion: None,
            tunnel: crate::tunnel::fresh(kind),
            construction: None,
            decay: None,
            popup: None,
            lineage: None,
        }
    }

    /// Plan decision T18: the hits of a tick are resolved as they are applied, in their order, so the second of two
    /// hits meets the Shields the first left: two hits of 10 on a Drone's 10 Shields cost it its Shields, then 10 of
    /// its 20 HP, where resolving both at the shots would have taken nothing but Shields. A third, of 10 more,
    /// destroys it, and each hit says what it took.
    #[test]
    fn the_second_hit_of_a_tick_meets_the_shields_the_first_left() {
        let mut objects = BTreeMap::from([(ObjectId(2), target(crate::defs::ObjectKind::Drone, 10.0))]);
        let mut events = Vec::new();
        let hits = [hit(10.0, 0.0), hit(10.0, 0.0)];
        assert_eq!(apply(&mut objects, &hits, &mut events), []);
        let drone = &objects[&ObjectId(2)];
        assert_eq!((drone.shields, drone.hp), (Some(0.0), Some(10.0)));
        let took = |events: &[SimEvent]| -> Vec<(f32, f32)> {
            events
                .iter()
                .filter_map(|e| match e {
                    SimEvent::Hit {
                        damage, shield_damage, ..
                    } => Some((*shield_damage, *damage)),
                    _ => None,
                })
                .collect()
        };
        assert_eq!(took(&events), [(10.0, 0.0), (0.0, 10.0)]);
        events.clear();
        assert_eq!(apply(&mut objects, &[hit(10.0, 0.0)], &mut events), [ObjectId(2)]);
        assert_eq!(took(&events), [(0.0, 10.0)]);
        assert!(matches!(events[1], SimEvent::Died { id: ObjectId(2), .. }));
    }
}
