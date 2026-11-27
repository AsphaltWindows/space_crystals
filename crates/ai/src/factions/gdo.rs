//! The GDO's computer player (`factions/gdo/overview.md`; `core/game_rules.md > Computer players (playtest tool)`:
//! its behavior "is the Director's placeholder"). From the GDO's Starting Conditions (`factions/gdo/economy.md >
//! Starting Conditions`: a Deployment Center, an Extraction Facility and two plates), each tick, in this order of
//! priority for the bank:
//!
//! 1. **Placing** what is ready: a plate on the nearest free patch, a building at the nearest spot near the Deployment
//!    Center, a Barracks toward the staging point (`crate::build`). A Place leaves its constructor free from the next
//!    tick.
//! 2. **The Extraction Facility** builds a plate while a patch of its field is still free (`factions/gdo/economy.md >
//!    ExtractionFacility`): "plates on every patch".
//! 3. **The Deployment Center** builds the opening, "Plate, Generator, Generator, Barracks" (with the Extraction
//!    Facility's first plate; `factions/gdo/economy.md > Starting Conditions > Opening Economy Baseline`, the golden
//!    game `gdo_opening`'s line), then a Generator whenever the Power planned falls short of what the plates still to
//!    come and the Barracks still wanted will draw (`factions/gdo/overview.md > GDOResources > Power`), else the second
//!    Barracks once its income allows ([`SECOND_BARRACKS_PLATES`]).
//! 4. **Each Barracks** trains Peacekeepers without pause (`factions/gdo/units/peacekeeper.md`), [`TRAIN_QUEUE`] queued,
//!    while Unit Control has room (`factions/gdo/overview.md > GDOResources > Unit Control`).
//!
//! A step that its bank can't pay for yet holds the bank: the steps after it wait. The army layer (`crate::army`)
//! leads the Peacekeepers, which the Barracks rally to the staging point.
//!
//! **The rush** (`Profile::Rush`, slice A2d; Q0147 point 2: "a small early army and an early first attack"): the
//! Deployment Center opens [`RUSH_OPENING`], a Barracks before the second Generator, and step 4 comes first, before
//! steps 2 and 3, until it has ordered its first wave's [`RUSH_WAVE`] Peacekeepers: Peacekeepers at once, the plates
//! and the Generators from what is left. Its first wave is those [`RUSH_WAVE`] (`FactionAi::waves`); then it plays,
//! and reinforces, as standard does.
//!
//! Every rule and number is a placeholder (Q0147 points 2 and 3, `crate::tuning::gdo`). Costs, times and Power come
//! from the defs.

use std::collections::BTreeMap;

use sc_sim::construction::recipe;
use sc_sim::{Command, ConstructionCommand, Constructor, Object, ObjectId, ObjectKind, ProductionCommand, Tick, Vec2};

use super::{FactionAi, Hook};
use crate::army::Waves;
use crate::build::{plate_spot, plate_spots, spot};
use crate::tuning::build::{PRODUCER_SHARE, SPOT_RETRY_TICKS};
use crate::tuning::gdo::{BARRACKS, OPENING, RUSH_OPENING, RUSH_WAVE, SECOND_BARRACKS_PLATES, TRAIN_QUEUE};
use crate::{Profile, Turn, View};

use ObjectKind::{Barracks, DeploymentCenter, ExtractionFacility, ExtractionPlate, Generator, Peacekeeper};

pub(super) const HOOK: Option<Hook> = Some(new);

fn new(_view: &View<'_>, profile: Profile) -> Box<dyn FactionAi> {
    Box::new(Gdo {
        rush: profile.rushes(),
        ..Gdo::default()
    })
}

/// The GDO's computer player (module docs).
#[derive(Debug, Default)]
struct Gdo {
    /// Whether it plays the rush (module docs).
    rush: bool,
    /// How many of the Deployment Center's opening buildings ([`OPENING`], or [`RUSH_OPENING`]) it has started.
    opened: usize,
    /// The Peacekeepers it has ordered.
    ordered: usize,
    /// For each constructor whose ready building found no spot, the tick from which it looks again.
    retry: BTreeMap<ObjectId, Tick>,
}

impl FactionAi for Gdo {
    fn think(&mut self, turn: &mut Turn<'_, '_>) {
        self.place(turn);
        if self.rushing() {
            self.ordered += train(turn);
            self.construct(turn);
        } else if self.construct(turn) {
            self.ordered += train(turn);
        }
    }

    fn fights(&self, unit: &Object) -> bool {
        unit.kind == Peacekeeper
    }

    fn rallies(&self, building: &Object) -> bool {
        building.kind == Barracks
    }

    /// The army layer's waves (`crate::tuning::army`), but a rush's first: its [`RUSH_WAVE`] Peacekeepers (module
    /// docs).
    fn waves(&self) -> Waves {
        let waves = Waves::default();
        match self.rush {
            true => Waves {
                first: RUSH_WAVE,
                ..waves
            },
            false => waves,
        }
    }
}

impl Gdo {
    /// Whether it still plays its rush opening (module docs): a rush that hasn't ordered its first wave yet.
    fn rushing(&self) -> bool {
        self.rush && self.ordered < RUSH_WAVE
    }

    /// The Deployment Center's opening (module docs).
    fn opening(&self) -> &'static [ObjectKind] {
        match self.rush {
            true => &RUSH_OPENING,
            false => &OPENING,
        }
    }

    /// 1: places each building ready to place, at a spot the placement check takes now; a constructor whose building
    /// finds none looks again [`SPOT_RETRY_TICKS`] later.
    fn place(&mut self, turn: &mut Turn<'_, '_>) {
        let view = turn.view();
        let now = view.tick();
        self.retry.retain(|_, &mut from| now < from);
        let ready: Vec<(ObjectId, ObjectKind, Vec2)> = (view.own())
            .filter_map(|o| Some((o.id, o.constructor?.ready()?, o.pos)))
            .filter(|(id, _, _)| !self.retry.contains_key(id))
            .collect();
        for (id, kind, at) in ready {
            let found = match kind {
                ExtractionPlate => plate_spot(turn, kind, at),
                Barracks => spot(turn, kind, toward(at, turn.staging())),
                _ => spot(turn, kind, at),
            };
            let Some(spot) = found else {
                self.retry.insert(id, now + SPOT_RETRY_TICKS);
                continue;
            };
            let place = ConstructionCommand::Place {
                origin: spot.origin,
                orientation: Default::default(),
            };
            if turn.give(construction(id, place)) {
                turn.take(spot.rect);
            }
        }
    }

    /// 2 and 3: starts the constructors' next buildings. Returns whether the steps after them may spend: `false` when
    /// one waits for the bank.
    fn construct(&mut self, turn: &mut Turn<'_, '_>) -> bool {
        let view = turn.view();
        let idle = |kind| (view.own()).filter(move |o| o.kind == kind && o.constructor.is_some_and(|c| c.is_idle()));
        let mut plan = Plan::of(turn);
        for ef in idle(ExtractionFacility) {
            let placeable = (plate_spots(turn, ef.pos).into_iter())
                .filter_map(|k| k.placement)
                .filter(|p| {
                    view.placement_check(ExtractionPlate, p.origin, Default::default())
                        .is_valid()
                })
                .count();
            if placeable <= plan.pending_plates {
                continue;
            }
            match start(turn, ef, ExtractionPlate) {
                Started::Yes => plan.started(ExtractionPlate),
                Started::Short => return false,
                Started::No => {}
            }
        }
        let opening = self.opening();
        for dc in idle(DeploymentCenter) {
            let next = match opening.get(self.opened) {
                Some(&kind) => Some(kind),
                None => plan.next(),
            };
            let Some(kind) = next else {
                continue;
            };
            match start(turn, dc, kind) {
                Started::Yes => {
                    plan.started(kind);
                    self.opened = (self.opened + 1).min(opening.len());
                }
                Started::Short => return false,
                Started::No => {}
            }
        }
        true
    }
}

/// 4: keeps [`TRAIN_QUEUE`] Peacekeepers queued at each Barracks, while Unit Control has room for them. Returns how
/// many it ordered.
fn train(turn: &mut Turn<'_, '_>) -> usize {
    let view = turn.view();
    let cap = view.army_cap();
    let room_for = cap.map(|cap| cap.resource.cost_of(Peacekeeper));
    // What the queues will take up of Unit Control once their units start (at most: the units started are counted).
    let mut queued: u32 = (view.own())
        .filter_map(|o| o.production.as_ref())
        .map(|p| p.queue.len() as u32)
        .sum();
    let mut ordered = 0;
    for barracks in view.own().filter(|o| o.kind == Barracks) {
        let Some(production) = &barracks.production else {
            continue;
        };
        if production.queue.len() >= TRAIN_QUEUE {
            continue;
        }
        if let (Some(cap), Some(cost)) = (cap, room_for)
            && cap.used.saturating_add(queued).saturating_add(cost) > cap.available
        {
            return ordered;
        }
        let Some(price) = trains(barracks, Peacekeeper) else {
            continue;
        };
        let command = Command::Production {
            objects: vec![barracks.id],
            command: ProductionCommand::Train(Peacekeeper),
        };
        if turn.buy(price, command) {
            queued += 1;
            ordered += 1;
        }
    }
    ordered
}

/// What the player's Power will be, and what the base still wants (module docs, step 3).
struct Plan {
    /// The Power generated less the Power drawn, once the buildings under construction and ready are placed.
    headroom: i32,
    /// The patches still to plate ([`plate_spots`]).
    free: usize,
    /// The plates under construction or ready.
    pending_plates: usize,
    /// The plates on the map.
    plates: usize,
    /// The Barracks on the map, under construction or ready.
    barracks: usize,
}

impl Plan {
    fn of(turn: &Turn<'_, '_>) -> Self {
        let view = turn.view();
        let power = view.power();
        let pending: Vec<ObjectKind> = (view.own())
            .filter_map(|o| match o.constructor? {
                Constructor::Constructing { kind, .. } | Constructor::Ready { kind } => Some(kind),
                Constructor::Idle => None,
            })
            .collect();
        let count = |kind| view.own().filter(|o| o.kind == kind).count();
        let anchor = (view.own())
            .find(|o| o.kind == ExtractionFacility)
            .map_or(Vec2::ZERO, |o| o.pos);
        Self {
            headroom: power.generated as i32 - power.consumed as i32
                + pending.iter().map(|&k| power_of(k)).sum::<i32>(),
            free: plate_spots(turn, anchor).len(),
            pending_plates: pending.iter().filter(|&&k| k == ExtractionPlate).count(),
            plates: count(ExtractionPlate),
            barracks: count(Barracks) + pending.iter().filter(|&&k| k == Barracks).count(),
        }
    }

    /// Counts a building the turn starts.
    fn started(&mut self, kind: ObjectKind) {
        self.headroom += power_of(kind);
        match kind {
            ExtractionPlate => self.pending_plates += 1,
            Barracks => self.barracks += 1,
            _ => {}
        }
    }

    /// The Barracks the base wants: one, and the second once it owns [`SECOND_BARRACKS_PLATES`] plates or has no
    /// patch left to plate.
    fn barracks_wanted(&self) -> usize {
        if self.plates >= SECOND_BARRACKS_PLATES || self.free == 0 {
            BARRACKS
        } else {
            1
        }
    }

    /// The Deployment Center's next building after the opening: a Generator while the Power planned falls short of
    /// what the plates still to come and the Barracks still wanted will draw, else a Barracks still wanted.
    fn next(&self) -> Option<ObjectKind> {
        let plates = self.free.saturating_sub(self.pending_plates) as i32;
        let barracks = self.barracks_wanted().saturating_sub(self.barracks) as i32;
        let draw = plates * -power_of(ExtractionPlate) + barracks * -power_of(Barracks);
        if self.headroom < draw {
            Some(Generator)
        } else if barracks > 0 {
            Some(Barracks)
        } else {
            None
        }
    }
}

/// Whether a constructor started a building.
enum Started {
    Yes,
    /// The bank doesn't cover it yet: it holds the bank.
    Short,
    /// It can't (not on its list, or the gate refused).
    No,
}

/// Has the idle constructor `by` start a `kind`, if the bank covers its cost.
fn start(turn: &mut Turn<'_, '_>, by: &Object, kind: ObjectKind) -> Started {
    let Some(cost) = recipe(by.kind, kind).map(|r| r.cost) else {
        return Started::No;
    };
    if !turn.can_pay(cost) {
        return Started::Short;
    }
    if turn.buy(cost, construction(by.id, ConstructionCommand::Construct(kind))) {
        Started::Yes
    } else {
        Started::No
    }
}

fn construction(by: ObjectId, command: ConstructionCommand) -> Command {
    Command::Construction {
        objects: vec![by],
        command,
    }
}

/// What `producer` pays for a `kind`, if it trains them.
fn trains(producer: &Object, kind: ObjectKind) -> Option<u32> {
    let production = producer.kind.def().structure.as_ref()?.production.as_ref()?;
    production.recipe(kind).map(|r| r.cost)
}

/// The Power a building of `kind` adds (negative: draws).
fn power_of(kind: ObjectKind) -> i32 {
    kind.def().structure.as_ref().map_or(0, |s| s.power)
}

/// The point [`PRODUCER_SHARE`] of the way from `from` to `staging`: where a production building goes.
fn toward(from: Vec2, staging: Option<Vec2>) -> Vec2 {
    staging.map_or(from, |s| from + (s - from) * PRODUCER_SHARE)
}
