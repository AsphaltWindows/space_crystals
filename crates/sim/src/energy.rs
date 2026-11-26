//! Energy (plan decision T75, slice M5-N4a): the pool an object's spells spend (`factions/cults/units/disciple.md >
//! Energy`: "The Disciple has its own energy pool, which it spends on its spells. It regenerates naturally over time").
//!
//! - **The def**: `ObjectDef::energy`, an [`EnergyDef`] in points, as the design gives them: the maximum, the start and
//!   how it regenerates ([`EnergyRegen`]), so many points a second (the Disciple's "Energy - MaxEnergy 200; starts at
//!   50; regenerates 0.5 per second", `factions/cults/units/disciple.md > Placeholder stats`) or so many a second for
//!   each unit inside the structure (the Shrine's, slice M5-N5). One field for units and structures, as
//!   `max_shields`; nothing else in the sim names a type that has one.
//! - **The state**: `Object::energy`, a whole number of [`ONE`]s a point (`economy::RATIO_ONE`s: `ARCHITECTURE.md >
//!   Determinism`, "Money and progress are integers"), so that every rate of the design adds up exactly and a
//!   threshold compares exactly: 0.5 a second is 2048 a tick ([`per_tick`]), and 50 to 200 takes 4800 ticks, 300 s, to
//!   the tick. An object comes into the game with its type's start ([`start`]; `Sim::new_object`), and so does one
//!   that becomes of the type (`Sim::change_kind`: the Recruit a Cathedral trains into a Disciple, `Sim::train`); one
//!   that changes from a type with a pool into another keeps its energy, at most the new maximum, and into a type
//!   without one has none. Hashed and saved only while set.
//! - **Regeneration** (`step`): every tick, right after the Shields' (`ARCHITECTURE.md > Tick pipeline`): after 5e's
//!   dead have left the game, so that they never regenerate, and before the economy, whose new units start at their
//!   start in their first tick. Up to the maximum, whoever owns the object, as the Shields. Wherever the unit is: a
//!   unit inside a building (a Disciple in a Garrison) regenerates too. Placeholder (not in the design: "It
//!   regenerates naturally over time" names no exception). An object that comes into the game in a tick regenerates
//!   from the next, wherever in the tick it came; one that gets its pool by a change of kind regenerates from the next
//!   energy step: the tick after for a unit the economy trains, that tick for a unit trained as its building is
//!   destroyed (the evacuation, in 5e's deaths or a debug removal), which ends it a tick's regeneration above its start.
//! - **Spending** is the spells' (Shroud's drain, slice M5-N4b; the Shrine's casts, slice M5-N5), each from its own
//!   order or command, through [`spend`]: all of a cost or nothing, never below 0. What a spell needs to start, [`has`]
//!   tells (`defs::SpellDef::needs`).

use std::collections::BTreeMap;

use crate::defs::{EnergyDef, EnergyRegen, ObjectKind};
use crate::economy::RATIO_ONE;
use crate::ids::ObjectId;
use crate::network::{Held, Place};
use crate::object::Object;
use crate::units::TICKS_PER_SECOND;

/// A point of energy, in `Object::energy`'s units.
pub const ONE: u32 = RATIO_ONE;

/// `points` of energy in `Object::energy`'s units, to the nearest: exact for every number of the design (a whole number
/// of points, or a rate such as 0.5 or 0.25 a second, [`per_tick`]); none below 0.
pub fn units(points: f32) -> u32 {
    (points * ONE as f32).round() as u32
}

/// `units` of `Object::energy` in points: for what shows it (the client's energy bar and InfoPanel).
pub fn points(units: u32) -> f32 {
    units as f32 / ONE as f32
}

/// What `per_second` points a second come to in a tick, in `Object::energy`'s units (plan decision T75: "per second ×
/// `RATIO_ONE` / 16"): 0.5 a second is 2048, 0.25 is 1024.
pub fn per_tick(per_second: f32) -> u32 {
    units(per_second / TICKS_PER_SECOND as f32)
}

/// What an object of `kind` has as it comes into the game, or becomes of the type: its type's start; none for a type
/// without an energy pool.
pub fn start(kind: ObjectKind) -> Option<u32> {
    kind.def().energy.map(|def| units(def.start))
}

/// The most energy an object of `kind` may have, its type's MaxEnergy; none for a type without an energy pool.
pub fn max(kind: ObjectKind) -> Option<u32> {
    kind.def().energy.map(|def| units(def.max))
}

/// An object's energy and its maximum, in points (`None` without an energy pool): what the client shows.
pub fn of(o: &Object) -> Option<(f32, f32)> {
    Some((points(o.energy?), points(max(o.kind)?)))
}

/// The energy of an object that changes from its kind into `kind`, with `energy` before (`Sim::change_kind`, module
/// docs): its energy kept at most the new maximum, or the new type's start if it had none; none if the new type has no
/// energy pool.
pub(crate) fn after_change(energy: Option<u32>, kind: ObjectKind) -> Option<u32> {
    let EnergyDef { max, start, .. } = kind.def().energy?;
    Some(energy.map_or(units(start), |energy| energy.min(units(max))))
}

/// What energy at `energy` comes to after a tick that adds `gain`, with a maximum of `max`: never above it.
pub fn regenerate(energy: u32, gain: u32, max: u32) -> u32 {
    energy.saturating_add(gain).min(max)
}

/// (M5-N4b) Whether `o` has at least `points` of energy ([`enough`]): what a spell needs to start
/// (`defs::SpellDef::needs`; the gate's `Reject::NotEnoughEnergy`). An object without an energy pool has none.
pub fn has(o: &Object, points: f32) -> bool {
    o.energy.is_some_and(|energy| enough(energy, points))
}

/// (M5-N4b) Whether `energy` (`Object::energy`'s units) is at least `points`: the one check of what a spell needs to
/// start ([`has`]; Shroud's start, `orders::shroud`).
pub fn enough(energy: u32, points: f32) -> bool {
    energy >= units(points)
}

/// (M5-N4b) Spends `cost` of `energy` (`Object::energy`'s units), if it has that much, and says whether it did: a
/// spell's spending, all of it or nothing (Shroud's drain, a tick's, `orders::shroud`). The one way energy goes down.
pub fn spend(energy: &mut u32, cost: u32) -> bool {
    match energy.checked_sub(cost) {
        Some(left) => {
            *energy = left;
            true
        }
        None => false,
    }
}

/// How many units each building holds (`Place::Building`), whether reserved for an order, on their way out or not.
fn inside(held: &BTreeMap<ObjectId, Held>) -> BTreeMap<ObjectId, u32> {
    let mut counts = BTreeMap::new();
    for h in held.values() {
        if let Place::Building(building) = h.place {
            *counts.entry(building).or_insert(0) += 1;
        }
    }
    counts
}

/// One tick of `o`'s regeneration, if it has energy: its type's rate, by the units inside it for a pool that
/// regenerates by them (`inside`, asked only then).
fn regenerate_object(o: &mut Object, inside: impl FnOnce(ObjectId) -> u32) {
    let (Some(energy), Some(EnergyDef { max, regen, .. })) = (o.energy, o.def().energy) else {
        return;
    };
    let gain = match regen {
        EnergyRegen::PerSecond(rate) => per_tick(rate),
        EnergyRegen::PerUnitInside(rate) => per_tick(rate).saturating_mul(inside(o.id)),
    };
    o.energy = Some(regenerate(energy, gain, units(max)));
}

/// The regeneration of a tick (module docs): every object before `first_new` with energy, in id order, then every held
/// unit's, which holds no one (an object created in this tick has its start, and regenerates from the next one,
/// `ARCHITECTURE.md > Tick pipeline`).
pub(crate) fn step(objects: &mut BTreeMap<ObjectId, Object>, held: &mut BTreeMap<ObjectId, Held>, first_new: ObjectId) {
    // The units inside each building, counted once, for the first pool that regenerates by them.
    let mut counts = None;
    for o in objects.range_mut(..first_new).map(|(_, o)| o) {
        if o.energy.is_some() {
            regenerate_object(o, |id| {
                let counts = counts.get_or_insert_with(|| inside(held));
                counts.get(&id).copied().unwrap_or(0)
            });
        }
    }
    for h in held.range_mut(..first_new).map(|(_, h)| h) {
        if h.object.energy.is_some() {
            regenerate_object(&mut h.object, |_| 0);
        }
    }
}

#[cfg(test)]
mod tests;
