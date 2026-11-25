//! Structures' placeholder visuals (until the art direction, Q0001, settles them; the design gives none): the
//! footprint as placed (turned, `Placement::w` × `Placement::h`), in its owner's colour inside a dark rim, with a
//! two-letter tag of its type on top (`crate::hud::selection_panel::object_label`: a Tunnel's names its tier). Drawn a
//! little inside the footprint, so that a selection frame (`selection::markers`, just inside the footprint) shows
//! around it, and over a map resource's whole placeholder, as a plate covers the patch it is placed onto. A Tunnel's
//! sides show their functions by its tier (`super::tunnel`). An underground structure (plan decision T32: the
//! Hideout) is a translucent outline instead ([`underground_mesh`]), drawn in a blending material
//! ([`underground_material`]): its owner sees the ground and the units over it, which walk across it; nobody else
//! sees it while it has an owner (`sc_sim::vision::hidden`).

use bevy::prelude::*;
use bevy::sprite_render::AlphaMode2d;
use sc_sim::{ObjectId, ObjectKind, SU_PER_GU};

use crate::shapes::{ColorMesh, darken};

/// How far inside the footprint the drawing starts, space units: past the selection frame's band (4 to 7 su in).
const INSET_SU: f32 = 8.0;
/// The width of the dark rim.
const RIM_SU: f32 = 5.0;
/// The drawing's z over its visual's anchor: above the highest part of a map resource's placeholder (0.002).
const BASE_Z: f32 = 0.003;
/// The z of what a structure shows on its roof (`super::tunnel`: up to 0.002 over it), over its body's fill.
pub(super) const LOOK_Z: f32 = BASE_Z + 0.002;
/// The tag's z: over the body and what its roof shows.
const TAG_Z: f32 = BASE_Z + 0.005;
/// The tag's text height, space units, on a footprint at least 2 tiles across and on a narrower one.
const TAG_SU: f32 = 22.0;
const SMALL_TAG_SU: f32 = 15.0;
const TAG_COLOR: Color = Color::srgb(0.97, 0.97, 0.94);
/// An underground structure's outline: how opaque its rim and its fill are (placeholders: the design says only that
/// expansions are underground and walked over, `factions/syndicate/structures/README.md > Tunnel Expansions`).
const UNDERGROUND_RIM_ALPHA: f32 = 0.85;
const UNDERGROUND_FILL_ALPHA: f32 = 0.2;

/// The tag of a structure's visual, or of its record's (`super::remembered`): a child of the visual, which the fog
/// hides with it.
#[derive(Component, Copy, Clone, Debug, PartialEq, Eq)]
pub struct StructureTag(pub ObjectId);

/// Whether objects of `kind` are underground structures (`StructureDef::underground`, plan decision T32).
pub fn underground(kind: ObjectKind) -> bool {
    kind.def().structure.as_ref().is_some_and(|s| s.underground)
}

/// How far a structure `w` × `h` tiles as placed is drawn from its centre: its footprint, less the inset.
pub(super) fn drawn_half(w: u32, h: u32) -> Vec2 {
    Vec2::new(w as f32, h as f32) * (SU_PER_GU / 2.0) - Vec2::splat(INSET_SU)
}

/// How far its fill reaches from its centre, inside the rim: where its roof shows things (`super::tunnel`).
pub(super) fn inner_half(w: u32, h: u32) -> Vec2 {
    drawn_half(w, h) - Vec2::splat(RIM_SU)
}

/// A structure `w` × `h` tiles as placed, in `color`, for more to be drawn on.
pub(super) fn structure_shape(w: u32, h: u32, color: Color) -> ColorMesh {
    let (half, inner) = (drawn_half(w, h), inner_half(w, h));
    let srgba = color.to_srgba();
    let mut mesh = ColorMesh::triangles();
    mesh.rect(-half, half, BASE_Z, darken(srgba, 0.4).into());
    mesh.rect(-inner, inner, BASE_Z + 0.001, darken(srgba, 0.8).into());
    mesh
}

/// A structure `w` × `h` tiles as placed, in `color`.
pub(super) fn structure_mesh(w: u32, h: u32, color: Color) -> Mesh {
    structure_shape(w, h, color).into_mesh()
}

/// An underground structure `w` × `h` tiles as placed, in `color`: its rim, nearly opaque, around a faint fill, for a
/// blending material ([`underground_material`]).
pub(super) fn underground_mesh(w: u32, h: u32, color: Color) -> Mesh {
    let (half, inner) = (drawn_half(w, h), inner_half(w, h));
    let rim = color.to_srgba().with_alpha(UNDERGROUND_RIM_ALPHA);
    let mut mesh = ColorMesh::triangles();
    // The rim as four bands, so that nothing is drawn twice under the fill.
    mesh.rect(Vec2::new(-half.x, inner.y), half, BASE_Z, rim.into());
    mesh.rect(-half, Vec2::new(half.x, -inner.y), BASE_Z, rim.into());
    mesh.rect(
        Vec2::new(-half.x, -inner.y),
        Vec2::new(-inner.x, inner.y),
        BASE_Z,
        rim.into(),
    );
    mesh.rect(
        Vec2::new(inner.x, -inner.y),
        Vec2::new(half.x, inner.y),
        BASE_Z,
        rim.into(),
    );
    let fill = darken(color.to_srgba(), 0.8).with_alpha(UNDERGROUND_FILL_ALPHA);
    mesh.rect(-inner, inner, BASE_Z, fill.into());
    mesh.into_mesh()
}

/// The material of an underground structure's visual: the vertex colours, alpha included, blended over what lies
/// below, `dim` × as bright (1 live, less for a record's: `super::remembered`).
pub(super) fn underground_material(dim: f32) -> ColorMaterial {
    ColorMaterial {
        color: Color::linear_rgb(dim, dim, dim),
        alpha_mode: AlphaMode2d::Blend,
        ..default()
    }
}

/// The tag of object `id`, a structure `w` × `h` tiles as placed that shows `label`, its text in `dim` × the tag
/// colour.
pub(super) fn tag(id: ObjectId, label: &'static str, (w, h): (u32, u32), dim: f32) -> impl Bundle {
    let size = if w.min(h) >= 2 { TAG_SU } else { SMALL_TAG_SU };
    let color = darken(TAG_COLOR.to_srgba(), dim);
    (
        Name::new("Structure tag"),
        StructureTag(id),
        Text2d::new(label),
        TextFont::from_font_size(FontSize::Px(size)),
        TextColor(color.into()),
        Transform::from_xyz(0.0, 0.0, TAG_Z),
    )
}
