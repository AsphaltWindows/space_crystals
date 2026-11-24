//! The units' modes in a game (plan decision T28 moved them here from `sim.rs`): what an ability command does to the
//! units it goes to ([`Sim::use_ability`]), and the Modes step of the tick ([`Sim::step_modes`], `ARCHITECTURE.md >
//! Tick pipeline`), which takes every mode under way one tick on: a Riot Shield going up or down (`riot_shield`) and
//! a Weapon Conversion (`conversion`, slice M3-B), which ends in a change of kind. A mode isn't an order
//! (`ARCHITECTURE.md > Adding a new system`): an ability leaves the units' orders as they are.
//!
//! **Abilities that wait for an attack** (design change 0153, `factions/colonists/units.md > Weapon Conversion >
//! Eligibility`: "Pressed during those phases, it is deferred, not rejected: the unit finishes Firing and Cooldown,
//! then converts immediately (instead of starting its Reloading phase)"): such an ability given to a unit Firing or
//! in Cooldown (`combat::mid_attack`) waits in its `AttackState::pending`, and the first Modes step after its
//! Cooldown's last tick uses it, before anything else of the tick, so the Reloading that the Cooldown's end began
//! never runs a tick: the unit acts as if given it in that tick. [`waits_for_attack`] says which abilities wait: the
//! Convert; design change 0172 adds raising the Riot Shield (slice M3-S), which needs no more than that line.

use super::Sim;
use crate::combat;
use crate::command::Ability;
use crate::ids::ObjectId;
use crate::object::Object;
use crate::{conversion, riot_shield};

impl Sim {
    /// `Command::Ability`, checked by the gate (`gate/orders.rs`: `units` are the player's own units that have the
    /// ability, and for a Convert those that may convert now): each of them uses `ability`, at once, in id order.
    pub(super) fn use_ability(&mut self, units: &[ObjectId], ability: Ability) {
        for id in units {
            let Some(o) = self.objects.get_mut(id) else {
                continue;
            };
            if waits_for_attack(ability) && combat::mid_attack(o) {
                if let Some(attack) = &mut o.attack {
                    attack.pending = Some(ability);
                }
                continue;
            }
            apply(o, ability);
        }
    }

    /// The Modes step, for the objects before `first_new` (one created this tick starts in the next), in id order: a
    /// Riot Shield on its way up or down goes on one tick (`riot_shield::step`), and a Weapon Conversion takes its
    /// next frame, or, with none left, completes (`conversion::step`): the unit becomes its other form
    /// ([`Sim::change_kind`]), its HP, Shields, orders, move and target kept. After the commands, so that a toggle
    /// or a Convert counts from its own tick; before 5a, so that the behaviours, the movement and the combat of the
    /// tick all see the mode it leaves, and a converted unit acts as its new form from the first phase of the tick.
    /// The kind of a unit changes no footprint, so phase 5b has nothing to follow.
    pub(super) fn step_modes(&mut self, first_new: ObjectId) {
        // The abilities that waited for an attack whose Cooldown is over (module docs), first, so that the steps
        // below count this tick for them as for one given now.
        for o in self.objects.range_mut(..first_new).map(|(_, o)| o) {
            if combat::mid_attack(o) {
                continue;
            }
            if let Some(ability) = o.attack.as_mut().and_then(|attack| attack.pending.take()) {
                apply(o, ability);
            }
        }
        riot_shield::step(&mut self.objects, first_new);
        for (id, to) in conversion::step(&mut self.objects, first_new) {
            self.change_kind(id, to, None);
            if let Some(o) = self.objects.get_mut(&id) {
                conversion::finish(o);
            }
        }
    }
}

/// Whether `ability`, given to a unit Firing or in Cooldown, waits for its Cooldown's end (module docs): a Convert
/// (design change 0153). The other abilities apply at once in any phase: the Riot Shield (until design change 0172's
/// raise, slice M3-S) and Set Drop-off Location.
fn waits_for_attack(ability: Ability) -> bool {
    matches!(ability, Ability::Convert)
}

/// `o` uses `ability` now: the shield goes up or down, the worker's AssignedDropOff changes, the conversion starts.
fn apply(o: &mut Object, ability: Ability) {
    match ability {
        Ability::RiotShield { raise } => riot_shield::toggle(o, raise),
        // `factions/colonists/units.md > Drone > Assigned Drop-off`: it "only updates AssignedDropOff"; the worker's
        // order goes on.
        Ability::SetDropOff(node) => {
            if let Some(worker) = &mut o.worker {
                worker.drop_off = Some(node);
            }
        }
        Ability::Convert => conversion::start(o),
    }
}

#[cfg(test)]
mod tests;
