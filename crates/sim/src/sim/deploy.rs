//! Beacon Deployment in a game (`crate::deploy`, slice M3-F): the gate's rule for a Deploy order, the pending Deploys
//! that reserve their sites, and the step right after phase 5a that starts the deployments of the Drones that got to
//! their sites, or fails their orders.

use super::Sim;
use super::gate::Reject;
use super::transform::hold_back;
use crate::deploy::{self, BEACON_DEPLOYMENT, on_site};
use crate::events::SimEvent;
use crate::ids::{ObjectId, PlayerId};
use crate::object::{Object, Placement};
use crate::orders::OrderKind;
use crate::orders::state::Behaviour;
use crate::placement::{self, BuildArea, Method, Mode, Occupant, PlacementCheck, Site};
use crate::scenario::BuildingOrientation;
use crate::transform::Transform;
use crate::units::TilePos;

/// The sites of `o`'s Deploy orders: its current order's first, then its queue's, in order.
fn deploy_sites(o: &Object) -> impl Iterator<Item = Placement> + '_ {
    let orders = o.orders.iter().flat_map(|orders| {
        let current = orders.current.iter().map(|active| &active.order);
        current.chain(&orders.queue)
    });
    orders.filter_map(|order| match order.kind {
        OrderKind::Deploy { origin } => Some(deploy::site(origin)),
        _ => None,
    })
}

impl Sim {
    /// The gate for `OrderKind::Deploy { origin }` (`factions/colonists/economy.md > Beacon Deployment > Deploy
    /// Command`), given `units`, the player's units that take orders: the units among them that deploy
    /// ([`deploy::deploys`]), exactly one, as a Drone becomes the Beacon ("each takes exactly one placement",
    /// `systems/controls/selection_and_hud.md > Shift while placing`; the client picks the one Drone of a group, design
    /// change 0125, slice M3-J1): else [`Reject::NoWorkers`] or [`Reject::SeveralConstructors`]. Its site must be
    /// allowed as far as the player knows, the sites of its pending Deploys reserved ([`Sim::deploy_check`], design
    /// changes 0120 and 0125): else [`Reject::Unplaceable`] with the first problem. Not reported by an event: the client
    /// checks the same before it sends the order.
    pub(super) fn check_deploy(
        &self,
        units: Vec<ObjectId>,
        origin: TilePos,
        queued: bool,
    ) -> Result<Vec<ObjectId>, Reject> {
        let deployers: Vec<ObjectId> = units
            .into_iter()
            .filter(|id| self.objects.get(id).is_some_and(|o| deploy::deploys(o.kind)))
            .collect();
        let drone = match deployers[..] {
            [] => return Err(Reject::NoWorkers),
            [drone] => drone,
            _ => return Err(Reject::SeveralConstructors),
        };
        let check = self.deploy_check(drone, origin, queued).ok_or(Reject::NoWorkers)?;
        if let Some(problem) = check.first_problem() {
            return Err(Reject::Unplaceable(problem));
        }
        Ok(vec![drone])
    }

    /// `core/entities.md > Placement Validation > Lingering build ghosts` (design change 0125): `player`'s pending
    /// Deploys, each a Drone and the footprint of the Beacon it is ordered to deploy into, current or queued: the
    /// Drones in id order, each one's current order first, then its queue's in order. Derived from the orders, so a
    /// site goes with its order: when the deployment starts (the Drone is a Beacon then), is cancelled or replaced by a
    /// new order, or fails. The client draws them for their owner (slice M3-J1); they reserve their sites against the
    /// owner's further placements ([`Sim::deploy_check`], [`Sim::warp_in_check`]) and block no one.
    pub fn pending_deploys(&self, player: PlayerId) -> Vec<(ObjectId, Placement)> {
        self.objects
            .values()
            .filter(|o| o.owner == Some(player))
            .flat_map(|o| deploy_sites(o).map(move |site| (o.id, site)))
            .collect()
    }

    /// The deployments' step, right after phase 5a and before 5b (`crate::deploy`; `ARCHITECTURE.md > Tick
    /// pipeline`): each Drone from before this tick (`first_new`) whose walk to its site is over
    /// ([`Behaviour::Deploy`]'s `arrived`, set by phase 5a), in id order, deploys there or fails its order.
    pub(super) fn step_deployments(&mut self, first_new: ObjectId) {
        let arrived: Vec<(ObjectId, TilePos)> = self
            .objects
            .range(..first_new)
            .filter_map(|(&id, o)| {
                let active = o.orders.as_ref()?.current.as_ref()?;
                match (active.order.kind, &active.state) {
                    (OrderKind::Deploy { origin }, Behaviour::Deploy(state)) if state.arrived => Some((id, origin)),
                    _ => None,
                }
            })
            .collect();
        for (drone, origin) in arrived {
            self.arrive(drone, origin);
        }
    }

    /// Drone `drone` is at the end of its walk to the site at `origin` (`> Deploy Command`, design change 0117). If it
    /// overlaps the footprint ([`on_site`]) and the site is valid on the truth ([`Sim::arrival_check`]), it deploys
    /// there ([`Sim::start_deployment`]). Otherwise "the order is cancelled and the Drone stops and idles" (`>
    /// Placement`): its current order ends, and its queued ones go on (the next one starts in the next tick, as
    /// after an order that completed: `systems/controls/commands.md > Unit Command`), with a
    /// `SimEvent::DeployFailed` whose problems are the arrival check's, none if the Drone stopped short of the
    /// footprint.
    fn arrive(&mut self, drone: ObjectId, origin: TilePos) {
        let Some(o) = self.objects.get(&drone) else {
            return;
        };
        let site = deploy::site(origin);
        let reached = deploy::deploys(o.kind) && on_site(o.kind, o.pos, &site);
        let problems = if reached {
            self.arrival_check(drone, origin)
        } else {
            PlacementCheck::default()
        };
        if reached && problems.is_valid() {
            self.start_deployment(drone, site);
            return;
        }
        let Some(o) = self.objects.get_mut(&drone) else {
            return;
        };
        if let Some(orders) = &mut o.orders {
            orders.current = None;
        }
        if let Some(mover) = &mut o.mover {
            mover.stop();
        }
        self.events.push(SimEvent::DeployFailed {
            drone,
            origin,
            problems,
        });
    }

    /// The check of a Deploy's site on arrival, `core/entities.md > Placement Validation > Worker-Built Structures`:
    /// "validates on arrival", "the real check happens on arrival" (design change 0120). That is [`placement::check`]
    /// with [`Method::WorkerBuilt`] in [`Mode::Arrival`]: every object as it is now, the Drone `drone` left out (design
    /// change 0118), with no sight rule ("No visibility requirement on arrival"). Pending sites don't count: they
    /// reserve their sites against placements, and block no one.
    fn arrival_check(&self, drone: ObjectId, origin: TilePos) -> PlacementCheck {
        let area = BuildArea::default();
        let site = Site {
            map: &self.map,
            area: &area,
            mode: Mode::Arrival,
            method: Method::WorkerBuilt,
        };
        let others = self.objects.values().filter(|o| o.id != drone).map(Occupant::of);
        placement::check(
            &site,
            others,
            BEACON_DEPLOYMENT.kind,
            origin,
            BuildingOrientation::default(),
        )
    }

    /// Drone `drone` begins deploying on `site` (`> A Beacon from the first frame`, design change 0109: "The moment
    /// deployment begins, the Drone becomes a Beacon under deployment"): it becomes the Beacon there and stays the same
    /// object ([`Sim::change_kind`]: its orders, its move and its worker go, it stands at the footprint's centre, and
    /// phase 5b claims the footprint), with its HP and Shields as they were (no frame done, their maxima are still the
    /// Drone's: `> HP/Shields Interpolation`; `crate::shields`) and what a Beacon does held back until completion
    /// ([`hold_back`], design change 0116).
    /// It transforms from the Drone ([`BEACON_DEPLOYMENT`]), keeping the load the Drone carried (design change 0114).
    /// Its first frame is this tick's (`Sim::step_transforms`).
    fn start_deployment(&mut self, drone: ObjectId, site: Placement) {
        let Some(o) = self.objects.get(&drone) else {
            return;
        };
        let (from, hp, shields) = (o.kind, o.hp, o.shields);
        let load = o.worker.map_or(0, |worker| worker.load);
        self.change_kind(drone, BEACON_DEPLOYMENT.kind, Some(site));
        let Some(o) = self.objects.get_mut(&drone) else {
            return;
        };
        o.hp = hp;
        o.shields = shields;
        hold_back(o);
        o.transform = Some(Transform {
            load,
            ..Transform::new(from, None, &BEACON_DEPLOYMENT)
        });
    }
}
