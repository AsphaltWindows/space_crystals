//! (M5-M1) The hit-point invariant H1: every object, on the map or held off it (`Sim::held`), has HP exactly when it
//! has a MaxHP, and its HP is above 0 and at most its MaxHP with its Augmentations (`Object::max_hp_with`, slice M5-N1a):
//! the M5 plan's "repair never lifts HP above its maximum" (slice M), for every way HP rises or MaxHP falls. That is a
//! repair (`sim/repairs.rs`), a regeneration (`crate::regeneration`: the Tunnel Network's and the expansions'), a change
//! of a unit's Augmentations (`augmentation::follow_max`: "MaxHP goes down: HP is unchanged, unless it is now above the
//! new MaxHP", `factions/cults/augmentation.md > Increments`), of its kind (`Sim::change_kind`: a trained unit keeps
//! its damage), of a Tunnel's tier, or a transformation's or a site's maximum. Its Shields, where its kind has them
//! (the Colonists'), are within 0 and its MaxShields now (`Object::max_shields`, `systems/combat.md > Shields`).
//!
//! **The dead are out of the game** when it is checked, after the step: an object brought to 0 HP or less dies, and
//! leaves in the phase that killed it, through `Sim::remove_objects` (phase 5e's hits, `combat::damage::apply`, and
//! the neutral expansions' decay, `Sim::step_decay`). Every other rule that lowers HP keeps
//! it above 0: a MaxHP that falls never kills (`transform::follow_max`, `augmentation::follow_max`), and a cancelled
//! transformation leaves at least `MIN_HP_AFTER_CANCEL`. So an object at 0 HP or less between ticks is one that a phase
//! left in the game.

use super::{Violation, every_object};
use crate::sim::Sim;

/// H1: every object's HP is above 0 and at most its MaxHP, and its Shields within 0 and its MaxShields (module docs).
pub(super) fn hit_points(sim: &Sim) -> Result<(), Violation> {
    let research = sim.research();
    for (o, held) in every_object(sim) {
        let fail = |message: String| {
            let held = if held { " (held)" } else { "" };
            Err(Violation {
                invariant: "H1",
                message: format!("#{} ({:?}){held} {message}", o.id.0, o.kind),
                at: o.pos,
            })
        };
        match (o.hp, o.max_hp_with(research)) {
            (None, None) => {}
            (Some(hp), Some(max)) => {
                if hp.is_nan() || hp <= 0.0 {
                    return fail(format!("has {hp} HP of {max}: it is dead, and still in the game"));
                }
                if hp > max {
                    return fail(format!("has {hp} HP, more than its MaxHP of {max}"));
                }
            }
            (Some(hp), None) => return fail(format!("has {hp} HP, and no MaxHP")),
            (None, Some(max)) => return fail(format!("has no HP, and a MaxHP of {max}")),
        }
        match (o.shields, o.max_shields()) {
            (None, None) => {}
            (Some(shields), Some(max)) => {
                if !(0.0..=max).contains(&shields) {
                    return fail(format!("has {shields} Shields, out of 0 to its MaxShields of {max}"));
                }
            }
            (Some(shields), None) => return fail(format!("has {shields} Shields, and no MaxShields")),
            (None, Some(max)) => return fail(format!("has no Shields, and a MaxShields of {max}")),
        }
    }
    Ok(())
}
