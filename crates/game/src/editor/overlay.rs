//! What the editor draws over the map: the brush footprint under the cursor, the ghost of an object about to be
//! placed (red where it can't go), the selection, spawns with problems, the symmetry axes, and each player slot's
//! start location and camera start. One overlay mesh is rebuilt when any of that changes; the ghosts reuse the
//! object placeholder meshes with a translucent material; the slot labels are world-space text kept at a constant
//! screen size.

use bevy::prelude::*;
use bevy::sprite_render::AlphaMode2d;
use sc_sim::{PlayerId, SU_PER_GU, ScenarioProblem, SpawnAt, TilePos};

use super::doc::EditorDoc;
use super::state::{EditorCursor, EditorProblems, EditorSelection, Selection};
use super::tools::{EditorTools, Symmetry, Tool};
use super::{objects, terrain};
use crate::camera::{CameraZoom, ViewportInsets, clamp_center, map_extent_su, viewport_rect, visible_extent_su};
use crate::coords::{DrawLayer, sim_to_world};
use crate::hud::HUD_INSETS;
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
const BAD_FILL: Color = Color::srgba(1.0, 0.25, 0.2, 0.30);
const BAD_EDGE: Color = Color::srgba(1.0, 0.3, 0.25, 0.95);
const SELECTION_EDGE: Color = Color::srgb(1.0, 0.9, 0.2);
const AXIS_COLOR: Color = Color::srgba(1.0, 1.0, 1.0, 0.35);
/// Line thickness in screen pixels.
const LINE_PX: f32 = 2.0;
/// Size of the slot labels in screen pixels.
const LABEL_PX: f32 = 14.0;

/// A slot's colour.
pub fn slot_color(slot: PlayerId) -> Color {
    SLOT_COLORS[usize::from(slot.0.max(1) - 1) % SLOT_COLORS.len()]
}

#[derive(Component)]
pub struct OverlayMesh;

/// The ghost of the object to be placed, and of its mirror image.
#[derive(Component)]
pub struct Ghost {
    pub mirror: bool,
}

#[derive(Component)]
pub struct SlotLabel;

#[derive(Resource)]
struct OverlayAssets {
    mesh: Handle<Mesh>,
    ghost_ok: Handle<ColorMaterial>,
    ghost_bad: Handle<ColorMaterial>,
}

pub struct EditorOverlayPlugin;

impl Plugin for EditorOverlayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_overlay).add_systems(
            Update,
            (draw_overlay, place_ghosts, place_slot_labels).in_set(ClientSystems::Present),
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
    // The mesh asset arrives with the first overlay that has something to draw: the renderer warns about empty
    // meshes.
    let mesh = meshes.reserve_handle();
    commands.spawn((
        Name::new("Editor overlay"),
        OverlayMesh,
        Mesh2d(mesh.clone()),
        MeshMaterial2d(materials.add(blend(Color::WHITE))),
        Transform::from_xyz(0.0, 0.0, DrawLayer::Overlays.base_z()),
        Visibility::Hidden,
    ));
    let assets = OverlayAssets {
        mesh,
        ghost_ok: materials.add(blend(Color::srgba(1.0, 1.0, 1.0, 0.6))),
        ghost_bad: materials.add(blend(Color::srgba(1.0, 0.35, 0.3, 0.6))),
    };
    for mirror in [false, true] {
        commands.spawn((
            Name::new("Editor ghost"),
            Ghost { mirror },
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

/// Builds the overlay in sim space: rectangles are given by their sim corners.
struct Overlay {
    mesh: ColorMesh,
    /// Line thickness in space units.
    line: f32,
}

impl Overlay {
    fn fill(&mut self, rect: Rect, color: Color) {
        self.mesh
            .rect(sim_to_world(rect.min), sim_to_world(rect.max), 0.0, color);
    }

    /// An outline just inside `rect`.
    fn outline(&mut self, rect: Rect, color: Color) {
        let t = self.line.min(rect.width() / 2.0).min(rect.height() / 2.0);
        let (a, b) = (rect.min, rect.max);
        for (p, q) in [
            (a, Vec2::new(b.x, a.y + t)),
            (Vec2::new(a.x, b.y - t), b),
            (Vec2::new(a.x, a.y + t), Vec2::new(a.x + t, b.y - t)),
            (Vec2::new(b.x - t, a.y + t), Vec2::new(b.x, b.y - t)),
        ] {
            self.mesh.rect(sim_to_world(p), sim_to_world(q), 0.001, color);
        }
    }

    fn cross(&mut self, at: Vec2, half: f32, color: Color) {
        let t = self.line / 2.0;
        self.mesh.rect(
            sim_to_world(at - Vec2::new(half, t)),
            sim_to_world(at + Vec2::new(half, t)),
            0.002,
            color,
        );
        self.mesh.rect(
            sim_to_world(at - Vec2::new(t, half)),
            sim_to_world(at + Vec2::new(t, half)),
            0.002,
            color,
        );
    }
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
        mesh: ColorMesh::triangles(),
        line: LINE_PX * su_per_pixel(*size, *insets, zoom.0),
    };

    // The symmetry axes.
    let extent = map_extent_su(map);
    let half = extent / 2.0;
    let t = o.line / 2.0;
    if matches!(tools.symmetry, Symmetry::MirrorX) {
        o.fill(Rect::new(half.x - t, 0.0, half.x + t, extent.y), AXIS_COLOR);
    }
    if matches!(tools.symmetry, Symmetry::MirrorY) {
        o.fill(Rect::new(0.0, half.y - t, extent.x, half.y + t), AXIS_COLOR);
    }
    if matches!(tools.symmetry, Symmetry::Point) {
        o.cross(half, SU_PER_GU, AXIS_COLOR);
    }

    // Player slots: the start location tile and the camera start with the game's view around it, kept on the map
    // as the game keeps it.
    let game_view = {
        let (_, viewport) = viewport_rect(*size, HUD_INSETS);
        visible_extent_su(viewport, size.physical.x, 1.0)
    };
    let game_rect = |pos: Vec2| Rect::from_center_size(clamp_center(pos, game_view, extent), game_view);
    for slot in &scenario.players {
        let color = slot_color(slot.id);
        if let Some(tile) = slot.start_location.filter(|t| map.in_bounds(*t)) {
            o.fill(tile_rect(tile), color.with_alpha(0.55));
            o.outline(tile_rect(tile), color);
        }
        if let Some(pos) = slot.camera_start.filter(|p| map.contains_point(*p)) {
            o.cross(pos, SU_PER_GU * 0.4, color);
            o.outline(game_rect(pos), color.with_alpha(0.45));
        }
    }

    // Spawns with problems.
    for problem in &problems.0 {
        let spawns = match problem {
            ScenarioProblem::InvalidSpawn { spawn, .. } => vec![*spawn],
            ScenarioProblem::OverlappingFootprints { first, second, .. } => vec![*first, *second],
            _ => Vec::new(),
        };
        for s in spawns.into_iter().filter_map(|i| scenario.spawns.get(i)) {
            o.outline(objects::spawn_rect(s.kind, s.at), BAD_EDGE);
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
            let pos = slot.and_then(|s| s.camera_start);
            pos.map(|c| Rect::from_center_size(c, Vec2::splat(SU_PER_GU)))
        }
        None => None,
    };
    if let Some(rect) = selected {
        o.outline(rect.inflate(o.line * 2.0), SELECTION_EDGE);
    }

    // What the tool would do under the cursor.
    if let Some(pos) = cursor.0 {
        let tile = TilePos::containing(pos);
        match tools.tool {
            Tool::Paint | Tool::Elevation => {
                for center in terrain::mirrored_tiles(map, tile, tools.symmetry) {
                    let tiles = terrain::brush_tiles(map, center, tools.brush);
                    if let (Some(first), Some(last)) = (tiles.first(), tiles.last()) {
                        let rect = tile_rect(*first).union(tile_rect(*last));
                        o.fill(rect, BRUSH_FILL);
                        o.outline(rect, BRUSH_EDGE);
                    }
                }
            }
            Tool::Fill => {
                for t in terrain::mirrored_tiles(map, tile, tools.symmetry) {
                    if map.in_bounds(t) {
                        o.outline(tile_rect(t), BRUSH_EDGE);
                    }
                }
            }
            Tool::Place => {
                for (at, ok) in placements(&doc, &tools, pos) {
                    let rect = objects::spawn_rect(tools.kind, at);
                    o.fill(rect, if ok { GOOD_FILL } else { BAD_FILL });
                    o.outline(rect, if ok { BRUSH_EDGE } else { BAD_EDGE });
                }
            }
            Tool::StartLocation if map.in_bounds(tile) => {
                o.outline(tile_rect(tile), slot_color(tools.slot));
            }
            Tool::CameraStart if map.contains_point(pos) => {
                o.cross(pos, SU_PER_GU * 0.4, slot_color(tools.slot));
                o.outline(game_rect(pos), slot_color(tools.slot).with_alpha(0.6));
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

/// Where the Place tool would put the object for the cursor at `pos`, and its mirror image, each with whether it
/// fits there.
fn placements(doc: &EditorDoc, tools: &EditorTools, pos: Vec2) -> Vec<(SpawnAt, bool)> {
    let scenario = doc.scenario();
    let at = objects::snap_spawn(tools.kind, pos);
    let mut found = vec![(at, objects::check_placement(scenario, tools.kind, at, None).is_ok())];
    if let Some(image) = objects::mirrored_spawn(&scenario.map, tools.kind, at, tools.symmetry) {
        found.push((
            image,
            objects::check_placement(scenario, tools.kind, image, None).is_ok(),
        ));
    }
    found
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
    for (ghost, mut mesh2d, mut material, mut transform, mut visibility) in &mut ghosts {
        let Some(&(at, ok)) = shown.get(usize::from(ghost.mirror)) else {
            visibility.set_if_neq(Visibility::Hidden);
            continue;
        };
        let center = objects::spawn_rect(kind, at).center();
        let z = DrawLayer::Overlays.base_z() - 1.0;
        transform.set_if_neq(Transform::from_translation(sim_to_world(center).extend(z)));
        if mesh2d.0 != mesh {
            mesh2d.0 = mesh.clone();
        }
        let wanted = if ok { &assets.ghost_ok } else { &assets.ghost_bad };
        if material.0 != *wanted {
            material.0 = wanted.clone();
        }
        visibility.set_if_neq(Visibility::Inherited);
    }
}

/// A slot marker's label: its text, slot, and the sim point it hangs from (below) or stands on (above).
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
                wanted.push(Label {
                    text: format!("P{} start", slot.id.0),
                    slot: slot.id,
                    at: t.corner() + Vec2::new(SU_PER_GU / 2.0, SU_PER_GU + gap),
                    below: true,
                });
            }
            if let Some(p) = slot.camera_start.filter(|p| map.contains_point(*p)) {
                wanted.push(Label {
                    text: format!("P{} camera", slot.id.0),
                    slot: slot.id,
                    at: p - Vec2::new(0.0, SU_PER_GU * 0.4 + gap),
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
            Transform::from_translation(sim_to_world(label.at).extend(DrawLayer::Overlays.base_z() + 1.0))
                .with_scale(Vec3::splat(scale)),
        ));
    }
    *shown = Some(wanted);
}
