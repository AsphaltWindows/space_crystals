//! Workers in the sim (slices M3-H and M3-H3; `crate::economy::gathering`, `crate::orders::gather`): the gate's
//! rules for their orders and their ability, what the removal path does for them, and what the interface asks
//! about them.

use glam::Vec2;

use super::{Reject, Sim};
use crate::economy::gathering::is_drop_off;
use crate::ids::{ObjectId, PlayerId};
use crate::object::ResourceState;
use crate::orders::state::Behaviour;
use crate::orders::{OrderKind, Orders};

/// The Gather order `orders` is carrying out, if it is one: its patch.
fn gathering(orders: &Orders) -> Option<ObjectId> {
    match orders.current.as_ref()?.order.kind {
        OrderKind::Gather(patch) => Some(patch),
        _ => None,
    }
}

impl Sim {
    /// The gate for `OrderKind::Gather(patch)`: a patch `player` sees or remembers, and takes to be gatherable, as
    /// it knows it (design change 0139, `Sim::check_gather_target`).
    pub(super) fn check_gather(&self, player: PlayerId, patch: ObjectId) -> Result<(), Reject> {
        self.check_gather_target(player, patch)
    }

    /// The gate for `OrderKind::DropOff(node)` and `Ability::SetDropOff(node)` (`> Unit Commands`: "own Egress
    /// Node"; `> Drone ObjectInterfaceState`: "an Egress Node or own refinery"): the target exists and is a
    /// drop-off of `player`'s, complete: an Egress Node being warped in takes nothing yet (design change 0116: "A
    /// Drone can't deliver to an Egress Node under Warp In", `crate::transform`) ([`Reject::NotDropOff`]). A player
    /// always sees its own structures.
    pub(super) fn check_drop_off(&self, player: PlayerId, node: ObjectId) -> Result<(), Reject> {
        let target = self.referenced(node, None)?;
        if target.owner != Some(player) || !is_drop_off(target.kind) || target.is_transforming() {
            return Err(Reject::NotDropOff(node));
        }
        Ok(())
    }

    /// The workers among `units`, which take a Gather or a Drop Off order; the others ignore it
    /// ([`Reject::NoWorkers`] if none is one).
    pub(super) fn workers_among(&self, mut units: Vec<ObjectId>) -> Result<Vec<ObjectId>, Reject> {
        units.retain(|id| self.objects.get(id).is_some_and(|o| o.worker.is_some()));
        if units.is_empty() {
            return Err(Reject::NoWorkers);
        }
        Ok(units)
    }

    /// The workers among `workers` that carry something, which take a Drop Off order (design change 0140, `>
    /// Drop Off order > Empty Drones`: "Drop Off is only resolved for a Drone that is carrying something"; there is
    /// no empty Drop Off order). The Director's choice for the gate: the empty ones are left out, and the command
    /// is rejected if none is left ([`Reject::NothingToDeliver`]). The client gives an empty Drone a Move to the
    /// node instead (slice M3-J2).
    pub(super) fn carrying_among(&self, mut workers: Vec<ObjectId>) -> Result<Vec<ObjectId>, Reject> {
        workers.retain(|id| {
            let worker = self.objects.get(id).and_then(|o| o.worker);
            worker.is_some_and(|worker| worker.load > 0)
        });
        if workers.is_empty() {
            return Err(Reject::NothingToDeliver);
        }
        Ok(workers)
    }

    /// `factions/colonists/economy.md > Egress Node > Egress Node ObjectInterfaceState`, F: Select Returning
    /// Drones — "all Drones whose AssignedDropOff is this Egress Node and which are currently executing a Gather
    /// command — whether mining, walking to the patch, carrying resources back, or dropping off", in id order.
    pub fn returning_drones(&self, node: ObjectId) -> Vec<ObjectId> {
        self.objects
            .values()
            .filter(|o| {
                o.worker.is_some_and(|worker| worker.drop_off == Some(node))
                    && o.orders.as_ref().and_then(gathering).is_some()
            })
            .map(|o| o.id)
            .collect()
    }

    /// The removal path, before the objects `gone` (sorted) leave: the workers carrying out a Gather at a patch
    /// among them keep it, and keep heading for where the patch was (`orders::state::Gather::spot`). Design change
    /// 0134 (`factions/colonists/units.md > Patch Occupancy > A patch that runs out`): "a depleted patch is treated
    /// like an occupied one". The behaviour looks round for another patch when the worker is there: at once for
    /// one waiting at it or carrying off its last load, on arrival for one walking to it, after its drop-off for
    /// one carrying a load back (`orders::gather`). The order goes on naming the patch
    /// (`orders::Order::outlives_target`).
    pub(super) fn keep_gatherers_on_spots(&mut self, gone: &[ObjectId]) {
        let spot_of = |id: ObjectId| -> Option<Vec2> {
            let o = self.objects.get(&id).filter(|_| gone.binary_search(&id).is_ok())?;
            matches!(o.resource, Some(ResourceState::CrystalPatch { .. })).then_some(o.pos)
        };
        let gatherers: Vec<(ObjectId, Vec2)> = self
            .objects
            .values()
            .filter(|o| o.worker.is_some())
            .filter_map(|o| Some((o.id, spot_of(gathering(o.orders.as_ref()?)?)?)))
            .collect();
        for (id, spot) in gatherers {
            let active = self
                .objects
                .get_mut(&id)
                .and_then(|o| o.orders.as_mut())
                .and_then(|orders| orders.current.as_mut());
            if let Some(active) = active
                && let Behaviour::Gather(state) = &mut active.state
            {
                state.spot = Some(spot);
            }
        }
    }
}
