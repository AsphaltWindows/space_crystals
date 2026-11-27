//! Combat — `systems/combat.md`. Phase 5e (`docs/movement.md > 4`): attack phases, the list of hits, damage and
//! deaths, shooting from the positions phase 5d committed.
//!
//! - **State**: a unit that can attack (`UnitDef::attack`) has an [`AttackState`] (`Object::attack`): the target
//!   its behaviour engages, and the phase of its attack sequence with the ticks spent in it (the design's
//!   BaseAttackChannel, `systems/controls/behaviors.md`). It is hashed and serialized like all state.
//! - **Phases** (`systems/combat.md > AttackPhase`; `phases`): Aiming, Firing, Cooldown and Reloading, each
//!   lasting exactly its duration in ticks. A sequence starts when no sequence runs and the engaged target is valid
//!   and in range (`targeting`); Aiming also lasts until the unit faces the target
//!   (`tuning::combat::FACING_TOLERANCE`). It turns the unit by what phase 5d left of its TurnRate: one turn budget per
//!   tick (`movement::Turns`).
//! - **Hits** (`damage`): an attack's effect lands at the end of its Firing phase, as a [`Hit`] on this tick's
//!   list. The list is applied in one place once every unit has acted, so two hits in one tick both count, whatever
//!   the order of their attackers (`ARCHITECTURE.md > Tick pipeline`).
//! - **Shields** (`systems/combat.md > Shields`, `crate::shields`; plan decision T18): a hit on a Colonists object
//!   takes its Shields first, at full value, and only the overflow goes on to the HP ([`resolve`]), as the list is
//!   applied: the second hit of a tick meets the Shields the first one left.
//! - **Deaths**: an object at 0 HP or less dies (`SimEvent::Died`). `step` returns the dead, and `Sim::step`
//!   removes them through the one removal path (`Sim::remove_objects`), which clears every reference to them.
//! - **Queries**: [`may_move`] and [`may_turn`] say whether a unit's attack sequence lets it move or turn this tick.
//!   The movement step holds a unit that may not (phase 5d), and the behaviours start a chase only when it may
//!   (`orders::engage`).
//! - **Vision** (`systems/combat.md > ValidTarget` 2): a sequence starts, and Aiming goes on, only while the unit's
//!   owner sees the target, by the vision phase 5f left at the end of the last tick (`vision`), an attacker revealed
//!   to the owner included. What a unit does about a target out of range or out of sight is its behaviour's
//!   (`orders::engage`: it closes in, or its Attack completes and it stops where it is, design change 0200).
//! - **Attacks from outside vision** (`core/vision.md`, design change 0177): each hit reveals its attacker to the
//!   owner of the object hit, if that player didn't see it (`vision::reveal`), once the tick's hits are applied.
//! - **Range** (`systems/combat.md > Range Measurement`, design change 0031): edge to edge, the gap between the
//!   attacker's shape and the target's (`shape`); [`reaches`] is the one check.
//!
//! - **Riot Shield** (`factions/gdo/units/peacekeeper.md > Riot Shield`, `riot_shield`): only a Peacekeeper whose
//!   shield is all the way down attacks (`riot_shield::allows_attack`): one whose shield is up or on its way (design
//!   change 0171), or whose raise waits for its shot (design change 0198), doesn't. Its behaviours engage nothing
//!   then; whatever is still engaged is dropped here, which cancels an aim, while Firing and Cooldown run on, and a
//!   reload keeps counting. Its shield all the way up halves the hits against it before armour
//!   ([`pre_armor_multiplier`]).
//!
//! - **Weapon Conversion** (`factions/colonists/units/weapon_conversion.md`, `crate::conversion`): a unit fights
//!   with the weapon of the form it has ([`weapon`]), which isn't its type while a conversion is pending (design change
//!   0198: the type switches at the press, the weapon when the conversion ends). A unit converting its weapon neither
//!   moves nor turns ([`may_move`], [`may_turn`]) and attacks nothing: this phase skips it, and its attack sequence,
//!   interrupted when the conversion started, begins again with the new weapon.
//!
//! - **Projectiles and location targets** (slice M4-O; `systems/combat.md > DoublyDisjointed`, plan decisions T45 to
//!   T48): a DoublyDisjointed attack locks where its target is at the end of Aiming, launches a projectile there at
//!   the end of Firing ([`projectiles`]), and goes on into Cooldown and Reloading, its base free ("The unit is free
//!   after the projectile spawns"). The projectile lands after its flight, in this phase, ahead of the tick's new hits.
//!   The attack may target a location as well as an object ([`AttackState::location`]: an Attack Ground's, plan
//!   decision T47): aimed at, locked and fired at the same way, again and again while the location stays engaged.
//! - **TailDisjointed attacks** (slice M5-O3; `systems/combat.md > TailDisjointed`, the Cults' Gunner's, design change
//!   0304): aimed at an object or a location and locked as a DoublyDisjointed attack is, with no projectile. At the end
//!   of Firing the effect strikes the locked spot (`phases`): whatever stands there then, whoever owns it, so a target
//!   that has left it is missed (`SimEvent::Missed`). The base stays held through Firing and Cooldown, as for a
//!   FullyConnected attack.
//! - **Area damage** (`systems/combat.md > AoEDamage`, plan decision T46): a landing projectile's blast hits whatever
//!   stands in its circle, whoever owns it, by the exact overlap of each object's shape ([`area`]), the one path of
//!   every area attack. Each object hit takes its share as a [`Hit`] on the tick's list; each one damaged reveals the
//!   shooter, if it is still alive, to its owner: when the damage lands, not at the throw, so a dodged projectile
//!   reveals nothing (design change 0293, answering Q0083 point 2; `core/vision.md > Attacks from outside vision`).
//!
//! - **Structures that attack** (plan decision T49, slice M4-S1; [`structures`]): the Laser Turret
//!   (`factions/syndicate/structures/laser_turret.md`), whose attack is its def's (`StructureDef::attack`). It runs the
//!   same phases and deals the same damage as a unit, from its footprint, with no turn rate (it fires in any
//!   direction) and no orders: it picks its own target in this phase, once it is complete (`Object::is_operational`).
//! - **Units' turrets** (slice M5-U1, plan decision T80; [`turret`]): a unit with TurretAttributes attacks from its
//!   turret, which has a facing of its own on the base's and engages its LockedTarget ([`AttackState::turret`]). In
//!   this phase it turns toward it first, at its own TurnRate and within its arc, and its sequence runs as a base's,
//!   at a target within the arc and once the turret faces it; its base is free to move and turn all the while.
//! - **Occupants that fire** (slice M5-N2, plan decision T72; `occupants`): the units inside a Garrison
//!   (`StructureDef::occupants_fire`, `factions/cults/structures/garrison.md`) fire from it, held off the map, right
//!   after every object's own attack: each its own sequence, from the footprint, with its target picked as a
//!   structure's and its weapon or its stand-in's, its Range bonus added.
//! - **Homing projectiles** (slice M5-K; [`projectiles::Homing`]): a HeadDisjointed attack (`systems/combat.md >
//!   HeadDisjointed`) launches at the end of its Firing a projectile that tracks its target and lands on it, wherever
//!   it is: "The projectile cannot miss".
//! - **Target Lock** (slice M5-K; [`target_lock`], `factions/gdo/units/rocketman.md > Target Lock`): the Rocketman's
//!   order locks onto one target ([`AttackState::target_lock`]). In this phase the lock aims at it for LockOnDuration,
//!   breaks when the target leaves LockRange or its owner's vision, and once locked fires homing missiles at it,
//!   from anywhere within LockRange.

mod area;
mod damage;
mod hash;
mod occupants;
mod phases;
pub mod projectiles;
mod step;
mod structures;
pub mod target_lock;
mod targeting;
#[cfg(test)]
pub(crate) mod testing;
pub(crate) mod turret;

use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::augmentation::{self, Research};
use crate::conversion;
use crate::defs::{AttackAttrs, AttackType, ObjectKind, TargetDomain};
use crate::ids::ObjectId;
use crate::object::Object;

pub use area::{hits_domain, overlap};
pub use damage::{Hit, Taken, directional_multiplier, hit_damage, pre_armor_multiplier, resolve};
pub(crate) use occupants::Threats;
pub use projectiles::{Projectile, flight_ticks};
pub(crate) use step::step;
pub use target_lock::TargetLock;
pub use targeting::{
    can_attack_now, can_target_ground, domain_allows, inside_min_range, max_range, object_box, object_domain, reaches,
    reaches_point, takes_attack_orders, within_range,
};
pub use turret::{TurretState, turret_facing};

/// `systems/combat.md > AttackPhase`, one value of the design's BaseAttackChannel
/// (`systems/controls/action_channels.md > BaseAttackChannel`). The phases run in this order; after Reloading the
/// sequence is over. A disjointed attack's Aiming at a location and its Firing at the location it locked have values of
/// their own (slice M4-O), tagged after the others in the hash, so that the states of attacks without them hash as they
/// did.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum AttackPhase {
    /// `systems/combat.md > Aiming` (`systems/controls/action_channels.md > BaseAiming`): the unit turns to face
    /// `target`, at most its TurnRate per tick. Interruptible: a different command, or a target that is no longer valid
    /// (gone, or out of range), cancels the sequence before anything is fired.
    Aiming { target: ObjectId },
    /// `systems/combat.md > Firing` (`systems/controls/action_channels.md > BaseFiring`): not interruptible. The
    /// attack's effect lands on `target` at the end of the phase. `None`: the target was destroyed meanwhile, and the
    /// attack hits nothing (a placeholder reading: the design doesn't say).
    Firing { target: Option<ObjectId> },
    /// `systems/combat.md > Cooldown`: not interruptible; the unit is still unresponsive.
    Cooldown,
    /// `systems/combat.md > Reloading`: interruptible: the unit is free to do other things, but its next attack waits
    /// for the reload to end, which keeps counting (a placeholder reading: interrupting it doesn't reset it).
    Reloading,
    /// `systems/combat.md > Aiming` at the ground location `at` (`systems/combat.md > LocationTarget`; an Attack Ground's,
    /// [`AttackState::location`]): the unit turns to face it. Interruptible, as Aiming at an object: a different command
    /// cancels it, and so does the location leaving the unit's range.
    AimingAt { at: Vec2 },
    /// `systems/combat.md > Firing` of a disjointed attack: not interruptible. `at` is the location locked at the end
    /// of Aiming: where the target was then, or the location aimed at. At the end of the phase a DoublyDisjointed
    /// attack's projectile launches there (`systems/combat.md > DoublyDisjointed`, `projectiles`), and a TailDisjointed
    /// attack strikes whatever stands there (`systems/combat.md > TailDisjointed`, slice M5-O3). `target`: the object
    /// aimed at, while it is in the game; none for a location.
    FiringAt { at: Vec2, target: Option<ObjectId> },
}

impl AttackPhase {
    /// `systems/combat.md > AttackPhase > Interruptible`.
    pub fn interruptible(self) -> bool {
        match self {
            AttackPhase::Aiming { .. } | AttackPhase::AimingAt { .. } | AttackPhase::Reloading => true,
            AttackPhase::Firing { .. } | AttackPhase::FiringAt { .. } | AttackPhase::Cooldown => false,
        }
    }
}

/// Whether an attack phase that can't be interrupted is under way on `o`: Firing or Cooldown (`systems/combat.md >
/// AttackPhase > Interruptible`). A mode toggled now waits for it: a Weapon Conversion and a raise of the Riot Shield
/// (design changes 0153 and 0172, `sim/modes.rs`). Idle, moving, Aiming and Reloading are all fine.
pub fn mid_attack(o: &Object) -> bool {
    (o.attack.as_ref())
        .and_then(|attack| attack.phase)
        .is_some_and(|phase| !phase.interruptible())
}

/// The AttackAttributes of the weapon `o` fights with now (`systems/combat.md > AttackAttributes`): its form's
/// (`conversion::form`), which is its type's but while a Legionnaire's change of form is pending (design change 0198:
/// its type is the form it works toward, its form the one whose weapon it has; `ARCHITECTURE.md > Object model > Type
/// and form`), with (M5-N1a) its Augmentations' effects at its owner's levels in `research`
/// (`augmentation::augmented`: an unarmed unit's Damage and FiringDuration), a copy. The one source of a unit's attack
/// attributes for what it does in the world: phase 5e and the behaviours (`orders::world::Unit::attrs`), which read it
/// by reference ([`weapon_ref`]). `None` for an object without an attack.
#[inline]
pub fn weapon(o: &Object, research: Research) -> Option<AttackAttrs> {
    weapon_ref(o, research, &mut None).copied()
}

/// (M5-N1a) [`weapon`] by reference: the weapon in hand without an Augmentation, else made in `slot`. The read of the
/// behaviours' and phase 5e's loops over every unit, with nothing copied for a unit without one (inlined into them, as
/// the other accessors are: `augmentation/stats.rs`).
#[inline]
pub fn weapon_ref<'a>(o: &Object, research: Research, slot: &'a mut Option<AttackAttrs>) -> Option<&'a AttackAttrs> {
    let attrs = weapon_in_hand(o)?;
    if o.augmentation.is_none() {
        return Some(attrs);
    }
    Some(slot.insert(augmentation::augmented(o, attrs, research)))
}

/// The weapon `o` holds as its form's type has it, before its Augmentations ([`weapon`]): for what they leave as it is,
/// its TargetDomain and its ranges (the threat it is to the others' scans, [`can_attack_now`]).
fn weapon_in_hand(o: &Object) -> Option<&'static AttackAttrs> {
    weapon_of(conversion::form(o))
}

/// The AttackAttributes of a unit of form `form` ([`weapon`]), or of a structure of type `form` that attacks
/// (`StructureDef::attack`, plan decision T49).
pub(crate) fn weapon_of(form: ObjectKind) -> Option<&'static AttackAttrs> {
    let def = form.def();
    match &def.unit {
        Some(unit) => unit.attack.as_ref(),
        None => def.structure.as_ref()?.attack.as_ref(),
    }
}

/// The domain `o`'s own attack threatens: its weapon's TargetDomain ([`weapon`]), if it has an attack it can use: a unit
/// that attacks, a structure that attacks once complete (`Object::is_operational`: a site does nothing but cancel). What
/// the auto-targeting scans rank first (`systems/controls/behaviors.md > BaseAutoTargeting > Target Selection Priority`:
/// "threatening targets first"): the behaviours' (`orders::world`), and a structure's own pick ([`structures`]), whose
/// snapshots add (M5-S2b) what a building's units inside threaten ([`Threats`], design change 0377).
pub(crate) fn threat(o: &Object) -> Option<TargetDomain> {
    weapon_in_hand(o)
        .filter(|_| o.attack.is_some() && o.is_operational())
        .map(|attack| attack.target_domain)
}

/// A unit's attack capability (`systems/combat.md > AttackAttributes`: only units that can attack have one).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AttackState {
    /// The object the unit's behaviour engages (`systems/combat.md > AttackTarget > UnitTarget`), set by its order
    /// (`orders::update`); `None`: it engages nothing.
    pub target: Option<ObjectId>,
    /// The phase of the running attack sequence; `None` while none runs and the weapon is ready.
    pub phase: Option<AttackPhase>,
    /// Ticks spent in `phase` so far.
    pub elapsed: u32,
    /// (M4-O) The ground location the unit's behaviour engages (`systems/combat.md > AttackTarget > LocationTarget`,
    /// plan decision T47): an Attack Ground's (`orders::attack_ground`), for as long as the order runs: it keeps
    /// firing at the location until another order (design change 0293, answering Q0083 point 3). At most one of it and
    /// [`AttackState::target`] is set. Left out of files while unset, and out of the hash, so that the states saved
    /// before it existed load and hash as they did.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<Vec2>,
    /// (M5-U1) A unit's turret, its facing and its LockedTarget ([`turret`], plan decision T80): for a unit with
    /// TurretAttributes, from the first time its turret locks or turns. Its attack sequence runs at the LockedTarget,
    /// not at [`AttackState::target`], which stays what its base's behaviour engages. Left out of files while unset,
    /// and out of the hash, as the location.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turret: Option<TurretState>,
    /// (M5-K) The Target Lock under way, locking or locked ([`target_lock`], `factions/gdo/units/rocketman.md > Target
    /// Lock`): set as its order starts (`orders::target_lock`), until it breaks. Left out of files while unset, and
    /// out of the hash, as the location.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_lock: Option<TargetLock>,
}

impl AttackState {
    /// The unit's behaviour engages `target`, or nothing, and no location. `systems/combat.md > Aiming`: a different
    /// command interrupts an attack being aimed at another target, or at a location; once Firing has begun, the
    /// sequence runs on (`systems/combat.md > Firing`, `systems/combat.md > Cooldown`), and a reload keeps counting
    /// (`systems/combat.md > Reloading`).
    pub fn engage(&mut self, target: Option<ObjectId>) {
        let interrupted = match self.phase {
            // (M5-U1) A turret aims at its LockedTarget, which the base's target leaves as it is (`AttackState::lock`).
            Some(AttackPhase::Aiming { target: aimed }) => self.turret.is_none() && Some(aimed) != target,
            Some(AttackPhase::AimingAt { .. }) => true,
            _ => false,
        };
        if interrupted {
            self.cancel();
        }
        self.target = target;
        self.location = None;
    }

    /// (M4-O) The unit's behaviour engages the ground location `at` (`systems/controls/command_behaviors.md >
    /// AttackingLocation`), and no object: as [`AttackState::engage`], an aim at anything else is interrupted.
    pub fn engage_ground(&mut self, at: Vec2) {
        let interrupted = match self.phase {
            Some(AttackPhase::AimingAt { at: aimed }) => aimed != at,
            Some(AttackPhase::Aiming { .. }) => true,
            _ => false,
        };
        if interrupted {
            self.cancel();
        }
        self.target = None;
        self.location = Some(at);
    }

    /// The removal path (`Sim::remove_objects`): the objects `gone` (sorted by id) left the game. The unit no longer
    /// engages them, nor does its turret lock them, nor (M5-K) its Target Lock; an attack aimed at one is cancelled,
    /// and one being fired at one hits nothing.
    pub fn forget(&mut self, gone: &[ObjectId]) {
        let is_gone = |id: ObjectId| gone.binary_search(&id).is_ok();
        if self.target.is_some_and(is_gone) {
            self.target = None;
        }
        if let Some(turret) = &mut self.turret
            && turret.locked.is_some_and(is_gone)
        {
            turret.locked = None;
        }
        // (M5-K) "The lock (or the locking in progress) breaks when [...] the target dies".
        if self.target_lock.is_some_and(|lock| is_gone(lock.target)) {
            self.target_lock = None;
        }
        match self.phase {
            Some(AttackPhase::Aiming { target }) if is_gone(target) => self.cancel(),
            Some(AttackPhase::Firing { target: Some(target) }) if is_gone(target) => {
                self.phase = Some(AttackPhase::Firing { target: None });
            }
            // The shot still goes to the location locked, a projectile or a TailDisjointed strike
            // (`systems/combat.md > DoublyDisjointed`, `systems/combat.md > TailDisjointed`).
            Some(AttackPhase::FiringAt {
                at,
                target: Some(target),
            }) if is_gone(target) => {
                self.phase = Some(AttackPhase::FiringAt { at, target: None });
            }
            _ => {}
        }
    }

    /// The sequence ends without an attack.
    fn cancel(&mut self) {
        self.phase = None;
        self.elapsed = 0;
    }
}

/// `systems/combat.md > AttackPhase`, `systems/combat.md > UnitBaseSource`: whether `o`'s attack sequence lets its base
/// move this tick. A unit that attacks from its base can't move while Aiming, Firing or in Cooldown; it may while
/// Reloading and when no sequence runs. A turret unit's base always may (`systems/combat.md > TurretSource`,
/// `TurretSourceBaseActions`), and so may every object without an attack. The movement step (5d) holds a unit that may
/// not, keeping its goal; the behaviours (5a) don't start a chase then. Before 5e it reads the phase the last tick
/// left, which is the one this tick runs in; a Stop or a new target in 5a has already cancelled an aim. A unit that
/// converts its weapon holds still whatever its attack (`conversion`: "The unit converts in place").
pub fn may_move(o: &Object) -> bool {
    base_is_free(o.kind, o.attack.as_ref()) && !conversion::is_converting(o)
}

/// `systems/combat.md > AttackPhase > UnitBaseSourceActions`: whether anything but the attack may turn `o`'s base
/// this tick. Not while Aiming, when the attack turns the base to face its target
/// (`systems/controls/action_channels.md > BaseAiming`: the orientation is overridden), nor in Firing and Cooldown,
/// when the orientation is locked (`systems/controls/action_channels.md > BaseFiring`,
/// `systems/controls/action_channels.md > BaseCooldown`). Turret units' bases are always free (`systems/combat.md >
/// TurretSource`). Nor while it converts its weapon (`conversion`).
pub fn may_turn(o: &Object) -> bool {
    base_is_free(o.kind, o.attack.as_ref()) && !conversion::is_converting(o)
}

/// Both rules come to the same in the design's phase table: the base is either free or held by the attack. For a
/// unit of `kind` whose attack is in `state` (the behaviours hold a unit's parts, not the object).
///
/// (M4-O) A DoublyDisjointed attack frees the base once its projectile is away: `systems/combat.md >
/// DoublyDisjointed`, "The unit is free after the projectile spawns", which plan decision T48 reads as the unit going
/// on into Cooldown and Reload free to move. Its Cooldown still can't be interrupted.
pub(crate) fn base_is_free(kind: ObjectKind, state: Option<&AttackState>) -> bool {
    let Some(state) = state else {
        return true;
    };
    let unit = kind.def().unit.as_ref();
    // Design change 0154: the attack source is the turret if the unit has one (its TurretAttributes), else the base.
    let turret = unit.is_some_and(|unit| unit.has_turret());
    let launched = state.phase == Some(AttackPhase::Cooldown)
        && unit
            .and_then(|unit| unit.attack.as_ref())
            .is_some_and(|attack| matches!(attack.attack_type, AttackType::DoublyDisjointed));
    turret || launched || matches!(state.phase, None | Some(AttackPhase::Reloading))
}

#[cfg(test)]
mod tests;
