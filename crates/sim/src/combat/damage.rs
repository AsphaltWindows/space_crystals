//! What a hit does — `systems/combat.md > SingleTargetDamage`, `> DamageModifierOrder`, `> DirectionalArmor`,
//! `> MinimumDamage`, `> Shields > Shield Damage Resolution` — and the tick's list of hits, applied together.
//!
//! **Shields** (plan decision T18; `crate::shields`) are resolved when a hit is applied, not when it is fired: the
//! shot puts the attack's Damage on the list with what the target's armour and position make of it ([`Hit`]), and
//! [`apply`] takes the target's Shields as the hits before it in the list left them ([`resolve`]). So the second of
//! two hits in a tick meets the Shields the first one left.
//!
//! **Shroud** (slice M5-N4b, plan decision T77; `crate::shroud`): before the list is applied, the hits on objects under
//! a cloud from outside it go into the cloud's buffer instead ([`absorb`]), single hits and area shares alike, and the
//! buffers due are released as area hits over their clouds ([`release`]), on the list after the tick's own. Where
//! each attack leaves from, which says whether it comes from under the cloud, its hits' [`Origins`] tell. With no
//! cloud, nothing of this runs.

use std::collections::BTreeMap;

use glam::Vec2;

use super::area::{self, hits_domain};
use super::targeting::Targets;
use crate::defs::TargetDomain;
use crate::events::SimEvent;
use crate::ids::ObjectId;
use crate::map::Map;
use crate::math::{self, PI};
use crate::object::Object;
use crate::riot_shield;
use crate::shroud::Clouds;
use crate::shroud::lattice::Union;
use crate::tuning::combat::{
    DIRECTIONAL_FRONT, DIRECTIONAL_HALF_ARC, DIRECTIONAL_REAR, DIRECTIONAL_SIDE, MINIMUM_DAMAGE,
};
use crate::units::{Tick, TilePos};
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

/// (M5-N4b) Where each attack whose hits are on a tick's list leaves from (`super::phases::Shooter::muzzle`: a unit's
/// centre, a structure's, the footprint's edge for a unit firing from inside a building; a projectile's launch point,
/// `super::Projectile::from`), for the Shroud's absorption ([`absorb`]): "an attack fired by a unit standing **under
/// the cloud** is not absorbed; it hits normally" (`factions/cults/units/disciple.md > Spells > Shroud`), a projectile
/// "judged by its position at launch" (plan decision T77). Kept only while there is a cloud: for each attack, the
/// place on the list of its first hit and its point; its hits run up to the next attack's first.
#[derive(Debug, Default)]
pub(super) struct Origins(Option<Vec<(usize, Vec2)>>);

impl Origins {
    /// Kept if `keep` (while there is a cloud), else nothing.
    pub(super) fn new(keep: bool) -> Self {
        Self(keep.then(Vec::new))
    }

    /// The attack whose hits go on the list from its place `first` on leaves from `at`.
    pub(super) fn mark(&mut self, first: usize, at: Vec2) {
        if let Some(list) = &mut self.0 {
            list.push((first, at));
        }
    }

    /// `other`'s attacks, whose hits now follow on the list from its place `offset` on.
    pub(super) fn append(&mut self, other: Origins, offset: usize) {
        if let (Some(list), Some(other)) = (&mut self.0, other.0) {
            list.extend(other.into_iter().map(|(first, at)| (first + offset, at)));
        }
    }

    /// Where the attack of the hit at `index` on the list leaves from, if kept.
    fn of(&self, index: usize) -> Option<Vec2> {
        let list = self.0.as_ref()?;
        let next = list.partition_point(|&(first, _)| first <= index);
        list.get(next.checked_sub(1)?).map(|&(_, at)| at)
    }
}

/// (M5-N4b) The Shroud's absorption (`crate::shroud`; `factions/cults/units/disciple.md > Spells > Shroud`): each hit of
/// the tick's list `hits` whose target is under a cloud of `clouds`, by its centre ("an object is under a cloud when its
/// **centre** is inside the cloud's area"), goes into that cloud's buffer, its raw damage (the attack's Damage, or an
/// area attack's share, before any modifier), and leaves the list, unless its attack leaves from under the same cloud
/// (`origins`: "an attack fired by a unit standing **under the cloud** is not absorbed; it hits normally. Designer's
/// reading: under the same connected cloud as its target"). "**All anti-ground damage** landing on objects under the
/// cloud, single-target and area alike, is **absorbed and redistributed**", and "It affects every ground object under
/// it ... **not air units**"; "Underground objects are affected (user, 2026-11-26; Q0113) ... protected like ground
/// objects": a target that a Ground area attack hits (`area::hits_domain`), whose hits only an anti-ground attack makes,
/// a Ground or a Universal one. No air unit is in the game yet: slice M5-H1 tests that its hits aren't absorbed.
/// `targets` is phase 5e's snapshot, where the hits found them. Returns the hits absorbed, in their order, whose
/// attackers their targets' owners still learn of (`super::step`).
pub(super) fn absorb(
    clouds: &mut Clouds,
    hits: &mut Vec<Hit>,
    origins: &Origins,
    targets: &Targets,
    tick: Tick,
) -> Vec<Hit> {
    let mut absorbed = Vec::new();
    let mut index = 0;
    hits.retain(|hit| {
        let at = index;
        index += 1;
        let Some(t) = targets.get(hit.target) else {
            return true;
        };
        if !hits_domain(TargetDomain::Ground, t.domain) {
            return true;
        }
        let Some(cloud) = clouds.over(t.pos) else {
            return true;
        };
        if origins.of(at).is_some_and(|from| clouds.covers(cloud, from)) {
            return true;
        }
        clouds.absorb(cloud, hit.damage, tick);
        absorbed.push(*hit);
        false
    });
    absorbed
}

/// (M5-N4b) The Shroud's releases in phase 5e of `tick` (`crate::shroud::Clouds::take_releases`): each buffer due is
/// released "as **one area hit over the whole connected cloud** ... Each ground object under it takes its share by
/// **overlap area** (systems/combat.md > AoEDamage), with armor applied once per release; the share falling on empty
/// cloud area is **lost**", over its cloud's circles (as it was, for a cloud that ended). The hits go on `hits`, one
/// for each object of `targets` that a Ground area attack hits (`area::hits_domain`: "Underground objects are affected
/// ... and take their share of releases, as ground area damage hits them (Q0071)") with a part of its shape under the
/// cloud, under the AoE rules (`area::share`: the share, the effective FullArmor and the minimum by the part under
/// it): "each receiver's share from a release is treated like any damage it receives: its own pre-armor reductions,
/// armor and minimum damage apply as always (systems/combat.md > DamageModifierOrder), but **not directional armor**:
/// a release has no direction (user, 2026-11-26; Q0113)". The areas are counted on the lattice (plan decision T77,
/// `shroud::lattice`). Each release is a [`SimEvent::ShroudReleased`] ahead of its hits, whose attacker is the cloud's
/// first member: none of the releases reveals anyone (`super::step`).
pub(super) fn release(
    clouds: &mut Clouds,
    targets: &Targets,
    tick: Tick,
    hits: &mut Vec<Hit>,
    events: &mut Vec<SimEvent>,
) {
    for cloud in clouds.take_releases(tick) {
        let union = Union::of(&cloud.members);
        let (Some(first), points) = (cloud.members.first(), union.count()) else {
            continue;
        };
        if points == 0 || cloud.buffer <= 0.0 {
            continue;
        }
        events.push(SimEvent::ShroudReleased {
            circles: cloud.circles(),
            damage: cloud.buffer,
        });
        for t in targets.all() {
            if !hits_domain(TargetDomain::Ground, t.domain) {
                continue;
            }
            let (lo, hi) = t.shape.bounds();
            let (inside, total) = union.overlap(lo, hi);
            if inside == 0 || total == 0 {
                continue;
            }
            let (inside, total, under) = (inside as f32, total as f32, points as f32);
            hits.push(area::share(first.id, t, cloud.buffer, inside, total, under, 1.0));
        }
    }
}

#[cfg(test)]
mod tests;
