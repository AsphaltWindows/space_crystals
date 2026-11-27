//! The Tunnel Network in a game (`crate::network`, slice M4-C): the held units as the sim shows them, and the two ways
//! a unit leaves the map and comes back ([`Sim::take_off_map`], [`Sim::put_on_map`], plan decision T30). Its parts
//! (plan decision T28):
//! - `network/entries.rs`: the gate's rule for Enter, and the step right after phase 5a that takes units in;
//! - `network/ejects.rs`: the gate's rules for a Tunnel's commands, the eject queues, and the network's step of the
//!   economy phase (regeneration, then the eject queues);
//! - `network/produced.rs`: a producer's release into the network.
//!
//! **Old ids come back** (plan decision T35): a unit that comes out is the object it was. It is put on the map in the
//! economy phase, after 5d, so it never moves in the tick it appears, and before 5f, which sees it; the steps after it
//! treat every object alike, whatever its id.

mod ejects;
mod entries;
mod produced;

use std::collections::BTreeMap;

use glam::Vec2;

use super::{Reject, Sim};
use crate::combat::{self, AttackState};
use crate::defs::ObjectKind;
use crate::events::SimEvent;
use crate::ids::{ObjectId, PlayerId};
use crate::math;
use crate::mover::Mover;
use crate::network::{Held, Place, Service};
use crate::orders::Orders;
use crate::tunnel;

impl Sim {
    /// Every unit held off the map (plan decision T30, `crate::network`), in id order: those inside the Tunnel
    /// Networks, the Agents building sites, and (slice M5-F1) those inside the buildings that hold units
    /// (`network::Place`). None of them is among [`Sim::objects`].
    pub fn held(&self) -> impl Iterator<Item = &Held> + '_ {
        self.held.values()
    }

    /// The held unit `id`, if it is off the map.
    pub fn held_unit(&self, id: ObjectId) -> Option<&Held> {
        self.held.get(&id)
    }

    /// `player`'s Tunnel Network (`factions/syndicate/tunnels.md > Tunnel Network`): the units inside it, in id
    /// order, those queued to come out included ([`Sim::reserved`]).
    pub fn network(&self, player: PlayerId) -> impl Iterator<Item = &Held> + '_ {
        self.held.values().filter(move |held| match held.place {
            Place::Network => held.object.owner == Some(player),
            Place::Site(_) | Place::Building(_) => false,
        })
    }

    /// The units reserved on a Tunnel's eject queue (`factions/syndicate/tunnel_menus.md > EjectMenu`: "each click
    /// reserves a specific unit (the tile's count drops at the click)"; design change 0253: auto-ejected units "don't
    /// appear in the EjectMenu" while they wait), sorted by id.
    pub fn reserved(&self) -> Vec<ObjectId> {
        let queues = self.objects.values().filter_map(|o| o.tunnel.as_ref());
        let mut reserved: Vec<ObjectId> = queues.flat_map(|t| t.eject.iter().map(|entry| entry.unit)).collect();
        reserved.sort_unstable();
        reserved
    }

    /// What the EjectMenu shows for `player` (`factions/syndicate/tunnel_menus.md > EjectMenu`: "a grid of unit type
    /// tiles representing all units currently in the Tunnel Network. Each tile shows the unit type icon and a count"):
    /// how many units of each kind are in the network and not reserved on an eject queue, by kind in `ObjectKind`'s
    /// order.
    pub fn network_counts(&self, player: PlayerId) -> BTreeMap<ObjectKind, u32> {
        let reserved = self.reserved();
        let mut counts = BTreeMap::new();
        for held in self.network(player) {
            if reserved.binary_search(&held.object.id).is_err() {
                *counts.entry(held.object.kind).or_insert(0) += 1;
            }
        }
        counts
    }

    /// The unit an Eject of `kind` would bring out for `player` now (`factions/syndicate/tunnel_menus.md > EjectMenu`,
    /// design change 0251: "of the chosen type, the one that has been inside the network longest"), among those free
    /// to leave: in the network, not reserved, and past the transit delay (design change 0245; a unit produced inside
    /// has none, 0246). Ties by id. Else why not: [`Reject::NotInNetwork`] with no free unit of the kind at all,
    /// [`Reject::InTransit`] while all of them only just walked in.
    pub fn next_out(&self, player: PlayerId, kind: ObjectKind) -> Result<ObjectId, Reject> {
        let reserved = self.reserved();
        let free = || {
            self.network(player)
                .filter(|held| held.object.kind == kind && reserved.binary_search(&held.object.id).is_err())
        };
        if free().next().is_none() {
            return Err(Reject::NotInNetwork(kind));
        }
        free()
            .filter(|held| held.may_exit(self.tick))
            .min_by_key(|held| (held.since, held.object.id))
            .map(|held| held.object.id)
            .ok_or(Reject::InTransit(kind))
    }

    /// The parent Tunnel of `expansion` (plan decision T32, [`tunnel::area::parent`]): its owner's Tunnel in operation
    /// whose Tunnel Area fully contains it.
    pub fn parent_tunnel(&self, expansion: ObjectId) -> Option<ObjectId> {
        let expansion = self.objects.get(&expansion)?;
        tunnel::area::parent(expansion, self.objects.values().filter(|o| o.tunnel.is_some()))
    }

    /// Plan decision T30: unit `id` leaves the map for `place`, and stays the object it is, with its id. Every
    /// reference to it on the map ends as for a removal, without the effects of a death (`Sim::forget_references`:
    /// "orders, attacks and follows on it end, and rallies on it reset"; no `Died`, no `Removed`, no refund). Its own
    /// orders, "queued ones included", its move and its attack are cleared (design change 0249: "it is idle
    /// inside"); what it carries stays (an Agent "keeps its load"), and so do its HP and its modes. `entered` says
    /// whether it walked in, for the transit delay (`Held::entered`). Returns whether there was such an object.
    ///
    /// Call it before 5b (a step right after 5a), as a change of kind: the grids hold no unit, but the movement,
    /// the combat and the vision of the tick must not see a unit that has gone.
    pub(super) fn take_off_map(&mut self, id: ObjectId, place: Place, entered: bool) -> bool {
        // Its orders go, and a Build among them, gone before its site started, is refunded (`sim/builds.rs`).
        self.drop_orders(id);
        let Some(mut object) = self.objects.remove(&id) else {
            return false;
        };
        if let Some(orders) = &mut object.orders {
            *orders = Orders::default();
        }
        if let Some(mover) = &mut object.mover {
            // Its modes' slowdown is its own (`Mover::speed_factor`), not its move's.
            *mover = Mover {
                speed_factor: mover.speed_factor,
                ..Mover::default()
            };
        }
        if let Some(attack) = &mut object.attack {
            *attack = AttackState::default();
        }
        object.prev_pos = object.pos;
        object.prev_facing = object.facing;
        let since = self.tick;
        self.held.insert(
            id,
            Held {
                object,
                place,
                since,
                entered,
                leaving: None,
                service: Service::default(),
            },
        );
        self.forget_references(&[id]);
        true
    }

    /// Plan decision T30: held unit `id` comes back onto the map at `pos`, facing along `facing`, the object it was
    /// (its id, HP, load and modes), without an order. The caller found the spot clear (`Sim::check_unit_fits`; out
    /// of an exit doorway, `Sim::doorway_spot`, slice M5-F1). Returns whether there was such a held unit. (M5-N2) It
    /// comes out with no attack under way: one that fired from inside (`combat::occupants`, plan decision T72: "A held
    /// object keeps it, and leaving clears it") drops its target and its sequence, as every unit held had at
    /// [`Sim::take_off_map`], and a unit without a weapon of its own (a Disciple, which fired its stand-in's) the
    /// sequence it had inside.
    pub(super) fn put_on_map(&mut self, id: ObjectId, pos: Vec2, facing: Vec2) -> bool {
        let Some(Held { mut object, .. }) = self.held.remove(&id) else {
            return false;
        };
        if object.attack.is_some() {
            object.attack = combat::weapon(&object, self.research()).map(|_| AttackState::default());
        }
        object.pos = pos;
        object.prev_pos = pos;
        object.facing = math::vector_angle(facing);
        object.prev_facing = object.facing;
        self.objects.insert(id, object);
        true
    }

    /// The held units `ids` leave the game (`factions/syndicate/tunnels.md > Losing Tunnels > Stranded units`: "any
    /// unit inside the Tunnel Network that can no longer exit through any remaining Tunnel [...] is destroyed"; slice
    /// M4-H calls it), each with a `SimEvent::Removed`, in id order; an eject queue that reserved one lets it go.
    pub(super) fn remove_held(&mut self, ids: &[ObjectId]) {
        let mut gone: Vec<ObjectId> = ids.to_vec();
        gone.sort_unstable();
        gone.dedup();
        gone.retain(|id| self.held.remove(id).is_some());
        if gone.is_empty() {
            return;
        }
        self.events.extend(gone.iter().map(|&id| SimEvent::Removed(id)));
        self.forget_references(&gone);
    }

    /// Defeat-time cleanup (plan decision T30), right after the defeat check: the units held for a defeated player
    /// leave the game ([`Sim::remove_held`]). A defeated player owns no structure (`core/game_rules.md > Defeat
    /// Condition`), so no Tunnel, and nothing inside their network could ever come out
    /// (`factions/syndicate/tunnels.md > Losing Tunnels > Stranded units`: with no Tunnel left, "every unit inside the
    /// network is destroyed"; the rule for a player who lives on is slice M4-H's). Their units on the map become
    /// neutral instead (`core/game_rules.md > On Defeat`, `economy::defeat`).
    pub(super) fn clear_defeated_networks(&mut self) {
        if self.held.is_empty() {
            return;
        }
        let defeated = |owner: Option<PlayerId>| {
            owner
                .and_then(|owner| self.player(owner))
                .is_some_and(|player| player.is_defeated())
        };
        let lost: Vec<ObjectId> = (self.held.values())
            .filter(|held| defeated(held.object.owner))
            .map(|held| held.object.id)
            .collect();
        self.remove_held(&lost);
    }
}

#[cfg(test)]
mod tests;
