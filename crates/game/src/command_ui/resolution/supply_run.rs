//! The targets of the supply run's commands (`factions/gdo/units/supply_chopper.md >
//! ObjectInterfaceState[SupplyChopper]`, `factions/gdo/economy.md > SupplyTower > ObjectInterfaceState[SupplyTower]`):
//! a Supply Delivery Station for a Pick Up Supplies and a Schedule Deliveries ([`station_target`]), an own Supply Tower
//! for a Drop Off Supplies and an Attach to Tower ([`tower_target`]), as the player knows them
//! (`sc_sim::orders::resolution::Aimed`, the gate's own view: `Sim::aimed`); and whether a Tower is free for the
//! selected choppers ([`tower_free`]), which the pointer previews.

use sc_sim::ObjectId;

use super::{MapTarget, UnitCapabilities};

/// `factions/gdo/units/supply_chopper.md > AwaitingTarget[PickUpSupplies] resolution` and `factions/gdo/economy.md >
/// SupplyTower > AwaitingTarget[ScheduleDeliveries] resolution`: "Left-click SupplyDeliveryStation": the station under
/// a click while Pick Up Supplies or Schedule Deliveries awaits its target
/// (`sc_sim::orders::resolution::Aimed::station`: one the player sees or remembers, by its record), whoever owns it.
/// Anything else is an invalid target ("Left-click anything else: no action").
pub fn station_target(target: &MapTarget) -> Option<ObjectId> {
    target.aimed.filter(|aimed| aimed.station).map(|aimed| aimed.id)
}

/// `factions/gdo/units/supply_chopper.md > AwaitingTarget[DropOffSupplies] resolution` and its Attach to Tower's:
/// "Left-click own SupplyTower with no attached chopper": the Supply Tower under a click while Drop Off Supplies or
/// Attach to Tower awaits its target, if it is one of the player's own in operation
/// (`sc_sim::orders::resolution::Aimed::supply_tower`, the gate's `sc_sim::supply_run::is_tower`). Whether another
/// chopper is attached to it, the gate decides (`sc_sim::Reject::TowerTaken`: a refused click is an invalid target,
/// `crate::command_ui::systems`), and the pointer previews ([`tower_free`]). Anything else is an invalid target.
pub fn tower_target(target: &MapTarget) -> Option<ObjectId> {
    target
        .aimed
        .filter(|aimed| aimed.supply_tower.is_some())
        .map(|aimed| aimed.id)
}

/// Whether the own Supply Tower under `target` is free for one of the choppers among `units`: no chopper is attached
/// to it, or one of them is ("with no attached chopper": the gate's rule, `sc_sim::Reject::TowerTaken`, an attached
/// chopper's own order breaking its attachment). False over anything but an own Supply Tower in operation.
pub fn tower_free(units: &[UnitCapabilities], target: &MapTarget) -> bool {
    let Some(aimed) = target.aimed else {
        return false;
    };
    let ours = |unit: &UnitCapabilities| {
        let chopper = unit.abilities.and_then(|abilities| abilities.chopper);
        chopper.is_some_and(|chopper| chopper.attached == Some(aimed.id))
    };
    match aimed.supply_tower {
        Some(None) => true,
        Some(Some(_)) => units.iter().any(ours),
        None => false,
    }
}
