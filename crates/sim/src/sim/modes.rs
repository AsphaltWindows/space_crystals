//! The units' modes in a game (plan decision T28 moved them here from `sim.rs`): what an ability command does to the
//! units it goes to ([`Sim::use_ability`]), and the Modes step of the tick ([`Sim::step_modes`], `ARCHITECTURE.md >
//! Tick pipeline`), which takes every mode under way one tick on: a Riot Shield going up or down (`riot_shield`) and
//! a Weapon Conversion (`conversion`, slice M3-B), which ends in a change of kind. A mode isn't an order
//! (`ARCHITECTURE.md > Adding a new system`): an ability leaves the units' orders as they are.
//!
//! **The Riot Shield's toggle** switches the unit's type at once (design change 0173: "The type switches **when the
//! toggle starts**"), through [`Sim::change_kind`] like a conversion's end, and the shield then goes up or down a
//! tick at a time from where it is (`riot_shield`).
//!
//! **Abilities that wait for an attack** (design change 0153, `factions/colonists/units.md > Weapon Conversion >
//! Eligibility`: "Pressed during those phases, it is deferred, not rejected: the unit finishes Firing and Cooldown,
//! then converts immediately (instead of starting its Reloading phase)"; design change 0172, `factions/gdo/units.md >
//! Peacekeeper > Riot Shield > During the toggle`: "the raise is **deferred**: the shot finishes, then the shield
//! starts rising immediately. Same as the Legionnaires' Convert"): such an ability given to a unit Firing or in
//! Cooldown (`combat::mid_attack`) waits in its `AttackState::pending`, and the first Modes step after its Cooldown's
//! last tick uses it, before anything else of the tick, so the Reloading that the Cooldown's end began hasn't run a
//! tick: the unit acts as if given it in that tick. [`waits_for_attack`] says which abilities wait: the Convert, and
//! raising the Riot Shield.

use super::Sim;
use crate::combat;
use crate::command::Ability;
use crate::ids::ObjectId;
use crate::{conversion, riot_shield};

impl Sim {
    /// `Command::Ability`, checked by the gate (`gate/orders.rs`: `units` are the player's own units that have the
    /// ability, and for a Convert those that may convert now): each of them uses `ability`, at once, in id order, or,
    /// Firing or in Cooldown, once its attack lets it (module docs).
    pub(super) fn use_ability(&mut self, units: &[ObjectId], ability: Ability) {
        for &id in units {
            let Some(o) = self.objects.get_mut(&id) else {
                continue;
            };
            if waits_for_attack(ability) && combat::mid_attack(o) {
                if let Some(attack) = &mut o.attack {
                    attack.pending = Some(ability);
                }
                continue;
            }
            self.apply_ability(id, ability);
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
        // below count this tick for them as for one given now: a raised shield's first tick up, a conversion's
        // first frame.
        let ready: Vec<(ObjectId, Ability)> = self
            .objects
            .range_mut(..first_new)
            .filter(|(_, o)| !combat::mid_attack(o))
            .filter_map(|(&id, o)| Some((id, o.attack.as_mut()?.pending.take()?)))
            .collect();
        for (id, ability) in ready {
            self.apply_ability(id, ability);
        }
        riot_shield::step(&mut self.objects, first_new);
        for (id, to) in conversion::step(&mut self.objects, first_new) {
            self.change_kind(id, to, None);
            if let Some(o) = self.objects.get_mut(&id) {
                conversion::finish(o);
            }
        }
    }

    /// Object `id` uses `ability` now: its Riot Shield's toggle switches its type (`riot_shield::toggled_kind`), its
    /// worker's AssignedDropOff changes, its conversion starts.
    fn apply_ability(&mut self, id: ObjectId, ability: Ability) {
        let Some(o) = self.objects.get_mut(&id) else {
            return;
        };
        match ability {
            Ability::RiotShield { raise } => {
                // A lowering cancels a raise that waits for an attack (`riot_shield`, a placeholder reading).
                if let Some(attack) = &mut o.attack
                    && !raise
                    && attack.pending == Some(Ability::RiotShield { raise: true })
                {
                    attack.pending = None;
                }
                if let Some(kind) = riot_shield::toggled_kind(o, raise) {
                    self.change_kind(id, kind, None);
                }
                if let Some(o) = self.objects.get_mut(&id) {
                    riot_shield::settle(o);
                }
            }
            // `factions/colonists/units.md > Drone > Assigned Drop-off`: it "only updates AssignedDropOff"; the
            // worker's order goes on.
            Ability::SetDropOff(node) => {
                if let Some(worker) = &mut o.worker {
                    worker.drop_off = Some(node);
                }
            }
            Ability::Convert => conversion::start(o),
        }
    }
}

/// Whether `ability`, given to a unit Firing or in Cooldown, waits for its Cooldown's end (module docs): a Convert
/// (design change 0153) and a raise of the Riot Shield (design change 0172). The others apply at once in any phase:
/// lowering the Riot Shield, which a unit attacking never needs (only a Peacekeeper whose shield is all the way down
/// attacks: `riot_shield::allows_attack`), and Set Drop-off Location.
fn waits_for_attack(ability: Ability) -> bool {
    matches!(ability, Ability::Convert | Ability::RiotShield { raise: true })
}

#[cfg(test)]
mod tests;
