//! On-site construction in a game (`crate::construction::site`, plan decision T31, slice M4-E): a structure a worker
//! builds where it stands, from the worker's arrival until the structure is complete, cancelled or destroyed. An
//! Agent's Tunnel so far (`factions/syndicate/units/agent.md > Building > Tunnel Construction Flow`).
//!
//! - **The start** ([`Sim::start_site`], from the arrivals' step right after phase 5a, `sim/builds.rs`):
//!   "Construction begins — the partially-built Tunnel appears at the location with the Agent inside it. The Tunnel
//!   starts at 10% HP (ConstructionHP Rule). The Agent is untargetable for the duration of construction." The
//!   structure comes into the game on its footprint, which phase 5b claims, at the rule's maximum HP; the Agent leaves
//!   the map for it (`Sim::take_off_map`, `network::Place::Site`, plan decision T30), and "keeps its load, in the site
//!   and in the network" (design change 0234). The site is a new object, so its first frame is the next tick's
//!   (`ARCHITECTURE.md > Tick pipeline > Spawn tick`): the maximum is 10% at the end of the arrival's tick and the
//!   whole after the site's frames.
//! - **The frames** ([`Sim::step_sites`], in the economy phase beside the transformations): one a tick ("Agent must
//!   be present for the duration", `factions/syndicate/tunnels.md > Construction Time`: one Agent builds at one
//!   speed), the HP following the maximum: "the ConstructionHP formula gives the maximum HP; damage taken stays"
//!   (0234, `transform::follow_max`).
//! - **Completion**: "The Tunnel becomes operational and the Agent is inside the Tunnel Network, available for
//!   redeployment from any Tunnel". Its functions wait for this (`Object::is_operational`); the Agent goes on to the
//!   network, free to leave at once (it walked into no Tunnel: no transit delay, `Held::entered`).
//! - **Cancel** ([`Sim::cancel_site`], `ConstructionCommand::CancelConstruction` on the site: "It does nothing but
//!   Cancel"): the site goes, refunded at the current top price (design changes 0233 and 0284: "construction and
//!   upgrade cancels refund the current top price"), and the Agent comes out.
//! - **Destruction** (`Sim::remove_objects`): "The Agent survives and emerges at the location. The Tunnel is lost and
//!   any Supplies spent are lost."
//!
//! Wherever the site goes, "Where the Agent comes out on cancel or destruction: at the centre of the footprint, which
//! is now free; it then idles" (0234): the removal path's rule for a unit held at a removed object
//! ([`Sim::site_builders`], [`Sim::release_builders`]).

use glam::Vec2;

use super::Sim;
use crate::construction::site::{Builder, Construction, site_def};
use crate::defs::{BuilderGoes, ObjectKind};
use crate::economy::RATIO_ONE;
use crate::events::SimEvent;
use crate::ids::ObjectId;
use crate::math;
use crate::network::Place;
use crate::object::Placement;
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
    pub(super) fn step_sites(&mut self, first_new: ObjectId) {
        let mut completed = Vec::new();
        for o in self.objects.range_mut(..first_new).map(|(_, o)| o) {
            if !o.is_site() {
                continue;
            }
            let before = o.max_hp();
            let done = o
                .construction
                .as_mut()
                .is_some_and(|site| site.progress.step(RATIO_ONE));
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
            }
        }
        self.events.push(SimEvent::SiteCompleted { site: id, worker });
        // A Tunnel that finishes takes back the neutral expansions it covers (slice M4-H, `sim/tunnels.rs`).
        self.reclaim(id);
    }

    /// Cancel Construction on site `site` (module docs): it leaves the game, its worker comes out at the centre of its
    /// footprint ([`Sim::release_builders`]), and its owner gets the current top price back (design change 0233, "the
    /// current construction price, not counting itself": [`Sim::build_price`] once it has gone), with a
    /// `SimEvent::SiteCancelled`. A player whose only structure it was is defeated in the tick
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
        let refund = self.build_price(owner, kind).unwrap_or(0);
        if let Some(bank) = self.players.iter_mut().find(|p| p.id == owner) {
            bank.credit_supplies(refund);
        }
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
