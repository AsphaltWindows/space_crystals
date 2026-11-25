//! The Peacekeeper's Riot Shield — `factions/gdo/units/peacekeeper.md > Riot Shield (toggled ability)`.
//!
//! **Two unit types** (design change 0173, `> Two unit types`): "shield down and shield up are **two unit types**,
//! **Peacekeeper** and **Peacekeeper (Shielded)**, like the Sabre and Rifle Legionnaire [...]. They share everything
//! except what the shield changes. Raising or lowering the shield switches the unit's type." Both carry the same
//! [`RiotShieldDef`], which names them (`RiotShieldDef::lowered`, `::raised`). So the client groups the two apart (`>
//! In a selection, the two types form separate SelectionGroups`) and gives each its own command: Raise Shield on the
//! Peacekeeper, Lower Shield on the Peacekeeper (Shielded), whose panel has no Attack.
//!
//! **A toggle sets a target state** (design change 0198, `> Two unit types`: "pressing R flips the Peacekeeper's target
//! (shield up or down), and the unit works to reach it. The type, and so the SelectionGroup, switches **at the press**,
//! including when the raise is deferred behind a shot"), the rule of `crate::toggle`. As a Legionnaire's
//! (`crate::conversion`), a Peacekeeper's type is the state it works toward, and its shield the state it is in
//! (`ARCHITECTURE.md > Object model > Type and form`): its type is the way the shield goes, and what commands the unit
//! reads; its shield is how far up it is ([`RiotShield`]), and what it does in the world, the speed, the attack and the
//! damage taken, reads the two together ([`mode`]).
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
//!   the direction being the unit's type. [`mode`] tells it as the design does, Lowered, Deferred, Raising, Raised or
//!   Lowering ([`ShieldMode`]). A Peacekeeper starts with its shield down; a Peacekeeper (Shielded) put in the game as
//!   one (a map's or a debug spawn) with its shield up. Hashed and serialized like all state.
//! - **Toggling** (`Sim::use_ability`, `sim/modes.rs`): the unit becomes the type the command asks for at once, with
//!   the tick's commands (`Sim::change_kind`, plan decision T15: the same object, its HP, orders, move and attack
//!   kept; the weapon is the same, so a shot or a reload goes on), and its shield then works toward it from where it
//!   is: "Pressing the toggle again mid-way **reverses from the current point** (halfway up → lowering takes only the
//!   half it already moved)" (design change 0171). Raising a shield that is up or on its way up changes nothing, nor
//!   does lowering one that is down or on its way down. The Modes step takes the shield one tick on ([`step`]), after
//!   the commands and before phase 5a, so a toggle in tick `t` from all the way down (or up) has the shield all the
//!   way up (or down) at the end of tick `t + toggle_ticks − 1`, and every phase of that tick already sees it so.
//! - **Pressed mid-attack** (design change 0172, `> During the toggle`): "pressed during the Firing or Cooldown phase
//!   (not interruptible), the raise is **deferred**: the shot finishes, then the shield starts rising immediately.
//!   Same as the Legionnaires' Convert". The unit is a Peacekeeper (Shielded) from the press on (design change 0198),
//!   its shield down ([`ShieldMode::Deferred`]) while it is Firing or in Cooldown (`combat::mid_attack`): its shot
//!   lands, and the first Modes step after its Cooldown's last tick starts the shield up. The raise leaves the weapon
//!   alone: the Reloading that the Cooldown's end began goes on behind the shield. "**Lower pressed while a raise is
//!   still deferred**" (design change 0198, answering Q0053 point 2): "the target flips back to shield down, so the
//!   waiting raise is dropped; the shield never starts rising": the unit is a Peacekeeper again, its shield down, its
//!   shot going on. A lowering never waits: a shield off the ground has no shot to wait for (its unit doesn't attack).
//! - **Effects** (`> Riot Shield`: "While the shield is up"; `> During the toggle`), from the shield's state:
//!   - **Half speed** from the start of a raise to the end of a lowering: the unit moves at the Peacekeeper
//!     (Shielded)'s MaxSpeed while its shield is off the ground, the Peacekeeper's halved ("MaxSpeed is halved (4 → 2
//!     space units/frame)"; "While raising **or** lowering, the Peacekeeper moves at half speed (as with the shield
//!     up)"), and at the Peacekeeper's while it is all the way down, whatever its type: its `Mover::speed_factor` is
//!     the ratio of that MaxSpeed to its type's ([`speed_factor`]). Movement and yielding go at that speed. (A raise
//!     that waits for a shot leaves the shield down, at full speed; the unit's attack holds it still meanwhile.)
//!   - **No attacking** but by a Peacekeeper whose shield is all the way down ([`allows_attack`]): with the shield up
//!     it "cannot attack", and "while raising **or** lowering, the Peacekeeper [...] **cannot attack**"; nor does a
//!     Peacekeeper (Shielded) whose raise waits for its shot, which lands. The behaviours engage nothing and pick no
//!     target (`orders::world::Unit::attrs`: no auto-targeting either), and phase 5e drops whatever is still engaged,
//!     which cancels an aim (`combat::step`; Firing and Cooldown run on, so a shot already fired lands). An Attack
//!     order the unit has goes on as a follow meanwhile (above), and attacks once the shield is down.
//!   - Incoming damage × `RiotShieldDef::damage_factor` (halved), a pre-armor multiplier from every direction
//!     (`combat::pre_armor_multiplier`, `systems/combat.md > DamageModifierOrder` step 1; [`damage_factor`]), "**only
//!     once the shield is fully up**; a half-raised shield gives none" (design change 0171).
//! - **Orders given during the toggle** (design change 0202, answering Q0053 point 1, `> During the toggle`): "which
//!   orders the unit accepts follows its new type from the press; carrying them out waits for the toggle to finish"
//!   ([`takes_attack_orders`]). A Peacekeeper (Shielded), its shield up or on its way up, takes no Attack order (the
//!   gate, `Sim::validate`, `Reject::CannotAttack`: "Attack orders are not possible"; design change 0075), and a
//!   right-click on an enemy makes it follow (design change 0074, `orders::resolution::Abilities`). A Peacekeeper
//!   whose shield is on its way down takes both, "**accepted and held**, and the attack starts as soon as the shield
//!   is fully down": its Attack follows the target until then (`orders::engage::attack`, as behind a raised shield).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::defs::{ObjectKind, RiotShieldDef};
use crate::hash::{StateHash, StateHasher};
use crate::ids::ObjectId;
use crate::object::Object;
use crate::toggle::{self, Press, Toward};

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

/// What a Riot Shield is doing, as the design tells it ([`mode`]): from its unit's type, how far up it is and, for a
/// raise that hasn't begun, the unit's attack.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ShieldMode {
    /// All the way down, on a Peacekeeper: it "behaves normally" (`> Riot Shield`: "While the shield is down").
    Lowered,
    /// All the way down, on a Peacekeeper (Shielded) Firing or in Cooldown (`combat::mid_attack`): the raise is
    /// deferred until its shot is over (design change 0172), the type switched already (design change 0198).
    Deferred,
    /// On its way up, on a Peacekeeper (Shielded), `left` ticks to go: `RiotShieldDef::toggle_ticks` for a raise that
    /// begins in the next Modes step.
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
            ShieldMode::Lowered | ShieldMode::Deferred => 0.0,
            ShieldMode::Raising { left } => toggle_ticks.saturating_sub(left) as f32 / ticks,
            ShieldMode::Raised => 1.0,
            ShieldMode::Lowering { left } => left.min(toggle_ticks) as f32 / ticks,
        }
    }
}

/// The Riot Shield of `o`'s type, if it has one.
pub fn def_of(o: &Object) -> Option<&'static RiotShieldDef> {
    def_of_kind(o.kind)
}

/// The Riot Shield of type `kind`, if it has one.
fn def_of_kind(kind: ObjectKind) -> Option<&'static RiotShieldDef> {
    kind.def().unit.as_ref()?.riot_shield.as_ref()
}

/// What `o`'s Riot Shield is doing (module docs), from its type and how far up it is, and whether its attack holds a
/// raise back (`toggle::toward`: a shield off the ground has begun, so a lowering never waits); `None` for an object
/// without one.
pub fn mode(o: &Object) -> Option<ShieldMode> {
    let shield = o.riot_shield?;
    let def = def_of(o)?;
    let ticks = def.toggle_ticks;
    let raising = o.kind == def.raised;
    let there = if raising { shield.up >= ticks } else { shield.up == 0 };
    Some(match (toggle::toward(o, there, shield.up > 0), raising) {
        (Toward::There, true) => ShieldMode::Raised,
        (Toward::There, false) => ShieldMode::Lowered,
        (Toward::Waits, _) => ShieldMode::Deferred,
        (Toward::Goes, true) => ShieldMode::Raising {
            left: ticks.saturating_sub(shield.up),
        },
        (Toward::Goes, false) => ShieldMode::Lowering {
            left: shield.up.min(ticks),
        },
    })
}

/// Whether `o`'s Riot Shield lets it attack now: only a Peacekeeper whose shield is all the way down attacks (module
/// docs); true for objects without one. What the unit does in the world asks it: the behaviours, auto-targeting and
/// phase 5e. Which orders it takes is its type's ([`takes_attack_orders`], design change 0202).
pub fn allows_attack(o: &Object) -> bool {
    mode(o).is_none_or(|mode| mode == ShieldMode::Lowered)
}

/// Whether a unit of type `kind` takes Attack orders (design change 0202, module docs: "which orders the unit
/// accepts follows its new type from the press"): every type but a Riot Shield's raised one, the Peacekeeper
/// (Shielded) ("Attack orders are not possible"). A Peacekeeper takes them while its shield is on its way down too,
/// and attacks once it is all the way down ([`allows_attack`]). The gate (`Sim::validate`), the default command
/// (`orders::resolution::Abilities`) and the client's panel and right-click ask it.
pub fn takes_attack_orders(kind: ObjectKind) -> bool {
    def_of_kind(kind).is_none_or(|def| kind != def.raised)
}

/// The pre-armor multiplier `o`'s Riot Shield gives the hits against it: its `RiotShieldDef::damage_factor` once it
/// is all the way up, else none (design change 0171: "a half-raised shield gives none").
pub fn damage_factor(o: &Object) -> f32 {
    match (mode(o), def_of(o)) {
        (Some(ShieldMode::Raised), Some(def)) => def.damage_factor,
        _ => 1.0,
    }
}

/// The `Mover::speed_factor` `o`'s Riot Shield gives it (module docs): it moves at the raised type's MaxSpeed while
/// its shield is off the ground, and at the lowered type's while it is all the way down, whatever its type: the factor
/// is the ratio of that MaxSpeed to its type's. A Peacekeeper whose shield is on its way down moves at half its own
/// MaxSpeed, a Peacekeeper (Shielded) whose raise waits for its shot (held still by its attack) at twice its own. 1
/// for an object without one.
pub fn speed_factor(o: &Object) -> f32 {
    let (Some(shield), Some(def)) = (o.riot_shield, def_of(o)) else {
        return 1.0;
    };
    let max_speed = |kind: ObjectKind| kind.def().unit.as_ref().map_or(0.0, |unit| unit.movement.max_speed());
    let moves_as = if shield.up == 0 { def.lowered } else { def.raised };
    let own = max_speed(o.kind);
    if own > 0.0 && moves_as != o.kind {
        max_speed(moves_as) / own
    } else {
        1.0
    }
}

/// The type a unit of type `kind` becomes when the player raises (`raise`) or lowers its Riot Shield (module docs):
/// its shield's raised or lowered type (design change 0198: "The type [...] switches **at the press**"). `None` for a
/// type without a shield, and when it is that type already: the shield goes that way already, and the toggle changes
/// nothing.
pub fn toggled_kind(kind: ObjectKind, raise: bool) -> Option<ObjectKind> {
    let def = def_of_kind(kind)?;
    let asked = if raise { def.raised } else { def.lowered };
    toggle::pressed([def.lowered, def.raised], kind, Press::To(asked))
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
/// takes it (module docs), but for a raise that waits for the unit's shot ([`ShieldMode::Deferred`]: it begins in the
/// first Modes step after the Cooldown's last tick), and the unit's speed follows.
pub(crate) fn step(objects: &mut BTreeMap<ObjectId, Object>, first_new: ObjectId) {
    for o in objects.range_mut(..first_new).map(|(_, o)| o) {
        let Some(shield) = o.riot_shield else {
            continue;
        };
        let Some(def) = def_of(o) else {
            continue;
        };
        let up = match mode(o) {
            None | Some(ShieldMode::Deferred) => continue,
            Some(ShieldMode::Raising { .. } | ShieldMode::Raised) => shield.up.saturating_add(1).min(def.toggle_ticks),
            Some(ShieldMode::Lowering { .. } | ShieldMode::Lowered) => {
                shield.up.min(def.toggle_ticks).saturating_sub(1)
            }
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
