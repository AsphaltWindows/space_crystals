//! (M6-CM) The channelling stance (`factions/cults/units/disciple.md > Spells > Mirage`: "when the parent Disciple
//! **starts channelling Shroud**, its illusions **play the same channelling animation**, but make **no cloud** (so a
//! watching enemy can tell which one is real while it channels)"): a ring at the feet of every unit that stands
//! channelling, its current order a Shroud (`sc_sim::shroud::stands_channelling`), a Disciple that channels and an
//! illusion that mirrors its parent's channel drawn alike, so that only the cloud tells them apart. Every player who
//! sees the unit (`crate::fog::Sight::sees`) sees it, as its body. A violet ring just outside the unit's silhouette,
//! which swells and settles once a second by the sim's ticks plus the render alpha, so that a QA screenshot of a tick
//! shows it alike. **Placeholder (not in the design: the animation's look; Q0001)**: the Disciple had no channelling
//! animation of its own before its illusions needed one to mirror.
//!
//! One vertex-coloured, alpha-blended mesh in the units' decals layer, under the units and over the structures they
//! stand on, rebuilt every frame while a unit stands so and hidden while none does: no entity per unit
//! (`ARCHITECTURE.md > Client architecture > Visuals`).

use std::f32::consts::TAU;

use bevy::prelude::*;
use sc_sim::{ObjectId, Shape, Sim, TICKS_PER_SECOND};

use crate::ClientMode;
use crate::coords::{DrawLayer, drawn_at_height, view_to_world};
use crate::fog::{LocalSight, Sight};
use crate::shapes::ColorMesh;
use crate::sim_driver::RenderAlpha;

/// The ring's colour and opacity, its width and how far outside the unit's silhouette it stands, su, and how much it
/// swells, a share of its radius (placeholders, Q0001).
const COLOR: Srgba = Srgba::rgb(0.62, 0.48, 0.95);
const ALPHA: f32 = 0.85;
const WIDTH: f32 = 3.0;
const MARGIN: f32 = 5.0;
const SWELL: f32 = 0.15;
/// The segments of a ring.
const SEGMENTS: usize = 32;
/// Over the selection rings in the units' decals layer, whose depth offsets stay under 66 for any map
/// (`crate::coords::depth_offset`).
const OVER_DECALS_Z: f32 = 80.0;

/// The stance's mesh entity.
#[derive(Component)]
pub struct StanceMesh;

/// One unit's ring: centred on `center` (Bevy world space), `radius` su across its middle.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct StanceRing {
    pub id: ObjectId,
    pub center: Vec2,
    pub radius: f32,
}

/// The rings to draw (module docs): one at the feet of each unit that stands channelling among those `sight` sees, at
/// its interpolated position, `alpha` of the way through the tick, in the ids' order.
pub fn stance_rings(sim: &Sim, sight: &Sight, alpha: f32) -> Vec<StanceRing> {
    let phase = (sim.tick() as f32 + alpha) / TICKS_PER_SECOND as f32 * TAU;
    let swell = 1.0 + SWELL * (0.5 - 0.5 * phase.cos());
    (sim.objects())
        .filter(|o| sc_sim::shroud::stands_channelling(o) && sight.sees(o))
        .map(|o| {
            let ground = o.prev_pos.lerp(o.pos, alpha);
            let (lo, hi) = Shape::of(o).bounds();
            StanceRing {
                id: o.id,
                center: view_to_world(drawn_at_height(ground, sim.map().height_at(ground))),
                radius: ((hi - lo).max_element() / 2.0 + MARGIN) * swell,
            }
        })
        .collect()
}

/// Draws `ring`, at `z`.
fn ring_shape(mesh: &mut ColorMesh, ring: StanceRing, z: f32) {
    let color: Color = COLOR.with_alpha(ALPHA).into();
    let point = |i: usize, radius: f32| ring.center + radius * Vec2::from_angle(i as f32 * TAU / SEGMENTS as f32);
    let (inner, outer) = (ring.radius - WIDTH / 2.0, ring.radius + WIDTH / 2.0);
    for i in 0..SEGMENTS {
        let corners = [
            point(i, inner),
            point(i, outer),
            point(i + 1, outer),
            point(i + 1, inner),
        ];
        mesh.quad(corners.map(|p| p.extend(z)), color);
    }
}

pub(super) fn spawn_stance_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // A mesh without vertices makes the renderer warn: a zero-area stand-in until the first ring.
    let mut placeholder = ColorMesh::triangles();
    placeholder.rect(Vec2::ZERO, Vec2::ZERO, 0.0, Color::NONE);
    commands.spawn((
        Name::new("Channelling stances"),
        StanceMesh,
        Mesh2d(meshes.add(placeholder.into_mesh())),
        // White and alpha-blended (`ColorMaterial::default`): the vertex colours fade the rings.
        MeshMaterial2d(materials.add(ColorMaterial::default())),
        Transform::IDENTITY,
        Visibility::Hidden,
    ));
}

/// Rebuilds the stances' mesh in a game (module docs); hidden while no unit the local player sees stands channelling.
pub(super) fn update_stance_mesh(
    sight: LocalSight,
    mode: Res<ClientMode>,
    alpha: Res<RenderAlpha>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut entity: Query<(&Mesh2d, &mut Visibility), With<StanceMesh>>,
) {
    let Ok((mesh2d, mut visibility)) = entity.single_mut() else {
        return;
    };
    let rings = match mode.is_game() {
        true => stance_rings(sight.sim(), &sight.get(), alpha.0),
        false => Vec::new(),
    };
    if rings.is_empty() {
        visibility.set_if_neq(Visibility::Hidden);
        return;
    }
    let z = DrawLayer::UnitDecals.base_z() + OVER_DECALS_Z;
    let mut mesh = ColorMesh::triangles();
    for ring in rings {
        ring_shape(&mut mesh, ring, z);
    }
    if let Err(error) = meshes.insert(&mesh2d.0, mesh.into_mesh()) {
        error!("channelling stances rebuild failed: {error}");
    }
    visibility.set_if_neq(Visibility::Inherited);
}
