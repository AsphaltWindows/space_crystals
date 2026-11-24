//! Fighting behaviours — `systems/controls/behaviors.md > AttackingObject`, `> BaseAutoTargeting`,
//! `> HoldingPosition`.
//!
//! [`close_in`] is the core of AttackingObject (steps 1 to 4): approach the target until the attack reaches it,
//! stand while it does, and go after it again when it leaves range. The Attack order ([`attack`]) adds the order's
//! ends (the target destroyed, lost from vision, or out of reach for good), its walk to a target its owner only
//! remembers (design change 0139) and its wait behind a raised Riot Shield (design change 0150); idle units
//! ([`idle`]) and AttackMove (`travel`) run it on the enemies they acquire themselves; Hold Position ([`hold`])
//! never moves.

use glam::Vec2;

use crate::combat::{inside_min_range, max_range, reaches};
use crate::defs::AttackAttrs;
use crate::ids::ObjectId;
use crate::math;
use crate::mover::{MoveGoal, MoveStatus};
use crate::units::{SU_PER_GU, gu};

use super::state::{AttackOrder, Auto, Pursuit};
use super::travel::{approach, walk_to_remembered};
use super::world::{Known, Scan, Unit, View, World};
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

/// `> AttackingObject`: one tick of an Attack order on `id`, as the unit's owner knows the target
/// ([`World::known`]). Returns whether the behaviour is complete.
/// - Step 5: the target is destroyed (the removal path completes the order, `Sim::remove_objects`), or the unit
///   can't attack it at all (phase 5e dropped it, or the unit has no attack).
/// - Step 6 (design change 0034): the target is no longer visible to the unit's owner, and isn't remembered (a
///   unit). The unit moves to its **last seen position** (MovingToLocation) and the order completes there; it is
///   never tracked through the fog, even if it comes back into sight on the way. A target the owner never saw while
///   the order ran (a queued Attack whose target left sight before its turn) has no such position, and the order
///   completes at once: a placeholder, the design doesn't say.
/// - **A remembered target** (design change 0139, `core/vision.md > Explored > Targeting remembered objects`): the
///   owner doesn't see it, but remembers it (a structure). "The order goes to the object's last-known position"
///   ([`walk_to_remembered`]), with nothing engaged, so that phase 5e reads nothing of it: the walk is the same
///   whether or not the target still stands. Once the owner sees it, the unit engages it again and attacks; once the
///   owner sees its place without it, the order ends where the unit is ("the unit stops there and takes its next
///   queued order or idles": `Sim::follow_targets`). A walk that ends without the owner seeing it (no way there)
///   ends the order.
/// - **The Riot Shield** (design change 0150: "Explicit Attack → follows the target while shielded, resumes
///   attacking when lowered"): while the unit may not attack, the order is on hold. The unit follows the target as a
///   Move to it would ([`approach`]: it follows a unit, and goes up to anything else and waits there), and the
///   order never completes for it; once the shield is down the unit engages the target again. Leaving sight ends it
///   as above.
/// - **A target inside the unit's MinRange** (design change 0182): the unit backs off, or the order fails
///   ([`too_close`]).
/// - `> Failure`: the approach gave up ([`Closing::GaveUp`]).
pub(super) fn attack(unit: &mut Unit, id: ObjectId, state: &mut AttackOrder, world: &World) -> bool {
    if state.lost {
        return unit.mover.status != MoveStatus::Moving;
    }
    let known = world.known(unit.owner, id);
    if let Known::Remembered { position, .. } = known {
        unit.engage(None);
        state.chasing = false;
        state.backing_off = None;
        state.on_hold = None;
        return walk_to_remembered(unit, &mut state.remembered, position, world);
    }
    if let Known::Seen(target) = known
        && !unit.may_attack
    {
        state.remembered = false;
        state.chasing = false;
        state.backing_off = None;
        state.last_seen = Some(target.pos);
        let follow = state.on_hold.get_or_insert_with(Default::default);
        approach(unit, id, follow, world);
        return false;
    }
    let target = match known {
        Known::Seen(target) => {
            // Back from the walk to a remembered target, or from behind the shield: the unit engages its target
            // again (the move it was on isn't the approach of `close_in`).
            if std::mem::take(&mut state.remembered) | state.on_hold.take().is_some() {
                unit.engage(Some(id));
            }
            if !unit.attack.as_ref().is_some_and(|attack| attack.target == Some(id)) {
                if state.chasing || state.backing_off.is_some() {
                    unit.mover.stop();
                }
                return true;
            }
            target
        }
        Known::Gone if state.on_hold.is_some() || unit.attack.as_ref().is_some_and(|a| a.target == Some(id)) => {
            if !unit.may_move() {
                // Firing and Cooldown run on; the unit sets off when they are over.
                return false;
            }
            unit.engage(None);
            state.chasing = false;
            state.backing_off = None;
            state.on_hold = None;
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
        Known::Gone | Known::Remembered { .. } => {
            if state.chasing || state.backing_off.is_some() {
                unit.mover.stop();
            }
            return true;
        }
    };
    state.last_seen = Some(target.pos);
    if let Some(attrs) = unit.attrs()
        && inside_min_range(attrs, unit.shape().gap(&target.shape))
    {
        return too_close(unit, attrs, state, target, world);
    }
    state.backing_off = None;
    match close_in(unit, &mut state.chasing, target, world) {
        Closing::InRange | Closing::Approaching => false,
        Closing::GaveUp => {
            unit.engage(None);
            true
        }
    }
}

/// `> AttackingObject > Target inside MinRange` (design change 0182): one tick of an Attack order whose target is
/// inside the unit's MinRange, too close to hit. Returns whether the order is complete.
/// - **A unit that can move backs off** "until the target is outside MinRange, then attacks": it walks straight away
///   from the target ([`back_off_spot`]) once its attack sequence lets it move (`combat::may_move`), and [`attack`]
///   goes on as ever once the target is outside MinRange ([`close_in`] stops it in range). A back-off that ended
///   with the target still inside starts again from where the unit stands if the target moved meanwhile (it follows
///   the unit); if it didn't, the unit is as far as it gets from it, and gives the target up (`> Failure`): backed
///   against a cliff, say.
/// - **A unit that can't move** (`Mover::can_move`: a deployed Artillery Rig, when there is one; a unit converting
///   its weapon too, while it does, though none has a MinRange) "can't carry it out: the order fails with feedback
///   ("Target too close"; same channel as other refusals)". Behaviours don't report: it marks the order
///   ([`AttackOrder::too_close`]), and the step right after this phase ends it with a `SimEvent::TargetTooClose`
///   (`Sim::step_too_close`).
fn too_close(unit: &mut Unit, attrs: &AttackAttrs, state: &mut AttackOrder, target: &View, world: &World) -> bool {
    let max_speed = unit.unit_def().map_or(0.0, |def| def.movement.max_speed());
    if !unit.mover.can_move(max_speed) {
        unit.engage(None);
        unit.halt();
        state.chasing = false;
        state.too_close = true;
        return false;
    }
    if std::mem::take(&mut state.chasing) {
        // It came too close while the unit went for it.
        unit.mover.stop();
    }
    if let Some(from) = state.backing_off {
        if unit.mover.status == MoveStatus::Moving {
            return false;
        }
        if target.pos == from {
            state.backing_off = None;
            unit.engage(None);
            return true;
        }
    }
    if unit.may_move() {
        let spot = back_off_spot(unit, attrs, target, world);
        unit.mover.go_to(MoveGoal::Point(spot), None, world.tick);
        state.backing_off = Some(target.pos);
    }
    false
}

/// Where a unit backs off to from `target`, inside its MinRange ([`too_close`]): straight away from it (straight back
/// from one right on top of it), as far as to bring the gap between them to the middle of the band from MinRange to
/// Range, so that whatever the shapes make of the distance walked the unit passes through the band, where [`close_in`]
/// stops it. Within the map; the path ends as near as the unit gets (`> MovingToLocation > Unreachable target
/// location`).
fn back_off_spot(unit: &Unit, attrs: &AttackAttrs, target: &View, world: &World) -> Vec2 {
    let gap = unit.shape().gap(&target.shape);
    let near = gu(attrs.min_range);
    let far = max_range(attrs, world.elevation(unit), target.elevation).max(near);
    let away = (unit.pos - target.pos)
        .try_normalize()
        .unwrap_or_else(|| -math::facing_vector(unit.facing));
    let size = Vec2::new(world.map.width() as f32, world.map.height() as f32) * SU_PER_GU;
    (unit.pos + away * ((near + far) / 2.0 - gap).max(1.0)).clamp(Vec2::ZERO, size)
}

/// `> HoldingPosition`, `> BaseAutoTargeting > Active during HoldPosition`: one tick. Locomotion stays Stationary:
/// the unit never moves. It engages the best ValidTarget within weapon range ([`World::pick`]) and disengages at
/// once when that target leaves range (or is no longer valid). A non-turret unit that can't turn in place engages
/// only what it already faces. Turrets have no state of their own yet, so a turret unit holds like the others. A
/// unit that can't attack now (its Riot Shield is up) holds without a target.
pub(super) fn hold(unit: &mut Unit, target: &mut Option<ObjectId>, world: &World) {
    let Some(attrs) = unit.attrs() else {
        if target.take().is_some() {
            unit.engage(None);
        }
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
        // Design change 0154: a unit has a turret by its own TurretAttributes, not by its base's CanHaveTurret.
        let facing_only = unit
            .unit_def()
            .is_some_and(|def| !def.has_turret() && !def.base_def().can_turn_in_place);
        *target = world.pick(unit, Scan::Weapon, facing_only, |_| true);
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
///   (or is no longer a ValidTarget, or comes inside the unit's MinRange: design change 0182, `Scan::Sight`), the
///   unit is idle again where it stands, with a new IdleOrigin next time. A target it can't get to is left alone for
///   [`GIVE_UP_RETRY_TICKS`] (design change 0181), so the unit doesn't set off for it tick after tick.
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
                let Some(target) = world.pick(unit, Scan::Sight, false, |c| Some(c.view.id) != skip) else {
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
