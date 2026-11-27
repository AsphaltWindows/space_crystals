//! (M5-K) Target Lock in phase 5e — `factions/gdo/units/rocketman.md > Target Lock (targeted ability)`: "The player
//! orders Target Lock on one enemy unit. **Locking:** the Rocketman aims at the target continuously for LockOnDuration.
//! The target may move; the Rocketman turns to keep aiming at it. Locking succeeds only if the target stays within
//! LockRange and visible for the whole duration. **Locked:** once locked, the Rocketman fires at the locked target from
//! then on, and every missile it fires at that target is **homing**". Its order is `OrderKind::TargetLock` (the
//! Director's reading, by plan decision T76's Shroud: `orders::target_lock`), which sets the lock as it starts
//! ([`AttackState::target_lock`]); this phase does the rest ([`step`]), for every unit whose type has a Target Lock
//! (`UnitDef::target_lock`).
//!
//! - **Locking**: from the tick the order starts, each tick the lock holds counts toward LockOnDuration
//!   (`TargetLockDef::lock_on_ticks`), and the unit turns toward its target's centre whenever its base may turn
//!   (`super::base_is_free`). Its own weapon's sequence under way, if any, runs on to its end meanwhile, and it starts
//!   none: it engages nothing until it is locked ("fires at the locked target from then on"). Placeholder (the design
//!   says only "aims at the target continuously"): the count is the order's ticks, not the ticks it faces the target
//!   (the Rocketman turns 180° a frame: it faces its target from its first tick), and the lock is complete only once
//!   no shot of its own weapon is being fired.
//! - **Locked**: it engages its target and fires at it with its locked missiles ([`attrs`]): its weapon, homing
//!   (HeadDisjointed: `super::projectiles::Homing`), "from anywhere within LockRange" and "even inside the normal
//!   attack's MinRange", with its Damage, ProjectileSpeed and timings ("locked missiles use the same Damage and
//!   timings, homing").
//! - **The breaks** ("The lock (or the locking in progress) breaks when"):
//!   - "the target leaves LockRange": measured as a Range is (`systems/combat.md > Range Measurement`: edge to edge),
//!     the locked missiles' Range, LockRange;
//!   - "the Rocketman loses vision of the target": its owner's vision, as every attack's (`systems/combat.md >
//!     ValidTarget` 2);
//!   - "the Rocketman is given **any** order": every order's start, in phase 5a (`orders::update`), a new Target Lock's
//!     too, which begins its own;
//!   - "or the target dies": the removal path (`AttackState::forget`).
//!
//!   In each case the unit no longer engages its target (an aim at it is cancelled), and its order completes in the
//!   next phase 5a, its queued orders going on. A missile whose Firing has begun is launched as it began, homing,
//!   whatever became of the lock (`systems/combat.md > Firing`: not interruptible). The unit's own weapon fires at a
//!   location (DoublyDisjointed: the Rocketman's), so a Firing at an object is always a locked missile's ([`step`]).
//! - **No walk into range** (placeholder: the design gives the lock none): the unit locks from where it stands, so a
//!   target beyond LockRange or out of sight as the lock begins breaks it at once.
//! - **While the order runs** the unit stands, never yields (it has an order: `movement::yielding`) and fires only at
//!   its target: it doesn't auto-target. An order given without Shift breaks the lock, and "a Shift-queued order waits
//!   behind the lock and runs when it ends" (`factions/gdo/units/rocketman.md > Target Lock`, design change 0438,
//!   answering Q0171; plan decision T76).

use serde::{Deserialize, Serialize};

use super::phases::{self, Shooter, Volley};
use super::targeting::{TargetView, Targets};
use super::{AttackPhase, AttackState, base_is_free};
use crate::defs::{AttackAttrs, AttackType, ObjectKind, TargetLockDef};
use crate::events::SimEvent;
use crate::ids::ObjectId;

/// (M5-K) A Target Lock under way (module docs), `AttackState::target_lock`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TargetLock {
    /// The object locked onto: an enemy unit or structure (the gate: `factions/gdo/units/rocketman.md > Target Lock`,
    /// "can target structures (user, 2026-11-26) as well as units").
    pub target: ObjectId,
    /// Ticks spent locking so far, up to LockOnDuration.
    pub elapsed: u32,
    /// Locked: it fires its locked missiles at the target.
    pub locked: bool,
}

impl TargetLock {
    /// A lock onto `target` as it begins.
    pub fn new(target: ObjectId) -> Self {
        Self {
            target,
            elapsed: 0,
            locked: false,
        }
    }
}

impl AttackState {
    /// (M5-K) A Target Lock onto `target` begins, in place of any under way (module docs).
    pub fn begin_target_lock(&mut self, target: ObjectId) {
        self.target_lock = Some(TargetLock::new(target));
    }

    /// (M5-K) The Target Lock under way, if any, ends: an order of another kind starts, or the unit has none
    /// (`orders::update`). Whatever its order engages next is the order's own business.
    pub fn end_target_lock(&mut self) {
        self.target_lock = None;
    }
}

/// The locked missiles of a unit whose weapon is `weapon` and whose Target Lock is `def` (module docs): HeadDisjointed,
/// with LockRange for their Range and no MinRange, the weapon's in everything else.
pub fn attrs(weapon: &AttackAttrs, def: &TargetLockDef) -> AttackAttrs {
    AttackAttrs {
        attack_type: AttackType::HeadDisjointed,
        range: def.lock_range,
        min_range: 0.0,
        ..*weapon
    }
}

/// Whether the lock of a unit whose locked missiles' shooter is `locked` holds on `t` (module docs): its owner sees it,
/// within LockRange.
fn holds(locked: &Shooter, t: &TargetView) -> bool {
    locked.can_target(t) && locked.sees(t) && locked.reaches(t)
}

/// One tick of phase 5e for a unit of type `kind`, which has a Target Lock `def` (module docs), its own weapon's
/// shooter `normal` and its locked missiles' `locked`: the lock's tick, if one is under way (its breaks, its locking
/// and its turn), then the tick of its attack sequence (`phases::run`), with its locked missiles while it is locked or
/// one is being fired.
#[expect(
    clippy::too_many_arguments,
    reason = "phase 5e's run of one attack sequence, and the lock's numbers"
)]
pub(super) fn step(
    state: &mut AttackState,
    facing: &mut f32,
    kind: ObjectKind,
    def: &TargetLockDef,
    normal: &Shooter,
    locked: &Shooter,
    targets: &Targets,
    volley: &mut Volley,
    events: &mut Vec<SimEvent>,
) {
    if let Some(lock) = state.target_lock {
        match targets.get(lock.target).filter(|t| holds(locked, t)) {
            // It left LockRange or its owner's sight (one that died is forgotten already: `AttackState::forget`).
            None => {
                state.target_lock = None;
                if state.target == Some(lock.target) {
                    state.engage(None);
                }
            }
            Some(t) => {
                let mut lock = lock;
                if !lock.locked {
                    lock.elapsed = lock.elapsed.saturating_add(1);
                    let firing = matches!(
                        state.phase,
                        Some(AttackPhase::Firing { .. } | AttackPhase::FiringAt { .. })
                    );
                    lock.locked = lock.elapsed >= def.lock_on_ticks && !firing;
                    if !lock.locked && base_is_free(kind, Some(state)) {
                        locked.turn_toward(facing, t.pos);
                    }
                }
                state.target_lock = Some(lock);
                // Locked, it engages its target, and its Aiming turns it from now on.
                if lock.locked {
                    state.engage(Some(lock.target));
                }
            }
        }
    }
    let fires_locked =
        matches!(state.phase, Some(AttackPhase::Firing { .. })) || state.target_lock.is_some_and(|lock| lock.locked);
    let shooter = if fires_locked { locked } else { normal };
    phases::run(state, facing, shooter, targets, volley, events);
}

#[cfg(test)]
mod tests;
