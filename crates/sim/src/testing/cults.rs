//! Scripted Cults lines (slice M5-L): a sibling of `testing::build_order`, `testing::colonists` and
//! `testing::syndicate` that plays a Cults player through the gate (Gather, a group's Build, Enter, Train Militant,
//! rally), and the reference layouts that the Cults' economy check is measured on (`factions/cults/economy.md > Economy
//! check`, design change 0346; `fields.rs`).
//!
//! A [`Cults`] line plays a Cults player the way a player would, with their own commands through the gate, never with
//! debug commands. Before each step, [`Cults::commands`] looks at the game as the last tick left it and gives the
//! player's commands for the next tick:
//! 1. **Dealing** ([`Cults::deal`]): the line's seats, in groups, in order: each a patch and how many Recruits it seats.
//!    Every idle Recruit of the player's (no order at all: the starting ones at the start, each one a Hall makes as it
//!    comes out, a builder with no seat once its building is up) is sent to gather a patch: in the first group with a
//!    free seat, the patch with the fewest Recruits, the earlier one among equals. A Recruit's seat is the patch of its
//!    first Gather, current or queued, so a builder keeps its seat. With no seat free the Recruit stays where it is,
//!    spare. The seats keep the layouts within the design's 4–5 Recruits a 2-tile face (design change 0320, answering
//!    Q0110: "The reference layouts and the Cults economy check should stay within the design's 4–5 Recruits per
//!    face"; `factions/cults/units/recruit.md > Gathering`: "The player's answer is more Storages"). Nothing is dealt
//!    while the player has no completed Storage to deliver to (`factions/cults/units/recruit.md > Gathering`: "With
//!    no completed Storage, the Recruit idles holding its load").
//! 2. **Building** ([`Cults::build`]): the line's buildings, in order, each once the one before it is complete, the
//!    bank covers its price (`Sim::build_price`) and the gate takes the Build from its crew ([`Crew`]): one order to
//!    the whole crew, a group's Build, paid once (`factions/cults/economy.md > Cults Building Mechanics > Group
//!    construction rules`), and each builder's Gather back to its seat queued after it (Shift), which it goes on to
//!    once the building is complete. A Build that fails before its site starts, or a site lost before it is complete,
//!    is ordered again (the gate refunds the first in full, `factions/cults/economy.md > Cults Building Mechanics >
//!    Group construction refunds`).
//! 3. **Militants** ([`Cults::militants`]): once the player owns a complete Armory (their first, by id), idle Recruits
//!    go into it rather than to a seat, one for each Militant the line still trains (`factions/cults/units/recruit.md >
//!    Armory Entry`), and the Armory trains a Militant of a Recruit inside whenever one is free and the bank covers it,
//!    one a tick (`ProductionCommand::TrainFrom`, `factions/cults/structures/armory.md > Training`). The Armory gets
//!    the line's rally point once complete.
//!
//! What the line pays (its Builds and Militants) is recorded by tick ([`Cults::spent_in`]), so that
//! [`Books`](super::build_order::Books) balance a Cults player's bank as they do the others', counting the Recruits'
//! deliveries. A Hall's Recruits cost nothing (`factions/cults/structures/recruitment_hall.md > Production Behavior`).
//!
//! [`reference_field`], [`crowd_field`] and [`face_field`] are the check's layouts (`fields.rs`).

mod fields;

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::augmentation::RecruitKind;
use crate::command::{Command, PlayerCommand};
use crate::defs::ObjectKind;
use crate::events::SimEvent;
use crate::ids::{ObjectId, PlayerId};
use crate::object::Object;
use crate::orders::{OrderKind, Target};
use crate::placement::footprint_size;
use crate::production::ProductionCommand;
use crate::scenario::BuildingOrientation;
use crate::shape::Shape;
use crate::sim::Sim;
use crate::units::{SU_PER_GU, Tick, TilePos};

use super::build_order::Order;

pub use fields::{
    FIELD_PAIRS, FIELD_PLAYER, FIELD_RESERVE, FIELD_STORAGES, SECOND_HALL, START_PAIR, crowd_field, face_field,
    reference_field,
};

/// The gap a Recruit mines at the design's best rate, edge to edge between its patch and the Storage: one tile
/// (`factions/cults/units/recruit.md > Gathering`: "the best layout leaves a 1-tile (64 su) gap"). A Recruit whose
/// patch has no own completed Storage this near mines at a loss ([`Crew::Far`]).
pub const IDEAL_GAP: f32 = SU_PER_GU;

/// Who builds one of the line's buildings ([`Cults::build`]).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Crew {
    /// Every Recruit of the player's that mines at a loss or not at all: seated at a patch with no own completed
    /// Storage within [`IDEAL_GAP`] of it, or spare (design change 0346: "Build the second Hall with **all the
    /// non-optimal Recruits**"). The Build waits until there is one.
    Far,
    /// The `n` Recruits gathering nearest to the site's centre (fewer if there are fewer), ties by id.
    Nearest(usize),
}

/// One of the line's buildings: a `kind` with the top-left tile of its footprint at `origin`, turned by
/// `orientation`, built by its `crew`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Build {
    pub kind: ObjectKind,
    pub origin: TilePos,
    pub orientation: BuildingOrientation,
    pub crew: Crew,
}

impl Build {
    /// A `kind` at `origin`, unturned, built by `crew`.
    pub const fn new(kind: ObjectKind, origin: TilePos, crew: Crew) -> Self {
        Self {
            kind,
            origin,
            orientation: BuildingOrientation {
                rotation: crate::object::Rotation::R0,
                flip_h: false,
                flip_v: false,
            },
            crew,
        }
    }

    /// The centre of its footprint.
    pub fn centre(&self) -> glam::Vec2 {
        let (w, h) = footprint_size(self.kind, self.orientation).unwrap_or((1, 1));
        self.origin.corner() + glam::Vec2::new(w as f32, h as f32) * (SU_PER_GU / 2.0)
    }
}

/// Something the line did or saw. Its commands are noted with the tick they apply in, what it saw of its buildings
/// with the tick it happened in.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Done {
    /// It sent `recruit` to gather `patch`.
    Dealt { recruit: ObjectId, patch: ObjectId },
    /// It ordered `build` from `crew`, for `price` Space Crystals.
    Ordered {
        build: Build,
        crew: Vec<ObjectId>,
        price: u32,
    },
    /// The site of its build began.
    Started { build: Build, site: ObjectId },
    /// The site was complete.
    Built { build: Build, site: ObjectId },
    /// The Build went before its site began, or its site before it was complete: the line orders it again.
    Lost { build: Build },
    /// It sent `recruit` into the Armory `armory`.
    Entered { recruit: ObjectId, armory: ObjectId },
    /// It had the Armory `armory` train a Militant of a Recruit inside, for `price`.
    Trained { armory: ObjectId, price: u32 },
}

/// A Cults player's scripted line (module docs).
#[derive(Clone, Debug)]
pub struct Cults {
    player: PlayerId,
    /// The seats, in groups, in order: each patch with how many Recruits it seats.
    groups: Vec<Vec<(ObjectId, u32)>>,
    /// The buildings still to order, in order.
    builds: VecDeque<Build>,
    /// The building ordered and not complete yet, with its site once it began.
    building: Option<(Build, Option<ObjectId>)>,
    /// How many Militants the line trains, and the Armory's rally point.
    militants: usize,
    armory_rally: Option<Target>,
    /// The Recruits sent into an Armory, and the Militants ordered.
    entered: usize,
    trained: usize,
    rallied: BTreeSet<ObjectId>,
    done: Vec<(Tick, Done)>,
    /// What the line's commands paid, by tick.
    paid: Vec<(Tick, u32)>,
}

impl Cults {
    /// A line for `player` that does nothing yet.
    pub fn new(player: PlayerId) -> Self {
        Self {
            player,
            groups: Vec::new(),
            builds: VecDeque::new(),
            building: None,
            militants: 0,
            armory_rally: None,
            entered: 0,
            trained: 0,
            rallied: BTreeSet::new(),
            done: Vec::new(),
            paid: Vec::new(),
        }
    }

    /// A group of seats, after the line's earlier ones (module docs, 1): each patch with how many Recruits it seats.
    pub fn deal(mut self, seats: impl IntoIterator<Item = (ObjectId, u32)>) -> Self {
        self.groups.push(seats.into_iter().collect());
        self
    }

    /// A building, after the line's earlier ones (module docs, 2).
    pub fn build(mut self, build: Build) -> Self {
        self.builds.push_back(build);
        self
    }

    /// `n` Militants from the player's first Armory, which rallies on `rally` (module docs, 3).
    pub fn militants(mut self, n: usize, rally: Option<Target>) -> Self {
        self.militants = n;
        self.armory_rally = rally;
        self
    }

    pub fn player(&self) -> PlayerId {
        self.player
    }

    /// Everything the line did and saw of its buildings, in order.
    pub fn done(&self) -> &[(Tick, Done)] {
        &self.done
    }

    /// Whether every building of the line is complete.
    pub fn built_all(&self) -> bool {
        self.builds.is_empty() && self.building.is_none()
    }

    /// What the line's orders of the step to tick `tick` paid, in Space Crystals.
    pub fn spent_in(&self, tick: Tick) -> u32 {
        self.paid
            .iter()
            .filter(|&&(t, _)| t == tick)
            .map(|&(_, paid)| paid)
            .sum()
    }

    /// The player's commands for the step to tick `sim.tick() + 1`, from the game as it is now (module docs). Nothing
    /// once the game is over or the player is defeated, as the gate would refuse it.
    pub fn commands(&mut self, sim: &Sim) -> Vec<PlayerCommand> {
        let tick = sim.tick() + 1;
        let player = self.player;
        let mut commands = Vec::new();
        let Some(me) = sim.player(player).filter(|p| !p.is_defeated()) else {
            return commands;
        };
        if sim.outcome().is_some() {
            return commands;
        }
        let mut crystals = me.crystals;
        self.follow(sim);

        let own = |kind| (sim.objects()).filter(move |o| o.owner == Some(player) && o.kind == kind);
        let recruits: Vec<&Object> = own(ObjectKind::Recruit).collect();
        let storages: Vec<Shape> = own(ObjectKind::Storage)
            .filter(|o| o.is_operational())
            .map(Shape::of)
            .collect();
        let armory = own(ObjectKind::Armory).find(|o| o.is_operational()).map(|o| o.id);
        let order = |units: Vec<ObjectId>, kind: OrderKind, queue: bool| PlayerCommand {
            player,
            command: Command::Order { units, kind, queue },
        };

        // The seats taken, by the Recruits' orders.
        let mut taken: BTreeMap<ObjectId, u32> = (self.groups.iter().flatten()).map(|&(patch, _)| (patch, 0)).collect();
        for patch in recruits.iter().filter_map(|o| seat_of(o)) {
            if let Some(n) = taken.get_mut(&patch) {
                *n += 1;
            }
        }
        // The idle Recruits: into the Armory while it takes more, else to a seat.
        let mut given = BTreeSet::new();
        for o in recruits.iter().filter(|o| is_idle(o)) {
            if let Some(armory) = armory
                && self.entered < self.militants
            {
                commands.push(order(vec![o.id], OrderKind::Enter(armory), false));
                self.done.push((tick, Done::Entered { recruit: o.id, armory }));
                self.entered += 1;
                given.insert(o.id);
                continue;
            }
            if storages.is_empty() {
                continue;
            }
            let Some(patch) = self.seat(sim, &mut taken) else {
                continue;
            };
            commands.push(order(vec![o.id], OrderKind::Gather(patch), false));
            self.done.push((tick, Done::Dealt { recruit: o.id, patch }));
            given.insert(o.id);
        }

        // The next building, once the one before it is complete.
        if self.building.is_none()
            && let Some(&build) = self.builds.front()
            && let Some(price) = sim.build_price(player, build.kind).map(|price| price.amount)
            && price <= crystals
        {
            let crew = self.crew(sim, build, &recruits, &storages, &given);
            let kind = OrderKind::Build {
                kind: build.kind,
                origin: build.origin,
                orientation: build.orientation,
            };
            let command = order(crew.clone(), kind, false);
            if !crew.is_empty() && sim.validate(&command).is_ok() {
                crystals -= price;
                commands.push(command);
                // Each builder's Gather back to its seat, after the building.
                let mut back: BTreeMap<ObjectId, Vec<ObjectId>> = BTreeMap::new();
                for o in recruits.iter().filter(|o| crew.contains(&o.id)) {
                    if let Some(patch) = seat_of(o) {
                        back.entry(patch).or_default().push(o.id);
                    }
                }
                for (patch, units) in back {
                    commands.push(order(units, OrderKind::Gather(patch), true));
                }
                self.paid.push((tick, price));
                self.done.push((tick, Done::Ordered { build, crew, price }));
                self.building = Some((build, None));
                self.builds.pop_front();
            }
        }

        if let Some(armory) = armory {
            if let Some(target) = self.armory_rally
                && self.rallied.insert(armory)
            {
                commands.push(PlayerCommand {
                    player,
                    command: Command::Production {
                        objects: vec![armory],
                        command: ProductionCommand::SetRally(target),
                    },
                });
            }
            let price = Order::new(ObjectKind::Armory, ObjectKind::Militant).recipe().cost;
            let train = PlayerCommand {
                player,
                command: Command::Production {
                    objects: vec![armory],
                    command: ProductionCommand::TrainFrom {
                        kind: ObjectKind::Militant,
                        from: RecruitKind::Plain,
                    },
                },
            };
            if self.trained < self.militants
                && price <= crystals
                && !sim.free_units(armory, RecruitKind::Plain).is_empty()
                && sim.validate(&train).is_ok()
            {
                commands.push(train);
                self.paid.push((tick, price));
                self.done.push((tick, Done::Trained { armory, price }));
                self.trained += 1;
            }
        }
        commands
    }

    /// Follows the building under way through the last step's events: its site began, or is complete; or the Build
    /// went before its site began (no Recruit holds it), or the site went before it was complete, and the building
    /// goes back to the front of the line.
    fn follow(&mut self, sim: &Sim) {
        let tick = sim.tick();
        let player = self.player;
        let Some((build, mut site)) = self.building else {
            return;
        };
        let is_ours = |id: ObjectId| {
            sim.object(id).is_some_and(|o| {
                o.owner == Some(player) && o.kind == build.kind && o.placement.is_some_and(|p| p.origin == build.origin)
            })
        };
        for e in sim.events() {
            match *e {
                SimEvent::SiteStarted { site: started, .. } if site.is_none() && is_ours(started) => {
                    site = Some(started);
                    self.done.push((tick, Done::Started { build, site: started }));
                }
                SimEvent::SiteBuilt { site: built } if site == Some(built) => {
                    self.done.push((tick, Done::Built { build, site: built }));
                    self.building = None;
                    return;
                }
                _ => {}
            }
        }
        let holds = |o: &Object| {
            let orders = o.orders.iter().flat_map(|orders| {
                let current = orders.current.iter().map(|active| active.order.kind);
                current.chain(orders.queue.iter().map(|order| order.kind))
            });
            orders.into_iter().any(|kind| {
                matches!(kind, OrderKind::Build { kind, origin, orientation }
                    if (kind, origin, orientation) == (build.kind, build.origin, build.orientation))
            })
        };
        let lost = match site.map(|site| (site, sim.object(site))) {
            None => !sim.objects().any(|o| o.owner == Some(player) && holds(o)),
            Some((site, Some(o))) if !o.is_site() => {
                self.done.push((tick, Done::Built { build, site }));
                self.building = None;
                return;
            }
            Some((_, o)) => o.is_none(),
        };
        if lost {
            self.done.push((tick, Done::Lost { build }));
            self.builds.push_front(build);
            self.building = None;
        } else {
            self.building = Some((build, site));
        }
    }

    /// The patch an idle Recruit is sent to (module docs, 1), its seat taken; `None` with no seat free.
    fn seat(&self, sim: &Sim, taken: &mut BTreeMap<ObjectId, u32>) -> Option<ObjectId> {
        for group in &self.groups {
            let free = (group.iter())
                .filter(|&&(patch, most)| taken[&patch] < most && sim.object(patch).is_some())
                .map(|&(patch, _)| patch);
            if let Some(patch) = free.min_by_key(|patch| taken[patch]) {
                *taken.entry(patch).or_default() += 1;
                return Some(patch);
            }
        }
        None
    }

    /// `build`'s crew ([`Crew`]) among the player's `recruits`, those `given` an order in this tick left out.
    fn crew(
        &self,
        sim: &Sim,
        build: Build,
        recruits: &[&Object],
        storages: &[Shape],
        given: &BTreeSet<ObjectId>,
    ) -> Vec<ObjectId> {
        let free = recruits.iter().filter(|o| !given.contains(&o.id));
        match build.crew {
            Crew::Far => free
                .filter(|o| match seat_of(o) {
                    Some(patch) => (sim.object(patch).map(Shape::of))
                        .is_some_and(|patch| storages.iter().all(|s| s.gap(&patch) > IDEAL_GAP)),
                    None => is_idle(o),
                })
                .map(|o| o.id)
                .collect(),
            Crew::Nearest(n) => {
                let centre = build.centre();
                let mut gathering: Vec<&&Object> = free.filter(|o| gathers(o)).collect();
                gathering.sort_by(|a, b| {
                    let (da, db) = (a.pos.distance_squared(centre), b.pos.distance_squared(centre));
                    da.total_cmp(&db).then(a.id.cmp(&b.id))
                });
                gathering.into_iter().take(n).map(|o| o.id).collect()
            }
        }
    }
}

/// Whether `o` has no order at all.
fn is_idle(o: &Object) -> bool {
    o.orders.as_ref().is_none_or(|orders| orders.is_idle())
}

/// Whether `o`'s current order is a Gather.
fn gathers(o: &Object) -> bool {
    let current = o.orders.as_ref().and_then(|orders| orders.current.as_ref());
    current.is_some_and(|active| matches!(active.order.kind, OrderKind::Gather(_)))
}

/// A Recruit's seat (module docs, 1): the patch of its first Gather, current or queued.
fn seat_of(o: &Object) -> Option<ObjectId> {
    let orders = o.orders.as_ref()?;
    let current = orders.current.iter().map(|active| active.order.kind);
    let mut kinds = current.chain(orders.queue.iter().map(|order| order.kind));
    kinds.find_map(|kind| match kind {
        OrderKind::Gather(patch) => Some(patch),
        _ => None,
    })
}
