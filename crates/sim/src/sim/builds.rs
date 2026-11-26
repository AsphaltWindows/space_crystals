//! Worker builds in a game (plan decision T51, slices M3-F and M4-E): a worker's order to build where it walks, a
//! Drone's Deploy (`sim/deploy.rs`) and a Build order (an Agent's Tunnel, `crate::construction::site`, whose site is
//! `sim/sites.rs`).
//!
//! - **The lingering build ghosts** ([`Sim::pending_builds`], `core/entities.md > Placement Validation > Lingering
//!   build ghosts`): every worker's build order, current or queued. Derived from the orders, so a ghost goes with its
//!   order: "when construction actually starts", or when "the order is cancelled, replaced by a new order, or fails".
//! - **The price** ([`Sim::build_price`], `factions/syndicate/tunnels.md > Cost Scaling > Construction Cost`, design
//!   change 0233): "Cost = current Tunnels owned (all tiers count), in Supplies", and "Lingering Tunnel ghosts count as
//!   Tunnels owned for the price of further orders, so two orders given in a row cost 1 and 2 (likewise a Tunnel under
//!   construction counts)". It is "Paid when the ghost is placed": an order pays as it is given ([`Sim::pay_build`],
//!   from `Sim::give_order`), and the gate refuses one the player can't pay ([`Sim::check_build`]). "If the order is
//!   cancelled, replaced, or fails before construction starts, it is refunded at the current top price"
//!   ([`Sim::refund_builds`]), which is "the current construction price, not counting itself"
//!   (`factions/syndicate/tunnels.md > Cost Scaling > Upgrade Costs`): the price once it has gone. So "the Supplies
//!   paid for what the player still has always equal the formula's total, so the order of starting and cancelling
//!   can't gain or lose Supplies" (the same).
//! - **The Laser Turret's price** (slice M4-S1, `SitePrice::Crystals`): "Cost: 100 Space Crystals (no Supplies)"
//!   (`factions/syndicate/structures/laser_turret.md`), by the Tunnel's rules, with no scaling: "an Agent's turret is
//!   paid **when the order is given** (ghost placed), like an Agent's Tunnel ..., with the same refunds if the order is
//!   cancelled, replaced, fails, or the Agent is killed before construction starts" (design change 0297 point 4,
//!   `factions/syndicate/structures/laser_turret.md > Construction`), in full, its "current top price" being its price.
//!   Every price is in its def's resource ([`BuildPrice`]), and so is every refund of it; an order that replaces Builds
//!   counts only the refunds in its own resource toward its price. Its cancelled site's 75% is `sim/sites.rs`'s.
//! - **When an order goes** before its site starts, it is refunded: a new order replaces it ([`Sim::drop_orders`],
//!   from `Sim::give_order`), its worker leaves the map with its queue (`Sim::take_off_map`, the same), it fails on
//!   arrival ([`Sim::step_deployments`]), or its worker leaves the game (`Sim::remove_objects`; design change 0289
//!   point 1: "If an Agent is killed before construction starts, its order is refunded at the current top price"). A
//!   defeated player's orders go unrefunded: their bank counts for nothing any more (`economy::defeat`).
//! - **The arrivals** ([`Sim::step_deployments`]): right after phase 5a, each worker whose walk to its site is over
//!   (`orders::build`) starts building there, or its order fails.
//! - **A group's Build** (plan decision T59, slice M5-E; [`shared`]): the workers that share builds (the Recruits) take
//!   a Build together, one order paid once, and build it from outside its site; its refunds go by its ticket, once,
//!   when its last holder drops it.

mod shared;

pub(super) use shared::Ticket;

use super::Sim;
use super::gate::Reject;
use crate::construction::site::{self, BuildPrice, shares_builds, site_def};
use crate::defs::{ObjectKind, Resource};
use crate::deploy::on_site;
use crate::events::SimEvent;
use crate::ids::{ObjectId, PlayerId};
use crate::object::Object;
use crate::orders::build::{PendingBuild, build_site};
use crate::orders::state::Behaviour;
use crate::orders::{Order, OrderKind, Orders};
use crate::placement::PlacementCheck;
use crate::scenario::BuildingOrientation;
use crate::tech;
use crate::units::TilePos;

/// A Build order that went before its site started, to be refunded ([`Sim::refund_builds`]).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(super) struct Dropped {
    /// The worker it was given to.
    pub worker: ObjectId,
    /// The worker's owner, who paid for it.
    pub owner: PlayerId,
    /// What it would have built.
    pub kind: ObjectKind,
    /// (M5-E) A group's Build (plan decision T59): its ticket, which is refunded once, when no unit holds it any more
    /// ([`Sim::refund_builds`]). `None` for one worker's.
    pub ticket: Option<Ticket>,
}

/// A unit's orders, its current one first, then its queue's in order.
fn orders_of(orders: &Orders) -> impl Iterator<Item = &Order> + '_ {
    let current = orders.current.iter().map(|active| &active.order);
    current.chain(&orders.queue)
}

/// The Build orders of `o` as [`Dropped`], the priced ones (a Deploy costs nothing), in order, a group's with its
/// ticket: none for an object without an owner or orders.
pub(super) fn dropped_of(o: &Object) -> Vec<Dropped> {
    let (Some(owner), Some(orders)) = (o.owner, &o.orders) else {
        return Vec::new();
    };
    let worker = o.id;
    orders_of(orders)
        .filter_map(|order| match order.kind {
            OrderKind::Build { kind, .. } => Some(Dropped {
                worker,
                owner,
                kind,
                ticket: Ticket::of(o, order.kind),
            }),
            _ => None,
        })
        .collect()
}

/// The price of a structure of `kind` by its def's rule, `owned` being how many of them the player has for it
/// (`factions/syndicate/tunnels.md > Cost Scaling > Construction Cost`: "Cost = current Tunnels owned"). `None` for a
/// kind no worker builds.
fn price_at(kind: ObjectKind, owned: u32) -> Option<BuildPrice> {
    Some(BuildPrice::of(site_def(kind)?.price, owned))
}

/// The gate's refusal of a Build its player can't pay in `resource` (design change 0233: "The order is refused, with
/// feedback, if the player can't pay").
fn short_of(resource: Resource) -> Reject {
    match resource {
        Resource::Crystals => Reject::InsufficientCrystals,
        Resource::Supplies => Reject::InsufficientSupplies,
    }
}

/// How many of `gone`, which go one at a time in order, of the same kind and owner as `gone[i]` are still there when
/// it goes: those after it.
fn later_alike(gone: &[Dropped], i: usize) -> u32 {
    let alike = |d: &&Dropped| (d.owner, d.kind) == (gone[i].owner, gone[i].kind);
    u32::try_from(gone[i + 1..].iter().filter(alike).count()).unwrap_or(u32::MAX)
}

impl Sim {
    /// `core/entities.md > Placement Validation > Lingering build ghosts` (design change 0125, plan decision T51):
    /// `player`'s pending worker builds, each a worker and the footprint of what it is ordered to build, current or
    /// queued: a Drone's Deploys and an Agent's Build. The workers in id order, each one's current order first, then
    /// its queue's in order. Derived from the orders, so a ghost goes with its order (module docs). The client draws
    /// them for their owner; they reserve their sites against the owner's further placements ([`Sim::deploy_check`],
    /// [`Sim::build_check`], [`Sim::warp_in_check`]), a Tunnel's its whole Tier 1 area against further Tunnels too
    /// (design change 0289 point 2), and block no one.
    ///
    /// (M5-E) A group's Build is one ghost, however many workers hold it (plan decision T59: one ghost per ticket), at
    /// the place of its holder with the lowest id.
    pub fn pending_builds(&self, player: PlayerId) -> Vec<PendingBuild> {
        self.pending_without(player, &[])
    }

    /// `player`'s pending worker builds ([`Sim::pending_builds`]) as they will be once the units `except` have dropped
    /// their orders: a ghost goes only with its last holder.
    pub(crate) fn pending_without(&self, player: PlayerId, except: &[ObjectId]) -> Vec<PendingBuild> {
        let mut pending = Vec::new();
        let mut tickets = Vec::new();
        for o in (self.objects.values()).filter(|o| o.owner == Some(player) && !except.contains(&o.id)) {
            for order in o.orders.iter().flat_map(orders_of) {
                let Some((kind, site)) = build_site(order.kind) else {
                    continue;
                };
                if let Some(ticket) = Ticket::of(o, order.kind) {
                    if tickets.contains(&ticket) {
                        continue;
                    }
                    tickets.push(ticket);
                }
                let worker = o.id;
                pending.push(PendingBuild { worker, kind, site });
            }
        }
        pending
    }

    /// How many structures of `kind` `player` has for its price (`factions/syndicate/tunnels.md > Cost Scaling >
    /// Construction Cost`): those it owns, "all tiers count", "a Tunnel under construction" included, and the
    /// "Lingering Tunnel ghosts": its Build orders of the kind, current or queued, a group's once (its ticket).
    fn owned_for_price(&self, player: PlayerId, kind: ObjectKind) -> u32 {
        let mine = self.objects.values().filter(|o| o.owner == Some(player));
        let standing = mine.clone().filter(|o| o.kind == kind).count();
        let mut tickets = Vec::new();
        let mut ordered = 0_usize;
        for o in mine {
            for order in o.orders.iter().flat_map(orders_of) {
                if !matches!(order.kind, OrderKind::Build { kind: ordered, .. } if ordered == kind) {
                    continue;
                }
                if let Some(ticket) = Ticket::of(o, order.kind) {
                    if tickets.contains(&ticket) {
                        continue;
                    }
                    tickets.push(ticket);
                }
                ordered += 1;
            }
        }
        u32::try_from(standing + ordered).unwrap_or(u32::MAX)
    }

    /// The price of `player`'s next Build of `kind` now (`factions/syndicate/tunnels.md > Cost Scaling > Construction
    /// Cost`, design change 0233), in its def's resource ([`BuildPrice`]): a Tunnel's in Supplies, one for each it has
    /// ([`Sim::owned_for_price`]), so 0 with none (`factions/syndicate/tunnels.md > Losing Tunnels`: "with no Tunnels
    /// owned, a new Tunnel costs 0 Supplies"); a Laser Turret's 100 Space Crystals (module docs). It is also what a
    /// Build that has just gone gives back, the "current top price" "not counting itself" (module docs). The gate, the
    /// client's tooltip and every refund ask it. `None` for a kind no worker builds.
    pub fn build_price(&self, player: PlayerId, kind: ObjectKind) -> Option<BuildPrice> {
        price_at(kind, self.owned_for_price(player, kind))
    }

    /// The gate for `OrderKind::Build { kind, origin, orientation }` (`core/entities.md > Placement Validation >
    /// Worker-Built Structures`, plan decision T51), given `units`, the player's units that take orders:
    /// - **Its builder**: the units among them that build `kind` ([`site::builds`]: the Agents for a Tunnel), exactly
    ///   one, else [`Reject::NoWorkers`] or [`Reject::SeveralConstructors`]. "Each Agent takes exactly one placement"
    ///   (`systems/controls/selection_and_hud.md > AwaitingTarget > Shift while placing`), as it "goes inside the
    ///   structure it builds": a Build `queued` with Shift to one with a Build already, current or queued, is
    ///   [`Reject::ConstructorBusy`] (without Shift, the new one replaces it).
    /// - **Its TechPrerequisite** (slice M4-K, `crate::tech`): the player owns what `kind` requires
    ///   (`StructureDef::requires`: the Laser Turret's Optics Lab, slice M4-S1), else [`Reject::Requires`]. Checked as
    ///   the order is given, which is when it is paid (design change 0233), as a constructor's Construct is, and only
    ///   then: `core/tech_and_research.md > Tech Prerequisites > Check Timing`, "checked only at the moment
    ///   construction is **started** (queued at the constructor, ...)", the moment the order is paid, so the worker's
    ///   arrival doesn't check it again (`Sim::arrive_to_build`), and an Optics Lab lost on the way stops nothing.
    /// - **The price** (design change 0233): "The order is refused, with feedback, if the player can't pay"
    ///   ([`Reject::InsufficientSupplies`], a Laser Turret's [`Reject::InsufficientCrystals`]): the price once the
    ///   Build orders it replaces have gone ([`Sim::build_price`]), out of the bank and their refunds in its resource
    ///   ([`Sim::replacing`]).
    /// - **The site**: allowed as far as the player knows ([`Sim::build_check`]), else [`Reject::Unplaceable`] with
    ///   the first problem.
    ///
    /// Not reported by an event, as a Deploy isn't: the client checks the same before it sends the order.
    ///
    /// (M5-E) Builders that share builds (the Recruits) take it together ([`Sim::check_shared_build`]).
    pub(super) fn check_build(
        &self,
        units: Vec<ObjectId>,
        (kind, origin, orientation): (ObjectKind, TilePos, BuildingOrientation),
        queued: bool,
    ) -> Result<Vec<ObjectId>, Reject> {
        let builders: Vec<&Object> = (units.iter())
            .filter_map(|id| self.objects.get(id))
            .filter(|o| site::builds(o.kind, kind))
            .collect();
        if !builders.is_empty() && builders.iter().all(|o| shares_builds(o.kind)) {
            return self.check_shared_build(&builders, (kind, origin, orientation), queued);
        }
        let builder = match builders[..] {
            [] => return Err(Reject::NoWorkers),
            [builder] => builder,
            _ => return Err(Reject::SeveralConstructors),
        };
        let replaced = dropped_of(builder);
        if queued && !replaced.is_empty() {
            return Err(Reject::ConstructorBusy(builder.id));
        }
        let player = builder.owner.ok_or(Reject::NoWorkers)?;
        self.check_requires(player, tech::structure_requires(kind).iter().copied())?;
        let (refunds, price) = self.replacing(&replaced, player, kind);
        let price = price.ok_or(Reject::NoWorkers)?;
        let bank = self.player(player).map_or(0, |p| price.banked(p));
        if u64::from(bank) + refunds < u64::from(price.amount) {
            return Err(short_of(price.resource));
        }
        let check = (self.build_check(builder.id, kind, origin, orientation, queued)).ok_or(Reject::NoWorkers)?;
        if let Some(problem) = check.first_problem() {
            return Err(Reject::Unplaceable(problem));
        }
        Ok(vec![builder.id])
    }

    /// A Build of `kind` for `player` that replaces the Build orders `replaced`, still in the game: what they give back
    /// as they go, one at a time in order ([`Sim::refund_builds`]), in the new one's resource (a Tunnel's Supplies
    /// don't pay for a Laser Turret), and the new one's price then.
    fn replacing(&self, replaced: &[Dropped], player: PlayerId, kind: ObjectKind) -> (u64, Option<BuildPrice>) {
        let owned_after = |kind: ObjectKind| {
            let going = replaced.iter().filter(|d| (d.owner, d.kind) == (player, kind)).count();
            (self.owned_for_price(player, kind)).saturating_sub(u32::try_from(going).unwrap_or(u32::MAX))
        };
        let price = price_at(kind, owned_after(kind));
        let refunds = (0..replaced.len())
            .filter_map(|i| {
                price_at(
                    replaced[i].kind,
                    owned_after(replaced[i].kind) + later_alike(replaced, i),
                )
            })
            .filter(|refund| price.is_some_and(|price| refund.resource == price.resource))
            .map(|refund| u64::from(refund.amount))
            .sum();
        (refunds, price)
    }

    /// The price of `order` given to unit `id`, as it is given: a Build's ([`Sim::build_price`], the order not yet
    /// among the unit's), with the player who pays it; `None` for any other order.
    pub(super) fn price_of_order(&self, id: ObjectId, order: OrderKind) -> Option<(PlayerId, BuildPrice)> {
        let OrderKind::Build { kind, .. } = order else {
            return None;
        };
        let owner = self.objects.get(&id)?.owner?;
        Some((owner, self.build_price(owner, kind)?))
    }

    /// `player` pays `price` for a Build order it has just given (design change 0233: "an Agent's build order pays the
    /// Supplies when it is given"; a Laser Turret's Space Crystals alike, design change 0297 point 4: "paid **when the
    /// order is given**"). The gate made sure the bank holds them ([`Sim::check_build`]).
    pub(super) fn pay_build(&mut self, player: PlayerId, price: BuildPrice) {
        if let Some(bank) = self.players.iter_mut().find(|p| p.id == player) {
            let paid = price.pay(bank);
            debug_assert!(paid, "the gate checks the price of a Build (`Sim::check_build`)");
        }
    }

    /// Unit `id`'s orders all go, current and queued, and the Build orders among them are refunded
    /// ([`Sim::refund_builds`]): a new order replaces them (`Sim::give_order`), or the unit leaves the map
    /// (`Sim::take_off_map`). Nothing for an object without orders.
    pub(super) fn drop_orders(&mut self, id: ObjectId) {
        let Some(o) = self.objects.get_mut(&id) else {
            return;
        };
        let dropped = dropped_of(o);
        if let Some(orders) = &mut o.orders {
            orders.current = None;
            orders.queue.clear();
        }
        self.refund_builds(&dropped);
    }

    /// Design change 0233: the Build orders `dropped`, which have just gone before their sites started, in order, are
    /// "refunded at the current top price", each the price once it has gone ([`Sim::build_price`]), with a
    /// `SimEvent::BuildRefunded`. They went one at a time in order, so each of those after one still counted when it
    /// went. A Laser Turret's is its whole price, which has no scaling (design change 0297 point 4: "with the same
    /// refunds" as an Agent's Tunnel).
    ///
    /// (M5-E) A group's Build once, by the first of its holders among `dropped`, and only once none holds it any more
    /// ([`Sim::refundable`]): design change 0340 (`factions/cults/economy.md > Cults Building Mechanics > Group
    /// construction refunds`), "the cost is refunded only once **no Recruit of the group still holds the order**".
    pub(super) fn refund_builds(&mut self, dropped: &[Dropped]) {
        let dropped = self.refundable(dropped, &[]);
        for (i, d) in dropped.iter().enumerate() {
            let owned = self
                .owned_for_price(d.owner, d.kind)
                .saturating_add(later_alike(&dropped, i));
            let Some(refund) = price_at(d.kind, owned) else {
                continue;
            };
            if let Some(bank) = self.players.iter_mut().find(|p| p.id == d.owner) {
                refund.refund(bank);
            }
            self.events.push(SimEvent::BuildRefunded {
                worker: d.worker,
                kind: d.kind,
                refund: refund.amount,
            });
        }
    }

    /// The arrivals' step, right after phase 5a and before 5b (`orders::build`; `ARCHITECTURE.md > Tick pipeline`):
    /// each worker from before this tick (`first_new`) whose walk to its site is over (its behaviour's `arrived`, set
    /// by phase 5a), in id order: a Drone deploys there or fails its order (`Sim::arrive`, slice M3-F), and so does a
    /// worker's Build (`Sim::arrive_to_build`). (M5-E) Then each group's Build with such a worker, once, in the order
    /// of its first such holder: its site starts, it fails, or it waits for its others ([`Sim::arrive_together`]).
    pub(super) fn step_deployments(&mut self, first_new: ObjectId) {
        let arrived: Vec<(ObjectId, OrderKind)> = self
            .objects
            .range(..first_new)
            .filter_map(|(&id, o)| {
                let active = o.orders.as_ref()?.current.as_ref()?;
                match (active.order.kind, &active.state) {
                    (OrderKind::Deploy { .. }, Behaviour::Deploy(walk))
                    | (OrderKind::Build { .. }, Behaviour::Build(walk))
                        if walk.arrived =>
                    {
                        Some((id, active.order.kind))
                    }
                    _ => None,
                }
            })
            .collect();
        let mut tickets = Vec::new();
        for (worker, order) in arrived {
            let ticket = self.objects.get(&worker).and_then(|o| Ticket::of(o, order));
            match (order, ticket) {
                (_, Some(ticket)) if !tickets.contains(&ticket) => tickets.push(ticket),
                (_, Some(_)) => {}
                (OrderKind::Deploy { origin }, None) => self.arrive(worker, origin),
                (
                    OrderKind::Build {
                        kind,
                        origin,
                        orientation,
                    },
                    None,
                ) => self.arrive_to_build(worker, (kind, origin, orientation)),
                _ => {}
            }
        }
        for ticket in tickets {
            self.arrive_together(ticket);
        }
    }

    /// Worker `worker` is at the end of its walk to build a `kind` at `origin`, turned by `orientation`
    /// (`core/entities.md > Placement Validation > Worker-Built Structures`: "The worker pathfinds to the location and
    /// validates on arrival"). If it overlaps the footprint ([`on_site`]) and the site is valid on the truth
    /// ([`Sim::arrival_check`]), its site starts there (`Sim::start_site`). Otherwise the order fails: it ends, the
    /// worker stops and idles, and its queued orders go on from the next tick (as for a Drone, `Sim::arrive`), with a
    /// `SimEvent::BuildFailed` whose problems are the arrival check's, none if it stopped short of the footprint; and
    /// the order is refunded (design change 0233: "If the order [...] fails before construction starts, it is refunded
    /// at the current top price").
    fn arrive_to_build(
        &mut self,
        worker: ObjectId,
        (kind, origin, orientation): (ObjectKind, TilePos, BuildingOrientation),
    ) {
        let Some(o) = self.objects.get(&worker) else {
            return;
        };
        let site = build_site(OrderKind::Build {
            kind,
            origin,
            orientation,
        });
        let reached = site.filter(|(_, site)| site::builds(o.kind, kind) && on_site(o.kind, o.pos, site));
        // No TechPrerequisite here (a Laser Turret's Optics Lab): `core/tech_and_research.md > Tech Prerequisites >
        // Check Timing`, "checked only at the moment construction is **started** (queued at the constructor, ...)",
        // which for a worker's Build is the order, paid as it is given (`Sim::check_build`).
        let problems = match reached {
            Some(_) => self.arrival_check(Some(worker), kind, origin, orientation),
            None => PlacementCheck::default(),
        };
        if let Some((_, site)) = reached
            && problems.is_valid()
            && self.next_id < u32::MAX
        {
            self.start_site(worker, kind, site);
            return;
        }
        let Some(o) = self.objects.get_mut(&worker) else {
            return;
        };
        let owner = o.owner;
        if let Some(orders) = &mut o.orders {
            orders.current = None;
        }
        if let Some(mover) = &mut o.mover {
            mover.stop();
        }
        self.events.push(SimEvent::BuildFailed {
            worker,
            kind,
            origin,
            orientation,
            problems,
        });
        if let Some(owner) = owner {
            let ticket = None;
            self.refund_builds(&[Dropped {
                worker,
                owner,
                kind,
                ticket,
            }]);
        }
    }
}
