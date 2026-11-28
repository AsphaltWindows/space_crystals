//! Surges: an attack whose hits spend from its object's energy pool (`crate::energy`), and Amplify
//! (`factions/syndicate/units/laser_battery.md > Battery`, `> Attack: constant beam with surges`, `> Amplify
//! (ability)`). Shaped by the def: an attack with a [`SurgeDef`] (`AttackAttrs::surge`) on a type with an energy pool;
//! nothing here names a type.
//!
//! - **A surge** is the attack's hit at the end of its Firing (`combat::phases`, a FullyConnected one, which can't
//!   miss). It uses the def's draw if the pool holds it, else whatever is left, or the whole pool once Amplify marked
//!   it ([`used`]), and deals `constant × used^(2/3)` ([`damage`]: "Damage per surge is proportional to the charge
//!   used^(2/3)"). Its charge is spent with the tick's hits (`combat::step`), before the energy step regenerates the
//!   pool, so the pool charges every tick, attacking or not.
//! - **The surge interval** is the attack's cycle: a surge is over once its reload is, and the next follows at once,
//!   at what the unit aims at, without a new aim (`phases::Shooter::beam_on`; "a **constant thin red laser** connects
//!   the Laser Battery to its target. Periodic **surges**"), while it could still begin an aim at it and faces it.
//!   Placeholder (not in the design: what keeps the beam on; another target it already faces needs no aim). So an empty
//!   pool fires just as often, each surge using what regenerated since the last (`> Battery`: "**Empty battery**").
//! - **Amplify** (`Ability::Amplify`): marks the unit's next surge to use the whole pool (`Object::amplified`; design
//!   change 0369: "pressing it makes the next surge use the whole battery. It does not fire at once"). An ability, so
//!   not queueable, its orders kept; the gate gives it to the units whose attack has surges ([`has`]), in either mode.
//!   A change of kind keeps the mark while the new type's attack has surges (`Sim::change_kind`).

use crate::defs::{ObjectKind, SurgeDef};
use crate::object::Object;
use crate::{energy, math};

/// The surges of `kind`'s attack, if it has them.
pub fn of_kind(kind: ObjectKind) -> Option<SurgeDef> {
    kind.def().unit.as_ref()?.attack?.surge
}

/// Whether `o` has Amplify: its attack has surges (the gate's `Reject::LacksAbility` otherwise).
pub fn has(o: &Object) -> bool {
    of_kind(o.kind).is_some()
}

/// What a surge of `def` uses of `energy` (`Object::energy`'s units): its draw if the pool holds it, else whatever is
/// left ("a surge uses the normal surge charge if the battery holds it, otherwise whatever is left"); the whole pool if
/// Amplify marked it (`amplified`).
pub fn used(def: &SurgeDef, energy: u32, amplified: bool) -> u32 {
    if amplified {
        energy
    } else {
        energy.min(energy::units(def.draw))
    }
}

/// The damage of a surge of `def` that uses `used` (`Object::energy`'s units): `constant × used^(2/3)`, `used` in
/// points,
/// as the square of its cube root through `math::cbrt`, the same bits on every platform.
pub fn damage(def: &SurgeDef, used: u32) -> f32 {
    let root = math::cbrt(energy::points(used));
    def.constant * root * root
}

/// A surge `o` would fire now: what it uses and deals (`combat::phases::Shooter`).
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct Surge {
    pub used: u32,
    pub damage: f32,
}

/// The surge of `def` that `o` would fire now, by its energy and its mark.
pub(crate) fn next(o: &Object, def: &SurgeDef) -> Surge {
    let used = used(def, o.energy.unwrap_or(0), o.amplified);
    Surge {
        used,
        damage: damage(def, used),
    }
}

/// `o` fired a surge that used `used`: it is spent from its pool (`energy::spend`; never more than the pool holds,
/// [`used`]) and its mark is cleared ("the next surge" only).
pub(crate) fn spend(o: &mut Object, used: u32) {
    if let Some(energy) = &mut o.energy {
        energy::spend(energy, used.min(*energy));
    }
    o.amplified = false;
}

/// Amplify (module docs): `o`'s next surge uses its whole pool, if its attack has surges.
pub(crate) fn amplify(o: &mut Object) {
    if has(o) {
        o.amplified = true;
    }
}

#[cfg(test)]
mod tests;
