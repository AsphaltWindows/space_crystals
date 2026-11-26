//! What the area displays share (slice M5-J2): the Tunnel Area display's (`super::tunnel_areas`) and the Recruitment
//! Area display's (`super::recruitment_areas`), the second built as the first (`factions/cults/structures/
//! recruitment_hall.md > Recruitment Area display`: "as the Tunnel Area display").
//!
//! - **One mesh under one handle** ([`spawn_area_mesh`]), in the terrain's z band like the build area overlay
//!   (`super::overlay::AREA_Z`): the terrain in front hides it and objects stand on it. The mesh asset arrives with the
//!   first area to draw, as the renderer warns about empty meshes.
//! - **Rebuilt in place only when what it shows changed** ([`AreaMesh::is_stale`]): what it shows, the tiles among it the
//!   player hasn't explored ([`unexplored`]: those stay black, `core/vision.md > Unexplored`), or the map
//!   (`MapView::builds`), never every frame; hidden while it has nothing to show ([`show`]).
//! - **The selection's own objects** ([`selected_own`]): "Selecting one of your Tunnels", "A selected own Hall"; never
//!   another player's.
//!
//! The tiles are tinted and outlined by `super::overlay`'s [`tint_tiles`](super::overlay::tint_tiles) and
//! [`outline`](super::overlay::outline).

use bevy::prelude::*;
use bevy::sprite_render::AlphaMode2d;
use sc_sim::{Map, Object, PlayerId, Sim, TilePos, VisibilityState};

use crate::coords::DrawLayer;
use crate::fog::Sight;
use crate::map_render::MapView;
use crate::selection::Selection;
use crate::shapes::ColorMesh;

/// The state of an area overlay's mesh ([`AreaMesh::redraw`]).
#[derive(Debug, Default)]
pub struct AreaMesh {
    /// Whether the mesh last built has anything to show.
    drawn: bool,
    /// The map meshes' build count (`MapView::builds`) when it was: a new map redraws it.
    map_builds: u32,
}

impl AreaMesh {
    /// Whether the mesh is to be rebuilt: `changed` says what it shows changed, or the map was rebuilt since it was.
    pub fn is_stale(&self, changed: bool, view: &MapView) -> bool {
        changed || self.map_builds != view.builds
    }

    /// Rebuilds the mesh under `handle` from `build`, `None` while there is nothing to show. Whether it was built;
    /// `name` names it in an error.
    pub fn redraw(
        &mut self,
        view: &MapView,
        (meshes, handle): (&mut Assets<Mesh>, &Handle<Mesh>),
        name: &str,
        build: impl FnOnce() -> Option<ColorMesh>,
    ) -> bool {
        self.map_builds = view.builds;
        self.drawn = false;
        let Some(mesh) = build() else {
            return false;
        };
        self.drawn = mesh.vertex_count() > 0;
        if self.drawn
            && let Err(error) = meshes.insert(handle, mesh.into_mesh())
        {
            error!("{name} mesh: {error}");
        }
        true
    }

    /// Whether the mesh last built has anything to show.
    pub fn drawn(&self) -> bool {
        self.drawn
    }
}

/// Spawns the entity of an area overlay named `name`, marked `marker`: a blended mesh under a handle reserved now, `z`
/// above the terrain entity's, hidden. Its mesh's handle.
pub fn spawn_area_mesh(
    commands: &mut Commands,
    (meshes, materials): (&Assets<Mesh>, &mut Assets<ColorMaterial>),
    name: &'static str,
    marker: impl Bundle,
    z: f32,
) -> Handle<Mesh> {
    let mesh = meshes.reserve_handle();
    let material = ColorMaterial {
        color: Color::WHITE,
        alpha_mode: AlphaMode2d::Blend,
        ..default()
    };
    commands.spawn((
        Name::new(name),
        marker,
        Mesh2d(mesh.clone()),
        MeshMaterial2d(materials.add(material)),
        Transform::from_xyz(0.0, 0.0, DrawLayer::Terrain.base_z() + z),
        Visibility::Hidden,
    ));
    mesh
}

/// Shows the overlay's entities while `visible`, and hides them while not.
pub fn show<M: Component>(shown: &mut Query<&mut Visibility, With<M>>, visible: bool) {
    let visibility = if visible {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    for mut mesh in shown {
        mesh.set_if_neq(visibility);
    }
}

/// The tiles of `cells` on `map` that the player of `sight` hasn't explored, in their order: an area overlay leaves
/// them out.
pub fn unexplored(map: &Map, sight: &Sight, cells: impl IntoIterator<Item = TilePos>) -> Vec<TilePos> {
    (cells.into_iter())
        .filter(|&t| map.in_bounds(t) && sight.tile(t) == VisibilityState::Unexplored)
        .collect()
}

/// `player`'s own objects in `selection`, the ActiveGroup or not, in id order; none of another player's.
pub fn selected_own<'a>(
    sim: &'a Sim,
    player: PlayerId,
    selection: &'a Selection,
) -> impl Iterator<Item = &'a Object> + 'a {
    let objects = selection.ids().filter_map(|id| sim.object(id));
    objects.filter(move |o| o.owner == Some(player))
}
