//! (M5-N5) Buffs (plan decision T75): an area ability's effect on the units it reached as it was cast, for its
//! duration (`factions/cults/structures/shrine.md > Abilities`: "The effect is a **buff applied to units at the moment
//! of casting**: qualifying units inside the area when the ability is cast receive the buff for its duration and keep
//! it even if they leave the area. Units that enter the area afterward do not receive it. Only the caster's own units
//! are affected"). (M5-N11) The Prayer-mobile's Inspire and Suppress are buffs too
//! (`factions/cults/units/prayer_mobile.md > Abilities`: "**A cast** (user, 2026-11-27): it spends energy and buffs the
//! infantry around it at that moment for a set duration"), Suppress's the status of a unit that can't attack
//! (`crate::suppress`).
//!
//! - **The state**: `Object::buffs`, at most one [`Buff`] of each ability, with the tick it ends at. Hashed and saved
//!   only while a unit has one.
//! - **The cast** ([`give`]; `Command::Cast`, applied in the commands phase, plan decision T76: "the units "inside the
//!   area when the ability is cast" are where the last tick left them"): every unit on the map that the ability takes
//!   in (`AbilityDef::affects`: the caster's owner's of the types it affects, or, Suppress's, every unit), whose centre
//!   is in its area, gets its buff for the ability's duration from the tick of the cast; one that has it already has it
//!   restarted ("**Recasting** on units that already have the effect **restarts its duration**; effects of the same
//!   ability **don't stack**"; Inspire's: "Recasting on units that are already inspired restarts the duration"). Units
//!   inside a building or a Tunnel Network aren't in the area, and the caster isn't in its own. The area
//!   ([`in_area`], `AbilityDef::area`): the Shrine's a circle round the point it is cast at ("a unit is affected when
//!   its **centre** is inside"; on its edge too: **Placeholder (not in the design)**); (M5-N11) Inspire's a circle
//!   round its caster ("within **4 grid units** of the vehicle"), on its edge too, as the Shrine's; Suppress's an arc
//!   ahead of its caster's facing (`crate::suppress::in_arc`). Both are where the last tick left the caster.
//! - **The effects** ([`step`]): every tick the buff lasts, its own included, right after the energy pools
//!   (`ARCHITECTURE.md > Tick pipeline`), so that the dead never heal: Health regeneration adds its HP a tick, 0.125
//!   (exact in an `f32`, as the Shields' 0.25), up to the unit's MaxHP with its Augmentations (`Object::max_hp_with`),
//!   30 HP over its 240 ticks; Energy regeneration adds its energy to the unit's own regeneration, 4096 a tick
//!   (`crate::energy::step`, [`energy_bonus`]), up to its MaxEnergy. Then each buff whose last tick this was ends.
//!   Wherever the unit is: one inside a building keeps its buff, which goes on working there. **Placeholder (not in
//!   the design: "keep it even if they leave the area" names no exception)**, as the energy's regeneration. An object
//!   that a command brought into the game in the tick has its effects from the next one (`ARCHITECTURE.md > Tick
//!   pipeline`). (M5-N11) Inspire's and Suppress's work through what reads them, in each tick of the buff: the
//!   unit's top speed (`Object::movement`, [`max_speed_bonus`]) and its weapon (`combat::weapon`, [`inspired`]), and
//!   whether it may start an attack (`crate::suppress::holds_fire`).
//! - **A change of kind** keeps the buffs whose ability affects the new kind (`Sim::change_kind`): a Recruit's Health
//!   regeneration goes on in the Militant it is trained into, and is lost in a Seeker; (M5-N11) a unit's Suppress goes
//!   on in any unit kind (a Peacekeeper raising its Riot Shield keeps it).

use std::collections::BTreeMap;

use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::augmentation::Research;
use crate::defs::{AbilityArea, AreaAbility, AttackAttrs, BuffEffect};
use crate::energy;
use crate::hash::{StateHash, StateHasher};
use crate::ids::{ObjectId, PlayerId};
use crate::network::Held;
use crate::object::Object;
use crate::suppress;
use crate::units::{TICKS_PER_SECOND, Tick, gu};

/// A buff a unit holds (`Object::buffs`): its ability's effect, until `until`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Buff {
    /// The ability whose effect it is.
    pub ability: AreaAbility,
    /// The first tick it no longer works in: the tick of its cast and the ability's duration in ticks
    /// (`AreaAbility::duration`). It works in every tick before it.
    pub until: Tick,
}

impl StateHash for Buff {
    fn hash(&self, h: &mut StateHasher) {
        let Buff { ability, until } = self;
        ability.hash(h);
        until.hash(h);
    }
}

/// Whether `o`'s type casts `ability`: a structure's (`StructureDef::abilities`: the Shrine's), or (M5-N11) a unit's
/// among its spells (`UnitDef::spells`, `SpellDef::Cast`: the Prayer-mobile's).
pub fn casts(o: &Object, ability: AreaAbility) -> bool {
    let def = o.def();
    let structure = def.structure.as_ref().and_then(|s| s.abilities);
    let unit = def.unit.as_ref().map(|unit| unit.spells);
    structure.is_some_and(|abilities| abilities.contains(&ability))
        || unit.is_some_and(|spells| spells.iter().any(|spell| spell.cast() == Some(ability)))
}

/// Whether a unit at `pos` is in the circle of `ability` round `at`: its centre at most the ability's radius from
/// `at` (module docs).
pub fn in_circle(ability: AreaAbility, at: Vec2, pos: Vec2) -> bool {
    let radius = gu(ability.def().radius);
    pos.distance_squared(at) <= radius * radius
}

/// (M5-N11) Whether a unit at `pos` is in the area of `ability` cast from `at`, its caster facing `facing` where it
/// casts round or ahead of itself (module docs): in the circle of the Shrine's round its point, and of Inspire round its
/// caster ([`in_circle`]); in Suppress's arc ahead of its caster ([`suppress::in_arc`]). (M6-GR) In Scan's circle,
/// which buffs no one (`BuffEffect::Sight`).
pub fn in_area(ability: AreaAbility, at: Vec2, facing: f32, pos: Vec2) -> bool {
    let def = ability.def();
    match def.area {
        AbilityArea::AtPoint | AbilityArea::AroundCaster | AbilityArea::AnyPoint => in_circle(ability, at, pos),
        AbilityArea::Ahead { arc } => suppress::in_arc(at, facing, arc, gu(def.radius), pos),
    }
}

/// Whether `ability`, cast by `caster` of `owner`'s, takes in `o` (module docs): another unit than the caster, of a type
/// it affects, and its owner's if it affects only its own (`Affects::own_only`).
fn reaches(ability: AreaAbility, caster: ObjectId, owner: PlayerId, o: &Object) -> bool {
    let affects = ability.def().affects;
    o.id != caster && affects.kinds(o.kind) && (!affects.own_only() || o.owner == Some(owner))
}

/// `owner`'s `caster` casts `ability` from `at`, facing `facing`, in tick `tick` (module docs): its buff, until `tick`
/// and its duration, to every unit in `objects` it takes in whose centre is in its area ([`in_area`]), restarted for
/// one that has it. How many units it reached.
pub(crate) fn give(
    objects: &mut BTreeMap<ObjectId, Object>,
    caster: ObjectId,
    owner: PlayerId,
    ability: AreaAbility,
    (at, facing): (Vec2, f32),
    tick: Tick,
) -> usize {
    let until = tick.saturating_add(ability.duration());
    let reached =
        (objects.values_mut()).filter(|o| reaches(ability, caster, owner, o) && in_area(ability, at, facing, o.pos));
    let mut count = 0;
    for o in reached {
        match o.buffs.iter_mut().find(|buff| buff.ability == ability) {
            // "Recasting on units that already have the effect restarts its duration".
            Some(buff) => buff.until = until,
            None => {
                o.buffs.push(Buff { ability, until });
                o.buffs.sort_by_key(|buff| buff.ability);
            }
        }
        count += 1;
    }
    count
}

/// What `o`'s buffs add to its energy's regeneration in a tick, in `Object::energy`'s units: an Energy regeneration's
/// rate (`crate::energy::step`). 0 for a unit without one.
pub(crate) fn energy_bonus(o: &Object) -> u32 {
    (o.buffs.iter())
        .map(|buff| match buff.ability.def().effect {
            BuffEffect::Energy { per_second } => energy::per_tick(per_second),
            BuffEffect::Health { .. } | BuffEffect::Inspire { .. } | BuffEffect::HoldFire | BuffEffect::Sight => 0,
        })
        .fold(0, u32::saturating_add)
}

/// (M5-N11) What `o`'s buffs add to its MaxSpeed, space units a tick: an Inspire's "+1.5 su/frame MaxSpeed", beside
/// its Augmentations' ("**Stacks with augmentation** (e.g. Improved Metabolism): different sources add up"), which
/// `Object::movement` adds it to. 0 for a unit without one.
pub(crate) fn max_speed_bonus(o: &Object) -> f32 {
    (o.buffs.iter())
        .map(|buff| match buff.ability.def().effect {
            BuffEffect::Inspire { max_speed, .. } => max_speed,
            BuffEffect::Health { .. } | BuffEffect::Energy { .. } | BuffEffect::HoldFire | BuffEffect::Sight => 0.0,
        })
        .fold(0.0, |sum, extra| sum + extra)
}

/// (M5-N11) `attrs`, the weapon `o` holds as its Augmentations make it (`augmentation::augmented`), with its Inspire's
/// faster attack ("Recruits also attack **25% faster** (their attack cycle × 0.75)"): for a unit of a type the buff
/// speeds up (`BuffEffect::Inspire`'s `faster_attacks`, the Recruit), its attack cycle (Aim + Firing + Cooldown +
/// Reload) times the buff's `attack_cycle`, to the nearest tick (a half up), the ticks it saves taken off its
/// FiringDuration: 12 → 9, a Firing of 8 → 5. **Placeholder (not in the design: which phases are shorter)**: Firing is
/// the Recruit's longest phase, its punch, the one Improved Metabolism shortens too, and the others last a tick or two.
/// After the Augmentations, so that the sources add up: an Improved Metabolism Recruit's 10 → 8 (Firing 6 → 4), below
/// the Augmentations' own floor of 2 frames where it takes it there (Level 2's cycle 6 → 5, Firing 2 → 1), never below
/// 0. Only its own attack, its kind's (`own`, the weapon in hand before the Augmentations): a weapon borrowed inside a
/// building fires as its lender's, as the Augmentations leave it (`combat::occupants`). **Placeholder (not in the
/// design)**. Every other unit's weapon is `attrs` unchanged.
pub(crate) fn inspired(o: &Object, own: &AttackAttrs, attrs: AttackAttrs) -> AttackAttrs {
    let faster = (o.buffs.iter()).find_map(|buff| match buff.ability.def().effect {
        BuffEffect::Inspire {
            attack_cycle,
            faster_attacks,
            ..
        } if faster_attacks.contains(&o.kind) => Some(attack_cycle),
        _ => None,
    });
    let theirs = o.def().unit.as_ref().and_then(|unit| unit.attack.as_ref());
    let (Some(factor), true) = (faster, theirs == Some(own)) else {
        return attrs;
    };
    let cycle = attrs.aim_ticks + attrs.firing_ticks + attrs.cooldown_ticks + attrs.reload_ticks;
    let saved = cycle.saturating_sub((cycle as f32 * factor).round() as u32);
    AttackAttrs {
        firing_ticks: attrs.firing_ticks.saturating_sub(saved),
        ..attrs
    }
}

/// One tick of `o`'s buffs, tick `tick`: their HP (module docs), then those whose last tick it is end.
fn step_object(o: &mut Object, research: Research, tick: Tick) {
    for buff in &o.buffs {
        if let BuffEffect::Health { per_second } = buff.ability.def().effect {
            let (Some(hp), Some(max)) = (o.hp, o.max_hp_with(research)) else {
                continue;
            };
            if hp < max {
                o.hp = Some((hp + per_second / TICKS_PER_SECOND as f32).min(max));
            }
        }
    }
    o.buffs.retain(|buff| buff.until > tick.saturating_add(1));
}

/// The buffs' tick `tick` (module docs): every object before `first_new` with a buff, in id order, then every held
/// unit's, at their owners' research (`research`, for their MaxHP).
pub(crate) fn step(
    objects: &mut BTreeMap<ObjectId, Object>,
    held: &mut BTreeMap<ObjectId, Held>,
    research: Research,
    first_new: ObjectId,
    tick: Tick,
) {
    for o in objects.range_mut(..first_new).map(|(_, o)| o) {
        if !o.buffs.is_empty() {
            step_object(o, research, tick);
        }
    }
    for h in held.range_mut(..first_new).map(|(_, h)| h) {
        if !h.object.buffs.is_empty() {
            step_object(&mut h.object, research, tick);
        }
    }
}

#[cfg(test)]
mod tests;
