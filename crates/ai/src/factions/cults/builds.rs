//! The Cults' buildings (`super` module docs, 2): one Build at a time, the first the base wants, at its site
//! (`sites.rs`), built by the Recruits nearest it. Once its site has started, the next may be ordered.
//!
//! - **What it wants** ([`Cults::build`]), in order: a **Storage** once every seat of the field is taken and none is
//!   under construction (`factions/cults/economy.md > Economy check`: "add Storages **as they're needed**: whenever the
//!   mining Recruits outnumber the room at the existing Storages"); the **Armory** while it has none; a **Recruitment
//!   Hall** while it has fewer than [`HALLS`]; the **Gunworks** once [`wants_gunworks`]. A rush's opening wants the
//!   Armory alone (`super` module docs, "The rush"). A kind whose site isn't found
//!   waits [`SITE_RETRY_TICKS`] before the next search, and the kinds after it go on meanwhile. The gate checks the
//!   TechPrerequisites (`factions/cults/units/recruit.md > Constructs`: the Armory "Requires: Storage", the Gunworks
//!   "Requires: Armory").
//! - **The order** (`factions/cults/economy.md > Cults Building Mechanics > Build Command`): one Build to its crew, the
//!   [`STORAGE_BUILDERS`], [`ARMORY_BUILDERS`] or [`HALL_BUILDERS`] Recruits nearest the site that mine or wait, paid
//!   as it is given ("the building's Space Crystals cost is deducted as soon as the build order is given"), each
//!   seated builder's way back to its seat queued after it (Shift: "On completion, all Recruits that participated
//!   remain on the field as ordinary Recruits, free to be issued new orders").
//! - **Following each** ([`Cults::follow`]): done once its site is complete; lost if no Recruit holds the Build before
//!   its site starts (the gate refunds it in full, `factions/cults/economy.md > Cults Building Mechanics > Group
//!   construction refunds`) or if its site goes. A site that no Recruit builds any more gets an Assist from a new crew
//!   (`factions/cults/economy.md > Assist Construction Command`), and a Build whose site hasn't started
//!   [`BUILD_PATIENCE_TICKS`] after it was given is given up, its holders stopped, so that they go back to work. A lost
//!   building is wanted again by the same rules.

use std::collections::{BTreeMap, BTreeSet};

use sc_sim::defs::SitePrice;
use sc_sim::{BuildingOrientation, Command, Object, ObjectId, ObjectKind, OrderKind, Tick, TilePos, Vec2};

use super::field::{Field, Seat, gathers, idle, to_seat};
use super::sites::{self, Site};
use super::{Cults, halls, own, wants_gunworks};
use crate::tuning::build::PRODUCER_SHARE;
use crate::tuning::cults::{
    ARMORY_BUILDERS, BUILD_PATIENCE_TICKS, HALL_BUILDERS, HALL_SEARCH_GU, HALLS, SITE_RETRY_TICKS, STORAGE_BUILDERS,
};
use crate::{Turn, View};

use ObjectKind::{Armory, Gunworks, Recruit, RecruitmentHall, Storage};

/// How far from its anchor the Armory's and the Gunworks' sites are sought, grid units: a technical bound of the
/// search, wide enough for any base.
const NEAR_GU: f32 = 16.0;

/// A building ordered and not complete yet (module docs).
#[derive(Clone, Debug)]
pub(super) struct Pending {
    pub kind: ObjectKind,
    pub origin: TilePos,
    pub orientation: BuildingOrientation,
    /// The tick its Build was given in.
    pub ordered: Tick,
    /// Its site, once it started.
    pub site: Option<ObjectId>,
}

impl Pending {
    /// Whether `o`'s orders, current or queued, hold its Build, or an Assist of its site.
    fn held_by(&self, o: &Object) -> bool {
        let Some(orders) = &o.orders else {
            return false;
        };
        let current = orders.current.iter().map(|active| active.order.kind);
        let mut kinds = current.chain(orders.queue.iter().map(|order| order.kind));
        kinds.any(|kind| match kind {
            OrderKind::Build {
                kind,
                origin,
                orientation,
            } => (kind, origin, orientation) == (self.kind, self.origin, self.orientation),
            OrderKind::Assist(site) => Some(site) == self.site,
            _ => false,
        })
    }
}

impl Cults {
    /// Follows the buildings ordered (module docs): each one's site, once it started; done once that is complete; lost
    /// if no Recruit holds its Build before then, or if its site goes.
    pub(super) fn follow(&mut self, view: &View<'_>) {
        self.builds.retain_mut(|pending| {
            if pending.site.is_none() {
                let at = |o: &&Object| o.placement.is_some_and(|p| p.origin == pending.origin);
                pending.site = own(view, pending.kind).find(at).map(|o| o.id);
            }
            match pending.site {
                Some(site) => view.own_object(site).is_some_and(|o| o.is_site()),
                None => view.own().any(|o| pending.held_by(o)),
            }
        });
    }

    /// The turn's building (module docs): the buildings ordered followed up, then, unless one hasn't started yet, the
    /// next one the base wants ordered. The Recruits it gives orders to join `given`.
    pub(super) fn build(&mut self, turn: &mut Turn<'_, '_>, field: &Field, given: &mut BTreeSet<ObjectId>) {
        let now = turn.view().tick();
        self.retry.retain(|_, &mut from| now < from);
        for pending in self.builds.clone() {
            self.follow_up(turn, field, &pending, given);
        }
        if self.builds.iter().any(|pending| pending.site.is_none()) {
            return;
        }
        let view = turn.view();
        let building = |kind| own(view, kind).any(|o| o.is_site());
        // A rush's opening builds the Armory alone (`super` module docs, "The rush").
        let opening = self.rushing();
        let wanted = [
            (Storage, !opening && self.field_full(field) && !building(Storage)),
            (Armory, own(view, Armory).next().is_none()),
            (RecruitmentHall, !opening && halls(view) < HALLS),
            (Gunworks, !opening && wants_gunworks(view)),
        ];
        for (kind, wants) in wanted {
            if !wants || self.retry.contains_key(&kind) {
                continue;
            }
            let Some(price) = price(kind) else {
                continue;
            };
            if !turn.can_pay(price) {
                return;
            }
            let Some(worker) = free_recruits(view, given).first().map(|o| o.id) else {
                return;
            };
            let Some(site) = self.site_for(turn, field, kind, worker) else {
                self.retry.insert(kind, now + SITE_RETRY_TICKS);
                continue;
            };
            let crew = crew(view, given, builders(kind), site.centre());
            let order = OrderKind::Build {
                kind,
                origin: site.origin,
                orientation: site.orientation,
            };
            if !turn.buy(price, orders(crew.clone(), order)) {
                self.retry.insert(kind, now + SITE_RETRY_TICKS);
                continue;
            }
            self.back_to_seats(turn, field, &crew, site.centre());
            given.extend(crew);
            self.builds.push(Pending {
                kind,
                origin: site.origin,
                orientation: site.orientation,
                ordered: now,
                site: None,
            });
            return;
        }
    }

    /// A building ordered, followed up (module docs): an Assist for a site no Recruit builds any more, and a Build
    /// given up once its site is [`BUILD_PATIENCE_TICKS`] late.
    fn follow_up(&mut self, turn: &mut Turn<'_, '_>, field: &Field, pending: &Pending, given: &mut BTreeSet<ObjectId>) {
        let view = turn.view();
        let now = view.tick();
        let holders: Vec<ObjectId> = view.own().filter(|o| pending.held_by(o)).map(|o| o.id).collect();
        match pending.site {
            Some(site) if holders.is_empty() => {
                let Some(o) = view.own_object(site) else {
                    return;
                };
                let crew = crew(view, given, builders(pending.kind), o.pos);
                if !crew.is_empty() && turn.give(orders(crew.clone(), OrderKind::Assist(site))) {
                    self.back_to_seats(turn, field, &crew, o.pos);
                    given.extend(crew);
                }
            }
            None if now >= pending.ordered + BUILD_PATIENCE_TICKS => {
                let stopped = turn.give(orders(holders.clone(), OrderKind::Stop));
                if stopped {
                    given.extend(holders);
                    self.builds
                        .retain(|b| (b.kind, b.origin) != (pending.kind, pending.origin));
                    self.retry.insert(pending.kind, now + SITE_RETRY_TICKS);
                }
            }
            _ => {}
        }
    }

    /// Where the next `kind` goes (`sites.rs`), by `worker`'s check.
    fn site_for(&self, turn: &Turn<'_, '_>, field: &Field, kind: ObjectKind, worker: ObjectId) -> Option<Site> {
        let view = turn.view();
        let hall = own(view, RecruitmentHall).next().map(|o| o.pos);
        let home = view.my_start().map(|s| s.point()).or(hall)?;
        match kind {
            Storage => {
                let anchor = own(view, Storage).next().map_or(home, |o| o.pos);
                sites::storage(turn, field, worker, anchor)
            }
            RecruitmentHall => sites::hall(turn, field, worker, (home, HALL_SEARCH_GU)),
            Armory => {
                let from = hall.unwrap_or(home);
                let anchor = turn.staging().map_or(from, |s| from + (s - from) * PRODUCER_SHARE);
                sites::near(turn, field, kind, worker, (anchor, NEAR_GU), |_| true)
            }
            _ => {
                let anchor = own(view, Armory).next().map_or(home, |o| o.pos);
                sites::near(turn, field, kind, worker, (anchor, NEAR_GU), |_| true)
            }
        }
    }

    /// Queues the way back to its seat of each of `crew` that has one, from the building at `at` (Shift), those of a
    /// seat together.
    fn back_to_seats(&self, turn: &mut Turn<'_, '_>, field: &Field, crew: &[ObjectId], at: Vec2) {
        let mut by_seat: BTreeMap<Seat, Vec<ObjectId>> = BTreeMap::new();
        for id in crew {
            if let Some(&seat) = self.seated.get(id) {
                by_seat.entry(seat).or_default().push(*id);
            }
        }
        for (seat, units) in by_seat {
            let Some(pair) = field.pair(seat) else {
                continue;
            };
            for command in to_seat(units, pair, at, true) {
                turn.give(command);
            }
        }
    }
}

/// An Order of `kind` to `units`, replacing their orders.
fn orders(units: Vec<ObjectId>, kind: OrderKind) -> Command {
    Command::Order {
        units,
        kind,
        queue: false,
    }
}

/// What a `kind` costs to order (`factions/cults/units/recruit.md > Constructs`), in Space Crystals.
fn price(kind: ObjectKind) -> Option<u32> {
    match kind.def().structure.as_ref()?.site.as_ref()?.price {
        SitePrice::Crystals(amount) => Some(amount),
        SitePrice::SuppliesPerOwned => None,
    }
}

/// How many Recruits build a `kind` (module docs).
fn builders(kind: ObjectKind) -> usize {
    match kind {
        Storage => STORAGE_BUILDERS,
        RecruitmentHall => HALL_BUILDERS,
        _ => ARMORY_BUILDERS,
    }
}

/// The player's Recruits that may build: those that mine or wait (no order at all), but those `given` an order in the
/// turn, in id order.
fn free_recruits<'a>(view: &View<'a>, given: &BTreeSet<ObjectId>) -> Vec<&'a Object> {
    own(view, Recruit)
        .filter(|o| !given.contains(&o.id) && (idle(o) || gathers(o)))
        .collect()
}

/// The crew of a building at `at`: the `n` Recruits that may build nearest it ([`free_recruits`]), ties by id.
fn crew(view: &View<'_>, given: &BTreeSet<ObjectId>, n: usize, at: Vec2) -> Vec<ObjectId> {
    let mut free = free_recruits(view, given);
    free.sort_by(|a, b| {
        (a.pos.distance_squared(at))
            .total_cmp(&b.pos.distance_squared(at))
            .then(a.id.cmp(&b.id))
    });
    free.into_iter().take(n).map(|o| o.id).collect()
}
