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
//!
//! (M6-SD) **Descending** ([`Sim::fly_in`], plan decision T96): an air unit goes in by a Tier 3 Tunnel's air entry
//! door instead, once its centre is over the door (`orders::enter`, the plan's D46): it descends into it for its
//! type's `UnitDef::air_door_ticks` (design change 0431: "it **descends** into the entry door [...] over **16 frames
//! (1 s)** [...], and it can be hit during both **as an air unit**"), on the map meanwhile, and leaves the map once the
//! descent is over. The door lets one aircraft begin its descent every `Tunnel::air_door_interval_ticks` (design change
//! 0251: "one aircraft per 8 frames [...] (the same rate for the air entry door)"), the first in id order of those over
//! it, so that two may be descending at once, as two may be rising out of the exit door.

use glam::Vec2;

use crate::economy::RATIO_ONE;
use crate::events::SimEvent;
use crate::holding;
use crate::ids::{ObjectId, PlayerId};
use crate::mirage;
use crate::network::{self, EntryPath, Paths, Place};
use crate::orders::OrderKind;
use crate::orders::state::{Behaviour, Enter};
use crate::progress::Progress;
use crate::shape::Shape;
use crate::sim::{Reject, Sim};
use crate::tunnel::TunnelState;

impl Sim {
    /// The gate for `OrderKind::Enter(target)` (`systems/controls/commands.md > Enter`: "Order the unit to enter an own
    /// structure that can hold units"), given `units`, the player's units that take orders. The target is the player's
    /// ([`Reject::NotOwned`]), and a Tunnel in operation ([`Reject::NotATunnel`]: "any Syndicate unit, when the target
    /// Tunnel's tier is sufficient for the unit's base category"), or a building that holds units and isn't under
    /// construction ([`Reject::UnderConstruction`]; design change 0347 point 4: "a building under construction
    /// **can't be entered**"; slice M5-F1). The units it lets in take the order ([`network::admits`]), the others
    /// ignore it; none is [`Reject::NoTransit`] at a Tunnel and [`Reject::NotAccepted`] at a building. A building's
    /// room is judged as a unit gets there, not here (0347 point 2). Any unit of the target's owner may enter, whatever
    /// faction its type belongs to: only a debug game has others. (M6-CM) An illusion doesn't go into a building that
    /// augments (`mirage::admitted`: "Illusions can't be augmented at the Altar themselves").
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
        let lets_in = |id: &ObjectId| {
            (self.objects.get(id))
                .is_some_and(|o| network::admits(object, o.kind).is_ok() && mirage::admitted(object, o))
        };
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
    /// into its target's entry path, and loses the order if it can't go on; (M6-SD) an air unit descends by its air
    /// entry door ([`Sim::fly_in`]).
    pub(in crate::sim) fn step_entries(&mut self, first_new: ObjectId) {
        let entering: Vec<(ObjectId, ObjectId, Enter, bool)> = self
            .objects
            .range(..first_new)
            .filter_map(|(&id, o)| {
                let active = o.orders.as_ref()?.current.as_ref()?;
                match (active.order.kind, &active.state) {
                    (OrderKind::Enter(target), Behaviour::Enter(state)) => {
                        Some((id, target, *state, network::uses_air_doors(o.kind)))
                    }
                    _ => None,
                }
            })
            .collect();
        for (unit, target, state, flies) in entering {
            if flies {
                self.fly_in(unit, target, state);
            } else {
                self.arrive_at_entry(unit, target, state.walked);
            }
        }
    }

    /// (M6-SD) Unit `unit`, an air unit (`network::uses_air_doors`), is on an Enter order to `target` in `state`
    /// (module docs, `orders::enter`):
    /// - **The target doesn't suit it** (it stands, is its owner's and lets the unit in, [`network::admits`]: a Tunnel
    ///   in operation at Tier 3, the tier it had before an upgrade under way): "the Enter order ends and the unit
    ///   idles" (`SimEvent::EnterFailed`), where it is, descending or not.
    /// - **It descends**: one more tick of its descent; the descent over, it goes into the network
    ///   (`SimEvent::EnteredNetwork`).
    /// - **Its centre is over the air entry door** (`network::over_door`, at its `network::door_spot`) **and the door
    ///   is free**
    ///   ([`TunnelState::door_free`]): its descent begins, and the door is taken for its interval
    ///   (`TunnelState::air_in`). A type that takes no time at the doors (`UnitDef::air_door_ticks` 0) goes in at
    ///   once (the plan's D46).
    /// - Otherwise it flies on to the door, or waits over it for its turn.
    fn fly_in(&mut self, unit: ObjectId, target: ObjectId, state: Enter) {
        let Some(o) = self.objects.get(&unit) else {
            return;
        };
        let suits = self.objects.get(&target).filter(|target| {
            target.owner.is_some() && target.owner == o.owner && network::admits(target, o.kind).is_ok()
        });
        let (Some(paths), Some(tunnel)) = (
            suits.and_then(Paths::of_object),
            suits.and_then(|target| target.tunnel.as_ref()),
        ) else {
            self.end_entry(unit, target, false);
            return;
        };
        if let Some(mut descent) = state.descent {
            if descent.step(RATIO_ONE) {
                self.go_in(unit, target);
            } else {
                self.descend(unit, descent);
            }
            return;
        }
        let half = o.def().half_extents().unwrap_or(Vec2::ZERO);
        let door = network::door_spot(&self.map, paths.air_entry, half);
        if !network::over_door(door, o.pos) || !TunnelState::door_free(tunnel.air_in, self.tick) {
            return;
        }
        let ticks = o.def().unit.as_ref().map_or(0, |unit| unit.air_door_ticks);
        if let Some(tunnel) = self.objects.get_mut(&target).and_then(|o| o.tunnel.as_mut()) {
            tunnel.air_in = Some(self.tick);
        }
        if ticks == 0 {
            self.go_in(unit, target);
        } else {
            self.descend(unit, Progress::new(ticks));
        }
    }

    /// (M6-SD) Air unit `unit`'s Enter descends, `descent` of it done (`Enter::descent`).
    fn descend(&mut self, unit: ObjectId, descent: Progress) {
        let active = (self.objects.get_mut(&unit))
            .and_then(|o| o.orders.as_mut())
            .and_then(|orders| orders.current.as_mut());
        if let Some(active) = active
            && let Behaviour::Enter(state) = &mut active.state
        {
            state.descent = Some(descent);
        }
    }

    /// Unit `unit` goes into the network by `tunnel` (`SimEvent::EnteredNetwork`).
    fn go_in(&mut self, unit: ObjectId, tunnel: ObjectId) {
        if self.take_off_map(unit, Place::Network, true) {
            self.events.push(SimEvent::EnteredNetwork { unit, tunnel });
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
            target.owner.is_some()
                && target.owner == o.owner
                && network::admits(target, o.kind).is_ok()
                && mirage::admitted(target, o)
        });
        let half = o.def().silhouette().map(|_| Shape::of(o).bound_half());
        let reached =
            (suits.and_then(EntryPath::of_object).zip(half)).is_some_and(|(path, half)| path.reached(o.pos, half));
        let building = suits.is_some_and(|target| holding::holds(target).is_some());
        match (reached, building) {
            (true, false) => self.go_in(unit, target),
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
