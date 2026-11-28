//! The Colonists' field and their Drones' gathering (`super` module docs, step 5; `factions/colonists/units/drone.md >
//! Gathering`): the field is the patches the player knows with crystals left within [`FIELD_RADIUS_GU`] of where its
//! Drones deliver; each Drone with no order is sent to the patch of the field the fewest of the player's Drones
//! gather, by their orders (as `sc_sim::testing::colonists`' `deal` does). The patch occupancy rule deals them out
//! from there (`factions/colonists/units/drone.md > Patch Occupancy`: "When a Drone arrives at an occupied patch, it
//! goes to the **nearest unoccupied patch to itself** instead"). A Drone whose patch runs out with none left near it
//! ends its Gather and is sent again (`systems/mechanics/gathering.md > When a Gather ends`); with the field mined
//! out, it waits, to deploy as a Beacon if one is wanted.

use sc_sim::{Command, ObjectId, OrderKind, Vec2, gu};

use super::Base;
use crate::tuning::colonists::FIELD_RADIUS_GU;
use crate::{Known, Turn, View};

/// The field around `node` (module docs): the Space Crystals patches the player knows, with crystals left as it last
/// saw them, within [`FIELD_RADIUS_GU`] of `node`; the nearest it first, ties by id.
pub(super) fn of<'v>(view: &'v View<'_>, node: Vec2) -> Vec<&'v Known> {
    let radius = gu(FIELD_RADIUS_GU) * gu(FIELD_RADIUS_GU);
    let mut field: Vec<&Known> = (view.patches())
        .filter(|k| k.crystals_left().is_some_and(|left| left > 0) && k.pos.distance_squared(node) <= radius)
        .collect();
    field.sort_by(|a, b| (a.pos.distance_squared(node).total_cmp(&b.pos.distance_squared(node))).then(a.id.cmp(&b.id)));
    field
}

/// The middle of the field's patches; `None` for a field without one.
pub(super) fn centre(field: &[&Known]) -> Option<Vec2> {
    let sum = field.iter().fold(Vec2::ZERO, |sum, k| sum + k.pos);
    (!field.is_empty()).then(|| sum / field.len() as f32)
}

/// Sends each of the player's Drones with no order, but `deployer`, to gather the patch of the field the fewest of its
/// Drones gather (module docs), ties to the patch nearest the Egress Node; the Drones sent to one patch with one
/// command.
pub(super) fn gather(turn: &mut Turn<'_, '_>, base: &Base<'_>, deployer: Option<ObjectId>) {
    let idle: Vec<ObjectId> = (base.workers.iter())
        .filter(|o| Some(o.id) != deployer && o.orders.as_ref().is_some_and(|orders| orders.is_idle()))
        .map(|o| o.id)
        .collect();
    if idle.is_empty() || base.field.is_empty() {
        return;
    }
    // How many of the player's Drones gather each patch of the field, by their orders.
    let mut on: Vec<(ObjectId, usize, Vec<ObjectId>)> = (base.field.iter()).map(|k| (k.id, 0, Vec::new())).collect();
    for o in &base.workers {
        let Some(orders) = &o.orders else {
            continue;
        };
        let current = orders.current.as_ref().map(|active| active.order.kind);
        let kinds = current.into_iter().chain(orders.queue.iter().map(|order| order.kind));
        for kind in kinds {
            if let OrderKind::Gather(patch) = kind
                && let Some(entry) = on.iter_mut().find(|(id, _, _)| *id == patch)
            {
                entry.1 += 1;
            }
        }
    }
    for drone in idle {
        // The field is in order, nearest the Egress Node first: `min_by_key` keeps the first of the fewest.
        if let Some(entry) = on.iter_mut().min_by_key(|(_, n, _)| *n) {
            entry.1 += 1;
            entry.2.push(drone);
        }
    }
    for (patch, _, drones) in on {
        if !drones.is_empty() {
            turn.give(Command::Order {
                units: drones,
                kind: OrderKind::Gather(patch),
                queue: false,
            });
        }
    }
}
