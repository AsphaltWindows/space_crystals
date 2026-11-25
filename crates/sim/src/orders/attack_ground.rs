//! Attack Ground — `systems/controls/commands.md > AttackGround`, `systems/controls/behaviors.md > AttackingLocation`
//! (slice M4-O, plan decision T47): a unit attacks a location, "the same as AttackingObject" with the location for
//! its target. It walks into range and stands, and phase 5e aims at the location and fires (`combat::phases`).
//!
//! - **Engaged** (`AttackState::location`): from the order's start (`orders::run::start`) for as long as it runs. No
//!   one needs to see the location.
//! - **In range** (`combat::reaches_point`: from the unit's shape to the point, `systems/combat.md > Range
//!   Measurement`): the unit stops, and phase 5e aims and fires.
//! - **Out of range**: MovingToLocation, to the point itself; the walk ends in range, where this behaviour stops it.
//!   It starts once the attack sequence lets the unit move (`combat::may_move`), as an Attack's approach does. A walk
//!   that ends out of range gives up, and the order completes (`systems/controls/behaviors.md > Failure`).
//! - **Inside its MinRange** (design change 0182, as for a target object): the unit backs off straight away from the
//!   location (`engage::back_off_spot`), and gives up if the back-off ends with the location still inside. A unit that
//!   can't move gives up at once. TODO(feedback): an Attack's "Target too close" (`SimEvent::TargetTooClose`) has no
//!   location counterpart yet; no M4 kind can target the ground and has a MinRange.
//! - **It keeps firing** (design change 0293, answering Q0083 point 3; it replaced "Behavior complete after attack
//!   effect is applied"): "like an attack on a unit that never dies, the behavior repeats its attack cycle at the
//!   location until the player gives another order (or Stop). Queued orders after an Attack Ground therefore never
//!   start on their own." Its sequences follow one another at the location (`combat::phases`), and the order never
//!   completes by itself, but by the Failure rule above, or once the unit can't attack the location any more: no
//!   attack in hand (a Riot Shield up), or a weapon that can't target the ground, whose location phase 5e lets go
//!   of. A projectile already launched lands whatever becomes of the order (`combat::projectiles`). (A Specter's
//!   Attack Ground follows its ammo rules, `factions/syndicate/units/specter.md`: slice N7's.)
//! - The point is the one clicked, with no snapping and no group destinations (plan decision T47): every unit given
//!   the order fires at it.

use glam::Vec2;

use super::engage::back_off_spot;
use super::state::AttackGround;
use super::world::{Unit, World};
use crate::combat::{inside_min_range, reaches_point};
use crate::elevation::Elevation;
use crate::mover::{MoveGoal, MoveStatus};

/// One tick of an Attack Ground at `at` (module docs). Returns whether the behaviour is complete.
pub(super) fn run(unit: &mut Unit, at: Vec2, state: &mut AttackGround, world: &World) -> bool {
    // The attack let go of a location its weapon can't target (`combat::phases`): the unit can't carry the order out.
    let engaged = unit.attack.as_deref().is_some_and(|attack| attack.location == Some(at));
    if !engaged {
        return true;
    }
    let Some(attrs) = unit.attrs() else {
        unit.engage(None);
        return true;
    };
    let shape = unit.shape();
    let ground = Elevation::ground(world.map, at);
    if reaches_point(attrs, &shape, world.elevation(unit), at, ground) {
        unit.halt();
        state.moving = false;
        return false;
    }
    if state.moving {
        if unit.mover.status == MoveStatus::Moving {
            return false;
        }
        // The walk, or the back-off, ended out of range: the unit is as near as it gets.
        state.moving = false;
        unit.engage(None);
        return true;
    }
    let gap = shape.distance_to(at);
    if inside_min_range(attrs, gap) {
        let max_speed = unit.unit_def().map_or(0.0, |def| def.movement.max_speed());
        if !unit.mover.can_move(max_speed) {
            unit.engage(None);
            return true;
        }
        if unit.may_move() {
            let spot = back_off_spot(unit, attrs, at, gap, ground, world);
            unit.mover.go_to(MoveGoal::Point(spot), None, world.tick);
            state.moving = true;
        }
        return false;
    }
    if unit.may_move() {
        unit.mover.go_to(MoveGoal::Point(at), None, world.tick);
        state.moving = true;
    }
    false
}

#[cfg(test)]
mod tests;
