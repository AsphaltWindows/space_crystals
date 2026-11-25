//! Scripted build orders (slice M2-J1), for tests that play an opening the way a player would: with the player's own
//! `Command::Construction` and `Command::Production`, through the command gate, never with debug commands. The
//! economy oracle (`tests/economy_oracle/`) checks `factions/gdo/economy.md > Opening Economy Baseline` with them,
//! and the golden game `gdo_opening` (`tests/golden/gdo.rs`) plays one. A Colonists player's line
//! ([`super::colonists`], slice M3-K1) pays for its Drones, Sabres and Warp Ins through one.
//!
//! A [`BuildOrder`] is a player's tracks, in priority order; a track is a sequence of [`Order`]s, each a building
//! for one of the player's constructors to construct, a unit for one of their producers to train, or a building for
//! one of their Beacons to warp in. Before each step, [`BuildOrder::commands`] looks at the game as the last tick
//! left it and gives the player's commands for the next tick:
//! 1. each building held ready to place is placed at once, at the first spot of a fixed scan that the placement
//!    check accepts ([`first_spot`]);
//! 2. then each track in turn issues its next order, once the bank covers its cost (as the tracks before it in this
//!    tick left the bank) and its builder is free: a constructor that is idle or places its ready building in this
//!    tick (the Place applies first), a producer with nothing queued, a Beacon (slice M3-K1) whose Warp In the
//!    placement check allows now, unturned (`Sim::warp_in_check`). Nothing that transforms is free: a building under
//!    Warp In and a Beacon under deployment take only their cancel (design change 0116). The builder is the player's
//!    first object of the order's `by` kind, by id; a constructor or producer that came into the game in a tick
//!    takes orders from the next (`ARCHITECTURE.md > Tick pipeline > Spawn tick`).
//!
//! An order waits for the orders before it in its track; the tracks don't wait for each other, but the earlier ones
//! get the money first. A track's orders may go to different builders: a line of one track is the design's "Plate,
//! Generator, Generator, Barracks". Every order a build order issues is one the gate takes: it pays from a bank that
//! nothing else in the tick draws on, as mining and deliveries pay only after the commands.
//!
//! [`Books`] keep a player's accounts for a step: the bank's change must be what the plates mined (read off the
//! patches under them, not off the bank) and what the workers delivered (the step's `SimEvent::ResourcesDelivered`,
//! plan decision T27: "books that count drop-offs"), less what the step's orders paid, plus the refunds the step's
//! events report.

use std::collections::{BTreeMap, BTreeSet};

use crate::command::{Command, PlayerCommand};
use crate::construction::{self, ConstructionCommand, Constructor};
use crate::defs::{ObjectKind, Recipe};
use crate::events::SimEvent;
use crate::ids::{ObjectId, PlayerId};
use crate::nav::TileRect;
use crate::object::ResourceState;
use crate::placement::{footprint_size, seeds_build_area};
use crate::production::ProductionCommand;
use crate::scenario::BuildingOrientation;
use crate::sim::Sim;
use crate::transform;
use crate::units::{Tick, TilePos};

/// One order of a build order: the player's building of kind `by` makes a `kind`, a building it constructs
/// (`ConstructionCommand::Construct`) or a unit it trains (`ProductionCommand::Train`).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Order {
    pub by: ObjectKind,
    pub kind: ObjectKind,
}

impl Order {
    pub const fn new(by: ObjectKind, kind: ObjectKind) -> Self {
        Self { by, kind }
    }

    /// The recipe `by` follows: its `Constructs` entry for a building, its `Produces` entry for a unit, its Warp In
    /// list's entry for a building a Beacon warps in. Panics if it makes no `kind`: a build order that asks for it is
    /// a mistake in the test.
    pub fn recipe(&self) -> &'static Recipe {
        construction::recipe(self.by, self.kind)
            .or_else(|| self.by.def().structure.as_ref()?.production.as_ref()?.recipe(self.kind))
            .or_else(|| transform::warp_in_recipe(self.by, self.kind))
            .unwrap_or_else(|| panic!("a {:?} makes no {:?}", self.by, self.kind))
    }

    /// Whether `by` warps `kind` in rather than constructing or training it.
    fn is_warp_in(&self) -> bool {
        transform::warp_in_recipe(self.by, self.kind).is_some()
    }
}

/// An order a build order issued: the tick its command applies in, which is the first of its recipe's frames
/// (`ARCHITECTURE.md > Economy > Recipe timing`), the builder, and what it paid.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Issued {
    pub tick: Tick,
    pub order: Order,
    pub by: ObjectId,
    pub cost: u32,
}

/// A ready building a build order placed: the tick its Place applies in, the building, its constructor and the
/// top-left tile of its footprint, unturned.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Placing {
    pub tick: Tick,
    pub kind: ObjectKind,
    pub by: ObjectId,
    pub origin: TilePos,
}

/// A player's scripted build order (module docs).
#[derive(Clone, Debug)]
pub struct BuildOrder {
    player: PlayerId,
    /// Each track's orders and how many of them it has issued.
    tracks: Vec<(Vec<Order>, usize)>,
    issued: Vec<Issued>,
    placed: Vec<Placing>,
    /// The ticks in which a ready building found no spot, and the building.
    stalled: Vec<(Tick, ObjectKind)>,
}

impl BuildOrder {
    /// A build order for `player`, without tracks yet.
    pub fn new(player: PlayerId) -> Self {
        Self {
            player,
            tracks: Vec::new(),
            issued: Vec::new(),
            placed: Vec::new(),
            stalled: Vec::new(),
        }
    }

    /// Adds a track after the others: it gets the money after them.
    pub fn track(mut self, orders: impl IntoIterator<Item = Order>) -> Self {
        self.tracks.push((orders.into_iter().collect(), 0));
        self
    }

    pub fn player(&self) -> PlayerId {
        self.player
    }

    /// Every order issued so far, in the order they were.
    pub fn issued(&self) -> &[Issued] {
        &self.issued
    }

    /// Every ready building placed so far.
    pub fn placed(&self) -> &[Placing] {
        &self.placed
    }

    /// The ticks in which a ready building found no spot on the map, and the building.
    pub fn stalled(&self) -> &[(Tick, ObjectKind)] {
        &self.stalled
    }

    /// Whether every track has issued all its orders.
    pub fn is_done(&self) -> bool {
        self.tracks.iter().all(|(orders, next)| *next >= orders.len())
    }

    /// The player's commands for the step to tick `sim.tick() + 1`, from the game as it is now (module docs): the
    /// Places of the ready buildings, then each track's next order if it can go. Nothing once the game is over or
    /// the player is defeated, as the gate would refuse it.
    pub fn commands(&mut self, sim: &Sim) -> Vec<PlayerCommand> {
        let tick = sim.tick() + 1;
        let mut commands = Vec::new();
        let Some(player) = sim.player(self.player).filter(|p| !p.is_defeated()) else {
            return commands;
        };
        if sim.outcome().is_some() {
            return commands;
        }
        let mut bank = player.crystals;
        let mut free: BTreeSet<ObjectId> = BTreeSet::new();
        let mut taken: Vec<TileRect> = Vec::new();
        for o in sim.objects().filter(|o| o.owner == Some(self.player)) {
            match o.constructor {
                Some(Constructor::Idle) => {
                    free.insert(o.id);
                }
                Some(Constructor::Ready { kind }) => match spot(sim, self.player, kind, &taken) {
                    Some((origin, rect)) => {
                        taken.push(rect);
                        commands.push(self.command(Command::Construction {
                            objects: vec![o.id],
                            command: ConstructionCommand::Place {
                                origin,
                                orientation: BuildingOrientation::default(),
                            },
                        }));
                        self.placed.push(Placing {
                            tick,
                            kind,
                            by: o.id,
                            origin,
                        });
                        free.insert(o.id);
                    }
                    None => self.stalled.push((tick, kind)),
                },
                Some(Constructor::Constructing { .. }) | None => {}
            }
            let idle = !o.is_transforming();
            if idle && o.production.as_ref().is_some_and(|p| p.queue.is_empty()) {
                free.insert(o.id);
            }
            if idle && !transform::warp_in_recipes(o.kind).is_empty() {
                free.insert(o.id);
            }
        }
        for (orders, next) in &mut self.tracks {
            let Some(&order) = orders.get(*next) else {
                continue;
            };
            let Some(by) = sim
                .objects()
                .find(|o| o.owner == Some(self.player) && o.kind == order.by)
                .map(|o| o.id)
            else {
                continue;
            };
            let cost = order.recipe().cost;
            let allowed = || {
                !order.is_warp_in()
                    || sim
                        .warp_in_check(by, order.kind, BuildingOrientation::default())
                        .is_some_and(|check| check.is_valid())
            };
            if !free.contains(&by) || cost > bank || !allowed() {
                continue;
            }
            bank -= cost;
            free.remove(&by);
            *next += 1;
            let command = if construction::recipe(order.by, order.kind).is_some() {
                Command::Construction {
                    objects: vec![by],
                    command: ConstructionCommand::Construct(order.kind),
                }
            } else if order.is_warp_in() {
                Command::Construction {
                    objects: vec![by],
                    command: ConstructionCommand::WarpIn {
                        kind: order.kind,
                        orientation: BuildingOrientation::default(),
                    },
                }
            } else {
                Command::Production {
                    objects: vec![by],
                    command: ProductionCommand::Train(order.kind),
                }
            };
            commands.push(PlayerCommand {
                player: self.player,
                command,
            });
            self.issued.push(Issued { tick, order, by, cost });
        }
        commands
    }

    fn command(&self, command: Command) -> PlayerCommand {
        PlayerCommand {
            player: self.player,
            command,
        }
    }
}

/// Where `player` may place a `kind` now: the first origin, row by row from the map's north-west corner, at which
/// the building, unturned, passes `Sim::placement_check`. `None` if there is none.
pub fn first_spot(sim: &Sim, player: PlayerId, kind: ObjectKind) -> Option<TilePos> {
    spot(sim, player, kind, &[]).map(|(origin, _)| origin)
}

/// [`first_spot`], leaving out the footprints `taken` by the Places issued before it in the same tick, and the
/// footprint it found. Only origins whose footprint reaches the build area are checked, as no other can pass.
fn spot(sim: &Sim, player: PlayerId, kind: ObjectKind, taken: &[TileRect]) -> Option<(TilePos, TileRect)> {
    let orientation = BuildingOrientation::default();
    let (w, h) = footprint_size(kind, orientation)?;
    let area = sim.build_area(player);
    let (width, height) = (sim.map().width() as i32, sim.map().height() as i32);
    (0..height)
        .flat_map(|y| (0..width).map(move |x| TilePos::new(x, y)))
        .map(|origin| (origin, TileRect::new(origin, w, h)))
        .find(|&(origin, rect)| {
            (seeds_build_area(kind) || area.reaches(rect))
                && !taken.iter().any(|&t| overlap(t, rect))
                && sim.placement_check(player, kind, origin, orientation).is_valid()
        })
}

fn overlap(a: TileRect, b: TileRect) -> bool {
    let (aw, ah, bw, bh) = (a.w as i32, a.h as i32, b.w as i32, b.h as i32);
    a.origin.x < b.origin.x + bw
        && b.origin.x < a.origin.x + aw
        && a.origin.y < b.origin.y + bh
        && b.origin.y < a.origin.y + ah
}

/// A player's accounts for one step (module docs): opened before it, closed after it.
#[derive(Clone, Debug)]
pub struct Books {
    player: PlayerId,
    bank: u32,
    /// What each patch under one of the player's plates holds.
    reserves: BTreeMap<ObjectId, u32>,
    /// The player's constructors, producers and transforming objects, whose refunds go to them.
    builders: BTreeSet<ObjectId>,
    /// The player's workers, whose deliveries go to them. A unit that comes into the game in a step can't deliver
    /// in it.
    workers: BTreeSet<ObjectId>,
}

/// What one step did to a player's bank ([`Books::close`]).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Entry {
    /// The bank as the step left it.
    pub bank: u32,
    /// What the player's plates mined: what their patches lost.
    pub mined: u32,
    /// Each patch that lost crystals in the step, by id, and how many: its plate's lump.
    pub paid: Vec<(ObjectId, u32)>,
    /// What the player's workers delivered: their `SimEvent::ResourcesDelivered` (slice M3-K1).
    pub delivered: u32,
    /// Each delivery of the step: the worker, and what it delivered.
    pub deliveries: Vec<(ObjectId, u32)>,
    /// What the player's commands paid.
    pub spent: u32,
    /// What came back: the `SimEvent::ConstructionCancelled`, `SimEvent::ProductionCancelled` and
    /// `SimEvent::TransformCancelled` of their buildings.
    pub refunded: u32,
}

impl Books {
    /// Opens `player`'s books before a step, from the game as it is.
    pub fn open(sim: &Sim, player: PlayerId) -> Self {
        let reserves = sim
            .objects()
            .filter(|o| o.owner == Some(player))
            .filter_map(|o| {
                let patch = o.extractor?.patch;
                match sim.object(patch)?.resource {
                    Some(ResourceState::CrystalPatch { remaining }) => Some((patch, remaining)),
                    _ => None,
                }
            })
            .collect();
        let builders = sim
            .objects()
            .filter(|o| {
                o.owner == Some(player) && (o.constructor.is_some() || o.production.is_some() || o.is_transforming())
            })
            .map(|o| o.id)
            .collect();
        let workers = sim
            .objects()
            .filter(|o| o.owner == Some(player) && o.worker.is_some())
            .map(|o| o.id)
            .collect();
        Self {
            player,
            bank: sim.player(player).map_or(0, |p| p.crystals),
            reserves,
            builders,
            workers,
        }
    }

    /// Closes the books after the step in which the player's commands paid `spent`. Panics if they don't balance:
    /// if the bank changed by anything but what the plates mined and the workers delivered, less `spent`, plus the
    /// refunds; or if a plated patch ran out, since a depleted patch's residual mining doesn't show on it
    /// (`factions/gdo/economy.md > ExtractionPlate > Depleted patch under a plate`).
    pub fn close(self, sim: &Sim, spent: u32) -> Entry {
        let paid: Vec<(ObjectId, u32)> = self
            .reserves
            .iter()
            .filter_map(|(&patch, &before)| {
                let Some(&ResourceState::CrystalPatch { remaining }) = sim.object(patch)?.resource.as_ref() else {
                    return None;
                };
                assert!(remaining > 0, "patch #{} ran out: the books can't follow it", patch.0);
                (remaining < before).then_some((patch, before - remaining))
            })
            .collect();
        let mined = paid.iter().map(|&(_, amount)| amount).sum();
        let refunded: u32 = sim
            .events()
            .iter()
            .filter_map(|e| match *e {
                SimEvent::ConstructionCancelled {
                    constructor: by,
                    refund,
                    ..
                }
                | SimEvent::ProductionCancelled {
                    producer: by, refund, ..
                }
                | SimEvent::TransformCancelled { id: by, refund, .. } => self.builders.contains(&by).then_some(refund),
                _ => None,
            })
            .sum();
        let deliveries: Vec<(ObjectId, u32)> = sim
            .events()
            .iter()
            .filter_map(|e| match *e {
                SimEvent::ResourcesDelivered { worker, amount, .. } => {
                    self.workers.contains(&worker).then_some((worker, amount))
                }
                _ => None,
            })
            .collect();
        let delivered = deliveries.iter().map(|&(_, amount)| amount).sum();
        let bank = sim.player(self.player).map_or(0, |p| p.crystals);
        assert_eq!(
            i64::from(bank),
            i64::from(self.bank) + i64::from(mined) + i64::from(delivered) - i64::from(spent) + i64::from(refunded),
            "tick {}: player {}'s bank went from {} to {bank}, but the plates mined {mined}, the workers delivered \
             {delivered}, the commands paid {spent} and {refunded} came back",
            sim.tick(),
            self.player.0,
            self.bank,
        );
        Entry {
            bank,
            mined,
            paid,
            delivered,
            deliveries,
            spent,
            refunded,
        }
    }
}
