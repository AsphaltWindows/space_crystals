//! The gathering markers (`systems/controls/selection_and_hud.md > CommandIndicators > Gathering markers`; design
//! change 0201 point 4, answering Q0052): "Gather, Drop Off and Set Drop-off each have their **own distinct marker**
//! on the target object, not the Move marker", and "Like all indicators, they are shown only to the owner while the
//! unit is selected". Their looks are TBD ("an art choice"): placeholders until the art (Q0001), each with a shape and
//! a colour of its own ([`GatheringMarker`]), around the target as the player knows it (`super::targets`, slice
//! M3-K2a: a patch remembered in the fog keeps its marker on its record).
//!
//! - The **Gather marker** (`> Indicator assignments`: "Gather (sets the source: crystal patch, refinery's output) |
//!   Object | Gather marker"): four corner brackets round the source, in the GatherResources pointer's cyan, on a
//!   worker's current Gather, and on its queued ones while Shift is held. The order follows the worker from patch to
//!   patch (design change 0132), and the marker with it. No refinery is in the game yet.
//! - The **Return Resources marker** ("Drop Off (return resources to an Egress Node or refinery) | Object | Return
//!   Resources marker"): a ring with four arrowheads pointing in, in the ReturnResources pointer's gold, on a Drop
//!   Off's drop-off.
//! - The **Set Drop-off marker** ("Set Drop-off | Object | Set Drop-off marker"): a dashed ring in a violet of its
//!   own, wider than the other rings so that it shows round a Drop Off's on the same node, on the AssignedDropOff of
//!   each selected worker ([`drop_off_indicators`]). Set Drop-off Location applies at once and leaves the
//!   AssignedDropOff it sets (`sc_sim::Ability::SetDropOff`: it "only updates AssignedDropOff"), as Set Rally Point
//!   leaves a rally point; so the marker shows as a rally point's does (design change 0087), while the worker is
//!   selected, on the drop-off it has now. That is also one its first Gather assigned (`> Assigned Drop-off >
//!   Auto-assignment`), the one its InfoPanel names. A reading: the design gives the marker, not when it shows.
//!
//! An order's marker flashes as the order is given (`super::Flash`), a Gather's and a Drop Off's too. Set Drop-off
//! isn't an order and gives no flash, as a rally point gives none.

use std::collections::BTreeSet;
use std::f32::consts::TAU;

use bevy::prelude::*;
use sc_sim::Sim;

use super::{Indicator, MarkerRing, OBJECT_GAP, own_selected, ring};
use crate::fog::Sight;
use crate::selection::ControlStates;
use crate::shapes::ColorMesh;

/// The Gather marker's cyan: the GatherResources pointer's (`super::super::cursor`), the Space Crystals' shards'.
pub const GATHER_COLOR: Color = Color::srgb(0.36, 0.90, 0.98);
/// The Return Resources marker's gold: the ReturnResources pointer's, the bank's.
pub const RETURN_COLOR: Color = Color::srgb(1.0, 0.84, 0.30);
/// The Set Drop-off marker's violet: none of the colour language's (`> Color language`) nor of a pointer's.
pub const SET_DROP_OFF_COLOR: Color = Color::srgb(0.78, 0.58, 1.0);
/// A Gather marker's brackets: each arm is this share of its side of the box.
const BRACKET_ARM: f32 = 0.3;
/// A Return Resources marker's arrowheads: how far in from the ring they reach, and half their width, su.
const ARROW_DEPTH: f32 = 11.0;
const ARROW_HALF_WIDTH: f32 = 6.0;
/// How much wider a Set Drop-off marker's ring is than an Object marker's, su.
const SET_DROP_OFF_REACH: f32 = 8.0;
/// A Set Drop-off marker's ring: this many pieces, drawn two on, two off.
const DASH_SEGMENTS: usize = 48;

/// The three gathering markers (module docs). Placeholders until the art (Q0001), named for the design's.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GatheringMarker {
    /// "Gather marker": on a Gather's source.
    Gather,
    /// "Return Resources marker": on a Drop Off's drop-off.
    ReturnResources,
    /// "Set Drop-off marker": on a worker's AssignedDropOff, the one Set Drop-off sets.
    SetDropOff,
}

impl GatheringMarker {
    pub const ALL: [GatheringMarker; 3] = [
        GatheringMarker::Gather,
        GatheringMarker::ReturnResources,
        GatheringMarker::SetDropOff,
    ];

    /// Its colour (placeholders, module docs).
    pub fn color(self) -> Color {
        match self {
            GatheringMarker::Gather => GATHER_COLOR,
            GatheringMarker::ReturnResources => RETURN_COLOR,
            GatheringMarker::SetDropOff => SET_DROP_OFF_COLOR,
        }
    }
}

/// The Set Drop-off markers of the selected workers of the player whose `sight` this is (module docs): one on each
/// worker's AssignedDropOff that the player knows of (`super::targets`), in the workers' id order, each drop-off once.
pub fn drop_off_indicators(sim: &Sim, sight: &Sight, states: &ControlStates) -> Vec<Indicator> {
    let mut seen = BTreeSet::new();
    own_selected(sim, states, sight.player())
        .filter_map(|o| o.worker.as_ref()?.drop_off)
        .filter(|&id| sight.target_view(sim, id).is_known() && seen.insert(id))
        .map(|id| Indicator::Gathering {
            id,
            marker: GatheringMarker::SetDropOff,
        })
        .collect()
}

/// Draws `marker` round its target as `at` places it (`super::marker_ring`), centred on `center` (Bevy world space,
/// its z the indicators'), `grow` su wider than at rest, its lines `width` wide.
pub(super) fn draw(
    mesh: &mut ColorMesh,
    marker: GatheringMarker,
    center: Vec3,
    at: MarkerRing,
    grow: f32,
    width: f32,
    color: Color,
) {
    let (c, z) = (center.truncate(), center.z);
    match marker {
        GatheringMarker::Gather => brackets(mesh, c, at.half + OBJECT_GAP + grow, width, z, color),
        GatheringMarker::ReturnResources => {
            ring(mesh, c, at.radius + grow, width, z, color);
            arrowheads(mesh, c, at.radius + grow - width / 2.0, z, color);
        }
        GatheringMarker::SetDropOff => dashed_ring(mesh, c, at.radius + SET_DROP_OFF_REACH + grow, width, z, color),
    }
}

/// Four corner brackets round the box of half extents `half` centred on `center`: an L at each corner, each arm
/// [`BRACKET_ARM`] of its side, centred on the box's edge. Each L is two rectangles that don't overlap: a corner drawn
/// twice would show darker as a flash fades.
fn brackets(mesh: &mut ColorMesh, center: Vec2, half: Vec2, width: f32, z: f32, color: Color) {
    let arm = half * 2.0 * BRACKET_ARM;
    let w = width / 2.0;
    for sx in [-1.0, 1.0] {
        for sy in [-1.0, 1.0] {
            let corner = center + Vec2::new(sx * half.x, sy * half.y);
            let along_x = [
                corner + Vec2::new(sx * w, sy * w),
                corner - Vec2::new(sx * arm.x, sy * w),
            ];
            let along_y = [
                corner + Vec2::new(sx * w, -sy * w),
                corner - Vec2::new(sx * w, sy * arm.y),
            ];
            for [a, b] in [along_x, along_y] {
                mesh.rect(a.min(b), a.max(b), z, color);
            }
        }
    }
}

/// Four arrowheads pointing at `center` from the four sides, their bases on the circle of radius `base`.
fn arrowheads(mesh: &mut ColorMesh, center: Vec2, base: f32, z: f32, color: Color) {
    for d in [Vec2::X, Vec2::Y, Vec2::NEG_X, Vec2::NEG_Y] {
        let tip = center + d * (base - ARROW_DEPTH);
        let side = d.perp() * ARROW_HALF_WIDTH;
        mesh.polygon(&[center + d * base - side, center + d * base + side, tip], z, color);
    }
}

/// A dashed ring around `center`: [`DASH_SEGMENTS`] pieces of a ring, two drawn and two left out in turn.
fn dashed_ring(mesh: &mut ColorMesh, center: Vec2, radius: f32, width: f32, z: f32, color: Color) {
    let (inner, outer) = ((radius - width / 2.0).max(0.0), radius + width / 2.0);
    let dir = |i: usize| Vec2::from_angle(i as f32 * TAU / DASH_SEGMENTS as f32);
    for i in (0..DASH_SEGMENTS).filter(|i| i % 4 < 2) {
        let (d0, d1) = (dir(i), dir(i + 1));
        let corners = [
            center + d0 * inner,
            center + d0 * outer,
            center + d1 * outer,
            center + d1 * inner,
        ];
        mesh.quad(corners.map(|p| p.extend(z)), color);
    }
}
