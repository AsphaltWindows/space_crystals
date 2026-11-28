//! The gate's checks of the supply loop (`crate::supply_run`): a Supply Chopper's three orders
//! (`OrderKind::PickUpSupplies`, `OrderKind::DropOffSupplies`, `OrderKind::AttachToTower`;
//! `factions/gdo/units/supply_chopper.md > ObjectInterfaceState[SupplyChopper]`) and a Supply Tower's Schedule
//! Deliveries (`Command::Schedule`; `factions/gdo/economy.md > SupplyTower > AwaitingTarget[ScheduleDeliveries]
//! resolution`).

use super::Checked;
use crate::economy::stations::is_station;
use crate::ids::{ObjectId, PlayerId};
use crate::object::Object;
use crate::orders::OrderKind;
use crate::sim::{Reject, Sim};
use crate::supply_run::{self, attached_chopper, carried, is_tower};

impl Sim {
    /// The gate for a chopper's order of `kind` (one of the three; any other passes as it is): the Supply Choppers
    /// among `units` take it, the others ignore it, else [`Reject::LacksAbility`].
    /// - **Pick Up Supplies** (`factions/gdo/units/supply_chopper.md > AwaitingTarget[PickUpSupplies] resolution`:
    ///   "Left-click SupplyDeliveryStation"): on a station the player knows ([`Sim::check_target_known`]), as it knows
    ///   it, else [`Reject::NotAStation`].
    /// - **Drop Off Supplies** and **Attach to Tower** (`factions/gdo/units/supply_chopper.md >
    ///   AwaitingTarget[DropOffSupplies] resolution`, `factions/gdo/units/supply_chopper.md >
    ///   AwaitingTarget[AttachToTower] resolution`: "Left-click own SupplyTower with no attached chopper"): on a Supply
    ///   Tower of the player's in operation ([`supply_run::is_tower`]), else [`Reject::NotASupplyTower`], that has no
    ///   chopper attached but one of `units` ([`Reject::TowerTaken`]: the one attached is given the order, which
    ///   breaks its attachment, `Sim::apply_command`). A Drop Off goes to the choppers that carry Supplies, else
    ///   [`Reject::NothingToDeliver`] ("Drop Off Supplies [...] Only available if chopper is carrying supplies").
    pub(super) fn check_supply_run(
        &self,
        player: PlayerId,
        units: Vec<ObjectId>,
        kind: OrderKind,
    ) -> Result<Vec<ObjectId>, Reject> {
        let choppers: Vec<ObjectId> = (units.into_iter())
            .filter(|id| self.objects.get(id).is_some_and(|o| o.chopper.is_some()))
            .collect();
        if choppers.is_empty() {
            return Err(Reject::LacksAbility);
        }
        match kind {
            OrderKind::PickUpSupplies(station) => self.check_station(player, station)?,
            OrderKind::DropOffSupplies(tower) | OrderKind::AttachToTower(tower) => {
                self.check_tower(player, tower, &choppers)?;
            }
            _ => return Ok(choppers),
        }
        if !matches!(kind, OrderKind::DropOffSupplies(_)) {
            return Ok(choppers);
        }
        let carrying: Vec<ObjectId> = (choppers.into_iter())
            .filter(|id| self.objects.get(id).is_some_and(|o| carried(o) > 0))
            .collect();
        if carrying.is_empty() {
            return Err(Reject::NothingToDeliver);
        }
        Ok(carrying)
    }

    /// The gate for a Schedule Deliveries of `towers` from `station` (`factions/gdo/economy.md > SupplyTower >
    /// ObjectInterfaceState[SupplyTower]`: "**S: Schedule Deliveries** [...] Only available if tower has an attached
    /// chopper"; `factions/gdo/economy.md > SupplyTower > AwaitingTarget[ScheduleDeliveries] resolution`: "Left-click
    /// SupplyDeliveryStation: sets ScheduledSDS to the clicked station"): the player's Supply Towers in operation
    /// among `towers` that have a chopper attached take it, the others ignore it ([`Reject::NoChopperAttached`] if
    /// none is left), on a station the player knows, as it knows it ([`Reject::NotAStation`]).
    pub(super) fn check_schedule(
        &self,
        player: PlayerId,
        towers: &[ObjectId],
        station: ObjectId,
    ) -> Result<Checked, Reject> {
        let attached = |o: &Object| is_tower(o) && attached_chopper(self.objects.values(), o.id).is_some();
        let towers = self.own_units(player, towers, attached)?;
        if towers.is_empty() {
            return Err(Reject::NoChopperAttached);
        }
        self.check_station(player, station)?;
        Ok(Checked::Schedule { towers, station })
    }

    /// A station target: one `player` knows ([`Sim::check_target_known`]) and takes to be a Supply Delivery Station,
    /// by its record if it only remembers it ([`Sim::known_kind`]), else [`Reject::NotAStation`].
    fn check_station(&self, player: PlayerId, station: ObjectId) -> Result<(), Reject> {
        self.check_target_known(player, station)?;
        if !is_station(self.known_kind(player, station)?) {
            return Err(Reject::NotAStation(station));
        }
        Ok(())
    }

    /// A tower target of `choppers`: a Supply Tower of `player`'s in operation, with no chopper attached but one of
    /// `choppers`.
    fn check_tower(&self, player: PlayerId, tower: ObjectId, choppers: &[ObjectId]) -> Result<(), Reject> {
        let target = self.referenced(tower, None)?;
        if target.owner != Some(player) || !supply_run::is_tower(target) {
            return Err(Reject::NotASupplyTower(tower));
        }
        match attached_chopper(self.objects.values(), tower) {
            Some(chopper) if !choppers.contains(&chopper) => Err(Reject::TowerTaken(tower)),
            _ => Ok(()),
        }
    }
}
