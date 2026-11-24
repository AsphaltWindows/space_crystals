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
use crate::units::TilePos;

/// `systems/combat.md > MinimumDamage > MinimumDamage`: every single-target hit deals at least this much.
pub const MINIMUM_DAMAGE: f32 = 0.5;

/// `systems/combat.md > DirectionalArmor`: the half-width of the front and rear arcs, radians (±45°, 90° arcs).
pub const DIRECTIONAL_HALF_ARC: f32 = PI / 4.0;
/// `systems/combat.md > DirectionalArmor`: the multiplier of a hit from the front arc.
pub const DIRECTIONAL_FRONT: f32 = 0.75;
/// `systems/combat.md > DirectionalArmor`: from either side arc.
pub const DIRECTIONAL_SIDE: f32 = 1.0;
/// `systems/combat.md > DirectionalArmor`: from the rear arc.
pub const DIRECTIONAL_REAR: f32 = 1.25;

/// One attack's effect on its target, waiting on the tick's list until every unit has acted: the attack's Damage, and
/// the target's modifiers as the shot found them (`systems/combat.md > DamageModifierOrder`), for [`resolve`].
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Hit {
    pub attacker: ObjectId,
    pub target: ObjectId,
    /// The attack's Damage, before any modifier: what the target's Shields take, "at full value".
    pub damage: f32,
    /// The product of the pre-armor multipliers on hits against the target (step 1, [`pre_armor_multiplier`]).
    pub pre_armor: f32,
    /// The target's PointArmor (step 2).
    pub point_armor: f32,
    /// The DirectionalArmor multiplier, from where the attacker stands (step 3, [`directional_multiplier`]); 1 for a
    /// target without directional armour.
    pub directional: f32,
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

/// Steps 1 to 4 of `> DamageModifierOrder` on `damage`, with `minimum` as step 4's floor.
fn hp_damage(damage: f32, pre_armor: f32, point_armor: f32, directional: f32, minimum: f32) -> f32 {
    ((damage * pre_armor - point_armor) * directional).max(minimum)
}

/// What `hit` takes from a target with `shields` (its current Shields; `None` for an object without them, which
/// takes [`hit_damage`]). `> Shields > Shield Damage Resolution`: "Incoming damage is applied to Shields first, at
/// full value — PointArmor and FullArmor do NOT reduce damage against Shields. If damage exceeds current Shields,
/// Shields drop to 0 and the remaining (overflow) damage is applied to HP using the normal SingleTargetDamage [...]
/// armor calculations above [...], with the MinimumDamage capped at the overflow amount [...]. If a unit's current
/// Shields fully absorb the incoming damage, no HP is lost and armor is never consulted." So the overflow goes
/// through the steps of `> DamageModifierOrder` ("Colonists Shields are resolved before all of this [...] only the
/// overflow enters step 1"), with the minimum `min(MinimumDamage, overflow)` (`> MinimumDamage > Shield overflow`:
/// "HP damage = max(overflow - armor, min(minimum, overflow))"). The pre-armor multipliers and the directional one
/// touch the overflow only, never the Shields (a placeholder reading of "at full value", question Q0042 point 3).
pub fn resolve(hit: &Hit, shields: Option<f32>) -> Taken {
    let Some(shields) = shields else {
        return Taken {
            shields: 0.0,
            hp: hit_damage(hit.damage, hit.pre_armor, hit.point_armor, hit.directional),
        };
    };
    let absorbed = hit.damage.min(shields).max(0.0);
    let overflow = hit.damage - absorbed;
    let hp = if overflow > 0.0 {
        let minimum = MINIMUM_DAMAGE.min(overflow);
        hp_damage(overflow, hit.pre_armor, hit.point_armor, hit.directional, minimum)
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
/// - the Peacekeeper's raised Riot Shield (`factions/gdo/units.md > Peacekeeper > Riot Shield`): "a 50% pre-armor
///   reduction ... multiplying with other pre-armor reductions" (`riot_shield::damage_factor`).
///
/// TODO: the Null Chamber exit buff (`factions/syndicate/structures.md > Tunnel Expansions > Null Chamber`)
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
}
