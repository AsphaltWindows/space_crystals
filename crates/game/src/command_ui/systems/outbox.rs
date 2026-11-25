//! Where the command flow's orders and commands go (`super`): the sim's next tick (`PendingCommands`), each one only
//! if the sim would take it now (`Sim::validate`), and the indicators (`OrderIssued`).

use std::collections::BTreeSet;

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use sc_sim::{
    Ability, Command, ConstructionCommand, ObjectId, OrderKind, PlayerCommand, PlayerId, ProductionCommand, Reject,
    Target, TunnelCommand,
};

use crate::command_ui::OrderIssued;
use crate::command_ui::interface::deploying;
use crate::command_ui::resolution::group_orders;
use crate::sim_driver::{PendingCommands, SimRes};

/// Where orders go: the sim's next tick, and the indicators.
#[derive(SystemParam)]
pub(in crate::command_ui) struct OrderOutbox<'w> {
    sim: Res<'w, SimRes>,
    pending: ResMut<'w, PendingCommands>,
    issued: MessageWriter<'w, OrderIssued>,
}

impl OrderOutbox<'_> {
    /// Gives `kind` to `player`'s units among `units` (`systems/controls/selection_and_hud.md > CommandPanel`:
    /// "Commands can only be issued to objects owned by the player"; only units take orders). Nothing when none of them
    /// is left, or when the sim would reject the order (`Sim::validate`: e.g. a point off the map, or an indestructible
    /// target to attack). An Attack that the gate refuses only because none of the units takes one yet goes all the
    /// same when a Lower Shield sent for the same tick makes one of them a Peacekeeper first
    /// ([`Self::attack_once_toggled`]).
    pub(super) fn give(&mut self, player: PlayerId, units: Vec<ObjectId>, kind: OrderKind, queue: bool) -> bool {
        let units = self.own_units(player, units);
        // `ARCHITECTURE.md > Commands, events and replays`: never a non-finite number, which a replay can't store.
        if units.is_empty() || !finite(kind) {
            return false;
        }
        let command = PlayerCommand {
            player,
            command: Command::Order {
                units: units.clone(),
                kind,
                queue,
            },
        };
        if let Err(reject) = self.sim.validate(&command)
            && !(reject == Reject::CannotAttack && self.attack_once_toggled(player, &units))
        {
            debug!("order {kind:?} not given: {reject}");
            return false;
        }
        self.pending.0.push(command);
        self.issued.write(OrderIssued {
            kind,
            units,
            queue,
            tick: self.sim.tick(),
        });
        true
    }

    /// Gives each of `orders` (a unit and its order, in the units' order) as few orders as possible: one per
    /// distinct order ([`group_orders`]), so the units sent to one location go as one group. Whether anything was
    /// given.
    pub(super) fn give_each(&mut self, player: PlayerId, orders: Vec<(ObjectId, OrderKind)>, queue: bool) -> bool {
        let mut given = false;
        for (kind, units) in group_orders(orders) {
            given |= self.give(player, units, kind, queue);
        }
        given
    }

    /// `player`'s `units` use `ability` (`Command::Ability`), at once: an ability isn't queued. Nothing when none of
    /// them is the player's, or when the sim would reject it (`Sim::validate`: e.g. none of them has the ability).
    pub(super) fn use_ability(&mut self, player: PlayerId, units: Vec<ObjectId>, ability: Ability) -> bool {
        let units = self.own_units(player, units);
        if units.is_empty() {
            return false;
        }
        let command = PlayerCommand {
            player,
            command: Command::Ability { units, ability },
        };
        if let Err(reject) = self.sim.validate(&command) {
            debug!("ability {ability:?} not used: {reject}");
            return false;
        }
        self.pending.0.push(command);
        true
    }

    /// `player`'s producers among `objects` carry out `command` (`Command::Production`), one command for all of
    /// them, at once: production isn't queued. Nothing when none of them is the player's producer, or when the sim
    /// would reject it (`Sim::validate`, the predicates the panel greys its buttons out by: e.g. a full queue, too
    /// few Space Crystals, nothing to cancel, a rally point off the map, which a non-finite point is too).
    pub(super) fn produce(&mut self, player: PlayerId, objects: Vec<ObjectId>, command: ProductionCommand) -> bool {
        let objects: Vec<ObjectId> = objects
            .into_iter()
            .filter(|&id| {
                self.sim
                    .object(id)
                    .is_some_and(|o| o.owner == Some(player) && o.production.is_some())
            })
            .collect();
        if objects.is_empty() {
            return false;
        }
        let sent = PlayerCommand {
            player,
            command: Command::Production { objects, command },
        };
        if let Err(reject) = self.sim.validate(&sent) {
            debug!("production command {command:?} not given: {reject}");
            return false;
        }
        self.pending.0.push(sent);
        true
    }

    /// `player`'s objects among `objects` carry out `command` (`Command::Construction`), one command for all of
    /// them, at once: construction isn't queued. Nothing when none of them is the player's, or when the sim would
    /// reject it (`Sim::validate`, the predicates the panel greys its buttons out by: e.g. none of them a constructor
    /// or a Beacon, too few Space Crystals, a constructor already busy, nothing to cancel; a Place or a Warp In the
    /// placement check refuses).
    pub(super) fn construct(&mut self, player: PlayerId, objects: Vec<ObjectId>, command: ConstructionCommand) -> bool {
        self.try_construct(player, objects, command).is_ok()
    }

    /// The same, saying why nothing was sent: the gate's refusal (`Reject::NoConstructor` when none of `objects` is
    /// the player's).
    pub(super) fn try_construct(
        &mut self,
        player: PlayerId,
        objects: Vec<ObjectId>,
        command: ConstructionCommand,
    ) -> Result<(), Reject> {
        let objects: Vec<ObjectId> = objects
            .into_iter()
            .filter(|&id| self.sim.object(id).is_some_and(|o| o.owner == Some(player)))
            .collect();
        if objects.is_empty() {
            return Err(Reject::NoConstructor);
        }
        let sent = PlayerCommand {
            player,
            command: Command::Construction { objects, command },
        };
        if let Err(reject) = self.sim.validate(&sent) {
            debug!("construction command {command:?} not given: {reject}");
            return Err(reject);
        }
        self.pending.0.push(sent);
        Ok(())
    }

    /// `player`'s Tunnel among `tunnels` carries out `command` (`Command::Tunnel`), at once: the first of them that is
    /// the player's, the one Tunnel of the ActiveGroup (Tunnels are Ungroupable). Nothing when none is, or when the
    /// sim would reject it (`Sim::validate`, the predicates the panel greys its buttons out by: no unit of the type
    /// free to leave, the Tunnel's tier).
    pub(super) fn tunnel(&mut self, player: PlayerId, tunnels: Vec<ObjectId>, command: TunnelCommand) -> bool {
        let own = |id: &ObjectId| self.sim.object(*id).is_some_and(|o| o.owner == Some(player));
        let Some(tunnel) = tunnels.into_iter().find(own) else {
            return false;
        };
        let sent = PlayerCommand {
            player,
            command: Command::Tunnel { tunnel, command },
        };
        if let Err(reject) = self.sim.validate(&sent) {
            debug!("tunnel command {command:?} not given: {reject}");
            return false;
        }
        self.pending.0.push(sent);
        true
    }

    /// `player`'s Drones that have a Deploy, in the sim or sent for its next tick ([`deploying`]).
    pub(super) fn deploying(&self, player: PlayerId) -> BTreeSet<ObjectId> {
        deploying(&self.sim, player, &self.pending.0)
    }

    /// Whether one of `units` takes an Attack once the mode toggles already sent for the next tick are applied, which
    /// the sim does before the orders sent after them (design change 0202: "which orders a unit accepts follows its
    /// new type from the press"; 0198: the press switches it): an Attack, by the panel or a right-click, right after
    /// a Lower Shield the sim hasn't applied yet, which the selection already shows as Peacekeepers
    /// (`PendingCommands::kind_of`). The gate takes it in that tick, and the units attack once their shields are
    /// down.
    fn attack_once_toggled(&self, player: PlayerId, units: &[ObjectId]) -> bool {
        units.iter().any(|&id| {
            (self.pending.kind_of(&self.sim, player, id)).is_some_and(sc_sim::riot_shield::takes_attack_orders)
        })
    }

    /// The units among `units` that are `player`'s and take orders.
    fn own_units(&self, player: PlayerId, units: Vec<ObjectId>) -> Vec<ObjectId> {
        units
            .into_iter()
            .filter(|&id| {
                self.sim
                    .object(id)
                    .is_some_and(|o| o.owner == Some(player) && o.orders.is_some())
            })
            .collect()
    }
}

/// Whether every number of the order is finite.
fn finite(kind: OrderKind) -> bool {
    match kind {
        OrderKind::Move(Target::Point(p))
        | OrderKind::AttackMove(p)
        | OrderKind::Patrol(p)
        | OrderKind::AttackGround(p) => p.is_finite(),
        OrderKind::Move(Target::Object(_)) | OrderKind::Attack(_) | OrderKind::Hold | OrderKind::Stop => true,
        OrderKind::Gather(_) | OrderKind::DropOff(_) | OrderKind::Deploy { .. } | OrderKind::Enter(_) => true,
    }
}
