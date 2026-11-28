//! On-site construction in a game (`crate::construction::site`, plan decision T31, slice M4-E): a structure a worker
//! builds where it stands, from the worker's arrival until the structure is complete, cancelled or destroyed. An
//! Agent's Tunnel (`factions/syndicate/units/agent.md > Building > Tunnel Construction Flow`), the one structure an
//! Agent builds (design change 0359: "Agents no longer build" the Laser Turret, which only Tunnels build).
//!
//! - **The start** ([`Sim::start_site`], from the arrivals' step right after phase 5a, `sim/builds.rs`):
//!   "Construction begins — the partially-built Tunnel appears at the location with the Agent inside it. The Tunnel
//!   starts at 10% HP (ConstructionHP Rule). The Agent is untargetable for the duration of construction." The
//!   structure comes into the game on its footprint, which phase 5b claims, at the rule's maximum HP; the Agent leaves
//!   the map for it (`Sim::take_off_map`, `network::Place::Site`, plan decision T30), and "keeps its load, in the site
//!   and in the network" (design change 0234). The site is a new object, so its first frame is the next tick's
//!   (`docs/architecture/tick-pipeline.md > Tick pipeline > Spawn tick`): the maximum is 10% at the end of the arrival's tick and the
//!   whole after the site's frames.
//! - **The frames** ([`Sim::step_sites`], in the economy phase beside the transformations): one a tick ("Agent must
//!   be present for the duration", `factions/syndicate/tunnels.md > Construction Time`: one Agent builds at one
//!   speed), the HP following the maximum: "the ConstructionHP formula gives the maximum HP; damage taken stays"
//!   (0234, `transform::follow_max`).
//! - **Completion**: "The Tunnel becomes operational and the Agent is inside the Tunnel Network, available for
//!   redeployment from any Tunnel". Its functions wait for this (`Object::is_operational`); the Agent goes on to the
//!   network, free to leave at once (it walked into no Tunnel: no transit delay, `Held::entered`; design change 0301,
//!   answering Q0087: "The Agent that completes a Tunnel can be ejected at once", `factions/syndicate/tunnels.md >
//!   Tunnel Network`).
//! - **Cancel** ([`Sim::cancel_site`], `ConstructionCommand::CancelConstruction` on the site: "It does nothing but
//!   Cancel"): the site goes, refunded at the current top price (design changes 0233 and 0284: "construction and
//!   upgrade cancels refund the current top price"), and the Agent comes out.
//! - **Destruction** (`Sim::remove_objects`): "The Agent survives and emerges at the location. The Tunnel is lost and
//!   any Supplies spent are lost."
//!
//! Wherever the site goes, "Where the Agent comes out on cancel or destruction: at the centre of the footprint, which
//! is now free; it then idles" (0234): the removal path's rule for a unit held at a removed object
//! ([`Sim::site_builders`], [`Sim::release_builders`]).
//!
//! The Cults' buildings, which their Recruits build from outside (`Builder::Recruits`, slice M5-E), take the same
//! steps: their frames, completion and cancel.

use std::collections::BTreeMap;

use glam::Vec2;

use super::Sim;
use crate::construction::site::{Builder, Construction, site_def};
use crate::defs::{BuilderGoes, ObjectKind};
use crate::economy::RATIO_ONE;
use crate::events::SimEvent;
use crate::ids::ObjectId;
use crate::math;
use crate::network::Place;
use crate::object::{Object, Placement};
use crate::orders::OrderKind;
use crate::spot_work;
use crate::transform::follow_max;

impl Sim {
    /// Worker `worker`, on its site, starts building a `kind` there (module docs): the structure comes into the game
    /// on `site`, its owner's, with its HP at the ConstructionHP maximum, and the worker leaves the map for it, its
    /// current order done; a `SimEvent::SiteStarted`. Its queued orders go with the rest of what it does on the map
    /// (`Sim::take_off_map`). The caller found the site valid, with an id left for it.
    pub(super) fn start_site(&mut self, worker: ObjectId, kind: ObjectKind, site: Placement) {
        let (Some(def), Some(owner)) = (site_def(kind), self.objects.get(&worker).and_then(|o| o.owner)) else {
            return;
        };
        // The order is carried out: no refund for it.
        if let Some(orders) = self.objects.get_mut(&worker).and_then(|o| o.orders.as_mut()) {
            orders.current = None;
        }
        // A surface site, which every player may see, its worker inside off the map and so seen by none
        // (`factions/syndicate/units/agent.md > While the Tunnel is under construction`: "What enemies see: the site,
        // like any surface structure, but not the Agent inside it").
        let id = self.insert_object(kind, Some(owner), site.center(), Some(site), None);
        if let Some(o) = self.objects.get_mut(&id) {
            o.construction = Some(Construction::new(def, Builder::Held(worker)));
            o.hp = o.max_hp();
        }
        self.take_off_map(worker, Place::Site(id), false);
        self.events.push(SimEvent::SiteStarted { worker, site: id });
    }

    /// The sites' step of the economy phase, right after the transformations (module docs): each site from before this
    /// tick (`first_new`), in id order, does a frame, its HP following its maximum up; a site whose frames are done is
    /// complete ([`Sim::complete_site`]).
    ///
    /// (M5-E) A site built from outside (`Builder::Recruits`) does a frame for each of its builders within reach of its
    /// footprint in the tick (`spot_work::counts`: the units on an Assist of it there), so that n of them build n times
    /// as fast (`factions/cults/economy.md > BaseConstructionTime Scaling`: "2 Recruits halve it, 3 Recruits reduce it
    /// to a third, and so on"), and none with none: "If all Recruits are pulled off a site, progress stalls at its
    /// current value (no decay)" (`factions/cults/economy.md > Cults Building Mechanics`), its HP as it was.
    pub(super) fn step_sites(&mut self, first_new: ObjectId) {
        let builders = match self.objects.values().any(Object::is_built_from_outside) {
            true => spot_work::counts(&self.objects, self.research(), spot_work::assisted),
            false => BTreeMap::new(),
        };
        let mut completed = Vec::new();
        for o in self.objects.range_mut(..first_new).map(|(_, o)| o) {
            let Some(site) = o.construction else {
                continue;
            };
            let speed = match site.builder {
                Builder::Recruits => RATIO_ONE.saturating_mul(builders.get(&o.id).copied().unwrap_or(0)),
                Builder::Held(_) | Builder::Tunnel(_) => RATIO_ONE,
            };
            if speed == 0 {
                continue;
            }
            let before = o.max_hp();
            let done = o.construction.as_mut().is_some_and(|site| site.progress.step(speed));
            let after = o.max_hp();
            if let (Some(hp), Some(before), Some(after)) = (&mut o.hp, before, after) {
                *hp = follow_max(*hp, before, after);
            }
            if done {
                completed.push(o.id);
            }
        }
        for id in completed {
            self.complete_site(id);
        }
    }

    /// Site `id` is complete (module docs): it is the structure it was building, in operation from now on, and its
    /// worker goes where its def says (`SiteDef::builder_goes`): the Tunnel's Agent into its owner's Tunnel Network,
    /// keeping its load; a `SimEvent::SiteCompleted`.
    fn complete_site(&mut self, id: ObjectId) {
        let Some(o) = self.objects.get_mut(&id) else {
            return;
        };
        let (Some(site), kind) = (o.construction.take(), o.kind) else {
            return;
        };
        let worker = match site.builder {
            Builder::Held(worker) => worker,
            // A Tunnel's expansion (slice M4-F).
            Builder::Tunnel(tunnel) => return self.complete_expansion(id, tunnel),
            // (M5-E) A Cults building, built from outside.
            Builder::Recruits => return self.complete_from_outside(id),
        };
        let tick = self.tick;
        let goes = site_def(kind).map_or(BuilderGoes::Network, |def| def.builder_goes);
        if let Some(held) = self.held.get_mut(&worker).filter(|held| held.place == Place::Site(id)) {
            match goes {
                BuilderGoes::Network => {
                    held.place = Place::Network;
                    held.since = tick;
                    held.entered = false;
                }
                // (M5-E) A site built from outside holds no one.
                BuilderGoes::Outside => {}
            }
        }
        self.events.push(SimEvent::SiteCompleted { site: id, worker });
        // A Tunnel that finishes takes back the neutral expansions it covers (slice M4-H, `sim/tunnels.rs`).
        self.reclaim(id);
    }

    /// (M5-E) Site `id`, built from outside, is complete (`Builder::Recruits`; module docs): what its kind does starts,
    /// as a transformation's does at its completion (`Sim::start_functions`: a Recruitment Hall's production, with the
    /// tick it is complete in, from which its claims count, `Production::completed_at`), and the Assists of it end,
    /// current and queued, so that its builders go on to their next orders from the next tick, a Shift chain's next
    /// building among them, or idle (`factions/cults/economy.md > Cults Building Mechanics`: "On completion, all
    /// Recruits that participated remain on the field as ordinary Recruits, free to be issued new orders"); a
    /// `SimEvent::SiteBuilt`.
    fn complete_from_outside(&mut self, id: ObjectId) {
        self.start_functions(id);
        for orders in self.objects.values_mut().filter_map(|o| o.orders.as_mut()) {
            if (orders.current.as_ref()).is_some_and(|active| active.order.kind == OrderKind::Assist(id)) {
                orders.current = None;
            }
            orders.queue.retain(|order| order.kind != OrderKind::Assist(id));
        }
        self.events.push(SimEvent::SiteBuilt { site: id });
    }

    /// Cancel Construction on site `site` (module docs): it leaves the game, its worker comes out at the centre of its
    /// footprint ([`Sim::release_builders`]), and its owner gets the current top price back (design change 0233, "the
    /// current construction price, not counting itself": [`Sim::build_price`] once it has gone), in its resource, a
    /// Cults building's 75% of it (`construction::site::BuildPrice::cancelled`), with a `SimEvent::SiteCancelled`. A
    /// player whose only structure it was is defeated in the tick
    /// (`economy::defeat`: the defeat check counts the structures at the end of the economy phase).
    pub(super) fn cancel_site(&mut self, site: ObjectId) {
        // A Tunnel's expansion has its own refund (slice M4-F, design change 0254).
        if (self.objects.get(&site)).is_some_and(|o| o.construction.is_some_and(|c| c.builder.tunnel().is_some())) {
            return self.cancel_expansion(site);
        }
        let Some((owner, kind)) = (self.objects.get(&site))
            .filter(|o| o.is_site())
            .and_then(|o| Some((o.owner?, o.kind)))
        else {
            return;
        };
        self.remove_objects(&[site]);
        // The price once it has gone, all of it for a Tunnel, 75% for a Cults building.
        let refund = (site_def(kind)).and_then(|def| Some(self.build_price(owner, kind)?.cancelled(def.price)));
        if let (Some(refund), Some(bank)) = (refund, self.players.iter_mut().find(|p| p.id == owner)) {
            refund.refund(bank);
        }
        let refund = refund.map_or(0, |refund| refund.amount);
        self.events.push(SimEvent::SiteCancelled { site, kind, refund });
    }

    /// The workers held in the sites among `gone`, which are about to leave the game (sorted by id), each with its
    /// site and the centre of its footprint, where it will come out ([`Sim::release_builders`]).
    pub(super) fn site_builders(&self, gone: &[ObjectId]) -> Vec<(ObjectId, ObjectId, Vec2)> {
        (gone.iter())
            .filter_map(|id| {
                let o = self.objects.get(id)?;
                Some((o.construction?.builder.unit()?, *id, o.pos))
            })
            .collect()
    }

    /// The removal path's rule for the units held at a removed object (plan decision T30), for the sites: each worker
    /// of `builders` ([`Sim::site_builders`]) still held in its site, which has left the game, comes out at the centre
    /// of its footprint, now free (design change 0234), idle, with what it carries and facing as it did, with a
    /// `SimEvent::LeftSite`. Whatever took the site: a cancel, a destruction, a debug removal.
    pub(super) fn release_builders(&mut self, builders: &[(ObjectId, ObjectId, Vec2)]) {
        for &(worker, site, centre) in builders {
            let Some(held) = self.held.get(&worker).filter(|held| held.place == Place::Site(site)) else {
                continue;
            };
            let facing = math::facing_vector(held.object.facing);
            if self.put_on_map(worker, centre, facing) {
                self.events.push(SimEvent::LeftSite { worker, site });
            }
        }
    }
}
