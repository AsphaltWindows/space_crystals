//! Occupants that fire — plan decision T72, slice M5-N2: the units a structure holds fire from it
//! (`StructureDef::occupants_fire`), the Garrison's (`factions/cults/structures/garrison.md`: "Infantry enter the
//! Garrison and fire from inside it [...]. Without units inside, the Garrison does not attack"). They stay held, off the
//! map (`crate::network::Held`), so nothing can target or hit them ("**Protection**: units inside cannot be targeted
//! and take no damage while the Garrison stands"), and the building has no attack of its own.
//!
//! - **When** ([`step`]): in phase 5e, right after the units' and the structures' own attacks: each operational
//!   building whose occupants fire, in id order, its units in the pool's order (`Sim::held_in`: longest inside first).
//! - **Their sequence**: each runs its own `AttackState`, which it keeps while inside and loses as it leaves
//!   (`Sim::put_on_map`: one whose type has no attack has one only while inside, none so far, a Disciple firing
//!   nothing since design change 0450), through the phases every attack shares (`super::phases`), from the building:
//!   its footprint,
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
//!   has it ("a Recruit inside attacks with the Militant's AttackAttributes"; "**Augmentations don't change the
//!   borrowed attack**"), else the unit's own weapon (`super::weapon`, (M5-N1a) with its Augmentations', which leave an
//!   armed unit's as it is), with the Range bonus of its kind ("**Militants and Gunners** inside get **+1 grid unit of
//!   Range**"). "**No spells from inside**": an attack is all they use. (M6-CM) "a **Disciple or illusion inside
//!   doesn't attack**" (design change 0450, replacing "Disciples fire as Militants"): with no stand-in and no weapon
//!   of its own, it has none to fire, so it fires nothing and threatens nothing ([`Threats`]): a Garrison holding only
//!   Disciples and illusions is no threat.
//! - **Revealed** (`core/vision.md > Attacks from outside vision`): a unit inside that hits an object of a player who
//!   doesn't see it reveals the building it fires from ([`building_of`]), where its shots come from.
//! - **Seen as a threat** ([`Threats`]; slice M5-S2b, design change 0377, answering Q0139): "enemies' target scans treat
//!   the Garrison as a threatening target while any occupant could attack the scanner's domain", so the building
//!   threatens what its occupants' attacks, as fired from inside, could attack, and nothing while none is inside.

use std::collections::BTreeMap;

use super::AttackState;
use super::phases::{self, Shooter, Volley};
use super::structures;
use super::targeting::Targets;
use crate::augmentation::Research;
use crate::defs::{AttackAttrs, OccupantFire, TargetDomain};
use crate::events::SimEvent;
use crate::ids::ObjectId;
use crate::map::Map;
use crate::network::{Held, Place};
use crate::object::Object;
use crate::suppress;
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
            Place::Building(_) | Place::Network | Place::Site(_) | Place::Carrier(_) => None,
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

/// The TargetDomain of the attack unit `o` fires from a building whose occupants fire by `fire` ([`weapon`]): the
/// stand-in type's, else its own. Read at no research: an Augmentation changes Damage and FiringDuration only
/// (`augmentation::augmented`), so the domain is the same at every level.
fn domain_inside(fire: &OccupantFire, o: &Object) -> Option<TargetDomain> {
    weapon(fire, o, Research::NONE).map(|attrs| attrs.target_domain)
}

/// (M5-S2b) What the units inside threaten, building by building, as the target scans rank their targets
/// (`systems/controls/behaviors.md > TurretAutonomousScanning > Target Selection Priority` 1, design change 0377,
/// answering Q0139: "**Buildings that fire through their occupants** (e.g. the Cults Garrison [...]) count as
/// threatening while **any occupant's weapon** (as fired from inside, e.g. a Recruit's Militant stand-in) could attack
/// this unit's domain"; `factions/cults/structures/garrison.md > Garrisoning`: "**Seen as a threat**"): for each
/// building whose occupants fire ([`fire_of`]: in operation) with a unit inside that has an attack there
/// ([`domain_inside`]), the TargetDomain all their attacks cover ([`covering`]), in id order. An empty building isn't
/// among them, nor one whose units inside have no attack there: it threatens nobody (0377: "An empty Garrison, or one
/// with no weapon that can reach the scanner's domain, is not a threat").
///
/// Both target snapshots, the behaviours' in phase 5a and phase 5e's, add it to the threat each object's own attack
/// makes (`super::threat`; [`Threats::add_to`]: `orders::world::World::with_occupants`,
/// `super::targeting::Targets::with_occupants`), each as it is taken (units go in after 5a): one pass over the held
/// units, each building looked up once, so a scan still reads one value a target.
#[derive(Debug)]
pub(crate) struct Threats(Vec<(ObjectId, TargetDomain)>);

impl Threats {
    /// What the units of `held` threaten from inside the buildings among `objects` (the type's docs).
    pub(crate) fn of(objects: &BTreeMap<ObjectId, Object>, held: &BTreeMap<ObjectId, Held>) -> Self {
        // Each building looked up once, at its first unit: its fire, and what its units threaten so far. Units that
        // went in together have ids in a row, so the building found last is tried first.
        let mut buildings: Vec<(ObjectId, Option<&'static OccupantFire>, Option<TargetDomain>)> = Vec::new();
        for held in held.values() {
            let Place::Building(id) = held.place else {
                continue;
            };
            let i = match buildings.iter().rposition(|&(building, ..)| building == id) {
                Some(i) => i,
                None => {
                    buildings.push((id, objects.get(&id).and_then(fire_of), None));
                    buildings.len() - 1
                }
            };
            let (_, fire, threat) = &mut buildings[i];
            // Universal covers every domain: the building's other units add nothing to it.
            if *threat == Some(TargetDomain::Universal) {
                continue;
            }
            if let Some(domain) = fire.and_then(|fire| domain_inside(fire, &held.object)) {
                *threat = Some(threat.map_or(domain, |threat| covering(threat, domain)));
            }
        }
        buildings.sort_unstable_by_key(|&(id, ..)| id);
        Self(
            (buildings.into_iter())
                .filter_map(|(id, _, threat)| Some((id, threat?)))
                .collect(),
        )
    }

    /// Adds what the units inside threaten to the threat of each building among a snapshot's `views`, which are in id
    /// order and each threaten what their own attack does (`super::threat`): `id` and `threat` are a view's id and its
    /// threat.
    pub(crate) fn add_to<V>(
        &self,
        views: &mut [V],
        id: impl Fn(&V) -> ObjectId,
        threat: impl Fn(&mut V) -> &mut Option<TargetDomain>,
    ) {
        for &(building, inside) in &self.0 {
            if let Ok(i) = views.binary_search_by_key(&building, &id) {
                let threat = threat(&mut views[i]);
                *threat = Some(threat.map_or(inside, |own| covering(own, inside)));
            }
        }
    }

    /// What the units inside `building` threaten, if it is one of them.
    #[cfg(test)]
    pub(crate) fn get(&self, building: ObjectId) -> Option<TargetDomain> {
        let i = self.0.binary_search_by_key(&building, |&(id, _)| id).ok()?;
        Some(self.0[i].1)
    }
}

/// The TargetDomain that covers both `a` and `b`: theirs if they are the same, else Universal, which covers either.
/// Technical: Ground with Air comes out Universal, which also covers the Underground that neither does
/// (`super::domain_allows`). No building takes in both a Ground-only and an Air-only attack (the Garrison's are the
/// Militant's, Universal, and the Gunner's, Ground), so no scan tells the two apart.
fn covering(a: TargetDomain, b: TargetDomain) -> TargetDomain {
    if a == b { a } else { TargetDomain::Universal }
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
        // (M5-N11) One suppressed before it went in holds its fire in here too (`crate::suppress`).
        let holds_fire = suppress::holds_fire(&held.object);
        // A unit whose type has no attack would fire its stand-in's with a sequence of its own while inside, which it
        // loses as it leaves (`Sim::put_on_map`): none so far, the Disciple firing nothing (design change 0450).
        let Object { attack, facing, .. } = &mut held.object;
        let state = attack.get_or_insert_with(AttackState::default);
        let sight = building.owner.and_then(|owner| vision.player(owner));
        let shooter = Shooter::occupant(map, building, unit, &attrs, sight).holding_fire(holds_fire);
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
        Place::Network | Place::Site(_) | Place::Carrier(_) => None,
    }
}

#[cfg(test)]
mod tests;
