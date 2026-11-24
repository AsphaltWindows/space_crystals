//! Fighting behaviours — `systems/controls/behaviors.md > AttackingObject`, `> BaseAutoTargeting`,
//! `> HoldingPosition`.
//!
//! [`close_in`] is the core of AttackingObject (steps 1 to 4): approach the target until the attack reaches it,
//! stand while it does, and go after it again when it leaves range. The Attack order ([`attack`]) adds the order's
//! ends (the target destroyed, lost from vision, or out of reach for good); idle units ([`idle`]) and AttackMove
//! (`travel`) run it on the enemies they acquire themselves; Hold Position ([`hold`]) never moves.

use crate::combat::reaches;
use crate::ids::ObjectId;
use crate::mover::{MoveGoal, MoveStatus};
use crate::units::gu;

use super::state::{AttackOrder, Auto, Pursuit};
use super::world::{Scan, Unit, View, World};
use super::{GIVE_UP_RETRY_TICKS, IDLE_LEASH_GU};

/// How an approach stands after [`close_in`].
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(super) enum Closing {
    /// The attack reaches the target: the unit stands, and phase 5e attacks.
    InRange,
    /// The unit is on its way to the target, or waits for its attack sequence to let it go.
    Approaching,
    /// The approach ended out of range: the move failed (`> AttackingObject > Failure`), or the unit is as near
    /// as it gets.
    GaveUp,
}

/// `> AttackingObject`, steps 1 to 4, one tick, against `target`, which the unit's owner sees.
/// - **In range** (`combat::reaches`: within Range and beyond MinRange, edge to edge): Locomotion = Stopping. The
///   unit stops its move; phase 5e aims and fires (`combat::phases`).
/// - **Out of range**: MovingToObject towards the target (`MoveGoal::Object`; phase 5c re-aims the path as the
///   target moves). The approach starts only while the attack sequence leaves the base free (`combat::may_move`):
///   infantry finishes Firing and Cooldown first.
///
/// `chasing` says whether the unit's current move is this approach; while it is, the mover's status is its outcome.
/// A move that failed, or ended as near as the unit gets to a target that hasn't moved since, gives the target up.
pub(super) fn close_in(unit: &mut Unit, chasing: &mut bool, target: &View, world: &World) -> Closing {
    let Some(attrs) = unit.attrs() else {
        return Closing::GaveUp;
    };
    if reaches(
        attrs,
        &unit.shape(),
        world.elevation(unit),
        &target.shape,
        target.elevation,
    ) {
        unit.halt();
        *chasing = false;
        return Closing::InRange;
    }
    if *chasing {
        match unit.mover.status {
            MoveStatus::Moving => return Closing::Approaching,
            // At the end of its path, and the target moved on in that very tick: after it again.
            MoveStatus::Arrived if target.pos != unit.mover.planned_target => {}
            MoveStatus::Arrived | MoveStatus::Failed | MoveStatus::TargetLost | MoveStatus::Idle => {
                *chasing = false;
                return Closing::GaveUp;
            }
        }
    }
    if unit.may_move() {
        // Reach 0: the approach ends when this function finds the target in range.
        let goal = MoveGoal::Object {
            id: target.id,
            reach: 0.0,
        };
        unit.mover.go_to(goal, None, world.tick);
        *chasing = true;
    }
    Closing::Approaching
}

/// `> AttackingObject`: one tick of an Attack order on `id`. Returns whether the behaviour is complete.
/// - Step 5: the target is destroyed (the removal path completes the order, `Sim::remove_objects`), or the unit
///   can't attack it at all (phase 5e dropped it, or the unit has no attack).
/// - Step 6 (design change 0034): the target is no longer visible to the unit's owner. The unit moves to its
///   **last seen position** (MovingToLocation) and the order completes there; it is never tracked through the fog,
///   even if it comes back into sight on the way. A target the owner never saw while the order ran (a queued Attack
///   whose target left sight before its turn) has no such position, and the order completes at once: a
///   placeholder, the design doesn't say.
/// - `> Failure`: the approach gave up ([`Closing::GaveUp`]).
pub(super) fn attack(unit: &mut Unit, id: ObjectId, state: &mut AttackOrder, world: &World) -> bool {
    if state.lost {
        return unit.mover.status != MoveStatus::Moving;
    }
    let engaged = unit.attack.as_ref().is_some_and(|attack| attack.target == Some(id));
    let Some(target) = world.get(id).filter(|_| engaged) else {
        if state.chasing {
            unit.mover.stop();
        }
        return true;
    };
    if !world.sees(unit.owner, target) {
        if !unit.may_move() {
            // Firing and Cooldown run on; the unit sets off when they are over.
            return false;
        }
        unit.engage(None);
        state.chasing = false;
        return match state.last_seen {
            Some(at) => {
                unit.mover.go_to(MoveGoal::Point(at), None, world.tick);
                state.lost = true;
                false
            }
            None => {
                unit.halt();
                true
            }
        };
    }
    state.last_seen = Some(target.pos);
    match close_in(unit, &mut state.chasing, target, world) {
        Closing::InRange | Closing::Approaching => false,
        Closing::GaveUp => {
            unit.engage(None);
            true
        }
    }
}

/// `> HoldingPosition`, `> BaseAutoTargeting > Active during HoldPosition`: one tick. Locomotion stays Stationary:
/// the unit never moves. It engages the best ValidTarget within weapon range ([`World::pick`]) and disengages at
/// once when that target leaves range (or is no longer valid). A non-turret unit that can't turn in place engages
/// only what it already faces. Turrets have no state of their own yet, so a turret unit holds like the others.
pub(super) fn hold(unit: &mut Unit, target: &mut Option<ObjectId>, world: &World) {
    let Some(attrs) = unit.attrs() else {
        return;
    };
    if let Some(id) = *target {
        let keeps = world
            .get(id)
            .is_some_and(|t| world.valid_target(unit, attrs, t) && world.within(unit, attrs, Scan::Weapon, t));
        if !keeps {
            unit.engage(None);
            *target = None;
        }
    }
    if target.is_none() {
        let facing_only = unit.unit_def().is_some_and(|def| {
            let base = def.base_def();
            !base.has_turret && !base.can_turn_in_place
        });
        *target = world.pick(unit, Scan::Weapon, None, facing_only);
        if target.is_some() {
            unit.engage(*target);
        }
    }
}

/// `> BaseAutoTargeting > Active during Idle`: one tick of a unit with no order at all.
/// - **Watching**: it scans for ValidTarget enemies within SightRange and acquires the best
///   ([`World::pick`]); where it stands then is its IdleOrigin. A move or a target left over from an order that
///   ended (its target died, say) ends here too.
/// - **Engaging**: AttackingObject on the acquired enemy ([`close_in`]). Farther than IdleLeashDistance from the
///   IdleOrigin, it disengages and returns there (MovingToLocation). If the target is destroyed or leaves SightRange
///   (or is no longer a ValidTarget), the unit is idle again where it stands, with a new IdleOrigin next time. A
///   target it can't get to is left alone for [`GIVE_UP_RETRY_TICKS`], so the unit doesn't set off for it tick after
///   tick.
/// - **Returning**: no scanning (`> Not active during: Move`); idle again once the move ends.
pub(super) fn idle(unit: &mut Unit, auto: &mut Auto, world: &World) {
    // Returning → Watching → Engaging at most: each pass moves on by one.
    for _ in 0..3 {
        match *auto {
            Auto::Watching { ignored } => {
                unit.halt();
                if unit.attack.as_ref().is_some_and(|attack| attack.target.is_some()) {
                    unit.engage(None);
                }
                let ignored = ignored.filter(|&(_, until)| world.tick < until);
                *auto = Auto::Watching { ignored };
                let skip = ignored.map(|(id, _)| id);
                let Some(target) = world.pick(unit, Scan::Sight, skip, false) else {
                    return;
                };
                unit.engage(Some(target));
                *auto = Auto::Engaging {
                    pursuit: Pursuit { target, chasing: false },
                    origin: unit.pos,
                };
            }
            Auto::Engaging { mut pursuit, origin } => {
                let target = unit.attrs().and_then(|attrs| {
                    world
                        .get(pursuit.target)
                        .filter(|t| world.valid_target(unit, attrs, t) && world.within(unit, attrs, Scan::Sight, t))
                });
                let Some(target) = target else {
                    unit.engage(None);
                    if pursuit.chasing {
                        unit.mover.stop();
                    }
                    *auto = Auto::default();
                    continue;
                };
                if unit.pos.distance(origin) > gu(IDLE_LEASH_GU) {
                    if unit.may_move() {
                        unit.engage(None);
                        unit.mover.go_to(MoveGoal::Point(origin), None, world.tick);
                        *auto = Auto::Returning;
                    }
                    return;
                }
                *auto = match close_in(unit, &mut pursuit.chasing, target, world) {
                    Closing::InRange | Closing::Approaching => Auto::Engaging { pursuit, origin },
                    Closing::GaveUp => {
                        unit.engage(None);
                        Auto::Watching {
                            ignored: Some((pursuit.target, world.tick.saturating_add(GIVE_UP_RETRY_TICKS))),
                        }
                    }
                };
                return;
            }
            Auto::Returning => {
                if unit.mover.status == MoveStatus::Moving {
                    return;
                }
                *auto = Auto::default();
            }
        }
    }
}
