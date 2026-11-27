//! The units' modes in a game (plan decision T28 moved them here from `sim.rs`): what an ability command does to the
//! units it goes to ([`Sim::use_ability`]), and the Modes step of the tick ([`Sim::step_modes`], `ARCHITECTURE.md >
//! Tick pipeline`), which takes every mode under way one tick on: a Riot Shield going up or down (`riot_shield`) and
//! a Weapon Conversion (`conversion`, slice M3-B). A mode isn't an order (`ARCHITECTURE.md > Adding a new system`): an
//! ability leaves the units' orders as they are.
//!
//! **A toggle sets a target state** (design change 0198, slice M3-T; `factions/colonists/units/weapon_conversion.md >
//! A toggle sets a target form`, `factions/gdo/units/peacekeeper.md > Riot Shield > Two unit types`; the rule is
//! `crate::toggle`'s): the Legionnaires' Convert and the Peacekeeper's Riot Shield switch the unit's type at the press
//! ("The unit's type, and so its SelectionGroup, switches at the press to the target form", "including when the raise
//! is deferred behind a shot"), through [`Sim::change_kind`] ([`Ability::switched_kind`]), and leave what the unit has
//! in the world, its weapon or its shield, to the mode's own state, which the Modes step then works toward the type
//! (`ARCHITECTURE.md > Object model > Type and form`). So the type is what the player commands, and a press reaches the
//! units as they are now: "however fast R is pressed, every unit that received the presses ends up in the same form". A
//! unit Firing or in Cooldown (`combat::mid_attack`: the phases that can't be interrupted) finishes them first (design
//! change 0153: "deferred, not rejected"; design change 0172: "the shot finishes, then the shield starts rising
//! immediately"): the first Modes step after its Cooldown's last tick, before the Reloading that followed has run a
//! tick, starts its conversion, or its shield up, as a press in that tick would.

use super::Sim;
use crate::command::Ability;
use crate::defs::ObjectKind;
use crate::ids::ObjectId;
use crate::{conversion, riot_shield};

impl Ability {
    /// The type a unit of type `kind` switches to at the press of this ability, if it does (module docs; design change
    /// 0198: "The type, and so the SelectionGroup, switches **at the press**"): a Convert flips a Legionnaire's
    /// (`conversion::converts_to`: "Pressing R **flips the unit's target form**"), a raise or a lowering of the Riot
    /// Shield makes a Peacekeeper the type it asks for, if it is the other one (`riot_shield::toggled_kind`). `None`
    /// for Set Drop-off Location, (M6-GD) a Buy and (M6-CM) a Mirage, and for a unit without the ability. What the
    /// client shows of a press before the sim has it.
    pub fn switched_kind(self, kind: ObjectKind) -> Option<ObjectKind> {
        match self {
            Ability::RiotShield { raise } => riot_shield::toggled_kind(kind, raise),
            Ability::SetDropOff(_) | Ability::Buy | Ability::Mirage => None,
            Ability::Convert => conversion::converts_to(kind),
        }
    }
}

impl Sim {
    /// `Command::Ability`, checked by the gate (`gate/orders.rs`: `units` are the player's own units that have the
    /// ability): each of them uses `ability`, at once, in id order. A toggle switches the unit's type (module docs): a
    /// Riot Shield's from where it is, a Legionnaire's keeping the form it has (`conversion::hold_form`: its weapon
    /// and its attack go on as they are) until the Modes step changes it, a conversion that hasn't started and whose
    /// form is the type again being dropped (`conversion::retarget`). A worker's AssignedDropOff changes. (M6-GD) A Buy
    /// starts each unit's purchase, paid at the press (`Sim::buy`), until its owner's bank can't pay for one. (M6-CM) A
    /// Mirage is each unit's cast (`Sim::cast_mirage`).
    pub(super) fn use_ability(&mut self, units: &[ObjectId], ability: Ability) {
        for &id in units {
            if ability == Ability::Mirage {
                self.cast_mirage(id);
                continue;
            }
            let Some(o) = self.objects.get_mut(&id) else {
                continue;
            };
            match ability {
                // (M6-CM) Cast above, the unit's borrow free for the illusions it makes.
                Ability::Mirage => {}
                // `factions/colonists/units/drone.md > Assigned Drop-off`: it "only updates AssignedDropOff"; the
                // worker's order goes on.
                Ability::SetDropOff(node) => {
                    if let Some(worker) = &mut o.worker {
                        worker.drop_off = Some(node);
                    }
                }
                // (M6-GD) `crate::stock`: paid at the press, its orders kept; served in id order until the bank can't
                // pay (design change 0083's rule for queuing).
                Ability::Buy => {
                    if !self.buy(id) {
                        break;
                    }
                }
                Ability::RiotShield { .. } | Ability::Convert => {
                    let Some(kind) = ability.switched_kind(o.kind) else {
                        continue;
                    };
                    conversion::hold_form(o);
                    self.change_kind(id, kind, None);
                    if let Some(o) = self.objects.get_mut(&id) {
                        conversion::retarget(o);
                        riot_shield::settle(o);
                    }
                }
            }
        }
    }

    /// The Modes step, for the objects before `first_new` (one created this tick starts in the next), in id order: a
    /// Riot Shield goes one tick on toward its unit's type (`riot_shield::step`), and a Weapon Conversion takes its
    /// next frame, or, its frames done, gives the unit its other form, or starts toward the unit's type
    /// (`conversion::step`); a raise or a conversion waits while the unit is Firing or in Cooldown (module docs).
    /// After the commands, so that a press counts from its own tick; before 5a, so that the behaviours, the movement
    /// and the combat of the tick all see the mode it leaves, and a converted unit fights with its new weapon from the
    /// first phase of the tick. Neither changes a kind: the types changed at the press.
    pub(super) fn step_modes(&mut self, first_new: ObjectId) {
        riot_shield::step(&mut self.objects, first_new);
        conversion::step(&mut self.objects, first_new);
    }
}

#[cfg(test)]
mod tests;
