//! Attack Ground — `systems/controls/commands.md > AttackGround`, `systems/controls/command_behaviors.md >
//! AttackingLocation` (slice M4-O, plan decision T47): a unit attacks a location, "the same as AttackingObject" with
//! the location for its target. It walks into range and stands, and phase 5e aims at the location and fires
//! (`combat::phases`).
//!
//! - **Engaged** (`AttackState::location`): from the order's start (`orders::run::start`) for as long as it runs, but
//!   while the unit backs off from it (below). No one needs to see the location.
//! - **In range** (`combat::reaches_point`: from the unit's shape to the point, `systems/combat.md > Range
//!   Measurement`), and (M5-U1) for a unit with a turret within its arc ("Range/arc check is against the target
//!   location", `combat::turret::in_arc`): the unit stops, and phase 5e aims and fires. Its turret's LockedTarget
//!   stays as it was ("TurretCommandState.LockedTarget is not set (turret fires at location, not a unit)"): the
//!   turret aims at the location while the order runs (`combat::turret::step`). (M5-N10) A unit that aims with its
//!   hull as well (`combat::turret::aims_by_hull`: the Hand of Heaven) stops in range wherever the location is
//!   (`Unit::takes_aim`), and asks phase 5d to turn its hull toward it (`Unit::aim_hull`), as for a target object
//!   (`engage::close_in`). (M6-S6a) A unit that can't turn in place stops only where the location will be in its arc
//!   at rest (`engage::rests_in_arc`, as for a target object).
//! - **Out of range**: MovingToLocation, to the point itself; the walk ends in range, where this behaviour stops it.
//!   It starts once the attack sequence lets the unit move (`combat::may_move`), as an Attack's approach does. A walk
//!   that ends out of range gives up, and the order completes (`systems/controls/command_behaviors.md > Failure`).
//! - (M6-S6a) **Can't aim from where it stands** (design change 0460, answering Q0184:
//!   `systems/controls/command_behaviors.md > AttackingLocation > Algorithm`, "a unit in range of the location that
//!   can't bring its weapon to bear (e.g. a narrow-arc wheeled vehicle beside the spot) **repositions** to a spot from
//!   which it can fire [...]; it doesn't circle the spot forever"): as for an explicit Attack (`engage::close_in`), a
//!   unit in range that can't aim from where it stands (`aim::stuck`), or whose walk ended in range with the location
//!   out of its arc, moves to a spot from which it can (`aim::spot`, the location its mark, `aim::Mark::point`, with
//!   no leash; `AttackGround::repositioning`), and fires from there. On its way it stops as soon as it can fire, and
//!   goes on though its way pass inside MinRange, since the spot lies beyond it. With no spot it does as before: it
//!   walks on, and gives up as near as it gets; a move to its spot that fails gives the order up. (Before, the Assault
//!   Bike circled a location in range far out of its arc for good, halting once a lap: Q0184.)
//! - **Inside its MinRange** (design change 0182, as for a target object: `systems/controls/command_behaviors.md >
//!   AttackingObject > Target inside MinRange`): an Attack's own rule, and code (`engage::too_close`, slice M5-O3). The
//!   unit backs off straight away from the location to MinRange + 1 grid unit (design change 0454, which replaced
//!   Q0054's middle of the band between MinRange and Range), letting go of the location meanwhile, and fires from
//!   there. One that can't move, or whose back-off ended with the location still inside ("cornered, or blocked behind",
//!   design change 0204 point 3: the location never moves), gives the order up with the "Target too close" feedback
//!   (`AttackGround::too_close`, then `Sim::step_too_close`'s `SimEvent::LocationTooClose`).
//! - **It keeps firing** (design change 0293, answering Q0083 point 3; it replaced "Behavior complete after attack
//!   effect is applied"): "like an attack on a unit that never dies, the behavior repeats its attack cycle at the
//!   location until the player gives another order (or Stop). Queued orders after an Attack Ground therefore never
//!   start on their own." Its sequences follow one another at the location (`combat::phases`), and the order never
//!   completes by itself, but by the Failure rules above, or once the unit can't attack the location any more: no
//!   attack in hand (a Riot Shield up), or a weapon that can't target the ground, whose location phase 5e lets go
//!   of. A projectile already launched lands whatever becomes of the order (`combat::projectiles`). (A Specter's
//!   Attack Ground follows its ammo rules, `factions/syndicate/units/specter.md`: slice N7's.)
//! - The point is the one clicked, with no snapping and no group destinations (plan decision T47): every unit given
//!   the order fires at it.
//! - (M5-H1) **A Glider strafes it** (`systems/controls/command_behaviors.md > AttackingLocation`, GliderMovement
//!   exception: "Same strafing run pattern as AttackingObject, targeting the ground location"): its move is a strafing
//!   run at the point (`Mover::strafe`, `movement::glider`) out to its attack's Range, and phase 5e fires while the
//!   point is within Range and its turret's arc. It never stops, backs off nor gives up: only another order ends it,
//!   or its weapon letting go of the location.

use glam::Vec2;

use super::aim::{self, Leash, Mark};
use super::engage::{TooNear, rests_in_arc, too_close};
use super::state::AttackGround;
use super::world::{Unit, World};
use crate::combat::{max_range, reaches_point};
use crate::defs::AttackAttrs;
use crate::elevation::Elevation;
use crate::mover::{MoveGoal, MoveStatus};

/// One tick of an Attack Ground at `at` (module docs). Returns whether the behaviour is complete.
pub(super) fn run(unit: &mut Unit, at: Vec2, state: &mut AttackGround, world: &World) -> bool {
    // The attack let go of a location its weapon can't target (`combat::phases`): the unit can't carry the order out.
    // One backing off let go of it itself.
    let engaged = unit.attack.as_deref().is_some_and(|attack| attack.location == Some(at));
    if !engaged && state.backing_off.is_none() {
        return true;
    }
    let Some(attrs) = unit.attrs() else {
        unit.engage(None);
        return true;
    };
    let shape = unit.shape();
    let ground = Elevation::ground(world.map, at);
    if unit.glides() {
        let goal = MoveGoal::Point(at);
        if !state.moving || unit.mover.goal != goal || unit.mover.strafe.is_none() {
            unit.mover.go_to(goal, None, world.tick);
            unit.mover.strafe(max_range(attrs, world.elevation(unit), ground));
            state.moving = true;
        }
        return false;
    }
    let from = TooNear {
        target: None,
        pos: at,
        gap: shape.distance_to(at),
        elevation: ground,
    };
    // (M6-S6a) A unit on its way to a spot to aim from goes on there though its way pass inside MinRange: the spot
    // lies beyond it (`orders::aim`).
    if state.repositioning.is_none()
        && let Some(complete) = too_close(unit, attrs, state.back_off(), from, world)
    {
        return complete;
    }
    let in_range = reaches_point(attrs, &shape, world.elevation(unit), at, ground);
    if state.repositioning.is_some() {
        match unit.mover.status {
            // On its way it stops as soon as it can fire.
            MoveStatus::Moving if !(in_range && unit.in_arc(at) && rests_in_arc(unit, at)) => return false,
            MoveStatus::Moving | MoveStatus::Arrived => state.repositioning = None,
            // The move to its spot failed (`systems/controls/command_behaviors.md > Failure`): as near as it gets.
            MoveStatus::Failed | MoveStatus::TargetLost | MoveStatus::Idle => {
                state.repositioning = None;
                unit.engage(None);
                return true;
            }
        }
    }
    if in_range && unit.takes_aim(at) && rests_in_arc(unit, at) {
        // A unit that aims with its hull, the location out of its arc and its turn cut where it stands, moves (design
        // change 0460), but for one that has just stopped its walk: it tries the turn first.
        let aims = unit.in_arc(at) || state.moving || !unit.may_move();
        if !aims
            && aim::stuck(unit, attrs, at, world)
            && let Some(spot) = spot(unit, attrs, at, ground, world)
        {
            return set_off(unit, state, spot, world);
        }
        unit.halt();
        unit.aim_hull(at);
        state.moving = false;
        return false;
    }
    if state.moving {
        if unit.mover.status == MoveStatus::Moving {
            return false;
        }
        state.moving = false;
        // (M6-S6a) As near as it gets, in range, out of its arc (design change 0460): it repositions.
        if in_range
            && unit.may_move()
            && let Some(spot) = spot(unit, attrs, at, ground, world)
        {
            return set_off(unit, state, spot, world);
        }
        // The walk ended out of range: the unit is as near as it gets.
        unit.engage(None);
        return true;
    }
    if unit.may_move() {
        // (M6-S6a) A location it can't come round onto from here (design change 0460).
        if in_range
            && aim::stuck(unit, attrs, at, world)
            && let Some(spot) = spot(unit, attrs, at, ground, world)
        {
            return set_off(unit, state, spot, world);
        }
        unit.mover.go_to(MoveGoal::Point(at), None, world.tick);
        state.moving = true;
    }
    false
}

/// The spot `unit` repositions to so as to bring its weapon, `attrs`, to bear on the location `at`, its ground at
/// `ground` (`aim::spot`: an order's own mark has no leash).
fn spot(unit: &Unit, attrs: &AttackAttrs, at: Vec2, ground: Elevation, world: &World) -> Option<Vec2> {
    aim::spot(unit, attrs, Mark::point(at, ground), Leash::Free, world)
}

/// The unit sets off for `spot` (`aim::go`), its move to it now the behaviour's ([`AttackGround::repositioning`]).
fn set_off(unit: &mut Unit, state: &mut AttackGround, spot: Vec2, world: &World) -> bool {
    aim::go(unit, spot, world);
    state.repositioning = Some(spot);
    state.moving = false;
    false
}

#[cfg(test)]
mod tests;
