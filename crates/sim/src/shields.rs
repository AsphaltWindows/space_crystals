//! Shields — `systems/combat.md > Shields` (plan decision T18, slice M3-B): "A Colonists-only stat. Every Colonists
//! unit and building has a MaxShields value in addition to MaxHP" (`ObjectDef::max_shields`).
//!
//! - **State**: `Object::shields`, the current Shields, beside `Object::hp`: `Some` for an object whose kind has
//!   MaxShields, `None` for every other. An object enters the game with full Shields (`Sim::insert_object`, as with
//!   its HP) and keeps them through a change of kind, at most the new maximum (`Sim::change_kind`): a Weapon
//!   Conversion's two forms have equal maxima, so "current HP/Shields values carry over unchanged"
//!   (`factions/colonists/units.md > Weapon Conversion > HP/Shields`). While an object transforms, its maximum is the
//!   interpolated one ([`Object::max_shields`]): each frame its Shields rise by what that maximum rises (design
//!   change 0110: "current += (newMax − oldMax) each frame", `transform::follow_max`), and a cancel leaves them the
//!   fraction of the maximum they had (design change 0111: "Shields by the same fraction (no minimum needed)",
//!   `transform::keep_fraction`; `sim/transform.rs`). Hashed and saved like all state.
//! - **Damage** (`> Shield Damage Resolution`; `combat::damage`): a hit takes the Shields first, at full value, and
//!   only the overflow goes on to the HP through the usual modifiers, its minimum capped at the overflow. Hits are
//!   resolved as the tick's list is applied (plan decision T18), so the second hit of a tick meets the Shields the
//!   first one left.
//! - **Regeneration** ([`step`]): `> ShieldRegenRate`, 0.25 per frame ([`SHIELD_REGEN_PER_TICK`]), "continuously (not
//!   gated by a delay after taking damage)", up to the maximum now ([`Object::max_shields`]): while the object
//!   transforms, the interpolated one (design changes 0110: "Shields regenerate normally up to the current maximum",
//!   and 0146). It runs right after phase 5e's dead have left the game and before the economy (`ARCHITECTURE.md > Tick
//!   pipeline`), so the dead never regenerate and a unit hit in a tick already regenerates in it. Whoever owns the
//!   object: a defeated player's objects, which have become neutral (`core/game_rules.md > On Defeat`), regenerate as
//!   well (`> ShieldRegenRate`: "Regeneration applies to every object with Shields, including neutral ones", design
//!   change 0146).
//!
//! TODO(Fusion Core): the Fusion Core's shield upgrades raise the rate (`> ShieldRegenRate`: "each level increasing
//! the rate; per-level values TBD", `factions/colonists/structures.md > Fusion Core > Research`).

use std::collections::BTreeMap;

use crate::ids::ObjectId;
use crate::object::Object;

/// `systems/combat.md > Shields > ShieldRegenRate`: "4 Shields per second (0.25 per frame). Constant base rate shared
/// by all Colonists units and buildings". A power of two: Shields that start whole stay exact as they regenerate.
pub const SHIELD_REGEN_PER_TICK: f32 = 0.25;

/// What Shields at `shields` come to after a tick's regeneration ([`SHIELD_REGEN_PER_TICK`]), with a maximum of `max`
/// now: never above it, and Shields above it (a maximum that fell) come down to it.
pub fn regenerate(shields: f32, max: f32) -> f32 {
    (shields + SHIELD_REGEN_PER_TICK).min(max)
}

/// The regeneration of a tick (module docs): every object before `first_new` with Shields, in id order (an object
/// created this tick has full Shields, and starts in the next, `ARCHITECTURE.md > Tick pipeline`).
pub(crate) fn step(objects: &mut BTreeMap<ObjectId, Object>, first_new: ObjectId) {
    for o in objects.range_mut(..first_new).map(|(_, o)| o) {
        if o.shields.is_none() {
            continue;
        }
        let max = o.max_shields();
        if let (Some(shields), Some(max)) = (&mut o.shields, max) {
            *shields = regenerate(*shields, max);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `> ShieldRegenRate`: 0.25 a tick, so 0 to 10 in 40 ticks, exactly, and never past the maximum.
    #[test]
    fn shields_regenerate_a_quarter_a_tick_up_to_the_maximum() {
        let mut shields = 0.0;
        for tick in 1..=40 {
            shields = regenerate(shields, 10.0);
            assert_eq!(shields, tick as f32 * 0.25);
        }
        assert_eq!(shields, 10.0);
        assert_eq!(regenerate(shields, 10.0), 10.0, "never above the maximum");
        assert_eq!(regenerate(9.9, 10.0), 10.0);
        // A maximum that fell below the Shields (none of the design's does) takes them down to it.
        assert_eq!(regenerate(12.0, 10.0), 10.0);
        // Up to the maximum now: a transforming object's interpolated one (design change 0110).
        assert_eq!(regenerate(14.9, 15.0), 15.0);
    }
}
