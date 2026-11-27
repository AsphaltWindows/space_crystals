//! The terrain: one mesh for the whole map, never an entity per tile (`ARCHITECTURE.md > Client architecture >
//! Visuals`), plus a grid overlay toggled with [`InputAction::ToggleGrid`].
//!
//! `core/art_direction.md > Elevation Display`, with placeholder colours until Q0001 settles the art:
//! - every tile is drawn at its corners' heights, lifted up the screen ([`crate::coords::drawn_position`]): a flat
//!   tile as one quad, a sloped one as a fan of four triangles around its center, which stands at the mean of its
//!   corners (the bilinear height there);
//! - each triangle is shaded by the direction it faces: lighter towards [`LIGHT_TOWARDS`], darker away from it;
//! - steep non-Traversible tiles read as cliff faces: rock-coloured, with a stratum line at every level;
//! - every vertex's z grows with its ground y, so with the depth test the terrain further south overdraws the
//!   terrain behind it where they overlap on screen;
//! - the grid follows the surface, in the terrain's own z band, so terrain in front hides it too.
//!
//! Both meshes are rebuilt in place, under the same handles, whenever the sim's map differs from the one they show
//! (map editor, new sim).

use bevy::prelude::*;
use sc_sim::{CornerPos, Map, Tile, TileLook, TilePos, TilePresetId};

use crate::ClientSystems;
use crate::coords::{DrawLayer, depth_offset, drawn_at_height, view_to_world};
use crate::input::{ActionState, InputAction};
use crate::shapes::{ColorMesh, lighten, shade, vertex_color_material};
use crate::sim_driver::SimRes;

/// Placeholder colours of the looks (the textures to come), in [`TileLook::ALL`] order: Plane, Rugged, Cliff,
/// Mountain, Water. A preset is drawn by its look (`core/map_editor.md > Custom Tile Presets`: custom presets use
/// the existing textures).
const LOOK_COLORS: [Srgba; 5] = [
    Srgba::rgb(0.42, 0.49, 0.32),
    Srgba::rgb(0.47, 0.38, 0.26),
    Srgba::rgb(0.30, 0.28, 0.27),
    Srgba::rgb(0.56, 0.55, 0.54),
    Srgba::rgb(0.14, 0.31, 0.55),
];
/// White mixed into a tile's colour per level of height: a hint of altitude besides the shading (placeholder).
const LIGHTEN_PER_LEVEL: f32 = 0.03;
/// Towards the light that shades the terrain (`core/art_direction.md > Elevation Display`), in ground axes: +x
/// east, +y south, +z up. Placeholder: from the north-west, the top left of the screen, 45° above the horizon.
pub const LIGHT_TOWARDS: Vec3 = Vec3::new(-1.0, -1.0, std::f32::consts::SQRT_2);
/// The height of one level when shading, in space units: steeper than the drawn offset, so that gentle slopes read
/// (placeholder).
const SHADING_SU_PER_LEVEL: f32 = 32.0;
/// The light every face gets, and the light that depends on its facing (Lambert). Flat ground keeps its colour.
const AMBIENT: f32 = 0.35;
const DIFFUSE: f32 = 0.65;
/// Non-Traversible tiles at least this many levels steep are drawn as cliff faces (`core/art_direction.md >
/// Elevation Display`).
pub const CLIFF_MIN_SLOPE: u8 = 2;
/// Cliff faces: the rock colour mixed into the tile's colour, how much, and the stratum lines along each level.
const CLIFF_ROCK: Srgba = Srgba::rgb(0.40, 0.36, 0.32);
const CLIFF_ROCK_AMOUNT: f32 = 0.5;
const STRATUM_SHADE: f32 = 0.62;
const STRATUM_WIDTH_SU: f32 = 2.0;
/// Grid lines (placeholder colour).
const GRID_COLOR: Color = Color::srgba(0.0, 0.0, 0.0, 0.35);
/// How far in front of the surface the grid, the strata and the editor's slope tint lie (z, about 2 su of ground
/// y): enough to win the depth test against the surface they lie on, too little to show through the terrain in
/// front of it.
pub const SURFACE_LIFT_Z: f32 = 0.002;

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

/// The placeholder colour of a look at `height` levels: the look's colour, lightened a little per level.
pub fn tile_color(look: TileLook, height: f32) -> Srgba {
    let index = TileLook::ALL.iter().position(|l| *l == look).unwrap_or(0);
    lighten(LOOK_COLORS[index], height * LIGHTEN_PER_LEVEL)
}

/// How much light a face gets relative to flat ground (1): Lambert's law towards [`LIGHT_TOWARDS`]. Its corners
/// are in shading space: ground x and y, and height in [`SHADING_SU_PER_LEVEL`]s.
pub fn light_factor(corners: [Vec3; 3]) -> f32 {
    let normal = (corners[1] - corners[0]).cross(corners[2] - corners[0]);
    // Whichever way the corners run, the face looks up.
    let normal = if normal.z < 0.0 { -normal } else { normal }.normalize_or_zero();
    if normal == Vec3::ZERO {
        return 1.0;
    }
    let light = LIGHT_TOWARDS.normalize();
    (AMBIENT + DIFFUSE * normal.dot(light).max(0.0)) / (AMBIENT + DIFFUSE * light.z)
}

/// A tile's surface as the terrain mesh draws it: its NW, NE, SE and SW corners, then its center at the corners'
/// mean height, each a ground point with its height in levels. A flat tile is one quad, a sloped one four
/// triangles from its edges to its center. Off the map, the ground continues at the height of the nearest point
/// on it ([`Map::height_at`]).
pub fn tile_surface(map: &Map, t: TilePos) -> [(Vec2, f32); 5] {
    let [nw, ne, sw, se] = t.corners().map(|c| {
        let height = map.corner(c).map_or_else(|| map.height_at(c.pos()), f32::from);
        (c.pos(), height)
    });
    let mean = (nw.1 + ne.1 + sw.1 + se.1) / 4.0;
    [nw, ne, se, sw, (t.center(), mean)]
}

/// A vertex in the terrain's z band (Bevy world coordinates, relative to the band's base): drawn at `height`, z by
/// its ground y, plus `lift_z`.
pub fn surface_vertex(ground: Vec2, height: f32, lift_z: f32) -> Vec3 {
    view_to_world(drawn_at_height(ground, height)).extend(depth_offset(ground.y) + lift_z)
}

/// A point of the terrain surface: its ground position and its height in levels.
#[derive(Copy, Clone, Debug)]
struct SurfacePoint {
    ground: Vec2,
    height: f32,
}

impl SurfacePoint {
    fn corner(map: &Map, c: CornerPos) -> Self {
        Self {
            ground: c.pos(),
            height: f32::from(map.corner(c).unwrap_or(0)),
        }
    }

    /// The vertex in the terrain mesh: drawn position, z by ground y.
    fn vertex(self, lift_z: f32) -> Vec3 {
        surface_vertex(self.ground, self.height, lift_z)
    }

    /// The point in shading space.
    fn shading(self) -> Vec3 {
        self.ground.extend(self.height * SHADING_SU_PER_LEVEL)
    }
}

/// The terrain mesh in world coordinates, tile by tile.
pub fn terrain_mesh(map: &Map) -> Mesh {
    let mut mesh = ColorMesh::triangles();
    for y in 0..map.height() as i32 {
        for x in 0..map.width() as i32 {
            let t = TilePos::new(x, y);
            if let Some(tile) = map.tile(t) {
                terrain_tile(&mut mesh, map, t, tile);
            }
        }
    }
    mesh.into_mesh()
}

/// The lowest and the highest of a tile's corner `heights`.
fn height_span(heights: [f32; 4]) -> (f32, f32) {
    heights
        .iter()
        .fold((f32::MAX, f32::MIN), |(lo, hi), &h| (lo.min(h), hi.max(h)))
}

/// A tile's colour before its faces are shaded, and whether it is drawn as a cliff face: its preset's look at the
/// mean of its corners' `heights` ([`tile_color`]), mixed with rock on a non-Traversible tile at least
/// [`CLIFF_MIN_SLOPE`] levels steep.
fn base_color(map: &Map, tile: Tile, heights: [f32; 4]) -> (Srgba, bool) {
    let (low, high) = height_span(heights);
    let mean = heights.iter().sum::<f32>() / 4.0;
    let preset = map.presets().get(tile.preset.0 as usize);
    let traversible = preset.is_some_and(|p| p.traversible);
    let cliff = !traversible && high - low >= f32::from(CLIFF_MIN_SLOPE);
    let color = tile_color(preset.map_or(TileLook::Plane, |p| p.look), mean);
    match cliff {
        true => (color.mix(&CLIFF_ROCK, CLIFF_ROCK_AMOUNT), true),
        false => (color, false),
    }
}

/// (M5-MM) The colour of tile `t` before the terrain mesh shades its faces: what the minimap paints it in, flat
/// (`crate::hud::minimap`). A tile off the map is a flat Plane's.
pub fn tile_base_color(map: &Map, t: TilePos) -> Srgba {
    let heights = t.corners().map(|c| SurfacePoint::corner(map, c).height);
    let tile = map.tile(t).unwrap_or(Tile::new(TilePresetId::PLANE));
    base_color(map, tile, heights).0
}

/// One tile: a quad if it is flat, else four shaded triangles around its center, and strata on a cliff face.
fn terrain_tile(mesh: &mut ColorMesh, map: &Map, t: TilePos, tile: Tile) {
    let [nw, ne, sw, se] = t.corners().map(|c| SurfacePoint::corner(map, c));
    let heights = [nw, ne, sw, se].map(|p| p.height);
    let (low, high) = height_span(heights);
    let mean = heights.iter().sum::<f32>() / 4.0;
    let (color, cliff) = base_color(map, tile, heights);
    if high == low {
        mesh.quad([nw, ne, se, sw].map(|p| p.vertex(0.0)), color.into());
        return;
    }
    let center = SurfacePoint {
        ground: t.center(),
        height: mean,
    };
    for (a, b) in [(nw, ne), (ne, se), (se, sw), (sw, nw)] {
        let face = [a, b, center];
        let face_color = shade(color, light_factor(face.map(SurfacePoint::shading)));
        mesh.triangle(face.map(|p| p.vertex(0.0)), face_color.into());
        if cliff {
            // A stratum at every whole level strictly between the tile's lowest and highest corner.
            for level in (low as u8 + 1)..(high as u8) {
                stratum(mesh, face, f32::from(level), shade(face_color, STRATUM_SHADE));
            }
        }
    }
}

/// The line where the face crosses height `level`, drawn as a thin band just in front of the face.
fn stratum(mesh: &mut ColorMesh, face: [SurfacePoint; 3], level: f32, color: Srgba) {
    let mut ends: Vec<SurfacePoint> = Vec::new();
    for (a, b) in [(face[0], face[1]), (face[1], face[2]), (face[2], face[0])] {
        if a.height == b.height {
            continue;
        }
        let t = (level - a.height) / (b.height - a.height);
        if (0.0..=1.0).contains(&t) {
            let p = SurfacePoint {
                ground: a.ground.lerp(b.ground, t),
                height: level,
            };
            if !ends.iter().any(|e| e.ground.distance(p.ground) < 0.01) {
                ends.push(p);
            }
        }
    }
    let [a, b] = ends[..] else {
        return;
    };
    let half = Vec3::new(0.0, STRATUM_WIDTH_SU / 2.0, 0.0);
    let (a, b) = (a.vertex(SURFACE_LIFT_Z), b.vertex(SURFACE_LIFT_Z));
    mesh.quad([a - half, b - half, b + half, a + half], color.into());
}

/// Grid lines along every tile edge, following the surface: an edge's height is linear between its corners.
pub fn grid_mesh(map: &Map) -> Mesh {
    let mut mesh = ColorMesh::lines();
    let (width, height) = (map.width() as i32, map.height() as i32);
    let point = |cx: i32, cy: i32| SurfacePoint::corner(map, CornerPos::new(cx, cy)).vertex(SURFACE_LIFT_Z);
    for cy in 0..=height {
        for cx in 0..=width {
            if cx < width {
                mesh.line_3d(point(cx, cy), point(cx + 1, cy), GRID_COLOR);
            }
            if cy < height {
                mesh.line_3d(point(cx, cy), point(cx, cy + 1), GRID_COLOR);
            }
        }
    }
    mesh.into_mesh()
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
    let grid_mesh = meshes.add(grid_mesh(map));
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
    // In the terrain's z band, so that the terrain in front of a line hides it.
    commands.spawn((
        Name::new("Grid overlay"),
        GridOverlay,
        Mesh2d(grid_mesh.clone()),
        MeshMaterial2d(materials.add(grid_material)),
        Transform::from_xyz(0.0, 0.0, DrawLayer::Terrain.base_z()),
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
        (&assets.grid_mesh, grid_mesh(map)),
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
    use bevy::mesh::VertexAttributeValues;

    use super::*;
    use crate::coords::lift;

    fn positions(mesh: &Mesh) -> Vec<[f32; 3]> {
        match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
            Some(VertexAttributeValues::Float32x3(p)) => p.clone(),
            other => panic!("positions: {other:?}"),
        }
    }

    #[test]
    fn flat_tiles_are_quads_and_sloped_tiles_four_triangles() {
        let mut map = Map::new(4, 3);
        assert_eq!(terrain_mesh(&map).count_vertices(), 4 * 4 * 3);
        // Raising corner (1, 1) slopes the four tiles around it: 12 vertices each instead of 4.
        map.set_corner(CornerPos::new(1, 1), 1);
        assert_eq!(terrain_mesh(&map).count_vertices(), 4 * 4 * 3 + 4 * 8);
    }

    #[test]
    fn vertices_are_lifted_by_their_height_and_sorted_by_their_ground() {
        let mut map = Map::new(2, 2);
        map.set_corner(CornerPos::new(1, 1), 2);
        let vertices = positions(&terrain_mesh(&map));
        // The raised middle corner is drawn 2 levels up the screen, at the depth of its ground row.
        assert!(vertices.contains(&[64.0, -(64.0 - lift(2.0)), depth_offset(64.0)]));
        // The south-west corner (ground y 128) lies in front of the north-west one (ground y 0).
        assert!(vertices.contains(&[0.0, -128.0, depth_offset(128.0)]));
        assert!(vertices.contains(&[0.0, 0.0, depth_offset(0.0)]));
        assert!(vertices.iter().all(|p| (0.0..=depth_offset(128.0)).contains(&p[2])));
    }

    #[test]
    fn slopes_facing_the_light_are_lighter() {
        let flat = [Vec3::ZERO, Vec3::X * 64.0, Vec3::Y * 64.0];
        assert!((light_factor(flat) - 1.0).abs() < 1e-6);
        // Rising towards the south-east, the face looks north-west, towards the light.
        let facing_light = [Vec3::ZERO, Vec3::new(64.0, 0.0, 32.0), Vec3::new(0.0, 64.0, 32.0)];
        let facing_away = [
            Vec3::new(0.0, 0.0, 64.0),
            Vec3::new(64.0, 0.0, 32.0),
            Vec3::new(0.0, 64.0, 32.0),
        ];
        assert!(light_factor(facing_light) > 1.0);
        assert!(light_factor(facing_away) < 1.0);
        // The corners' order doesn't matter; a degenerate face is unshaded.
        assert_eq!(
            light_factor(facing_light),
            light_factor([facing_light[1], facing_light[0], facing_light[2]])
        );
        assert_eq!(light_factor([Vec3::ZERO; 3]), 1.0);
    }

    #[test]
    fn steep_non_traversible_tiles_are_cliff_faces_with_strata() {
        // The same south-facing drop of 3 levels as a Plane and as a Cliff.
        let plane = Map::from_ascii(".\n\n33\n00").unwrap();
        let cliff = Map::from_ascii("c\n\n33\n00").unwrap();
        let (plane, cliff) = (terrain_mesh(&plane), terrain_mesh(&cliff));
        assert_eq!(plane.count_vertices(), 12);
        // Levels 1 and 2 cross the fan's triangles six times: six bands of 4 vertices.
        assert_eq!(cliff.count_vertices(), 12 + 6 * 4);
        // A gentle non-Traversible slope is no cliff.
        assert_eq!(
            terrain_mesh(&Map::from_ascii("~\n\n11\n00").unwrap()).count_vertices(),
            12
        );
    }

    #[test]
    fn higher_ground_is_lighter_and_looks_differ() {
        let low = tile_color(TileLook::Plane, 0.0);
        let high = tile_color(TileLook::Plane, 2.0);
        assert!(high.red > low.red && high.green > low.green && high.blue > low.blue);
        for (i, a) in TileLook::ALL.iter().enumerate() {
            for b in &TileLook::ALL[i + 1..] {
                assert_ne!(tile_color(*a, 0.0), tile_color(*b, 0.0), "{a:?} and {b:?}");
            }
        }
    }

    /// `core/map_editor.md > Custom Tile Presets`: a custom preset is drawn with its look's colour.
    #[test]
    fn custom_presets_are_drawn_by_their_look() {
        let mut map = Map::from_ascii("~.").unwrap();
        let ice = map.add_preset(sc_sim::TilePreset {
            name: "Ice".into(),
            buildable: false,
            traversible: true,
            rugged: false,
            drillable: false,
            recruitable: false,
            look: TileLook::Water,
        });
        map.set_tile(TilePos::new(1, 0), Tile::new(ice));
        let colors = match terrain_mesh(&map).attribute(Mesh::ATTRIBUTE_COLOR) {
            Some(VertexAttributeValues::Float32x4(c)) => c.clone(),
            other => panic!("colors: {other:?}"),
        };
        // Two flat tiles: a quad each.
        assert_eq!(colors.len(), 8);
        assert_eq!(colors[0], colors[4], "the custom tile looks like the water tile");
    }

    #[test]
    fn the_grid_has_a_line_per_tile_edge_on_the_surface() {
        let mut map = Map::new(4, 3);
        let edges = 4 * (3 + 1) + 3 * (4 + 1);
        assert_eq!(grid_mesh(&map).count_vertices(), 2 * edges);
        map.set_corner(CornerPos::new(4, 3), 3);
        let lifted = positions(&grid_mesh(&map))
            .iter()
            .any(|p| p[0] == 256.0 && p[1] == -(192.0 - lift(3.0)));
        assert!(lifted, "the grid follows the raised corner");
    }
}
