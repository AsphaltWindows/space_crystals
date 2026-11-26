//! Occupants that fire — plan decision T72, slice M5-N2: the units a structure holds fire from it
//! (`StructureDef::occupants_fire`), the Garrison's (`factions/cults/structures/garrison.md`: "Infantry enter the
//! Garrison and fire from inside it [...]. Without units inside, the Garrison does not attack"). They stay held, off the
//! map (`crate::network::Held`), so nothing can target or hit them ("**Protection**: units inside cannot be targeted
//! and take no damage while the Garrison stands"), and the building has no attack of its own.
//!
//! - **When** ([`step`]): in phase 5e, right after the units' and the structures' own attacks: each operational
//!   building whose occupants fire, in id order, its units in the pool's order (`Sim::held_in`: longest inside first).
//! - **Their sequence**: each runs its own `AttackState`, which it keeps while inside and loses as it leaves
//!   (`Sim::put_on_map`; a Disciple, whose type has no attack, has one only while inside), through the phases every
//!   attack shares (`super::phases`), from the building: its footprint,
//!   its elevation, its owner and its owner's vision (`super::phases::Shooter::occupant`), firing in any direction, as a
//!   structure does. Range and MinRange are measured from the footprint's edge ("**Range** is measured **from the
//!   Garrison's edge** (its footprint), including a Gunner's MinRange"), and the shots leave from the footprint's edge
//!   nearest their target (`super::phases::Shooter::muzzle`).
//! - **Targets** ("each unit inside **picks its own target** by the normal targeting rules (as the Laser Turret:
//!   factions/syndicate/structures/laser_turret.md > Targeting)"): the structures' own pick (`super::structures::pick`,
//!   plan decision T49): it keeps its target while valid and in range, else the threatening first, then the closest,
//!   then the lowest id. Among "enemies the player sees within their (bonus) range": its owner's vision, which the
//!   building's SightRange is part of, and to which the units inside add nothing, being off the map ("Units inside see
//!   with the Garrison's SightRange").
//! - **Weapons** ([`weapon`]): a stand-in's attack for the kinds that fire another kind's, as the stand-in type's def
//!   has it ("a Recruit or Disciple inside attacks with the Militant's AttackAttributes"; "**Augmentations don't change
//!   the borrowed attack**"), else the unit's own weapon (`super::weapon`, (M5-N1a) with its Augmentations', which leave
//!   an armed unit's as it is), with the Range bonus of its kind ("**Militants
//!   and Gunners** inside get **+1 grid unit of Range**"). "**No spells from inside**": an attack is all they use.
//! - **Revealed** (`core/vision.md > Attacks from outside vision`): a unit inside that hits an object of a player who
//!   doesn't see it reveals the building it fires from ([`building_of`]), where its shots come from.

use std::collections::BTreeMap;

use super::AttackState;
use super::phases::{self, Shooter, Volley};
use super::structures;
use super::targeting::Targets;
use crate::augmentation::Research;
use crate::defs::{AttackAttrs, OccupantFire};
use crate::events::SimEvent;
use crate::ids::ObjectId;
use crate::map::Map;
use crate::network::{Held, Place};
use crate::object::Object;
use crate::units::Tick;
use crate::vision::Vision;

/// How the occupants of `building` fire: its type's [`OccupantFire`] while it is in operation (`Object::is_operational`:
/// a site does nothing but cancel), `None` for anything else.
fn fire_of(building: &Object) -> Option<&'static OccupantFire> {
    let fire = building.def().structure.as_ref()?.occupants_fire.as_ref()?;
    building.is_operational().then_some(fire)
}

/// The units that fire from inside the buildings of the map's first objects (those before `first_new`, as for every
/// attacker of phase 5e), building by building in id order, each building's in the pool's order: by `Held::since`,
/// ties by id. None: phase 5e has nothing to do for them ([`step`]).
pub(super) fn firing(
    objects: &BTreeMap<ObjectId, Object>,
    held: &BTreeMap<ObjectId, Held>,
    first_new: ObjectId,
) -> Vec<ObjectId> {
    let mut inside: Vec<(ObjectId, Tick, ObjectId)> = (held.values())
        .filter_map(|held| match held.place {
            Place::Building(building) if building < first_new => Some((building, held.since, held.object.id)),
            Place::Building(_) | Place::Network | Place::Site(_) => None,
        })
        .filter(|&(building, ..)| objects.get(&building).and_then(fire_of).is_some())
        .collect();
    inside.sort_unstable();
    inside.into_iter().map(|(.., unit)| unit).collect()
}

/// The attack unit `o` fires from a building whose occupants fire by `fire` (module docs): the stand-in type's, as its
/// def has it, for a kind that fires another kind's ([`OccupantFire::weapon_kind`]), which no Augmentation changes,
/// else its own weapon (`super::weapon`, the form's, (M5-N1a) with its Augmentations' at its owner's levels in
/// `research`), its Range raised by its kind's bonus ([`OccupantFire::range_bonus`]). `None` for a unit without one.
pub(super) fn weapon(fire: &OccupantFire, o: &Object, research: Research) -> Option<AttackAttrs> {
    let mut attrs = if fire.fires_own(o.kind) {
        super::weapon(o, research)?
    } else {
        *super::weapon_of(fire.weapon_kind(o.kind))?
    };
    attrs.range += fire.range_bonus(o.kind);
    Some(attrs)
}

/// Phase 5e's fire of the units `firing` ([`firing`]) from the buildings they are in (module docs), after every
/// object's own attack: each picks its target ([`structures::pick`]) and runs its sequence ([`phases::run`]) from its
/// building, its hits and its projectiles going to `volley` with the others of the tick. (M5-N1a) Their weapons are as
/// their Augmentations make them at their owners' levels in `research` ([`weapon`]).
#[expect(clippy::too_many_arguments, reason = "phase 5e's state, as `super::step` holds it")]
pub(super) fn step(
    firing: &[ObjectId],
    objects: &BTreeMap<ObjectId, Object>,
    held: &mut BTreeMap<ObjectId, Held>,
    map: &Map,
    vision: &Vision,
    targets: &Targets,
    auto_targeting: bool,
    research: Research,
    volley: &mut Volley,
    events: &mut Vec<SimEvent>,
) {
    for &unit in firing {
        let Some(held) = held.get_mut(&unit) else {
            continue;
        };
        let Place::Building(building) = held.place else {
            continue;
        };
        let Some((building, fire)) = objects.get(&building).and_then(|b| Some((b, fire_of(b)?))) else {
            continue;
        };
        let Some(attrs) = weapon(fire, &held.object, research) else {
            continue;
        };
        // A unit whose type has no attack (the Disciple) fires its stand-in's with a sequence of its own while inside,
        // which it loses as it leaves (`Sim::put_on_map`).
        let Object { attack, facing, .. } = &mut held.object;
        let state = attack.get_or_insert_with(AttackState::default);
        let sight = building.owner.and_then(|owner| vision.player(owner));
        let shooter = Shooter::occupant(map, building, unit, &attrs, sight);
        structures::pick(state, &shooter, targets, auto_targeting);
        // A structure fires in any direction: the unit's facing, which nothing reads while it is inside, stays.
        phases::run(state, facing, &shooter, targets, volley, events);
    }
}

/// The building unit `id` fires from, if it is held in one (module docs: what a hit of its reveals).
pub(super) fn building_of<'o>(
    objects: &'o BTreeMap<ObjectId, Object>,
    held: &BTreeMap<ObjectId, Held>,
    id: ObjectId,
) -> Option<&'o Object> {
    match held.get(&id)?.place {
        Place::Building(building) => objects.get(&building),
        Place::Network | Place::Site(_) => None,
    }
}

#[cfg(test)]
mod tests;
