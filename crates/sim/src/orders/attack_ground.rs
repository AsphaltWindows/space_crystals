//! Attack Ground — `systems/controls/commands.md > AttackGround`, `systems/controls/behaviors.md > AttackingLocation`
//! (slice M4-O, plan decision T47): a unit attacks a location, "the same as AttackingObject" with the location for
//! its target. It walks into range and stands, and phase 5e aims at the location and fires (`combat::phases`).
//!
//! - **Engaged** (`AttackState::location`): from the order's start (`orders::run::start`) for as long as it runs, but
//!   while the unit backs off from it (below). No one needs to see the location.
//! - **In range** (`combat::reaches_point`: from the unit's shape to the point, `systems/combat.md > Range
//!   Measurement`): the unit stops, and phase 5e aims and fires.
//! - **Out of range**: MovingToLocation, to the point itself; the walk ends in range, where this behaviour stops it.
//!   It starts once the attack sequence lets the unit move (`combat::may_move`), as an Attack's approach does. A walk
//!   that ends out of range gives up, and the order completes (`systems/controls/behaviors.md > Failure`).
//! - **Inside its MinRange** (design change 0182, as for a target object: `systems/controls/behaviors.md >
//!   AttackingObject > Target inside MinRange`): an Attack's own rule, and code (`engage::too_close`, slice M5-O3).
//!   The unit backs off straight away from the location to the middle of the band between MinRange and Range, letting
//!   go of the location meanwhile, and fires from there. One that can't move, or whose back-off ended with the
//!   location still inside ("cornered, or blocked behind", design change 0204 point 3: the location never moves),
//!   gives the order up with the "Target too close" feedback (`AttackGround::too_close`, then `Sim::step_too_close`'s
//!   `SimEvent::LocationTooClose`).
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

use glam::Vec2;

use super::engage::{TooNear, too_close};
use super::state::AttackGround;
use super::world::{Unit, World};
use crate::combat::reaches_point;
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
    let from = TooNear {
        target: None,
        pos: at,
        gap: shape.distance_to(at),
        elevation: ground,
    };
    if let Some(complete) = too_close(unit, attrs, state.back_off(), from, world) {
        return complete;
    }
    if reaches_point(attrs, &shape, world.elevation(unit), at, ground) {
        unit.halt();
        state.moving = false;
        return false;
    }
    if state.moving {
        if unit.mover.status == MoveStatus::Moving {
            return false;
        }
        // The walk ended out of range: the unit is as near as it gets.
        state.moving = false;
        unit.engage(None);
        return true;
    }
    if unit.may_move() {
        unit.mover.go_to(MoveGoal::Point(at), None, world.tick);
        state.moving = true;
    }
    false
}

#[cfg(test)]
mod tests;
