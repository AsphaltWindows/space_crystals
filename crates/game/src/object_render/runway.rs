//! (M5-N7) The look of a building whose aircraft take off along it (`Look::Runway`, `sc_sim::defs::ExitRule::Runway`:
//! the Airstrip, `factions/cults/structures/airstrip.md`: "Size 2x4 (a runway)"), over its structure body
//! (`super::structures`): a dark strip down its length, with a bar across each end and a dashed centre line, which
//! stops short of the tag in the middle (`crate::kinds::object_label`) so as not to cover it. Drawn alike at both
//! ends, so that it needs only the footprint as placed: which end is B and which C, the door arrows show
//! (`crate::door_arrows`). **Placeholder (not in the design: the runway's look, until the art direction, Q0001)**.

use bevy::prelude::*;

use super::structures::LOOK_Z;
use crate::shapes::ColorMesh;

/// The strip's colour, and its markings'.
const STRIP_COLOR: Color = Color::srgb(0.16, 0.16, 0.18);
const MARK_COLOR: Color = Color::srgb(0.92, 0.92, 0.86);
/// How much of the roof's width across the strip covers.
const STRIP_SHARE: f32 = 0.6;
/// How far in from the roof's ends the strip stops, and the end bars' and the dashes' sizes, space units.
const END_MARGIN_SU: f32 = 4.0;
const BAR_SU: f32 = 4.0;
const DASH_SU: f32 = 14.0;
const DASH_GAP_SU: f32 = 10.0;
const DASH_WIDTH_SU: f32 = 3.0;
/// How far from the middle the dashes stay clear of the tag, space units.
const TAG_CLEAR_SU: f32 = 30.0;

/// Draws the runway on a roof that reaches `half` from its centre (`super::structures::inner_half`), down its longer
/// axis (module docs).
pub(super) fn draw(mesh: &mut ColorMesh, half: Vec2) {
    // The axis along the runway and the one across it, and how far the strip reaches along each.
    let (along, across) = if half.x >= half.y {
        (Vec2::X, Vec2::Y)
    } else {
        (Vec2::Y, Vec2::X)
    };
    let length = along.dot(half) - END_MARGIN_SU;
    let width = across.dot(half) * STRIP_SHARE;
    let rect = |mesh: &mut ColorMesh, from: f32, to: f32, wide: f32, z: f32, color: Color| {
        let (a, b) = (along * from - across * wide, along * to + across * wide);
        mesh.rect(a.min(b), a.max(b), z, color);
    };
    rect(mesh, -length, length, width, LOOK_Z, STRIP_COLOR);
    // Each end's bar, a bar's width in from the strip's end, and the dashes from the tag out to a bar's width short of
    // it.
    let bar = length - 2.0 * BAR_SU;
    let mut at = TAG_CLEAR_SU;
    for side in [-1.0, 1.0] {
        rect(
            mesh,
            side * bar,
            side * (length - BAR_SU),
            width - BAR_SU,
            LOOK_Z + 0.001,
            MARK_COLOR,
        );
    }
    while at + DASH_SU <= bar - BAR_SU {
        for side in [-1.0, 1.0] {
            rect(
                mesh,
                side * at,
                side * (at + DASH_SU),
                DASH_WIDTH_SU / 2.0,
                LOOK_Z + 0.001,
                MARK_COLOR,
            );
        }
        at += DASH_SU + DASH_GAP_SU;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The Airstrip's roof, 4 × 2 tiles as placed and turned (`super::super::structures::inner_half`): the strip and
    /// its markings lie inside it, down its longer axis either way, and the dashes keep clear of the tag.
    #[test]
    fn the_runway_runs_down_the_longer_axis_inside_the_roof() {
        for (w, h) in [(4, 2), (2, 4)] {
            let half = super::super::structures::inner_half(w, h);
            let mut mesh = ColorMesh::triangles();
            draw(&mut mesh, half);
            let mesh = mesh.into_mesh();
            let Some(bevy::mesh::VertexAttributeValues::Float32x3(points)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION)
            else {
                panic!("no positions");
            };
            let points: Vec<Vec2> = points.iter().map(|p| Vec2::new(p[0], p[1])).collect();
            assert!(!points.is_empty());
            let (long, short) = if w > h { (0, 1) } else { (1, 0) };
            for p in &points {
                assert!(p.x.abs() <= half.x && p.y.abs() <= half.y, "{p} outside {half}");
            }
            let reach = |axis: usize| points.iter().map(|p| p[axis].abs()).fold(0.0, f32::max);
            assert!(reach(long) > half[long] - 5.0, "down its length");
            assert!(reach(short) <= half[short] * STRIP_SHARE + 0.01, "across its width");
            // The strip's corners are at its ends, and the dashes begin clear of the tag.
            assert!(points.iter().all(|p| p[long].abs() >= TAG_CLEAR_SU), "clear of the tag");
        }
    }
}
