//! Units' placeholder visuals (until the art direction, Q0001, settles them; `ARCHITECTURE.md > Client architecture >
//! Visuals`): a shape of its own per kind, its silhouette's size, in its owner's colour, with a tick showing its
//! facing. A body turns with its facing only if it is round ([`body_turns`]); the Colonists' units carry a two-letter
//! tag of their type ([`UnitTag`]). A Peacekeeper's raised Riot Shield is a pale arc in front of it
//! ([`riot_shield_mesh`]).

use bevy::prelude::*;
use sc_sim::{ObjectId, ObjectKind};

use crate::hud::selection_panel::short_label;
use crate::shapes::{ColorMesh, darken};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum UnitShape {
    Round,
    Square,
    /// A square with its corners cut off.
    Octagon,
}

/// The placeholder shape of a `kind` unit: round for the Peacekeeper, an octagon for the Drone (a HoverVehicle), and
/// a square for the HeavyInfantry: the Guard and the Legionnaires.
fn shape(kind: ObjectKind) -> UnitShape {
    match kind {
        ObjectKind::Peacekeeper => UnitShape::Round,
        ObjectKind::Drone => UnitShape::Octagon,
        _ => UnitShape::Square,
    }
}

/// Whether a `kind` unit's body turns with its facing: only a round one does. A square or an octagon stays
/// axis-aligned, like the unit's collision box (`sc_sim::ObjectDef::half_extents`), and only its tick turns
/// ([`super::FacingTick`]): a turned square makes crowded units look as if they overlapped.
pub(super) fn body_turns(kind: ObjectKind) -> bool {
    shape(kind) == UnitShape::Round
}

/// Whether a `kind` unit carries its type's tag ([`UnitTag`]): the Colonists' units. The Legionnaire's two forms
/// share one silhouette (`factions/colonists/units.md > Weapon Conversion`), and a square is the Guard's too.
pub(super) fn tagged(kind: ObjectKind) -> bool {
    matches!(
        kind,
        ObjectKind::Drone | ObjectKind::SabreLegionnaire | ObjectKind::RifleLegionnaire
    )
}

/// The tag of a unit's visual ([`tagged`]): a child of the visual, which the fog hides with it, and which stays
/// upright on a body that doesn't turn.
#[derive(Component, Copy, Clone, Debug, PartialEq, Eq)]
pub struct UnitTag(pub ObjectId);

/// The tag's text height, space units: two letters inside a 36-su body.
const TAG_SU: f32 = 14.0;
const TAG_COLOR: Color = Color::srgb(0.97, 0.97, 0.94);

/// The tag of object `id`, a `kind` unit: its type's short label ([`short_label`]), over the body and its tick.
pub(super) fn tag(id: ObjectId, kind: ObjectKind) -> impl Bundle {
    (
        Name::new("Unit tag"),
        UnitTag(id),
        Text2d::new(short_label(kind)),
        TextFont::from_font_size(FontSize::Px(TAG_SU)),
        TextColor(TAG_COLOR),
        Transform::from_xyz(0.0, 0.0, 0.003),
    )
}

/// A unit (placeholders until Q0001 settles the art): its silhouette's size in a shape of its own ([`shape`]), filled
/// with its owner's colour inside a dark rim, and (`with_tick`) a light tick through its front, along local +x, which
/// the visual's rotation turns to the unit's facing ([`facing_tick`]).
pub(super) fn unit_mesh(kind: ObjectKind, color: Color, with_tick: bool) -> Mesh {
    let half = kind.def().half_extents().unwrap_or(Vec2::splat(12.0));
    let outline = |inset: f32| -> Vec<Vec2> {
        let h = half - Vec2::splat(inset);
        match shape(kind) {
            UnitShape::Round => (0..16)
                .map(|i| {
                    let a = i as f32 * std::f32::consts::TAU / 16.0;
                    Vec2::new(h.x * a.cos(), h.y * a.sin())
                })
                .collect(),
            UnitShape::Square => vec![
                Vec2::new(-h.x, -h.y),
                Vec2::new(h.x, -h.y),
                Vec2::new(h.x, h.y),
                Vec2::new(-h.x, h.y),
            ],
            UnitShape::Octagon => {
                let cut = h.min_element() * 0.35;
                vec![
                    Vec2::new(-h.x + cut, -h.y),
                    Vec2::new(h.x - cut, -h.y),
                    Vec2::new(h.x, -h.y + cut),
                    Vec2::new(h.x, h.y - cut),
                    Vec2::new(h.x - cut, h.y),
                    Vec2::new(-h.x + cut, h.y),
                    Vec2::new(-h.x, h.y - cut),
                    Vec2::new(-h.x, -h.y + cut),
                ]
            }
        }
    };
    let srgba = color.to_srgba();
    let mut mesh = ColorMesh::triangles();
    mesh.polygon(&outline(0.0), 0.0, darken(srgba, 0.45).into());
    mesh.polygon(&outline(2.5), 0.001, color);
    if with_tick {
        facing_tick(&mut mesh, kind, half);
    }
    mesh.into_mesh()
}

/// A `kind` unit's facing tick, along local +x, to just in front of its body: from its centre, or from 60% of the
/// way out on a unit with a tag ([`tagged`]), so that the tick doesn't cross the tag's letters.
fn facing_tick(mesh: &mut ColorMesh, kind: ObjectKind, half: Vec2) {
    let start = if tagged(kind) { 0.6 * half.x } else { 0.0 };
    mesh.segment(
        Vec2::new(start, 0.0),
        Vec2::new(half.x + 5.0, 0.0),
        3.0,
        0.002,
        Color::srgb(0.96, 0.96, 0.92),
    );
}

/// The facing tick of a `kind` visual whose body doesn't turn, alone.
pub(super) fn tick_mesh(kind: ObjectKind) -> Mesh {
    let mut mesh = ColorMesh::triangles();
    facing_tick(&mut mesh, kind, kind.def().half_extents().unwrap_or(Vec2::splat(12.0)));
    mesh.into_mesh()
}

/// A `kind` unit's raised Riot Shield: a pale band in a dark rim, 100° of a ring just outside its body, centred on
/// local +x, its front, over the tip of its facing tick.
pub(super) fn riot_shield_mesh(kind: ObjectKind) -> Mesh {
    let half = kind.def().half_extents().unwrap_or(Vec2::splat(12.0));
    let radius = half.x.max(half.y);
    let mut mesh = ColorMesh::triangles();
    let band = |mesh: &mut ColorMesh, inner: f32, outer: f32, z: f32, color: Color| {
        let steps = 10;
        let half_arc = 50_f32.to_radians();
        for i in 0..steps {
            let angle = |i: i32| -half_arc + 2.0 * half_arc * i as f32 / steps as f32;
            let (a, b) = (Vec2::from_angle(angle(i)), Vec2::from_angle(angle(i + 1)));
            mesh.polygon(&[a * inner, a * outer, b * outer, b * inner], z, color);
        }
    };
    band(
        &mut mesh,
        radius + 3.0,
        radius + 9.0,
        0.003,
        Color::srgb(0.10, 0.13, 0.18),
    );
    band(
        &mut mesh,
        radius + 4.0,
        radius + 8.0,
        0.004,
        Color::srgb(0.80, 0.90, 1.00),
    );
    mesh.into_mesh()
}
