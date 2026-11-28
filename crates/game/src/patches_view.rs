//! (M6-SB) The patches on the ground in the client (`sc_sim::patches`; `factions/syndicate/units/shadebringer.md >
//! Sludge (toggle)`: "While on, the Shadebringer leaves a **trail of sludge** on the ground in its wake. Each patch of
//! sludge lasts **X seconds** and then disappears"). Display only: the sim keeps the patches (`Sim::patches`), whatever
//! the client shows.
//!
//! - **The trail** is a band of discs, one a patch, each as wide as the trail was as it was laid
//!   (`sc_sim::patches::Patch::radius`): an opaque olive brown with a darker rim, every rim drawn first and every body
//!   over them, so that the band has one outline round the whole trail and none of the discs' edges inside it. In the
//!   ground decals layer (`DrawLayer::GroundDecals`), over the terrain and under every object, so that the units in it
//!   stand on it. Each disc at its centre's height (`crate::coords::drawn_at_height`), as a unit's feet are drawn. The
//!   layer's z is the mesh's transform's, as the Shroud's clouds have theirs: alpha blending draws the blended meshes
//!   in the order of their transforms' z, so the effects' (the HP bars, the buffs' marks) come over the trail.
//! - **Who sees it**: a patch shows to the local player while the tile its centre lies in is Visible to them
//!   (`crate::fog::Sight::at`), every patch while the fog doesn't apply: **Placeholder (not in the design: who sees the
//!   sludge)**, as an object on a Visible tile is seen (`core/vision.md`). It is no object: nothing remembers it on an
//!   Explored tile.
//! - **Its end**: over the last `LookTuning::sludge_fade_ticks` of its life a patch shrinks, to [`SHRUNK`] of its
//!   radius as it goes, by the sim's ticks plus the render alpha, so that a QA screenshot of a tick shows it alike.
//! - (Slice M6-SC) Smoke's patches are another look (they block sight), which this layer leaves out: it draws
//!   sludge's alone ([`sludge_discs`]).
//!
//! One vertex-coloured mesh, rebuilt every frame while a patch shows and hidden while none does: no entity per patch
//! (`ARCHITECTURE.md > Client architecture > Visuals`). Placeholders (Q0001, the design being silent on the look): the
//! colours, the rim, the shrinking; (M5-X2) its length is a tunable (`crate::tuning::look`), the colours stay here.

#[cfg(test)]
mod tests;

use std::f32::consts::TAU;

use bevy::prelude::*;
use sc_sim::defs::PatchKind;
use sc_sim::{Sim, Tick, VisibilityState};

use crate::coords::{DrawLayer, drawn_at_height, view_to_world};
use crate::fog::{LocalSight, Sight};
use crate::shapes::ColorMesh;
use crate::sim_driver::RenderAlpha;
use crate::tuning::ClientTuning;
use crate::{ClientMode, ClientSystems};

/// The sludge's body and its rim (placeholders, Q0001): an olive brown, and a darker one round the band.
const SLUDGE: Color = Color::srgb(0.36, 0.32, 0.12);
const SLUDGE_RIM: Color = Color::srgb(0.20, 0.17, 0.06);
/// How wide the rim is, su: the body's discs are this much smaller than the patches.
const RIM: f32 = 3.0;
/// The share of its radius a patch shrinks to by its end (module docs).
pub const SHRUNK: f32 = 0.4;
/// The segments of a disc.
const SEGMENTS: usize = 20;

pub struct PatchesViewPlugin;

impl Plugin for PatchesViewPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ClientTuning>();
        app.add_systems(Startup, spawn_patches_mesh)
            .add_systems(Update, update_patches_mesh.in_set(ClientSystems::Present));
    }
}

/// The trail's mesh entity.
#[derive(Component)]
pub struct PatchesMesh;

/// One patch's disc as drawn: centred on `center` (Bevy world space), `radius` su.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Disc {
    pub center: Vec2,
    pub radius: f32,
}

/// How much of its radius a patch that lasts `until` shows `alpha` of the way through the sim's latest tick, `tick`
/// (module docs): all of it until its last `fade` ticks, then less, down to [`SHRUNK`] at its last tick's end. As drawn
/// now, the latest tick's end is alpha 1, and its last tick on the ground is the one before `until`
/// (`sc_sim::patches::Patch::until`).
pub fn shown_share(tick: Tick, alpha: f32, until: Tick, fade: Tick) -> f32 {
    let left = (until as f32 - tick as f32 - alpha).max(0.0);
    match fade {
        0 => 1.0,
        _ => SHRUNK + (1.0 - SHRUNK) * (left / fade as f32).min(1.0),
    }
}

/// The sludge's discs to draw (module docs): one for each sludge patch whose centre `sight` shows Visible, at its
/// centre's height, shrunk over its last `fade` ticks, `alpha` of the way through the tick; in the patches' order.
pub fn sludge_discs(sim: &Sim, sight: &Sight, alpha: f32, fade: Tick) -> Vec<Disc> {
    let patches = sim.patches();
    if patches.count(PatchKind::Sludge) == 0 {
        return Vec::new();
    }
    (patches.iter())
        .filter(|patch| patch.kind == PatchKind::Sludge && sight.at(patch.centre) == VisibilityState::Visible)
        .map(|patch| {
            let height = sim.map().height_at(patch.centre);
            Disc {
                center: view_to_world(drawn_at_height(patch.centre, height)),
                radius: patch.radius * shown_share(sim.tick(), alpha, patch.until, fade),
            }
        })
        .collect()
}

/// Draws a disc round `center`, `radius` su, at `z`.
fn disc_shape(mesh: &mut ColorMesh, center: Vec2, radius: f32, z: f32, color: Color) {
    let points: Vec<Vec2> = (0..SEGMENTS)
        .map(|i| center + radius * Vec2::from_angle(i as f32 * TAU / SEGMENTS as f32))
        .collect();
    mesh.polygon(&points, z, color);
}

fn spawn_patches_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // A mesh without vertices makes the renderer warn: a zero-area stand-in until the first patch.
    let mut placeholder = ColorMesh::triangles();
    placeholder.rect(Vec2::ZERO, Vec2::ZERO, 0.0, Color::NONE);
    commands.spawn((
        Name::new("Ground patches"),
        PatchesMesh,
        Mesh2d(meshes.add(placeholder.into_mesh())),
        // White (`ColorMaterial::default`): the vertex colours are the trail's.
        MeshMaterial2d(materials.add(ColorMaterial::default())),
        // Its layer's z (module docs): blended before the effects' mesh, whose transform is at 0.
        Transform::from_xyz(0.0, 0.0, DrawLayer::GroundDecals.base_z()),
        Visibility::Hidden,
    ));
}

/// Rebuilds the trail's mesh in a game (module docs); hidden while no patch the local player sees is on the ground.
fn update_patches_mesh(
    sight: LocalSight,
    mode: Res<ClientMode>,
    alpha: Res<RenderAlpha>,
    tuning: Res<ClientTuning>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut entity: Query<(&Mesh2d, &mut Visibility), With<PatchesMesh>>,
) {
    let Ok((mesh2d, mut visibility)) = entity.single_mut() else {
        return;
    };
    let discs = match mode.is_game() {
        true => sludge_discs(sight.sim(), &sight.get(), alpha.0, tuning.look.sludge_fade_ticks),
        false => Vec::new(),
    };
    if discs.is_empty() {
        visibility.set_if_neq(Visibility::Hidden);
        return;
    }
    // The layer's z is the transform's (`spawn_patches_mesh`).
    let z = 0.0;
    let mut mesh = ColorMesh::triangles();
    for disc in &discs {
        disc_shape(&mut mesh, disc.center, disc.radius, z, SLUDGE_RIM);
    }
    for disc in &discs {
        disc_shape(&mut mesh, disc.center, (disc.radius - RIM).max(0.0), z, SLUDGE);
    }
    if let Err(error) = meshes.insert(&mesh2d.0, mesh.into_mesh()) {
        error!("ground patches rebuild failed: {error}");
    }
    visibility.set_if_neq(Visibility::Inherited);
}
