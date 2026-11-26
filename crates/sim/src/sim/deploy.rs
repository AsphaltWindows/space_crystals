//! Beacon Deployment in a game (`crate::deploy`, slice M3-F): the gate's rule for a Deploy order, and a Drone's arrival
//! at its site, where it deploys or its order fails. Its pending Deploys, which reserve their sites, and the step right
//! after phase 5a that brings it to its arrival are every worker build's (`sim/builds.rs`, plan decision T51).

use super::Sim;
use super::gate::Reject;
use super::transform::hold_back;
use crate::defs::ObjectKind;
use crate::deploy::{self, BEACON_DEPLOYMENT, on_site};
use crate::events::SimEvent;
use crate::ids::ObjectId;
use crate::object::Placement;
use crate::placement::{self, BuildArea, Method, Mode, Occupant, PlacementCheck, Site};
use crate::scenario::BuildingOrientation;
use crate::transform::Transform;
use crate::units::TilePos;

impl Sim {
    /// The gate for `OrderKind::Deploy { origin }` (`factions/colonists/economy.md > Beacon Deployment > Deploy
    /// Command`), given `units`, the player's units that take orders: the units among them that deploy
    /// ([`deploy::deploys`]), exactly one, as a Drone becomes the Beacon ("each takes exactly one placement",
    /// `systems/controls/interface.md > Shift while placing`; the client picks the one Drone of a group, design
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

    /// Drone `drone` is at the end of its walk to the site at `origin` (`factions/colonists/economy.md > Deploy
    /// Command`, design change 0117). If it overlaps the footprint ([`on_site`]) and the site is valid on the truth
    /// ([`Sim::arrival_check`]), it deploys there ([`Sim::start_deployment`]). Otherwise "the order is cancelled and
    /// the Drone stops and idles" (`factions/colonists/economy.md > Placement`): its current order ends, and its queued
    /// ones go on (the next one starts in the next tick, as after an order that completed:
    /// `systems/controls/commands.md > Unit Command`), with a `SimEvent::DeployFailed` whose problems are the arrival
    /// check's, none if the Drone stopped short of the footprint.
    pub(super) fn arrive(&mut self, drone: ObjectId, origin: TilePos) {
        let Some(o) = self.objects.get(&drone) else {
            return;
        };
        let site = deploy::site(origin);
        let reached = deploy::deploys(o.kind) && on_site(o.kind, o.pos, &site);
        let problems = if reached {
            self.arrival_check(
                Some(drone),
                BEACON_DEPLOYMENT.kind,
                origin,
                BuildingOrientation::default(),
            )
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

    /// The check on arrival of a worker build's site, a `kind` at `origin` turned by `orientation` (a Deploy's Beacon,
    /// a Build's structure), `core/entities.md > Placement Validation > Worker-Built Structures`: "validates on
    /// arrival", "the real check happens on arrival" (design change 0120). That is [`placement::check`] with
    /// [`Method::WorkerBuilt`] in [`Mode::Arrival`]: every object as it is now, the worker `worker` left out (design
    /// change 0118), with no sight rule ("No visibility requirement on arrival"). (M5-E) A group's Build leaves nobody
    /// out (`worker` `None`; design change 0339: a Recruit of the order on the footprint blocks it too,
    /// `sim/builds/shared.rs`). Pending sites don't count: they reserve their sites against placements, and block no
    /// one. A Tunnel's area by the truth too: the other Tunnels' tiers as they are (design change 0237: "The real check
    /// on arrival uses the truth"), its cells' terrain as it is (0242).
    pub(super) fn arrival_check(
        &self,
        worker: Option<ObjectId>,
        kind: ObjectKind,
        origin: TilePos,
        orientation: BuildingOrientation,
    ) -> PlacementCheck {
        let area = BuildArea::default();
        let site = Site {
            map: &self.map,
            area: &area,
            mode: Mode::Arrival,
            method: Method::WorkerBuilt,
        };
        let others = self.objects.values().filter(|o| Some(o.id) != worker).map(Occupant::of);
        placement::check(&site, others, kind, origin, orientation)
    }

    /// Drone `drone` begins deploying on `site` (`factions/colonists/economy.md > A Beacon from the first frame`,
    /// design change 0109: "The moment deployment begins, the Drone becomes a Beacon under deployment"): it becomes the
    /// Beacon there and stays the same object ([`Sim::change_kind`]: its orders, its move and its worker go, it stands
    /// at the footprint's centre, and phase 5b claims the footprint), with its HP and Shields as they were (no frame
    /// done, their maxima are still the Drone's: `factions/colonists/economy.md > HP/Shields Interpolation`;
    /// `crate::shields`) and what a Beacon does held back until completion ([`hold_back`], design change 0116).
    /// It transforms from the Drone by its def's deployment ([`deploy::deployment`], [`BEACON_DEPLOYMENT`]), keeping
    /// the load the Drone carried (design change 0114). Its first frame is this tick's (`Sim::step_transforms`).
    fn start_deployment(&mut self, drone: ObjectId, site: Placement) {
        let Some(o) = self.objects.get(&drone) else {
            return;
        };
        let (from, hp, shields) = (o.kind, o.hp, o.shields);
        let Some(recipe) = deploy::deployment(from) else {
            return;
        };
        let load = o.worker.map_or(0, |worker| worker.load);
        self.change_kind(drone, recipe.kind, Some(site));
        let Some(o) = self.objects.get_mut(&drone) else {
            return;
        };
        o.hp = hp;
        o.shields = shields;
        hold_back(o);
        o.transform = Some(Transform {
            load,
            ..Transform::new(from, None, recipe)
        });
    }
}
