//! What the editor draws over the map: what the tool under the cursor would change ([`previews`]), the
//! selection, spawns and tiles with problems, the symmetry axes and the dimmed locked part of a symmetric map
//! ([`tints`]), each player slot's start position and camera start with their labels ([`markers`]), and on request
//! (K) a tint of flat, sloped and too steep tiles.
//!
//! Everything sits where the terrain is drawn (`crate::coords::drawn_position`): outlines follow the surface. One
//! overlay mesh, on top of everything, is rebuilt when any of that changes. The tints are meshes of their own in the
//! terrain's z band, like the grid, so the terrain in front hides them; they are rebuilt when the map changes. The
//! ghosts reuse the object placeholder meshes with a translucent material; the slot labels are world-space text
//! kept at a constant screen size.

pub mod markers;
mod previews;
mod shapes;
pub mod tints;

use std::collections::BTreeSet;

use bevy::prelude::*;
use bevy::sprite_render::AlphaMode2d;
use sc_sim::{SU_PER_GU, ScenarioProblem, SlotId, TilePos};

pub use self::markers::{Ghost, SlotLabel};
use self::shapes::{Overlay, near_map, tile_rect};
pub use self::tints::{LockedTint, SlopeTint, locked_mesh, slope_mesh};
use super::doc::EditorDoc;
use super::objects;
use super::state::{EditorCursor, EditorProblems, EditorSelection, Selection};
use super::terrain;
use super::tools::{EditorTools, Tool};
use crate::camera::{CameraZoom, ViewportInsets, clamp_center, viewport_rect, visible_extent_su};
use crate::coords::{DrawLayer, drawn_bounds, drawn_position};
use crate::hud::HUD_INSETS;
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
/// The outline of what an edit changes in the locked part: the images of the edited tiles.
const IMAGE_EDGE: Color = Color::srgba(1.0, 1.0, 1.0, 0.4);
const GOOD_FILL: Color = Color::srgba(0.4, 1.0, 0.5, 0.20);
const BAD_FILL: Color = Color::srgba(1.0, 0.25, 0.2, 0.35);
const BAD_EDGE: Color = Color::srgba(1.0, 0.3, 0.25, 0.95);
const SELECTION_EDGE: Color = Color::srgb(1.0, 0.9, 0.2);
/// A height dab's preview: the corners it sets, and the corners the slopes around them move.
const TARGET_DOT: Color = Color::srgb(1.0, 1.0, 1.0);
const CASCADE_DOT: Color = Color::srgb(1.0, 0.8, 0.35);
/// Line thickness in screen pixels.
const LINE_PX: f32 = 2.0;
/// The most ghosts at once: a placement and its images under a four-fold symmetry.
const MAX_GHOSTS: usize = 4;

/// A slot's colour.
pub fn slot_color(slot: SlotId) -> Color {
    SLOT_COLORS[usize::from(slot.0.max(1) - 1) % SLOT_COLORS.len()]
}

#[derive(Component)]
pub struct OverlayMesh;

#[derive(Resource)]
struct OverlayAssets {
    mesh: Handle<Mesh>,
    slope_mesh: Handle<Mesh>,
    locked_mesh: Handle<Mesh>,
    ghost_ok: Handle<ColorMaterial>,
    ghost_bad: Handle<ColorMaterial>,
}

pub struct EditorOverlayPlugin;

impl Plugin for EditorOverlayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_overlay).add_systems(
            Update,
            (
                draw_overlay,
                tints::draw_slopes,
                tints::draw_locked,
                markers::place_ghosts,
                markers::place_slot_labels,
            )
                .in_set(ClientSystems::Present),
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
    let locked_mesh = meshes.reserve_handle();
    commands.spawn((
        Name::new("Editor overlay"),
        OverlayMesh,
        Mesh2d(mesh.clone()),
        MeshMaterial2d(materials.add(blend(Color::WHITE))),
        Transform::from_xyz(0.0, 0.0, DrawLayer::Overlays.base_z()),
        Visibility::Hidden,
    ));
    // In the terrain's z band, so that the terrain in front of a tile hides its tint.
    for (name, handle, locked) in [
        ("Editor slope tint", &slope_mesh, false),
        ("Editor locked tint", &locked_mesh, true),
    ] {
        let mut tint = commands.spawn((
            Name::new(name),
            Mesh2d(handle.clone()),
            MeshMaterial2d(materials.add(blend(Color::WHITE))),
            Transform::from_xyz(0.0, 0.0, DrawLayer::Terrain.base_z()),
            Visibility::Hidden,
        ));
        if locked {
            tint.insert(LockedTint);
        } else {
            tint.insert(SlopeTint);
        }
    }
    let assets = OverlayAssets {
        mesh,
        slope_mesh,
        locked_mesh,
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
    let map = doc.map();
    let terrain = &map.terrain;
    let mut o = Overlay {
        map: terrain,
        mesh: ColorMesh::triangles(),
        line: LINE_PX * su_per_pixel(*size, *insets, zoom.0),
    };

    previews::symmetry_axes(&mut o, map);

    // Player slots: the start tile, and the camera start with the game's view around it, kept on the map's drawn
    // area as the game keeps it. Generated slots are fainter.
    let game_view = {
        let (_, viewport) = viewport_rect(*size, HUD_INSETS);
        visible_extent_su(viewport, size.physical.x, 1.0)
    };
    let bounds = drawn_bounds(terrain);
    let game_rect =
        |pos: Vec2| Rect::from_center_size(clamp_center(drawn_position(terrain, pos), game_view, bounds), game_view);
    let marker = SU_PER_GU * 0.4;
    for (id, slot) in map.slot_ids().zip(&map.slots) {
        let alpha = if map.is_slot_editable(id) {
            1.0
        } else {
            markers::GENERATED_ALPHA
        };
        let color = slot_color(id).with_alpha(alpha);
        if terrain.in_bounds(slot.start) {
            o.tile_fill(slot.start, color.with_alpha(0.55 * alpha));
            o.tiles_outline(&BTreeSet::from([slot.start]), color);
        }
        if let Some(pos) = slot.camera_start.filter(|p| terrain.contains_point(*p)) {
            o.cross(drawn_position(terrain, pos), marker, color);
            o.view_outline(game_rect(pos), color.with_alpha(0.45 * alpha));
        }
    }

    // Spawns and tiles with problems.
    for problem in &problems.0 {
        let spawns = match problem {
            ScenarioProblem::InvalidSpawn { spawn, .. } => vec![*spawn],
            ScenarioProblem::OverlappingFootprints { first, second, .. } => vec![*first, *second],
            ScenarioProblem::TooSteep { tile, .. } if terrain.in_bounds(*tile) => {
                o.tiles_outline(&BTreeSet::from([*tile]), BAD_EDGE);
                Vec::new()
            }
            _ => Vec::new(),
        };
        for s in spawns.into_iter().filter_map(|i| map.spawns.get(i)) {
            let rect = objects::spawn_rect(s.kind, s.at);
            if near_map(terrain, rect) {
                o.surface_outline(rect, BAD_EDGE);
            }
        }
    }

    // The selection.
    let selected = match selection.0 {
        Some(Selection::Spawn(i)) => map.spawns.get(i).map(|s| objects::spawn_rect(s.kind, s.at)),
        Some(Selection::Start(s)) => map.slot(s).map(|slot| tile_rect(slot.start)),
        Some(Selection::CameraStart(s)) => {
            if let Some(c) = map.slot(s).and_then(|slot| slot.camera_start).filter(|c| c.is_finite()) {
                let around = Rect::from_center_size(drawn_position(terrain, c), Vec2::splat(SU_PER_GU));
                o.view_outline(around, SELECTION_EDGE);
            }
            None
        }
        None => None,
    };
    if let Some(rect) = selected
        .map(|r| r.inflate(o.line * 2.0))
        .filter(|r| near_map(terrain, *r))
    {
        o.surface_outline(rect, SELECTION_EDGE);
    }

    // What the tool would do under the cursor; red where the locked part refuses it.
    if let Some(pos) = cursor.0 {
        let tile = TilePos::containing(pos);
        let region = map.region();
        let slot = slot_color(tools.slot);
        match tools.tool {
            Tool::Paint | Tool::Elevation | Tool::Fill | Tool::Start if !region.tile(tile) => {
                previews::locked_cursor(&mut o, tile);
            }
            Tool::CameraStart if !region.point(pos) => previews::locked_cursor(&mut o, tile),
            Tool::Paint => previews::paint_preview(&mut o, map, &tools, tile),
            Tool::Elevation => previews::height_preview(&mut o, map, &tools, tile),
            Tool::Fill => previews::fill_preview(&mut o, map, tile),
            Tool::Place => {
                for (at, problem) in previews::placements(map, &tools, pos) {
                    previews::placement_preview(&mut o, tools.kind, at, problem);
                }
            }
            Tool::Start if terrain.in_bounds(tile) => {
                let images = terrain::image_tiles(map, tile);
                o.tiles_outline(&BTreeSet::from([tile]), slot);
                for image in images.into_iter().skip(1) {
                    o.tiles_outline(&BTreeSet::from([image]), IMAGE_EDGE);
                }
            }
            Tool::CameraStart if terrain.contains_point(pos) => {
                o.cross(drawn_position(terrain, pos), marker, slot);
                o.view_outline(game_rect(pos), slot.with_alpha(0.6));
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
