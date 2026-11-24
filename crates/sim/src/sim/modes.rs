//! The units' modes in a game (plan decision T28 moved them here from `sim.rs`): what an ability command does to the
//! units it goes to ([`Sim::use_ability`]), and the Modes step of the tick ([`Sim::step_modes`], `ARCHITECTURE.md >
//! Tick pipeline`), which takes every mode under way one tick on. A mode isn't an order (`ARCHITECTURE.md > Adding a
//! new system`): an ability leaves the units' orders as they are.

use super::Sim;
use crate::command::Ability;
use crate::ids::ObjectId;
use crate::riot_shield;

impl Sim {
    /// `Command::Ability`, checked by the gate (`gate/orders.rs`: `units` are the player's own units that have the
    /// ability): each of them uses `ability`, at once, in id order.
    pub(super) fn use_ability(&mut self, units: &[ObjectId], ability: Ability) {
        for id in units {
            let Some(o) = self.objects.get_mut(id) else {
                continue;
            };
            match ability {
                Ability::RiotShield { raise } => riot_shield::toggle(o, raise),
                // `factions/colonists/units.md > Drone > Assigned Drop-off`: it "only updates AssignedDropOff"; the
                // worker's order goes on.
                Ability::SetDropOff(node) => {
                    if let Some(worker) = &mut o.worker {
                        worker.drop_off = Some(node);
                    }
                }
            }
        }
    }

    /// The Modes step, for the objects before `first_new` (one created this tick starts in the next): a Riot Shield
    /// on its way up or down goes on one tick (`riot_shield::step`). After the commands, so that a toggle counts
    /// from its own tick; before 5a, so that the behaviours, the movement and the combat of the tick all see the mode
    /// it leaves.
    pub(super) fn step_modes(&mut self, first_new: ObjectId) {
        riot_shield::step(&mut self.objects, first_new);
    }
}
