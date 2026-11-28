//! The supply loop's part of the economy phase (`crate::supply_run`; `factions/gdo/units/supply_chopper.md`,
//! `factions/gdo/economy.md > SupplyTower`): what the Supply Choppers down on a spot do there, in id order, after the
//! workers' timed work (`super::gathering`), so that a pick-up that ended in the tick has taken its Supply first.
//!
//! - **A pick-up** (`factions/gdo/units/supply_chopper.md > Supply Pickup`): a chopper down on a station with a Pick
//!   Up Supplies there takes all its Supplies that no worker's pick-up has claimed (`orders::world::pick_up`;
//!   `factions/syndicate/units/agent.md > Supplies`: the Agents and the choppers share a station by those claims), as
//!   soon as there are any, through the stations' own take (`stations::take`), which notifies the workers'
//!   owners of a station it empties. Its order ends, and it lifts off: **Placeholder (not in the design: whether a
//!   chopper stays down on a station after its pick-up)**, the station's one spot free for the next.
//! - **A drop-off** (`factions/gdo/units/supply_chopper.md > ObjectInterfaceState[SupplyChopper]`,
//!   `factions/gdo/economy.md > SupplyTower > Landing and Attachment`): a chopper down on a tower of its owner's in
//!   operation, with a Drop Off or an Attach to Tower there or attached to it, pays all it carries into its owner's
//!   Supplies (`SimEvent::ResourcesDelivered`). A Drop Off ends there, and lifts the chopper off the tower unless it is
//!   attached to it.
//! - **The repair** (`factions/gdo/economy.md > SupplyTower > Landing and Attachment`: at no cost, at the repairers'
//!   rate): a chopper down on its attached tower gains its MaxHP over its repair's frames (`repair::terms`, the
//!   repairers' `Terms::frames`) a tick, up to its MaxHP, at the speed of its tower's owner's Power
//!   (`factions/gdo/overview.md > Power`, `PowerSummary::speed`): **the Director's reading**, the repair being one of
//!   the tower's timers, as an Airfield's rearm is (`crate::landing::rearm`).

use std::collections::BTreeMap;

use super::gathering::credit;
use super::stations::{self, is_station, notify_empty};
use super::{PowerSummary, RATIO_ONE};
use crate::augmentation::Research;
use crate::defs::Resource;
use crate::events::SimEvent;
use crate::ids::{ObjectId, PlayerId};
use crate::object::Object;
use crate::orders::OrderKind;
use crate::player::Player;
use crate::repair;
use crate::supply_run::{self, Cargo, is_tower};

/// The phase's part (module docs), for the choppers down on a spot from before this tick (`first_new`): one that came
/// into the game down on its tower in it has its first turn in the next.
pub(super) fn step(
    objects: &mut BTreeMap<ObjectId, Object>,
    players: &mut [Player],
    power: &BTreeMap<PlayerId, PowerSummary>,
    first_new: ObjectId,
    events: &mut Vec<SimEvent>,
) {
    let down: Vec<ObjectId> = (objects.range(..first_new).map(|(_, o)| o))
        .filter(|o| o.chopper.is_some() && crate::landing::down(o))
        .map(|o| o.id)
        .collect();
    for id in down {
        let Some(o) = objects.get(&id) else {
            continue;
        };
        let (Some(at), current) = (crate::landing::at(o), current_order(o)) else {
            continue;
        };
        let Some(spot) = objects.get(&at) else {
            continue;
        };
        let (station, tower, kind, owner) = (
            is_station(spot.kind),
            is_tower(spot) && spot.owner == o.owner,
            spot.kind,
            spot.owner,
        );
        if station && current == Some(OrderKind::PickUpSupplies(at)) {
            pick_up(objects, id, at, events);
        } else if tower {
            drop_off(objects, players, id, at, current, events);
            let speed = owner.map_or(RATIO_ONE, |owner| {
                power.get(&owner).copied().unwrap_or_default().speed(kind)
            });
            if objects.get(&id).and_then(supply_run::attached_to) == Some(at) {
                repair_on(objects, Research::new(players), id, speed);
            }
        }
    }
}

/// The kind of `o`'s current order, if it has one.
fn current_order(o: &Object) -> Option<OrderKind> {
    Some(o.orders.as_ref()?.current.as_ref()?.order.kind)
}

/// Ends chopper `o`'s current order, and lifts it off its spot if `lift` (module docs).
fn end_order(o: &mut Object, lift: bool) {
    if let Some(orders) = &mut o.orders {
        orders.current = None;
    }
    if let (true, true, Some(mover)) = (lift, o.chopper.is_some(), o.mover.as_mut()) {
        crate::landing::lift_off(o.kind, &mut o.landing, &mut mover.rise);
    }
}

/// Chopper `id`, down on station `station` with a Pick Up Supplies there, takes all its unclaimed Supplies, if there
/// are any (module docs).
fn pick_up(objects: &mut BTreeMap<ObjectId, Object>, id: ObjectId, station: ObjectId, events: &mut Vec<SimEvent>) {
    let claimed = claimed_at(objects, station);
    let held = objects.get(&station).and_then(stations::supplies).unwrap_or(0);
    let unclaimed = held.saturating_sub(claimed);
    if unclaimed == 0 {
        return;
    }
    let Some((taken, emptied)) = stations::take(objects, station, unclaimed) else {
        return;
    };
    if emptied {
        notify_empty(objects, station, events);
    }
    let Some(o) = objects.get_mut(&id) else {
        return;
    };
    if let Some(chopper) = &mut o.chopper {
        let carried = chopper.supplies().saturating_add(taken);
        chopper.cargo = (carried > 0).then_some(Cargo::Supplies(carried));
    }
    end_order(o, true);
}

/// The Supplies of station `station` that the workers picking them up there now have claimed
/// (`orders::world::pick_up`): one each.
pub(crate) fn claimed_at(objects: &BTreeMap<ObjectId, Object>, station: ObjectId) -> u32 {
    let picking = (objects.values()).filter(|o| crate::orders::world::pick_up(objects, o) == Some(station));
    u32::try_from(picking.count()).unwrap_or(u32::MAX)
}

/// Chopper `id`, down on its owner's tower `tower` with `current` its order, drops off what it carries there, if its
/// order or its attachment says so, and a Drop Off ends (module docs).
fn drop_off(
    objects: &mut BTreeMap<ObjectId, Object>,
    players: &mut [Player],
    id: ObjectId,
    tower: ObjectId,
    current: Option<OrderKind>,
    events: &mut Vec<SimEvent>,
) {
    let Some(o) = objects.get_mut(&id) else {
        return;
    };
    let home = supply_run::attached_to(o) == Some(tower);
    let ordered = matches!(current, Some(OrderKind::DropOffSupplies(at) | OrderKind::AttachToTower(at)) if at == tower);
    let amount = supply_run::carried(o);
    if amount > 0 && (home || ordered) {
        if let Some(chopper) = &mut o.chopper {
            chopper.cargo = None;
        }
        if let Some(player) = players.iter_mut().find(|p| Some(p.id) == o.owner) {
            credit(player, Resource::Supplies, amount);
        }
        events.push(SimEvent::ResourcesDelivered {
            worker: id,
            drop_off: tower,
            amount,
            resource: Resource::Supplies,
        });
    }
    if current == Some(OrderKind::DropOffSupplies(tower)) {
        end_order(o, !home);
    }
}

/// Chopper `id`, down on its attached tower, gains a tick of the repairers' rate at `speed` (module docs).
fn repair_on(objects: &mut BTreeMap<ObjectId, Object>, research: Research, id: ObjectId, speed: u32) {
    let Some(o) = objects.get(&id) else {
        return;
    };
    let (Some((hp, max)), Some(terms)) = (repair::damage(o, research), repair::terms(o.kind)) else {
        return;
    };
    let rate = max / terms.frames * speed as f32 / RATIO_ONE as f32;
    let restored = (hp + rate).min(max);
    if let Some(o) = objects.get_mut(&id) {
        o.hp = Some(restored);
    }
}
