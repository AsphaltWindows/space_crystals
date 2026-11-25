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
/// visible, or remembered from earlier sight", as its owner knows them: `World::known_patches_near`, design change
/// 0139; "a Drone never finds a patch hidden in fog", but the map's are known from the start, 0159) and "not covered
/// by a structure", all but `except`, the one it comes from. "The nearest unoccupied patch to itself" is the nearest
/// centre to centre, the lowest id among equals, of those not taken that it can reach: design change 0141 treats a
/// patch it can't reach like an occupied one (`World::reachable`, by the grids its owner knows; a wall of units shows
/// only when a leg ends short of the patch). A patch in sight is taken as [`taken`] says; one out of it only if
/// another of its owner's workers claimed it in this tick: what others do there, the owner doesn't see.
pub(super) fn search(world: &World, claims: &Claims, unit: &Unit, def: &WorkerDef, except: Option<ObjectId>) -> Found {
    let radius = gu(def.search_radius);
    // (distance squared, id) of the nearest free patch, and (out of reach, distance squared, id) of the nearest.
    let mut free: Option<(f32, ObjectId)> = None;
    let mut any: Option<(bool, f32, ObjectId)> = None;
    for patch in world.known_patches_near(unit.owner, unit.pos, radius) {
        if Some(patch.id) == except || patch.covered {
            continue;
        }
        let far = patch.pos.distance_squared(unit.pos);
        // In id order, so the first of equals stays.
        let reachable = world.reachable(unit, &patch.shape);
        let taken = if patch.seen {
            taken(world, claims, def, patch.id, unit.id, false)
        } else {
            !def.shares_patches && claims.by_other_of(patch.id, unit.id, unit.owner)
        };
        if reachable && !taken && free.is_none_or(|(best, _)| far.total_cmp(&best).is_lt()) {
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
///
/// Never for a worker of a type that shares patches (`WorkerDef::shares_patches`: `factions/syndicate/units.md >
/// Agent > Gathering rules`, "Agents share patches (no occupancy rule)"). Such workers aren't among those that mine,
/// wait or claim here either (`World::patch_mined`, [`claim`]): the occupancy is the other workers' alone.
pub(super) fn taken(
    world: &World,
    claims: &Claims,
    def: &WorkerDef,
    patch: ObjectId,
    me: ObjectId,
    waiting: bool,
) -> bool {
    !def.shares_patches
        && (world.patch_mined(patch) || claims.by_other(patch, me) || (!waiting && world.waited_at(patch, me)))
}

/// `unit` claims `patch` for this tick ([`Claims`]), unless its type shares patches ([`taken`]).
pub(super) fn claim(claims: &mut Claims, def: &WorkerDef, patch: ObjectId, unit: &Unit) {
    if !def.shares_patches {
        claims.claim(patch, unit.id, unit.owner);
    }
}
