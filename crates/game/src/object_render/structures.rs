//! Structures' placeholder visuals (until the art direction, Q0001, settles them; the design gives none): the
//! footprint as placed (turned, `Placement::w` × `Placement::h`), in its owner's colour inside a dark rim, with a
//! two-letter tag of its type ([`short_label`]) on top. Drawn a little inside the footprint, so that a selection
//! frame (`selection::markers`, just inside the footprint) shows around it, and over a map resource's whole
//! placeholder, as a plate covers the patch it is placed onto.

use bevy::prelude::*;
use sc_sim::{ObjectId, ObjectKind, SU_PER_GU};

use crate::hud::selection_panel::short_label;
use crate::shapes::{ColorMesh, darken};

/// How far inside the footprint the drawing starts, space units: past the selection frame's band (4 to 7 su in).
const INSET_SU: f32 = 8.0;
/// The width of the dark rim.
const RIM_SU: f32 = 5.0;
/// The drawing's z over its visual's anchor: above the highest part of a map resource's placeholder (0.002).
const BASE_Z: f32 = 0.003;
/// The tag's text height, space units, on a footprint at least 2 tiles across and on a narrower one.
const TAG_SU: f32 = 22.0;
const SMALL_TAG_SU: f32 = 15.0;
const TAG_COLOR: Color = Color::srgb(0.97, 0.97, 0.94);

/// The tag of a structure's visual, or of its record's (`super::remembered`): a child of the visual, which the fog
/// hides with it.
#[derive(Component, Copy, Clone, Debug, PartialEq, Eq)]
pub struct StructureTag(pub ObjectId);

/// A structure `w` × `h` tiles as placed, in `color`.
pub(super) fn structure_mesh(w: u32, h: u32, color: Color) -> Mesh {
    let half = Vec2::new(w as f32, h as f32) * (SU_PER_GU / 2.0) - Vec2::splat(INSET_SU);
    let inner = half - Vec2::splat(RIM_SU);
    let srgba = color.to_srgba();
    let mut mesh = ColorMesh::triangles();
    mesh.rect(-half, half, BASE_Z, darken(srgba, 0.4).into());
    mesh.rect(-inner, inner, BASE_Z + 0.001, darken(srgba, 0.8).into());
    mesh.into_mesh()
}

/// The tag of object `id`, a `kind` structure `w` × `h` tiles as placed, its text in `dim` × the tag colour.
pub(super) fn tag(id: ObjectId, kind: ObjectKind, (w, h): (u32, u32), dim: f32) -> impl Bundle {
    let size = if w.min(h) >= 2 { TAG_SU } else { SMALL_TAG_SU };
    let color = darken(TAG_COLOR.to_srgba(), dim);
    (
        Name::new("Structure tag"),
        StructureTag(id),
        Text2d::new(short_label(kind)),
        TextFont::from_font_size(FontSize::Px(size)),
        TextColor(color.into()),
        Transform::from_xyz(0.0, 0.0, BASE_Z + 0.002),
    )
}
