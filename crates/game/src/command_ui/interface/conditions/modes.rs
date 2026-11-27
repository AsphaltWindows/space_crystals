//! The selected units' modes ([`Modes`]): which of them take no Attack, carry a load, aren't in operation yet, or
//! (M6-GD) hold still for their drones, (M6-GH1) which are aircraft, with their bases, and which of those have no
//! ammo left, (M6-GA) which deploy, into which type, and which can't move, and (M6-CM) which are illusions.

use std::collections::{BTreeMap, BTreeSet};

use sc_sim::{ObjectId, ObjectKind, ShieldMode, Sim, aircraft, combat, mirage, riot_shield, tether, vehicle_deploy};

use crate::selection::Selection;

/// What the selected units' modes let them do now: which of them take no Attack (`factions/gdo/units/peacekeeper.md >
/// Riot Shield > During the toggle`, design change 0202: "which orders a unit accepts follows its new type from the
/// press": a Peacekeeper (Shielded), whose shield is up, on its way up or waiting for its shot to go up, but not a
/// Peacekeeper whose shield is on its way down, which takes an Attack and holds it until its shield is down; (M5-N4a)
/// and a unit without a weapon, a Disciple), by the type it groups as in the selection (the type from the press,
/// `crate::selection::ControlState::toggle`) and the sim's own query (`sc_sim::combat::takes_attack_orders`, so that
/// the panel and the right-click never judge a type otherwise than the gate), which of the workers carry a load, which
/// a right-click on a drop-off reads (`factions/colonists/units/drone.md > Right-Click Resolution`: "Own Egress Node
/// (carrying resources)") and the Agent's Drop Off, and which of the selected objects aren't in operation yet
/// (`sc_sim::Object::is_operational`: one that transforms, or a site under construction), and (M6-GD) which of the
/// units hold still for their drones (`sc_sim::tether::holds`: "The Drone Operator **cannot move** while it has a drone
/// active"), and (M6-GH1) which are aircraft, with their bases (`sc_sim::aircraft`), and which of these have no ammo
/// left, taking no Attack then (`sc_sim::aircraft::armed`, the gate's `sc_sim::Reject::OutOfAmmo`). Which way a shield
/// goes is its unit's type (design change 0173), whose panel offers the command that turns it. (M6-GA) Which of the
/// units deploy, and into which type (`sc_sim::vehicle_deploy::under_way`): one takes the orders the type it becomes
/// will carry out (`factions/gdo/units/artillery_rig.md > Modes`: "the **last order given** [...] is **held**, and it
/// is the first thing the unit tries once deployment finishes"), the gate's rule (`sc_sim::vehicle_deploy`), so it is
/// judged as that type ([`Modes::becomes`]); and which of the others can't move (`sc_sim::vehicle_deploy::immobile`: a
/// deployed Rig, "**cannot move**"), the gate refusing them every order that would move them (`sc_sim::Reject::
/// CannotMove`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Modes {
    /// The units whose type takes no Attack: a Peacekeeper (Shielded), or a unit without a weapon; (M6-GH1) and the
    /// aircraft out of ammo ([`Modes::empty`]).
    held: BTreeSet<ObjectId>,
    /// (M5-N4a) Of them, those whose Riot Shield holds them back (`sc_sim::riot_shield::takes_attack_orders`): the
    /// Peacekeepers (Shielded), which would take an Attack with their shields lowered (`super::WhyUnavailable::ShieldsUp`).
    shielded: BTreeSet<ObjectId>,
    loaded: BTreeSet<ObjectId>,
    transforming: BTreeSet<ObjectId>,
    /// (M6-GD) The Drone Operators whose drones are out.
    tethered: BTreeSet<ObjectId>,
    /// (M6-GH1) The aircraft, each with the Airfield it is assigned to, if any (`sc_sim::aircraft::base_of`).
    bases: BTreeMap<ObjectId, Option<ObjectId>>,
    /// (M6-GH1) Of them, those with no ammo left.
    empty: BTreeSet<ObjectId>,
    /// (M6-GA) The units that deploy, each with the type it becomes.
    deploying: BTreeMap<ObjectId, ObjectKind>,
    /// (M6-GA) The units of a type that can't move, but those that deploy.
    immobile: BTreeSet<ObjectId>,
    /// (M6-CM) The illusions (`sc_sim::mirage`), which a building that refuses illusions doesn't take in: the Altar. Read
    /// for a selection its player commands, whose own they are.
    illusions: BTreeSet<ObjectId>,
}

impl Modes {
    /// The modes of `selection`'s objects in `sim`.
    pub fn of(sim: &Sim, selection: &Selection) -> Self {
        let empty: BTreeSet<ObjectId> = (selection.ids())
            .filter(|&id| sim.object(id).is_some_and(|o| !aircraft::armed(o)))
            .collect();
        let deploying: BTreeMap<ObjectId, ObjectKind> = (selection.ids())
            .filter_map(|id| Some((id, vehicle_deploy::under_way(sim.object(id)?)?.0)))
            .collect();
        let becomes = |id: ObjectId, kind: ObjectKind| deploying.get(&id).copied().unwrap_or(kind);
        Self {
            held: (selection.members())
                .map(|(id, kind)| (id, becomes(id, kind)))
                .filter(|&(_, kind)| kind.def().unit.is_some() && !combat::takes_attack_orders(kind))
                .map(|(id, _)| id)
                .chain(empty.iter().copied())
                .collect(),
            shielded: (selection.members())
                .filter(|&(_, kind)| !riot_shield::takes_attack_orders(kind))
                .map(|(id, _)| id)
                .collect(),
            loaded: (selection.ids())
                .filter(|&id| (sim.object(id)).is_some_and(|o| o.worker.is_some_and(|worker| worker.load > 0)))
                .collect(),
            transforming: selection
                .ids()
                .filter(|&id| sim.object(id).is_some_and(|o| !o.is_operational()))
                .collect(),
            illusions: (selection.ids())
                .filter(|&id| sim.object(id).is_some_and(mirage::is_illusion))
                .collect(),
            tethered: (selection.ids())
                .filter(|&id| sim.object(id).is_some_and(tether::holds))
                .collect(),
            bases: (selection.ids())
                .filter_map(|id| sim.object(id).filter(|o| o.aircraft.is_some()))
                .map(|o| (o.id, aircraft::base_of(o)))
                .collect(),
            empty,
            immobile: (selection.members())
                .filter(|&(id, kind)| !deploying.contains_key(&id) && vehicle_deploy::immobile(kind))
                .map(|(id, _)| id)
                .collect(),
            deploying,
        }
    }

    /// (M6-GA) The type `id`, of type `kind`, is judged as: the type it becomes while it deploys, `kind` otherwise.
    pub fn becomes(&self, id: ObjectId, kind: ObjectKind) -> ObjectKind {
        self.deploying.get(&id).copied().unwrap_or(kind)
    }

    /// (M6-GA) Whether `id` deploys.
    pub fn deploying(&self, id: ObjectId) -> bool {
        self.deploying.contains_key(&id)
    }

    /// (M6-GA) The same, with `ids` deploying into `into`, and taking an Attack as `into` does: for the unit tests.
    pub fn with_deploying(mut self, ids: impl IntoIterator<Item = ObjectId>, into: ObjectKind) -> Self {
        for id in ids {
            self.deploying.insert(id, into);
            self.immobile.remove(&id);
            match combat::takes_attack_orders(into) {
                true => self.held.remove(&id),
                false => self.held.insert(id),
            };
        }
        self
    }

    /// (M6-GA) Whether `id` is of a type that can't move, and doesn't deploy: the gate refuses it every order that would
    /// move it (`sc_sim::Reject::CannotMove`).
    pub fn immobile(&self, id: ObjectId) -> bool {
        self.immobile.contains(&id)
    }

    /// (M6-GA) The same, with `ids` of a type that can't move: for the unit tests.
    pub fn with_immobile(mut self, ids: impl IntoIterator<Item = ObjectId>) -> Self {
        self.immobile.extend(ids);
        self
    }

    /// (M6-GH1) Whether `id` is an aircraft, and if so the Airfield it is assigned to, if any
    /// (`sc_sim::orders::resolution::Abilities::aircraft`, which its right-click on an own Airfield reads).
    pub fn aircraft(&self, id: ObjectId) -> Option<Option<ObjectId>> {
        self.bases.get(&id).copied()
    }

    /// (M6-GH1) Whether `id` is an aircraft with no ammo left: it takes no Attack nor Attack Ground (`Reject::OutOfAmmo`).
    pub fn empty(&self, id: ObjectId) -> bool {
        self.empty.contains(&id)
    }

    /// (M6-GH1) The same, with `ids` aircraft assigned to `base` with `ammo` left: for the unit tests.
    pub fn with_aircraft(mut self, ids: impl IntoIterator<Item = ObjectId>, base: Option<ObjectId>, ammo: u32) -> Self {
        for id in ids {
            self.bases.insert(id, base);
            if ammo == 0 {
                self.empty.insert(id);
                self.held.insert(id);
            }
        }
        self
    }

    /// (M6-GD) Whether `id` holds still for its drone, out (`sc_sim::tether::holds`): the gate refuses it every order
    /// that would move it (`sc_sim::Reject::DroneOut`).
    pub fn tethered(&self, id: ObjectId) -> bool {
        self.tethered.contains(&id)
    }

    /// (M6-GD) The same, with `ids` holding still for their drones: for the unit tests.
    pub fn with_tethered(mut self, ids: impl IntoIterator<Item = ObjectId>) -> Self {
        self.tethered.extend(ids);
        self
    }

    /// (M6-CM) Whether `id` is an illusion (`sc_sim::mirage`), which the Altar doesn't take in
    /// (`sc_sim::orders::resolution::Abilities::illusion`).
    pub fn illusion(&self, id: ObjectId) -> bool {
        self.illusions.contains(&id)
    }

    /// (M6-CM) The same, with `ids` illusions: for the unit tests.
    pub fn with_illusions(mut self, ids: impl IntoIterator<Item = ObjectId>) -> Self {
        self.illusions.extend(ids);
        self
    }

    /// Whether `id` is a worker that carries a load.
    pub fn carries(&self, id: ObjectId) -> bool {
        self.loaded.contains(&id)
    }

    /// The same, with `ids` carrying a load: for the unit tests.
    pub fn with_loaded(mut self, ids: impl IntoIterator<Item = ObjectId>) -> Self {
        self.loaded.extend(ids);
        self
    }

    /// The same, with `ids` transforming: for the unit tests.
    pub fn with_transforming(mut self, ids: impl IntoIterator<Item = ObjectId>) -> Self {
        self.transforming.extend(ids);
        self
    }

    /// The same, with `ids` units without a weapon (a Disciple's): for the unit tests.
    pub fn with_unarmed(mut self, ids: impl IntoIterator<Item = ObjectId>) -> Self {
        self.held.extend(ids);
        self
    }

    /// Whether `id` isn't in operation yet: a Beacon under deployment, a building under Warp In
    /// (`factions/colonists/economy.md > Beacon Deployment > While transforming: only Cancel`, design change 0116:
    /// "no rally point, no other commands"), a site under construction (`factions/syndicate/units/agent.md > While the
    /// Tunnel is under construction`: "It does nothing but Cancel"); none of its functions work yet.
    pub fn transforming(&self, id: ObjectId) -> bool {
        self.transforming.contains(&id)
    }

    /// Whether `id` takes an Attack: not a Peacekeeper (Shielded), nor a unit without a weapon
    /// (`sc_sim::combat::takes_attack_orders`). True for an object that isn't a unit, which an Attack never goes to.
    pub fn can_attack(&self, id: ObjectId) -> bool {
        !self.held.contains(&id)
    }

    /// Whether `id` takes no Attack for its Riot Shield, up: a Peacekeeper (Shielded).
    pub fn shielded(&self, id: ObjectId) -> bool {
        self.shielded.contains(&id)
    }
}

/// The modes of units whose Riot Shields do these, for the unit tests: a shield down or on its way down is a
/// Peacekeeper's, which takes an Attack; one up, on its way up or waiting for its shot to go up is a Peacekeeper
/// (Shielded)'s, which takes none (design change 0202, `sc_sim::riot_shield::takes_attack_orders`).
impl FromIterator<(ObjectId, ShieldMode)> for Modes {
    fn from_iter<I: IntoIterator<Item = (ObjectId, ShieldMode)>>(shields: I) -> Self {
        let held: BTreeSet<ObjectId> = (shields.into_iter())
            .filter(|&(_, mode)| !matches!(mode, ShieldMode::Lowered | ShieldMode::Lowering { .. }))
            .map(|(id, _)| id)
            .collect();
        Self {
            shielded: held.clone(),
            held,
            loaded: BTreeSet::new(),
            transforming: BTreeSet::new(),
            tethered: BTreeSet::new(),
            bases: BTreeMap::new(),
            empty: BTreeSet::new(),
            deploying: BTreeMap::new(),
            immobile: BTreeSet::new(),
            illusions: BTreeSet::new(),
        }
    }
}
