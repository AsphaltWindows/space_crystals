//! Workers in the sim (slice M3-H; `crate::economy::gathering`, `crate::orders::gather`): the gate's rules for
//! their orders and their ability, what the removal path does for them, and what the interface asks about them.

use glam::Vec2;

use super::{Reject, Sim};
use crate::economy::gathering::{gatherable, is_drop_off};
use crate::ids::{ObjectId, PlayerId};
use crate::object::ResourceState;
use crate::orders::gather::{self, Claims};
use crate::orders::state::Behaviour;
use crate::orders::world::World;
use crate::orders::{OrderKind, Orders};

/// The Gather order `orders` is carrying out, if it is one: its patch.
fn gathering(orders: &Orders) -> Option<ObjectId> {
    match orders.current.as_ref()?.order.kind {
        OrderKind::Gather(patch) => Some(patch),
        _ => None,
    }
}

impl Sim {
    /// The gate for `OrderKind::Gather(patch)` (`factions/colonists/units.md > Drone > Unit Commands`: "right-click
    /// Space Crystal patch"): the target exists, is a Space Crystals patch with no mining structure on it
    /// ([`Reject::NotGatherable`]; question Q0037 point 11), and `player` sees it, as for every order's target
    /// ([`Reject::NotVisible`], question Q0017 point 2).
    pub(super) fn check_gather(&self, player: PlayerId, patch: ObjectId) -> Result<(), Reject> {
        let target = self.referenced(patch, None)?;
        if !gatherable(&self.objects, patch) {
            return Err(Reject::NotGatherable(patch));
        }
        self.check_target_seen(player, target)
    }

    /// The gate for `OrderKind::DropOff(node)` and `Ability::SetDropOff(node)` (`> Unit Commands`: "own Egress
    /// Node"; `> Drone ObjectInterfaceState`: "an Egress Node or own refinery"): the target exists and is a
    /// drop-off of `player`'s ([`Reject::NotDropOff`]). A player always sees its own structures.
    pub(super) fn check_drop_off(&self, player: PlayerId, node: ObjectId) -> Result<(), Reject> {
        let target = self.referenced(node, None)?;
        if target.owner != Some(player) || !is_drop_off(target.kind) {
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
    /// among them go on at the nearest free patch they know near it, as if they had arrived at an occupied one
    /// (question Q0037 point 7: "treated as occupied, so the redirect rule applies. With no patch left, the Drone
    /// delivers any load and idles"; `gather::free_patch`, `gather::retarget`). At once, not on arrival where the
    /// patch was: no order ever names an object that is gone. A worker on its way back keeps going, and turns to
    /// the new patch after its drop-off.
    pub(super) fn redirect_gatherers(&mut self, gone: &[ObjectId]) {
        let patch_at = |id: ObjectId| -> Option<Vec2> {
            let o = self.objects.get(&id).filter(|_| gone.binary_search(&id).is_ok())?;
            matches!(o.resource, Some(ResourceState::CrystalPatch { .. })).then_some(o.pos)
        };
        let gatherers: Vec<(ObjectId, Vec2)> = self
            .objects
            .values()
            .filter(|o| o.worker.is_some())
            .filter_map(|o| Some((o.id, patch_at(gathering(o.orders.as_ref()?)?)?)))
            .collect();
        if gatherers.is_empty() {
            return;
        }
        // The same scan as an arrival's, on the game as it is now. No claims: workers on their way don't count
        // (`factions/colonists/units.md > Drone > Patch Occupancy (one Drone per patch)`).
        let world = World::new(&self.map, &self.vision, &self.objects, self.tick, self.auto_targeting);
        let claims = Claims::default();
        let moves: Vec<(ObjectId, Option<ObjectId>)> = gatherers
            .into_iter()
            .filter_map(|(id, around)| {
                let o = self.objects.get(&id)?;
                Some((id, gather::free_patch(&world, &claims, o.pos, o.owner, around, gone)))
            })
            .collect();
        for (id, to) in moves {
            let Some(o) = self.objects.get_mut(&id) else {
                continue;
            };
            let (Some(worker), Some(orders)) = (&mut o.worker, &mut o.orders) else {
                continue;
            };
            let goes_on = orders.current.as_mut().is_some_and(|active| match &mut active.state {
                Behaviour::Gather(state) => gather::retarget(&mut active.order.kind, state, worker, to),
                _ => false,
            });
            if !goes_on {
                orders.current = None;
                if let Some(mover) = &mut o.mover {
                    mover.stop();
                }
            }
        }
    }
}
