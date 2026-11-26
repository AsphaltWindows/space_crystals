//! `systems/controls/command_behaviors.md > AttackingObject > Target inside MinRange` in a game (design change 0182):
//! the step right after phase 5a that ends the Attack orders their units can't carry out, the target inside the unit's
//! MinRange and the unit unable to move, or to back off further (design change 0204 point 3), with their feedback;
//! and the Attack Ground orders whose location is (slice M5-O3; `systems/controls/command_behaviors.md >
//! AttackingLocation`, as AttackingObject). The behaviour found it (`orders::engage::too_close`); behaviours don't
//! report, so this step does, as `Sim::step_deployments` does for the Deploys.

use super::Sim;
use crate::events::SimEvent;
use crate::ids::ObjectId;
use crate::orders::OrderKind;
use crate::orders::state::Behaviour;

impl Sim {
    /// Each unit from before this tick (`first_new`) whose Attack order phase 5a marked as too close
    /// (`AttackOrder::too_close`), or whose Attack Ground order (`AttackGround::too_close`), in id order: "the order
    /// fails with feedback ("Target too close"; same channel as other refusals)". Its current order ends and its
    /// queued ones go on (the next one starts in the next tick, as after an order that completed:
    /// `systems/controls/commands.md > Unit Command`), with a `SimEvent::TargetTooClose`, or a
    /// `SimEvent::LocationTooClose` for a location. The behaviour has already disengaged the target and stopped the
    /// unit.
    pub(super) fn step_too_close(&mut self, first_new: ObjectId) {
        let failed: Vec<(ObjectId, SimEvent)> = self
            .objects
            .range(..first_new)
            .filter_map(|(&unit, o)| {
                let active = o.orders.as_ref()?.current.as_ref()?;
                let event = match (active.order.kind, &active.state) {
                    (OrderKind::Attack(target), Behaviour::Attack(state)) if state.too_close => {
                        SimEvent::TargetTooClose { unit, target }
                    }
                    (OrderKind::AttackGround(at), Behaviour::AttackGround(state)) if state.too_close => {
                        SimEvent::LocationTooClose { unit, at }
                    }
                    _ => return None,
                };
                Some((unit, event))
            })
            .collect();
        for (unit, event) in failed {
            if let Some(orders) = self.objects.get_mut(&unit).and_then(|o| o.orders.as_mut()) {
                orders.current = None;
            }
            self.events.push(event);
        }
    }
}
