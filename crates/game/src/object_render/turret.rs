//! A structure that attacks (plan decision T49: the Laser Turret, slice M4-S1), its placeholder look until the art
//! direction (Q0001) settles it; the design gives none. A structure's footprint (`super::structures`) with a round
//! head on its roof ([`draw_head`]) and a barrel ([`TurretBarrel`]) that turns with the turret's facing, which the sim
//! turns at once to face what it fires at (plan decision T49: "A structure has no turn rate (it fires in any
//! direction)"; `sc_sim::combat`), interpolated between ticks as a unit's facing tick is ([`place_barrels`]). A site
//! has neither (`super::site`); a record of a turret has its head and no barrel (`super::remembered`: where the barrel
//! pointed isn't remembered).

use bevy::prelude::*;
use sc_sim::ObjectId;

use super::structures::LOOK_Z;
use crate::coords::{facing_to_rotation, lerp_facing};
use crate::shapes::{ColorMesh, darken};
use crate::sim_driver::{RenderAlpha, SimRes};

/// The barrel of a turret's visual: a child of the visual, turned by the object's facing.
#[derive(Component, Copy, Clone, Debug, PartialEq, Eq)]
pub struct TurretBarrel(pub ObjectId);

/// The head's radius, space units: well inside a 1x1 footprint's fill (19 su from its centre).
const HEAD_SU: f32 = 13.0;
/// The width of the head's dark rim.
const HEAD_RIM_SU: f32 = 3.0;
/// The barrel along local +x, space units: from inside the head to a little past the footprint's edge (32 su).
const BARREL_FROM_SU: f32 = 6.0;
const BARREL_TO_SU: f32 = 36.0;
const BARREL_WIDTH_SU: f32 = 8.0;
/// The barrel's dark casing and its light core (placeholder colours).
const BARREL_CASING: Color = Color::srgb(0.10, 0.11, 0.14);
const BARREL_CORE: Color = Color::srgb(0.78, 0.80, 0.84);

/// A turret's head on its roof, in shades of `color`, its owner's: a dark disc with a darker rim, under the tag
/// (`super::structures::tag`), which stays legible on it.
pub(super) fn draw_head(mesh: &mut ColorMesh, color: Color) {
    let srgba = color.to_srgba();
    let disc = |radius: f32| -> Vec<Vec2> {
        (0..20)
            .map(|i| Vec2::from_angle(i as f32 * std::f32::consts::TAU / 20.0) * radius)
            .collect()
    };
    mesh.polygon(&disc(HEAD_SU), LOOK_Z, darken(srgba, 0.25).into());
    mesh.polygon(&disc(HEAD_SU - HEAD_RIM_SU), LOOK_Z + 0.0005, darken(srgba, 0.5).into());
}

/// The barrel along local +x, which its visual's rotation turns to the turret's facing: a light core in a dark casing,
/// over the head.
fn draw_barrel(mesh: &mut ColorMesh) {
    let (from, to) = (Vec2::new(BARREL_FROM_SU, 0.0), Vec2::new(BARREL_TO_SU, 0.0));
    mesh.segment(from, to, BARREL_WIDTH_SU, LOOK_Z + 0.001, BARREL_CASING);
    let core_to = to - Vec2::new(2.0, 0.0);
    mesh.segment(from, core_to, BARREL_WIDTH_SU - 4.0, LOOK_Z + 0.0015, BARREL_CORE);
}

/// The barrel alone, for its own entity ([`TurretBarrel`]).
pub(super) fn barrel_mesh() -> Mesh {
    let mut mesh = ColorMesh::triangles();
    draw_barrel(&mut mesh);
    mesh.into_mesh()
}

/// A turret's head and its barrel pointing east (local +x), on a structure's mesh: a turret's placeholder shape, where
/// no object's facing turns it (`super::object_mesh`: the map editor's markers).
pub(super) fn draw_fixed(mesh: &mut ColorMesh, color: Color) {
    draw_head(mesh, color);
    draw_barrel(mesh);
}

/// Turns each barrel to its turret's facing, interpolated between the last tick's and this one's
/// (`ARCHITECTURE.md > Client architecture > Visuals > Interpolation`), as `super::place_object_visuals` does a
/// unit's facing tick.
pub(super) fn place_barrels(
    sim: Res<SimRes>,
    alpha: Res<RenderAlpha>,
    mut barrels: Query<(&TurretBarrel, &mut Transform)>,
) {
    for (barrel, mut transform) in &mut barrels {
        let Some(object) = sim.object(barrel.0) else {
            continue;
        };
        let facing = lerp_facing(object.prev_facing, object.facing, alpha.0);
        transform.set_if_neq(Transform::from_rotation(facing_to_rotation(facing)));
    }
}
