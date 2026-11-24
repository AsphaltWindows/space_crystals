//! The unit path overlay, a development view toggled with [`InputAction::TogglePaths`] (or shown from the start with
//! `--show-paths`): each unit's remaining path as a thin line from where the unit is drawn through its remaining
//! waypoints (`docs/movement.md > 6`), lifted by the terrain's height along the way, in the unit's colour; and a
//! frame around each unit in the colour of its move's status ([`status_color`], `docs/movement.md > 8`). Only for
//! the units the local player sees (`crate::fog::Sight::sees`): the fog of war hides the others' paths too.

use bevy::prelude::*;
use sc_sim::{Map, MoveStatus, Mover, Tick};

use crate::ClientSystems;
use crate::coords::{DrawLayer, drawn_position, view_to_world};
use crate::fog::LocalSight;
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
/// The status frame stands this far outside the unit's silhouette, space units.
const FRAME_MARGIN: f32 = 4.0;

/// Placeholder colours of the status frames (a development view).
const MOVING: Color = Color::srgb(0.25, 0.9, 0.3);
const HELD_UP: Color = Color::srgb(1.0, 0.75, 0.1);
const FAILED: Color = Color::srgb(1.0, 0.15, 0.15);
const ARRIVED: Color = Color::srgb(0.3, 0.6, 1.0);

/// The colour of a unit's status frame: green while it moves, amber while it is held up (no progress for a while:
/// about to be stuck, or stuck and recovering, `Mover::is_held_up`), red after its move failed, blue after it
/// arrived. None for a unit that had no move, or was stopped.
pub fn status_color(mover: &Mover, tick: Tick) -> Option<Color> {
    match mover.status {
        MoveStatus::Moving if mover.is_held_up(tick) => Some(HELD_UP),
        MoveStatus::Moving => Some(MOVING),
        MoveStatus::Failed | MoveStatus::TargetLost => Some(FAILED),
        MoveStatus::Arrived => Some(ARRIVED),
        MoveStatus::Idle => None,
    }
}

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
/// has a path or a status to show.
fn draw_paths(
    overlay: Res<PathOverlay>,
    sim: Res<SimRes>,
    alpha: Res<RenderAlpha>,
    sight: LocalSight,
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
        let sight = sight.get();
        for object in sim.objects().filter(|o| sight.sees(o)) {
            let Some(mover) = &object.mover else {
                continue;
            };
            let color = lighten(player_color(object.owner).to_srgba(), 0.4).into();
            let mut from = object.prev_pos.lerp(object.pos, alpha.0);
            let at = from;
            for &to in mover.remaining_path() {
                segment_on_terrain(&mut mesh, map, from, to, z, color);
                from = to;
            }
            if let (Some(status), Some(half)) = (status_color(mover, sim.tick()), object.def().half_extents()) {
                // The unit is drawn at its centre's height: so is its frame.
                let centre = view_to_world(drawn_position(map, at));
                let reach = half + Vec2::splat(FRAME_MARGIN);
                let corners =
                    [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)].map(|(x, y)| centre + reach * Vec2::new(x, y));
                for i in 0..4 {
                    mesh.segment(corners[i], corners[(i + 1) % 4], WIDTH, z, status);
                }
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

#[cfg(test)]
mod tests {
    use sc_sim::mover::HALF_STUCK_TICKS;
    use sc_sim::{MoveGoal, Vec2 as SimVec2};

    use super::*;

    #[test]
    fn each_status_has_its_colour() {
        let mut mover = Mover::default();
        assert_eq!(status_color(&mover, 5), None, "idle");
        mover.go_to(MoveGoal::Point(SimVec2::ONE), None, 5);
        mover.follow(vec![SimVec2::ZERO, SimVec2::ONE], SimVec2::ONE, 0);
        assert_eq!(status_color(&mover, 6), Some(MOVING));
        assert_eq!(status_color(&mover, 5 + HALF_STUCK_TICKS), Some(HELD_UP));
        mover.finish(MoveStatus::Failed);
        assert_eq!(status_color(&mover, 40), Some(FAILED));
        mover.finish(MoveStatus::Arrived);
        assert_eq!(status_color(&mover, 40), Some(ARRIVED));
        mover.stop();
        assert_eq!(status_color(&mover, 40), None);
    }
}
