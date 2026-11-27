//! Idle units that overlap move apart — `systems/units.md > UnitCollision > Units that already overlap > Idle units
//! push apart` (design change 0126, answering question Q0045 point 1; it amends 0106's "nothing is pushed apart",
//! which still holds for every unit with a command).
//!
//! "An idle unit (no active command, the same set that yields) overlapping another unit moves off automatically
//! until they no longer overlap; two idle units both move apart. Units with an active command (incl. Hold
//! Position, mining) are not moved. Units never stay stacked indefinitely."
//!
//! - **Who**: a unit that yields (`yielding::can_yield`: an owner, no order at all, no goal, not in an attack
//!   sequence, able to move) and is at rest, whose box overlaps another unit's of its layer by I1's own predicate
//!   (`space::overlaps`; ground and underground, the layers with hard collision). Only the exceptions of
//!   `space::blocks` ever bring two boxes to overlap, so today it is a Drone whose Gather ended inside another.
//! - **Which way**: away from the overlapping unit with the lowest id, along the axis on which it is out soonest
//!   (the smaller overlap; x on a tie). On an axis where the two centres coincide, the unit with the lower id goes
//!   the negative way (west, north) and the other the positive: no randomness, and two idle units part.
//!   If nothing lets it move that way (a wall, a unit it doesn't overlap), it tries the other axis, then the
//!   two ways back through the unit.
//! - **How far**: `Movement::separate_speed_su` a tick, at most its own speed, and no farther than to where the two
//!   boxes touch: minimal, just until clear. It is a plain step for every movement model, like a yielding unit's
//!   sidestep: a unit with momentum takes it without any, so it neither overshoots nor swings back.
//! - **Collision stays hard**: the step goes through `World::shuffle` like any other, so it never enters a unit
//!   the mover doesn't already overlap, nor a blocked tile. A unit shut in on every side stays where it is.
//!
//! A stack of `n` idle units on one point comes apart one unit at a time, the highest id first: within
//! `18 × (n − 1)` ticks for Drones (36 su at 2 su a tick each), and sooner while two of them move at once
//! (`tests/movement_separate.rs`).

use glam::Vec2;

use super::collide::World;
use super::space::{overlaps, resting};
use crate::mover::{Locomotion, Mover, Orientation};
use crate::ruleset::rules;

/// One tick of an idle unit at rest (module docs): if its box overlaps another unit's, it takes a step off it.
/// Returns whether it overlapped one, moved or not. `speed` is its own, space units per tick.
pub(super) fn step(pos: &mut Vec2, mover: &mut Mover, speed: f32, world: &mut World, touched: &mut Vec<usize>) -> bool {
    let tuning = &rules().tuning.movement;
    let (Some(layer), me) = (world.layer, *world.body()) else {
        return false;
    };
    // The overlapping unit with the lowest id: bodies are in id order.
    let mut first: Option<usize> = None;
    layer.near(*pos - me.half, *pos + me.half, |index| {
        let other = &world.bodies[index as usize];
        if other.id != me.id && overlaps(&me, *pos, other) {
            first = Some(first.map_or(index as usize, |found| found.min(index as usize)));
        }
    });
    let Some(other) = first.map(|index| world.bodies[index]) else {
        return false;
    };
    let (offset, reach) = (*pos - other.pos, me.half + other.half);
    let overlap = reach - offset.abs();
    // The way out on each axis: away from the other's middle; where the middles coincide, by id.
    let away = |axis: usize| {
        if offset[axis] != 0.0 {
            offset[axis] > 0.0
        } else {
            me.id > other.id
        }
    };
    let near = usize::from(overlap.y < overlap.x);
    let step = speed.min(tuning.separate_speed_su);
    let from = *pos;
    for (axis, positive) in [
        (near, away(near)),
        (1 - near, away(1 - near)),
        (near, !away(near)),
        (1 - near, !away(1 - near)),
    ] {
        // Where the two boxes touch on that side, and a step towards it.
        let clear = resting(other.pos[axis], reach[axis], !positive) - from[axis];
        let mut d = Vec2::ZERO;
        d[axis] = if positive { clear.min(step) } else { clear.max(-step) };
        touched.clear();
        let to = world.shuffle(from, d, touched);
        if to != from {
            *pos = to;
            break;
        }
    }
    mover.sidestep = None;
    mover.velocity = *pos - from;
    mover.locomotion = if mover.velocity == Vec2::ZERO {
        Locomotion::Stationary
    } else {
        Locomotion::Moving
    };
    mover.orientation = Orientation::Maintaining;
    mover.blocked_ticks = 0;
    true
}
