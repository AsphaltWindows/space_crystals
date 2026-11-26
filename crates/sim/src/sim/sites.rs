//! On-site construction in a game (`crate::construction::site`, plan decision T31, slice M4-E): a structure a worker
//! builds where it stands, from the worker's arrival until the structure is complete, cancelled or destroyed. An
//! Agent's Tunnel (`factions/syndicate/units/agent.md > Building > Tunnel Construction Flow`), and its Laser Turret by
//! the same flow (slice M4-S1, `factions/syndicate/structures/laser_turret.md > Construction`: "**By an Agent**: built
//! using the same flow as Tunnel construction").
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
//!   network, free to leave at once (it walked into no Tunnel: no transit delay, `Held::entered`; design change 0301,
//!   answering Q0087: "The Agent that completes a Tunnel can be ejected at once", `factions/syndicate/tunnels.md >
//!   Tunnel Network`). A Laser Turret's Agent comes out beside it instead (`BuilderGoes::Beside`;
//!   `factions/syndicate/structures/laser_turret.md > Construction`: "On successful completion, the Agent emerges on
//!   the surface adjacent to the finished structure"): "at the **first clear spot next to the turret**, searched in a
//!   fixed order starting from the side it came from; if none is clear, it waits inside until one is" (design change
//!   0298 point 7; [`Sim::step_beside`], by the spots around a footprint, plan decision T62: `Sim::spot_around`).
//! - **Cancel** ([`Sim::cancel_site`], `ConstructionCommand::CancelConstruction` on the site: "It does nothing but
//!   Cancel"): the site goes, refunded at the current top price (design changes 0233 and 0284: "construction and
//!   upgrade cancels refund the current top price"), a Laser Turret's at 75% of its price, rounded down, like an
//!   expansion's (design change 0297 point 5: "Cancelling a site: 75% refund for both, Agent-built or Tunnel-built";
//!   `construction::site::BuildPrice::cancelled`), and the Agent comes out.
//! - **Destruction** (`Sim::remove_objects`): "The Agent survives and emerges at the location. The Tunnel is lost and
//!   any Supplies spent are lost."
//!
//! Wherever the site goes, "Where the Agent comes out on cancel or destruction: at the centre of the footprint, which
//! is now free; it then idles" (0234): the removal path's rule for a unit held at a removed object
//! ([`Sim::site_builders`], [`Sim::release_builders`]); so does an Agent still waiting inside its finished Laser
//! Turret.
//!
//! The Laser Turret's site is a Tunnel site in all that the design says of the flow: "An **Agent's site is visible**
//! on the surface like a Tunnel site (the Agent inside is not)" (design change 0297 point 6: `vision::hidden` hides no
//! surface site), and "An **Agent's turret doesn't use a Tunnel's operation slot** ..., even inside that Tunnel's area:
//! the Agent builds it, not the Tunnel" (design change 0298 point 9: `tunnel::Operation` is a Tunnel's own). Until
//! slice M4-S2 a turret finished inside an own Tunnel Area stands upright, as outside.

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
use crate::placement::Round;
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
        // A surface site, which every player may see, its worker inside off the map and so seen by none (design change
        // 0297 point 6: "An Agent's site is visible ... (the Agent inside is not)").
        let id = self.insert_object(kind, Some(owner), site.center(), Some(site), None);
        if let Some(o) = self.objects.get_mut(&id) {
            o.construction = Some(Construction::new(def, Builder::Held(worker)));
            o.hp = o.max_hp();
        }
        // The worker builds it, not a Tunnel whose area it may stand in: no Tunnel's operation slot is taken (design
        // change 0298 point 9).
        self.take_off_map(worker, Place::Site(id), false);
        self.events.push(SimEvent::SiteStarted { worker, site: id });
    }

    /// The sites' step of the economy phase, right after the transformations (module docs): each site from before this
    /// tick (`first_new`), in id order, does a frame, its HP following its maximum up; a site whose frames are done is
    /// complete ([`Sim::complete_site`]). Then the workers who come out beside what they finished do, if they can
    /// ([`Sim::step_beside`]).
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
        self.step_beside();
    }

    /// The workers held in the structures they finished, waiting to come out beside them (`BuilderGoes::Beside`: a
    /// Laser Turret's Agent), in id order: each comes out at the first clear spot next to its structure's footprint
    /// (`factions/syndicate/structures/laser_turret.md > Construction`: "the Agent emerges on the surface adjacent to
    /// the finished structure"), facing away from it, idle, with what it carries, with a `SimEvent::LeftSite`; with no
    /// spot clear, "it waits inside until one is" (design change 0298 point 7), a later tick. In the economy phase, as
    /// an ejected unit: it never moves in the tick it appears, and phase 5f sees it.
    ///
    /// The spot is the first clear one of the ring against the footprint (plan decision T62, [`Sim::spot_around`]; for
    /// an Agent beside a turret, the eight directions out of it, a side's middle or a corner), "searched in a fixed
    /// order starting from the side it came from" (0298 point 7; `Round::BothWays`: that side first, then round both
    /// ways, of two equally far the lower number first).
    fn step_beside(&mut self) {
        if self.held.is_empty() {
            return;
        }
        let waiting: Vec<(ObjectId, ObjectId)> = (self.held.iter())
            .filter_map(|(&worker, held)| match held.place {
                Place::Site(site) => (self.objects.get(&site))
                    .is_some_and(|o| !o.is_site())
                    .then_some((worker, site)),
                Place::Network | Place::Building(_) => None,
            })
            .collect();
        for (worker, structure) in waiting {
            let Some(held) = self.held.get(&worker) else {
                continue;
            };
            // The side it came from: it walked onto the site facing the way it went.
            let (kind, came_from) = (held.object.kind, -math::facing_vector(held.object.facing));
            let Some(placement) = self.objects.get(&structure).and_then(|o| o.placement) else {
                continue;
            };
            let Some((spot, outward)) = self.spot_around(kind, &placement, 0..=0, came_from, Round::BothWays) else {
                continue;
            };
            if self.put_on_map(worker, spot, outward) {
                self.events.push(SimEvent::LeftSite {
                    worker,
                    site: structure,
                });
            }
        }
    }

    /// Site `id` is complete (module docs): it is the structure it was building, in operation from now on, and its
    /// worker goes where its def says (`SiteDef::builder_goes`): the Tunnel's Agent into its owner's Tunnel Network,
    /// keeping its load; the Laser Turret's out beside it, later in the step ([`Sim::step_beside`]), held inside it
    /// until then; a `SimEvent::SiteCompleted`.
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
                // Out beside the structure, by the step after the completions (`Sim::step_beside`), not into the
                // network (plan decision T51).
                BuilderGoes::Beside => {}
            }
        }
        self.events.push(SimEvent::SiteCompleted { site: id, worker });
        // A Tunnel that finishes takes back the neutral expansions it covers (slice M4-H, `sim/tunnels.rs`).
        self.reclaim(id);
    }

    /// Cancel Construction on site `site` (module docs): it leaves the game, its worker comes out at the centre of its
    /// footprint ([`Sim::release_builders`]), and its owner gets the current top price back (design change 0233, "the
    /// current construction price, not counting itself": [`Sim::build_price`] once it has gone), in its resource, a
    /// Laser Turret's 75% of it (design change 0297 point 5; `construction::site::BuildPrice::cancelled`), with a
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
        // The price once it has gone, all of it for a Tunnel, 75% for a Laser Turret (design change 0297 point 5).
        let refund = (site_def(kind)).and_then(|def| Some(self.build_price(owner, kind)?.cancelled(def.price)));
        if let (Some(refund), Some(bank)) = (refund, self.players.iter_mut().find(|p| p.id == owner)) {
            refund.refund(bank);
        }
        let refund = refund.map_or(0, |refund| refund.amount);
        self.events.push(SimEvent::SiteCancelled { site, kind, refund });
    }

    /// The workers held in the sites among `gone`, which are about to leave the game (sorted by id), each with its
    /// site and the centre of its footprint, where it will come out ([`Sim::release_builders`]); then, in id order, the
    /// workers still waiting inside a finished structure among them for a spot beside it ([`Sim::step_beside`]), who
    /// come out at the centre of its footprint too.
    pub(super) fn site_builders(&self, gone: &[ObjectId]) -> Vec<(ObjectId, ObjectId, Vec2)> {
        let mut builders: Vec<(ObjectId, ObjectId, Vec2)> = (gone.iter())
            .filter_map(|id| {
                let o = self.objects.get(id)?;
                Some((o.construction?.builder.unit()?, *id, o.pos))
            })
            .collect();
        let beside = |o: &crate::object::Object| {
            !o.is_site() && site_def(o.kind).is_some_and(|def| def.builder_goes == BuilderGoes::Beside)
        };
        if !self.held.is_empty() && gone.iter().any(|id| self.objects.get(id).is_some_and(beside)) {
            builders.extend(self.held.iter().filter_map(|(&worker, held)| {
                let Place::Site(structure) = held.place else {
                    return None;
                };
                let o = self.objects.get(&structure).filter(|o| beside(o))?;
                gone.binary_search(&structure)
                    .is_ok()
                    .then_some((worker, structure, o.pos))
            }));
        }
        builders
    }

    /// The removal path's rule for the units held at a removed object (plan decision T30), for the sites: each worker
    /// of `builders` ([`Sim::site_builders`]) still held in its site, which has left the game, comes out at the centre
    /// of its footprint, now free (design change 0234), idle, with what it carries and facing as it did, with a
    /// `SimEvent::LeftSite`. Whatever took the site: a cancel, a destruction, a debug removal; and so for a finished
    /// Laser Turret its Agent still waits in ([`Sim::step_beside`]).
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
