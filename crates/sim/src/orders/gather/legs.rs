//! A worker's legs: to its patch, or to where the patch was, and back to its drop-off (`orders::gather`).
//!
//! **Legs are moves to a point, not to the object** (a deviation from the plan's sketch, which named
//! `MoveGoal::Object`): the point nearest to the worker where it stands against a face of the target
//! ([`approach_point`]). A move to an object is planned to the point nearest the object's **centre** that the
//! unit can stand on, ties going to the lowest tile: always the north side of a patch in a row and of an Egress
//! Node, wherever the worker comes from. A Drone south of a row of patches would walk round the row's end to mine
//! from the north. The point nearest to the worker keeps trips straight and spreads workers along a face.
//!
//! **A leg to a patch that ends short of it** (the mover arrived as near as it gets behind units that won't move,
//! `systems/controls/behaviors.md > MovingToLocation > Occupied target location`, or gave up) counts as arriving
//! there, out of reach: "a patch the Drone can't get within MiningReach of (path blocked by units, structures or
//! terrain) is treated like an occupied patch" (design change 0141). A leg back to the drop-off that ends short is
//! begun again after [`LEG_RETRY_TICKS`], a back-off: what stopped it is a unit that won't move, and that is rarely
//! gone a tick later. A worker that set off again at once would ask for a path every dozen ticks and jitter against
//! its blocker; standing, it is a unit at rest, which the units behind it route around at once.

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
///
/// `beside` widens each face's span by as much at either end: 0 for a patch or a whole drop-off; for a hatch, what
/// lets a worker stand beside the square within DropOffReach of it ([`return_leg`]).
fn approach_point(unit: &Unit, target: &Shape, beside: f32, world: &World) -> Vec2 {
    let half = unit.kind.def().half_extents().unwrap_or(Vec2::ZERO);
    let (lo, hi) = target.bounds();
    let (from, to) = (lo - beside, hi + beside);
    // Where the unit's centre may be along a face: the face's span less the unit's half width, or its middle.
    let along = |from: f32, to: f32, half: f32, at: f32| {
        if to - from >= 2.0 * half {
            at.clamp(from + half, to - half)
        } else {
            (from + to) / 2.0
        }
    };
    let (x, y) = (
        along(from.x, to.x, half.x, unit.pos.x),
        along(from.y, to.y, half.y, unit.pos.y),
    );
    // North, south, west, east.
    let faces = [
        Vec2::new(x, lo.y - half.y),
        Vec2::new(x, hi.y + half.y),
        Vec2::new(lo.x - half.x, y),
        Vec2::new(hi.x + half.x, y),
    ];
    // The same faces without `beside`: a face is one only if the worker could stand in front of the target itself
    // there (a hatch's faces inside its structure, and those along its wall, aren't).
    let (x, y) = (
        along(lo.x, hi.x, half.x, unit.pos.x),
        along(lo.y, hi.y, half.y, unit.pos.y),
    );
    let fronts = [
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
    let open = |p: Vec2| !known.iter().any(|other| overlaps(p, other));
    faces
        .into_iter()
        .zip(fronts)
        .filter(|&(p, front)| open(p) && (beside == 0.0 || open(front)))
        .map(|(p, _)| p)
        .min_by(|a, b| a.distance_squared(unit.pos).total_cmp(&b.distance_squared(unit.pos)))
        .unwrap_or_else(|| unit.pos.clamp(lo - half, hi + half))
}

/// The leg to `target`: it begins, unless it is under way. One that ended is the caller's business: at a patch it
/// counts as arriving (design change 0141).
///
/// `beside` is `None` for a worker that stands in front of its target, wholly within a face ([`approach_point`]), and
/// its reach for one that may stand anywhere against a face from where it is within that reach of the target: at a
/// hatch (`return_leg`).
pub(super) fn leg(state: &mut Gather, unit: &mut Unit, target: &Shape, beside: Option<f32>, world: &World) {
    if !state.moving {
        // Its Silhouette's near edge is then within the reach of the target's: a space unit less, for the rounding.
        let beside = beside.map_or(0.0, |reach| {
            let half = unit.kind.def().half_extents().unwrap_or(Vec2::ZERO);
            (reach + 2.0 * half.max_element() - 1.0).max(0.0)
        });
        let to = approach_point(unit, target, beside, world);
        leg_to(state, unit, to, world);
    }
}

/// The leg to the point `to`: it begins, unless one is under way. The station's legs head for a spot of their own
/// (`station::spot`, slice M4-I).
pub(super) fn leg_to(state: &mut Gather, unit: &mut Unit, to: Vec2, world: &World) {
    if !state.moving {
        unit.mover.go_to(MoveGoal::Point(to), None, world.tick);
        state.moving = true;
    }
}

/// The leg back to the drop-off point `target`: it begins, or begins again [`LEG_RETRY_TICKS`] after it ended short
/// of the drop-off (module docs). The worker never gives up (design change 0255: "Agents waiting at the hatch wait
/// forever"; "The Drones' code placeholder (retry every 2 s, never give up) is consistent with this; consider the
/// same rule for Drones settled").
///
/// To a hatch (`hatch_reach`: the worker's DropOffReach there, `None` for a drop-off that takes loads anywhere) it
/// heads for the point nearest to it against the hatch's wall **within reach of the hatch**: in front of the square,
/// or beside it by up to the reach (`factions/syndicate/units/agent.md > Gathering rules`: "an Agent waits within
/// reach (or as close as it can get) while another drops off"), so that those waiting stand side by side along the
/// wall and the next one is in reach when the hatch frees.
pub(super) fn return_leg(state: &mut Gather, unit: &mut Unit, target: &Shape, hatch_reach: Option<f32>, world: &World) {
    if state.moving && unit.mover.status != MoveStatus::Moving {
        state.moving = false;
        state.retry_at = world.tick.saturating_add(LEG_RETRY_TICKS);
    }
    if world.tick >= state.retry_at {
        leg(state, unit, target, hatch_reach, world);
    }
}
