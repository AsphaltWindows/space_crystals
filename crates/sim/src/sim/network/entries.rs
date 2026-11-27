//! Entering the Tunnel Network (`crate::network`, slice M4-C), or a building that holds units (`crate::holding`, slice
//! M5-F1): the gate's rule for an Enter order, and the step right after phase 5a that takes units in. One rule for
//! both, by the one entry path (`network::EntryPath`).
//!
//! **Entering** ([`Sim::step_entries`], `factions/syndicate/tunnels.md > Tunnel Network > Walking in`): right after 5a,
//! in id order, each unit whose Enter order's walk brought it into the entry path leaves the map. "There is no fixed
//! hold time per unit": a unit goes in in the tick it gets there, however many went before it, so "an army files in
//! about as fast as it can walk in single file". An order whose target is gone or no longer suits the unit ends, and
//! so does one whose walk ended short of the path ("the normal Failure rule applies", design change 0247), or that got
//! to a building's path with its pool full (design change 0347 point 2, `factions/cults/structures/README.md >
//! Entering pool buildings`: "a unit arriving when the building is full has its Enter **end** and idles beside the
//! building (the normal Failure rule)"), with a `SimEvent::EnterFailed`.

use crate::events::SimEvent;
use crate::holding;
use crate::ids::{ObjectId, PlayerId};
use crate::network::{self, EntryPath, Place};
use crate::orders::OrderKind;
use crate::orders::state::Behaviour;
use crate::shape::Shape;
use crate::sim::{Reject, Sim};

impl Sim {
    /// The gate for `OrderKind::Enter(target)` (`systems/controls/commands.md > Enter`: "Order the unit to enter an own
    /// structure that can hold units"), given `units`, the player's units that take orders. The target is the player's
    /// ([`Reject::NotOwned`]), and a Tunnel in operation ([`Reject::NotATunnel`]: "any Syndicate unit, when the target
    /// Tunnel's tier is sufficient for the unit's base category"), or a building that holds units and isn't under
    /// construction ([`Reject::UnderConstruction`]; design change 0347 point 4: "a building under construction
    /// **can't be entered**"; slice M5-F1). The units it lets in take the order ([`network::admits`]), the others
    /// ignore it; none is [`Reject::NoTransit`] at a Tunnel and [`Reject::NotAccepted`] at a building. A building's
    /// room is judged as a unit gets there, not here (0347 point 2). Any unit of the target's owner may enter, whatever
    /// faction its type belongs to: only a debug game has others.
    pub(in crate::sim) fn check_enter(
        &self,
        player: PlayerId,
        units: Vec<ObjectId>,
        target: ObjectId,
    ) -> Result<Vec<ObjectId>, Reject> {
        let object = self.referenced(target, Some(player))?;
        let building = holding::holds(object).is_some();
        if building && !object.is_operational() {
            return Err(Reject::UnderConstruction(target));
        }
        if !building && !network::is_exit(object) {
            return Err(Reject::NotATunnel(target));
        }
        let lets_in = |id: &ObjectId| (self.objects.get(id)).is_some_and(|o| network::admits(object, o.kind).is_ok());
        let admitted: Vec<ObjectId> = units.into_iter().filter(lets_in).collect();
        if admitted.is_empty() {
            return Err(match building {
                true => Reject::NotAccepted(target),
                false => Reject::NoTransit {
                    tunnel: target,
                    kind: None,
                },
            });
        }
        Ok(admitted)
    }

    /// The entries' step, right after phase 5a and before 5b (module docs; `ARCHITECTURE.md > Tick pipeline`): each
    /// unit from before this tick (`first_new`) whose current order is an Enter, in id order, goes in if it has walked
    /// into its target's entry path, and loses the order if it can't go on.
    pub(in crate::sim) fn step_entries(&mut self, first_new: ObjectId) {
        let entering: Vec<(ObjectId, ObjectId, bool)> = self
            .objects
            .range(..first_new)
            .filter_map(|(&id, o)| {
                let active = o.orders.as_ref()?.current.as_ref()?;
                match (active.order.kind, &active.state) {
                    (OrderKind::Enter(target), Behaviour::Enter(state)) => Some((id, target, state.walked)),
                    _ => None,
                }
            })
            .collect();
        for (unit, target, walked) in entering {
            self.arrive_at_entry(unit, target, walked);
        }
    }

    /// Unit `unit` is on an Enter order to `target`, its walk over or not (`walked`).
    /// - **The target suits it** (it stands, is its owner's and lets the unit in, [`network::admits`]) **and the unit
    ///   has walked into the entry path** ([`EntryPath::reached`], (M6-SV) by its box as it lies: a vehicle's
    ///   Silhouette turned with its facing, `Shape::bound_half`): it goes into the network
    ///   (`SimEvent::EnteredNetwork`), or into the building's pool (`SimEvent::EnteredBuilding`) if the pool has room
    ///   ([`Sim::room`]), on its way or at the end of its walk alike, so a line never pauses. With no room, the order
    ///   ends as below, `full` (design change 0347 point 2).
    /// - **The target doesn't suit it** (`factions/syndicate/tunnels.md > Tunnel Network > Walking in > Target Tunnel
    ///   gone or unsuitable`), **or its walk ended short of the path** (the normal Failure rule, design change 0247;
    ///   the occupied-target rule's arrival behind standing units is such an end too): "the Enter order ends and the
    ///   unit idles" (`SimEvent::EnterFailed`). Its queued orders go on from the next tick, as after any order that
    ///   ends.
    /// - Otherwise it walks on.
    fn arrive_at_entry(&mut self, unit: ObjectId, target: ObjectId, walked: bool) {
        let Some(o) = self.objects.get(&unit) else {
            return;
        };
        let suits = self.objects.get(&target).filter(|target| {
            target.owner.is_some() && target.owner == o.owner && network::admits(target, o.kind).is_ok()
        });
        let half = o.def().silhouette().map(|_| Shape::of(o).bound_half());
        let reached =
            (suits.and_then(EntryPath::of_object).zip(half)).is_some_and(|(path, half)| path.reached(o.pos, half));
        let building = suits.is_some_and(|target| holding::holds(target).is_some());
        match (reached, building) {
            (true, false) => {
                if self.take_off_map(unit, Place::Network, true) {
                    self.events.push(SimEvent::EnteredNetwork { unit, tunnel: target });
                }
            }
            (true, true) if self.room(target) > 0 => {
                if self.take_off_map(unit, Place::Building(target), true) {
                    self.events.push(SimEvent::EnteredBuilding { unit, building: target });
                }
            }
            (true, true) => self.end_entry(unit, target, true),
            (false, _) if suits.is_some() && !walked => {}
            (false, _) => self.end_entry(unit, target, false),
        }
    }

    /// `unit`'s Enter order on `target` ends where it stands, the unit idle (`SimEvent::EnterFailed`; `full`: for want
    /// of room in a building's pool).
    fn end_entry(&mut self, unit: ObjectId, target: ObjectId, full: bool) {
        let Some(o) = self.objects.get_mut(&unit) else {
            return;
        };
        if let Some(orders) = &mut o.orders {
            orders.current = None;
        }
        if let Some(mover) = &mut o.mover {
            mover.stop();
        }
        self.events.push(SimEvent::EnterFailed { unit, target, full });
    }
}
