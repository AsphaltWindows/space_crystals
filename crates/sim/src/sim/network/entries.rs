//! Entering the Tunnel Network (`crate::network`, slice M4-C): the gate's rule for an Enter order, and the step right
//! after phase 5a that takes units in.
//!
//! **Entering** ([`Sim::step_entries`], `factions/syndicate/tunnels.md > Tunnel Network > Walking in`): right after 5a,
//! in id order, each unit whose Enter order's walk brought it into the entry path leaves the map. "There is no fixed
//! hold time per unit": a unit goes in in the tick it gets there, however many went before it, so "an army files in
//! about as fast as it can walk in single file". An order whose Tunnel is gone or no longer suits the unit ends, and so
//! does one whose walk ended short of the path ("the normal Failure rule applies", design change 0247), with a
//! `SimEvent::EnterFailed`.

use crate::events::SimEvent;
use crate::ids::{ObjectId, PlayerId};
use crate::network::{self, Paths, Place};
use crate::orders::OrderKind;
use crate::orders::state::Behaviour;
use crate::sim::{Reject, Sim};

impl Sim {
    /// The gate for `OrderKind::Enter(tunnel)` (`systems/controls/commands.md > Enter`: "any Syndicate unit, when the
    /// target Tunnel's tier is sufficient for the unit's base category"), given `units`, the player's units that take
    /// orders. The Tunnel is the player's ([`Reject::NotOwned`]) and in operation ([`Reject::NotATunnel`]). The units
    /// its tier lets in take the order ([`network::admits`]), the others ignore it; none is [`Reject::NoTransit`].
    /// Any unit of the Tunnel's owner may enter, whatever faction its type belongs to: only a debug game has others.
    pub(in crate::sim) fn check_enter(
        &self,
        player: PlayerId,
        units: Vec<ObjectId>,
        tunnel: ObjectId,
    ) -> Result<Vec<ObjectId>, Reject> {
        let target = self.referenced(tunnel, Some(player))?;
        if !network::is_exit(target) {
            return Err(Reject::NotATunnel(tunnel));
        }
        let lets_in = |id: &ObjectId| (self.objects.get(id)).is_some_and(|o| network::admits(target, o.kind).is_ok());
        let admitted: Vec<ObjectId> = units.into_iter().filter(lets_in).collect();
        if admitted.is_empty() {
            return Err(Reject::NoTransit { tunnel, kind: None });
        }
        Ok(admitted)
    }

    /// The entries' step, right after phase 5a and before 5b (module docs; `ARCHITECTURE.md > Tick pipeline`): each
    /// unit from before this tick (`first_new`) whose current order is an Enter, in id order, goes into the network
    /// if it has walked into its Tunnel's entry path, and loses the order if it can't go on.
    pub(in crate::sim) fn step_entries(&mut self, first_new: ObjectId) {
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
}
