//! What the editor draws over the map: what the tool under the cursor would change (the brush's tiles, the corners
//! a height dab moves, tiles too steep for the painted preset, the ghosts of an object about to be placed, red where
//! it can't go), the selection, spawns and tiles with problems, the symmetry axes, each player slot's start location
//! and camera start, and on request (K) a tint of flat, sloped and too steep tiles.
//!
//! Everything sits where the terrain is drawn (`crate::coords::drawn_position`): outlines follow the surface. One
//! overlay mesh, on top of everything, is rebuilt when any of that changes. The slope tint is a mesh of its own in
//! the terrain's z band, like the grid, so the terrain in front hides it; it is rebuilt when the map changes. The
//! ghosts reuse the object placeholder meshes with a translucent material; the slot labels are world-space text
//! kept at a constant screen size.

use std::collections::BTreeSet;

use bevy::prelude::*;
use bevy::sprite_render::AlphaMode2d;
use sc_sim::{Map, PlayerId, SU_PER_GU, ScenarioProblem, SpawnAt, Symmetry, TilePos};

use super::doc::EditorDoc;
use super::objects::{self, PlaceProblem};
use super::state::{EditorCursor, EditorProblems, EditorSelection, EditorUi, Selection};
use super::terrain;
use super::tools::{EditorTools, Tool};
use crate::camera::{CameraZoom, ViewportInsets, clamp_center, map_extent_su, viewport_rect, visible_extent_su};
use crate::coords::{DrawLayer, drawn_at_height, drawn_bounds, drawn_position, footprint_height, view_to_world};
use crate::hud::HUD_INSETS;
use crate::map_render::{SURFACE_LIFT_Z, surface_vertex, tile_surface};
use crate::object_render::placeholder_mesh;
use crate::platform::SurfaceSize;
use crate::shapes::ColorMesh;
use crate::{ClientMode, ClientSystems};

/// Placeholder colours of player slots 1 to 8 (until Q0001 settles the art).
pub const SLOT_COLORS: [Color; 8] = [
    Color::srgb(0.30, 0.55, 1.00),
    Color::srgb(1.00, 0.35, 0.30),
    Color::srgb(0.35, 0.85, 0.40),
    Color::srgb(1.00, 0.85, 0.25),
    Color::srgb(0.75, 0.45, 1.00),
    Color::srgb(0.30, 0.90, 0.90),
    Color::srgb(1.00, 0.60, 0.20),
    Color::srgb(1.00, 0.50, 0.80),
];
const BRUSH_FILL: Color = Color::srgba(1.0, 1.0, 1.0, 0.16);
const BRUSH_EDGE: Color = Color::srgba(1.0, 1.0, 1.0, 0.85);
const GOOD_FILL: Color = Color::srgba(0.4, 1.0, 0.5, 0.20);
const BAD_FILL: Color = Color::srgba(1.0, 0.25, 0.2, 0.35);
const BAD_EDGE: Color = Color::srgba(1.0, 0.3, 0.25, 0.95);
const SELECTION_EDGE: Color = Color::srgb(1.0, 0.9, 0.2);
const AXIS_COLOR: Color = Color::srgba(1.0, 1.0, 1.0, 0.45);
/// A height dab's preview: the corners it sets, and the corners the slopes around them move.
const TARGET_DOT: Color = Color::srgb(1.0, 1.0, 1.0);
const CASCADE_DOT: Color = Color::srgb(1.0, 0.8, 0.35);
/// The slope tint (placeholder colours): flat, sloped within the maximum slope, too steep for the tile's preset.
const FLAT_TINT: Color = Color::srgba(0.25, 0.95, 0.35, 0.22);
const SLOPED_TINT: Color = Color::srgba(1.0, 0.72, 0.1, 0.32);
const STEEP_TINT: Color = Color::srgba(1.0, 0.1, 0.1, 0.5);
/// Line thickness in screen pixels.
const LINE_PX: f32 = 2.0;
/// Size of the slot labels in screen pixels.
const LABEL_PX: f32 = 14.0;
/// The most ghosts at once: a placement and its images under a four-fold symmetry.
const MAX_GHOSTS: usize = 4;

/// A slot's colour.
pub fn slot_color(slot: PlayerId) -> Color {
    SLOT_COLORS[usize::from(slot.0.max(1) - 1) % SLOT_COLORS.len()]
}

#[derive(Component)]
pub struct OverlayMesh;

/// The slope tint's mesh (K).
#[derive(Component)]
pub struct SlopeTint;

/// The ghost of the object to be placed (`image` 0) or of one of its symmetry images.
#[derive(Component)]
pub struct Ghost {
    pub image: usize,
}

#[derive(Component)]
pub struct SlotLabel;

#[derive(Resource)]
struct OverlayAssets {
    mesh: Handle<Mesh>,
    slope_mesh: Handle<Mesh>,
    ghost_ok: Handle<ColorMaterial>,
    ghost_bad: Handle<ColorMaterial>,
}

pub struct EditorOverlayPlugin;

impl Plugin for EditorOverlayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_overlay).add_systems(
            Update,
            (draw_overlay, draw_slopes, place_ghosts, place_slot_labels).in_set(ClientSystems::Present),
        );
    }
}

fn blend(color: Color) -> ColorMaterial {
    ColorMaterial {
        color,
        alpha_mode: AlphaMode2d::Blend,
        ..default()
    }
}

fn spawn_overlay(mut commands: Commands, meshes: Res<Assets<Mesh>>, mut materials: ResMut<Assets<ColorMaterial>>) {
    // The mesh assets arrive with the first overlay that has something to draw: the renderer warns about empty
    // meshes.
    let mesh = meshes.reserve_handle();
    let slope_mesh = meshes.reserve_handle();
    commands.spawn((
        Name::new("Editor overlay"),
        OverlayMesh,
        Mesh2d(mesh.clone()),
        MeshMaterial2d(materials.add(blend(Color::WHITE))),
        Transform::from_xyz(0.0, 0.0, DrawLayer::Overlays.base_z()),
        Visibility::Hidden,
    ));
    // In the terrain's z band, so that the terrain in front of a tile hides its tint.
    commands.spawn((
        Name::new("Editor slope tint"),
        SlopeTint,
        Mesh2d(slope_mesh.clone()),
        MeshMaterial2d(materials.add(blend(Color::WHITE))),
        Transform::from_xyz(0.0, 0.0, DrawLayer::Terrain.base_z()),
        Visibility::Hidden,
    ));
    let assets = OverlayAssets {
        mesh,
        slope_mesh,
        ghost_ok: materials.add(blend(Color::srgba(1.0, 1.0, 1.0, 0.6))),
        ghost_bad: materials.add(blend(Color::srgba(1.0, 0.3, 0.25, 0.7))),
    };
    for image in 0..MAX_GHOSTS {
        commands.spawn((
            Name::new("Editor ghost"),
            Ghost { image },
            Mesh2d::default(),
            MeshMaterial2d(assets.ghost_ok.clone()),
            Transform::default(),
            Visibility::Hidden,
        ));
    }
    commands.insert_resource(assets);
}

/// Space units per screen pixel of the editor's view.
fn su_per_pixel(size: SurfaceSize, insets: ViewportInsets, zoom: f32) -> f32 {
    let (_, viewport) = viewport_rect(size, insets);
    visible_extent_su(viewport, size.physical.x, zoom).x / viewport.x.max(1) as f32 * size.scale_factor
}

/// Builds the overlay from shapes in view space (where the terrain is drawn), into Bevy world coordinates.
struct Overlay<'a> {
    map: &'a Map,
    mesh: ColorMesh,
    /// Line thickness in space units.
    line: f32,
}

impl Overlay<'_> {
    fn polygon(&mut self, points: &[Vec2], z: f32, color: Color) {
        let world: Vec<Vec2> = points.iter().map(|p| view_to_world(*p)).collect();
        self.mesh.polygon(&world, z, color);
    }

    fn segment(&mut self, a: Vec2, b: Vec2, z: f32, color: Color) {
        self.mesh
            .segment(view_to_world(a), view_to_world(b), self.line, z, color);
    }

    /// Where tile `t` is drawn: its NW, NE, SE and SW corners and its center ([`tile_surface`]).
    fn drawn_tile(&self, t: TilePos) -> [Vec2; 5] {
        tile_surface(self.map, t).map(|(ground, height)| drawn_at_height(ground, height))
    }

    /// Fills tile `t` where the terrain draws it.
    fn tile_fill(&mut self, t: TilePos, color: Color) {
        let [nw, ne, se, sw, center] = self.drawn_tile(t);
        for (a, b) in [(nw, ne), (ne, se), (se, sw), (sw, nw)] {
            self.polygon(&[a, b, center], 0.0, color);
        }
    }

    /// Outlines the area `tiles` cover where the terrain draws it: every tile edge without a neighbour in the set.
    fn tiles_outline(&mut self, tiles: &BTreeSet<TilePos>, color: Color) {
        for &t in tiles {
            let [nw, ne, se, sw, _] = self.drawn_tile(t);
            for ((dx, dy), a, b) in [((0, -1), nw, ne), ((1, 0), ne, se), ((0, 1), se, sw), ((-1, 0), sw, nw)] {
                let neighbour = t.x.checked_add(dx).zip(t.y.checked_add(dy));
                if !neighbour.is_some_and(|(x, y)| tiles.contains(&TilePos::new(x, y))) {
                    self.segment(a, b, 0.001, color);
                }
            }
        }
    }

    /// The ground segment from `a` to `b`, where the terrain draws it.
    fn surface_line(&mut self, a: Vec2, b: Vec2, z: f32, color: Color) {
        for pair in surface_path(self.map, a, b).windows(2) {
            self.segment(pair[0], pair[1], z, color);
        }
    }

    /// Outlines the ground rectangle `rect` where the terrain draws it.
    fn surface_outline(&mut self, rect: Rect, color: Color) {
        let corners = rect_corners(rect);
        for i in 0..4 {
            self.surface_line(corners[i], corners[(i + 1) % 4], 0.001, color);
        }
    }

    /// Outlines a rectangle of view space.
    fn view_outline(&mut self, rect: Rect, color: Color) {
        let corners = rect_corners(rect);
        for i in 0..4 {
            self.segment(corners[i], corners[(i + 1) % 4], 0.001, color);
        }
    }

    /// A cross at the view point `at`.
    fn cross(&mut self, at: Vec2, half: f32, color: Color) {
        self.segment(at - Vec2::new(half, 0.0), at + Vec2::new(half, 0.0), 0.002, color);
        self.segment(at - Vec2::new(0.0, half), at + Vec2::new(0.0, half), 0.002, color);
    }

    /// A square dot `size` across at the view point `at`.
    fn dot(&mut self, at: Vec2, size: f32, color: Color) {
        self.polygon(
            &rect_corners(Rect::from_center_size(at, Vec2::splat(size))),
            0.003,
            color,
        );
    }
}

/// A rectangle's corners, around it.
fn rect_corners(rect: Rect) -> [Vec2; 4] {
    let (a, b) = (rect.min, rect.max);
    [a, Vec2::new(b.x, a.y), b, Vec2::new(a.x, b.y)]
}

/// The view points along the ground segment from `a` to `b` where its drawn path bends: its ends and its crossings
/// of the tile grid on the map, between which the height along an axis-aligned segment is linear.
fn surface_path(map: &Map, a: Vec2, b: Vec2) -> Vec<Vec2> {
    if !a.is_finite() || !b.is_finite() {
        return Vec::new();
    }
    let extent = map_extent_su(map);
    let mut ts = vec![0.0, 1.0];
    for (from, to, size) in [(a.x, b.x, extent.x), (a.y, b.y, extent.y)] {
        if from == to {
            continue;
        }
        // Only grid lines on the map: off it the ground is flat along the segment.
        let (lo, hi) = (from.min(to).max(0.0), from.max(to).min(size));
        for k in (lo / SU_PER_GU).ceil() as i64..=(hi / SU_PER_GU).floor() as i64 {
            let t = (k as f32 * SU_PER_GU - from) / (to - from);
            if t > 0.0 && t < 1.0 {
                ts.push(t);
            }
        }
    }
    ts.sort_by(f32::total_cmp);
    ts.dedup();
    ts.into_iter().map(|t| drawn_position(map, a.lerp(b, t))).collect()
}

/// Whether the ground rectangle `rect` is near enough to the map to draw: a damaged file's spawn may lie anywhere.
fn near_map(map: &Map, rect: Rect) -> bool {
    let area = Rect::from_corners(Vec2::ZERO, map_extent_su(map)).inflate(SU_PER_GU * 4.0);
    rect.min.is_finite() && rect.max.is_finite() && !area.intersect(rect).is_empty()
}

fn tile_rect(t: TilePos) -> Rect {
    Rect::from_corners(t.corner(), t.corner() + Vec2::splat(SU_PER_GU))
}

#[expect(clippy::too_many_arguments, reason = "a system's parameters")]
fn draw_overlay(
    mode: Res<ClientMode>,
    doc: Res<EditorDoc>,
    tools: Res<EditorTools>,
    cursor: Res<EditorCursor>,
    selection: Res<EditorSelection>,
    problems: Res<EditorProblems>,
    view: (Res<SurfaceSize>, Res<ViewportInsets>, Res<CameraZoom>),
    assets: Res<OverlayAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut overlay: Query<&mut Visibility, With<OverlayMesh>>,
) {
    let (size, insets, zoom) = view;
    if *mode != ClientMode::Editor {
        for mut visibility in &mut overlay {
            visibility.set_if_neq(Visibility::Hidden);
        }
        return;
    }
    let changed = doc.is_changed()
        || tools.is_changed()
        || cursor.is_changed()
        || selection.is_changed()
        || problems.is_changed()
        || zoom.is_changed()
        || size.is_changed()
        || mode.is_changed();
    if !changed {
        return;
    }
    let scenario = doc.scenario();
    let map = &scenario.map;
    let mut o = Overlay {
        map,
        mesh: ColorMesh::triangles(),
        line: LINE_PX * su_per_pixel(*size, *insets, zoom.0),
    };

    symmetry_axes(&mut o, tools.symmetry);

    // Player slots: the start location tile, and the camera start with the game's view around it, kept on the
    // map's drawn area as the game keeps it.
    let game_view = {
        let (_, viewport) = viewport_rect(*size, HUD_INSETS);
        visible_extent_su(viewport, size.physical.x, 1.0)
    };
    let bounds = drawn_bounds(map);
    let game_rect =
        |pos: Vec2| Rect::from_center_size(clamp_center(drawn_position(map, pos), game_view, bounds), game_view);
    let marker = SU_PER_GU * 0.4;
    for slot in &scenario.players {
        let color = slot_color(slot.id);
        if let Some(tile) = slot.start_location.filter(|t| map.in_bounds(*t)) {
            o.tile_fill(tile, color.with_alpha(0.55));
            o.tiles_outline(&BTreeSet::from([tile]), color);
        }
        if let Some(pos) = slot.camera_start.filter(|p| map.contains_point(*p)) {
            o.cross(drawn_position(map, pos), marker, color);
            o.view_outline(game_rect(pos), color.with_alpha(0.45));
        }
    }

    // Spawns and tiles with problems.
    for problem in &problems.0 {
        let spawns = match problem {
            ScenarioProblem::InvalidSpawn { spawn, .. } => vec![*spawn],
            ScenarioProblem::OverlappingFootprints { first, second, .. } => vec![*first, *second],
            ScenarioProblem::TooSteep { tile, .. } if map.in_bounds(*tile) => {
                o.tiles_outline(&BTreeSet::from([*tile]), BAD_EDGE);
                Vec::new()
            }
            _ => Vec::new(),
        };
        for s in spawns.into_iter().filter_map(|i| scenario.spawns.get(i)) {
            let rect = objects::spawn_rect(s.kind, s.at);
            if near_map(map, rect) {
                o.surface_outline(rect, BAD_EDGE);
            }
        }
    }

    // The selection.
    let selected = match selection.0 {
        Some(Selection::Spawn(i)) => scenario.spawns.get(i).map(|s| objects::spawn_rect(s.kind, s.at)),
        Some(Selection::StartLocation(p)) => {
            let slot = scenario.players.iter().find(|s| s.id == p);
            slot.and_then(|s| s.start_location).map(tile_rect)
        }
        Some(Selection::CameraStart(p)) => {
            let slot = scenario.players.iter().find(|s| s.id == p);
            if let Some(c) = slot.and_then(|s| s.camera_start).filter(|c| c.is_finite()) {
                let around = Rect::from_center_size(drawn_position(map, c), Vec2::splat(SU_PER_GU));
                o.view_outline(around, SELECTION_EDGE);
            }
            None
        }
        None => None,
    };
    if let Some(rect) = selected.map(|r| r.inflate(o.line * 2.0)).filter(|r| near_map(map, *r)) {
        o.surface_outline(rect, SELECTION_EDGE);
    }

    // What the tool would do under the cursor.
    if let Some(pos) = cursor.0 {
        let tile = TilePos::containing(pos);
        match tools.tool {
            Tool::Paint => paint_preview(&mut o, &tools, tile),
            Tool::Elevation => height_preview(&mut o, &tools, tile),
            Tool::Fill => {
                let tiles = terrain::image_tiles(map, tile, tools.symmetry)
                    .into_iter()
                    .filter(|t| map.in_bounds(*t));
                for t in tiles {
                    o.tiles_outline(&BTreeSet::from([t]), BRUSH_EDGE);
                }
            }
            Tool::Place => {
                for (at, problem) in placements(&doc, &tools, pos) {
                    placement_preview(&mut o, tools.kind, at, problem);
                }
            }
            Tool::StartLocation if map.in_bounds(tile) => {
                o.tiles_outline(&BTreeSet::from([tile]), slot_color(tools.slot));
            }
            Tool::CameraStart if map.contains_point(pos) => {
                o.cross(drawn_position(map, pos), marker, slot_color(tools.slot));
                o.view_outline(game_rect(pos), slot_color(tools.slot).with_alpha(0.6));
            }
            _ => {}
        }
    }
    let empty = o.mesh.vertex_count() == 0;
    if !empty && let Err(error) = meshes.insert(&assets.mesh, o.mesh.into_mesh()) {
        error!("editor overlay: {error}");
    }
    for mut visibility in &mut overlay {
        visibility.set_if_neq(if empty {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        });
    }
}

/// The symmetry's mirror axes, following the surface, or its center of rotation (`core/map_editor.md >
/// Symmetry`).
fn symmetry_axes(o: &mut Overlay, symmetry: Symmetry) {
    let extent = map_extent_su(o.map);
    let center = extent / 2.0;
    let vertical = matches!(symmetry, Symmetry::MirrorVerticalAxis | Symmetry::MirrorBothAxes);
    let horizontal = matches!(symmetry, Symmetry::MirrorHorizontalAxis | Symmetry::MirrorBothAxes);
    if vertical {
        o.surface_line(
            Vec2::new(center.x, 0.0),
            Vec2::new(center.x, extent.y),
            0.002,
            AXIS_COLOR,
        );
    }
    if horizontal {
        o.surface_line(
            Vec2::new(0.0, center.y),
            Vec2::new(extent.x, center.y),
            0.002,
            AXIS_COLOR,
        );
    }
    let drawn_center = drawn_position(o.map, center);
    match symmetry {
        Symmetry::Rotate180 => o.cross(drawn_center, SU_PER_GU, AXIS_COLOR),
        Symmetry::Rotate90 => {
            o.cross(drawn_center, SU_PER_GU, AXIS_COLOR);
            o.view_outline(Rect::from_center_size(drawn_center, Vec2::splat(SU_PER_GU)), AXIS_COLOR);
        }
        _ => {}
    }
}

/// The Paint tool's dab under the cursor: its tiles with their images, red where the preset would be refused as
/// too steep ([`terrain::refuses`]).
fn paint_preview(o: &mut Overlay, tools: &EditorTools, tile: TilePos) {
    let map = o.map;
    let tiles = terrain::dab_tiles(map, tile, tools.brush, tools.symmetry);
    for &t in &tiles {
        let changes = map.tile(t).is_some_and(|current| current.preset != tools.preset);
        let refused = changes && terrain::refuses(map, t, tools.preset);
        o.tile_fill(t, if refused { BAD_FILL } else { BRUSH_FILL });
    }
    o.tiles_outline(&tiles, BRUSH_EDGE);
}

/// The Height tool's dab under the cursor: its tiles with their images, and every corner it would move, from
/// where it is drawn now to where it would be: white for the corners the brush sets, amber for the ones the slopes
/// around them move ([`Map::set_corner_heights`] on a copy of the map).
fn height_preview(o: &mut Overlay, tools: &EditorTools, tile: TilePos) {
    let map = o.map;
    let tiles = terrain::dab_tiles(map, tile, tools.brush, tools.symmetry);
    for &t in &tiles {
        o.tile_fill(t, BRUSH_FILL);
    }
    o.tiles_outline(&tiles, BRUSH_EDGE);
    let corners = terrain::dab_corners(map, tile, tools.brush, tools.symmetry);
    let targets = terrain::height_targets(map, corners, tools.elevation_mode, tools.elevation);
    let mut after = map.clone();
    let size = o.line * 2.5;
    for c in terrain::shape(&mut after, &targets) {
        let (Some(from), Some(to)) = (map.corner(c), after.corner(c)) else {
            continue;
        };
        let (a, b) = (
            drawn_at_height(c.pos(), f32::from(from)),
            drawn_at_height(c.pos(), f32::from(to)),
        );
        let color = if targets.contains_key(&c) {
            TARGET_DOT
        } else {
            CASCADE_DOT
        };
        o.segment(a, b, 0.003, color.with_alpha(0.6));
        o.dot(b, size, color);
    }
}

/// One placement of the Place tool: its footprint on the surface, green where it fits, red where it doesn't, with
/// its sloped tiles marked when it needs flat ground.
fn placement_preview(o: &mut Overlay, kind: sc_sim::ObjectKind, at: SpawnAt, problem: Option<PlaceProblem>) {
    let map = o.map;
    let rect = objects::spawn_rect(kind, at);
    if !near_map(map, rect) {
        return;
    }
    let (fill, edge) = match problem {
        None => (GOOD_FILL, BRUSH_EDGE),
        Some(_) => (BAD_FILL, BAD_EDGE),
    };
    let tiles: BTreeSet<TilePos> = objects::footprint_tiles(kind, at).into_iter().collect();
    if tiles.is_empty() {
        // A unit: its silhouette's rectangle.
        let corners = rect_corners(rect).map(|p| drawn_position(map, p));
        o.polygon(&corners, 0.0, fill);
        o.surface_outline(rect, edge);
        return;
    }
    for &t in &tiles {
        o.tile_fill(t, fill);
        if matches!(problem, Some(PlaceProblem::NotFlat(_))) && !map.is_flat(t) {
            o.tile_fill(t, BAD_FILL);
        }
    }
    o.tiles_outline(&tiles, edge);
}

/// Where the Place tool would put the object for the cursor at `pos`, then its symmetry images, each with why it
/// can't go there.
fn placements(doc: &EditorDoc, tools: &EditorTools, pos: Vec2) -> Vec<(SpawnAt, Option<PlaceProblem>)> {
    let scenario = doc.scenario();
    let check = |at| objects::check_placement(scenario, tools.kind, at, None).err();
    let at = objects::snap_spawn(tools.kind, pos);
    let mut found = vec![(at, check(at))];
    for (_, image) in objects::spawn_images(&scenario.map, tools.kind, at, tools.symmetry) {
        found.push((image, check(image)));
    }
    found
}

/// The slope tint: each tile green if flat, amber if sloped within the maximum slope, red if too steep for its
/// preset (`core/map.md > Elevation`), lying on the surface in the terrain's z band.
pub fn slope_mesh(map: &Map) -> ColorMesh {
    let mut mesh = ColorMesh::triangles();
    for y in 0..map.height() as i32 {
        for x in 0..map.width() as i32 {
            let t = TilePos::new(x, y);
            let flat = map.is_flat(t);
            let color = if map.is_too_steep(t) {
                STEEP_TINT
            } else if flat {
                FLAT_TINT
            } else {
                SLOPED_TINT
            };
            let [nw, ne, se, sw, center] =
                tile_surface(map, t).map(|(ground, height)| surface_vertex(ground, height, SURFACE_LIFT_Z));
            if flat {
                mesh.quad([nw, ne, se, sw], color);
            } else {
                for (a, b) in [(nw, ne), (ne, se), (se, sw), (sw, nw)] {
                    mesh.triangle([a, b, center], color);
                }
            }
        }
    }
    mesh
}

/// Shows the slope tint while it is on (K), rebuilt when the document changed since it was built.
fn draw_slopes(
    mode: Res<ClientMode>,
    doc: Res<EditorDoc>,
    ui: Res<EditorUi>,
    assets: Res<OverlayAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut built: Local<Option<u64>>,
    mut tint: Query<&mut Visibility, With<SlopeTint>>,
) {
    let visible = *mode == ClientMode::Editor && ui.slopes;
    if visible && *built != Some(doc.revision()) {
        let mesh = slope_mesh(&doc.scenario().map);
        if mesh.vertex_count() > 0
            && let Err(error) = meshes.insert(&assets.slope_mesh, mesh.into_mesh())
        {
            error!("editor slope tint: {error}");
        }
        *built = Some(doc.revision());
    }
    for mut visibility in &mut tint {
        visibility.set_if_neq(if visible {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
}

#[expect(clippy::too_many_arguments, reason = "a system's parameters")]
fn place_ghosts(
    mode: Res<ClientMode>,
    doc: Res<EditorDoc>,
    tools: Res<EditorTools>,
    cursor: Res<EditorCursor>,
    assets: Res<OverlayAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut ghost_meshes: Local<Vec<(sc_sim::ObjectKind, Handle<Mesh>)>>,
    mut ghosts: Query<(
        &Ghost,
        &mut Mesh2d,
        &mut MeshMaterial2d<ColorMaterial>,
        &mut Transform,
        &mut Visibility,
    )>,
) {
    let shown = match (*mode, tools.tool, cursor.0) {
        (ClientMode::Editor, Tool::Place, Some(pos)) => placements(&doc, &tools, pos),
        _ => Vec::new(),
    };
    let kind = tools.kind;
    let mesh = match ghost_meshes.iter().find(|(k, _)| *k == kind) {
        Some((_, handle)) => handle.clone(),
        None => {
            let handle = meshes.add(placeholder_mesh(kind));
            ghost_meshes.push((kind, handle.clone()));
            handle
        }
    };
    let map = &doc.scenario().map;
    for (ghost, mut mesh2d, mut material, mut transform, mut visibility) in &mut ghosts {
        let Some(&(at, problem)) = shown.get(ghost.image) else {
            visibility.set_if_neq(Visibility::Hidden);
            continue;
        };
        // Standing where the object would: on its footprint's height.
        let height = match (at, objects::tile_size(kind)) {
            (SpawnAt::Tile(o), Some((w, h))) => footprint_height(map, o, w, h),
            _ => map.height_at(objects::spawn_rect(kind, at).center()),
        };
        let center = drawn_at_height(objects::spawn_rect(kind, at).center(), height);
        let z = DrawLayer::Overlays.base_z() - 1.0;
        transform.set_if_neq(Transform::from_translation(view_to_world(center).extend(z)));
        if mesh2d.0 != mesh {
            mesh2d.0 = mesh.clone();
        }
        // `core/map.md > Elevation > Map resources only on flat terrain`: red on any sloped footprint tile too.
        let wanted = if problem.is_none() {
            &assets.ghost_ok
        } else {
            &assets.ghost_bad
        };
        if material.0 != *wanted {
            material.0 = wanted.clone();
        }
        visibility.set_if_neq(Visibility::Inherited);
    }
}

/// A slot marker's label: its text, slot, and the view point it hangs from (below) or stands on (above).
#[derive(Clone, PartialEq)]
struct Label {
    text: String,
    slot: PlayerId,
    at: Vec2,
    below: bool,
}

/// Labels "P1 start" under each start location and "P1 camera" over each camera start, at a constant size on
/// screen.
fn place_slot_labels(
    mut commands: Commands,
    mode: Res<ClientMode>,
    doc: Res<EditorDoc>,
    view: (Res<SurfaceSize>, Res<ViewportInsets>, Res<CameraZoom>),
    labels: Query<Entity, With<SlotLabel>>,
    mut shown: Local<Option<(Vec<Label>, f32)>>,
) {
    let (size, insets, zoom) = view;
    // Space units per screen pixel: the labels' scale and the unit of their distance from the marker.
    let scale = su_per_pixel(*size, *insets, zoom.0);
    let gap = 3.0 * scale;
    let mut wanted = Vec::new();
    if *mode == ClientMode::Editor {
        let map = &doc.scenario().map;
        for slot in &doc.scenario().players {
            if let Some(t) = slot.start_location.filter(|t| map.in_bounds(*t)) {
                let bottom = drawn_position(map, t.corner() + Vec2::new(SU_PER_GU / 2.0, SU_PER_GU));
                wanted.push(Label {
                    text: format!("P{} start", slot.id.0),
                    slot: slot.id,
                    at: bottom + Vec2::new(0.0, gap),
                    below: true,
                });
            }
            if let Some(p) = slot.camera_start.filter(|p| map.contains_point(*p)) {
                wanted.push(Label {
                    text: format!("P{} camera", slot.id.0),
                    slot: slot.id,
                    at: drawn_position(map, p) - Vec2::new(0.0, SU_PER_GU * 0.4 + gap),
                    below: false,
                });
            }
        }
    }
    let wanted = (wanted, scale);
    if shown.as_ref() == Some(&wanted) {
        return;
    }
    for entity in &labels {
        commands.entity(entity).despawn();
    }
    for label in &wanted.0 {
        let anchor = if label.below {
            bevy::sprite::Anchor::TOP_CENTER
        } else {
            bevy::sprite::Anchor::BOTTOM_CENTER
        };
        commands.spawn((
            Name::new("Slot label"),
            SlotLabel,
            Text2d::new(label.text.clone()),
            TextFont::from_font_size(FontSize::Px(LABEL_PX)),
            TextColor(slot_color(label.slot)),
            anchor,
            Transform::from_translation(view_to_world(label.at).extend(DrawLayer::Overlays.base_z() + 1.0))
                .with_scale(Vec3::splat(scale)),
        ));
    }
    *shown = Some(wanted);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn surface_paths_bend_at_the_grid_lines_on_the_map() {
        // A ramp rising 0 → 2 → 4 eastwards over two tiles.
        let map = Map::from_ascii("..\n\n024\n024").unwrap();
        let path = surface_path(&map, Vec2::new(-64.0, 32.0), Vec2::new(256.0, 32.0));
        let xs: Vec<f32> = path.iter().map(|p| p.x).collect();
        assert_eq!(
            xs,
            [-64.0, 0.0, 64.0, 128.0, 256.0],
            "the ends, then the grid lines on the map only"
        );
        assert_eq!(path[1], drawn_position(&map, Vec2::new(0.0, 32.0)));
        assert_eq!(path[3].y, 32.0 - crate::coords::lift(4.0));
        // A damaged file's spawn far away costs no more than the map's size.
        let far = surface_path(&map, Vec2::new(-1e30, 0.0), Vec2::new(1e30, 0.0));
        assert!(far.len() <= 5);
        assert!(surface_path(&map, Vec2::NAN, Vec2::ZERO).is_empty());
        assert!(!near_map(&map, Rect::new(1e12, 1e12, 1e12 + 64.0, 1e12 + 64.0)));
        assert!(near_map(&map, Rect::new(-64.0, -64.0, 0.0, 0.0)));
    }

    #[test]
    fn the_slope_tint_has_a_quad_per_flat_tile_and_four_triangles_per_sloped_one() {
        let map = Map::from_ascii("...\n\n0011\n0011").unwrap();
        // Tiles (0, 0) and (2, 0) are flat, (1, 0) slopes.
        assert_eq!(slope_mesh(&map).vertex_count(), 4 + 12 + 4);
    }
}
