//! What the game allows a selection's commands now, as the sim has it (`systems/controls/interface.md >
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
//! A Tunnel's EjectMenu tiles are its Ejects, which the gate answers too (`factions/syndicate/tunnel_menus.md >
//! EjectMenu`, slice M4-J2): a tile is greyed while no unit of its type is in the network free to leave, while every
//! one of them only just walked in (design change 0288 point 1), or while this Tunnel's tier doesn't let the type out;
//! and its count is the network's ([`Conditions::network_count`]). Its ExpandMenu's entries are greyed by the gate's
//! answer to an Expand short of its placement (`Sim::can_expand`, slice M4-F: an operation under way, a Tier
//! Requirement above this Tunnel's tier, a prerequisite missing, too few Space Crystals), where the expansion goes
//! being the ghost's to show; its X while it performs no operation. Its Q, Upgrade Tunnel (slice M4-G), asks the gate
//! for its Upgrade, by the truth (`Sim::can_upgrade`: already at its top tier, an operation under way, too few
//! Supplies, or something that blocks the larger area, never said what), and its hint names the price
//! ([`Conditions::price`], `Sim::upgrade_price`).
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
//! "a Build order that can't be paid is refused"). (Slice M4-S1's D, Build Laser Turret, is gone with design change
//! 0359: the Agent's D is Repair, slice M5-N8's, which awaits its target.) A Build's price is in its own
//! resource (`sc_sim::construction::site::BuildPrice`): a Cults building's in Space Crystals. Its S, Drop Off, is greyed
//! while none of its recipients carries a load (`factions/syndicate/units/agent.md > DefaultState commands`: "Always
//! visible, greyed out when Agent is not carrying resources"), the gate's own test (`Reject::NothingToDeliver`). A
//! site's panel is its Cancel alone ([`Conditions::site`], design change 0237), an expansion's too.
//!
//! A pool building's kind buttons (design change 0337, `factions/cults/structures/README.md > Choosing Recruits,
//! Release and Eject All`: "Each kind's button shows the unit with an **icon for its augments** and the count inside
//! (unreserved); it is greyed if there are none") show its free units of each kind ([`Conditions::pool_count`],
//! `Sim::pool_counts`), and are greyed while there are none of the kind, as the gate would refuse every entry of its
//! KindMenu (`Reject::NoRecruit`); a KindMenu's training and Release ask the gate (a Release with no free unit of the
//! kind left: "the second press is greyed"), and so does its Eject All (`Command::Holding`: nothing in the pool, "Only
//! available if StoredRecruits is not empty"). Its W, Train Gunner, asks the gate too, which refuses it while the player
//! owns no Gunworks (`Reject::Requires`, "Requires a Gunworks"; slice M5-G2).
//!
//! An Altar's unit type and state tiles (slice M5-N1b, `factions/cults/augmentation.md > Altar >
//! ObjectInterfaceState[Altar]`: "each tile shows the count of that unit type in StoredUnits; disabled if there are
//! none"; "Each tile shows the count of unreserved units of that type and state in StoredUnits; disabled if there are
//! none") show its free units of each type and state ([`Conditions::stored_count`], `Sim::pool_counts_of`), and are
//! greyed while there are none, as the gate would refuse every entry of their menus (`Reject::NoRecruit`; a type tile
//! says so of its first state's); its orders and Releases ask the gate ("Available only if the group still has an
//! unreserved unit, the AugmentationQueue is not full, and the player has sufficient Space Crystals").
//!
//! (M5-N3) A Temple's keys (`factions/cults/structures/temple.md > Placeholder stats`: "**Q** Enhanced Constitution
//! (next level), **W** Improved Metabolism (next level)") show what the gate says of each technology of their line
//! ([`Conditions::key`]): the first one not completed (`Reject::Researched`), "In progress" while it is queued or in
//! progress at any of the player's structures (`Reject::ResearchUnderWay`: "its key shows "In progress" meanwhile"),
//! nothing once both are completed; Level 2 greyed while the player owns no Cathedral (`Reject::Requires`), any of them
//! while the bank can't pay, with their price in the hint. Its releases of one Recruit of a kind count the staff of the
//! kind and are greyed while there are none, as the gate would refuse them (`Reject::NoRecruit`: "it is greyed if
//! there are none").
//!
//! (M5-N4b) A Disciple's R, Shroud (`factions/cults/units/disciple.md > Placeholder stats`), asks the gate for its
//! Shroud order, unqueued, to the one Disciple of the ActiveGroup: it is greyed below 10 energy
//! (`Reject::NotEnoughEnergy`: "needs at least **10 energy** to start").
//!
//! (M5-N5) A Shrine's Q and W (`factions/cults/structures/shrine.md > Placeholder stats`) ask the gate for their cast
//! wherever it would be cast (`Sim::can_cast`): greyed while the Shrine hasn't the energy it costs
//! (`Reject::NotEnoughEnergy`; N5's acceptance: "too little energy greys the command"), or while it is a site. Where
//! it is cast is the click's (`crate::command_ui::resolution::target_resolution`: a point out of vision is an invalid
//! target). The commands each button would send are `probes.rs`'.

mod modes;
mod probes;

use std::collections::BTreeMap;

use sc_sim::{Constructor, ObjectId, ObjectKind, PlayerId, RecruitKind, Reject, Sim};

use sc_sim::construction::site::BuildPrice;

pub use modes::Modes;

use super::{PanelCommand, Planned, commands, recipients};
use crate::selection::Selection;

/// Why a button is greyed out (design change 0162, `systems/controls/interface.md > CommandPanel > Pressing
/// an unavailable command`: "nothing is issued, and the player gets the same feedback as when an order is refused,
/// naming the reason"), from the check that greys it: what pressing it says (`crate::command_ui::feedback`).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum WhyUnavailable {
    /// The command gate would refuse the command it sends (`Sim::validate`): too few Space Crystals, a full queue,
    /// nothing to cancel, a constructor or a Beacon already busy.
    Refused(Reject),
    /// No recipient takes an Attack, and one of them is a Peacekeeper (Shielded) (`factions/gdo/units/peacekeeper.md >
    /// Riot Shield`: "the player must lower the shield before ordering an attack"; design change 0202).
    ShieldsUp,
    /// Another command awaits its target, or a building is placed (`systems/controls/interface.md >
    /// AwaitingTarget > Other commands while targeting`, design change 0066: "unavailable until the player cancels").
    Waiting,
    /// The type can't carry it out (`systems/controls/interface.md > BasicCombatUnitInterfaceState`: Reverse
    /// without CanReverse, AttackGround without CanTargetGround; (M5-N4a) Attack without a weapon, the Disciple's
    /// "AttackAttributes - None").
    NotForThisType,
    /// It isn't in the game yet: a building or a unit whose costs are TBD (design change 0124), a build menu none of
    /// whose buildings is, an order the sim has none of (Reverse).
    NotYet,
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
    /// queue (`sc_sim::Sim::network_counts`; `factions/syndicate/tunnel_menus.md > EjectMenu`: "the tile's count drops
    /// at the click"); none while the ActiveGroup isn't a Tunnel.
    network: BTreeMap<ObjectKind, u32>,
    /// The price of each Build the ActiveGroup offers (`PanelCommand::Build`), in its resource, as the sim would charge
    /// it now (`Sim::build_price`, design change 0233): a Tunnel's in Supplies, a Cults building's in Space Crystals
    /// (slice M5-J1); and of a Tunnel's Upgrade, in Supplies (`PanelCommand::UpgradeTunnel`, `Sim::upgrade_price`,
    /// slice M4-G).
    prices: BTreeMap<PanelCommand, BuildPrice>,
    /// Whether the ActiveGroup is a site under construction (`sc_sim::Object::is_site`; design change 0237: "a Tunnel
    /// site is a structure that does nothing but Cancel"), whose panel is its X Cancel Construction alone, whatever
    /// its type (`super::templates::SITE_GRID`).
    pub site: bool,
    /// The free units of each kind in the active pool building (`Sim::pool_counts`, by `RecruitKind as usize`): those
    /// no order reserves (slice M5-F2); (M5-N3) the staff of each kind in the ActiveGroup's Temples, which each let
    /// one out (`sc_sim::HoldCommand::ReleaseOne`; **Placeholder (not in the design: several Temples' counts are
    /// their sum)**); none while the ActiveGroup is neither.
    pool: [u32; 4],
    /// (M5-N1b) The free units of each type its tiles show in the active Altar, by kind (`Sim::pool_counts_of`); none
    /// while the ActiveGroup isn't an Altar.
    stored: BTreeMap<ObjectKind, [u32; 4]>,
}

impl Conditions {
    /// The conditions of `player`'s `selection` in `sim`: the modes, the ActiveGroup's constructor state, and
    /// `Sim::validate`'s answer to the `Command::Production`, `Command::Construction` or `Command::Ability` each such
    /// command the ActiveGroup may show would send ([`commands`]; `probes::refusal`). Raise and Lower Shield and Convert are never
    /// refused: the gate takes them whenever a recipient has the mode (design change 0198). The modes are those of
    /// the types the selection groups its objects as ([`Modes::of`]). A Warp In menu entry asks for its Warp In
    /// unturned, and leaves
    /// out the placement's refusal (`Reject::Unplaceable`, the gate's last check): the player turns the ghost before it
    /// is placed, and the ghost shows each tile's problem. An Agent's Build asks for its Build order, unqueued, the
    /// same way (`probes::build_probe`), and so does a Recruits' Construct menu entry, for the whole ActiveGroup (slice
    /// M5-J1: a group's Build, paid once, its TechPrerequisite and the bank the gate's). A Tunnel's EjectMenu tile and its Cancel ask for their `Command::Tunnel`, and the
    /// network's counts are the player's while the ActiveGroup opens the EjectMenu. An ExpandMenu entry asks
    /// `Sim::can_expand`, the gate's Expand short of its placement, for the same reason as a Warp In's. A pool
    /// building's Eject All asks for its `Command::Holding`, and its kind buttons count its free units. (M5-N3) A
    /// Temple's Research asks for its `Command::Production` of each technology the Temple offers, its key showing the
    /// answers ([`Conditions::key`]), priced at the technology's cost; its releases ask for their `Command::Holding` and
    /// count its staff. (M5-N4b) A Disciple's Shroud asks for its `Command::Order`, unqueued. (M5-N5) A Shrine's cast
    /// asks `Sim::can_cast`, and its releases count its participants as a Temple's count its staff.
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
        let kinds = PanelCommand::KindMenu(RecruitKind::Plain);
        let releases = PanelCommand::ReleaseOne(RecruitKind::Plain);
        let pool = if commands.contains(&releases) {
            let counts = recipients(selection, releases)
                .into_iter()
                .map(|temple| sim.pool_counts(temple));
            counts.fold([0; 4], |sum, counts| {
                std::array::from_fn(|kind| sum[kind] + counts[kind])
            })
        } else {
            (commands.contains(&kinds))
                .then(|| recipients(selection, kinds))
                .and_then(|buildings| buildings.first().copied())
                .map_or([0; 4], |building| sim.pool_counts(building))
        };
        let types = (commands.iter()).filter_map(|&command| match command {
            PanelCommand::AugmentMenu(unit) => Some((command, unit)),
            _ => None,
        });
        let stored = types
            .filter_map(|(command, unit)| {
                let altar = *recipients(selection, command).first()?;
                Some((unit, sim.pool_counts_of(altar, unit)))
            })
            .collect();
        let refused = (commands.iter())
            .filter_map(|&command| Some((command, probes::refusal(sim, player, selection, command)?)))
            .collect();
        let prices = (commands.iter())
            .filter_map(|&command| match command {
                PanelCommand::Build(kind) => Some((command, sim.build_price(player, kind)?)),
                PanelCommand::Research(tech) => Some((command, BuildPrice::crystals(tech.def().cost))),
                PanelCommand::UpgradeTunnel => {
                    let tunnel = *recipients(selection, command).first()?;
                    Some((command, BuildPrice::supplies(sim.upgrade_price(tunnel)?)))
                }
                _ => None,
            })
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
            pool,
            stored,
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
            pool: [0; 4],
            stored: BTreeMap::new(),
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

    /// Why placeholder entry `planned` is greyed (design change 0162): it isn't in the game yet. (The ExpandMenu's
    /// Laser Turret and the Armory's Gunner, which said their prerequisites, are in the game since slices M4-S2 and
    /// M5-G2, greyed by their gate: `Reject::Requires`.)
    pub fn planned(&self, _planned: Planned) -> WhyUnavailable {
        WhyUnavailable::NotYet
    }

    /// The same, with `counts` the active pool building's free units by kind: for the unit tests.
    pub fn with_pool(mut self, counts: [u32; 4]) -> Self {
        self.pool = counts;
        self
    }

    /// A kind button's count: the free units of `kind` in the active pool building, those no order reserves.
    pub fn pool_count(&self, kind: RecruitKind) -> u32 {
        self.pool[kind as usize]
    }

    /// The same, with `counts` the active Altar's free units of type `unit`, by kind: for the unit tests.
    pub fn with_stored(mut self, unit: ObjectKind, counts: [u32; 4]) -> Self {
        self.stored.insert(unit, counts);
        self
    }

    /// (M5-N1b) An Altar's type tile's count: its free units of type `unit`, of every kind.
    pub fn stored_count(&self, unit: ObjectKind) -> u32 {
        self.stored.get(&unit).map_or(0, |counts| counts.iter().sum())
    }

    /// (M5-N1b) An Altar's state tile's count: its free units of type `unit` with the Augmentations of kind `state`.
    pub fn stored_state_count(&self, unit: ObjectKind, state: RecruitKind) -> u32 {
        self.stored.get(&unit).map_or(0, |counts| counts[state as usize])
    }

    /// The same, with `command` priced at `price`: for the unit tests.
    pub fn pricing(mut self, command: PanelCommand, price: BuildPrice) -> Self {
        self.prices.insert(command, price);
        self
    }

    /// What a Build or a Tunnel's Upgrade costs now, in its resource (`Sim::build_price`, `Sim::upgrade_price`), and a
    /// Temple's Research, its technology's Space Crystals (slice M5-N3; `core/tech_and_research.md > Research > Shared
    /// Rules` 2: "The cost is deducted when the technology is queued"): `None` for another command, and for the Upgrade
    /// of a Tunnel at its top tier.
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

    /// (M5-N3) What the slot of `command` shows now: a Temple's key, which holds its line's Level 1, the line's next
    /// level, the first of its technologies the gate doesn't refuse as completed (`Reject::Researched`), and "In
    /// progress" while that one is queued or in progress (`Reject::ResearchUnderWay`: `PanelCommand::Researching`;
    /// `factions/cults/structures/temple.md > Placeholder stats`: "**Q** Enhanced Constitution (next level)", "its key
    /// shows "In progress" meanwhile"); nothing once the line is completed (`core/tech_and_research.md > Research >
    /// Shared Rules` 4: "Once completed, it no longer appears in any research menu"). Any other command itself.
    pub fn key(&self, command: PanelCommand) -> Option<PanelCommand> {
        let PanelCommand::Research(tech) = command else {
            return Some(command);
        };
        let research = PanelCommand::Research;
        let next = (tech.def().line.techs().into_iter())
            .find(|&tech| self.refusal(research(tech)) != Some(Reject::Researched(tech)))?;
        Some(match self.refusal(research(next)) {
            Some(Reject::ResearchUnderWay(_)) => PanelCommand::Researching(next),
            _ => research(next),
        })
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
    /// attack"; design change 0202: a Peacekeeper whose shield is on its way down takes it; (M5-N4a) a unit without a
    /// weapon, a Disciple, takes none: `sc_sim::combat::takes_attack_orders`, the gate's rule), a production or
    /// construction command, a Warp In or a toggle while the gate would take it, anything else always.
    pub fn allows(&self, selection: &Selection, command: PanelCommand) -> bool {
        self.unavailable(selection, command).is_none()
    }

    /// Why the game doesn't let `command` go to its recipients in `selection` now ([`Conditions::allows`]), if it
    /// doesn't: their types, their loads (a Drop Off none of whose recipients carries anything, the gate's
    /// `Reject::NothingToDeliver`), a pool building's free units (a kind button with none of its kind, which every
    /// entry of its KindMenu would want: `Reject::NoRecruit`), an Altar's (a type or state tile with none, said as the
    /// first state's, Unaugmented, for a type tile), a Temple's key in progress (`Reject::ResearchUnderWay`, which its
    /// Research would get), or the gate's refusal.
    pub fn unavailable(&self, selection: &Selection, command: PanelCommand) -> Option<WhyUnavailable> {
        match command {
            PanelCommand::Attack => {
                let recipients = recipients(selection, command);
                if recipients.iter().any(|&id| self.modes.can_attack(id)) {
                    None
                } else if recipients.iter().any(|&id| self.modes.shielded(id)) {
                    Some(WhyUnavailable::ShieldsUp)
                } else {
                    // (M5-N4a) Units without a weapon: a Disciple's Attack, greyed (its panel's "attack commands").
                    Some(WhyUnavailable::NotForThisType)
                }
            }
            PanelCommand::DropOff => {
                let loaded = recipients(selection, command)
                    .into_iter()
                    .any(|id| self.modes.carries(id));
                (!loaded).then_some(WhyUnavailable::Refused(Reject::NothingToDeliver))
            }
            PanelCommand::KindMenu(kind) => {
                (self.pool_count(kind) == 0).then_some(WhyUnavailable::Refused(Reject::NoRecruit(kind)))
            }
            PanelCommand::AugmentMenu(unit) => {
                (self.stored_count(unit) == 0).then_some(WhyUnavailable::Refused(Reject::NoRecruit(RecruitKind::Plain)))
            }
            PanelCommand::AugmentState(unit, state) => {
                (self.stored_state_count(unit, state) == 0).then_some(WhyUnavailable::Refused(Reject::NoRecruit(state)))
            }
            PanelCommand::Researching(tech) => Some(WhyUnavailable::Refused(Reject::ResearchUnderWay(tech))),
            _ => self.refusal(command).map(WhyUnavailable::Refused),
        }
    }
}
