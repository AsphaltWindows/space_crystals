//! (M5-V4) Crushing — `systems/units.md > UnitCollision > Crushing`, plan decision T86 with design changes 0418 and
//! 0419 (`docs/movement.md > 6.13`): "Units whose UnitBase can crush (TrackedVehicle, Mech, and a **DrillUnit in its
//! above-ground mode** [...]; never underground) crush enemy units whose UnitBase has Crushable = true (Light
//! Infantry)" (`UnitBaseDef::crushes`, `UnitBaseDef::crushable`). "**Effect**: instant death of the crushed unit."
//!
//! - **Whose move crushes** ([`drives`]): "**Trigger**: the crusher's Silhouette overlaps a crushable enemy **while
//!   the crusher is moving** (Locomotion = Moving or Reversing)", and "**Turning in place crushes**" (design change
//!   0418: "Silhouette overlap while Moving, Reversing **or Stationary + Turning**"). It is decided as the unit's turn
//!   of phase 5d begins, before its move, which every part of asks the collision exceptions (`space::blocks`): the
//!   probe, the lanes, the turn's every part. The Locomotion that move will set isn't known yet, and last tick's
//!   would never let a crusher parked against an enemy start nor spin. Every move a crusher makes in its turn sets
//!   Moving or Reversing, or spins it in place, but for a **Stopping** one: braking to rest with no way to steer
//!   (`super::model::Model::coast`), and braking against its leg before it sets off the other way (`super::tracked`).
//!   A literal reading: Stopping isn't in the trigger's list, so a crusher that brakes crushes nothing, and the enemy
//!   infantry in its way stops it at contact as any unit does. A unit whose attack holds its base doesn't move.
//! - **Collision** (`space::blocks`, rule 3): such a move passes through the crushable enemies of its layer ("those
//!   enemies are not obstacles to them"; a spin's "don't block the spin"), and nothing else changes: a crusher that
//!   stands and doesn't turn blocks them ("a normal hard obstacle; crushable units cannot walk into it"), they keep
//!   avoiding crushers, and own and allied crushables block a crusher ("No friendly crushing"), its spin too, which
//!   "stops at contact". So a crusher's follower goes round its own infantry and straight through the enemy's.
//! - **The kill** ([`victims`]): at its commit, the crushable enemies its Silhouette overlaps where it ends, or
//!   swept on the way (its turn, then its move: `Shape` and the rotated shape, `systems/units.md > UnitCollision >
//!   Silhouette Rotation`), found through the movement grid's query, leave the layer at once: the units after it in
//!   the tick no longer meet them, and those not yet moved don't move. They die through the removal path in the same
//!   phase (`Sim::remove_objects`, which frees their Unit Control and their lineage with them), each with a
//!   `SimEvent::Crushed` in place of a `Died`. It isn't damage: no Hit, no armour, no reveal, no Shields. So no
//!   overlap with a crushed unit outlives its tick, and the no-overlap invariant I1, checked between ticks, needs no
//!   exception.
//! - **Enemies** (`economy::defeat::allied`): another player's units. Neutral ones are nobody's enemy, and block.
//!
//! The DrillUnit has no above-ground mode yet (slice M6-ST1 builds it, and 0419's crushing with it): underground it
//! never crushes, and its base's flag is false.

use std::collections::BTreeMap;

use glam::Vec2;

use super::Chased;
use super::space::{Body, KIND_CRUSHABLE, Layer};
use crate::defs::{MovementAttrs, UnitBaseDef};
use crate::economy::defeat;
use crate::events::SimEvent;
use crate::ids::{ObjectId, PlayerId};
use crate::math;
use crate::mover::{MoveGoal, Mover, PathState};
use crate::object::Object;
use crate::production::ExitDoor;
use crate::shape::Shape;
use crate::tuning::movement::TURN_TRY_SU;
use crate::vision::Hiding;

/// One crush of the tick (module docs).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) struct Crush {
    /// The crushed unit: an index into the tick's bodies.
    pub(super) victim: usize,
    /// Its crusher, likewise.
    pub(super) by: usize,
}

/// Whether the players `a` and `b` are enemies: each a player, of another side (`economy::defeat::allied`). A neutral
/// unit is nobody's enemy.
fn enemies(a: Option<PlayerId>, b: Option<PlayerId>) -> bool {
    a.zip(b).is_some_and(|(a, b)| !defeat::allied(a, b))
}

/// `space::blocks`'s rule 3: whether the move of `mover` goes through `other` (module docs): its move this tick
/// crushes ([`Body::crushes`]), and `other` is a crushable enemy of its.
pub(super) fn crushes(mover: &Body, other: &Body) -> bool {
    mover.crushes && other.crushable && enemies(mover.owner, other.owner)
}

/// Whether the move a unit of base `base` and movement numbers `movement` makes in its turn of this tick crushes
/// (module docs; [`Body::crushes`]), as its turn begins: `mover` as it is, facing `facing`, `held` if its attack holds
/// its base, in the exit doorway `door` if it is in one. With no momentum its move is Moving, a spin, or none at
/// all. With momentum it walks out of its doorway by its model (`super::doorway`), or else steers along its path if
/// `steers` (its path's goal is still ahead, [`steers`]) and doesn't brake against its leg ([`brakes`]), and coasts
/// to rest if not: those two are Stopping.
pub(super) fn drives(
    base: &UnitBaseDef,
    movement: &MovementAttrs,
    mover: &Mover,
    facing: f32,
    held: bool,
    door: Option<&ExitDoor>,
    steers: impl FnOnce() -> bool,
) -> bool {
    if !base.crushes || held {
        return false;
    }
    if mover.momentum == Vec2::ZERO {
        return true;
    }
    match door {
        Some(ExitDoor::Centre(_)) => true,
        Some(ExitDoor::Side(_)) if mover.goal == MoveGoal::None => true,
        _ => steers() && !brakes(movement, mover, facing),
    }
}

/// Whether the unit steers along its path this tick (`super::follow`), its shape where it stands being `shape` and
/// the object it heads for `chased`: it has a goal and a path to it, and an object goal is still there and not
/// within its reach. Else it stands, or coasts to rest.
pub(super) fn steers(mover: &Mover, shape: &Shape, chased: Option<&Chased>) -> bool {
    let reached = match mover.goal {
        MoveGoal::None => return false,
        MoveGoal::Point(_) => false,
        MoveGoal::Object { reach, .. } => chased.is_none_or(|chased| shape.gap(&chased.shape) <= reach),
    };
    !reached && mover.path_state == PathState::Ready && usize::from(mover.next) < mover.path.len()
}

/// Whether a SpeedTurnRadiusMovement unit, `mover` facing `facing`, brakes against the way its leg drives this tick:
/// it still moves the other way, faster than one tick's Deceleration takes away, so its move is Stopping
/// (`super::tracked`: "Moving the other way from its leg, it brakes to rest first"). These alone decide the drive's
/// Stopping (`Tracked::drive`), whatever the step it steers for. The other models are no crusher's but the Mech's
/// TurnRateMovement, which has no speed to brake from.
fn brakes(movement: &MovementAttrs, mover: &Mover, facing: f32) -> bool {
    let MovementAttrs::SpeedTurnRadius { deceleration, .. } = *movement else {
        return false;
    };
    // The drive's signed speed (`tracked::speed_of`) along the way its leg drives (`Tracked::of`: backwards on a
    // Reverse leg, for a unit that can back up).
    let speed = mover.momentum.length();
    let current = if mover.momentum.dot(math::facing_vector(facing)) < 0.0 {
        -speed
    } else {
        speed
    };
    let along = if mover.reverses && movement.reverse_speed().is_some() {
        -current
    } else {
        current
    };
    along + deceleration.unwrap_or(f32::INFINITY) < 0.0
}

/// The crushes of the crusher `me` (an index into `bodies`) in its turn of this tick, whose move crushed ([`drives`]),
/// noted in `crushed`: it moved from `from`, facing `start`, to where its body now lies (`bodies[me]`, as it
/// committed). Each crushable enemy of its layer that its shape overlaps there, or overlapped on the way: along its
/// turn, which every model makes before its move and where it stands, tried as `World::turn` tries it (in parts
/// along which its corners move at most `TURN_TRY_SU`), and along its move in the shape it ends in (`Shape::sweep`,
/// which counts touching as free). None if it neither moved nor turned ("a crusher that is stationary
/// and **not** turning stays a hard obstacle"). Those crushed earlier in the tick have left the layer. Once each, in
/// the grid's order.
pub(super) fn victims(bodies: &[Body], layer: &Layer, me: usize, from: Vec2, start: f32, crushed: &mut Vec<Crush>) {
    let now = bodies[me];
    let turned = now.oriented.map_or(0.0, |lies| math::wrap_angle(lies.facing - start));
    if now.pos == from && turned == 0.0 {
        return;
    }
    let moved = now.pos - from;
    let ending = now.shape_at(now.pos);
    let leaving = now.shape_at(from);
    // Its turn, where it began, in parts as `World::turn` tries it: the shape at the end of part `part`, those before
    // the facing it ended at.
    let (half, parts) = match now.oriented {
        Some(lies) if turned != 0.0 => {
            let parts = (turned.abs() * lies.half.length() / TURN_TRY_SU).ceil().max(1.0) as u32;
            (lies.half, parts)
        }
        _ => (Vec2::ZERO, 0),
    };
    let turning = |part: u32| {
        let facing = math::wrap_angle(start + turned * (part as f32 / parts as f32));
        Shape::oriented(from, half, math::facing_vector(facing))
    };
    // A shape that turns lies within its circumscribed circle at every facing.
    let reach = now.oriented.map_or(now.half, |lies| Vec2::splat(lies.half.length()));
    let first = crushed.len();
    layer.near_any(
        from.min(now.pos) - reach,
        from.max(now.pos) + reach,
        KIND_CRUSHABLE,
        |index| {
            let index = index as usize;
            let other = &bodies[index];
            if index == me
                || !other.crushable
                || !enemies(now.owner, other.owner)
                || crushed[first..].iter().any(|c| c.victim == index)
            {
                return;
            }
            let shape = other.shape_at(other.pos);
            let swept = ending.overlaps(&shape)
                || (moved != Vec2::ZERO && leaving.sweep(moved, &shape).is_some())
                || (1..parts).any(|part| turning(part).overlaps(&shape));
            if swept {
                crushed.push(Crush { victim: index, by: me });
            }
        },
    );
}

/// The tick's crushes (`crushed`, of the tick's `bodies`) as the sim tells them: a `SimEvent::Crushed` for each, in
/// the order they came, the crushed unit where it died and as hidden as it was (`Hiding::of`), as a death is told.
/// Returns the crushed, for the removal path.
pub(super) fn tell(
    objects: &BTreeMap<ObjectId, Object>,
    bodies: &[Body],
    crushed: &[Crush],
    events: &mut Vec<SimEvent>,
) -> Vec<ObjectId> {
    let mut dead = Vec::with_capacity(crushed.len());
    for crush in crushed {
        let Some(o) = objects.get(&bodies[crush.victim].id) else {
            continue;
        };
        events.push(SimEvent::Crushed {
            id: o.id,
            kind: o.kind,
            owner: o.owner,
            pos: o.pos,
            hiding: Hiding::of(o),
            crusher: bodies[crush.by].id,
        });
        dead.push(o.id);
    }
    dead.sort_unstable();
    dead
}

#[cfg(test)]
mod tests;
