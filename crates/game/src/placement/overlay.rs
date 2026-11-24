//! The build area overlay (`factions/gdo/structures.md > DeploymentCenter > ObjectInterfaceState[DeploymentCenter] >
//! AwaitingPlacement`: "The player's current GDO build area is highlighted as a semi-transparent overlay on the
//! ground. The overlay shows the **build area only**"; design change 0093: "one clean shape, which doesn't flicker as
//! units move"). Whether a tile is valid right now shows on the ghost (`super::ghost`), never here.
//!
//! One mesh, under one handle, rebuilt in place only when the local player's build area (`Sim::build_area`) or the
//! map under it changed: the overlay keeps the area it last drew and compares the new one with it (`BuildArea`'s
//! `PartialEq`, as the sim suggests), so it follows every placement and every loss without redrawing every frame.
//!
//! The look, placeholders until the art (Q0001): every tile of the area tinted on the terrain's surface, as the
//! terrain mesh draws it (tile by tile rather than by `BuildArea::runs`: a run's tiles may slope), and a brighter line
//! along the area's edge. Building it takes about 0.2 ms for a whole 28 × 28 area (dev profile). It lies in the
//! terrain's z band like the fog
//! layer and the grid, so the terrain in front hides it and objects stand on it, just in front of the fog
//! ([`AREA_Z`]): the player sees their whole area, also where the fog is dark (placeholder: the design doesn't say).

use bevy::prelude::*;
use bevy::sprite_render::AlphaMode2d;
use sc_sim::{BuildArea, Map, TilePos};

use super::placing_now;
use crate::ClientMode;
use crate::coords::DrawLayer;
use crate::fog::FOG_Z;
use crate::map_render::{MapView, SURFACE_LIFT_Z, surface_vertex, tile_surface};
use crate::selection::ControlStates;
use crate::shapes::ColorMesh;
use crate::sim_driver::{LocalPlayer, SimRes};

/// An area tile's tint, and the area's edge (placeholder colours).
pub const AREA_FILL: Color = Color::srgba(0.30, 0.70, 1.0, 0.16);
pub const AREA_EDGE: Color = Color::srgba(0.45, 0.85, 1.0, 0.85);
/// The edge line's width, space units.
const EDGE_SU: f32 = 3.0;
/// The overlay's z above the terrain entity's: in front of the fog layer ([`FOG_Z`]), far behind the ground objects'
/// band.
pub const AREA_Z: f32 = FOG_Z + 0.01;

/// What the build area overlay shows.
#[derive(Resource, Debug, Default)]
pub struct BuildAreaOverlay {
    /// The area its mesh was last built from.
    pub shown: Option<BuildArea>,
    /// The map meshes' build count (`MapView::builds`) when it was: a new map redraws it.
    map_builds: u32,
    /// Whether it shows now: while the local player places a building and has an area.
    pub visible: bool,
    /// How many times its mesh was built.
    pub builds: u32,
}

/// The overlay's mesh.
#[derive(Component)]
pub struct BuildAreaMesh;

#[derive(Resource)]
pub(super) struct OverlayAssets {
    mesh: Handle<Mesh>,
}

pub(super) fn spawn_overlay(
    mut commands: Commands,
    meshes: Res<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // The mesh asset arrives with the first area to draw: the renderer warns about empty meshes.
    let mesh = meshes.reserve_handle();
    let material = ColorMaterial {
        color: Color::WHITE,
        alpha_mode: AlphaMode2d::Blend,
        ..default()
    };
    commands.spawn((
        Name::new("Build area overlay"),
        BuildAreaMesh,
        Mesh2d(mesh.clone()),
        MeshMaterial2d(materials.add(material)),
        Transform::from_xyz(0.0, 0.0, DrawLayer::Terrain.base_z() + AREA_Z),
        Visibility::Hidden,
    ));
    commands.insert_resource(OverlayAssets { mesh });
}

/// Shows the local player's build area while they place a building ([`placing_now`]), redrawn when it changed.
#[expect(clippy::too_many_arguments, reason = "a system's parameters")]
pub(super) fn draw_overlay(
    mode: Res<ClientMode>,
    sim: Res<SimRes>,
    player: Res<LocalPlayer>,
    states: Res<ControlStates>,
    view: Res<MapView>,
    assets: Res<OverlayAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut overlay: ResMut<BuildAreaOverlay>,
    mut shown: Query<&mut Visibility, With<BuildAreaMesh>>,
) {
    let placing = placing_now(*mode, &sim, player.0, &states).is_some();
    if placing {
        let area = sim.build_area(player.0);
        if overlay.shown.as_ref() != Some(&area) || overlay.map_builds != view.builds {
            let mesh = area_mesh(sim.map(), &area);
            if mesh.vertex_count() > 0
                && let Err(error) = meshes.insert(&assets.mesh, mesh.into_mesh())
            {
                error!("build area overlay mesh: {error}");
            }
            overlay.shown = Some(area);
            overlay.map_builds = view.builds;
            overlay.builds += 1;
        }
    }
    let visible = placing && overlay.shown.as_ref().is_some_and(|area| !area.is_empty());
    if overlay.visible != visible {
        overlay.visible = visible;
    }
    let visibility = if visible {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    for mut overlay in &mut shown {
        overlay.set_if_neq(visibility);
    }
}

/// The overlay's mesh: each tile of `area` tinted on `map`'s surface (a quad if it is flat, else four triangles
/// around its centre), and a line along every tile edge where the area ends. Bevy world coordinates in the terrain's
/// z band, relative to the overlay's entity.
pub fn area_mesh(map: &Map, area: &BuildArea) -> ColorMesh {
    let mut mesh = ColorMesh::triangles();
    let tiles: Vec<TilePos> = area.tiles().collect();
    for &t in &tiles {
        let surface = tile_surface(map, t);
        let [nw, ne, se, sw, center] = surface.map(|(ground, height)| surface_vertex(ground, height, SURFACE_LIFT_Z));
        if surface[..4].iter().all(|&(_, height)| height == surface[0].1) {
            mesh.quad([nw, ne, se, sw], AREA_FILL);
        } else {
            for (a, b) in [(nw, ne), (ne, se), (se, sw), (sw, nw)] {
                mesh.triangle([a, b, center], AREA_FILL);
            }
        }
    }
    let inside: std::collections::HashSet<TilePos> = tiles.iter().copied().collect();
    for &t in &tiles {
        let [nw, ne, se, sw, _] =
            tile_surface(map, t).map(|(ground, height)| surface_vertex(ground, height, SURFACE_LIFT_Z));
        let sides = [
            (TilePos::new(t.x, t.y - 1), nw, ne),
            (TilePos::new(t.x + 1, t.y), ne, se),
            (TilePos::new(t.x, t.y + 1), se, sw),
            (TilePos::new(t.x - 1, t.y), sw, nw),
        ];
        for (neighbour, a, b) in sides {
            if !inside.contains(&neighbour) {
                edge(&mut mesh, a, b);
            }
        }
    }
    mesh
}

/// A line from `a` to `b` [`EDGE_SU`] wide, each end at its own z (the terrain's depth grows southwards).
fn edge(mesh: &mut ColorMesh, a: Vec3, b: Vec3) {
    let along = (b - a).truncate();
    let side = along.normalize_or_zero().perp() * (EDGE_SU / 2.0);
    let side = side.extend(0.0);
    mesh.quad([a - side, b - side, b + side, a + side], AREA_EDGE);
}

#[cfg(test)]
mod tests {
    use sc_sim::{ObjectKind, Placement, Rotation};

    use super::*;

    /// Design change 0093: the overlay is the area only, one clean shape: every area tile once, and an edge only
    /// where the area ends.
    #[test]
    fn the_overlay_is_the_area_and_its_edge() {
        let map = Map::new(20, 20);
        let placement = Placement {
            origin: TilePos::new(5, 5),
            w: 2,
            h: 2,
            rotation: Rotation::R0,
            flip_h: false,
            flip_v: false,
        };
        // A Generator (2 × 2, extension 1) adds 4 × 4 tiles.
        let area = BuildArea::of(&map, [(ObjectKind::Generator, &placement)]);
        assert_eq!(area.tiles().count(), 16);
        // 16 quads, and 16 tile edges around the square.
        assert_eq!(area_mesh(&map, &area).vertex_count(), 16 * 4 + 16 * 4);
        assert_eq!(area_mesh(&map, &BuildArea::default()).vertex_count(), 0);
    }
}
