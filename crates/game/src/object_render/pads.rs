//! (M6-GH2) The look of a building its aircraft land on (`Look::Pads`, `sc_sim::landing`: the Airfield,
//! `factions/gdo/structures/airfield.md > Landing slots`: "An Airfield has **4 landing slots**, one per possible
//! assignment"), over its structure body (`super::structures`): a dark pad with a pale rim on each of its landing slots
//! (`sc_sim::landing::layout`: the centres of its footprint's quarters), where its aircraft land and stand, clear of
//! the tag in the middle (`crate::kinds::object_label`). A square footprint's slots are the same however it is turned,
//! so the pads need only its type. **Placeholder (not in the design: the pads' look, until the art direction,
//! Q0001)**.

use bevy::prelude::*;
use sc_sim::ObjectKind;

use super::structures::LOOK_Z;
use crate::shapes::ColorMesh;

/// A pad's colour, and its rim's.
const PAD_COLOR: Color = Color::srgb(0.16, 0.16, 0.18);
const RIM_COLOR: Color = Color::srgb(0.92, 0.92, 0.86);
/// A pad's half side, space units: a little more than a Hornet's (its Silhouette 48 × 48 su), and its rim's width.
const PAD_HALF_SU: f32 = 28.0;
const RIM_SU: f32 = 3.0;

/// Draws the pads of a `kind` building on its roof (module docs); none for a kind without landing slots.
pub(super) fn draw(mesh: &mut ColorMesh, kind: ObjectKind) {
    let support = kind.def().structure.as_ref().and_then(|structure| structure.airfield);
    let (rim, pad) = (Vec2::splat(PAD_HALF_SU), Vec2::splat(PAD_HALF_SU - RIM_SU));
    for slot in sc_sim::landing::layout(Vec2::ZERO, support) {
        mesh.rect(slot - rim, slot + rim, LOOK_Z, RIM_COLOR);
        mesh.rect(slot - pad, slot + pad, LOOK_Z + 0.001, PAD_COLOR);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The points of `kind`'s pads (`draw`).
    fn points(kind: ObjectKind) -> Vec<Vec2> {
        let mut mesh = ColorMesh::triangles();
        draw(&mut mesh, kind);
        let mesh = mesh.into_mesh();
        match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
            Some(bevy::mesh::VertexAttributeValues::Float32x3(points)) => {
                points.iter().map(|p| Vec2::new(p[0], p[1])).collect()
            }
            _ => Vec::new(),
        }
    }

    /// The Airfield's roof, 3 × 3 tiles (`super::super::structures::inner_half`), has a pad on each of its 4 slots, 48
    /// su off its centre along both axes, inside the roof and clear of its middle, where the tag is; a building without
    /// landing slots has none.
    #[test]
    fn the_airfield_has_a_pad_on_each_slot_inside_its_roof() {
        let (w, h) = ObjectKind::Airfield.def().tile_size().expect("a structure");
        let half = super::super::structures::inner_half(w, h);
        let drawn = points(ObjectKind::Airfield);
        assert!(!drawn.is_empty());
        for p in &drawn {
            assert!(p.x.abs() <= half.x && p.y.abs() <= half.y, "{p} outside {half}");
            assert!(
                p.x.abs() >= 48.0 - PAD_HALF_SU && p.y.abs() >= 48.0 - PAD_HALF_SU,
                "{p} on the tag"
            );
        }
        for quarter in [
            Vec2::new(-1.0, -1.0),
            Vec2::new(1.0, -1.0),
            Vec2::new(-1.0, 1.0),
            Vec2::new(1.0, 1.0),
        ] {
            let mine: Vec<Vec2> = drawn.iter().copied().filter(|p| p.signum() == quarter).collect();
            assert!(!mine.is_empty(), "a pad in the quarter {quarter}");
            let centre = mine.iter().copied().sum::<Vec2>() / mine.len() as f32;
            assert!(centre.distance(quarter * 48.0) < 1e-3, "on its slot: {centre}");
        }
        assert_eq!(points(ObjectKind::Generator), []);
    }
}
