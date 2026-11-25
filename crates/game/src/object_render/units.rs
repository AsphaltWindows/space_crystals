//! Units' placeholder visuals (until the art direction, Q0001, settles them; `ARCHITECTURE.md > Client architecture >
//! Visuals`): a shape of its own per kind, its silhouette's size, in its owner's colour, with a tick showing its
//! facing. A body turns with its facing only if it is round ([`body_turns`]); the Colonists' units, the Peacekeepers
//! and the Agent carry a two-letter tag of their type ([`UnitTag`]), drawn above the command indicators and upright.
//! A Peacekeeper's raised Riot Shield is a pale arc in front of it ([`riot_shield_mesh`]), the Peacekeeper
//! (Shielded)'s too: the two types share their body (design change 0173).

use bevy::prelude::*;
use sc_sim::{ObjectId, ObjectKind};

use super::{Look, ObjectVisual, UnitShape};
use crate::command_ui::indicators::INDICATORS_Z;
use crate::coords::DrawLayer;
use crate::kinds::short_label;
use crate::shapes::{ColorMesh, darken};

/// The placeholder shape of a `kind` unit, its look's (`crate::kinds`): round for the Peacekeeper, shielded or not,
/// an octagon for the Drone (a HoverVehicle), and a square for the HeavyInfantry: the Guard, the Agent and the
/// Legionnaires.
fn shape(kind: ObjectKind) -> UnitShape {
    Look::of(kind).unit_shape().unwrap_or(UnitShape::Square)
}

/// Whether a `kind` unit's body turns with its facing: only a round one does. A square or an octagon stays
/// axis-aligned, like the unit's collision box (`sc_sim::ObjectDef::half_extents`), and only its tick turns
/// ([`super::FacingTick`]): a turned square makes crowded units look as if they overlapped.
pub(super) fn body_turns(kind: ObjectKind) -> bool {
    shape(kind) == UnitShape::Round
}

/// Whether a `kind` unit carries its type's tag ([`UnitTag`]), by its look (`crate::kinds`): the Colonists' units,
/// the Peacekeeper's two types (slice M3-J3: M3-K2's walkthrough found the Peacekeepers unlabelled on the map), and
/// the Agent (slice M4-A). The Legionnaire's two forms share one silhouette
/// (`factions/colonists/units/weapon_conversion.md`), as the Peacekeeper's two types share their body (design change
/// 0173); a square is the Guard's too, which goes without, and the Agent's, which its AG tells apart.
pub(super) fn tagged(kind: ObjectKind) -> bool {
    Look::of(kind).tagged()
}

/// The tag of a unit's visual ([`tagged`]): a child of the visual, which the fog hides with it. It is drawn above the
/// command indicators ([`tag_z`]) and stays upright, on a body that turns too ([`upright_tags`]).
#[derive(Component, Copy, Clone, Debug, PartialEq, Eq)]
pub struct UnitTag(pub ObjectId);

/// The tag's text height, space units: two letters inside a 36-su body, and across most of a Peacekeeper's 24.
const TAG_SU: f32 = 14.0;
const TAG_COLOR: Color = Color::srgb(0.97, 0.97, 0.94);
/// How far above the command indicators a tag is drawn (at its unit's ground y), in the overlays layer.
const TAG_OVER_INDICATORS: f32 = 0.1;

/// A tag's z over its visual's: up into the overlays layer, just above the command indicators
/// (`command_ui::indicators::INDICATORS_Z`), y-sorted as its unit is (slice M3-J3: M3-K2's walkthrough found the
/// Bio Portal's rally marker, where its Rifle Legionnaire stood, hiding the unit's tag). So a marker on a unit never
/// hides its tag; the tags draw above every unit's body, the combat effects too further south. A structure's tag
/// stays with its structure, under the units that cross it (`super::structures`).
fn tag_z() -> f32 {
    DrawLayer::Overlays.base_z() - DrawLayer::GroundUnits.base_z() + INDICATORS_Z + TAG_OVER_INDICATORS
}

/// The tag of object `id`, a `kind` unit: its type's short label ([`short_label`]), above the indicators ([`tag_z`]).
pub(super) fn tag(id: ObjectId, kind: ObjectKind) -> impl Bundle {
    (
        Name::new("Unit tag"),
        UnitTag(id),
        Text2d::new(short_label(kind)),
        TextFont::from_font_size(FontSize::Px(TAG_SU)),
        TextColor(TAG_COLOR),
        Transform::from_xyz(0.0, 0.0, tag_z()),
    )
}

/// Keeps every tag upright: its rotation undoes its visual's, which turns with a round body's facing (a Peacekeeper's,
/// [`body_turns`]) and is none otherwise. After the visuals are placed (`super::place_object_visuals`).
pub(super) fn upright_tags(
    visuals: Query<&Transform, (With<ObjectVisual>, Without<UnitTag>)>,
    mut tags: Query<(&ChildOf, &mut Transform), With<UnitTag>>,
) {
    for (parent, mut transform) in &mut tags {
        let Ok(visual) = visuals.get(parent.parent()) else {
            continue;
        };
        let upright = Transform::from_xyz(0.0, 0.0, tag_z()).with_rotation(visual.rotation.inverse());
        transform.set_if_neq(upright);
    }
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
