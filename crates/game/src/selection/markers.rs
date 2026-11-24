//! What the selection looks like on the map (`ARCHITECTURE.md > Client architecture > Visuals`): a marker under
//! each object the local player has selected, and the box while a selection box is dragged
//! (`systems/controls/selection_and_hud.md > BoxSelection`).
//!
//! Placeholders until the art (Q0001), the design gives none: a ring around a unit's silhouette and a frame inside a
//! tile-based object's footprint, coloured by the object's relation to the local player (own green, ally blue, enemy
//! red, neutral yellow), drawn in the ground-decal layer, under the objects. The ActiveGroup's rings are solid and
//! thick, the other groups' dashed and thin (frames: thick and thin). Meshes are made once per shape, relation and
//! activity. A selected record (a GDO player's remembered SupplyDeliveryStation) is marked where it is shown.

use std::collections::BTreeMap;

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use sc_sim::{ObjectId, ObjectKind, SU_PER_GU};

use super::ControlStates;
use super::pick::{Relation, Selectable};
use crate::command_ui::InterfaceState;
use crate::coords::{DrawLayer, view_to_world};
use crate::fog::LocalSight;
use crate::input::ActionState;
use crate::shapes::{ColorMesh, vertex_color_material};
use crate::sim_driver::{LocalPlayer, RenderAlpha};
use crate::{ClientMode, ClientSystems};

/// Marker colours by relation (placeholders).
const OWN_COLOR: Color = Color::srgb(0.30, 0.95, 0.40);
const ALLY_COLOR: Color = Color::srgb(0.35, 0.75, 1.00);
const ENEMY_COLOR: Color = Color::srgb(1.00, 0.28, 0.22);
const NEUTRAL_COLOR: Color = Color::srgb(1.00, 0.85, 0.30);
/// Line widths of the ActiveGroup's markers and of the other groups'.
const ACTIVE_WIDTH_SU: f32 = 3.0;
const INACTIVE_WIDTH_SU: f32 = 2.0;
/// A unit's ring lies this far outside its silhouette's corners; a frame this far inside a footprint.
const RING_GAP_SU: f32 = 4.0;
const FRAME_INSET_SU: f32 = 4.0;
/// Segments of a ring; a dashed ring draws every other one.
const RING_SEGMENTS: usize = 28;
/// The box being dragged: its colour and line width (placeholders).
const DRAG_BOX_COLOR: Color = Color::srgb(0.45, 1.0, 0.55);
const DRAG_BOX_WIDTH_SU: f32 = 2.0;

/// The colour of markers for objects of this relation.
pub fn relation_color(relation: Relation) -> Color {
    match relation {
        Relation::Own => OWN_COLOR,
        Relation::Ally => ALLY_COLOR,
        Relation::Enemy => ENEMY_COLOR,
        Relation::Neutral => NEUTRAL_COLOR,
    }
}

/// The marker under a selected object.
#[derive(Component, Copy, Clone, Debug, PartialEq, Eq)]
pub struct SelectionMarker {
    pub id: ObjectId,
    pub relation: Relation,
    /// The object belongs to the ActiveGroup.
    pub active: bool,
}

/// One edge of the box being dragged: top, bottom, left, right.
#[derive(Component, Copy, Clone, Debug, PartialEq, Eq)]
pub struct DragBoxEdge(pub u8);

/// The marker entity of each object the local player has selected.
#[derive(Resource, Default, Debug)]
pub struct SelectionMarkers {
    pub by_id: BTreeMap<ObjectId, Entity>,
}

/// A marker's outline: a ring for a unit, a frame for a tile-based object of a `w` × `h` footprint.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum MarkerShape {
    Ring(ObjectKind),
    Frame { w: u32, h: u32 },
}

/// Mesh and material handles, made once and shared.
#[derive(Resource, Default)]
struct MarkerAssets {
    material: Option<Handle<ColorMaterial>>,
    meshes: BTreeMap<(MarkerShape, Relation, bool), Handle<Mesh>>,
}

pub struct SelectionMarkersPlugin;

impl Plugin for SelectionMarkersPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MarkerAssets>()
            .init_resource::<SelectionMarkers>()
            .add_systems(Startup, spawn_drag_box)
            .add_systems(Update, (sync_markers, draw_drag_box).in_set(ClientSystems::Present));
    }
}

/// What the markers show: the local player's selection in a game mode (none in the editor), drawn where the frame
/// draws the objects, or the records shown in their place.
#[derive(SystemParam)]
struct Shown<'w> {
    mode: Res<'w, ClientMode>,
    sight: LocalSight<'w>,
    alpha: Res<'w, RenderAlpha>,
    player: Res<'w, LocalPlayer>,
    states: Res<'w, ControlStates>,
}

impl Shown<'_> {
    /// Each selected object as it is drawn, its marker's shape, and whether it is in the ActiveGroup.
    fn markers(&self) -> Vec<(Selectable, MarkerShape, bool)> {
        if !self.mode.is_game() {
            return Vec::new();
        }
        let (sim, sight) = (self.sight.sim(), self.sight.get());
        let selection = self.states.selection(self.player.0);
        selection
            .ids()
            .filter_map(|id| {
                let (shown, placement) = match sim.object(id).filter(|o| sight.sees(o)) {
                    Some(object) => {
                        let pos = object.prev_pos.lerp(object.pos, self.alpha.0);
                        let shown = Selectable::of(object, self.player.0, sim.map(), pos);
                        (shown, object.placement)
                    }
                    None => {
                        let record = sight.remembered_record(id)?;
                        let shown = Selectable::remembered(record, self.player.0, sim.map());
                        (shown, Some(record.placement))
                    }
                };
                let shape = match placement {
                    Some(p) => MarkerShape::Frame { w: p.w, h: p.h },
                    None => MarkerShape::Ring(shown.kind),
                };
                Some((shown, shape, selection.in_active_group(id)))
            })
            .collect()
    }
}

/// The meshes and the material the markers are made of.
#[derive(SystemParam)]
struct MarkerMeshes<'w> {
    assets: ResMut<'w, MarkerAssets>,
    meshes: ResMut<'w, Assets<Mesh>>,
    materials: ResMut<'w, Assets<ColorMaterial>>,
}

impl MarkerMeshes<'_> {
    fn material(&mut self) -> Handle<ColorMaterial> {
        let materials = &mut self.materials;
        self.assets
            .material
            .get_or_insert_with(|| materials.add(vertex_color_material()))
            .clone()
    }

    fn mesh(&mut self, shape: MarkerShape, relation: Relation, active: bool) -> Handle<Mesh> {
        let meshes = &mut self.meshes;
        self.assets
            .meshes
            .entry((shape, relation, active))
            .or_insert_with(|| meshes.add(marker_mesh(shape, relation, active)))
            .clone()
    }
}

/// Spawns, restyles, places and despawns the markers so that they follow the local player's selection.
fn sync_markers(
    mut commands: Commands,
    shown: Shown,
    mut meshes: MarkerMeshes,
    mut index: ResMut<SelectionMarkers>,
    mut markers: Query<(&mut SelectionMarker, &mut Mesh2d, &mut Transform)>,
) {
    let wanted = shown.markers();
    let keep: BTreeMap<ObjectId, ()> = wanted.iter().map(|(s, _, _)| (s.id, ())).collect();
    index.by_id.retain(|id, entity| {
        let kept = keep.contains_key(id);
        if !kept {
            commands.entity(*entity).despawn();
        }
        kept
    });
    for (object, shape, active) in wanted {
        let marker = SelectionMarker {
            id: object.id,
            relation: object.relation,
            active,
        };
        let transform =
            Transform::from_translation(view_to_world(object.drawn).extend(DrawLayer::GroundDecals.z_at(object.depth)));
        match index.by_id.get(&object.id).and_then(|&e| markers.get_mut(e).ok()) {
            Some((mut current, mut mesh, mut placed)) => {
                if *current != marker {
                    *current = marker;
                    mesh.0 = meshes.mesh(shape, object.relation, active);
                }
                placed.set_if_neq(transform);
            }
            None => {
                let entity = commands
                    .spawn((
                        Name::new(format!("Selection marker #{}", object.id.0)),
                        marker,
                        Mesh2d(meshes.mesh(shape, object.relation, active)),
                        MeshMaterial2d(meshes.material()),
                        transform,
                    ))
                    .id();
                index.by_id.insert(object.id, entity);
            }
        }
    }
}

/// A marker's mesh, centered on the object's drawn anchor (Bevy's +y up).
fn marker_mesh(shape: MarkerShape, relation: Relation, active: bool) -> Mesh {
    let color = relation_color(relation);
    let width = if active { ACTIVE_WIDTH_SU } else { INACTIVE_WIDTH_SU };
    let mut mesh = ColorMesh::triangles();
    match shape {
        MarkerShape::Ring(kind) => {
            let half = kind.def().half_extents().unwrap_or(Vec2::splat(12.0));
            // Around the silhouette's corners, so a square one stays inside too.
            let outer = half.length() + RING_GAP_SU + ACTIVE_WIDTH_SU;
            let inner = outer - width;
            let at = |i: usize, r: f32| {
                let a = i as f32 * std::f32::consts::TAU / RING_SEGMENTS as f32;
                Vec2::new(r * a.cos(), r * a.sin()).extend(0.0)
            };
            let step = if active { 1 } else { 2 };
            for i in (0..RING_SEGMENTS).step_by(step) {
                mesh.quad([at(i, inner), at(i, outer), at(i + 1, outer), at(i + 1, inner)], color);
            }
        }
        MarkerShape::Frame { w, h } => {
            let half = Vec2::new(w as f32, h as f32) * SU_PER_GU / 2.0 - Vec2::splat(FRAME_INSET_SU);
            let inner = half - Vec2::splat(width);
            mesh.rect(Vec2::new(-half.x, inner.y), half, 0.0, color);
            mesh.rect(-half, Vec2::new(half.x, -inner.y), 0.0, color);
            mesh.rect(Vec2::new(-half.x, -inner.y), Vec2::new(-inner.x, inner.y), 0.0, color);
            mesh.rect(Vec2::new(inner.x, -inner.y), Vec2::new(half.x, inner.y), 0.0, color);
        }
    }
    mesh.into_mesh()
}

/// The four edges of the drag box: one unit square, stretched by each edge's transform.
fn spawn_drag_box(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut square = ColorMesh::triangles();
    square.rect(Vec2::splat(-0.5), Vec2::splat(0.5), 0.0, DRAG_BOX_COLOR);
    let mesh = meshes.add(square.into_mesh());
    let material = materials.add(vertex_color_material());
    for edge in 0..4 {
        commands.spawn((
            Name::new("Drag box edge"),
            DragBoxEdge(edge),
            Mesh2d(mesh.clone()),
            MeshMaterial2d(material.clone()),
            Transform::IDENTITY,
            Visibility::Hidden,
        ));
    }
}

/// `> BoxSelection`: shows the box (`Pointer::drag_box`, view space) while it is dragged, in the overlay layer.
fn draw_drag_box(
    actions: Res<ActionState>,
    mode: Res<ClientMode>,
    states: Res<ControlStates>,
    player: Res<LocalPlayer>,
    mut edges: Query<(&DragBoxEdge, &mut Transform, &mut Visibility)>,
) {
    // A box selects nothing while a target is awaited (`super::systems::select_on_map`): none is drawn then.
    let targeting = states.of(player.0).interface != InterfaceState::DefaultState;
    let dragged = actions.pointer().drag_box.filter(|_| mode.is_game() && !targeting);
    for (edge, mut transform, mut visibility) in &mut edges {
        let Some(rect) = dragged else {
            visibility.set_if_neq(Visibility::Hidden);
            continue;
        };
        let (center, size) = edge_of(rect, edge.0);
        let z = DrawLayer::Overlays.base_z();
        transform.set_if_neq(Transform::from_translation(view_to_world(center).extend(z)).with_scale(size.extend(1.0)));
        visibility.set_if_neq(Visibility::Inherited);
    }
}

/// The center (view space) and size of edge `edge` (top, bottom, left, right) of the box `rect`.
fn edge_of(rect: Rect, edge: u8) -> (Vec2, Vec2) {
    let w = DRAG_BOX_WIDTH_SU;
    let size = rect.size();
    let mid = rect.center();
    match edge {
        0 => (Vec2::new(mid.x, rect.min.y), Vec2::new(size.x + w, w)),
        1 => (Vec2::new(mid.x, rect.max.y), Vec2::new(size.x + w, w)),
        2 => (Vec2::new(rect.min.x, mid.y), Vec2::new(w, size.y + w)),
        _ => (Vec2::new(rect.max.x, mid.y), Vec2::new(w, size.y + w)),
    }
}
