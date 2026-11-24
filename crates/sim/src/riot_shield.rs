//! The Peacekeeper's Riot Shield — `factions/gdo/units.md > Peacekeeper > Riot Shield (toggled ability)`.
//!
//! **Two unit types** (design change 0173, `> Two unit types`): "shield down and shield up are **two unit types**,
//! **Peacekeeper** and **Peacekeeper (Shielded)**, like the Sabre and Rifle Legionnaire [...]. They share everything
//! except what the shield changes. Raising or lowering the shield switches the unit's type." Both carry the same
//! [`RiotShieldDef`], which names them (`RiotShieldDef::lowered`, `::raised`), and "the type switches **when the
//! toggle starts**": a unit whose shield is up or on its way up is a Peacekeeper (Shielded), one whose shield is down
//! or on its way down a Peacekeeper. So the client groups the two apart (`> In a selection, the two types form
//! separate SelectionGroups`) and gives each its own command: Raise Shield on the Peacekeeper, Lower Shield on the
//! Peacekeeper (Shielded), whose panel has no Attack.
//!
//! A **mode**, not an order. The player raises and lowers it with an ability command (`Command::Ability`), which
//! applies at once and leaves the unit's orders alone (`Sim::use_ability`), so the shield goes up while the unit walks,
//! holds or stands idle. A mode isn't a task (design change 0023, `systems/units.md > UnitCollision > Ground Collision
//! > Friendly yielding`): an idle unit with its shield up still yields, at its shielded speed.
//!
//! **Orders are kept** (design change 0150, `> Riot Shield > Orders are kept`): toggling never clears the command
//! queue, and an order the unit can't carry out while shielded is on hold, not dropped. An explicit Attack follows
//! its target while the shield is up and attacks it again once it is down (`orders::engage::attack`); an AttackMove
//! or a Patrol only moves meanwhile, and engages along the way again once it is down (`orders::travel::march`: the
//! scans find nothing while the unit may not attack). **Not queueable** (design change 0151): the toggle always
//! applies at once; the command has no queue flag.
//!
//! - **State**: a unit whose type has a [`RiotShieldDef`] has a [`RiotShield`] (`Object::riot_shield`): how far up
//!   the shield is, in ticks of the toggle time (design change 0171: "Shield state = progress 0..1 with direction"),
//!   the direction being the unit's type. [`mode`] tells it as the design does, Lowered, Raising, Raised or Lowering
//!   ([`ShieldMode`]). A Peacekeeper starts with its shield down; a Peacekeeper (Shielded) put in the game as one (a
//!   map's or a debug spawn) with its shield up. Hashed and serialized like all state.
//! - **Toggling** (`Sim::use_ability`, `sim/modes.rs`): the unit becomes the type the command asks for at once, with
//!   the tick's commands (`Sim::change_kind`, plan decision T15: the same object, its HP, orders, move and attack
//!   kept; the weapon is the same, so a reload goes on), and its shield sets off that way from where it is: "Pressing
//!   the toggle again mid-way **reverses from the current point** (halfway up → lowering takes only the half it
//!   already moved)" (design change 0171). Raising a shield that is up or on its way up changes nothing, nor does
//!   lowering one that is down or on its way down. The Modes step takes the shield one tick on ([`step`]), after the
//!   commands and before phase 5a, so a toggle in tick `t` from all the way down (or up) has the shield all the way up
//!   (or down) at the end of tick `t + toggle_ticks − 1`, and every phase of that tick already sees it so.
//! - **Pressed mid-attack** (design change 0172, `> During the toggle`): "pressed during the Firing or Cooldown phase
//!   (not interruptible), the raise is **deferred**: the shot finishes, then the shield starts rising immediately.
//!   Same as the Legionnaires' Convert". A raise given then waits as the unit's pending ability
//!   (`combat::AttackState::pending`), the unit still a Peacekeeper, and the first Modes step after its Cooldown uses
//!   it, as a raise given in that tick would be (`sim/modes.rs`). Unlike a conversion, it leaves the weapon alone: the
//!   Reloading that the Cooldown's end began goes on behind the shield. A lowering never waits: a Peacekeeper
//!   (Shielded) never attacks.
//! - **Effects** (`> Riot Shield`: "While the shield is up"; `> During the toggle`):
//!   - **Half speed** from the start of a raise to the end of a lowering: the Peacekeeper (Shielded) moves at its own
//!     MaxSpeed, the Peacekeeper's halved ("MaxSpeed is halved (4 → 2 space units/frame)"), and a Peacekeeper whose
//!     shield is on its way down at that speed too ("While raising **or** lowering, the Peacekeeper moves at half
//!     speed (as with the shield up)"): its `Mover::speed_factor` is the ratio of the two types' MaxSpeeds until its
//!     shield is all the way down ([`speed_factor`]). Movement and yielding go at that speed.
//!   - **No attacking** but by a Peacekeeper whose shield is all the way down ([`allows_attack`]): with the shield up
//!     it "cannot attack", and "while raising **or** lowering, the Peacekeeper [...] **cannot attack**". The
//!     behaviours engage nothing and pick no target (`orders::world::Unit::attrs`: no auto-targeting either), phase 5e
//!     drops whatever is still engaged, which cancels an aim (`combat::step`; Firing and Cooldown run on, so a shot
//!     already fired lands, and a raise pressed then waits for them, above), and the gate takes no new Attack order
//!     for the unit (`Sim::validate`, `Reject::CannotAttack`: "Attack orders are not possible"; design change 0075),
//!     while one it had goes on as a follow (above).
//!   - Incoming damage × `RiotShieldDef::damage_factor` (halved), a pre-armor multiplier from every direction
//!     (`combat::pre_armor_multiplier`, `systems/combat.md > DamageModifierOrder` step 1; [`damage_factor`]), "**only
//!     once the shield is fully up**; a half-raised shield gives none" (design change 0171).
//!
//! Placeholder readings, where the design is silent:
//! - **Attack orders while lowering**: none, as while the shield is up. "Cannot attack" (design change 0171) is read
//!   to cover a new Attack order too, as "the player must lower the shield before ordering an attack" says of a
//!   raised one: a unit is ordered to attack once its shield is all the way down. So a right-click on an enemy
//!   follows it meanwhile (design change 0074), and an Attack leaves the unit out (0075).
//! - **A lowering given while a raise waits for an attack** (only a command made by hand can do it: a Peacekeeper's
//!   panel offers Raise Shield alone) cancels the raise: the shield stays down.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::defs::{ObjectKind, RiotShieldDef};
use crate::hash::{StateHash, StateHasher};
use crate::ids::ObjectId;
use crate::object::Object;

/// A unit's Riot Shield (module docs): how far up it is. Which way it goes is the unit's type ([`mode`]).
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RiotShield {
    /// The ticks of the toggle time the shield is up: 0 all the way down, `RiotShieldDef::toggle_ticks` all the way
    /// up.
    pub up: u32,
}

impl RiotShield {
    /// The shield a unit of `kind`, whose type carries `def`, starts with when it comes into the game: down on the
    /// lowered type, up on the raised one (module docs).
    pub fn fresh(kind: ObjectKind, def: &RiotShieldDef) -> Self {
        let up = if kind == def.raised { def.toggle_ticks } else { 0 };
        Self { up }
    }
}

/// What a Riot Shield is doing, as the design tells it ([`mode`]): from its unit's type and how far up it is.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ShieldMode {
    /// All the way down, on a Peacekeeper: it "behaves normally" (`> Riot Shield`: "While the shield is down").
    Lowered,
    /// On its way up, on a Peacekeeper (Shielded), `left` ticks to go.
    Raising { left: u32 },
    /// All the way up, on a Peacekeeper (Shielded).
    Raised,
    /// On its way down, on a Peacekeeper, `left` ticks to go.
    Lowering { left: u32 },
}

impl ShieldMode {
    /// How far up the shield is, from 0 (down) to 1 (up), with a toggle time of `toggle_ticks`: for drawing it.
    pub fn progress(self, toggle_ticks: u32) -> f32 {
        let ticks = toggle_ticks.max(1) as f32;
        match self {
            ShieldMode::Lowered => 0.0,
            ShieldMode::Raising { left } => toggle_ticks.saturating_sub(left) as f32 / ticks,
            ShieldMode::Raised => 1.0,
            ShieldMode::Lowering { left } => left.min(toggle_ticks) as f32 / ticks,
        }
    }
}

/// The Riot Shield of `o`'s type, if it has one.
pub fn def_of(o: &Object) -> Option<&'static RiotShieldDef> {
    o.def().unit.as_ref()?.riot_shield.as_ref()
}

/// What `o`'s Riot Shield is doing (module docs), from its type and how far up it is; `None` for an object without
/// one.
pub fn mode(o: &Object) -> Option<ShieldMode> {
    let shield = o.riot_shield?;
    let def = def_of(o)?;
    let ticks = def.toggle_ticks;
    Some(if o.kind == def.raised {
        match ticks.saturating_sub(shield.up) {
            0 => ShieldMode::Raised,
            left => ShieldMode::Raising { left },
        }
    } else {
        match shield.up.min(ticks) {
            0 => ShieldMode::Lowered,
            left => ShieldMode::Lowering { left },
        }
    })
}

/// Whether `o`'s Riot Shield lets it attack now: only a Peacekeeper whose shield is all the way down attacks (module
/// docs); true for objects without one. The one "is this unit shielded" check of the sim and the client: the
/// behaviours, phase 5e, auto-targeting, the gate, the default command and the command panel all ask it.
pub fn allows_attack(o: &Object) -> bool {
    mode(o).is_none_or(|mode| mode == ShieldMode::Lowered)
}

/// The pre-armor multiplier `o`'s Riot Shield gives the hits against it: its `RiotShieldDef::damage_factor` once it
/// is all the way up, else none (design change 0171: "a half-raised shield gives none").
pub fn damage_factor(o: &Object) -> f32 {
    match (mode(o), def_of(o)) {
        (Some(ShieldMode::Raised), Some(def)) => def.damage_factor,
        _ => 1.0,
    }
}

/// The `Mover::speed_factor` `o`'s Riot Shield gives it (module docs): a Peacekeeper whose shield is on its way down
/// moves at the raised type's MaxSpeed, the ratio of the two types' MaxSpeeds; anything else at its own type's.
pub fn speed_factor(o: &Object) -> f32 {
    match (mode(o), def_of(o)) {
        (Some(ShieldMode::Lowering { .. }), Some(def)) => {
            let max_speed = |kind: ObjectKind| kind.def().unit.as_ref().map_or(0.0, |unit| unit.movement.max_speed());
            let lowered = max_speed(def.lowered);
            if lowered > 0.0 {
                max_speed(def.raised) / lowered
            } else {
                1.0
            }
        }
        _ => 1.0,
    }
}

/// The type `o` becomes when the player raises (`raise`) or lowers its Riot Shield (module docs): its shield's raised
/// or lowered type. `None` for an object without a shield, and when it is that type already: the shield goes that way
/// already, and the toggle changes nothing.
pub(crate) fn toggled_kind(o: &Object, raise: bool) -> Option<ObjectKind> {
    o.riot_shield?;
    let def = def_of(o)?;
    let kind = if raise { def.raised } else { def.lowered };
    (kind != o.kind).then_some(kind)
}

/// `o`'s speed follows its type and its Riot Shield ([`speed_factor`]), after a toggle or a tick of the shield. The
/// shield is the only mode that sets the speed of a unit that carries one. Nothing for an object without one.
pub(crate) fn settle(o: &mut Object) {
    if o.riot_shield.is_none() {
        return;
    }
    let factor = speed_factor(o);
    if let Some(mover) = &mut o.mover {
        mover.speed_factor = factor;
    }
}

/// The Riot Shields' part of the Modes step: on each object before `first_new`, in id order (`ARCHITECTURE.md > Tick
/// pipeline`: one created this tick starts in the next), the shield goes one tick further the way its unit's type
/// takes it (module docs), and the unit's speed follows.
pub(crate) fn step(objects: &mut BTreeMap<ObjectId, Object>, first_new: ObjectId) {
    for o in objects.range_mut(..first_new).map(|(_, o)| o) {
        let Some(shield) = o.riot_shield else {
            continue;
        };
        let Some(def) = def_of(o) else {
            continue;
        };
        let up = if o.kind == def.raised {
            shield.up.saturating_add(1).min(def.toggle_ticks)
        } else {
            shield.up.min(def.toggle_ticks).saturating_sub(1)
        };
        if up != shield.up {
            o.riot_shield = Some(RiotShield { up });
            settle(o);
        }
    }
}

impl StateHash for RiotShield {
    /// As an `Option` of how far up it is, `None` all the way down: a shield that is down hashes as it did before the
    /// two types (design change 0173), so only a game that raises one hashes otherwise.
    fn hash(&self, h: &mut StateHasher) {
        let RiotShield { up } = *self;
        match up {
            0 => h.tag(0),
            up => {
                h.tag(1);
                up.hash(h);
            }
        }
    }
}

#[cfg(test)]
mod tests;
