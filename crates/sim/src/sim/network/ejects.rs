//! Leaving the Tunnel Network (`crate::network`, slice M4-C): the gate's rules for a Tunnel's commands, its eject
//! queue, and the network's step of the economy phase (regeneration, then the eject queues).
//!
//! **Ejecting** ([`Sim::step_network`], `factions/syndicate/tunnel_menus.md > EjectMenu`, design changes 0250, 0251 and
//! 0253): each Tunnel's queue lets its first unit out when the exit path's spot is clear: "Each unit appears at the
//! exit path's spot just outside Side A. If that spot is blocked (by any unit or structure), the next unit waits until
//! it clears; no other spot is tried." No minimum gap is left between two units (0250 "replaces the earlier 'every 8
//! frames minimum'"): "the exit rate is set by unit speed and size", by the unit before walking off the spot.
//!
//! (M6-SD) **Two lanes** (plan decision T96: "Eject entries for air kinds release from the air exit door on their own
//! lane, one per 8 frames, beside the ground lane. The queue keeps its order within each lane"): a queue's ground units
//! come out of Side A's exit path, one after another as above, and its air units out of the Tier 3 air exit door, one
//! every `Tunnel::air_door_interval_ticks` ("**Air units** leave through the Tier 3 **air exit door**, in parallel
//! with ground units"). Neither lane waits for the other.

use glam::Vec2;

use crate::augmentation::Research;
use crate::command::TunnelCommand;
use crate::defs::ObjectKind;
use crate::events::SimEvent;
use crate::ids::{ObjectId, PlayerId};
use crate::math;
use crate::network::{self, Eject, Paths};
use crate::orders::{OrderKind, Target};
use crate::progress::Progress;
use crate::shape::Shape;
use crate::sim::gate::Checked;
use crate::sim::{Reject, Sim};
use crate::tunnel::TunnelState;

impl Sim {
    /// The gate for `Command::Tunnel` (plan decision T36): the Tunnel is the player's ([`Reject::NotOwned`]) and in
    /// operation ([`Reject::NotATunnel`]). An **Eject** (`factions/syndicate/tunnel_menus.md > EjectMenu`): its tier
    /// lets units of the kind out ("Unit types whose base category exceeds this Tunnel's tier are [...] disabled":
    /// [`Reject::NoTransit`]), and a unit of the kind is free to leave ([`Sim::next_out`]), which the command
    /// reserves. An **Expand**, an **Upgrade** and a **Cancel** are the Tunnel's operation's (`sim/expansions.rs`,
    /// `sim/upgrades.rs`).
    pub(in crate::sim) fn check_tunnel(
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
            TunnelCommand::Expand {
                kind,
                origin,
                orientation,
            } => self.check_expand(player, target, kind, origin, orientation),
            TunnelCommand::Upgrade => self.check_upgrade(player, target),
            TunnelCommand::Cancel => self.check_cancel_operation(target),
        }
    }

    /// Held unit `unit` joins the end of `tunnel`'s eject queue, to come out with `order` (`> EjectMenu`: "Ejected
    /// units are queued"; design change 0253: auto-ejected units "join the parent Tunnel's exit queue in order with
    /// manual ejects, first come, first served").
    pub(in crate::sim) fn queue_eject(&mut self, tunnel: ObjectId, unit: ObjectId, order: Option<OrderKind>) {
        if !self.held.contains_key(&unit) {
            return;
        }
        if let Some(state) = self.objects.get_mut(&tunnel).and_then(|o| o.tunnel.as_mut()) {
            state.eject.push(Eject { unit, order });
        }
    }

    /// The network's step of the economy phase, after the producers (module docs): every unit inside the networks
    /// regenerates (`> Tunnel Network > Regeneration inside`, design change 0249), then each Tunnel with an eject
    /// queue, in id order, lets the first unit of each of its two lanes out if it can: its ground lane's by Side A's
    /// exit path ([`Sim::eject_next`]), (M6-SD) its air lane's by its air exit door ([`Sim::eject_air`]).
    pub(in crate::sim) fn step_network(&mut self) {
        if self.held.is_empty() {
            return;
        }
        for held in self.held.values_mut() {
            held.regenerate(Research::new(&self.players));
        }
        let queued: Vec<ObjectId> = self
            .objects
            .values()
            .filter(|o| o.tunnel.as_ref().is_some_and(|t| !t.eject.is_empty()))
            .map(|o| o.id)
            .collect();
        for tunnel in queued {
            self.eject_next(tunnel);
            self.eject_air(tunnel);
        }
    }

    /// (M6-SD) The first entry of `tunnel`'s eject queue in its air lane (`air`) or its ground lane, with its index in
    /// the queue (module docs): the first whose unit goes out by the air doors (`network::uses_air_doors`), or the
    /// first whose unit doesn't, an entry whose unit is no longer held among them.
    fn lane_front(&self, tunnel: ObjectId, air: bool) -> Option<(usize, Eject)> {
        let queue = &self.objects.get(&tunnel)?.tunnel.as_ref()?.eject;
        let flies =
            |entry: &Eject| (self.held.get(&entry.unit)).is_some_and(|h| network::uses_air_doors(h.object.kind));
        queue
            .iter()
            .enumerate()
            .find(|(_, entry)| flies(entry) == air)
            .map(|(i, &entry)| (i, entry))
    }

    /// `factions/syndicate/tunnel_menus.md > EjectMenu` (design change 0250): the first unit of `tunnel`'s ground lane
    /// ([`Sim::lane_front`]) comes out, if the exit path's spot is clear for it: its box there overlaps no blocked
    /// tile, no blocking footprint and no unit of its layer (`Sim::check_unit_fits`, the rule of every unit put on the
    /// map). Else the lane waits: "no other spot is tried". The unit appears facing out of Side A
    /// (`SimEvent::LeftNetwork`) and gets its order: its rally's (design change 0253), or, "with no rally point, a unit
    /// walks about 1 grid unit straight out of Side A and idles (a blocked walk follows the normal Failure rule)", a
    /// Move there. One unit a tick at most: the next would stand on it.
    ///
    /// (M6-SV) Its box is the one it has where it appears ([`exit_half`], plan decision T100: "Syndicate vehicles
    /// [...] out of a Tier 2+ Tunnel's Side A exit path"): a vehicle's Silhouette turned to face out of Side A, so that
    /// it stands against the wall whichever way the Tunnel is turned, a Bike 56 su deep out of it, and its spot is
    /// checked with it turned so.
    ///
    /// An entry the Tunnel can't serve leaves the queue, and its unit stays in the network, free again: a unit that
    /// is no longer held, and one the tier no longer lets out (a tier a debug command lowered).
    fn eject_next(&mut self, tunnel: ObjectId) {
        let Some((index, entry)) = self.lane_front(tunnel, false) else {
            return;
        };
        let Some(target) = self.objects.get(&tunnel) else {
            return;
        };
        let Some(paths) = Paths::of_object(target) else {
            return;
        };
        let unit = entry.unit;
        let kind = self.held.get(&unit).map(|held| held.object.kind);
        let served = kind.filter(|&kind| network::admits(target, kind).is_ok());
        let facing = math::vector_angle(paths.outward);
        let stand = served
            .and_then(|kind| exit_half(kind, paths.outward))
            .map(|half| paths.exit_stand(half));
        let (Some(kind), Some(stand)) = (served, stand) else {
            self.remove_eject(tunnel, index);
            return;
        };
        if !self.map.contains_point(stand) || self.check_unit_fits_facing(kind, stand, facing, None).is_err() {
            return;
        }
        self.remove_eject(tunnel, index);
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

    /// (M6-SD) The first unit of `tunnel`'s air lane ([`Sim::lane_front`]) comes out of its air exit door, if the door
    /// is free (`factions/syndicate/tunnel_menus.md > EjectMenu`: "**Air units** leave through the Tier 3 **air exit
    /// door**, in parallel with ground units; since aircraft don't collide, the door has its own **exit rate: one
    /// aircraft per 8 frames (0.5 s)**", [`TunnelState::door_free`]). It appears over the door, facing out of Side A as
    /// every unit out of a Tunnel does, and the door is taken for its interval (`TunnelState::air_out`); it rises out
    /// of it for its type's `UnitDef::air_door_ticks` (design change 0431: "it **rises** out of the exit door over **16
    /// frames (1 s)** [...], and it can be hit during both **as an air unit**"), holding still on the map meanwhile
    /// (`Mover::rise`), and gets its order at once, as a runway's aircraft does (`Mover::takeoff`): its rally's (design
    /// change 0253), which it starts on once risen. With none it stays over the door: **Placeholder (not in the design:
    /// an aircraft out of the air doors with no rally point hovers where it rose)**, as aircraft never block the way
    /// out.
    ///
    /// (M6-GK) A HoverCraft stays inside the map (`systems/units.md > Air Collision > Map edge`, `Map::clamp_box`): it
    /// appears at the point nearest the door's centre where its box, facing out, lies on the map
    /// (`network::door_spot`), the door's centre itself but where a door's tuned offset would put it past the edge, and
    /// waits while not even that spot takes it (`Sim::check_unit_fits`).
    ///
    /// An entry the Tunnel can't serve leaves the queue, as on the ground lane.
    fn eject_air(&mut self, tunnel: ObjectId) {
        let Some((index, entry)) = self.lane_front(tunnel, true) else {
            return;
        };
        let (Some(target), Some(held)) = (self.objects.get(&tunnel), self.held.get(&entry.unit)) else {
            return;
        };
        let (Some(paths), Some(state)) = (Paths::of_object(target), target.tunnel.as_ref()) else {
            return;
        };
        let (unit, kind) = (entry.unit, held.object.kind);
        if !TunnelState::door_free(state.air_out, self.tick) {
            return;
        }
        let Some(half) = exit_half(kind, paths.outward).filter(|_| network::admits(target, kind).is_ok()) else {
            self.remove_eject(tunnel, index);
            return;
        };
        let spot = network::door_spot(&self.map, paths.air_exit, half);
        if self
            .check_unit_fits_facing(kind, spot, math::vector_angle(paths.outward), None)
            .is_err()
        {
            return;
        }
        self.remove_eject(tunnel, index);
        if !self.put_on_map(unit, spot, paths.outward) {
            return;
        }
        if let Some(state) = self.objects.get_mut(&tunnel).and_then(|o| o.tunnel.as_mut()) {
            state.air_out = Some(self.tick);
        }
        if let Some(o) = self.objects.get_mut(&unit) {
            let ticks = o.def().unit.as_ref().map_or(0, |unit| unit.air_door_ticks);
            if let Some(mover) = o.mover.as_mut().filter(|_| ticks > 0) {
                mover.rise = Some(Progress::new(ticks));
            }
        }
        self.events.push(SimEvent::LeftNetwork { unit, tunnel });
        if let Some(order) = entry.order {
            self.give_order(&[unit], order, false);
        }
    }

    /// The entry at `index` of `tunnel`'s eject queue leaves it.
    fn remove_eject(&mut self, tunnel: ObjectId, index: usize) {
        if let Some(state) = self.objects.get_mut(&tunnel).and_then(|o| o.tunnel.as_mut())
            && index < state.eject.len()
        {
            state.eject.remove(index);
        }
    }
}

/// (M6-SV) The half extents, world axes, of the box of a unit of `kind` that comes out of a Tunnel facing `outward`,
/// out of Side A (plan decision T100, the side doorways' rule: `sim/producers/release.rs`'s `exit_half`): its
/// Silhouette's bounding box turned to that direction (slice M5-V1, `Shape::bound_half`). A Bike out of a Side A that
/// faces north or south is 32 su across the side and 56 su deep out of it; a square Silhouette's is its own. `None`
/// for a type with no Silhouette.
pub(in crate::sim) fn exit_half(kind: ObjectKind, outward: Vec2) -> Option<Vec2> {
    kind.def().silhouette()?;
    Some(Shape::silhouette(kind, Vec2::ZERO, math::vector_angle(outward)).bound_half())
}
