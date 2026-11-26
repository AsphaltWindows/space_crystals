//! The selected units' modes ([`Modes`]): which of them take no Attack, carry a load, or aren't in operation yet.

use std::collections::BTreeSet;

use sc_sim::{ObjectId, ShieldMode, Sim, riot_shield};

use crate::selection::Selection;

/// What the selected units' modes let them do now: which of them take no Attack (`factions/gdo/units/peacekeeper.md >
/// Riot Shield > During the toggle`, design change 0202: "which orders a unit accepts follows its new type from the
/// press": a Peacekeeper (Shielded), whose shield is up, on its way up or waiting for its shot to go up, but not a
/// Peacekeeper whose shield is on its way down, which takes an Attack and holds it until its shield is down), by the
/// type it groups as in the selection (the type from the press, `crate::selection::ControlState::toggle`) and the sim's
/// own query (`sc_sim::riot_shield::takes_attack_orders`, so that the panel and the right-click never judge a type
/// otherwise than the gate), which of the workers carry a load, which a right-click on a drop-off reads
/// (`factions/colonists/units/drone.md > Right-Click Resolution`: "Own Egress Node (carrying resources)") and the
/// Agent's Drop Off, and which of the selected objects aren't in operation yet (`sc_sim::Object::is_operational`: one
/// that transforms, or a site under construction). Which way a shield goes is its unit's type (design change 0173),
/// whose panel offers the command that turns it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Modes {
    /// The units whose type takes no Attack.
    held: BTreeSet<ObjectId>,
    loaded: BTreeSet<ObjectId>,
    transforming: BTreeSet<ObjectId>,
}

impl Modes {
    /// The modes of `selection`'s objects in `sim`.
    pub fn of(sim: &Sim, selection: &Selection) -> Self {
        Self {
            held: (selection.members())
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
        }
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

    /// Whether `id` isn't in operation yet: a Beacon under deployment, a building under Warp In
    /// (`factions/colonists/economy.md > Beacon Deployment > While transforming: only Cancel`, design change 0116:
    /// "no rally point, no other commands"), a site under construction (`factions/syndicate/units/agent.md > While the
    /// Tunnel is under construction`: "It does nothing but Cancel"); none of its functions work yet.
    pub fn transforming(&self, id: ObjectId) -> bool {
        self.transforming.contains(&id)
    }

    /// Whether `id` takes an Attack: not a Peacekeeper (Shielded) (`sc_sim::riot_shield::takes_attack_orders`).
    pub fn can_attack(&self, id: ObjectId) -> bool {
        !self.held.contains(&id)
    }
}

/// The modes of units whose Riot Shields do these, for the unit tests: a shield down or on its way down is a
/// Peacekeeper's, which takes an Attack; one up, on its way up or waiting for its shot to go up is a Peacekeeper
/// (Shielded)'s, which takes none (design change 0202, `sc_sim::riot_shield::takes_attack_orders`).
impl FromIterator<(ObjectId, ShieldMode)> for Modes {
    fn from_iter<I: IntoIterator<Item = (ObjectId, ShieldMode)>>(shields: I) -> Self {
        Self {
            held: (shields.into_iter())
                .filter(|&(_, mode)| !matches!(mode, ShieldMode::Lowered | ShieldMode::Lowering { .. }))
                .map(|(id, _)| id)
                .collect(),
            loaded: BTreeSet::new(),
            transforming: BTreeSet::new(),
        }
    }
}
