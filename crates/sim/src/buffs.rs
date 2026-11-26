//! (M5-N5) Buffs (plan decision T75): an area ability's effect on the units it reached as it was cast, for its
//! duration (`factions/cults/structures/shrine.md > Abilities`: "The effect is a **buff applied to units at the moment
//! of casting**: qualifying units inside the area when the ability is cast receive the buff for its duration and keep
//! it even if they leave the area. Units that enter the area afterward do not receive it. Only the caster's own units
//! are affected").
//!
//! - **The state**: `Object::buffs`, at most one [`Buff`] of each ability, with the tick it ends at. Hashed and saved
//!   only while a unit has one.
//! - **The cast** ([`give`]; `Command::Cast`, applied in the commands phase, plan decision T76: "the units "inside the
//!   area when the ability is cast" are where the last tick left them"): every unit of the caster's owner on the map,
//!   of a type the ability affects, whose centre is in its circle ("a unit is affected when its **centre** is inside";
//!   on its edge too: **Placeholder (not in the design)**), gets its buff for the ability's duration from the tick of
//!   the cast; one that has it already has it restarted ("**Recasting** on units that already have the effect
//!   **restarts its duration**; effects of the same ability **don't stack**"). Units inside a building or a Tunnel
//!   Network aren't in the area.
//! - **The effects** ([`step`]): every tick the buff lasts, its own included, right after the energy pools
//!   (`ARCHITECTURE.md > Tick pipeline`), so that the dead never heal: Health regeneration adds its HP a tick, 0.125
//!   (exact in an `f32`, as the Shields' 0.25), up to the unit's MaxHP with its Augmentations (`Object::max_hp_with`),
//!   30 HP over its 240 ticks; Energy regeneration adds its energy to the unit's own regeneration, 4096 a tick
//!   (`crate::energy::step`, [`energy_bonus`]), up to its MaxEnergy. Then each buff whose last tick this was ends.
//!   Wherever the unit is: one inside a building keeps its buff, which goes on working there. **Placeholder (not in
//!   the design: "keep it even if they leave the area" names no exception)**, as the energy's regeneration. An object
//!   that a command brought into the game in the tick has its effects from the next one (`ARCHITECTURE.md > Tick
//!   pipeline`).
//! - **A change of kind** keeps the buffs whose ability affects the new kind (`Sim::change_kind`): a Recruit's Health
//!   regeneration goes on in the Militant it is trained into, and is lost in a Seeker.

use std::collections::BTreeMap;

use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::augmentation::Research;
use crate::defs::{AreaAbility, BuffEffect};
use crate::energy;
use crate::hash::{StateHash, StateHasher};
use crate::ids::{ObjectId, PlayerId};
use crate::network::Held;
use crate::object::Object;
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

/// Whether `o`'s type casts `ability` (`StructureDef::abilities`).
pub fn casts(o: &Object, ability: AreaAbility) -> bool {
    let abilities = o.def().structure.as_ref().and_then(|s| s.abilities);
    abilities.is_some_and(|abilities| abilities.contains(&ability))
}

/// Whether a unit at `pos` is in the circle of `ability` cast at `at`: its centre at most the ability's radius from
/// `at` (module docs).
pub fn in_circle(ability: AreaAbility, at: Vec2, pos: Vec2) -> bool {
    let radius = gu(ability.def().radius);
    pos.distance_squared(at) <= radius * radius
}

/// `owner`'s `ability`, cast at `at` in tick `tick` (module docs): its buff, until `tick` and its duration, to every
/// unit of theirs in `objects` of a type it affects whose centre is in its circle, restarted for one that has it. How
/// many units it reached.
pub(crate) fn give(
    objects: &mut BTreeMap<ObjectId, Object>,
    owner: PlayerId,
    ability: AreaAbility,
    at: Vec2,
    tick: Tick,
) -> usize {
    let until = tick.saturating_add(ability.duration());
    let reached = (objects.values_mut())
        .filter(|o| o.owner == Some(owner) && ability.affects(o.kind) && in_circle(ability, at, o.pos));
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
            BuffEffect::Health { .. } => 0,
        })
        .fold(0, u32::saturating_add)
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
