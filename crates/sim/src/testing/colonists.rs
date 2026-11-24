//! Scripted Colonists lines (slice M3-K1, plan decision T27: "a sibling of `testing::build_order`, `testing::colonists`:
//! Train, Gather, Deploy and Warp In through the gate, with books that count drop-offs"), and the reference field
//! that the Colonists' economy targets are measured on.
//!
//! A [`Colonists`] line plays a Colonists player the way a player would, with their own commands through the gate,
//! never with debug commands. Before each step, [`Colonists::commands`] looks at the game as the last tick left it and
//! gives the player's commands for the next tick:
//! 1. **Gathering**, either way a player would send Drones:
//!    - [`Colonists::gather`]: at the start, the player's Drones are sent to one patch with one command, as one
//!      right-click sends a selection, and the occupancy rule deals them out over the field
//!      (`factions/colonists/units.md > Drone > Patch Occupancy`: "When a Drone arrives at an occupied patch, it goes
//!      to the nearest unoccupied patch"); new Drones follow the rally point.
//!    - [`Colonists::deal`]: each Drone is sent to a patch of its own, the starting ones at the start and each new
//!      one as it comes out (unless it deploys): to the patch the fewest of the player's Drones gather now, by their
//!      orders, the earlier of the line's patches first.
//! 2. **Rally points**: each portal of a kind the line rallies gets its rally point once it can produce, the starting
//!    ones at the start and a warped-in one once it is complete. A Drone rallied on a patch gathers it (design change
//!    0136, `systems/controls/selection_and_hud.md > What a rally point makes a new unit do`).
//! 3. **Deploying** (`factions/colonists/units.md > Drone`: "With 5 BeaconCapacity per Beacon and the Drone's own 1
//!    freed when it starts deploying, each Beacon is a net +6: beyond the starting 10 BeaconCapacity, roughly every
//!    6th Drone must deploy as a Beacon to keep Drone production going"): a Drone a portal releases deploys, at the
//!    first of the line's sites that the deploy check takes, when the room it leaves under the player's Beacon
//!    Capacity is at most 1 ([`room`]): the next Drone takes that 1 as it starts, and the one after it needs this
//!    Beacon. The Drone frees its 1 as it starts deploying and its Beacon brings 5 within seconds, so a portal that
//!    trains Drones without a break never waits for room. From the Colonists' Starting Conditions (10 Beacon
//!    Capacity, four Drones) that is the 5th Drone made and every 6th after it: the 11th, 17th, 23rd.
//! 4. **What the player pays for** goes through a [`BuildOrder`] (`super::build_order`): Drones and Sabres trained
//!    at the portals, buildings warped in on the Beacons, each as soon as the bank covers it and its builder is free.
//!
//! Gathering, rallies and deploys cost nothing, so the build order's bank is the player's: [`super::build_order::
//! Books`] balance a Colonists player's steps as they do a GDO player's, counting the Drones' deliveries.
//!
//! [`reference_field`] is `factions/gdo/economy.md > Economy Targets (GDO vs Colonists) > Reference geometry` (design
//! change 0107) with a Colonists start on it.

use std::collections::{BTreeMap, BTreeSet};

use glam::Vec2;

use crate::command::{Command, PlayerCommand};
use crate::defs::ObjectKind;
use crate::events::SimEvent;
use crate::ids::{ObjectId, PlayerId};
use crate::orders::{OrderKind, Target};
use crate::player::Faction;
use crate::production::ProductionCommand;
use crate::scenario::{Scenario, SpawnAt};
use crate::sim::Sim;
use crate::units::{Tick, TilePos};

use super::ScenarioBuilder;
use super::build_order::BuildOrder;

/// A Drone the line sent to deploy: the tick its order applies in, the Drone, and the top-left tile of its Beacon's
/// footprint.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Deploying {
    pub tick: Tick,
    pub drone: ObjectId,
    pub origin: TilePos,
}

/// A Colonists player's scripted line (module docs).
#[derive(Clone, Debug)]
pub struct Colonists {
    line: BuildOrder,
    /// The patch the starting Drones gather, if they do.
    gather: Option<ObjectId>,
    /// The patches the Drones are dealt over, in the order they are preferred.
    deal: Vec<ObjectId>,
    /// The rally point of each kind of portal that gets one.
    rallies: Vec<(ObjectKind, Target)>,
    /// Where Beacons are deployed, in order; each site takes one.
    sites: Vec<TilePos>,
    started: bool,
    /// The portals whose rally point the line has set.
    rallied: BTreeSet<ObjectId>,
    deploys: Vec<Deploying>,
}

impl Colonists {
    /// A line for `line`'s player that pays through `line`, and does nothing else yet.
    pub fn new(line: BuildOrder) -> Self {
        Self {
            line,
            gather: None,
            deal: Vec::new(),
            rallies: Vec::new(),
            sites: Vec::new(),
            started: false,
            rallied: BTreeSet::new(),
            deploys: Vec::new(),
        }
    }

    /// The player's Drones gather `patch` from the start, sent by one command.
    pub fn gather(mut self, patch: ObjectId) -> Self {
        self.gather = Some(patch);
        self
    }

    /// Each of the player's Drones gathers a patch of its own among `patches` (module docs, 1): the starting Drones
    /// from the start, and each new one that doesn't deploy from the tick after it comes out.
    pub fn deal(mut self, patches: impl IntoIterator<Item = ObjectId>) -> Self {
        self.deal.extend(patches);
        self
    }

    /// Every portal of kind `portal` gets `target` as its rally point once it can produce.
    pub fn rally(mut self, portal: ObjectKind, target: Target) -> Self {
        self.rallies.push((portal, target));
        self
    }

    /// The sites the Beacons deploy on, in order, by the top-left tile of their footprint (module docs, 3). Without
    /// sites no Drone deploys.
    pub fn deploy_at(mut self, sites: impl IntoIterator<Item = TilePos>) -> Self {
        self.sites.extend(sites);
        self
    }

    pub fn player(&self) -> PlayerId {
        self.line.player()
    }

    /// The build order that pays for the player's Drones, units and Warp Ins.
    pub fn line(&self) -> &BuildOrder {
        &self.line
    }

    /// Every Drone sent to deploy so far, in order.
    pub fn deploys(&self) -> &[Deploying] {
        &self.deploys
    }

    /// What the line's orders of the step to tick `tick` paid.
    pub fn spent_in(&self, tick: Tick) -> u32 {
        self.line
            .issued()
            .iter()
            .filter(|i| i.tick == tick)
            .map(|i| i.cost)
            .sum()
    }

    /// The player's commands for the step to tick `sim.tick() + 1`, from the game as it is now (module docs). Nothing
    /// once the game is over or the player is defeated, as the gate would refuse it.
    pub fn commands(&mut self, sim: &Sim) -> Vec<PlayerCommand> {
        let tick = sim.tick() + 1;
        let player = self.player();
        let mut commands = Vec::new();
        if sim.player(player).is_none_or(|p| p.is_defeated()) || sim.outcome().is_some() {
            return commands;
        }
        let command = |command| PlayerCommand { player, command };
        let own = || sim.objects().filter(move |o| o.owner == Some(player));

        // The Drones released in the last tick, in the order they came out.
        let released: Vec<ObjectId> = sim
            .events()
            .iter()
            .filter_map(|e| match *e {
                SimEvent::UnitProduced { unit, .. } => Some(unit),
                _ => None,
            })
            .filter(|&unit| {
                sim.object(unit)
                    .is_some_and(|o| o.owner == Some(player) && o.kind == ObjectKind::Drone)
            })
            .collect();
        // The Drones on each patch the line deals, by their orders, those about to be dealt left out; each Drone
        // dealt in this tick adds itself.
        let mut on: BTreeMap<ObjectId, u32> = self.deal.iter().map(|&patch| (patch, 0)).collect();
        for o in own().filter(|o| !released.contains(&o.id)) {
            let current = o.orders.as_ref().and_then(|orders| orders.current.as_ref());
            if let Some(OrderKind::Gather(patch)) = current.map(|c| c.order.kind)
                && let Some(n) = on.get_mut(&patch)
            {
                *n += 1;
            }
        }
        let deal = self.deal.clone();
        let mut deal = |drone: ObjectId, commands: &mut Vec<PlayerCommand>| {
            let Some(patch) = deal.iter().copied().min_by_key(|patch| on[patch]) else {
                return;
            };
            *on.entry(patch).or_default() += 1;
            commands.push(command(Command::Order {
                units: vec![drone],
                kind: OrderKind::Gather(patch),
                queue: false,
            }));
        };

        if !self.started {
            self.started = true;
            let drones: Vec<ObjectId> = own().filter(|o| o.worker.is_some()).map(|o| o.id).collect();
            if !self.deal.is_empty() {
                for &drone in &drones {
                    deal(drone, &mut commands);
                }
            } else if let Some(patch) = self.gather
                && !drones.is_empty()
            {
                commands.push(command(Command::Order {
                    units: drones,
                    kind: OrderKind::Gather(patch),
                    queue: false,
                }));
            }
        }

        for o in own().filter(|o| o.production.is_some() && !o.is_transforming()) {
            let Some(&(_, target)) = self.rallies.iter().find(|(kind, _)| *kind == o.kind) else {
                continue;
            };
            if self.rallied.insert(o.id) {
                commands.push(command(Command::Production {
                    objects: vec![o.id],
                    command: ProductionCommand::SetRally(target),
                }));
            }
        }

        let mut sent = 0;
        for drone in released {
            let site = (room(sim, player) + 6 * sent <= 1)
                .then(|| {
                    let used = self.deploys.len();
                    self.sites.iter().skip(used).copied().find(|&origin| {
                        sim.deploy_check(drone, origin, false)
                            .is_some_and(|check| check.is_valid())
                    })
                })
                .flatten();
            let Some(origin) = site else {
                deal(drone, &mut commands);
                continue;
            };
            commands.push(command(Command::Order {
                units: vec![drone],
                kind: OrderKind::Deploy { origin },
                queue: false,
            }));
            self.deploys.push(Deploying { tick, drone, origin });
            sent += 1;
        }

        commands.extend(self.line.commands(sim));
        commands
    }
}

/// The room `player` has under their Beacon Capacity once the Beacons on their way are up: what is available less
/// what is used ([`Sim::army_cap`]), plus 5 for each Beacon under deployment (`factions/colonists/economy.md > Beacon
/// Deployment > Beacon Capacity during deployment`, design change 0112: it provides its 5 at completion), and 6 for
/// each Drone with a Deploy order (its own 1 freed as it starts, and its Beacon's 5). 0 for a player without
/// Beacon Capacity.
pub fn room(sim: &Sim, player: PlayerId) -> i64 {
    let Some(cap) = sim.army_cap(player) else {
        return 0;
    };
    let deploying = sim
        .objects()
        .filter(|o| o.owner == Some(player) && o.is_deploying())
        .count();
    let walking = sim.pending_deploys(player).len();
    i64::from(cap.available) - i64::from(cap.used) + 5 * deploying as i64 + 6 * walking as i64
}

/// The player of [`reference_field`].
pub const FIELD_PLAYER: PlayerId = PlayerId(1);
/// [`reference_field`]'s patches, from the west, are objects 1 to 8.
pub const FIELD_PATCHES: u32 = 8;
/// [`reference_field`]'s middle patch, the fourth from the west: the one the Drones are sent to and rallied on.
pub const FIELD_MIDDLE: ObjectId = ObjectId(4);
/// [`reference_field`]'s patches in the order Drones are dealt over them ([`Colonists::deal`]): the middle ones
/// first, west and east of the node in turn, as `tests/gathering_soak.rs` deals its Drones.
pub const FIELD_DEAL: [ObjectId; 8] = [
    ObjectId(4),
    ObjectId(5),
    ObjectId(3),
    ObjectId(6),
    ObjectId(2),
    ObjectId(7),
    ObjectId(1),
    ObjectId(8),
];
/// [`reference_field`]'s Egress Node.
pub const FIELD_NODE: ObjectId = ObjectId(9);
/// [`reference_field`]'s Robo Portal.
pub const FIELD_PORTAL: ObjectId = ObjectId(10);
/// [`reference_field`]'s Beacon sites, by the top-left tile of their footprints: west and east of the portal, out of
/// the Drones' way between the node and the row, the nearer ones first. Eight Beacons hold Beacon Capacity for 48
/// Drones.
pub const FIELD_SITES: [TilePos; 8] = [
    TilePos::new(2, 11),
    TilePos::new(12, 11),
    TilePos::new(2, 14),
    TilePos::new(12, 14),
    TilePos::new(0, 11),
    TilePos::new(14, 11),
    TilePos::new(0, 14),
    TilePos::new(14, 14),
];

/// `factions/gdo/economy.md > Economy Targets (GDO vs Colonists) > Reference geometry` (design change 0107; Q0038
/// point 2) with player 1 playing the Colonists: 8 adjacent patches in a row at tiles (4, 3) to (11, 3), ids 1 to 8
/// from the west; the Egress Node (id 9) on tiles (7, 6) to (8, 7), on the row's long side, centred along it, with 2
/// grid units of open ground between its edge and the middle patches' (128 space units); the Robo Portal (id 10) on
/// tiles (6, 11) to (9, 14), 3 tiles south of the node, out of the Drones' way; and `drones` Drones (ids 11 on) in
/// blocks beside the node, west and east of it in turn, as `tests/gathering_soak.rs` stands them. With the four of
/// the Starting Conditions (`factions/colonists/economy.md > Starting Conditions`: "one pre-built Egress Node, one
/// pre-built Robo Portal [...] and four Drones"; 100 Space Crystals, `ScenarioBuilder::faction`) it is the Colonists'
/// start on the reference field; with more, the field at a number of Drones. Player 2 has nothing. "The reference
/// layout is only a benchmark" (design change 0108): nothing in the game assumes it.
pub fn reference_field(drones: u32) -> Scenario {
    let mut builder =
        ScenarioBuilder::new(&vec![".".repeat(16); 17].join("\n")).faction(FIELD_PLAYER, Faction::Colonists);
    for x in 0..FIELD_PATCHES as i32 {
        builder = builder.spawn(ObjectKind::CrystalPatch, None, SpawnAt::Tile(TilePos::new(4 + x, 3)));
    }
    builder = builder
        .spawn(
            ObjectKind::EgressNode,
            Some(FIELD_PLAYER),
            SpawnAt::Tile(TilePos::new(7, 6)),
        )
        .spawn(
            ObjectKind::RoboPortal,
            Some(FIELD_PLAYER),
            SpawnAt::Tile(TilePos::new(6, 11)),
        );
    for i in 0..drones as i32 {
        // Beside the node, west and east in turn: three rows of up to four on each side.
        let (side, place) = (if i % 2 == 0 { -1.0 } else { 1.0 }, i / 2);
        let at = Vec2::new(
            512.0 + side * (92.0 + 44.0 * (place / 3) as f32),
            402.0 + 44.0 * (place % 3) as f32,
        );
        builder = builder.spawn(ObjectKind::Drone, Some(FIELD_PLAYER), SpawnAt::Point(at));
    }
    builder.build()
}

/// [`reference_field`]'s Drones, when it has `drones`.
pub fn field_drones(drones: u32) -> Vec<ObjectId> {
    (0..drones).map(|i| ObjectId(FIELD_PORTAL.0 + 1 + i)).collect()
}
