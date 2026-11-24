//! A worker's legs: to its patch, or to where the patch was, and back to its drop-off (the module docs of
//! `orders::gather`: legs are moves to a point).

use glam::Vec2;

use crate::defs::ObjectKind;
use crate::mover::{MoveGoal, MoveStatus};
use crate::shape::Shape;
use crate::units::SU_PER_GU;

use super::super::state::Gather;
use super::super::world::{Unit, World};
use super::LEG_RETRY_TICKS;

/// The shape a Space Crystals patch centred on `spot` had: where a patch that is gone was (design change 0134,
/// `state::Gather::spot`).
pub(super) fn spot_shape(spot: Vec2) -> Shape {
    let (w, h) = ObjectKind::CrystalPatch.def().tile_size().unwrap_or((1, 1));
    Shape::rect(spot, Vec2::new(w as f32, h as f32) * (SU_PER_GU / 2.0))
}

/// The point a leg to `target` heads for (module docs): the point nearest to the unit where its Silhouette stands
/// against one of the target's four faces, wholly within that face's span, so that a worker stands in front of its
/// own patch and not half in front of the next one in a row. A face is left out if the worker couldn't stand
/// there for a structure or a map resource its owner knows of (the faces between the patches of a row;
/// `World::known_fixed`, design change 0139: what stands out of the owner's sight is what it remembers there). With
/// no face left, the point nearest to the unit where it touches the target at all; the mover takes it from there (the
/// nearest point it can reach, design change 0025).
fn approach_point(unit: &Unit, target: &Shape, world: &World) -> Vec2 {
    let half = unit.kind.def().half_extents().unwrap_or(Vec2::ZERO);
    let (lo, hi) = target.bounds();
    // Where the unit's centre may be along a face: the face's span less the unit's half width, or its middle.
    let along = |from: f32, to: f32, half: f32, at: f32| {
        if to - from >= 2.0 * half {
            at.clamp(from + half, to - half)
        } else {
            (from + to) / 2.0
        }
    };
    let (x, y) = (
        along(lo.x, hi.x, half.x, unit.pos.x),
        along(lo.y, hi.y, half.y, unit.pos.y),
    );
    // North, south, west, east.
    let faces = [
        Vec2::new(x, lo.y - half.y),
        Vec2::new(x, hi.y + half.y),
        Vec2::new(lo.x - half.x, y),
        Vec2::new(hi.x + half.x, y),
    ];
    let overlaps = |p: Vec2, other: &Shape| {
        let (other_lo, other_hi) = other.bounds();
        p.x - half.x < other_hi.x && other_lo.x < p.x + half.x && p.y - half.y < other_hi.y && other_lo.y < p.y + half.y
    };
    let known = world.known_fixed(unit.owner);
    faces
        .into_iter()
        .filter(|&p| !known.iter().any(|other| overlaps(p, other)))
        .min_by(|a, b| a.distance_squared(unit.pos).total_cmp(&b.distance_squared(unit.pos)))
        .unwrap_or_else(|| unit.pos.clamp(lo - half, hi + half))
}

/// The leg to `target`: it begins, unless it is under way. One that ended is the caller's business: at a patch it
/// counts as arriving (design change 0141).
pub(super) fn leg(state: &mut Gather, unit: &mut Unit, target: &Shape, world: &World) {
    if !state.moving {
        let to = approach_point(unit, target, world);
        unit.mover.go_to(MoveGoal::Point(to), None, world.tick);
        state.moving = true;
    }
}

/// The leg back to the drop-off `target`: it begins, or begins again [`LEG_RETRY_TICKS`] after it ended short of
/// the drop-off (module docs).
pub(super) fn return_leg(state: &mut Gather, unit: &mut Unit, target: &Shape, world: &World) {
    if state.moving && unit.mover.status != MoveStatus::Moving {
        state.moving = false;
        state.retry_at = world.tick.saturating_add(LEG_RETRY_TICKS);
    }
    if world.tick >= state.retry_at {
        leg(state, unit, target, world);
    }
}
