//! Where the indicators show a target object (slice M3-K2a): `core/vision.md > Explored > Targeting remembered
//! objects` (design change 0139): "Nothing about the real state leaks: the cursor, the command and any rally marker
//! treat the remembered object as present until the player sees otherwise." Every marker and route leg that names an
//! object goes by what the local player knows of it (`crate::fog::Sight::target_view`):
//!
//! - while the player sees the object, where the object is drawn;
//! - while the player remembers it, where its record is drawn: its last-known position and footprint, wherever the
//!   object really is (a Beacon whose deployment was cancelled in the fog walks away as a Drone) and whether or not
//!   it still exists;
//! - nowhere while the player knows of nothing there: a unit out of sight, or an object seen gone.

use bevy::prelude::*;
use sc_sim::{ObjectId, Placement, SU_PER_GU, Sim, TargetView};

use crate::fog::Sight;
use crate::object_render::{body_height, object_height};

/// Where `sight` shows object `id`, ground space: where it is while the player sees it, its last-known position while
/// the player remembers it; none while the player knows of nothing there.
pub(super) fn shown_at(sim: &Sim, sight: &Sight, id: ObjectId) -> Option<Vec2> {
    match sight.target_view(sim, id) {
        TargetView::Visible => sim.object(id).map(|o| o.pos),
        TargetView::Remembered { position, .. } => Some(position),
        TargetView::Gone => None,
    }
}

/// The box an Object marker surrounds, as `sight` shows object `id` at the render `alpha`: its centre on the ground,
/// the height it is drawn at, and its half extents, su. A seen object's footprint, or its type's box where it is
/// drawn; a remembered one's record's footprint. None while the player knows of nothing there.
pub(super) fn object_box(sim: &Sim, sight: &Sight, id: ObjectId, alpha: f32) -> Option<(Vec2, f32, Vec2)> {
    match sight.target_view(sim, id) {
        TargetView::Visible => {
            let o = sim.object(id)?;
            let pos = o.prev_pos.lerp(o.pos, alpha);
            let height = body_height(sim.map(), o, pos);
            let (center, half) = match &o.placement {
                Some(footprint) => footprint_box(footprint),
                None => (pos, o.def().half_extents().unwrap_or(Vec2::splat(SU_PER_GU / 4.0))),
            };
            Some((center, height, half))
        }
        TargetView::Remembered { position, footprint } => {
            let height = object_height(sim.map(), position, Some(&footprint));
            let (center, half) = footprint_box(&footprint);
            Some((center, height, half))
        }
        TargetView::Gone => None,
    }
}

/// A footprint's centre (ground space) and half extents, su.
fn footprint_box(footprint: &Placement) -> (Vec2, Vec2) {
    let size = Vec2::new(footprint.w as f32, footprint.h as f32);
    (footprint.center(), size * SU_PER_GU / 2.0)
}
