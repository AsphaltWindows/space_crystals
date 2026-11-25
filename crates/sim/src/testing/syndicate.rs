//! Scripted Syndicate lines (slice M4-L): a sibling of `testing::build_order` and `testing::colonists` that plays a
//! Syndicate player through the gate (Train, Eject, Gather, Gather Supplies, Build Tunnel, Expand) with books for both
//! of its banks, and the reference layouts that the Syndicate's economy targets are measured on
//! (`factions/syndicate/units/agent.md > Gathering > Space Crystals > Syndicate economy check`, design changes 0259
//! and 0294).
//!
//! A [`Syndicate`] line plays a Syndicate player the way a player would, with their own commands through the gate,
//! never with debug commands. Before each step, [`Syndicate::commands`] looks at the game as the last tick left it and
//! gives the player's commands for the next tick:
//! 1. **Gathering** ([`Syndicate::gather`]): at the start, the player's Agents are sent to one patch with one command,
//!    as one right-click sends a selection. They share it (`factions/syndicate/units/agent.md > Gathering > Space
//!    Crystals`: "any number of Agents may mine the same Space Crystals patch at the same time").
//! 2. **Rally points** ([`Syndicate::rally`]): each Hideout gets the line's rally point once it can produce, the
//!    starting one at the start and an expanded one once it is complete. A unit it makes leaves the network by its
//!    parent Tunnel and does what a right-click on the target would (`factions/syndicate/structures/README.md >
//!    Tunnel Expansions > Rally Point Behavior`): an Agent rallied on a patch gathers it.
//! 3. **Jobs** ([`Syndicate::job`]): the player's `n`th Agent (the starting ones by id, then those the Hideouts make,
//!    in the order they are finished) gets a [`Job`] once it is on the map and the job can start: gathering Supplies
//!    at a station (`factions/syndicate/units/agent.md > Gathering > Supplies`), or building a Tunnel on a site once
//!    the Supplies bank covers its price (`Sim::build_price`, `factions/syndicate/tunnels.md > Cost Scaling >
//!    Construction Cost`) and the build check takes the site.
//! 4. **Ejecting** ([`Syndicate::eject_builders`]): an Agent that finishes a Tunnel goes into the Tunnel Network
//!    (`factions/syndicate/units/agent.md > Building > Tunnel Construction Flow`). The line ejects it by that Tunnel
//!    (`TunnelCommand::Eject`, `factions/syndicate/tunnels.md > EjectMenu`) and gives it a job once it is out.
//! 5. **Expanding** ([`Syndicate::expand`]): the line's expansions, in order, each built by the player's `n`th Tunnel
//!    (by id, the starting one first) once the Tunnel may build it now (`Sim::can_expand`: in operation, with no
//!    operation under way, its prerequisites met), the bank covers it after the build order's orders of the tick, and
//!    the expansion check takes its site (`TunnelCommand::Expand`, `factions/syndicate/tunnels.md > AwaitingPlacement
//!    (Expansion)`). **Upgrading** ([`Syndicate::upgrade`]) likewise: the line's upgrades, in order, each of the
//!    player's `n`th Tunnel once the gate would take it (`Sim::can_upgrade`: in operation, below its top tier, with no
//!    operation under way, its larger area clear by the truth) and the Supplies left after the tick's Builds cover its
//!    price (`Sim::upgrade_price`, `factions/syndicate/tunnels.md > Cost Scaling > Upgrade Costs`), as the Tunnel's
//!    one operation of the tick (`TunnelCommand::Upgrade`, slice M4-G).
//! 6. **What the player trains** goes through a [`BuildOrder`]: Agents at the first Hideout. Units behind a
//!    prerequisite ([`Syndicate::after`]) are trained by the line itself, in order, each while the player owns a
//!    complete structure of that kind, as the Hideout's panel offers the unit only then, and once the bank covers it
//!    and its producer (the player's first of its kind, by id) has nothing queued: the Guards behind an Optics Lab
//!    (`factions/syndicate/structures/hideout.md > ObjectInterfaceState[Hideout]`: "Only available if the player owns
//!    at least one Optics Lab"). A prerequisite lost stops them, as the gate would refuse them.
//!
//! The build order gets the Space Crystals first, then the units behind prerequisites, then the expansions; the Builds
//! get the Supplies first, then the upgrades. [`Books`](super::build_order::Books) balance the player's Space
//! Crystals, counting the Agents' deliveries; [`SupplyBooks`] balance their Supplies.
//!
//! [`reference_field`], [`stacked_field`] and [`opening_field`] are the check's reference layouts (`fields.rs`).
//! [`skirmish`] plays `dev_skirmish` with a Syndicate line, alone or against a GDO one, for the golden games.

mod books;
mod fields;
pub mod skirmish;

use std::collections::BTreeSet;

use crate::command::{Command, PlayerCommand, TunnelCommand};
use crate::construction::site::expansion_def;
use crate::defs::ObjectKind;
use crate::events::SimEvent;
use crate::ids::{ObjectId, PlayerId};
use crate::orders::{OrderKind, Target};
use crate::production::ProductionCommand;
use crate::scenario::BuildingOrientation;
use crate::sim::Sim;
use crate::units::{Tick, TilePos};

use super::build_order::{BuildOrder, Order};

pub use books::SupplyBooks;
pub use fields::{
    FIELD_HIDEOUT, FIELD_PATCH, FIELD_PLAYER, FIELD_RESERVE, FIELD_TUNNEL, STACKED_PATCH, STACKED_TUNNEL, field_agents,
    opening_field, reference_field, stacked_agents, stacked_field,
};

/// What the line sets one of the player's Agents to ([`Syndicate::job`]).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Job {
    /// Gather Supplies at this Supply Delivery Station.
    Supplies(ObjectId),
    /// Gather this patch.
    Crystals(ObjectId),
    /// Build a Tunnel with the top-left tile of its footprint here, turned so.
    Tunnel(TilePos, BuildingOrientation),
}

/// An expansion of the line's ([`Syndicate::expand`]): the player's `tunnel`th Tunnel builds a `kind` with the
/// top-left tile of its footprint at `origin`, turned by `orientation`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Expansion {
    pub tunnel: usize,
    pub kind: ObjectKind,
    pub origin: TilePos,
    pub orientation: BuildingOrientation,
}

/// Something the line did, with the tick its command applies in.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Done {
    /// It gave Agent `agent` (its `n`th) `job`.
    Job { n: usize, agent: ObjectId, job: Job },
    /// It ordered an eject of a `kind` by Tunnel `tunnel`.
    Eject { tunnel: ObjectId, kind: ObjectKind },
    /// Tunnel `tunnel` started its expansion `expansion`.
    Expand { tunnel: ObjectId, expansion: Expansion },
    /// Tunnel `tunnel` started its upgrade, for `price` Supplies.
    Upgrade { tunnel: ObjectId, price: u32 },
    /// Producer `by` was ordered to train a unit behind a prerequisite.
    Train { by: ObjectId, order: Order },
}

/// A Syndicate player's scripted line (module docs).
#[derive(Clone, Debug)]
pub struct Syndicate {
    line: BuildOrder,
    /// The patch the starting Agents gather, if they do.
    gather: Option<ObjectId>,
    /// Every Hideout's rally point, if the line sets one.
    rally: Option<Target>,
    /// Each job, by the Agent it goes to (its place in [`Syndicate::agents`]).
    jobs: Vec<(usize, Job)>,
    /// The job of an Agent that comes out of the Tunnel it built, if the line ejects it.
    after_build: Option<Job>,
    expansions: Vec<Expansion>,
    /// The upgrades still to start, in order, each by the place of its Tunnel among the player's ([`Syndicate::upgrade`]).
    upgrades: Vec<usize>,
    /// The units behind a prerequisite still to train, in order, each with its prerequisite.
    gated: Vec<(ObjectKind, Order)>,
    started: bool,
    /// The player's Agents in order: the starting ones by id, then the made ones as they are finished.
    agents: Vec<ObjectId>,
    rallied: BTreeSet<ObjectId>,
    /// The Agents that built a Tunnel and are to come out of it, by that Tunnel.
    builders: Vec<(ObjectId, ObjectId)>,
    done: Vec<(Tick, Done)>,
    /// What the line's own commands paid, by tick: Space Crystals and Supplies.
    paid: Vec<(Tick, u32, u32)>,
}

impl Syndicate {
    /// A line for `line`'s player that pays for its units through `line`, and does nothing else yet.
    pub fn new(line: BuildOrder) -> Self {
        Self {
            line,
            gather: None,
            rally: None,
            jobs: Vec::new(),
            after_build: None,
            expansions: Vec::new(),
            upgrades: Vec::new(),
            gated: Vec::new(),
            started: false,
            agents: Vec::new(),
            rallied: BTreeSet::new(),
            builders: Vec::new(),
            done: Vec::new(),
            paid: Vec::new(),
        }
    }

    /// The player's Agents gather `patch` from the start, sent by one command.
    pub fn gather(mut self, patch: ObjectId) -> Self {
        self.gather = Some(patch);
        self
    }

    /// Every Hideout of the player's gets `target` as its rally point once it can produce.
    pub fn rally(mut self, target: Target) -> Self {
        self.rally = Some(target);
        self
    }

    /// The player's `n`th Agent (module docs, 3) gets `job` once it is on the map and the job can start.
    pub fn job(mut self, n: usize, job: Job) -> Self {
        self.jobs.push((n, job));
        self
    }

    /// An Agent that finishes a Tunnel is ejected by it, and gets `job` once it is out (module docs, 4).
    pub fn eject_builders(mut self, job: Job) -> Self {
        self.after_build = Some(job);
        self
    }

    /// The player's `tunnel`th Tunnel builds an expansion, after the line's earlier ones (module docs, 5).
    pub fn expand(mut self, tunnel: usize, kind: ObjectKind, origin: TilePos) -> Self {
        let orientation = BuildingOrientation::default();
        self.expansions.push(Expansion {
            tunnel,
            kind,
            origin,
            orientation,
        });
        self
    }

    /// The player's `tunnel`th Tunnel upgrades to the tier above its own, after the line's earlier upgrades (module
    /// docs, 5).
    pub fn upgrade(mut self, tunnel: usize) -> Self {
        self.upgrades.push(tunnel);
        self
    }

    /// The units `orders` train, after the line's earlier ones of this kind, each while the player owns a complete
    /// `prerequisite` (module docs, 6).
    pub fn after(mut self, prerequisite: ObjectKind, orders: impl IntoIterator<Item = Order>) -> Self {
        self.gated.extend(orders.into_iter().map(|order| (prerequisite, order)));
        self
    }

    pub fn player(&self) -> PlayerId {
        self.line.player()
    }

    /// The build order that pays for the player's units.
    pub fn line(&self) -> &BuildOrder {
        &self.line
    }

    /// The player's Agents so far, in the order the jobs count them.
    pub fn agents(&self) -> &[ObjectId] {
        &self.agents
    }

    /// Everything the line did besides training, in order.
    pub fn done(&self) -> &[(Tick, Done)] {
        &self.done
    }

    /// What the line's orders of the step to tick `tick` paid, in Space Crystals.
    pub fn spent_in(&self, tick: Tick) -> u32 {
        let trained: u32 = (self.line.issued().iter())
            .filter(|i| i.tick == tick)
            .map(|i| i.cost)
            .sum();
        trained + self.paid_in(tick).0
    }

    /// What the line's orders of the step to tick `tick` paid in Supplies.
    pub fn supplies_spent_in(&self, tick: Tick) -> u32 {
        self.paid_in(tick).1
    }

    fn paid_in(&self, tick: Tick) -> (u32, u32) {
        let paid = self.paid.iter().filter(|&&(t, _, _)| t == tick);
        paid.fold((0, 0), |(c, s), &(_, crystals, supplies)| (c + crystals, s + supplies))
    }

    /// The player's commands for the step to tick `sim.tick() + 1`, from the game as it is now (module docs). Nothing
    /// once the game is over or the player is defeated, as the gate would refuse it.
    pub fn commands(&mut self, sim: &Sim) -> Vec<PlayerCommand> {
        let tick = sim.tick() + 1;
        let player = self.player();
        let mut commands = Vec::new();
        let Some(me) = sim.player(player).filter(|p| !p.is_defeated()) else {
            return commands;
        };
        if sim.outcome().is_some() {
            return commands;
        }
        let (mut crystals, mut supplies) = (me.crystals, me.supplies);
        let command = |command| PlayerCommand { player, command };
        let own = || sim.objects().filter(move |o| o.owner == Some(player));
        let complete = |o: &&crate::object::Object| o.construction.is_none();

        if !self.started {
            self.started = true;
            self.agents = own().filter(|o| o.kind == ObjectKind::Agent).map(|o| o.id).collect();
            if let Some(patch) = self.gather
                && !self.agents.is_empty()
            {
                commands.push(command(Command::Order {
                    units: self.agents.clone(),
                    kind: OrderKind::Gather(patch),
                    queue: false,
                }));
            }
        }
        for e in sim.events() {
            match *e {
                SimEvent::UnitProduced { unit, .. } => {
                    let held = sim.held_unit(unit).map(|held| &held.object);
                    let made = sim.object(unit).or(held);
                    if made.is_some_and(|o| o.owner == Some(player) && o.kind == ObjectKind::Agent) {
                        self.agents.push(unit);
                    }
                }
                SimEvent::SiteCompleted { site, worker }
                    if self.after_build.is_some() && sim.object(site).is_some_and(|o| o.owner == Some(player)) =>
                {
                    let kind = ObjectKind::Agent;
                    commands.push(command(Command::Tunnel {
                        tunnel: site,
                        command: TunnelCommand::Eject { kind },
                    }));
                    self.builders.push((site, worker));
                    self.done.push((tick, Done::Eject { tunnel: site, kind }));
                }
                _ => {}
            }
        }
        // A builder out of its Tunnel gets its job.
        if let Some(job) = self.after_build {
            let out: Vec<ObjectId> = (self.builders.iter())
                .filter(|&&(_, worker)| sim.object(worker).is_some())
                .map(|&(_, worker)| worker)
                .collect();
            self.builders.retain(|(_, worker)| !out.contains(worker));
            for agent in out {
                let n = self.agents.iter().position(|&a| a == agent).unwrap_or(usize::MAX);
                if let Some(order) = self.job_order(sim, agent, job, &mut supplies) {
                    commands.push(command(order));
                    self.done.push((tick, Done::Job { n, agent, job }));
                }
            }
        }

        if let Some(target) = self.rally {
            for o in own().filter(|o| o.kind == ObjectKind::Hideout && o.production.is_some()) {
                if o.construction.is_none() && self.rallied.insert(o.id) {
                    commands.push(command(Command::Production {
                        objects: vec![o.id],
                        command: ProductionCommand::SetRally(target),
                    }));
                }
            }
        }

        for (n, job) in std::mem::take(&mut self.jobs) {
            let agent = self.agents.get(n).copied().filter(|&agent| sim.object(agent).is_some());
            match agent.and_then(|agent| Some((agent, self.job_order(sim, agent, job, &mut supplies)?))) {
                Some((agent, order)) => {
                    commands.push(command(order));
                    self.done.push((tick, Done::Job { n, agent, job }));
                }
                None => self.jobs.push((n, job)),
            }
        }

        let trained = self.line.commands(sim);
        let issued: Vec<_> = self.line.issued().iter().filter(|i| i.tick == tick).collect();
        crystals -= issued.iter().map(|i| i.cost).sum::<u32>();
        let busy: BTreeSet<ObjectId> = issued.iter().map(|i| i.by).collect();
        commands.extend(trained);

        // The next unit behind a prerequisite, while the player owns one and its producer is free.
        if let Some(&(prerequisite, order)) = self.gated.first()
            && own().filter(complete).any(|o| o.kind == prerequisite)
            && let Some(by) = own().filter(|o| o.kind == order.by).min_by_key(|o| o.id)
            && !busy.contains(&by.id)
            && !by.is_transforming()
            && by.production.as_ref().is_some_and(|p| p.queue.is_empty())
            && order.recipe().cost <= crystals
        {
            let cost = order.recipe().cost;
            crystals -= cost;
            commands.push(command(Command::Production {
                objects: vec![by.id],
                command: ProductionCommand::Train(order.kind),
            }));
            self.paid.push((tick, cost, 0));
            self.done.push((tick, Done::Train { by: by.id, order }));
            self.gated.remove(0);
        }

        // The next expansion, if its Tunnel may build it now and the bank covers it.
        let mut tunnels: Vec<ObjectId> = own().filter(|o| o.kind == ObjectKind::Tunnel).map(|o| o.id).collect();
        tunnels.sort_unstable();
        let mut expanding = None;
        if let Some(&expansion) = self.expansions.first() {
            let cost = expansion_def(expansion.kind).map_or(u32::MAX, |def| def.cost);
            let tunnel = tunnels.get(expansion.tunnel).copied();
            let site_ok = |tunnel| {
                (sim.expansion_check(tunnel, expansion.kind, expansion.origin, expansion.orientation))
                    .is_some_and(|check| check.is_valid())
            };
            if let Some(tunnel) = tunnel
                && cost <= crystals
                && sim.can_expand(player, tunnel, expansion.kind).is_ok()
                && site_ok(tunnel)
            {
                commands.push(command(Command::Tunnel {
                    tunnel,
                    command: TunnelCommand::Expand {
                        kind: expansion.kind,
                        origin: expansion.origin,
                        orientation: expansion.orientation,
                    },
                }));
                self.paid.push((tick, cost, 0));
                self.done.push((tick, Done::Expand { tunnel, expansion }));
                self.expansions.remove(0);
                expanding = Some(tunnel);
            }
        }

        // The next upgrade, if the gate would take it and the Supplies left cover it: never in the tick its Tunnel
        // starts an expansion, its one operation.
        if let Some(&n) = self.upgrades.first()
            && let Some(tunnel) = tunnels.get(n).copied().filter(|&tunnel| expanding != Some(tunnel))
            && let Some(price) = sim.upgrade_price(tunnel).filter(|&price| price <= supplies)
            && sim.can_upgrade(player, tunnel).is_ok()
        {
            commands.push(command(Command::Tunnel {
                tunnel,
                command: TunnelCommand::Upgrade,
            }));
            self.paid.push((tick, 0, price));
            self.done.push((tick, Done::Upgrade { tunnel, price }));
            self.upgrades.remove(0);
        }
        commands
    }

    /// The order that sets `agent` to `job` now, if it can start: a Build needs the Supplies its price takes, out of
    /// `supplies`, and a site the build check takes. Records what a Build pays.
    fn job_order(&mut self, sim: &Sim, agent: ObjectId, job: Job, supplies: &mut u32) -> Option<Command> {
        let kind = match job {
            Job::Supplies(station) => OrderKind::Gather(station),
            Job::Crystals(patch) => OrderKind::Gather(patch),
            Job::Tunnel(origin, orientation) => {
                let tunnel = ObjectKind::Tunnel;
                let price = sim.build_price(self.player(), tunnel)?;
                let check = sim.build_check(agent, tunnel, origin, orientation, false)?;
                if price > *supplies || !check.is_valid() {
                    return None;
                }
                *supplies -= price;
                self.paid.push((sim.tick() + 1, 0, price));
                OrderKind::Build {
                    kind: tunnel,
                    origin,
                    orientation,
                }
            }
        };
        Some(Command::Order {
            units: vec![agent],
            kind,
            queue: false,
        })
    }
}
