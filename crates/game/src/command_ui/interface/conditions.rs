//! What the game allows a selection's commands now, as the sim has it (`systems/controls/selection_and_hud.md >
//! CommandPanel`: "derived from ControlState and game state each tick"): the selected units' modes ([`Modes`]), the
//! ActiveGroup's constructor state, and the command gate's answer to each production and construction command the
//! ActiveGroup offers, and to its toggles ([`Conditions`]).
//!
//! A button is available exactly when the sim would take its command now (plan decision T10: "button availability uses
//! the same `sc_sim` predicates as the gate"): the panel asks `Sim::validate` with the command it would send, to the
//! objects it would go to, so a greyed-out Train is one the gate refuses (`factions/gdo/structures/barracks.md >
//! ObjectInterfaceState[Barracks]`: "Only available if BuildQueue has fewer than 20 entries and player has sufficient
//! Space Crystals (Unit Control is checked when production starts: factions/gdo/overview.md > Unit Control)"; "X:
//! Cancel Production [...] Only available if BuildQueue is not empty"; design changes 0081 and 0143: the army cap
//! leaves a Train available, the Colonists' Beacon Capacity alike), and so is a greyed-out build menu entry
//! (`factions/gdo/structures/deployment_center.md > ObjectInterfaceState[DeploymentCenter]`: "Only available if player
//! has sufficient Space Crystals"; "Other build options unavailable" while the Deployment Center constructs or holds a
//! building ready), and so is a greyed-out Warp In menu entry (`factions/colonists/economy.md > Warp In`: the Beacon
//! busy deploying, or too few Space Crystals; where the building would stand, the ghost shows). The mode toggles are
//! never greyed out: the gate takes a Convert or a Raise or Lower Shield whenever a recipient has the mode, at any
//! point of a toggle or a shot (design change 0198: a press flips the target state).
//!
//! Every greyed-out button has its reason ([`WhyUnavailable`], from the same check that greys it), which pressing it
//! shows (design change 0162). The Egress Node's F shows only while the node has Drones to select
//! (`factions/colonists/economy.md > Egress Node > Egress Node ObjectInterfaceState`, [`Conditions::shows`]).
//!
//! A Tunnel's EjectMenu tiles are its Ejects, which the gate answers too (`factions/syndicate/tunnels.md > EjectMenu`,
//! slice M4-J2): a tile is greyed while no unit of its type is in the network free to leave, while every one of them
//! only just walked in (design change 0288 point 1), or while this Tunnel's tier doesn't let the type out; and its
//! count is the network's ([`Conditions::network_count`]). Its ExpandMenu's entries are greyed by the gate's answer to
//! an Expand short of its placement (`Sim::can_expand`, slice M4-F: an operation under way, a Tier Requirement above
//! this Tunnel's tier, a prerequisite missing, too few Space Crystals), where the expansion goes being the ghost's to
//! show; its X while it performs no operation. Its Q, Upgrade Tunnel (slice M4-G), asks the gate for its Upgrade, by
//! the truth (`Sim::can_upgrade`: already at its top tier, an operation under way, too few Supplies, or something that
//! blocks the larger area, never said what), and its hint names the price ([`Conditions::price`],
//! `Sim::upgrade_price`).
//!
//! **Prerequisites** (slice M4-K, `core/tech_and_research.md > Tech Prerequisites`: "Options whose TechPrerequisite is
//! not met are unavailable in the build menu"): every entry the gate refuses for a type the player doesn't own
//! (`Reject::Requires`, `sc_sim::tech`) is greyed with "Requires …" (`crate::command_ui::feedback::requires_text`),
//! whatever its kind: the Hideout's Guard ("Requires an Optics Lab"), an ExpandMenu entry, a GDO building or unit,
//! a Warp In. A placeholder entry whose prerequisite is in the game says it the same way while the player owns none,
//! and that it isn't in the game yet once they own one ([`Conditions::planned`]).
//!
//! The Agent's A, Build Tunnel (`factions/syndicate/units/agent.md > Agent ObjectInterfaceState`, slice M4-J1), asks
//! for its Build order where it would stand anywhere, and leaves out the placement's refusal as a Warp In does: it is
//! greyed while the bank can't pay the Tunnel's price ([`Conditions::price`], `Sim::build_price`; design change 0233:
//! "a Build order that can't be paid is refused"). So is its D, Build Laser Turret (slice M4-S1), greyed while the
//! player owns no Optics Lab ("Requires an Optics Lab", the gate's first check) or the bank holds less than its 100
//! Space Crystals, its price in its own resource (`sc_sim::construction::site::BuildPrice`). Its S, Drop Off, is greyed
//! while none of its recipients carries a load (`factions/syndicate/units/agent.md > DefaultState commands`: "Always
//! visible, greyed out when Agent is not carrying resources"), the gate's own test (`Reject::NothingToDeliver`). A
//! site's panel is its Cancel alone ([`Conditions::site`], design change 0237), an expansion's too.

use std::collections::{BTreeMap, BTreeSet};

use sc_sim::{
    BuildingOrientation, Command, ConstructionCommand, Constructor, ObjectId, ObjectKind, OrderKind, PlayerCommand,
    PlayerId, Reject, ShieldMode, Sim, TilePos, riot_shield,
};

use sc_sim::construction::site::BuildPrice;

use super::{PanelCommand, Planned, commands, recipients};
use crate::command_ui::feedback::requires_text;
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

/// Why a button is greyed out (design change 0162, `systems/controls/selection_and_hud.md > CommandPanel > Pressing
/// an unavailable command`: "nothing is issued, and the player gets the same feedback as when an order is refused,
/// naming the reason"), from the check that greys it: what pressing it says (`crate::command_ui::feedback`).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum WhyUnavailable {
    /// The command gate would refuse the command it sends (`Sim::validate`): too few Space Crystals, a full queue,
    /// nothing to cancel, a constructor or a Beacon already busy.
    Refused(Reject),
    /// Every recipient is a Peacekeeper (Shielded), which takes no Attack (`factions/gdo/units/peacekeeper.md > Riot
    /// Shield`: "the player must lower the shield before ordering an attack"; design change 0202).
    ShieldsUp,
    /// Another command awaits its target, or a building is placed (`systems/controls/selection_and_hud.md >
    /// AwaitingTarget > Other commands while targeting`, design change 0066: "unavailable until the player cancels").
    Waiting,
    /// The type can't carry it out (`systems/controls/selection_and_hud.md > BasicCombatUnitInterfaceState`: Reverse
    /// without CanReverse, AttackGround without CanTargetGround).
    NotForThisType,
    /// It isn't in the game yet: a building or a unit whose costs are TBD (design change 0124), a build menu none of
    /// whose buildings is, an order the sim has none of (Reverse).
    NotYet,
    /// A placeholder entry's own reason, the message the gate's refusal will give once its command is in the game
    /// ([`Conditions::planned`], slice M4-J2): the ExpandMenu's Laser Turret "Requires an Optics Lab".
    Placeholder(&'static str),
}

/// What the game allows a selection's commands now: its units' [`Modes`], the ActiveGroup's constructor state, and
/// each production or construction command or toggle the ActiveGroup may show that the command gate would refuse,
/// with why.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Conditions {
    /// The selected units' modes.
    pub modes: Modes,
    /// The state of the ActiveGroup's constructor, if it is one: a Deployment Center or an Extraction Facility, one
    /// instance (they are Ungroupable), whose panel follows it (`super::state_grid`). Whether the ActiveGroup
    /// transforms is its key's (`crate::selection::GroupKey::unfinished`, design change 0185: the unfinished
    /// buildings of a type form a group of their own, whose panel is the Cancel alone).
    pub constructor: Option<Constructor>,
    /// The production and construction commands and the toggles the gate would refuse now, for the objects they go
    /// to ([`recipients`]).
    refused: BTreeMap<PanelCommand, Reject>,
    /// The Drones the active Egress Node's F selects (`sc_sim::Sim::returning_drones`), in id order: none while the
    /// ActiveGroup isn't a node that offers F.
    returning: Vec<ObjectId>,
    /// The EjectMenu's counts: the units of each type in the player's Tunnel Network that aren't reserved on an eject
    /// queue (`sc_sim::Sim::network_counts`; `factions/syndicate/tunnels.md > EjectMenu`: "the tile's count drops at
    /// the click"); none while the ActiveGroup isn't a Tunnel.
    network: BTreeMap<ObjectKind, u32>,
    /// The price of each Build the ActiveGroup offers (`PanelCommand::Build`), in its resource, as the sim would charge
    /// it now (`Sim::build_price`, design change 0233): a Tunnel's in Supplies, a Laser Turret's in Space Crystals
    /// (slice M4-S1); and of a Tunnel's Upgrade, in Supplies (`PanelCommand::UpgradeTunnel`, `Sim::upgrade_price`,
    /// slice M4-G).
    prices: BTreeMap<PanelCommand, BuildPrice>,
    /// Whether the ActiveGroup is a site under construction (`sc_sim::Object::is_site`; design change 0237: "a Tunnel
    /// site is a structure that does nothing but Cancel"), whose panel is its X Cancel Construction alone, whatever
    /// its type (`super::templates::SITE_GRID`).
    pub site: bool,
    /// The placeholder entries' prerequisites that the player owns now (`super::Planned::requires`,
    /// `Sim::requirement_met`): none in the unit tests' conditions but those they give ([`Conditions::owning`]).
    owned: BTreeSet<ObjectKind>,
}

impl Conditions {
    /// The conditions of `player`'s `selection` in `sim`: the modes, the ActiveGroup's constructor state, and
    /// `Sim::validate`'s answer to the `Command::Production`, `Command::Construction` or `Command::Ability` each such
    /// command the ActiveGroup may show would send ([`commands`]). Raise and Lower Shield and Convert are never
    /// refused: the gate takes them whenever a recipient has the mode (design change 0198). The modes are those of
    /// the types the selection groups its objects as ([`Modes::of`]). A Warp In menu entry asks for its Warp In
    /// unturned, and leaves
    /// out the placement's refusal (`Reject::Unplaceable`, the gate's last check): the player turns the ghost before it
    /// is placed, and the ghost shows each tile's problem. An Agent's Build asks for its Build order, unqueued, the
    /// same way ([`build_probe`]). A Tunnel's EjectMenu tile and its Cancel ask for their `Command::Tunnel`, and the
    /// network's counts are the player's while the ActiveGroup opens the EjectMenu. An ExpandMenu entry asks
    /// `Sim::can_expand`, the gate's Expand short of its placement, for the same reason as a Warp In's.
    pub fn of(sim: &Sim, player: PlayerId, selection: &Selection) -> Self {
        let commands = selection
            .active()
            .map(|active| commands(active.group_type()))
            .unwrap_or_default();
        let returning = commands
            .contains(&PanelCommand::SelectReturningDrones)
            .then(|| recipients(selection, PanelCommand::SelectReturningDrones))
            .and_then(|nodes| nodes.first().copied())
            .map(|node| sim.returning_drones(node))
            .unwrap_or_default();
        let network = if commands.contains(&PanelCommand::EjectMenu) {
            sim.network_counts(player)
        } else {
            BTreeMap::new()
        };
        let refused = commands
            .iter()
            .filter_map(|&command| {
                let objects = recipients(selection, command);
                if let PanelCommand::Expand(kind) = command {
                    let reject = sim.can_expand(player, *objects.first()?, kind).err()?;
                    return Some((command, reject));
                }
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
                } else if let Some(tunnel) = command.tunnel() {
                    Command::Tunnel {
                        tunnel: *objects.first()?,
                        command: tunnel,
                    }
                } else if let PanelCommand::Build(kind) = command {
                    build_probe(objects, kind)
                } else {
                    Command::Ability {
                        units: objects,
                        ability: command.ability()?,
                    }
                };
                let sent = PlayerCommand { player, command: sent };
                let reject = sim.validate(&sent).err()?;
                let placed = matches!(command, PanelCommand::WarpIn(_) | PanelCommand::Build(_))
                    && matches!(reject, Reject::Unplaceable(_));
                (!placed).then_some((command, reject))
            })
            .collect();
        let prices = (commands.iter())
            .filter_map(|&command| match command {
                PanelCommand::Build(kind) => Some((command, sim.build_price(player, kind)?)),
                PanelCommand::UpgradeTunnel => {
                    let tunnel = *recipients(selection, command).first()?;
                    Some((command, BuildPrice::supplies(sim.upgrade_price(tunnel)?)))
                }
                _ => None,
            })
            .collect();
        let owned = (Planned::ALL.into_iter())
            .filter_map(Planned::requires)
            .filter(|&kind| sim.requirement_met(player, kind))
            .collect();
        let group = selection.active_group();
        let first = group
            .as_ref()
            .and_then(|group| group.instances.first().copied())
            .and_then(|id| sim.object(id));
        Self {
            modes: Modes::of(sim, selection),
            constructor: first.and_then(|o| o.constructor),
            refused,
            returning,
            network,
            prices,
            site: first.is_some_and(|o| o.is_site()),
            owned,
        }
    }

    /// These modes, no constructor and no command refused: for the unit tests.
    pub fn with_modes(modes: Modes) -> Self {
        Self {
            modes,
            constructor: None,
            refused: BTreeMap::new(),
            returning: Vec::new(),
            network: BTreeMap::new(),
            prices: BTreeMap::new(),
            site: false,
            owned: BTreeSet::new(),
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

    /// The same, with the Drones `drones` returning to the active Egress Node: for the unit tests.
    pub fn with_returning(mut self, drones: Vec<ObjectId>) -> Self {
        self.returning = drones;
        self
    }

    /// The Drones the active Egress Node's F selects, in id order.
    pub fn returning(&self) -> &[ObjectId] {
        &self.returning
    }

    /// The same, with the ActiveGroup a site under construction: for the unit tests.
    pub fn on_site(mut self) -> Self {
        self.site = true;
        self
    }

    /// The same, with the player owning a `kind` that counts as a prerequisite: for the unit tests.
    pub fn owning(mut self, kind: ObjectKind) -> Self {
        self.owned.insert(kind);
        self
    }

    /// Why placeholder entry `planned` is greyed (design change 0162): the reason its command's gate will give, where
    /// it is known and true now (slice M4-J2's rule), so that the button says the same before and after the slice that
    /// brings it: its prerequisite while the player owns none ([`Planned::requires`]: the Laser Turret's "Requires an
    /// Optics Lab", as the gate words `Reject::Requires`, slice M4-K), else that it isn't in the game yet.
    pub fn planned(&self, planned: Planned) -> WhyUnavailable {
        match planned.requires() {
            Some(kind) if !self.owned.contains(&kind) => WhyUnavailable::Placeholder(requires_text(kind)),
            _ => WhyUnavailable::NotYet,
        }
    }

    /// The same, with `command` priced at `price`: for the unit tests.
    pub fn pricing(mut self, command: PanelCommand, price: BuildPrice) -> Self {
        self.prices.insert(command, price);
        self
    }

    /// What a Build or a Tunnel's Upgrade costs now, in its resource (`Sim::build_price`, `Sim::upgrade_price`): `None`
    /// for another command, and for the Upgrade of a Tunnel at its top tier.
    pub fn price(&self, command: PanelCommand) -> Option<BuildPrice> {
        self.prices.get(&command).copied()
    }

    /// The same, with `counts` the network's: for the unit tests.
    pub fn with_network(mut self, counts: impl IntoIterator<Item = (ObjectKind, u32)>) -> Self {
        self.network = counts.into_iter().collect();
        self
    }

    /// An EjectMenu tile's count: how many units of `kind` are in the player's Tunnel Network and not reserved on an
    /// eject queue.
    pub fn network_count(&self, kind: ObjectKind) -> u32 {
        self.network.get(&kind).copied().unwrap_or(0)
    }

    /// Whether `command` shows in its slot now: the Egress Node's F only while there are Drones for it to select
    /// (`factions/colonists/economy.md > Egress Node > Egress Node ObjectInterfaceState`: "Only available if at least
    /// one such Drone exists", read as hidden otherwise, as the ExtractionFacility's Cancels are "Only visible
    /// when"), anything else always.
    pub fn shows(&self, command: PanelCommand) -> bool {
        command != PanelCommand::SelectReturningDrones || !self.returning.is_empty()
    }

    /// Why the gate would refuse production or construction command or toggle `command` now, if it would.
    pub fn refusal(&self, command: PanelCommand) -> Option<Reject> {
        self.refused.get(&command).copied()
    }

    /// Whether the game lets `command` go to its recipients in `selection` now: Attack while one of them takes it
    /// (`factions/gdo/units/peacekeeper.md > Riot Shield`: "the player must lower the shield before ordering an
    /// attack"; design change 0202: a Peacekeeper whose shield is on its way down takes it), a production or
    /// construction command, a Warp In or a toggle while the gate would take it, anything else always.
    pub fn allows(&self, selection: &Selection, command: PanelCommand) -> bool {
        self.unavailable(selection, command).is_none()
    }

    /// Why the game doesn't let `command` go to its recipients in `selection` now ([`Conditions::allows`]), if it
    /// doesn't: their types, their loads (a Drop Off none of whose recipients carries anything, the gate's
    /// `Reject::NothingToDeliver`), or the gate's refusal.
    pub fn unavailable(&self, selection: &Selection, command: PanelCommand) -> Option<WhyUnavailable> {
        match command {
            PanelCommand::Attack => {
                let may = recipients(selection, command)
                    .into_iter()
                    .any(|id| self.modes.can_attack(id));
                (!may).then_some(WhyUnavailable::ShieldsUp)
            }
            PanelCommand::DropOff => {
                let loaded = recipients(selection, command)
                    .into_iter()
                    .any(|id| self.modes.carries(id));
                (!loaded).then_some(WhyUnavailable::Refused(Reject::NothingToDeliver))
            }
            _ => self.refusal(command).map(WhyUnavailable::Refused),
        }
    }
}

/// The Build order an Agent's A would send (`PanelCommand::Build`), to ask the gate whether the button is available:
/// unqueued, unturned, at the map's north-west corner. Where it stands matters only to the placement's check, the
/// gate's last (`Reject::Unplaceable`), which the panel leaves out: the ghost judges each spot as it is placed.
fn build_probe(units: Vec<ObjectId>, kind: ObjectKind) -> Command {
    let kind = OrderKind::Build {
        kind,
        origin: TilePos::new(0, 0),
        orientation: BuildingOrientation::default(),
    };
    Command::Order {
        units,
        kind,
        queue: false,
    }
}
