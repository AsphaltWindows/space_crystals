//! The unit path overlay, a development view toggled with [`InputAction::TogglePaths`] (or shown from the start with
//! `--show-paths`): each unit's remaining path as a thin line from where the unit is drawn through its remaining
//! waypoints (`docs/movement.md > 6`), lifted by the terrain's height along the way, in the unit's colour.

use bevy::prelude::*;
use sc_sim::Map;

use crate::ClientSystems;
use crate::coords::{DrawLayer, drawn_position, view_to_world};
use crate::input::{ActionState, InputAction};
use crate::object_render::player_color;
use crate::shapes::{ColorMesh, lighten, vertex_color_material};
use crate::sim_driver::{RenderAlpha, SimRes};

/// Whether the overlay is shown.
#[derive(Resource, Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct PathOverlay {
    pub visible: bool,
}

/// The overlay's entity: one mesh for every path.
#[derive(Component)]
pub struct PathOverlayMesh;

pub struct PathOverlayPlugin {
    /// Shown from the start.
    pub visible: bool,
}

impl Plugin for PathOverlayPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PathOverlay { visible: self.visible })
            .add_systems(Startup, spawn_overlay)
            .add_systems(Update, toggle_paths.in_set(ClientSystems::Actions))
            .add_systems(Update, draw_paths.in_set(ClientSystems::Present));
    }
}

/// A path is cut into pieces this long (space units) so that it follows the terrain's height.
const PIECE: f32 = 16.0;
/// The line's width in space units.
const WIDTH: f32 = 2.0;

fn spawn_overlay(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // A mesh without vertices makes the renderer warn: a zero-area stand-in until the first path.
    let mut placeholder = ColorMesh::triangles();
    placeholder.rect(Vec2::ZERO, Vec2::ZERO, 0.0, Color::NONE);
    commands.spawn((
        Name::new("Path overlay"),
        PathOverlayMesh,
        Mesh2d(meshes.add(placeholder.into_mesh())),
        MeshMaterial2d(materials.add(vertex_color_material())),
        Transform::IDENTITY,
        Visibility::Hidden,
    ));
}

fn toggle_paths(actions: Res<ActionState>, mut overlay: ResMut<PathOverlay>) {
    if actions.just_pressed(InputAction::TogglePaths) {
        overlay.visible = !overlay.visible;
    }
}

/// Rebuilds the mesh every frame while the overlay is shown, since units move between ticks; hidden while no unit
/// has a path.
fn draw_paths(
    overlay: Res<PathOverlay>,
    sim: Res<SimRes>,
    alpha: Res<RenderAlpha>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut entity: Query<(&Mesh2d, &mut Visibility), With<PathOverlayMesh>>,
) {
    let Ok((mesh2d, mut visibility)) = entity.single_mut() else {
        return;
    };
    let mut mesh = ColorMesh::triangles();
    if overlay.visible {
        let map = sim.map();
        let z = DrawLayer::Overlays.base_z();
        for object in sim.objects() {
            let Some(mover) = &object.mover else {
                continue;
            };
            let color = lighten(player_color(object.owner).to_srgba(), 0.4).into();
            let mut from = object.prev_pos.lerp(object.pos, alpha.0);
            for &to in mover.remaining_path() {
                segment_on_terrain(&mut mesh, map, from, to, z, color);
                from = to;
            }
        }
    }
    if mesh.vertex_count() == 0 {
        visibility.set_if_neq(Visibility::Hidden);
        return;
    }
    if let Err(error) = meshes.insert(&mesh2d.0, mesh.into_mesh()) {
        error!("path overlay rebuild failed: {error}");
    }
    visibility.set_if_neq(Visibility::Inherited);
}

/// The ground segment from `a` to `b` as drawn on the terrain.
fn segment_on_terrain(mesh: &mut ColorMesh, map: &Map, a: Vec2, b: Vec2, z: f32, color: Color) {
    let pieces = ((b - a).length() / PIECE).ceil().clamp(1.0, 4096.0) as u32;
    let at = |i: u32| view_to_world(drawn_position(map, a.lerp(b, i as f32 / pieces as f32)));
    for i in 0..pieces {
        mesh.segment(at(i), at(i + 1), WIDTH, z, color);
    }
}
