//! Weapon Conversion — `factions/colonists/units.md > Weapon Conversion` (plan decision T26, slice M3-B): "Sabre
//! Legionnaire and Rifle Legionnaire are two forms of the same underlying unit [...] differing only in
//! AttackAttributes. Either form may convert into the other at any time via the Convert command."
//!
//! - **An ability, not an order** (T26): `Ability::Convert` (`> Convert Command`: "Issue the Convert command (panel
//!   button R)") applies at once, with the tick's commands (`Sim::use_ability`), and leaves the unit's orders as they
//!   are (`> Orders are kept`, design change 0150): "The unit holds still for the 4 frames, then carries on with its
//!   current orders using the other weapon". Nothing queues it, so Shift means nothing for it (design change 0151: "Not
//!   queueable"). It has "No resource cost" (`> Cost`).
//! - **Who converts** (`> Eligibility`; the gate, `sim/gate/orders.rs`): the player's units that have another form
//!   ([`converts_to`]: "units may be mixed in selection; each converts to its respective other form") and have no
//!   conversion yet, neither under way nor waiting ([`may_convert`]), whatever their attack is doing (design change
//!   0153: "Pressed during those phases, it is deferred, not rejected"). A unit converting already, or waiting to, is
//!   left out (a placeholder reading): "Once initiated, the conversion runs to completion over its 4 frames and cannot
//!   be cancelled" (`> Completion`), and a second Convert, turning it back, would cancel it; so a second R pressed
//!   while a group still converts leaves it one form. The gate refuses the command (`Reject::CannotConvert`) only if
//!   none of them may.
//! - **The start** ([`start`]): "it may be issued freely while the unit is idle, moving, or in the Aiming or
//!   Reloading phases (both Interruptible = true), interrupting that phase immediately": an aim or a reload under way
//!   ends, and the new weapon starts ready (design change 0152: "a reload in progress on the old weapon is not carried
//!   over"). The unit keeps what its order engages.
//! - **Deferred mid-attack** (design change 0153, `> Eligibility`): "It cannot interrupt the Firing or Cooldown
//!   phases (both Interruptible = false). Pressed during those phases, it is deferred [...]: the unit finishes Firing
//!   and Cooldown, then converts immediately (instead of starting its Reloading phase). So one press of R converts a
//!   whole group in a fight, the mid-swing units a few frames later than the rest." The Convert waits as the unit's
//!   pending ability (`combat::AttackState::pending`, [`waits_to_convert`]) while it fights on, and the first Modes
//!   step after its Cooldown's last tick, before the Reloading that followed has run a tick, uses it, as a Convert
//!   given in that tick would (`Sim::step_modes`). Any ability can wait so (`sim/modes.rs`: design change 0172 will
//!   hold a Riot Shield raised mid-attack the same way, slice M3-S); the rule for "mid-attack" is
//!   `combat::mid_attack`.
//! - **Converting** ([`Conversion`], `Object::conversion`; `> Convert Command`: "The unit converts in place over 4
//!   frames", [`CONVERT_TICKS`]): it holds still. It neither moves nor turns (`combat::may_move`, `combat::may_turn`:
//!   phase 5d holds it and keeps its goal, and the behaviours start no chase), it makes way for no one (its
//!   `Mover::speed_factor` is 0, so `Mover::can_move` says it can't move in its mode), and it attacks nothing (phase
//!   5e skips it). It stays "fully attackable throughout the 4-frame conversion — no invulnerability window" (`>
//!   Vulnerability`). Its behaviours run as ever: a target they engage waits for the new weapon.
//! - **Timing** (as `ARCHITECTURE.md > Economy > Recipe timing` counts frames): converted in tick `t`, the unit spends
//!   ticks `t` to `t + 3` converting, and is its other form from tick `t + 4` on. The Modes step of that tick
//!   completes it ([`step`], `Sim::step_modes`), before any other phase: the same object changes its kind
//!   (`Sim::change_kind`, plan decision T15), with a `SimEvent::KindChanged`. Its HP and Shields are unchanged (`>
//!   HP/Shields`: "No interpolation — since both forms share identical MaxHP/MaxShields, current HP/Shields values
//!   carry over unchanged"), and so are its orders, its move and its target.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::command::Ability;
use crate::defs::ObjectKind;
use crate::hash::{StateHash, StateHasher};
use crate::ids::ObjectId;
use crate::mover::Mover;
use crate::object::Object;

/// `> Convert Command`: "The unit converts in place over 4 frames".
pub const CONVERT_TICKS: u32 = 4;

/// A unit's conversion under way (module docs): `Object::conversion` from its first frame on until it is its other
/// form, `None` otherwise.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Conversion {
    /// The frames still to come after the one under way: the Modes step of each tick takes one, and completes the
    /// conversion when none is left ([`step`]). [`CONVERT_TICKS`] until the first Modes step, then 3 in the tick of
    /// the command and 0 in its fourth tick.
    pub left: u32,
}

impl Conversion {
    /// The frames done, the one under way included: 1 in the tick of the command, [`CONVERT_TICKS`] in its fourth.
    pub fn frames_done(self) -> u32 {
        CONVERT_TICKS.saturating_sub(self.left)
    }
}

/// `> Weapon Conversion`: the other form of a `kind` unit, the one a Convert turns it into, if it has one: the Sabre
/// Legionnaire's is the Rifle Legionnaire, and the other way round (`> Sabre Legionnaire > ObjectInterfaceState`: "R:
/// Convert to Rifle Legionnaire"; `> Rifle Legionnaire > ObjectInterfaceState`: "R: Convert to Sabre Legionnaire").
pub fn converts_to(kind: ObjectKind) -> Option<ObjectKind> {
    match kind {
        ObjectKind::SabreLegionnaire => Some(ObjectKind::RifleLegionnaire),
        ObjectKind::RifleLegionnaire => Some(ObjectKind::SabreLegionnaire),
        _ => None,
    }
}

/// `> Eligibility`: whether a Convert goes to `o` (the gate, module docs): it has another form ([`converts_to`]),
/// and no conversion yet, neither under way nor waiting ([`waits_to_convert`], design change 0153).
pub fn may_convert(o: &Object) -> bool {
    converts_to(o.kind).is_some() && o.conversion.is_none() && !waits_to_convert(o)
}

/// Whether a Convert given to `o` mid-attack waits for its Cooldown's end (design change 0153, module docs). The unit
/// isn't converting yet ([`is_converting`]): it fights on.
pub fn waits_to_convert(o: &Object) -> bool {
    (o.attack.as_ref()).is_some_and(|attack| attack.pending == Some(Ability::Convert))
}

/// Whether `o` is converting: it holds still and attacks nothing (module docs). One whose Convert waits for its
/// attack isn't, yet ([`waits_to_convert`]).
pub fn is_converting(o: &Object) -> bool {
    o.conversion.is_some()
}

/// `Command::Ability` with `Ability::Convert`, to a unit the gate let through ([`may_convert`]), now: not Firing nor
/// in Cooldown (`Sim::use_ability` holds the Convert till then, design change 0153). It starts converting (module
/// docs). Nothing for a unit that may not.
pub(crate) fn start(o: &mut Object) {
    if !may_convert(o) {
        return;
    }
    o.conversion = Some(Conversion { left: CONVERT_TICKS });
    // "interrupting that phase immediately": Aiming or Reloading (a Convert given while Firing or in Cooldown waited
    // for its end); the new weapon starts ready (design change 0152).
    if let Some(attack) = &mut o.attack {
        attack.phase = None;
        attack.elapsed = 0;
    }
    // It can't move in this mode: it makes way for no one (`Mover::can_move`).
    if let Some(mover) = &mut o.mover {
        mover.speed_factor = 0.0;
    }
}

/// The conversions' part of the Modes step: each unit before `first_new` that converts, in id order, takes its next
/// frame. Returns those that have none left, with the form each becomes, for `Sim::step_modes` to complete
/// ([`finish`] after `Sim::change_kind`).
pub(crate) fn step(objects: &mut BTreeMap<ObjectId, Object>, first_new: ObjectId) -> Vec<(ObjectId, ObjectKind)> {
    let mut complete = Vec::new();
    for o in objects.range_mut(..first_new).map(|(_, o)| o) {
        let Some(conversion) = &mut o.conversion else {
            continue;
        };
        if conversion.left > 0 {
            conversion.left -= 1;
        } else if let Some(to) = converts_to(o.kind) {
            complete.push((o.id, to));
        } else {
            // A form without another one (a hand-edited save) has nothing to become.
            finish(o);
        }
    }
    complete
}

/// A conversion is over: `o` no longer converts, and moves at its full speed again. A Legionnaire has no other mode
/// that slows it (a Riot Shield's would, `riot_shield`).
pub(crate) fn finish(o: &mut Object) {
    o.conversion = None;
    if let Some(mover) = &mut o.mover {
        mover.speed_factor = Mover::default().speed_factor;
    }
}

impl StateHash for Conversion {
    fn hash(&self, h: &mut StateHasher) {
        let Conversion { left } = self;
        left.hash(h);
    }
}

#[cfg(test)]
mod tests;
