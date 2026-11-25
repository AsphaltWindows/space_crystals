//! A toggle sets a target form — design change 0198 (`factions/colonists/units.md > Weapon Conversion > A toggle sets
//! a target form`, `factions/gdo/units.md > Peacekeeper > Riot Shield > Two unit types`): the one rule of the
//! Legionnaires' Weapon Conversion (`crate::conversion`) and the Peacekeeper's Riot Shield (`crate::riot_shield`),
//! implemented once (plan decision T54). Each keeps its own state, in its own `Object` field, hashed as before.
//!
//! - **The press** ([`pressed`]): a unit with two forms, a type each, switches its type at the press to the form the
//!   press asks for, the other one for a Convert ("Pressing R **flips the unit's target form**"), the raised or the
//!   lowered type for a raise or a lowering of the shield: "The unit's type, and so its SelectionGroup, switches at the
//!   press to the target form". A press for the type the unit has already changes nothing.
//! - **The work** ([`toward`]): "the unit then works to get itself into that form": the Modes step takes the unit's
//!   own state (the form it fights with, how far up its shield is) toward its type, a tick at a time. A change that
//!   hasn't begun waits while the unit is Firing or in Cooldown (`combat::mid_attack`): "deferred, not rejected" (design
//!   change 0153), "the shot finishes, then the shield starts rising immediately" (design change 0172). So "however
//!   fast R is pressed, every unit that received the presses ends up in the same form".

use crate::combat;
use crate::defs::ObjectKind;
use crate::object::Object;

/// What a press asks of a unit with two forms (module docs).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Press {
    /// The other form: a Convert.
    Flip,
    /// This form: a raise of the Riot Shield (its raised type) or a lowering (its lowered type).
    To(ObjectKind),
}

/// The other one of a unit's two `forms`, for one of type `kind`; `None` if `kind` is neither.
pub fn other(forms: [ObjectKind; 2], kind: ObjectKind) -> Option<ObjectKind> {
    let [a, b] = forms;
    if kind == a {
        Some(b)
    } else if kind == b {
        Some(a)
    } else {
        None
    }
}

/// The type a unit of type `kind`, whose two forms are `forms`, switches to at `press` (module docs): `None` when it is
/// that type already, so that the press changes nothing, and for a type that is neither form.
pub fn pressed(forms: [ObjectKind; 2], kind: ObjectKind, press: Press) -> Option<ObjectKind> {
    let other = other(forms, kind)?;
    let target = match press {
        Press::Flip => other,
        Press::To(form) => form,
    };
    (target != kind).then_some(target)
}

/// Where a unit's own state stands toward its type in a Modes step (module docs, [`toward`]).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Toward {
    /// It is at the type: the change is over.
    There,
    /// A change that hasn't begun, held back by the unit's shot: it begins in the first Modes step after the
    /// Cooldown's last tick.
    Waits,
    /// It goes on toward the type, or begins to.
    Goes,
}

/// Where `o`'s own state stands toward its type (module docs): `there` if it is at the type; else it waits if its
/// change hasn't `begun` and `o` is Firing or in Cooldown, and goes on otherwise.
pub fn toward(o: &Object, there: bool, begun: bool) -> Toward {
    if there {
        Toward::There
    } else if !begun && combat::mid_attack(o) {
        Toward::Waits
    } else {
        Toward::Goes
    }
}

#[cfg(test)]
mod tests {
    use glam::Vec2;

    use super::*;
    use crate::combat::AttackPhase;
    use crate::ids::{ObjectId, PlayerId};
    use crate::scenarios;
    use crate::testing::TestSim;

    use ObjectKind::{Guard, Peacekeeper, PeacekeeperShielded, RifleLegionnaire, SabreLegionnaire};

    const LEGIONNAIRE: [ObjectKind; 2] = [SabreLegionnaire, RifleLegionnaire];
    const PEACEKEEPER: [ObjectKind; 2] = [Peacekeeper, PeacekeeperShielded];

    /// A Convert flips the type to the other form; a raise or a lowering makes the type the one it asks for, and
    /// changes nothing on a unit of that type already; a type with neither form has none to switch to.
    #[test]
    fn a_press_sets_the_type_to_the_form_it_asks_for() {
        assert_eq!(
            pressed(LEGIONNAIRE, SabreLegionnaire, Press::Flip),
            Some(RifleLegionnaire)
        );
        assert_eq!(
            pressed(LEGIONNAIRE, RifleLegionnaire, Press::Flip),
            Some(SabreLegionnaire)
        );
        let raise = Press::To(PeacekeeperShielded);
        let lower = Press::To(Peacekeeper);
        assert_eq!(pressed(PEACEKEEPER, Peacekeeper, raise), Some(PeacekeeperShielded));
        assert_eq!(pressed(PEACEKEEPER, PeacekeeperShielded, raise), None);
        assert_eq!(pressed(PEACEKEEPER, PeacekeeperShielded, lower), Some(Peacekeeper));
        assert_eq!(pressed(PEACEKEEPER, Peacekeeper, lower), None);
        assert_eq!(pressed(LEGIONNAIRE, Guard, Press::Flip), None);
        assert_eq!(
            (other(PEACEKEEPER, Peacekeeper), other(PEACEKEEPER, Guard)),
            (Some(PeacekeeperShielded), None)
        );
    }

    /// A change that is over is there, and one under way goes on whatever its unit does; one that hasn't begun waits
    /// while its unit is Firing or in Cooldown, the phases that can't be interrupted, and begins in any other.
    #[test]
    fn a_change_that_has_not_begun_waits_for_the_shot() {
        let mut t = TestSim::new(scenarios::empty(8, 6));
        t.spawn_unit_at(1, RifleLegionnaire, PlayerId(1), Vec2::splat(100.0))
            .step();
        let mut o = t.sim().object(ObjectId(1)).expect("the Rifle Legionnaire").clone();
        let phases = [
            (None, false),
            (Some(AttackPhase::Aiming { target: ObjectId(9) }), false),
            (Some(AttackPhase::Firing { target: None }), true),
            (Some(AttackPhase::Cooldown), true),
            (Some(AttackPhase::Reloading), false),
        ];
        for (phase, waits) in phases {
            o.attack.as_mut().expect("it attacks").phase = phase;
            let not_begun = if waits { Toward::Waits } else { Toward::Goes };
            assert_eq!(
                [
                    toward(&o, true, false),
                    toward(&o, false, true),
                    toward(&o, false, false)
                ],
                [Toward::There, Toward::Goes, not_begun],
                "{phase:?}"
            );
        }
    }
}
