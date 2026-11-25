//! `dev_skirmish` played by script from its real start (slice M4-L): the golden games `syndicate_opening` and
//! `syndicate_vs_gdo` (`tests/golden/syndicate.rs`), and longer runs of the same games. Player 1 plays the Syndicate
//! from slot 1's Syndicate Starting Layout (a Tunnel with its Hideout, one Agent, 100 Space Crystals and 1 Supply,
//! `factions/syndicate/tunnels.md > Starting Conditions`) as a [`Syndicate`] line; player 2 plays the GDO from slot
//! 2's GDO Starting Layout as a [`BuildOrder`], or does nothing. Both give their own commands, which the gate takes,
//! as players would:
//! - [`Skirmish::opening`]: player 1's Agents gather [`PATCH`], 3 grid units from the crystal hatch, and its Hideout,
//!   rallied on it, trains [`AGENTS`] more. The third of them ([`BUILDER`]) builds a second Tunnel at [`TUNNEL_SITE`]
//!   with the starting Supply, is ejected by it once it is built, and gathers Supplies at the map's station
//!   ([`STATION`]). The first Tunnel expands an Optics Lab ([`OPTICS_LAB_SITE`]). Player 2 does nothing.
//! - [`Skirmish::versus_gdo`]: the same, and then [`GUARDS`] Guards, which the Hideout trains only while the player
//!   owns the Optics Lab (`factions/syndicate/structures/hideout.md > Produces`: "Only available if the player owns
//!   at least one Optics Lab"). With its rally point cleared, they stay in the Tunnel Network; the first
//!   [`GUARDS_OUT`] are ejected by the second Tunnel, the forward one, and hold the top of the plateau's west ramp
//!   above it ([`GUARD_POST`]), and the rest wait in the network. Player 2 plays the Colonists' games' Barracks line
//!   (the fastest clean Barracks) with [`PEACEKEEPERS`] Peacekeepers, which gather at home ([`GDO_STAGING`]) and
//!   attack-move in waves of [`WAVE`] to the Syndicate's Tunnel nearest it, fighting what they meet on the way; a
//!   Peacekeeper whose Tunnel has fallen goes on to the next. The script knows where the Tunnels are, as a player who
//!   scouted would.
//!
//! Nothing here assumes the game goes as the goldens' do: the players stop once the game is over or they are
//! defeated, a lost Optics Lab stops the Guards (`Syndicate::after`), and the Guards come out only while the forward
//! Tunnel stands.

use std::collections::BTreeMap;

use crate::command::{Command, PlayerCommand, TunnelCommand};
use crate::defs::ObjectKind;
use crate::events::SimEvent;
use crate::ids::{ObjectId, PlayerId};
use crate::object::{Object, Rotation};
use crate::orders::{OrderKind, Target};
use crate::player::Faction;
use crate::production::ProductionCommand;
use crate::scenario::{BuildingOrientation, Scenario};
use crate::scenarios;
use crate::sim::Sim;
use crate::testing::build_order::{BuildOrder, Order};
use crate::units::{Tick, TilePos};

use super::{Done, Job, Syndicate};

use ObjectKind::{
    Agent, Barracks, DeploymentCenter, ExtractionFacility, ExtractionPlate, Generator, Guard, Hideout, OpticsLab,
    Peacekeeper, PeacekeeperShielded, Tunnel,
};

/// The Syndicate's player, in slot 1.
pub const SYNDICATE: PlayerId = PlayerId(1);
/// The GDO's, in slot 2.
pub const GDO: PlayerId = PlayerId(2);
/// `dev_skirmish`'s patch on tile (2, 7): of slot 1's field (the map's first spawns, in its order), the one in the row
/// of slot 1's crystal hatch (6, 7), 3 grid units from it edge to edge (design change 0294). The starting Agent is
/// sent to it, and the Hideout is rallied on it.
pub const PATCH: ObjectId = ObjectId(3);
/// `dev_skirmish`'s Supply Delivery Station, on the plateau in the middle of the map: the spawn after slot 1's field.
pub const STATION: ObjectId = ObjectId(9);
/// How many Agents the Hideout trains, one after the other.
pub const AGENTS: usize = 4;
/// The Agent that builds the second Tunnel, by its place among the player's Agents (`Syndicate::job`): the starting
/// one is 0, so this is the third the Hideout trains.
pub const BUILDER: usize = 3;
/// The second Tunnel's site, the top-left tile of its footprint: (20, 16) to (23, 19), on flat Plane between player
/// 1's base and the plateau's west ramp, turned as the first ([`TURNED`]): Side A south, the Supplies hatch (Side D)
/// east, toward the station. Its Tier 1 area, (18, 14) to (25, 21), lies on Plane, clear of the first Tunnel's
/// area (4, 4) to (11, 11), the lake and the cliffs.
pub const TUNNEL_SITE: TilePos = TilePos::new(20, 16);
/// How player 1's Tunnels are turned: Side A south (`scenarios::dev_skirmish`).
pub const TURNED: BuildingOrientation = BuildingOrientation {
    rotation: Rotation::R180,
    flip_h: false,
    flip_v: false,
};
/// The Optics Lab's site, the top-left tile of its 2×2 footprint, in the first Tunnel's area: (10, 4) to (11, 5), its
/// north-east corner, off the Agents' way between Side A, the crystal hatch and the field.
pub const OPTICS_LAB_SITE: TilePos = TilePos::new(10, 4);
/// [`Skirmish::versus_gdo`]: the Guards the Hideout trains once the Optics Lab stands.
pub const GUARDS: usize = 6;
/// [`Skirmish::versus_gdo`]: how many of them are ejected by the forward Tunnel; the rest stay in the network.
pub const GUARDS_OUT: usize = 4;
/// [`Skirmish::versus_gdo`]: where the Guards out of the forward Tunnel wait: on the plateau, at the top of its west
/// ramp (the gap in its cliffs on tiles (26, 22) and (26, 23)), on the Peacekeepers' way. Below the ramp the
/// Peacekeepers, on the high ground, would outrange and outsee them (`core/vision.md > ElevationModifier`).
pub const GUARD_POST: TilePos = TilePos::new(28, 23);
/// [`Skirmish::versus_gdo`]: player 2's Barracks' rally point, on open ground north-west of its base, where its
/// Peacekeepers wait for their wave (the Colonists' games' too).
pub const GDO_STAGING: TilePos = TilePos::new(50, 36);
/// [`Skirmish::versus_gdo`]: the Peacekeepers player 2 trains.
pub const PEACEKEEPERS: usize = 16;
/// [`Skirmish::versus_gdo`]: player 2's Peacekeepers set off once this many of them wait at [`GDO_STAGING`].
pub const WAVE: usize = 6;

/// `dev_skirmish` with player 1 playing the Syndicate (slot 1) and player 2 the GDO (slot 2).
pub fn scenario() -> Scenario {
    let mut scenario = scenarios::dev_skirmish();
    scenario.setup.players[0].faction = Faction::Syndicate;
    scenario
}

/// The players of a [`scenario`] game (module docs).
#[derive(Clone, Debug)]
pub struct Skirmish {
    syndicate: Syndicate,
    gdo: Option<BuildOrder>,
    /// Whether the Hideout's rally point was cleared for the Guards.
    cleared: bool,
    /// How many Guards were ordered out by the forward Tunnel.
    ejected: usize,
    /// Whether player 2's Barracks has its rally point.
    rallied: bool,
    /// Player 2's Peacekeepers sent with a wave, each with the Tunnel it was sent to.
    sent: BTreeMap<ObjectId, ObjectId>,
}

impl Skirmish {
    /// The Syndicate's opening (module docs): four Agents, the second Tunnel, Supplies, the Optics Lab.
    pub fn opening() -> Self {
        Self::new(syndicate(0), None)
    }

    /// The Syndicate against the GDO (module docs): the opening and [`GUARDS`] Guards, against player 2's Barracks
    /// line and its [`PEACEKEEPERS`] Peacekeepers in waves.
    pub fn versus_gdo() -> Self {
        let gdo = BuildOrder::new(GDO).track(
            [
                Order::new(ExtractionFacility, ExtractionPlate),
                Order::new(DeploymentCenter, Generator),
                Order::new(DeploymentCenter, Generator),
                Order::new(DeploymentCenter, Barracks),
            ]
            .into_iter()
            .chain([Order::new(Barracks, Peacekeeper); PEACEKEEPERS]),
        );
        Self::new(syndicate(GUARDS), Some(gdo))
    }

    fn new(syndicate: Syndicate, gdo: Option<BuildOrder>) -> Self {
        Self {
            syndicate,
            gdo,
            cleared: false,
            ejected: 0,
            rallied: false,
            sent: BTreeMap::new(),
        }
    }

    /// Player 1's line.
    pub fn syndicate(&self) -> &Syndicate {
        &self.syndicate
    }

    /// Player 2's build order, if player 2 plays.
    pub fn gdo(&self) -> Option<&BuildOrder> {
        self.gdo.as_ref()
    }

    /// What `player`'s orders of the step to tick `tick` paid in Space Crystals: what the step takes from their bank,
    /// as nothing else the players do costs any.
    pub fn spent_in(&self, player: PlayerId, tick: Tick) -> u32 {
        if player == SYNDICATE {
            return self.syndicate.spent_in(tick);
        }
        let issued = self.gdo.iter().flat_map(|line| line.issued());
        issued.filter(|i| i.tick == tick).map(|i| i.cost).sum()
    }

    /// What player 1's orders of the step to tick `tick` paid in Supplies.
    pub fn supplies_spent_in(&self, tick: Tick) -> u32 {
        self.syndicate.supplies_spent_in(tick)
    }

    /// Both players' commands for the step to tick `sim.tick() + 1`, from the game as it is now (module docs).
    pub fn commands(&mut self, sim: &Sim) -> Vec<PlayerCommand> {
        let tick = sim.tick() + 1;
        let mut commands = self.syndicate.commands(sim);
        let plays = |player| sim.outcome().is_none() && sim.player(player).is_some_and(|p| !p.is_defeated());
        if self.gdo.is_some() && plays(SYNDICATE) {
            self.guards(sim, tick, &mut commands);
        }
        if let Some(line) = &mut self.gdo {
            commands.extend(line.commands(sim));
        }
        if self.gdo.is_some() && plays(GDO) {
            self.peacekeepers(sim, &mut commands);
        }
        commands
    }

    /// Player 1's Guards (module docs): the Hideout's rally point cleared as the first is ordered, so that they stay
    /// in the network; the first [`GUARDS_OUT`] ejected by the forward Tunnel while it is in operation, one a tick,
    /// each sent to [`GUARD_POST`] once out.
    fn guards(&mut self, sim: &Sim, tick: Tick, commands: &mut Vec<PlayerCommand>) {
        let command = |command| PlayerCommand {
            player: SYNDICATE,
            command,
        };
        let first = (self.syndicate.done().iter()).find_map(|&(at, done)| match done {
            Done::Train { by, order } if at == tick && order.kind == Guard => Some(by),
            _ => None,
        });
        if !self.cleared
            && let Some(hideout) = first
        {
            // The producer itself as the target clears its rally point (design change 0086).
            commands.push(command(Command::Production {
                objects: vec![hideout],
                command: ProductionCommand::SetRally(Target::Object(hideout)),
            }));
            self.cleared = true;
        }
        let forward = (sim.objects()).find(|o| {
            o.owner == Some(SYNDICATE)
                && o.kind == Tunnel
                && o.is_operational()
                && o.placement.is_some_and(|p| p.origin == TUNNEL_SITE)
        });
        let free = sim.network_counts(SYNDICATE).get(&Guard).copied().unwrap_or(0);
        if self.ejected < GUARDS_OUT
            && free > 0
            && let Some(forward) = forward
        {
            commands.push(command(Command::Tunnel {
                tunnel: forward.id,
                command: TunnelCommand::Eject { kind: Guard },
            }));
            self.ejected += 1;
        }
        let out: Vec<ObjectId> = (sim.events().iter())
            .filter_map(|e| match *e {
                SimEvent::LeftNetwork { unit, .. } => Some(unit),
                _ => None,
            })
            .filter(|&unit| {
                sim.object(unit)
                    .is_some_and(|o| o.owner == Some(SYNDICATE) && o.kind == Guard)
            })
            .collect();
        if !out.is_empty() {
            commands.push(command(Command::Order {
                units: out,
                kind: OrderKind::AttackMove(GUARD_POST.center()),
                queue: false,
            }));
        }
    }

    /// Player 2's Peacekeepers (module docs): the Barracks rallied to [`GDO_STAGING`]; a wave of the idle ones that
    /// never set off once there are [`WAVE`] of them, and at once each sent one that is idle and whose Tunnel has
    /// fallen, attack-moving to the centre of player 1's Tunnel nearest the staging point.
    fn peacekeepers(&mut self, sim: &Sim, commands: &mut Vec<PlayerCommand>) {
        let command = |command| PlayerCommand { player: GDO, command };
        let own = || sim.objects().filter(|o| o.owner == Some(GDO));
        if !self.rallied
            && let Some(barracks) = own().find(|o| o.kind == Barracks)
        {
            commands.push(command(Command::Production {
                objects: vec![barracks.id],
                command: ProductionCommand::SetRally(Target::Point(GDO_STAGING.center())),
            }));
            self.rallied = true;
        }
        let staging = GDO_STAGING.center();
        let Some(target) = (sim.objects())
            .filter(|o| o.owner == Some(SYNDICATE) && o.kind == Tunnel)
            .min_by(|a, b| (a.pos.distance_squared(staging)).total_cmp(&b.pos.distance_squared(staging)))
        else {
            return;
        };
        let idle = |o: &&Object| {
            matches!(o.kind, Peacekeeper | PeacekeeperShielded)
                && o.orders.as_ref().is_none_or(|orders| orders.is_idle())
        };
        let standing = |tunnel: &ObjectId| sim.object(*tunnel).is_some_and(|o| o.owner == Some(SYNDICATE));
        let again = own()
            .filter(idle)
            .filter(|o| self.sent.get(&o.id).is_some_and(|tunnel| !standing(tunnel)));
        let waiting: Vec<ObjectId> = own()
            .filter(idle)
            .filter(|o| !self.sent.contains_key(&o.id))
            .map(|o| o.id)
            .collect();
        let mut going: Vec<ObjectId> = again.map(|o| o.id).collect();
        if waiting.len() >= WAVE {
            going.extend(waiting);
        }
        if going.is_empty() {
            return;
        }
        going.sort_unstable();
        for &unit in &going {
            self.sent.insert(unit, target.id);
        }
        commands.push(command(Command::Order {
            units: going,
            kind: OrderKind::AttackMove(target.pos),
            queue: false,
        }));
    }
}

/// Player 1's line: the starting Agent sent to [`PATCH`] and the Hideout rallied on it; [`AGENTS`] Agents trained
/// one after the other; the [`BUILDER`]th building the second Tunnel at [`TUNNEL_SITE`], ejected by it once built,
/// and gathering Supplies at [`STATION`]; the first Tunnel's Optics Lab; then `guards` Guards behind it. The Agents get
/// the money first, then the Guards, then the Optics Lab.
fn syndicate(guards: usize) -> Syndicate {
    let line = BuildOrder::new(SYNDICATE).track(vec![Order::new(Hideout, Agent); AGENTS]);
    Syndicate::new(line)
        .gather(PATCH)
        .rally(Target::Object(PATCH))
        .job(BUILDER, Job::Tunnel(TUNNEL_SITE, TURNED))
        .eject_builders(Job::Supplies(STATION))
        .expand(0, OpticsLab, OPTICS_LAB_SITE)
        .after(OpticsLab, vec![Order::new(Hideout, Guard); guards])
}
