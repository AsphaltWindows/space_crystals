//! (M5-K) The Target Lock order's behaviour (`OrderKind::TargetLock`), `factions/gdo/units/rocketman.md > Target Lock
//! (targeted ability)`. The lock is phase 5e's (`crate::combat::target_lock`: its locking, its breaks and its homing
//! missiles); the order holds the unit while it lasts.
//!
//! - **The start** (`super::run::start`) begins the lock, in place of any under way, for a unit whose type has a
//!   Target Lock (`UnitDef::target_lock`), and stops the unit, as every order's but a Move to a point does. Every other
//!   order's start ends the lock ("the Rocketman is given **any** order [...] **without Shift**"; one queued behind the
//!   lock starts once it has ended).
//! - **It stands**: nothing gives it a goal while the lock lasts, and with an order it never yields
//!   (`movement::yielding::can_yield`: "no active command"). It engages only its target, once locked, and doesn't
//!   auto-target (`super::engage::idle` runs only without an order).
//! - **It completes** in the first tick the lock is gone: broken in the last phase 5e (the target left LockRange or its
//!   owner's sight), or forgotten with its target (the removal path, which also ends the order itself). The orders
//!   queued behind it start then ("a Shift-queued order waits behind the lock and runs when it ends", design change
//!   0438, answering Q0171).

use super::world::Unit;
use crate::ids::ObjectId;

/// The start of a Target Lock onto `target` (module docs): the lock begins, for a unit whose type has one.
pub(super) fn start(unit: &mut Unit, target: ObjectId) {
    if unit.unit_def().is_some_and(|def| def.target_lock.is_some())
        && let Some(attack) = &mut unit.attack
    {
        attack.begin_target_lock(target);
    }
}

/// One tick of a Target Lock order (module docs): whether it is complete, its lock gone. A unit whose type has no
/// Target Lock (only a save edited by hand gives it the order) completes it at once, having begun none.
pub(super) fn run(unit: &Unit) -> bool {
    !(unit.attack.as_deref()).is_some_and(|attack| attack.target_lock.is_some())
}
