//! A site under construction (`sc_sim::Object::is_site`, slice M4-J1): a Tunnel an Agent builds
//! (`factions/syndicate/units/agent.md > Tunnel Construction Flow`: "the partially-built Tunnel appears at the
//! location with the Agent inside it"), an expansion a Tunnel builds (slice M4-F), a Laser Turret (slice M4-S1). It is
//! drawn by being a site, whatever its kind, and shows how far its construction has come.
//!
//! **Placeholder look** until the art (Q0001; the design gives none): the structure's footprint in its owner's colour
//! turned half way to a pale sand ([`site_color`]), braced at its corners by a scaffold ([`draw`]), without what a
//! finished structure shows on its roof (a Tunnel's sides); a progress bar inside the footprint near its south edge,
//! clear of the health bar above it (`crate::combat_effects`): a dark track drawn with the body, and its fill a child
//! of the visual ([`SiteProgress`]), as long as the construction's share done (`sc_sim::Progress::ratio`). The bar
//! shows on every site the local player sees, anyone's (placeholder reading: the design says only that enemies see the
//! site, `factions/syndicate/units/agent.md > While the Tunnel is under construction > What enemies see`).
//!
//! A site the local player remembers (`core/vision.md > Explored`: "shown in their last-known state", slice M4-I2) is
//! drawn as a site too, dimmed as every record is (`super::remembered`), its bar at the progress last seen
//! (`sc_sim::vision::LastKnown::site`), which no longer moves: the site may have grown or been finished since.

use bevy::prelude::*;
use sc_sim::ObjectId;

use super::structures::{LOOK_Z, inner_half};
use crate::shapes::ColorMesh;
use crate::sim_driver::SimRes;

/// The pale sand a site's colour turns towards, and the scaffold's colour.
const SITE_TINT: Srgba = Srgba::rgb(0.86, 0.80, 0.58);
const SCAFFOLD_COLOR: Color = Color::srgba(0.98, 0.92, 0.62, 0.85);
/// The scaffold's beams' width, su, and where each corner's brace stops: that share of the way from the centre.
const BEAM_SU: f32 = 4.0;
const BRACE_STOP: f32 = 0.4;
/// The progress bar: its track's colour, its fill's, its height and how far inside the roof's edge it lies, su.
const TRACK_COLOR: Color = Color::srgb(0.10, 0.10, 0.12);
const FILL_COLOR: Color = Color::srgb(0.98, 0.84, 0.30);
const BAR_SU: f32 = 10.0;
const BAR_INSET_SU: f32 = 8.0;
/// The fill's z over its visual's anchor: over the track, under the structure's tag.
const FILL_Z: f32 = LOOK_Z + 0.001;

/// The progress bar's fill of a site's visual: a child of the visual, which the fog hides with it (`crate::fog`).
#[derive(Component, Copy, Clone, Debug, PartialEq)]
pub struct SiteProgress {
    /// The site.
    pub id: ObjectId,
    /// The share of the construction done that the bar shows, 0 to 1.
    pub shown: f32,
    /// Whether it is the fill of the site's record (`super::remembered`), which shows the progress last seen and
    /// stays there, rather than of the live site, which [`place_site_progress`] keeps up with its construction.
    pub remembered: bool,
    /// The site's footprint as placed, tiles.
    size: (u32, u32),
}

impl SiteProgress {
    /// The fill of live site `id`, `size` tiles as placed, with nothing shown yet.
    pub(super) fn new(id: ObjectId, size: (u32, u32)) -> Self {
        Self {
            id,
            shown: 0.0,
            remembered: false,
            size,
        }
    }

    /// The fill of the record of site `id`, `size` tiles as placed, which was `ratio` done when last seen, with its
    /// transform ([`fill_transform`]).
    pub(super) fn remembered(id: ObjectId, size: (u32, u32), ratio: f32) -> (Self, Transform) {
        let fill = Self {
            id,
            shown: ratio,
            remembered: true,
            size,
        };
        (fill, fill_transform(size, ratio))
    }
}

/// The colour of a site's body, from its owner's `color`: half way to a pale sand.
pub fn site_color(color: Color) -> Color {
    color.to_srgba().mix(&SITE_TINT, 0.45).into()
}

/// Where the bar lies on a roof whose fill reaches `inner` from the centre: its west and east ends' x, and its south
/// and north edges' y.
fn bar(inner: Vec2) -> (Vec2, Vec2) {
    let a = Vec2::new(-inner.x + BAR_INSET_SU, -inner.y + BAR_INSET_SU);
    (a, Vec2::new(inner.x - BAR_INSET_SU, a.y + BAR_SU))
}

/// Draws a site's scaffold and its progress bar's track on its roof, whose fill reaches `inner` from its centre: a
/// brace from each corner along the diagonals, stopping short of the centre so that the structure's tag there stays
/// legible, and the track along its south edge.
pub(super) fn draw(mesh: &mut ColorMesh, inner: Vec2) {
    for (x, y) in [(-1.0, -1.0), (-1.0, 1.0), (1.0, -1.0), (1.0, 1.0)] {
        let from = Vec2::new(x, y) * inner;
        mesh.segment(from, from * BRACE_STOP, BEAM_SU, LOOK_Z, SCAFFOLD_COLOR);
    }
    let (a, b) = bar(inner);
    mesh.rect(a - Vec2::splat(1.5), b + Vec2::splat(1.5), LOOK_Z + 0.0005, TRACK_COLOR);
}

/// The fill's mesh: a unit square from the origin, which [`fill_transform`] stretches over the bar's done part.
pub(super) fn fill_mesh() -> Mesh {
    let mut mesh = ColorMesh::triangles();
    mesh.rect(Vec2::ZERO, Vec2::ONE, 0.0, FILL_COLOR);
    mesh.into_mesh()
}

/// Where the fill of a site `size` tiles as placed stands with `ratio` of its construction done.
pub(super) fn fill_transform((w, h): (u32, u32), ratio: f32) -> Transform {
    let (a, b) = bar(inner_half(w, h));
    let length = (b.x - a.x) * ratio.clamp(0.0, 1.0);
    Transform::from_translation(a.extend(FILL_Z)).with_scale(Vec3::new(length.max(0.001), b.y - a.y, 1.0))
}

/// Stretches each live site's fill over the share of its construction done; a record's stays as last seen.
pub(super) fn place_site_progress(sim: Res<SimRes>, mut fills: Query<(&mut SiteProgress, &mut Transform)>) {
    for (mut fill, mut transform) in &mut fills {
        if fill.remembered {
            continue;
        }
        let construction = sim.object(fill.id).and_then(|o| o.construction.as_ref());
        let Some(ratio) = construction.map(|c| c.progress.ratio()) else {
            continue;
        };
        if fill.shown != ratio {
            fill.shown = ratio;
            *transform = fill_transform(fill.size, ratio);
        }
    }
}
