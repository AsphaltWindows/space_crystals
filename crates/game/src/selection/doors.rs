//! The door arrows of the selected buildings (design change 0394, `core/entities.md > Entrance and exit (production
//! and research buildings)`: "The same arrows show on a placed building while it is selected"): the arrows of each
//! building the local player has selected (`crate::door_arrows`), where the markers mark it (`super::markers`): a live
//! one where it stands, or a record where it shows in its place; finished or not; whoever owns it (the design names
//! no owner, and the arrows tell no more than its type and the way it is turned show); in a game mode only.
//!
//! One mesh, under one handle, rebuilt in place only when what it shows changes ([`SelectedDoors`]); in the overlays'
//! band under the placement ghost ([`DOORS_Z`]), so that they show over whatever stands at a doorway.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use sc_sim::{ObjectKind, Placement};

use super::ControlStates;
use crate::coords::DrawLayer;
use crate::door_arrows::{door_arrows, draw_arrows};
use crate::fog::LocalSight;
use crate::shapes::{ColorMesh, vertex_color_material};
use crate::sim_driver::{LocalPlayer, SimRes};
use crate::{ClientMode, ClientSystems};

/// The arrows' z over the overlays' band base: under the placement ghost's (`crate::placement::ghost`), over
/// everything on the map.
const DOORS_Z: f32 = -3.0;

/// What the selected buildings' door arrows show.
#[derive(Resource, Debug, Default)]
pub struct SelectedDoors {
    /// The selected buildings with doorways, by their type and footprint as placed, in id order.
    pub shown: Vec<(ObjectKind, Placement)>,
    /// The sim's generation (`SimRes::generation`) when the mesh was built: a new map draws it again.
    generation: u32,
    /// How many times its mesh was built: only when what it shows changed.
    pub builds: u32,
}

/// The selected buildings' door arrows' mesh.
#[derive(Component)]
pub struct SelectedDoorsMesh;

#[derive(Resource)]
struct DoorAssets {
    mesh: Handle<Mesh>,
}

pub(super) struct SelectedDoorsPlugin;

impl Plugin for SelectedDoorsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedDoors>()
            .add_systems(Startup, spawn_doors)
            .add_systems(Update, draw_doors.in_set(ClientSystems::Present));
    }
}

fn spawn_doors(mut commands: Commands, meshes: Res<Assets<Mesh>>, mut materials: ResMut<Assets<ColorMaterial>>) {
    // The mesh asset arrives with the first arrows: the renderer warns about empty meshes.
    let mesh = meshes.reserve_handle();
    commands.spawn((
        Name::new("Selected buildings' door arrows"),
        SelectedDoorsMesh,
        Mesh2d(mesh.clone()),
        MeshMaterial2d(materials.add(vertex_color_material())),
        Transform::from_xyz(0.0, 0.0, DrawLayer::Overlays.base_z() + DOORS_Z),
        Visibility::Hidden,
    ));
    commands.insert_resource(DoorAssets { mesh });
}

/// What the arrows show: the local player's selection in a game mode, each building as it is drawn.
#[derive(SystemParam)]
struct Selected<'w> {
    mode: Res<'w, ClientMode>,
    sight: LocalSight<'w>,
    player: Res<'w, LocalPlayer>,
    states: Res<'w, ControlStates>,
}

impl Selected<'_> {
    /// The selected buildings with doorways ([`SelectedDoors::shown`]): a live one where it stands, if the local
    /// player sees it, else its record where the record shows it.
    fn buildings(&self) -> Vec<(ObjectKind, Placement)> {
        if !self.mode.is_game() {
            return Vec::new();
        }
        let (sim, sight) = (self.sight.sim(), self.sight.get());
        (self.states.selection(self.player.0).ids())
            .filter_map(|id| match sim.object(id).filter(|o| sight.sees(o)) {
                Some(object) => Some((object.kind, object.placement?)),
                None => sight
                    .remembered_record(id)
                    .map(|record| (record.kind, record.placement)),
            })
            .filter(|(kind, placement)| !door_arrows(*kind, placement).is_empty())
            .collect()
    }
}

/// Draws the arrows when what they show changed: the mesh rebuilt in place, or hidden while none shows.
fn draw_doors(
    selected: Selected,
    generation: Res<SimRes>,
    assets: Res<DoorAssets>,
    mut doors: ResMut<SelectedDoors>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mesh: Query<&mut Visibility, With<SelectedDoorsMesh>>,
) {
    let sim = selected.sight.sim();
    let generation = generation.generation();
    let shown = selected.buildings();
    if doors.shown == shown && doors.generation == generation {
        return;
    }
    let visibility = if shown.is_empty() {
        Visibility::Hidden
    } else {
        let mut arrows = ColorMesh::triangles();
        for (kind, placement) in &shown {
            draw_arrows(&mut arrows, sim.map(), &door_arrows(*kind, placement), 0.0);
        }
        if let Err(error) = meshes.insert(&assets.mesh, arrows.into_mesh()) {
            error!("door arrows mesh: {error}");
        }
        doors.builds += 1;
        Visibility::Inherited
    };
    doors.shown = shown;
    doors.generation = generation;
    for mut shown in &mut mesh {
        shown.set_if_neq(visibility);
    }
}
