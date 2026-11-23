//! The terrain: one mesh for the whole map, never an entity per tile (`ARCHITECTURE.md > Client architecture >
//! Visuals`), plus a grid overlay toggled with [`InputAction::ToggleGrid`].
//!
//! Placeholder look until Q0001 settles the art: one flat colour per tile preset
//! (`core/map.md > DefaultTilePresets`), lighter for higher elevation (`core/map.md > TilePlacement`), and a lit
//! edge on the higher tile plus a shadow on the lower tile wherever the elevation changes. Both meshes are rebuilt
//! in place, under the same handles, whenever the sim's map differs from the one they show (map editor, new sim).

use bevy::prelude::*;
use sc_sim::{Map, SU_PER_GU, Tile, TilePos};

use crate::ClientSystems;
use crate::coords::{DrawLayer, sim_to_world};
use crate::input::{ActionState, InputAction};
use crate::shapes::{ColorMesh, darken, lighten, vertex_color_material};
use crate::sim_driver::SimRes;

/// Placeholder colours of the default presets, in `TilePresetId` order: Plane, Rugged Terrain, Cliff, Mountain,
/// Water (`core/map.md > DefaultTilePresets`).
const PRESET_COLORS: [Srgba; 5] = [
    Srgba::rgb(0.42, 0.49, 0.32),
    Srgba::rgb(0.47, 0.38, 0.26),
    Srgba::rgb(0.30, 0.28, 0.27),
    Srgba::rgb(0.56, 0.55, 0.54),
    Srgba::rgb(0.14, 0.31, 0.55),
];
/// Custom presets (`core/map.md > TilePreset`) take these in turn until they get textures.
const CUSTOM_PRESET_COLORS: [Srgba; 4] = [
    Srgba::rgb(0.60, 0.36, 0.52),
    Srgba::rgb(0.30, 0.52, 0.50),
    Srgba::rgb(0.62, 0.58, 0.30),
    Srgba::rgb(0.38, 0.36, 0.62),
];
/// White mixed into a tile's colour per elevation level, and the most it can add up to.
const LIGHTEN_PER_LEVEL: f32 = 0.08;
const MAX_ELEVATION_LIGHTEN: f32 = 0.7;
/// Elevation change cue: depth (su) and colour of the lit strip on the higher tile and the shadow on the lower one.
const EDGE_LIGHT_SU: f32 = 3.0;
const EDGE_LIGHT_AMOUNT: f32 = 0.4;
const EDGE_SHADOW_SU: f32 = 7.0;
const EDGE_SHADOW_FACTOR: f32 = 0.5;
/// Grid lines (placeholder colour).
const GRID_COLOR: Color = Color::srgba(0.0, 0.0, 0.0, 0.35);

#[derive(Component)]
pub struct Terrain;

#[derive(Component)]
pub struct GridOverlay;

/// Mesh and material handles of the terrain and grid, created once and rebuilt in place.
#[derive(Resource)]
pub struct MapRenderAssets {
    pub terrain_mesh: Handle<Mesh>,
    pub grid_mesh: Handle<Mesh>,
}

/// What the map view shows.
#[derive(Resource, Default, Debug)]
pub struct MapView {
    pub grid_visible: bool,
    /// The map the meshes were built from.
    shown: Option<Map>,
    /// How many times the meshes were built.
    pub builds: u32,
}

pub struct MapRenderPlugin;

impl Plugin for MapRenderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MapView>()
            .add_systems(Startup, spawn_map)
            .add_systems(Update, rebuild_map_meshes.in_set(ClientSystems::SyncSim))
            .add_systems(Update, toggle_grid.in_set(ClientSystems::Actions))
            .add_systems(Update, show_grid.in_set(ClientSystems::Present));
    }
}

/// The placeholder colour of `tile`: its preset's colour, lightened by its elevation.
pub fn tile_color(tile: Tile) -> Srgba {
    let index = tile.preset.0 as usize;
    let base = PRESET_COLORS
        .get(index)
        .copied()
        .unwrap_or_else(|| CUSTOM_PRESET_COLORS[(index - PRESET_COLORS.len()) % CUSTOM_PRESET_COLORS.len()]);
    lighten(
        base,
        (f32::from(tile.elevation) * LIGHTEN_PER_LEVEL).min(MAX_ELEVATION_LIGHTEN),
    )
}

/// The terrain mesh in world coordinates: one quad per tile, then the elevation cues above them.
pub fn terrain_mesh(map: &Map) -> Mesh {
    let mut mesh = ColorMesh::triangles();
    let tiles = || {
        (0..map.height() as i32)
            .flat_map(|y| (0..map.width() as i32).map(move |x| TilePos::new(x, y)))
            .filter_map(|p| Some((p, map.tile(p)?)))
    };
    for (p, tile) in tiles() {
        let (a, b) = tile_corners(p);
        mesh.rect(sim_to_world(a), sim_to_world(b), 0.0, tile_color(tile).into());
    }
    // Every edge once: each tile with its east and south neighbours.
    for (p, tile) in tiles() {
        for q in [TilePos::new(p.x + 1, p.y), TilePos::new(p.x, p.y + 1)] {
            let Some(other) = map.tile(q) else {
                continue;
            };
            if other.elevation == tile.elevation {
                continue;
            }
            let ((high, high_tile), (low, low_tile)) = if tile.elevation > other.elevation {
                ((p, tile), (q, other))
            } else {
                ((q, other), (p, tile))
            };
            let (a, b) = edge_strip(low, high, EDGE_SHADOW_SU);
            let shadow = darken(tile_color(low_tile), EDGE_SHADOW_FACTOR);
            mesh.rect(sim_to_world(a), sim_to_world(b), 0.01, shadow.into());
            let (a, b) = edge_strip(high, low, EDGE_LIGHT_SU);
            let light = lighten(tile_color(high_tile), EDGE_LIGHT_AMOUNT);
            mesh.rect(sim_to_world(a), sim_to_world(b), 0.02, light.into());
        }
    }
    mesh.into_mesh()
}

/// Grid lines along every tile edge, in world coordinates.
pub fn grid_mesh(width: u32, height: u32) -> Mesh {
    let mut mesh = ColorMesh::lines();
    let size = Vec2::new(width as f32, height as f32) * SU_PER_GU;
    for x in 0..=width {
        let x = x as f32 * SU_PER_GU;
        mesh.line(
            sim_to_world(Vec2::new(x, 0.0)),
            sim_to_world(Vec2::new(x, size.y)),
            0.0,
            GRID_COLOR,
        );
    }
    for y in 0..=height {
        let y = y as f32 * SU_PER_GU;
        mesh.line(
            sim_to_world(Vec2::new(0.0, y)),
            sim_to_world(Vec2::new(size.x, y)),
            0.0,
            GRID_COLOR,
        );
    }
    mesh.into_mesh()
}

/// Top-left and bottom-right corners of a tile, in sim space units.
fn tile_corners(p: TilePos) -> (Vec2, Vec2) {
    (p.corner(), p.corner() + Vec2::splat(SU_PER_GU))
}

/// The strip of `tile` that is `depth` su deep along its side facing the adjacent `neighbor` (sim coordinates).
fn edge_strip(tile: TilePos, neighbor: TilePos, depth: f32) -> (Vec2, Vec2) {
    let (min, max) = tile_corners(tile);
    match (neighbor.x - tile.x, neighbor.y - tile.y) {
        (1, 0) => (Vec2::new(max.x - depth, min.y), max),
        (-1, 0) => (min, Vec2::new(min.x + depth, max.y)),
        (0, 1) => (Vec2::new(min.x, max.y - depth), max),
        (0, -1) => (min, Vec2::new(max.x, min.y + depth)),
        offset => panic!("tiles {tile:?} and {neighbor:?} are not adjacent ({offset:?})"),
    }
}

fn spawn_map(
    mut commands: Commands,
    sim: Res<SimRes>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut view: ResMut<MapView>,
) {
    let map = sim.map();
    let terrain_mesh = meshes.add(terrain_mesh(map));
    let grid_mesh = meshes.add(grid_mesh(map.width(), map.height()));
    let grid_material = ColorMaterial {
        alpha_mode: bevy::sprite_render::AlphaMode2d::Blend,
        ..vertex_color_material()
    };
    commands.spawn((
        Name::new("Terrain"),
        Terrain,
        Mesh2d(terrain_mesh.clone()),
        MeshMaterial2d(materials.add(vertex_color_material())),
        Transform::from_xyz(0.0, 0.0, DrawLayer::Terrain.base_z()),
    ));
    commands.spawn((
        Name::new("Grid overlay"),
        GridOverlay,
        Mesh2d(grid_mesh.clone()),
        MeshMaterial2d(materials.add(grid_material)),
        Transform::from_xyz(0.0, 0.0, DrawLayer::GroundDecals.base_z()),
        Visibility::Hidden,
    ));
    commands.insert_resource(MapRenderAssets {
        terrain_mesh,
        grid_mesh,
    });
    view.shown = Some(map.clone());
    view.builds += 1;
}

/// Rebuilds both meshes when the sim's map is not the one they show.
fn rebuild_map_meshes(
    sim: Res<SimRes>,
    assets: Res<MapRenderAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut view: ResMut<MapView>,
) {
    if !sim.is_changed() || view.shown.as_ref() == Some(sim.map()) {
        return;
    }
    let map = sim.map();
    for (handle, mesh) in [
        (&assets.terrain_mesh, terrain_mesh(map)),
        (&assets.grid_mesh, grid_mesh(map.width(), map.height())),
    ] {
        if let Err(error) = meshes.insert(handle, mesh) {
            error!("map mesh rebuild failed: {error}");
        }
    }
    view.shown = Some(map.clone());
    view.builds += 1;
}

fn toggle_grid(actions: Res<ActionState>, mut view: ResMut<MapView>) {
    if actions.just_pressed(InputAction::ToggleGrid) {
        view.grid_visible = !view.grid_visible;
    }
}

fn show_grid(view: Res<MapView>, mut grid: Query<&mut Visibility, With<GridOverlay>>) {
    let visibility = if view.grid_visible {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    for mut v in &mut grid {
        v.set_if_neq(visibility);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sc_sim::TilePresetId;

    #[test]
    fn one_quad_per_tile_plus_two_strips_per_elevation_edge() {
        let mut map = Map::new(4, 3);
        assert_eq!(terrain_mesh(&map).count_vertices(), 4 * 4 * 3);
        // A raised tile in the middle of the top row has 3 neighbours: west, east and south.
        map.set_tile(TilePos::new(1, 0), Tile::new(TilePresetId::PLANE, 2));
        assert_eq!(terrain_mesh(&map).count_vertices(), 4 * 4 * 3 + 3 * 2 * 4);
    }

    #[test]
    fn higher_ground_is_lighter_and_presets_differ() {
        let low = tile_color(Tile::new(TilePresetId::PLANE, 0));
        let high = tile_color(Tile::new(TilePresetId::PLANE, 2));
        assert!(high.red > low.red && high.green > low.green && high.blue > low.blue);
        let water = tile_color(Tile::new(TilePresetId::WATER, 0));
        assert_ne!(low, water);
        // Custom presets get a colour too.
        let _ = tile_color(Tile::new(sc_sim::TilePresetId(9), 0));
    }

    #[test]
    fn edge_strips_lie_inside_their_tile_on_the_shared_side() {
        let t = TilePos::new(2, 3);
        let (a, b) = edge_strip(t, TilePos::new(3, 3), 4.0);
        assert_eq!((a, b), (Vec2::new(188.0, 192.0), Vec2::new(192.0, 256.0)));
        let (a, b) = edge_strip(t, TilePos::new(2, 2), 4.0);
        assert_eq!((a, b), (Vec2::new(128.0, 192.0), Vec2::new(192.0, 196.0)));
    }

    #[test]
    fn grid_has_a_line_per_tile_edge() {
        assert_eq!(grid_mesh(4, 3).count_vertices(), 2 * ((4 + 1) + (3 + 1)));
    }
}
