//! (M5-H1) Air units in flight (plan decision T81's slice, H1: "air units are drawn raised, with a ground shadow, at
//! their true position, even past the edge"; placeholders until the art direction, Q0001, settles them). An air unit's
//! body is drawn in the air layer (`DrawLayer::Air`: over every ground unit and the Shroud's clouds),
//! [`FLIGHT_LIFT_LEVELS`] above the ground under it, and its shadow, a dark ellipse as wide as its silhouette
//! ([`FlightShadow`]), on that ground, under the ground units, where their selection rings are. Both are drawn at the
//! sim's `Object::pos`, past the map's edge too, where the sim counts the unit as being at the nearest edge tile
//! (`Object::map_pos`, `systems/units.md > Air Collision`): the player sees where it flies. What is drawn on a unit
//! follows its body ([`body_height`]): its picking box and selection ring (`crate::selection::pick`), its HP bar
//! (`crate::combat_effects`), the marker of an order on it (`crate::command_ui::indicators`) and its shots' ends.

use bevy::prelude::*;
use sc_sim::defs::Domain;
use sc_sim::{Map, ObjectId, ObjectKind, Placement};

use super::{ObjectVisual, object_height};
use crate::coords::{DrawLayer, lift};
use crate::shapes::ColorMesh;

/// How high an air unit's body is drawn above the ground under it, in terrain levels (`crate::coords::lift`: 16 su a
/// level, so 48 su up the screen, more than a Sparrow's own height). Placeholder (Q0001).
pub const FLIGHT_LIFT_LEVELS: f32 = 3.0;

/// A shadow's darkness: black at this alpha. Placeholder (Q0001).
const SHADOW_ALPHA: f32 = 0.35;

/// A shadow's height over its width: an ellipse, the sun high in the south (placeholder, Q0001).
const SHADOW_FLATTEN: f32 = 0.6;

/// How far under the ground units' selection rings a shadow is drawn, z within their layer.
const SHADOW_UNDER_RINGS_Z: f32 = 0.005;

/// Whether a `kind` unit flies: an air unit (`systems/units.md > Air Collision`, its base's Domain), the Sparrow's
/// Glider among them.
pub fn flies(kind: ObjectKind) -> bool {
    (kind.def().unit.as_ref()).is_some_and(|unit| unit.base_def().domain == Domain::Air)
}

/// How far above the ground an object of `kind` is drawn, in terrain levels: [`FLIGHT_LIFT_LEVELS`] for an air unit,
/// nothing for anything else.
pub fn flight_lift(kind: ObjectKind) -> f32 {
    if flies(kind) { FLIGHT_LIFT_LEVELS } else { 0.0 }
}

/// The height an object of `kind` at `pos` is drawn at, in levels: the ground's (`super::object_height`), an air
/// unit's lifted by its flight ([`flight_lift`]).
pub fn body_height(map: &Map, kind: ObjectKind, pos: Vec2, placement: Option<&Placement>) -> f32 {
    object_height(map, pos, placement) + flight_lift(kind)
}

/// The shadow of an air unit's visual: a child of the visual, which the fog hides with it, kept on the ground under
/// it ([`place_shadows`]).
#[derive(Component, Copy, Clone, Debug, PartialEq, Eq)]
pub struct FlightShadow(pub ObjectId);

/// A `kind` unit's shadow: a black ellipse at [`SHADOW_ALPHA`], as wide as its silhouette, [`SHADOW_FLATTEN`] as tall.
pub(super) fn shadow_mesh(kind: ObjectKind) -> Mesh {
    let half = kind.def().half_extents().unwrap_or(Vec2::splat(12.0));
    let outline: Vec<Vec2> = (0..20)
        .map(|i| {
            let a = i as f32 * std::f32::consts::TAU / 20.0;
            Vec2::new(half.x * a.cos(), half.x * SHADOW_FLATTEN * a.sin())
        })
        .collect();
    let mut mesh = ColorMesh::triangles();
    mesh.polygon(&outline, 0.0, Color::srgba(0.0, 0.0, 0.0, SHADOW_ALPHA));
    mesh.into_mesh()
}

/// A shadow's transform under a visual turned by `rotation`: down the screen by the flight's lift, back to the ground
/// it flies over, and down into the layer of the ground units' selection rings (`DrawLayer::UnitDecals`, y-sorted as
/// its unit is); its own rotation undoes the visual's, so that it never turns round its body.
pub(super) fn shadow_transform(rotation: Quat) -> Transform {
    let back = rotation.inverse();
    let below = DrawLayer::UnitDecals.base_z() - DrawLayer::Air.base_z() - SHADOW_UNDER_RINGS_Z;
    Transform::from_translation(back * Vec3::new(0.0, -lift(FLIGHT_LIFT_LEVELS), below)).with_rotation(back)
}

/// Keeps every shadow on the ground under its visual, after the visuals are placed (`super::place_object_visuals`).
pub(super) fn place_shadows(
    visuals: Query<&Transform, (With<ObjectVisual>, Without<FlightShadow>)>,
    mut shadows: Query<(&ChildOf, &mut Transform), With<FlightShadow>>,
) {
    for (parent, mut transform) in &mut shadows {
        let Ok(visual) = visuals.get(parent.parent()) else {
            continue;
        };
        transform.set_if_neq(shadow_transform(visual.rotation));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coords::{drawn_at_height, ground_translation};

    /// A Sparrow flies and nothing else of its faction does; its body is drawn 48 su above the ground, and its shadow,
    /// turned or not, lands where the ground under it is drawn, in the rings' layer.
    #[test]
    fn a_flying_unit_s_shadow_lies_on_the_ground_under_its_raised_body() {
        assert!(flies(ObjectKind::Sparrow));
        assert!(!flies(ObjectKind::Militant) && !flies(ObjectKind::HandOfHeaven) && !flies(ObjectKind::Airstrip));
        assert_eq!(lift(flight_lift(ObjectKind::Sparrow)), 48.0);
        let (ground, height) = (Vec2::new(300.0, 200.0), 2.0);
        let body = ground_translation(DrawLayer::Air, ground, height + FLIGHT_LIFT_LEVELS);
        let under = ground_translation(DrawLayer::UnitDecals, ground, height) - Vec3::Z * SHADOW_UNDER_RINGS_Z;
        for facing in [0.0, 1.0, -2.5] {
            let rotation = Quat::from_rotation_z(facing);
            let visual = Transform::from_translation(body).with_rotation(rotation);
            let shadow = visual * shadow_transform(rotation);
            assert!(
                shadow.translation.distance(under) < 1e-3,
                "{} vs {under}",
                shadow.translation
            );
            assert!(shadow.rotation.angle_between(Quat::IDENTITY) < 1e-5);
        }
        assert_eq!(drawn_at_height(ground, height).x, body.x);
    }
}
