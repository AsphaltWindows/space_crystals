//! **What the worker goes by** (design change 0139, `core/vision.md > Explored > Targeting remembered objects`:
//! "Nothing about the real state leaks"): its patch as its owner knows it (`World::known_patch`, the target view of
//! `crate::targets`), as every order on an object does. A patch the owner sees is the patch as it is. One out of its
//! sight is where the owner remembers it, standing or not, mined by others or not, plated since or not; and so are
//! the patches the worker looks round among and the objects it stands clear of on its way (`World::known_fixed`).
//! The map's patches are known from the start (design change 0159). A Gather on a patch out of its owner's sight
//! notes where the owner remembers it (`state::Gather::spot`), so that if the patch turns out gone once the worker
//! sees its place, the order goes on as for a patch that ran out: there, it looks round (0139: "Gather: the redirect
//! rules").

use crate::orders::OrderKind;
use crate::orders::state::Gather;
use crate::orders::world::{KnownPatch, Unit, World};
use crate::shape::Shape;

use super::legs::spot_shape;

/// The order's patch, as the worker's owner knows it (module docs).
pub(super) enum Site {
    /// A patch workers can gather: one the owner sees, or one it remembers out of its sight (design change 0139),
    /// which counts as standing where the owner remembers it until the owner sees its place.
    Patch(KnownPatch),
    /// The shape of a patch that is gone for the workers: one under a plate (0137), or one gone to the owner, where
    /// it was (design change 0134, `state::Gather::spot`).
    Gone(Shape),
    /// Nothing to head for: the order names no patch the owner knows of (a Drop Off, or a save edited by hand).
    Nowhere,
}

/// The order's patch as the worker's owner knows it now ([`Site`]).
pub(super) fn site(kind: OrderKind, state: &Gather, unit: &Unit, world: &World) -> Site {
    let OrderKind::Gather(patch) = kind else {
        return Site::Nowhere;
    };
    match world.known_patch(unit.owner, patch) {
        Some(known) if !known.covered => Site::Patch(known),
        Some(known) => Site::Gone(known.shape),
        None => state.spot.map_or(Site::Nowhere, |spot| Site::Gone(spot_shape(spot))),
    }
}

/// Design change 0139 (module docs): a Gather notes where its patch is while the patch is out of its owner's sight,
/// so that it heads on there if the patch turns out gone (`state::Gather::spot`). In sight, it goes by the patch. A
/// patch gone to the owner leaves the spot as it is: where the patch was (design change 0134).
pub(super) fn note_spot(kind: OrderKind, state: &mut Gather, unit: &Unit, world: &World) {
    if let OrderKind::Gather(patch) = kind
        && let Some(known) = world.known_patch(unit.owner, patch)
    {
        state.spot = (!known.seen).then_some(known.pos);
    }
}
