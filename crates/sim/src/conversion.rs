//! Weapon Conversion — `factions/colonists/units.md > Weapon Conversion` (plan decision T26, slice M3-B; design change
//! 0198, slice M3-T): "Sabre Legionnaire and Rifle Legionnaire are two forms of the same underlying unit [...]
//! differing only in AttackAttributes. Either form may convert into the other at any time via the Convert command."
//!
//! **A toggle sets a target form** (design change 0198, `> A toggle sets a target form`): "Pressing R **flips the
//! unit's target form**; the unit then works to get itself into that form. So however fast R is pressed, every unit
//! that received the presses ends up in the same form". While a change is pending, a Legionnaire is two forms at once
//! (`ARCHITECTURE.md > Object model > Type and form`):
//! - its **type**, `Object::kind`, is the form it works toward: "The unit's type, and so its SelectionGroup, switches
//!   at the press to the target form" (`Sim::change_kind`, the same object). Whatever commands the unit reads it: the
//!   client's selection, groups and panel, the gate, the default command;
//! - its **form** ([`form`]) is the one it has in the world now, whose weapon it fights with (`combat::weapon`): the
//!   one it converts from, or still fights with while its conversion waits, as its [`Conversion`] holds it
//!   (`Object::conversion`), else its type. The forms share everything but the weapon ("an identical physical
//!   profile"), so the shape, the sight and the speed are its type's as well as its form's.
//!
//! - **An ability, not an order** (T26): `Ability::Convert` flips the type of each unit it goes to, at once, with the
//!   tick's commands (`Sim::use_ability`), and leaves the unit's orders as they are (`> Orders are kept`, design change
//!   0150): "The unit holds still for the 4 frames, then carries on with its current orders using the other weapon".
//!   Nothing queues it, so Shift means nothing for it (design change 0151: "Not queueable"). It has "No resource cost"
//!   (`> Cost`).
//! - **Who** (the gate, `sim/gate/orders.rs`): every one of the player's units with another form ([`converts_to`]),
//!   whatever it is doing, a conversion waiting or under way included: a press is a flip of the target, never refused.
//!   The client sends it to the ActiveGroup alone (`> Convert Command`: "It goes only to the ActiveGroup"), whose
//!   units all have one type, and switches their group at the press (design change 0198).
//! - **Working toward the type** (the Modes step, [`step`]; `> A toggle sets a target form`: "Then each unit works
//!   toward its target"):
//!   - **waiting** (design change 0153, `> Eligibility`: "It cannot interrupt the Firing or Cooldown phases [...].
//!     Pressed during those phases, it is deferred, not rejected: the unit finishes Firing and Cooldown, then converts
//!     immediately (instead of starting its Reloading phase)"): a unit whose form isn't its type, Firing or in
//!     Cooldown (`combat::mid_attack`), finishes them with its form's weapon ([`waits_to_convert`]); the first Modes
//!     step after its Cooldown's last tick, before the Reloading that followed has run a tick, starts its conversion,
//!     so one press converts a whole group in a fight, the mid-swing units a few frames later. Pressed again
//!     meanwhile, "it just takes the new target; if that is its current weapon again, it doesn't convert at all"
//!     (`> Eligibility`: "the waiting conversion is dropped. The shot already under way still finishes");
//!   - **the start**: "it may be issued freely while the unit is idle, moving, or in the Aiming or Reloading phases
//!     (both Interruptible = true), interrupting that phase immediately": an aim or a reload under way ends, and the
//!     new weapon starts ready (design change 0152: "a reload in progress on the old weapon is not carried over").
//!     The unit keeps what its order engages;
//!   - **converting** ([`is_converting`]; `> Convert Command`: "The unit converts in place over 4 frames", its def's
//!     `ConversionDef::ticks`): it holds still. It neither moves nor turns (`combat::may_move`, `combat::may_turn`:
//!     phase 5d holds it and keeps its goal, and the behaviours start no chase), it makes way for no one (its
//!     `Mover::speed_factor` is 0, so `Mover::can_move` says it can't move in its mode), and it attacks nothing
//!     (phase 5e skips it). It stays "fully attackable throughout the 4-frame conversion — no invulnerability window"
//!     (`> Vulnerability`). Its behaviours run as ever: a target they engage waits for the new weapon. "Once the
//!     4-frame conversion has actually started, it runs to completion and cannot be interrupted" (`> Completion`),
//!     whatever the presses do to its type meanwhile;
//!   - **the end**: the unit has its other form, its HP and Shields unchanged (`> HP/Shields`: "No interpolation —
//!     since both forms share identical MaxHP/MaxShields, current HP/Shields values carry over unchanged"), and so are
//!     its orders, its move and its target. "If the target form changed meanwhile, the unit converts back right
//!     after" (`> Completion`): the same Modes step starts the conversion back (a round trip takes 8 frames, longer
//!     than either form's reload, as `> Orders are kept > New weapon starts ready` reckons).
//! - **Timing** (as `ARCHITECTURE.md > Economy > Recipe timing` counts frames): pressed in tick `t`, the unit is of its
//!   new type from tick `t` on; converting from `t` (or from the tick after its Cooldown), it spends ticks `t` to
//!   `t + 3` converting and has its new form from tick `t + 4` on: the Modes step of that tick completes the
//!   conversion, before any other phase. Its type changed at the press (with a `SimEvent::KindChanged`); the end of the
//!   conversion changes only its form.

use std::collections::BTreeMap;

use serde::{Deserialize, Deserializer, Serialize};

use crate::combat;
use crate::defs::{ConversionDef, ObjectKind};
use crate::hash::{StateHash, StateHasher};
use crate::ids::ObjectId;
use crate::mover::Mover;
use crate::object::Object;

/// A Legionnaire's change of form that is pending (module docs): `Object::conversion` from the press that makes its
/// type another form than the one it has until it has its type's form again, `None` otherwise.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Conversion {
    /// The form the unit has now, whose weapon it fights with: the one it converts from, or, while its conversion
    /// waits, the one it fights with until its Cooldown is over. Its type (`Object::kind`) is the form it works
    /// toward.
    pub form: ObjectKind,
    /// The conversion under way, from `form` to its other form: the frames still to come after the one under way, 3
    /// in the tick it starts and 0 in its fourth; the Modes step of each tick takes one, and completes the conversion
    /// when none is left ([`step`]). `None` while it hasn't started: the unit's attack holds it back (it is Firing or
    /// in Cooldown), or the press came after this tick's Modes step began (a unit created this tick).
    pub left: Option<u32>,
}

impl Conversion {
    /// The frames of the conversion under way done, the one under way included: 1 in the tick it starts, all of its
    /// form's (`ConversionDef::ticks`: 4) in its last; `None` while none is under way.
    pub fn frames_done(self) -> Option<u32> {
        self.left.map(|left| ticks(self.form).saturating_sub(left))
    }
}

/// The conversion of a `kind` unit, its def's (`UnitDef::conversion`): the Legionnaire's two forms and the frames a
/// conversion takes.
fn def_of(kind: ObjectKind) -> Option<ConversionDef> {
    kind.def().unit.as_ref()?.conversion
}

/// The frames a conversion from `form` takes (`> Convert Command`: "The unit converts in place over 4 frames").
fn ticks(form: ObjectKind) -> u32 {
    def_of(form).map_or(0, |conversion| conversion.ticks)
}

/// `> Weapon Conversion`: the other form of a `kind` unit, the one a Convert turns it into, if it has one
/// (`ConversionDef::forms`): the Sabre Legionnaire's is the Rifle Legionnaire, and the other way round (`> Sabre
/// Legionnaire > ObjectInterfaceState`: "R: Convert to Rifle Legionnaire"; `> Rifle Legionnaire >
/// ObjectInterfaceState`: "R: Convert to Sabre Legionnaire").
pub fn converts_to(kind: ObjectKind) -> Option<ObjectKind> {
    def_of(kind)?.other(kind)
}

/// The form `o` has in the world now (module docs): the one its pending conversion holds, else its type. What the
/// unit fights with (`combat::weapon`) and what the client draws it as.
pub fn form(o: &Object) -> ObjectKind {
    form_of(o.kind, o.conversion.as_ref())
}

/// The form of a unit of type `kind` whose pending conversion is `conversion` ([`form`]).
pub fn form_of(kind: ObjectKind, conversion: Option<&Conversion>) -> ObjectKind {
    conversion.map_or(kind, |conversion| conversion.form)
}

/// Whether `o` is converting, the 4 frames under way: it holds still and attacks nothing (module docs). One whose
/// conversion waits for its attack isn't, yet ([`waits_to_convert`]).
pub fn is_converting(o: &Object) -> bool {
    o.conversion.is_some_and(|conversion| conversion.left.is_some())
}

/// Whether `o`'s conversion waits (design change 0153, module docs): its form isn't its type, and its conversion
/// hasn't started. It fights on with its form's weapon until its Cooldown is over.
pub fn waits_to_convert(o: &Object) -> bool {
    o.conversion
        .is_some_and(|conversion| conversion.left.is_none() && conversion.form != o.kind)
}

/// A press is about to switch `o`'s type (`Sim::use_ability`, before `Sim::change_kind`): the form it has stays its
/// form, now one its type may no longer be, so that `Sim::change_kind` keeps its weapon, and its attack, as they are.
/// Nothing for a unit that has a pending conversion already, which holds its form.
pub(crate) fn hold_form(o: &mut Object) {
    if o.conversion.is_none() && converts_to(o.kind).is_some() {
        o.conversion = Some(Conversion {
            form: o.kind,
            left: None,
        });
    }
}

/// After a press switched `o`'s type: a conversion that hasn't started and whose form is the type again is dropped,
/// the unit having nothing to change ("if that is its current weapon again, it doesn't convert at all"). One under
/// way runs on.
pub(crate) fn retarget(o: &mut Object) {
    if o.conversion.is_some_and(|c| c.left.is_none() && c.form == o.kind) {
        finish(o);
    }
}

/// The conversions' part of the Modes step (module docs): each unit before `first_new` with a pending conversion, in
/// id order, takes the next frame of the one under way, or, its frames done, has its other form; then one that isn't
/// converting works toward its type: it has it, and its conversion is over; or it starts converting to it, unless
/// it is Firing or in Cooldown, and waits.
pub(crate) fn step(objects: &mut BTreeMap<ObjectId, Object>, first_new: ObjectId) {
    for o in objects.range_mut(..first_new).map(|(_, o)| o) {
        let Some(conversion) = &mut o.conversion else {
            continue;
        };
        match conversion.left {
            Some(0) => {
                // `> Completion`: its four frames are done.
                conversion.form = converts_to(conversion.form).unwrap_or(conversion.form);
                conversion.left = None;
            }
            Some(left) => {
                conversion.left = Some(left - 1);
                continue;
            }
            None => {}
        }
        let Conversion { form, .. } = *conversion;
        // A type that is neither the form nor its other one (a hand-edited save) has nothing to convert to.
        if form == o.kind || converts_to(form) != Some(o.kind) {
            finish(o);
        } else if !combat::mid_attack(o) {
            start(o);
        }
    }
}

/// `o` starts converting from its form (module docs), this tick its first frame: an aim or a reload under way ends
/// (a Firing or a Cooldown never is: the conversion waited for its end), the new weapon to start ready (design
/// change 0152), and it can't move in this mode, so it makes way for no one (`Mover::can_move`).
fn start(o: &mut Object) {
    if let Some(conversion) = &mut o.conversion {
        conversion.left = Some(ticks(conversion.form).saturating_sub(1));
    }
    if let Some(attack) = &mut o.attack {
        attack.phase = None;
        attack.elapsed = 0;
    }
    if let Some(mover) = &mut o.mover {
        mover.speed_factor = 0.0;
    }
}

/// `o` has its type's form: its conversion is over, and it moves at its full speed again. A Legionnaire has no other
/// mode that slows it (a Riot Shield's would, `riot_shield`).
fn finish(o: &mut Object) {
    o.conversion = None;
    if let Some(mover) = &mut o.mover {
        mover.speed_factor = Mover::default().speed_factor;
    }
}

/// `Object::conversion` as a file has it. A state saved before design change 0198 has a conversion under way as its
/// frames left alone (`{"left": 2}`), its unit's type then being the form it converted from: it loads as none, the
/// unit keeping that form, and its press is lost (a save in the 4 frames of a conversion; its other change, a Convert
/// or a raise waiting for a shot in the attack's `pending`, which files no longer have, is lost alike).
pub(crate) fn saved<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<Conversion>, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Saved {
        Now(Conversion),
        Before {
            #[serde(rename = "left")]
            _left: u32,
        },
    }
    Ok(match Option::<Saved>::deserialize(deserializer)? {
        Some(Saved::Now(conversion)) => Some(conversion),
        Some(Saved::Before { .. }) | None => None,
    })
}

impl StateHash for Conversion {
    fn hash(&self, h: &mut StateHasher) {
        let Conversion { form, left } = self;
        form.hash(h);
        left.hash(h);
    }
}

#[cfg(test)]
mod tests;
