//! The Supply Delivery Stations as workers gather them (slice M4-I) — `core/map.md > SupplyDeliveryStation`,
//! `factions/syndicate/units/agent.md > Gathering > Supplies` (design change 0256, which answers question Q0067).
//!
//! A station holds Supplies (`ResourceState::SupplyStation::current_supplies`), which an Agent picks up one at a time
//! ("CarryCapacity: 1 Supply per trip") and carries to its Tunnel's Supplies hatch (`super::gathering`,
//! `orders::gather`). Its deliveries, DeliverySize once DeliveryInterval has run while it is empty, are the sim's
//! (`Sim::update_supply_stations`).
//!
//! - **A pick-up** is timed work, PickUpDuration, as mining is (`super::gathering::step`), and "any number pick up at
//!   once, as at patches" (0256 point 1). "A pick-up starts only if a Supply remains that no running pick-up has
//!   claimed" (point 2): the job claims a Supply as its pick-up starts, from the stations' unclaimed Supplies as phase
//!   5a began and the claims of the tick (`orders::world::World::unclaimed_supplies`, `orders::gather`).
//! - **"A Supply leaves the station when the pick-up ends"** (point 2, [`take`]): in the economy phase, in the pick-up's
//!   last frame, as the worker takes its load. A pick-up interrupted before that takes nothing (as mining does, design
//!   change 0130), and the station keeps the Supply it had claimed.
//! - **An empty station** (point 3, [`notify_empty`]): the pick-up that takes its last Supply notifies the owners of
//!   the workers gathering there (`SimEvent::SupplyStationEmpty`), and those workers wait beside it for ever, keeping
//!   their job, until a delivery refills it (`orders::gather`).

use std::collections::BTreeMap;

use crate::defs::ObjectKind;
use crate::events::SimEvent;
use crate::ids::{ObjectId, PlayerId};
use crate::object::{Object, ResourceState};
use crate::orders::OrderKind;

/// Whether objects of `kind` are Supply Delivery Stations, where the workers that gather Supplies pick them up.
pub fn is_station(kind: ObjectKind) -> bool {
    kind == ObjectKind::SupplyDeliveryStation
}

/// The Supplies `o` holds now, if it is a station.
pub fn supplies(o: &Object) -> Option<u32> {
    match o.resource {
        Some(ResourceState::SupplyStation { current_supplies, .. }) => Some(current_supplies),
        _ => None,
    }
}

/// A pick-up at `station` ended (0256 point 2: "A Supply leaves the station when the pick-up ends"): takes up to
/// `capacity` Supplies from it. Returns how many it took, and whether that emptied the station; `None` if `station`
/// is no station. A station that holds none gives none (a pick-up that claimed no Supply: a save edited by hand).
pub(super) fn take(objects: &mut BTreeMap<ObjectId, Object>, station: ObjectId, capacity: u32) -> Option<(u32, bool)> {
    let Some(ResourceState::SupplyStation { current_supplies, .. }) = objects.get_mut(&station)?.resource.as_mut()
    else {
        return None;
    };
    let amount = (*current_supplies).min(capacity);
    *current_supplies -= amount;
    Some((amount, amount > 0 && *current_supplies == 0))
}

/// 0256 point 3 ("An empty station: the owner gets a notification"): `station` gave its last Supply, and each player
/// with a worker whose Gather is there gets `SimEvent::SupplyStationEmpty`, in id order. **Placeholder reading**: the
/// station is nobody's, so "the owner" is read as the owner of the Agents picking up there: those whose current order
/// is a Gather on the station, wherever they are in their trips. A worker that comes to a station already empty
/// waits there without a notification.
pub(super) fn notify_empty(objects: &BTreeMap<ObjectId, Object>, station: ObjectId, events: &mut Vec<SimEvent>) {
    let gathers_there = |o: &Object| {
        let current = o.orders.as_ref().and_then(|orders| orders.current.as_ref());
        o.worker.is_some() && current.is_some_and(|active| active.order.kind == OrderKind::Gather(station))
    };
    let mut owners: Vec<PlayerId> = (objects.values())
        .filter(|o| gathers_there(o))
        .filter_map(|o| o.owner)
        .collect();
    owners.sort_unstable();
    owners.dedup();
    events.extend(
        owners
            .into_iter()
            .map(|owner| SimEvent::SupplyStationEmpty { station, owner }),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object::Placement;
    use crate::units::TilePos;

    /// A station holding `current_supplies`, as `Sim::insert_object` makes one.
    fn station(id: u32, current_supplies: u32) -> Object {
        let kind = ObjectKind::SupplyDeliveryStation;
        let placement = Placement::new(TilePos::new(2, 2), kind.def().footprint);
        Object {
            id: ObjectId(id),
            kind,
            owner: None,
            pos: placement.center(),
            prev_pos: placement.center(),
            facing: 0.0,
            prev_facing: 0.0,
            hp: None,
            shields: None,
            placement: Some(placement),
            resource: Some(ResourceState::SupplyStation {
                delivery_size: 4,
                delivery_interval: 960,
                current_supplies,
                next_delivery_in: None,
            }),
            orders: None,
            mover: None,
            attack: None,
            riot_shield: None,
            extractor: None,
            production: None,
            constructor: None,
            worker: None,
            transform: None,
            conversion: None,
            tunnel: None,
            construction: None,
            decay: None,
            popup: None,
            lineage: Vec::new(),
            augmentation: Default::default(),
            energy: None,
        }
    }

    /// 0256 point 2: a pick-up that ends takes its Supply; the last one empties the station, and a station that holds
    /// none gives none.
    #[test]
    fn a_pick_up_takes_a_supply_as_it_ends() {
        let mut objects: BTreeMap<ObjectId, Object> = [(ObjectId(1), station(1, 2))].into();
        assert_eq!(take(&mut objects, ObjectId(1), 1), Some((1, false)));
        assert_eq!(take(&mut objects, ObjectId(1), 1), Some((1, true)));
        assert_eq!(supplies(&objects[&ObjectId(1)]), Some(0));
        assert_eq!(take(&mut objects, ObjectId(1), 1), Some((0, false)), "none to take");
        assert_eq!(take(&mut objects, ObjectId(2), 1), None, "no station");
        assert!(is_station(ObjectKind::SupplyDeliveryStation) && !is_station(ObjectKind::CrystalPatch));
    }
}
