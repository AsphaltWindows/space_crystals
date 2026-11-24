//! Occupancy and the search for another patch (`factions/colonists/units.md > Drone > Patch Occupancy (one Drone
//! per patch)`, design changes 0131 to 0134 and 0141; the module docs of `orders::gather`).

use crate::defs::WorkerDef;
use crate::ids::ObjectId;
use crate::units::gu;

use super::super::world::{Unit, World};
use super::Claims;

/// What a worker found when it looked round ([`search`]).
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct Found {
    /// The nearest free patch it can reach.
    pub free: Option<ObjectId>,
    /// The nearest patch of all, free or not: of those it can reach if there are any, else of those it can't.
    pub any: Option<ObjectId>,
}

/// `> Patch Occupancy > Search radius` (design changes 0123 and 0131): the patches `unit` looks at are "within a
/// radius around itself" (`WorkerDef::search_radius`, centre to centre), those its owner knows ("currently
/// visible, or remembered from earlier sight": `World::knows`; "a Drone never finds a patch hidden in fog") and
/// "not covered by a structure", all but `except`, the one it comes from. "The nearest unoccupied patch to itself"
/// is the nearest centre to centre, the lowest id among equals, of those not [`taken`] that it can reach: design
/// change 0141 treats a patch it can't reach like an occupied one (`World::reachable`, by the grids its owner
/// knows; a wall of units shows only when a leg ends short of the patch).
pub(super) fn search(world: &World, claims: &Claims, unit: &Unit, def: &WorkerDef, except: Option<ObjectId>) -> Found {
    let radius = gu(def.search_radius);
    // (distance squared, id) of the nearest free patch, and (out of reach, distance squared, id) of the nearest.
    let mut free: Option<(f32, ObjectId)> = None;
    let mut any: Option<(bool, f32, ObjectId)> = None;
    for patch in world.patches() {
        let far = patch.pos.distance_squared(unit.pos);
        if far > radius * radius
            || Some(patch.id) == except
            || world.patch_covered(patch.id)
            || !world.knows(unit.owner, patch)
        {
            continue;
        }
        // In id order, so the first of equals stays.
        let reachable = world.reachable(unit, &patch.shape);
        if reachable
            && !taken(world, claims, patch.id, unit.id, false)
            && free.is_none_or(|(best, _)| far.total_cmp(&best).is_lt())
        {
            free = Some((far, patch.id));
        }
        let nearer = |(unreachable, best, _): (bool, f32, ObjectId)| {
            (!reachable).cmp(&unreachable).then(far.total_cmp(&best)).is_lt()
        };
        if any.is_none_or(nearer) {
            any = Some((!reachable, far, patch.id));
        }
    }
    Found {
        free: free.map(|(_, id)| id),
        any: any.map(|(.., id)| id),
    }
}

/// `> Patch Occupancy (one Drone per patch)`: whether `patch` is taken for worker `me` (the module docs of
/// `orders::gather`): a worker of any player mines it, or another one claimed it in this tick, or, unless `me`
/// is `waiting` at it itself, another one waits at it within reach and gets it as soon as it frees (design change
/// 0133). Among the workers waiting there, the claims alone decide: the lowest id claims first.
pub(super) fn taken(world: &World, claims: &Claims, patch: ObjectId, me: ObjectId, waiting: bool) -> bool {
    world.patch_mined(patch) || claims.by_other(patch, me) || (!waiting && world.waited_at(patch, me))
}
