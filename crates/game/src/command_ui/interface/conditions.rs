//! What the game allows a selection's commands now, as the sim has it (`systems/controls/selection_and_hud.md >
//! CommandPanel`: "derived from ControlState and game state each tick"): the selected units' modes ([`Modes`]), the
//! ActiveGroup's constructor state, and the command gate's answer to each production and construction command the
//! ActiveGroup offers, and to its Convert ([`Conditions`]).
//!
//! A button is available exactly when the sim would take its command now (plan decision T10: "button availability
//! uses the same `sc_sim` predicates as the gate"): the panel asks `Sim::validate` with the command it would send,
//! to the objects it would go to, so a greyed-out Train is one the gate refuses (`factions/gdo/structures.md >
//! Barracks > ObjectInterfaceState[Barracks]`: "Only available if BuildQueue has fewer than 20 entries and player has
//! sufficient Space Crystals (Unit Control is checked when production starts: factions/gdo/overview.md > Unit
//! Control)"; "X: Cancel Production [...] Only available if BuildQueue is not empty"; design changes 0081 and 0143:
//! the army cap leaves a Train available, the Colonists' Beacon Capacity alike), and so is a greyed-out build menu
//! entry (`> DeploymentCenter > ObjectInterfaceState[DeploymentCenter]`: "Only available if player has sufficient
//! Space Crystals"; "Other build options unavailable" while the Deployment Center constructs or holds a building
//! ready), and so is a greyed-out Convert (`factions/colonists/units.md > Weapon Conversion > Eligibility`: not while
//! Firing or in Cooldown; it goes to the recipients that may convert, `Reject::CannotConvert` when none may).

use std::collections::{BTreeMap, BTreeSet};

use sc_sim::{Command, Constructor, ObjectId, PlayerCommand, PlayerId, Reject, RiotShield, Sim};

use super::{PanelCommand, commands, recipients};
use crate::selection::Selection;

/// What the selected units' modes let them do now, as the sim has them: their Riot Shields (`factions/gdo/units.md >
/// Peacekeeper > Riot Shield`), by id, and which of the workers carry a load, which a right-click on a drop-off reads
/// (`factions/colonists/units.md > Drone > Right-Click Resolution`: "Own Egress Node (carrying resources)").
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Modes {
    shields: BTreeMap<ObjectId, RiotShield>,
    loaded: BTreeSet<ObjectId>,
}

impl Modes {
    /// The modes of `selection`'s objects in `sim`.
    pub fn of(sim: &Sim, selection: &Selection) -> Self {
        let mut modes: Self = selection
            .ids()
            .filter_map(|id| Some((id, sim.object(id)?.riot_shield?)))
            .collect();
        modes.loaded = (selection.ids())
            .filter(|&id| (sim.object(id)).is_some_and(|o| o.worker.is_some_and(|worker| worker.load > 0)))
            .collect();
        modes
    }

    /// Whether `id` is a worker that carries a load.
    pub fn carries(&self, id: ObjectId) -> bool {
        self.loaded.contains(&id)
    }

    /// Whether `id` may attack now: not while its Riot Shield is up, nor on its way up or down
    /// (`RiotShield::allows_attack`).
    pub fn can_attack(&self, id: ObjectId) -> bool {
        self.shields.get(&id).is_none_or(|shield| shield.allows_attack())
    }

    /// Whether `ids` carry Riot Shields and all of those are up or on their way up: a toggle lowers them, and
    /// otherwise raises them all (placeholder: the design doesn't say what a toggle does to a group whose shields
    /// differ).
    pub fn shields_up(&self, ids: &[ObjectId]) -> bool {
        let mut shields = ids.iter().filter_map(|id| self.shields.get(id)).peekable();
        shields.peek().is_some() && shields.all(|shield| shield.is_going_up())
    }
}

impl FromIterator<(ObjectId, RiotShield)> for Modes {
    fn from_iter<I: IntoIterator<Item = (ObjectId, RiotShield)>>(shields: I) -> Self {
        Self {
            shields: shields.into_iter().collect(),
            loaded: BTreeSet::new(),
        }
    }
}

/// What the game allows a selection's commands now: its units' [`Modes`], the ActiveGroup's constructor state, and
/// each production or construction command or Convert the ActiveGroup may show that the command gate would refuse,
/// with why.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Conditions {
    /// The selected units' modes.
    pub modes: Modes,
    /// The state of the ActiveGroup's constructor, if it is one: a Deployment Center or an Extraction Facility, one
    /// instance (they are Ungroupable), whose panel follows it (`super::state_grid`).
    pub constructor: Option<Constructor>,
    /// The production and construction commands and the Convert the gate would refuse now, for the objects they go
    /// to ([`recipients`]).
    refused: BTreeMap<PanelCommand, Reject>,
}

impl Conditions {
    /// The conditions of `player`'s `selection` in `sim`: the modes, the ActiveGroup's constructor state, and
    /// `Sim::validate`'s answer to the `Command::Production`, `Command::Construction` or `Command::Ability` each such
    /// command the ActiveGroup may show would send ([`commands`]). A toggled ability's is never refused: the gate
    /// takes it whenever a recipient carries it.
    pub fn of(sim: &Sim, player: PlayerId, selection: &Selection) -> Self {
        let commands = selection
            .active()
            .map(|active| commands(active.kind))
            .unwrap_or_default();
        let refused = commands
            .into_iter()
            .filter_map(|command| {
                let objects = recipients(selection, command);
                let sent = if let Some(production) = command.production() {
                    Command::Production {
                        objects,
                        command: production,
                    }
                } else if let Some(construction) = command.construction() {
                    Command::Construction {
                        objects,
                        command: construction,
                    }
                } else {
                    Command::Ability {
                        units: objects,
                        ability: command.ability(false).filter(|_| !command.toggles())?,
                    }
                };
                let sent = PlayerCommand { player, command: sent };
                sim.validate(&sent).err().map(|reject| (command, reject))
            })
            .collect();
        let constructor = selection
            .active_group()
            .and_then(|group| group.instances.first().copied())
            .and_then(|id| sim.object(id)?.constructor);
        Self {
            modes: Modes::of(sim, selection),
            constructor,
            refused,
        }
    }

    /// These modes, no constructor, and no command refused: for the unit tests.
    pub fn with_modes(modes: Modes) -> Self {
        Self {
            modes,
            constructor: None,
            refused: BTreeMap::new(),
        }
    }

    /// The same, with the ActiveGroup's constructor in state `constructor`: for the unit tests.
    pub fn with_constructor(mut self, constructor: Constructor) -> Self {
        self.constructor = Some(constructor);
        self
    }

    /// The same, with `command` refused for `reject`: for the unit tests.
    pub fn refusing(mut self, command: PanelCommand, reject: Reject) -> Self {
        self.refused.insert(command, reject);
        self
    }

    /// Why the gate would refuse production or construction command or Convert `command` now, if it would.
    pub fn refusal(&self, command: PanelCommand) -> Option<Reject> {
        self.refused.get(&command).copied()
    }

    /// Whether the game lets `command` go to its recipients in `selection` now: Attack while one of them can attack
    /// (`factions/gdo/units.md > Peacekeeper > Riot Shield`: "the player must lower the shield before ordering an
    /// attack"), a production or construction command or a Convert while the gate would take it, anything else
    /// always.
    pub fn allows(&self, selection: &Selection, command: PanelCommand) -> bool {
        match command {
            PanelCommand::Attack => recipients(selection, command)
                .into_iter()
                .any(|id| self.modes.can_attack(id)),
            _ => self.refusal(command).is_none(),
        }
    }
}
