//! A group's Build (plan decision T59, slice M5-E): the workers that share builds (`WorkerDef::shares_builds`, the
//! Recruits) take a Build together, and build its one site from outside it — `factions/cults/economy.md > Build
//! Command`: "Select one or more Recruits", "All selected Recruits walk to the location and begin construction from
//! outside the site"; `factions/cults/economy.md > Cults Building Mechanics`.
//!
//! - **The ticket** ([`Ticket`]): a Build given to several such workers at once is one order, which each of them holds,
//!   current or queued: the same owner, kind, origin and orientation. Derived from the orders, never stored. It is one
//!   lingering ghost (`Sim::pending_builds`), one structure for the price of the next (`Sim::owned_for_price`), and is
//!   paid once, as it is given ([`Sim::give_shared_build`]; `factions/cults/economy.md > Build Command`: "the
//!   building's Space Crystals cost is deducted as soon as the build order is given (the order requires enough Space
//!   Crystals at that moment)").
//! - **The gate** ([`Sim::check_shared_build`]): every builder among the units takes it, so no
//!   `Reject::SeveralConstructors`; and a Build queued with Shift is a further building, built after the earlier ones,
//!   so no `Reject::ConstructorBusy` (design change 0072: Shift chains). Its TechPrerequisite, its price and its site
//!   are checked as for one worker's, its site with the whole group left out ([`Sim::group_build_check`]).
//! - **The refunds** (`Sim::refund_builds`): in full, once, when its last holder drops it before its site starts:
//!   re-ordered, killed, gone into a building, or failed together (`factions/cults/economy.md > Build Command`: "if
//!   the build order is cancelled before the building is placed (the Recruits are given another order before arriving,
//!   or placement fails validation on arrival), the full Space Crystals cost is refunded"). Design change 0340
//!   (`factions/cults/economy.md > Cults Building Mechanics > Group construction refunds`): "the cost is refunded only
//!   once **no Recruit of the group still holds the order**"; "**All builders killed** before the start: **full
//!   refund**"; "each queued building is refunded in full when its **last holder drops it** before its construction
//!   starts".
//! - **The walk**: each holder walks to its own spot beside the footprint, round it (`crate::spot_work`, design change
//!   0339's wall-hug), the spots' step's, not to the footprint's centre (`orders::build`).
//! - **The start** ([`Sim::arrive_together`], from the arrivals' step right after phase 5a): at the first holder that
//!   ends its walk within reach of the footprint, by the check on arrival on the truth with nobody left out
//!   (`core/entities.md > Placement Validation > Worker-Built Structures`: "validates on arrival"); design change 0339
//!   (`factions/cults/economy.md > Cults Building Mechanics > Group construction rules`): "the site starts **as soon as
//!   the first Recruit of the order arrives** (placement is checked then)", and "units on the footprint at that
//!   moment, **including Recruits of the same order**, block it": a holder still on its way over the footprint fails
//!   it as any unit does, and nothing pushes it off. The site comes into the game on the footprint, with its owner, at
//!   the ConstructionHP Rule's 10%, its production held back until it is complete, as a transformation's
//!   (`Sim::start_functions`); every holder's Build of the ticket, current or queued, becomes an Assist of the site
//!   (`OrderKind::Assist`, `orders::assist`), the current ones keeping their spots and walks. A `SimEvent::SiteStarted`
//!   names the first arrival.
//! - **The failure**: a check on arrival that fails, or every holder's walk over and none within reach, fails the
//!   whole order (0339: "the whole order fails (refunded once in full ...)", "nothing retries"): it goes from every
//!   holder, current and queued, the current ones stop, a `SimEvent::BuildFailed` names the first arrival (or the
//!   holder with the lowest id), and the ticket is refunded once.

use super::{Dropped, orders_of, short_of};
use crate::construction::site::{Builder, Construction, shares_builds, site_def};
use crate::defs::ObjectKind;
use crate::events::SimEvent;
use crate::ids::{ObjectId, PlayerId};
use crate::object::{Object, Placement};
use crate::orders::build::footprint;
use crate::orders::state::{Behaviour, SpotWork};
use crate::orders::{Order, OrderGroup, OrderKind};
use crate::placement::PlacementCheck;
use crate::scenario::BuildingOrientation;
use crate::shape::Shape;
use crate::sim::transform::hold_back;
use crate::sim::{Reject, Sim};
use crate::spot_work::within_reach;
use crate::tech;
use crate::units::TilePos;

/// A group's Build (module docs): the order its holders share.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(in crate::sim) struct Ticket {
    pub owner: PlayerId,
    pub kind: ObjectKind,
    pub origin: TilePos,
    pub orientation: BuildingOrientation,
}

impl Ticket {
    /// The ticket of `order` held by `o`, if it is a Build of a player's worker that shares builds.
    pub fn of(o: &Object, order: OrderKind) -> Option<Ticket> {
        Ticket::held(o.kind, o.owner, order)
    }

    /// The ticket of `order` held by a unit of `kind` and `owner`.
    fn held(kind: ObjectKind, owner: Option<PlayerId>, order: OrderKind) -> Option<Ticket> {
        let OrderKind::Build {
            kind: building,
            origin,
            orientation,
        } = order
        else {
            return None;
        };
        if !shares_builds(kind) {
            return None;
        }
        Some(Ticket {
            owner: owner?,
            kind: building,
            origin,
            orientation,
        })
    }

    /// Its footprint, turned as ordered; `None` for a kind without one.
    fn site(&self) -> Option<Placement> {
        footprint(self.kind, self.origin, self.orientation)
    }
}

/// What the arrivals' step sees of a ticket's holders whose current order it is ([`Sim::arrive_together`]).
#[derive(Default)]
struct Walks {
    /// The holder with the lowest id.
    first: Option<ObjectId>,
    /// The first holder, by id, whose walk is over within reach of the footprint.
    in_reach: Option<ObjectId>,
    /// Whether every holder's walk is over.
    all_over: bool,
}

impl Sim {
    /// The gate for a group's Build (module docs), `builders` being the units of the command that build `kind`, all of
    /// them workers that share builds: its TechPrerequisite ([`Reject::Requires`], as for one worker's, `Sim::check_build`),
    /// its price out of the bank and the refunds of the tickets the order replaces, those whose every holder is among
    /// the builders, without Shift ([`Reject::InsufficientCrystals`]), and its site as far as the player knows, the
    /// group left out ([`Sim::group_build_check`], [`Reject::Unplaceable`]). Every builder takes it.
    pub(in crate::sim) fn check_shared_build(
        &self,
        builders: &[&Object],
        (kind, origin, orientation): (ObjectKind, TilePos, BuildingOrientation),
        queued: bool,
    ) -> Result<Vec<ObjectId>, Reject> {
        let ids: Vec<ObjectId> = builders.iter().map(|o| o.id).collect();
        let player = builders.first().and_then(|o| o.owner).ok_or(Reject::NoWorkers)?;
        self.check_requires(player, tech::structure_requires(kind).iter().copied())?;
        let replaced = if queued { Vec::new() } else { self.dropped_by(&ids) };
        let (refunds, price) = self.replacing(&replaced, player, kind);
        let price = price.ok_or(Reject::NoWorkers)?;
        let bank = self.player(player).map_or(0, |p| price.banked(p));
        if u64::from(bank) + refunds < u64::from(price.amount) {
            return Err(short_of(price.resource));
        }
        let check = (self.group_build_check(&ids, kind, origin, orientation, queued)).ok_or(Reject::NoWorkers)?;
        if let Some(problem) = check.first_problem() {
            return Err(Reject::Unplaceable(problem));
        }
        Ok(ids)
    }

    /// The Build orders that go if every unit of `units` drops its orders, as they would be refunded: each worker's
    /// own, and each ticket once, if every one of its holders is among them ([`Sim::refundable`]).
    fn dropped_by(&self, units: &[ObjectId]) -> Vec<Dropped> {
        let dropped: Vec<Dropped> = (units.iter().filter_map(|id| self.objects.get(id)))
            .flat_map(super::dropped_of)
            .collect();
        self.refundable(&dropped, units)
    }

    /// The Build orders among `dropped` that are refunded as they go (`Sim::refund_builds`): one worker's each, and a
    /// group's once, by the first of its holders among them, and only if no unit but those of `except` holds it any
    /// more (module docs).
    pub(in crate::sim) fn refundable(&self, dropped: &[Dropped], except: &[ObjectId]) -> Vec<Dropped> {
        let mut kept: Vec<Dropped> = Vec::with_capacity(dropped.len());
        for d in dropped {
            if let Some(ticket) = d.ticket
                && (kept.iter().any(|k| k.ticket == Some(ticket)) || self.holds(ticket, except))
            {
                continue;
            }
            kept.push(*d);
        }
        kept
    }

    /// Whether a unit of its owner's, but those of `except`, holds `ticket`, current or queued.
    fn holds(&self, ticket: Ticket, except: &[ObjectId]) -> bool {
        (self.objects.values())
            .filter(|o| o.owner == Some(ticket.owner) && !except.contains(&o.id))
            .any(|o| (o.orders.iter().flat_map(orders_of)).any(|order| Ticket::of(o, order.kind) == Some(ticket)))
    }

    /// Whether `kind`, given to `units`, is a group's Build (module docs): a Build, and every unit a worker that shares
    /// builds (the gate gave it to the builders alone).
    pub(in crate::sim) fn gives_shared_build(&self, units: &[ObjectId], kind: OrderKind) -> bool {
        matches!(kind, OrderKind::Build { .. })
            && !units.is_empty()
            && (units.iter()).all(|id| self.objects.get(id).is_some_and(|o| shares_builds(o.kind)))
    }

    /// A group's Build is given to `units` (module docs, from `Sim::give_order`): without Shift, every unit's orders
    /// go first, the Builds among them refunded as their last holders drop them; then each unit takes the order, in
    /// order group `group', and the price is paid once, if any of them took it (a full queue drops it for a unit).
    pub(in crate::sim) fn give_shared_build(
        &mut self,
        units: &[ObjectId],
        kind: OrderKind,
        queue: bool,
        group: OrderGroup,
    ) {
        if !queue {
            for &id in units {
                self.drop_orders(id);
            }
        }
        let price = units.first().and_then(|&id| self.price_of_order(id, kind));
        let mut given = false;
        for id in units {
            let order = Order {
                kind,
                group,
                slot: None,
            };
            given |= (self.objects.get_mut(id).and_then(|o| o.orders.as_mut())).is_some_and(|o| o.give(order, queue));
        }
        if let (true, Some((player, price))) = (given, price) {
            self.pay_build(player, price);
        }
    }

    /// The arrivals of `ticket` (module docs), from the arrivals' step, which found a holder whose walk is over: its
    /// site starts at the first holder within reach of the footprint if the check on arrival allows it, the order's
    /// own Recruits on the footprint counting as any unit does; the order fails if that check doesn't, or if every
    /// holder's walk is over and none is within reach. Otherwise it waits for the holders on their way.
    pub(in crate::sim) fn arrive_together(&mut self, ticket: Ticket) {
        let Some(site) = ticket.site() else {
            return;
        };
        let walks = self.walks(ticket, &site);
        let Some(first) = walks.first else {
            return;
        };
        match walks.in_reach {
            Some(arrived) => {
                // Nobody left out: any unit on the footprint blocks it, a Recruit of the order too (design change 0339).
                let problems = self.arrival_check(None, ticket.kind, ticket.origin, ticket.orientation);
                if problems.is_valid() && self.next_id < u32::MAX {
                    self.start_shared_site(ticket, arrived, site);
                } else {
                    self.fail_ticket(ticket, arrived, problems);
                }
            }
            None if walks.all_over => self.fail_ticket(ticket, first, PlacementCheck::default()),
            None => {}
        }
    }

    /// The walks of `ticket`'s holders whose current order it is, to their spots beside `site`.
    fn walks(&self, ticket: Ticket, site: &Placement) -> Walks {
        let mut walks = Walks {
            all_over: true,
            ..Walks::default()
        };
        for o in self.objects.values().filter(|o| o.owner == Some(ticket.owner)) {
            let Some(active) = o.orders.as_ref().and_then(|orders| orders.current.as_ref()) else {
                continue;
            };
            let Behaviour::Build(walk) = active.state else {
                continue;
            };
            if Ticket::of(o, active.order.kind) != Some(ticket) {
                continue;
            }
            walks.first.get_or_insert(o.id);
            if !walk.arrived {
                walks.all_over = false;
            } else if walks.in_reach.is_none() && within_reach(&Shape::of(o), site) {
                walks.in_reach = Some(o.id);
            }
        }
        walks
    }

    /// `ticket`'s site starts on `site`, `arrived` its first holder there (module docs): the structure comes into the
    /// game under construction, built from outside (`Builder::Recruits`), and every holder's Build of it becomes an
    /// Assist of it.
    fn start_shared_site(&mut self, ticket: Ticket, arrived: ObjectId, site: Placement) {
        let Some(def) = site_def(ticket.kind) else {
            return;
        };
        let id = self.insert_object(ticket.kind, Some(ticket.owner), site.center(), Some(site), None);
        if let Some(o) = self.objects.get_mut(&id) {
            o.construction = Some(Construction::new(def, Builder::Recruits));
            o.hp = o.max_hp();
            // Its production, a Recruitment Hall's, starts at its completion (`Sim::complete_site`), from which its
            // claims count (`Production::completed_at`).
            hold_back(o);
        }
        for o in (self.objects.values_mut()).filter(|o| o.owner == Some(ticket.owner)) {
            let (kind, owner) = (o.kind, o.owner);
            let Some(orders) = &mut o.orders else {
                continue;
            };
            let is = |order: OrderKind| Ticket::held(kind, owner, order) == Some(ticket);
            if let Some(active) = &mut orders.current
                && is(active.order.kind)
            {
                let spot = match active.state {
                    Behaviour::Build(walk) => walk.spot,
                    _ => None,
                };
                active.order.kind = OrderKind::Assist(id);
                active.state = Behaviour::Assist(SpotWork { spot });
            }
            for order in orders.queue.iter_mut().filter(|order| is(order.kind)) {
                order.kind = OrderKind::Assist(id);
            }
        }
        self.events.push(SimEvent::SiteStarted {
            worker: arrived,
            site: id,
        });
    }

    /// `ticket` fails (module docs): it goes from every holder, current and queued, those whose current order it was
    /// stop, and it is refunded once, after a `SimEvent::BuildFailed` that names `worker` with `problems`.
    pub(in crate::sim) fn fail_ticket(&mut self, ticket: Ticket, worker: ObjectId, problems: PlacementCheck) {
        for o in (self.objects.values_mut()).filter(|o| o.owner == Some(ticket.owner)) {
            let (kind, owner) = (o.kind, o.owner);
            let Some(orders) = &mut o.orders else {
                continue;
            };
            let is = |order: OrderKind| Ticket::held(kind, owner, order) == Some(ticket);
            if (orders.current.as_ref()).is_some_and(|active| is(active.order.kind)) {
                orders.current = None;
                if let Some(mover) = &mut o.mover {
                    mover.stop();
                }
            }
            orders.queue.retain(|order| !is(order.kind));
        }
        self.events.push(SimEvent::BuildFailed {
            worker,
            kind: ticket.kind,
            origin: ticket.origin,
            orientation: ticket.orientation,
            problems,
        });
        self.refund_builds(&[Dropped {
            worker,
            owner: ticket.owner,
            kind: ticket.kind,
            ticket: Some(ticket),
        }]);
    }
}
