//! `dev_skirmish` played by script from its real start (slice M3-K1): the golden games `colonists_opening` and
//! `colonists_vs_gdo` (`tests/golden/colonists.rs`), and longer runs of the same games. Player 1 plays the Colonists
//! from slot 1's Colonists Starting Layout (an Egress Node, a Robo Portal, four Drones and 100 Space Crystals,
//! `factions/colonists/economy.md > Starting Conditions`) as a [`Colonists`] line; player 2 plays the GDO from slot
//! 2's GDO Starting Layout as a [`BuildOrder`], or does nothing. Both give their own commands, which the gate takes,
//! as players would:
//! - [`Skirmish::opening`]: player 1 gathers, trains Drones, deploys Beacons as its Beacon Capacity needs them,
//!   warps in a Bio Portal on its first Beacon, trains two Sabre Legionnaires there and converts the first to a Rifle
//!   Legionnaire; player 2 does nothing.
//! - [`Skirmish::versus_gdo`]: the same with more Drones and eight Sabres, each converted to a Rifle Legionnaire to
//!   meet the Peacekeepers' range, which wait east of the Bio Portal ([`DEFENCE`]); player 2 plays the golden game
//!   `gdo_opening`'s line, the fastest clean Barracks (Plate, Generator, Generator, Barracks) and ten Peacekeepers,
//!   which gather at home ([`GDO_STAGING`]) and attack-move to the north-east end of player 1's field
//!   ([`GDO_RALLY`]) in waves of [`WAVE`], fighting what they meet on the way.
//!
//! Nothing here assumes the game goes as the goldens' do: the players stop once the game is over or they are
//! defeated, a lost Bio Portal stops the Sabres, and a Sabre lost before it converts isn't converted. Played on to 10
//! minutes (as of slice M3-K3, with the Drone's numbers of design change 0205), `versus_gdo` has its last
//! Peacekeeper die at 2:54, after four Rifle Legionnaires (2:50 after three before 0205); then player 1 gathers and
//! deploys on, stopping at its 30th Drone and its seventh Beacon site, player 2 has nothing left to do, and neither
//! is defeated.

use std::collections::BTreeSet;

use crate::command::{Ability, Command, PlayerCommand};
use crate::defs::ObjectKind;
use crate::events::SimEvent;
use crate::ids::{ObjectId, PlayerId};
use crate::orders::{OrderKind, Target};
use crate::player::Faction;
use crate::production::ProductionCommand;
use crate::scenario::Scenario;
use crate::scenarios;
use crate::sim::Sim;
use crate::testing::build_order::{BuildOrder, Order};
use crate::units::{Tick, TilePos};

use super::Colonists;

use ObjectKind::{
    Barracks, Beacon, BioPortal, DeploymentCenter, Drone, ExtractionFacility, ExtractionPlate, Generator, Peacekeeper,
    PeacekeeperShielded, RoboPortal, SabreLegionnaire as Sabre,
};

/// The Colonists' player, in slot 1.
pub const COLONISTS: PlayerId = PlayerId(1);
/// The GDO's, in slot 2.
pub const GDO: PlayerId = PlayerId(2);
/// `dev_skirmish`'s patch on tile (3, 3): of slot 1's field (the map's first spawns, in its order), the one nearest
/// slot 1's Egress Node. The starting Drones are sent to it, and the Robo Portal is rallied on it.
pub const PATCH: ObjectId = ObjectId(4);
/// Player 1's Beacon sites, by the top-left tile of their footprints, east of its Robo Portal (tiles (6, 6) to (9,
/// 9)), clear of its field and of the Drones' way to it, with a free tile between any two footprints. The Bio Portal
/// is warped in on the first: tiles (11, 7) to (13, 9).
pub const SITES: [TilePos; 8] = [
    TilePos::new(11, 7),
    TilePos::new(11, 4),
    TilePos::new(14, 4),
    TilePos::new(15, 8),
    TilePos::new(11, 11),
    TilePos::new(14, 11),
    TilePos::new(17, 4),
    TilePos::new(18, 8),
];
/// [`Skirmish::opening`]: the Bio Portal's rally point, on open ground east of it.
pub const OPENING_RALLY: TilePos = TilePos::new(17, 7);
/// [`Skirmish::versus_gdo`]: where player 2's Peacekeepers attack-move to, on open ground at the north-east end of
/// player 1's field, past its last patch (7, 2).
pub const GDO_RALLY: TilePos = TilePos::new(10, 2);
/// [`Skirmish::versus_gdo`]: player 2's Barracks' rally point, on open ground north-west of its base (its Deployment
/// Center on tiles (55, 39) to (58, 42)), where its Peacekeepers wait for their wave.
pub const GDO_STAGING: TilePos = TilePos::new(50, 36);
/// [`Skirmish::versus_gdo`]: player 2's Peacekeepers set off once this many of them wait at [`GDO_STAGING`].
pub const WAVE: usize = 5;
/// [`Skirmish::versus_gdo`]: player 1's Legionnaires wait here, east of its Bio Portal, where player 2's
/// Peacekeepers come from: round the west side of the lake north of the map's middle (tiles (22, 6) to (29, 13)).
pub const DEFENCE: TilePos = TilePos::new(17, 10);
/// A Sabre that converts does so this many ticks after it comes out.
pub const CONVERT_AFTER: Tick = 16;

/// `dev_skirmish` with player 1 playing the Colonists (slot 1) and player 2 the GDO (slot 2).
pub fn scenario() -> Scenario {
    let mut scenario = scenarios::dev_skirmish();
    scenario.setup.players[0].faction = Faction::Colonists;
    scenario
}

/// The players of a [`scenario`] game (module docs).
#[derive(Clone, Debug)]
pub struct Skirmish {
    colonists: Colonists,
    gdo: Option<BuildOrder>,
    /// Whether player 2's Barracks has its rally point.
    rallied: bool,
    /// Player 2's Peacekeepers sent with a wave.
    sent: BTreeSet<ObjectId>,
    /// The Sabres player 1 trained, each with the tick it came out in.
    sabres: Vec<(Tick, ObjectId)>,
    /// How many of them convert to Rifle Legionnaires, the first first.
    converts: usize,
}

impl Skirmish {
    /// The Colonists' opening (module docs): 12 Drones, the Bio Portal, two Sabres rallied to [`OPENING_RALLY`], the
    /// first converted.
    pub fn opening() -> Self {
        Self::new(colonists(12, 2, OPENING_RALLY), None, 1)
    }

    /// The Colonists against the GDO (module docs): 30 Drones, the Bio Portal, eight Sabres rallied to [`DEFENCE`],
    /// each converted to a Rifle Legionnaire to meet the Peacekeepers' range; player 2's Barracks line, its ten
    /// Peacekeepers sent to [`GDO_RALLY`] in two waves.
    pub fn versus_gdo() -> Self {
        let gdo = BuildOrder::new(GDO).track(
            [
                Order::new(ExtractionFacility, ExtractionPlate),
                Order::new(DeploymentCenter, Generator),
                Order::new(DeploymentCenter, Generator),
                Order::new(DeploymentCenter, Barracks),
            ]
            .into_iter()
            .chain([Order::new(Barracks, Peacekeeper); 10]),
        );
        Self::new(colonists(30, 8, DEFENCE), Some(gdo), usize::MAX)
    }

    fn new(colonists: Colonists, gdo: Option<BuildOrder>, converts: usize) -> Self {
        Self {
            colonists,
            gdo,
            rallied: false,
            sent: BTreeSet::new(),
            sabres: Vec::new(),
            converts,
        }
    }

    /// Player 1's line.
    pub fn colonists(&self) -> &Colonists {
        &self.colonists
    }

    /// Player 2's build order, if player 2 plays.
    pub fn gdo(&self) -> Option<&BuildOrder> {
        self.gdo.as_ref()
    }

    /// The Sabres player 1 trained, each with the tick it came out in.
    pub fn sabres(&self) -> &[(Tick, ObjectId)] {
        &self.sabres
    }

    /// What `player`'s orders of the step to tick `tick` paid: what the step takes from their bank, as nothing else
    /// the players do costs anything.
    pub fn spent_in(&self, player: PlayerId, tick: Tick) -> u32 {
        if player == COLONISTS {
            return self.colonists.spent_in(tick);
        }
        let issued = self.gdo.iter().flat_map(|line| line.issued());
        issued.filter(|i| i.tick == tick).map(|i| i.cost).sum()
    }

    /// Both players' commands for the step to tick `sim.tick() + 1`, from the game as it is now (module docs).
    pub fn commands(&mut self, sim: &Sim) -> Vec<PlayerCommand> {
        let tick = sim.tick() + 1;
        let mut commands = self.colonists.commands(sim);
        let plays = |player| sim.outcome().is_none() && sim.player(player).is_some_and(|p| !p.is_defeated());

        // Sabres convert CONVERT_AFTER ticks after they come out, as many as the game converts, if they are still
        // Sabres: a Convert to one pressed already would flip it back (design change 0198).
        let new = sim.events().iter().filter_map(|e| match *e {
            SimEvent::UnitProduced { unit, .. } => sim
                .object(unit)
                .filter(|o| o.owner == Some(COLONISTS) && o.kind == Sabre)
                .map(|o| (sim.tick(), o.id)),
            _ => None,
        });
        self.sabres.extend(new);
        let due: Vec<ObjectId> = (self.sabres.iter().take(self.converts))
            .filter(|&&(out, sabre)| tick == out + CONVERT_AFTER && sim.object(sabre).is_some_and(|o| o.kind == Sabre))
            .map(|&(_, sabre)| sabre)
            .collect();
        if !due.is_empty() && plays(COLONISTS) {
            commands.push(PlayerCommand {
                player: COLONISTS,
                command: Command::Ability {
                    units: due,
                    ability: Ability::Convert,
                },
            });
        }

        if let Some(line) = &mut self.gdo {
            commands.extend(line.commands(sim));
        }
        if self.gdo.is_some() && plays(GDO) {
            let own = || sim.objects().filter(|o| o.owner == Some(GDO));
            if !self.rallied
                && let Some(barracks) = own().find(|o| o.kind == Barracks)
            {
                commands.push(PlayerCommand {
                    player: GDO,
                    command: Command::Production {
                        objects: vec![barracks.id],
                        command: ProductionCommand::SetRally(Target::Point(GDO_STAGING.center())),
                    },
                });
                self.rallied = true;
            }
            // The Peacekeepers that wait for their wave: those that have never set off and have no order.
            let waiting: Vec<ObjectId> = own()
                .filter(|o| matches!(o.kind, Peacekeeper | PeacekeeperShielded) && !self.sent.contains(&o.id))
                .filter(|o| o.orders.as_ref().is_none_or(|orders| orders.is_idle()))
                .map(|o| o.id)
                .collect();
            if waiting.len() >= WAVE {
                self.sent.extend(&waiting);
                commands.push(PlayerCommand {
                    player: GDO,
                    command: Command::Order {
                        units: waiting,
                        kind: OrderKind::AttackMove(GDO_RALLY.center()),
                        queue: false,
                    },
                });
            }
        }
        commands
    }
}

/// Player 1's line: the starting Drones sent to [`PATCH`] with one command and the Robo Portal rallied on it; a Drone
/// trained whenever the portal is free, `drones` in all; a Drone deploying at [`SITES`] whenever the room under the
/// player's Beacon Capacity falls to 1; the first Beacon warping in a Bio Portal, which trains `sabres` Sabre
/// Legionnaires rallied to `rally`. The Warp In and the Sabres get the money first.
fn colonists(drones: usize, sabres: usize, rally: TilePos) -> Colonists {
    let line = BuildOrder::new(COLONISTS)
        .track([Order::new(Beacon, BioPortal)])
        .track(vec![Order::new(BioPortal, Sabre); sabres])
        .track(vec![Order::new(RoboPortal, Drone); drones]);
    Colonists::new(line)
        .gather(PATCH)
        .rally(RoboPortal, Target::Object(PATCH))
        .rally(BioPortal, Target::Point(rally.center()))
        .deploy_at(SITES)
}
