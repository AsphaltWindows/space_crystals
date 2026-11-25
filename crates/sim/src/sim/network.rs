//! The Tunnel Network in a game (`crate::network`, slice M4-C): the held units as the sim shows them, the two ways a
//! unit leaves the map and comes back ([`Sim::take_off_map`], [`Sim::put_on_map`], plan decision T30), the gate's
//! rules for Enter and Eject, the step right after phase 5a that takes units in, a producer's release into the
//! network, and the network's step of the economy phase (regeneration, then the eject queues).
//!
//! - **Entering** ([`Sim::step_entries`], `factions/syndicate/tunnels.md > Tunnel Network > Walking in`): right after
//!   5a, in id order, each unit whose Enter order's walk brought it into the entry path leaves the map. "There is no
//!   fixed hold time per unit": a unit goes in in the tick it gets there, however many went before it, so "an army
//!   files in about as fast as it can walk in single file". An order whose Tunnel is gone or no longer suits the unit
//!   ends, and so does one whose walk ended short of the path ("the normal Failure rule applies", design change
//!   0247), with a `SimEvent::EnterFailed`.
//! - **Ejecting** ([`Sim::step_network`], `> EjectMenu`, design changes 0250, 0251 and 0253): each Tunnel's queue
//!   lets its first unit out when the exit path's spot is clear: "Each unit appears at the exit path's spot just
//!   outside Side A. If that spot is blocked (by any unit or structure), the next unit waits until it clears; no
//!   other spot is tried." No minimum gap is left between two units (0250 "replaces the earlier 'every 8 frames
//!   minimum'"): "the exit rate is set by unit speed and size", by the unit before walking off the spot.
//! - **Old ids come back** (plan decision T35): a unit that comes out is the object it was. It is put on the map in
//!   the economy phase, after 5d, so it never moves in the tick it appears, and before 5f, which sees it; the steps
//!   after it treat every object alike, whatever its id.

use std::collections::BTreeMap;

use glam::Vec2;

use super::gate::Checked;
use super::{Reject, Sim};
use crate::combat::AttackState;
use crate::command::TunnelCommand;
use crate::defs::ObjectKind;
use crate::events::SimEvent;
use crate::ids::{ObjectId, PlayerId};
use crate::math;
use crate::mover::Mover;
use crate::network::{self, Eject, Held, Paths, Place};
use crate::object::Object;
use crate::orders::state::Behaviour;
use crate::orders::{OrderKind, Orders, Target};
use crate::production::Rally;

impl Sim {
    /// Every unit held off the map (plan decision T30, `crate::network`), in id order: those inside the Tunnel
    /// Networks. None of them is among [`Sim::objects`].
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
        })
    }

    /// The units reserved on a Tunnel's eject queue (`> EjectMenu`: "each click reserves a specific unit (the tile's
    /// count drops at the click)"; design change 0253: auto-ejected units "don't appear in the EjectMenu" while they
    /// wait), sorted by id.
    pub fn reserved(&self) -> Vec<ObjectId> {
        let queues = self.objects.values().filter_map(|o| o.tunnel.as_ref());
        let mut reserved: Vec<ObjectId> = queues.flat_map(|t| t.eject.iter().map(|entry| entry.unit)).collect();
        reserved.sort_unstable();
        reserved
    }

    /// What the EjectMenu shows for `player` (`> EjectMenu`: "a grid of unit type tiles representing all units
    /// currently in the Tunnel Network. Each tile shows the unit type icon and a count"): how many units of each
    /// kind are in the network and not reserved on an eject queue, by kind in `ObjectKind`'s order.
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

    /// The unit an Eject of `kind` would bring out for `player` now (`> EjectMenu`, design change 0251: "of the
    /// chosen type, the one that has been inside the network longest"), among those free to leave: in the network,
    /// not reserved, and past the transit delay (design change 0245; a unit produced inside has none, 0246). Ties
    /// by id. Else why not: [`Reject::NotInNetwork`] with no free unit of the kind at all, [`Reject::InTransit`]
    /// while all of them only just walked in.
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

    /// The parent Tunnel of `expansion` (plan decision T32, [`network::parent`]): its owner's Tunnel in operation
    /// whose Tunnel Area fully contains it.
    pub fn parent_tunnel(&self, expansion: ObjectId) -> Option<ObjectId> {
        let expansion = self.objects.get(&expansion)?;
        network::parent(expansion, self.objects.values().filter(|o| o.tunnel.is_some()))
    }

    /// The gate for `OrderKind::Enter(tunnel)` (`systems/controls/commands.md > Enter`: "any Syndicate unit, when the
    /// target Tunnel's tier is sufficient for the unit's base category"), given `units`, the player's units that take
    /// orders. The Tunnel is the player's ([`Reject::NotOwned`]) and in operation ([`Reject::NotATunnel`]). The units
    /// its tier lets in take the order ([`network::admits`]), the others ignore it; none is [`Reject::NoTransit`].
    /// Any unit of the Tunnel's owner may enter, whatever faction its type belongs to: only a debug game has others.
    pub(super) fn check_enter(
        &self,
        player: PlayerId,
        units: Vec<ObjectId>,
        tunnel: ObjectId,
    ) -> Result<Vec<ObjectId>, Reject> {
        let target = self.referenced(tunnel, Some(player))?;
        if !network::is_exit(target) {
            return Err(Reject::NotATunnel(tunnel));
        }
        let lets_in = |id: &ObjectId| {
            (self.objects.get(id)).is_some_and(|o| network::admits(target, o.kind).is_ok())
        };
        let admitted: Vec<ObjectId> = units.into_iter().filter(lets_in).collect();
        if admitted.is_empty() {
            return Err(Reject::NoTransit { tunnel, kind: None });
        }
        Ok(admitted)
    }

    /// The gate for `Command::Tunnel` (plan decision T36): the Tunnel is the player's ([`Reject::NotOwned`]) and in
    /// operation ([`Reject::NotATunnel`]). An **Eject** (`factions/syndicate/tunnels.md > EjectMenu`): its tier lets
    /// units of the kind out ("Unit types whose base category exceeds this Tunnel's tier are [...] disabled":
    /// [`Reject::NoTransit`]), and a unit of the kind is free to leave ([`Sim::next_out`]), which the command
    /// reserves.
    pub(super) fn check_tunnel(
        &self,
        player: PlayerId,
        tunnel: ObjectId,
        command: TunnelCommand,
    ) -> Result<Checked, Reject> {
        let target = self.referenced(tunnel, Some(player))?;
        if !network::is_exit(target) {
            return Err(Reject::NotATunnel(tunnel));
        }
        match command {
            TunnelCommand::Eject { kind } => {
                if network::admits(target, kind).is_err() {
                    let kind = Some(kind);
                    return Err(Reject::NoTransit { tunnel, kind });
                }
                let unit = self.next_out(player, kind)?;
                Ok(Checked::Eject { tunnel, unit })
            }
        }
    }

    /// Held unit `unit` joins the end of `tunnel`'s eject queue, to come out with `order` (`> EjectMenu`: "Ejected
    /// units are queued"; design change 0253: auto-ejected units "join the parent Tunnel's exit queue in order with
    /// manual ejects, first come, first served").
    pub(super) fn queue_eject(&mut self, tunnel: ObjectId, unit: ObjectId, order: Option<OrderKind>) {
        if !self.held.contains_key(&unit) {
            return;
        }
        if let Some(state) = self.objects.get_mut(&tunnel).and_then(|o| o.tunnel.as_mut()) {
            state.eject.push(Eject { unit, order });
        }
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
            },
        );
        self.forget_references(&[id]);
        true
    }

    /// Plan decision T30: held unit `id` comes back onto the map at `pos`, facing along `facing`, the object it was
    /// (its id, HP, load and modes), without an order. The caller found the spot clear (`Sim::check_unit_fits`).
    /// Returns whether there was such a held unit.
    pub(super) fn put_on_map(&mut self, id: ObjectId, pos: Vec2, facing: Vec2) -> bool {
        let Some(Held { mut object, .. }) = self.held.remove(&id) else {
            return false;
        };
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
    /// neutral instead (`> On Defeat`, `economy::defeat`).
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

    /// The entries' step, right after phase 5a and before 5b (module docs; `ARCHITECTURE.md > Tick pipeline`): each
    /// unit from before this tick (`first_new`) whose current order is an Enter, in id order, goes into the network
    /// if it has walked into its Tunnel's entry path, and loses the order if it can't go on.
    pub(super) fn step_entries(&mut self, first_new: ObjectId) {
        let entering: Vec<(ObjectId, ObjectId, bool)> = self
            .objects
            .range(..first_new)
            .filter_map(|(&id, o)| {
                let active = o.orders.as_ref()?.current.as_ref()?;
                match (active.order.kind, &active.state) {
                    (OrderKind::Enter(tunnel), Behaviour::Enter(state)) => Some((id, tunnel, state.walked)),
                    _ => None,
                }
            })
            .collect();
        for (unit, tunnel, walked) in entering {
            self.arrive_at_entry(unit, tunnel, walked);
        }
    }

    /// Unit `unit` is on an Enter order to `tunnel`, its walk over or not (`walked`).
    /// - **The Tunnel suits it** (it stands, is its owner's and its tier lets the unit in, [`network::admits`]) **and
    ///   the unit has walked into the entry path** ([`Paths::reached_entry`]): it goes into the network
    ///   (`SimEvent::EnteredNetwork`), on its way or at the end of its walk alike, so a line never pauses.
    /// - **The Tunnel doesn't suit it** (`> Walking in > Target Tunnel gone or unsuitable`), **or its walk ended short
    ///   of the path** (the normal Failure rule, design change 0247; the occupied-target rule's arrival behind
    ///   standing units is such an end too): "the Enter order ends and the unit idles" (`SimEvent::EnterFailed`). Its
    ///   queued orders go on from the next tick, as after any order that ends.
    /// - Otherwise it walks on.
    fn arrive_at_entry(&mut self, unit: ObjectId, tunnel: ObjectId, walked: bool) {
        let Some(o) = self.objects.get(&unit) else {
            return;
        };
        let target = self.objects.get(&tunnel).filter(|target| {
            target.owner.is_some() && target.owner == o.owner && network::admits(target, o.kind).is_ok()
        });
        let reached = (target.and_then(Paths::of_object).zip(o.def().half_extents()))
            .is_some_and(|(paths, half)| paths.reached_entry(o.pos, half));
        if reached {
            if self.take_off_map(unit, Place::Network, true) {
                self.events.push(SimEvent::EnteredNetwork { unit, tunnel });
            }
            return;
        }
        if target.is_some() && !walked {
            return;
        }
        let Some(o) = self.objects.get_mut(&unit) else {
            return;
        };
        if let Some(orders) = &mut o.orders {
            orders.current = None;
        }
        if let Some(mover) = &mut o.mover {
            mover.stop();
        }
        self.events.push(SimEvent::EnterFailed { unit, tunnel });
    }

    /// `factions/syndicate/structures.md > Rally Point Behavior` (plan decision T35): `producer`, an underground
    /// expansion (`ExitRule::Network`), finished its unit. The unit comes into the game inside its owner's Tunnel
    /// Network, with no transit delay (design change 0246: "they can exit as soon as they are produced"), and:
    /// - "**No rally point, or rally point set on the parent Tunnel**: Unit stays in the Tunnel Network"; "**Rally
    ///   point on any own Tunnel**: the unit stays inside the network" (design change 0252);
    /// - "**Rally point set on the surface**: Unit auto-ejects from the parent Tunnel (Side A) and moves to the rally
    ///   point", which is "the unit's default right-click action at the rally target" (design change 0136,
    ///   [`Sim::rally_order_of`]); "**Rally point on an expansion**: treated as the surface location above it"
    ///   (0252: a Move to its centre, not an object target). It joins "the parent Tunnel's exit queue in order with
    ///   manual ejects" (0253).
    ///
    /// A unit whose producer has no parent Tunnel (plan decision T32: derived, [`Sim::parent_tunnel`]), or whose
    /// parent's tier doesn't let it out, stays in the network. Nothing holds a release up: the network has no
    /// doorway to be occupied.
    pub(super) fn release_into_network(&mut self, producer: ObjectId) {
        let Some(building) = self.objects.get(&producer) else {
            return;
        };
        let (Some(owner), Some(production)) = (building.owner, &building.production) else {
            return;
        };
        let (Some(kind), rally, pos) = (production.current(), production.rally, building.pos) else {
            return;
        };
        if kind.def().unit.is_none() || self.next_id == u32::MAX {
            return;
        }
        if let Some(production) = self.objects.get_mut(&producer).and_then(|o| o.production.as_mut()) {
            production.release();
        }
        let id = ObjectId(self.next_id);
        self.next_id += 1;
        let object = self.new_object(id, kind, Some(owner), pos, None, None);
        let order = rally.and_then(|rally| self.surface_rally_order(&object, rally));
        let since = self.tick;
        self.held.insert(
            id,
            Held {
                object,
                place: Place::Network,
                since,
                entered: false,
            },
        );
        self.events.push(SimEvent::UnitProduced { producer, unit: id });
        let Some(order) = order else {
            return;
        };
        let parent = (self.parent_tunnel(producer))
            .filter(|parent| (self.objects.get(parent)).is_some_and(|t| network::admits(t, kind).is_ok()));
        if let Some(parent) = parent {
            self.queue_eject(parent, id, Some(order));
        }
    }

    /// The order a unit `unit` produced into the network leaves it with for `rally`, its producer's rally point, or
    /// `None` if the rally keeps it inside (`> Rally Point Behavior`, [`Sim::release_into_network`]): a rally on a
    /// Tunnel of its owner's keeps it inside; one on an underground structure of its owner's is the surface spot
    /// above it; any other gives the unit's default command on the target.
    fn surface_rally_order(&self, unit: &Object, rally: Rally) -> Option<OrderKind> {
        if let Rally::Object { id, .. } = rally
            && let Some(target) = self.objects.get(&id)
            && target.owner == unit.owner
        {
            if target.tunnel.is_some() {
                return None;
            }
            if target.def().structure.as_ref().is_some_and(|s| s.underground) {
                return Some(OrderKind::Move(Target::Point(target.pos)));
            }
        }
        self.rally_order_of(unit, rally)
    }

    /// The network's step of the economy phase, after the producers (module docs): every unit inside the networks
    /// regenerates (`> Tunnel Network > Regeneration inside`, design change 0249), then each Tunnel with an eject
    /// queue, in id order, lets its first unit out if it can ([`Sim::eject_next`]).
    pub(super) fn step_network(&mut self) {
        if self.held.is_empty() {
            return;
        }
        for held in self.held.values_mut() {
            held.regenerate();
        }
        let queued: Vec<ObjectId> = self
            .objects
            .values()
            .filter(|o| o.tunnel.as_ref().is_some_and(|t| !t.eject.is_empty()))
            .map(|o| o.id)
            .collect();
        for tunnel in queued {
            self.eject_next(tunnel);
        }
    }

    /// `factions/syndicate/tunnels.md > EjectMenu` (design change 0250): the first unit of `tunnel`'s eject queue
    /// comes out, if the exit path's spot is clear for it: its box there overlaps no blocked tile, no blocking
    /// footprint and no unit of its layer (`Sim::check_unit_fits`, the rule of every unit put on the map). Else the
    /// queue waits: "no other spot is tried". The unit appears facing out of Side A (`SimEvent::LeftNetwork`) and
    /// gets its order: its rally's (design change 0253), or, "with no rally point, a unit walks about 1 grid unit
    /// straight out of Side A and idles (a blocked walk follows the normal Failure rule)", a Move there. One unit a
    /// tick at most: the next would stand on it.
    ///
    /// An entry the Tunnel can't serve leaves the queue, and its unit stays in the network, free again: a unit that
    /// is no longer held, and one the tier no longer lets out (a tier a debug command lowered).
    fn eject_next(&mut self, tunnel: ObjectId) {
        let Some(target) = self.objects.get(&tunnel) else {
            return;
        };
        let (Some(paths), Some(&entry)) = (
            Paths::of_object(target),
            target.tunnel.as_ref().and_then(|t| t.eject.first()),
        ) else {
            return;
        };
        let unit = entry.unit;
        let kind = self.held.get(&unit).map(|held| held.object.kind);
        let served = kind.filter(|&kind| network::admits(target, kind).is_ok());
        let stand = served
            .and_then(|kind| kind.def().half_extents())
            .map(|half| paths.exit_stand(half));
        let (Some(kind), Some(stand)) = (served, stand) else {
            self.pop_eject(tunnel);
            return;
        };
        if !self.map.contains_point(stand) || self.check_unit_fits(kind, stand, None).is_err() {
            return;
        }
        self.pop_eject(tunnel);
        if !self.put_on_map(unit, stand, paths.outward) {
            return;
        }
        self.events.push(SimEvent::LeftNetwork { unit, tunnel });
        let walk_out = paths.walk_out(stand);
        let order = entry
            .order
            .or_else(|| (self.map.contains_point(walk_out)).then_some(OrderKind::Move(Target::Point(walk_out))));
        if let Some(order) = order {
            self.give_order(&[unit], order, false);
        }
    }

    /// The first entry of `tunnel`'s eject queue leaves it.
    fn pop_eject(&mut self, tunnel: ObjectId) {
        if let Some(state) = self.objects.get_mut(&tunnel).and_then(|o| o.tunnel.as_mut())
            && !state.eject.is_empty()
        {
            state.eject.remove(0);
        }
    }
}

#[cfg(test)]
mod tests;
