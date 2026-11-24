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
//! Firing or in Cooldown; it goes to the recipients that may convert, `Reject::CannotConvert` when none may), and a
//! greyed-out Warp In menu entry (`factions/colonists/economy.md > Warp In`: the Beacon busy deploying, or too few
//! Space Crystals; where the building would stand, the ghost shows).
//!
//! Every greyed-out button has its reason ([`WhyUnavailable`], from the same check that greys it), which pressing it
//! shows (design change 0162).

use std::collections::{BTreeMap, BTreeSet};

use sc_sim::{
    BuildingOrientation, Command, ConstructionCommand, Constructor, Object, ObjectId, PlayerCommand, PlayerId, Reject,
    RiotShield, Sim,
};

use super::{PanelCommand, commands, recipients};
use crate::selection::Selection;

/// What the selected units' modes let them do now: their Riot Shields (`factions/gdo/units.md > Peacekeeper > Riot
/// Shield`), by id, as the sim has them, and which of the selected objects transform (`sc_sim::Object::transform`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Modes {
    shields: BTreeMap<ObjectId, RiotShield>,
    transforming: BTreeSet<ObjectId>,
}

impl Modes {
    /// The modes of `selection`'s objects in `sim`.
    pub fn of(sim: &Sim, selection: &Selection) -> Self {
        let mut modes: Self = selection
            .ids()
            .filter_map(|id| Some((id, sim.object(id)?.riot_shield?)))
            .collect();
        modes.transforming = selection
            .ids()
            .filter(|&id| sim.object(id).is_some_and(Object::is_transforming))
            .collect();
        modes
    }

    /// The same, with `ids` transforming: for the unit tests.
    pub fn with_transforming(mut self, ids: impl IntoIterator<Item = ObjectId>) -> Self {
        self.transforming.extend(ids);
        self
    }

    /// Whether `id` transforms: a Beacon under deployment, a building under Warp In
    /// (`factions/colonists/economy.md > Beacon Deployment > While transforming: only Cancel`, design change 0116:
    /// "no rally point, no other commands"; none of its functions work yet).
    pub fn transforming(&self, id: ObjectId) -> bool {
        self.transforming.contains(&id)
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
            transforming: BTreeSet::new(),
        }
    }
}

/// Why a button is greyed out (design change 0162, `systems/controls/selection_and_hud.md > CommandPanel > Pressing
/// an unavailable command`: "nothing is issued, and the player gets the same feedback as when an order is refused,
/// naming the reason"), from the check that greys it: what pressing it says (`crate::command_ui::feedback`).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum WhyUnavailable {
    /// The command gate would refuse the command it sends (`Sim::validate`): too few Space Crystals, a full queue,
    /// nothing to cancel, a constructor or a Beacon already busy, no recipient that may convert.
    Refused(Reject),
    /// Every recipient's Riot Shield keeps it from attacking (`factions/gdo/units.md > Peacekeeper > Riot Shield`:
    /// "the player must lower the shield before ordering an attack").
    ShieldsUp,
    /// Another command awaits its target, or a building is placed (`> AwaitingTarget > Other commands while
    /// targeting`, design change 0066: "unavailable until the player cancels").
    Waiting,
    /// The type can't carry it out (`> BasicCombatUnitInterfaceState`: Reverse without CanReverse, AttackGround
    /// without CanTargetGround).
    NotForThisType,
    /// It isn't in the game yet: a building whose costs are TBD (design change 0124), a build menu none of whose
    /// buildings is, an order the sim has none of (Reverse, AttackGround).
    NotYet,
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
    /// Every object of the ActiveGroup transforms (design change 0116): its panel is the Cancel alone
    /// (`super::TRANSFORMING_GRID`). **Reading**: a group of portals of which only some transform (they are
    /// Groupable) shows the portal panel, whose commands the gate gives to the working ones only (production and the
    /// rally start at completion).
    pub transforming: bool,
    /// The production and construction commands and the Convert the gate would refuse now, for the objects they go
    /// to ([`recipients`]).
    refused: BTreeMap<PanelCommand, Reject>,
}

impl Conditions {
    /// The conditions of `player`'s `selection` in `sim`: the modes, the ActiveGroup's constructor state, and
    /// `Sim::validate`'s answer to the `Command::Production`, `Command::Construction` or `Command::Ability` each such
    /// command the ActiveGroup may show would send ([`commands`]). A toggled ability's is never refused: the gate
    /// takes it whenever a recipient carries it. A Warp In menu entry asks for its Warp In unturned, and leaves out the
    /// placement's refusal (`Reject::Unplaceable`, the gate's last check): the player turns the ghost before it is
    /// placed, and the ghost shows each tile's problem.
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
                } else if let PanelCommand::WarpIn(kind) = command {
                    let orientation = BuildingOrientation::default();
                    Command::Construction {
                        objects,
                        command: ConstructionCommand::WarpIn { kind, orientation },
                    }
                } else {
                    Command::Ability {
                        units: objects,
                        ability: command.ability(false).filter(|_| !command.toggles())?,
                    }
                };
                let sent = PlayerCommand { player, command: sent };
                let reject = sim.validate(&sent).err()?;
                let placed = matches!(command, PanelCommand::WarpIn(_)) && matches!(reject, Reject::Unplaceable(_));
                (!placed).then_some((command, reject))
            })
            .collect();
        let group = selection.active_group();
        let constructor = group
            .as_ref()
            .and_then(|group| group.instances.first().copied())
            .and_then(|id| sim.object(id)?.constructor);
        let transforming = group.is_some_and(|group| {
            group
                .instances
                .iter()
                .all(|&id| sim.object(id).is_some_and(Object::is_transforming))
        });
        Self {
            modes: Modes::of(sim, selection),
            constructor,
            transforming,
            refused,
        }
    }

    /// These modes, no constructor, nothing transforming and no command refused: for the unit tests.
    pub fn with_modes(modes: Modes) -> Self {
        Self {
            modes,
            constructor: None,
            transforming: false,
            refused: BTreeMap::new(),
        }
    }

    /// The same, with the whole ActiveGroup transforming: for the unit tests.
    pub fn while_transforming(mut self) -> Self {
        self.transforming = true;
        self
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
    /// attack"), a production or construction command, a Warp In or a Convert while the gate would take it, anything
    /// else always.
    pub fn allows(&self, selection: &Selection, command: PanelCommand) -> bool {
        self.unavailable(selection, command).is_none()
    }

    /// Why the game doesn't let `command` go to its recipients in `selection` now ([`Conditions::allows`]), if it
    /// doesn't: their shields, or the gate's refusal.
    pub fn unavailable(&self, selection: &Selection, command: PanelCommand) -> Option<WhyUnavailable> {
        match command {
            PanelCommand::Attack => {
                let may = recipients(selection, command)
                    .into_iter()
                    .any(|id| self.modes.can_attack(id));
                (!may).then_some(WhyUnavailable::ShieldsUp)
            }
            _ => self.refusal(command).map(WhyUnavailable::Refused),
        }
    }
}
