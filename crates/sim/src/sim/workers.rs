//! Workers in the sim (slices M3-H and M3-H3; `crate::economy::gathering`, `crate::orders::gather`): the gate's
//! rules for their orders and their ability, what the removal path does for them, and what the interface asks
//! about them.

use glam::Vec2;

use super::{Reject, Sim};
use crate::economy::drop_off::crystal_point_of;
use crate::economy::gathering::{is_drop_off, worker_def};
use crate::ids::{ObjectId, PlayerId};
use crate::object::{Object, ResourceState};
use crate::orders::state::{Behaviour, GatherPhase};
use crate::orders::{OrderKind, Orders};

/// The Gather order `orders` is carrying out, if it is one: its patch.
fn gathering(orders: &Orders) -> Option<ObjectId> {
    match orders.current.as_ref()?.order.kind {
        OrderKind::Gather(patch) => Some(patch),
        _ => None,
    }
}

/// The drop-off worker `o` is on its way back to with a load, in a Gather, if it is: its AssignedDropOff, which the
/// return leg heads for (`orders::gather`, phase ToDropOff), or the one it is unloading at, DroppingOff. (A worker
/// unloading somewhere else, its AssignedDropOff changed meanwhile, isn't on its way to that one; a Drop Off order
/// heads for a drop-off of its own.)
fn returning_to(o: &Object) -> Option<ObjectId> {
    let worker = o.worker.filter(|worker| worker.load > 0)?;
    let drop_off = worker.drop_off?;
    let active = o.orders.as_ref()?.current.as_ref()?;
    let (OrderKind::Gather(_), Behaviour::Gather(gather)) = (active.order.kind, &active.state) else {
        return None;
    };
    let returning = match gather.phase {
        GatherPhase::ToDropOff => true,
        GatherPhase::AtHatch { .. } | GatherPhase::DroppingOff { .. } => gather.node == Some(drop_off),
        _ => false,
    };
    returning.then_some(drop_off)
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

    /// Whether `o` takes `Ability::SetDropOff`: a worker of a type with a lasting AssignedDropOff
    /// (`WorkerDef::keeps_drop_off`: the Drone; `factions/syndicate/units/agent.md > Gathering rules`: "there is no
    /// Set Drop-off command").
    pub(super) fn sets_drop_off(o: &Object) -> bool {
        o.worker.is_some() && worker_def(o.kind).is_some_and(|def| def.keeps_drop_off)
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

    /// The removal path, before the objects `gone` (sorted) leave: design change 0170, answering question Q0046
    /// (`factions/colonists/units.md > Assigned Drop-off > Destroyed mid-trip`): "if its drop-off is destroyed while
    /// the Drone is already on its way there, it switches at once to the nearest own Egress Node (or idles holding its
    /// load if there is none)". "How the loss is detected ... is the Director's choice; the user's goal is to avoid
    /// performance cost": the destruction reassigns, once, and the return leg never checks that its drop-off still
    /// stands. Each worker on its way back to a drop-off among `gone` ([`returning_to`]) takes its owner's nearest
    /// to where it is, by the rule of a return trip that starts without one (design change 0135, `> Drop-off gone`;
    /// [`Sim::nearest_drop_off`]), or none if none is left: its return leg then goes on as one that starts without
    /// one, and with none to take the worker idles holding its load (`orders::gather`).
    ///
    /// The worker turns when it did while the leg checked every tick: in the first phase 5a after the loss, in the
    /// tick of a removal by a command and in the next one for a death in phase 5e. Only the assignment shows sooner
    /// for a death, from the end of its tick. The drop-offs it chooses from are those at the destruction ("at
    /// once"): an Egress Node whose Warp In completes later in that tick isn't one of them.
    ///
    /// Any other worker forgets a drop-off that goes (`Worker::forget`), and takes the nearest as its next return
    /// trip starts (0135).
    pub(super) fn reassign_drop_offs(&mut self, gone: &[ObjectId]) {
        let is_gone = |id: ObjectId| gone.binary_search(&id).is_ok();
        let lost = |id: &ObjectId| self.objects.get(id).is_some_and(|o| is_drop_off(o.kind));
        if !gone.iter().any(lost) {
            return;
        }
        let returning: Vec<(ObjectId, Vec2, Option<PlayerId>)> = (self.objects.values())
            .filter(|o| !is_gone(o.id) && returning_to(o).is_some_and(is_gone))
            .map(|o| (o.id, o.pos, o.owner))
            .collect();
        for (id, pos, owner) in returning {
            let nearest = self.nearest_drop_off(pos, owner, is_gone);
            if let Some(worker) = self.objects.get_mut(&id).and_then(|o| o.worker.as_mut()) {
                worker.drop_off = nearest;
            }
        }
    }

    /// The drop-off of `owner`'s nearest to `pos`, by its centre or its crystal hatch's
    /// (`economy::drop_off::DropPoint::centre`), the lowest id among equals, of those that take
    /// loads (an Egress Node being warped in takes nothing, design change 0116) and aren't `gone`: the rule of the
    /// return leg's `orders::gather::nearest_drop_off`, here on the game rather than on the behaviours' snapshot.
    fn nearest_drop_off(
        &self,
        pos: Vec2,
        owner: Option<PlayerId>,
        is_gone: impl Fn(ObjectId) -> bool,
    ) -> Option<ObjectId> {
        owner?;
        (self.objects.values())
            .filter(|o| o.owner == owner && !o.is_transforming() && !is_gone(o.id))
            .filter_map(|o| Some((o.id, crystal_point_of(o)?.centre.distance_squared(pos))))
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(id, _)| id)
    }
}
