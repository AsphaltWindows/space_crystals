//! The ghosts of the object about to be placed and its images, and the labels of the slots' markers.

use bevy::prelude::*;
use sc_sim::{ObjectKind, SU_PER_GU, SlotId, SpawnAt};

use super::previews::placements;
use super::{OverlayAssets, slot_color, su_per_pixel};
use crate::ClientMode;
use crate::camera::{CameraZoom, ViewportInsets};
use crate::coords::{DrawLayer, drawn_at_height, drawn_position, footprint_height, view_to_world};
use crate::editor::doc::EditorDoc;
use crate::editor::objects;
use crate::editor::state::EditorCursor;
use crate::editor::tools::{EditorTools, Tool};
use crate::object_render::placeholder_mesh;
use crate::platform::SurfaceSize;

/// Size of the slot labels in screen pixels.
const LABEL_PX: f32 = 14.0;
/// How opaque the markers and labels of generated slots are, against 1 for editable ones.
pub(super) const GENERATED_ALPHA: f32 = 0.55;

/// The ghost of the object to be placed (`image` 0) or of one of its symmetric images.
#[derive(Component)]
pub struct Ghost {
    pub image: usize,
}

#[derive(Component)]
pub struct SlotLabel;

#[expect(clippy::too_many_arguments, reason = "a system's parameters")]
pub(super) fn place_ghosts(
    mode: Res<ClientMode>,
    doc: Res<EditorDoc>,
    tools: Res<EditorTools>,
    cursor: Res<EditorCursor>,
    assets: Res<OverlayAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut ghost_meshes: Local<Vec<(ObjectKind, Handle<Mesh>)>>,
    mut ghosts: Query<(
        &Ghost,
        &mut Mesh2d,
        &mut MeshMaterial2d<ColorMaterial>,
        &mut Transform,
        &mut Visibility,
    )>,
) {
    // Review finding: only when something they show changed.
    if !(mode.is_changed() || doc.is_changed() || tools.is_changed() || cursor.is_changed()) {
        return;
    }
    let shown = match (*mode, tools.tool, cursor.0) {
        (ClientMode::Editor, Tool::Place, Some(pos)) => placements(doc.map(), &tools, pos),
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
    let map = &doc.map().terrain;
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

/// A slot marker's label: its text, slot, whether the slot is edited or generated, and the view point it hangs
/// from (below) or stands on (above).
#[derive(Clone, PartialEq)]
pub(super) struct Label {
    text: String,
    slot: SlotId,
    editable: bool,
    at: Vec2,
    below: bool,
}

/// Labels "S1 start" under each start position and "S1 camera" over each camera start, at a constant size on
/// screen. Generated slots' labels are fainter.
pub(super) fn place_slot_labels(
    mut commands: Commands,
    mode: Res<ClientMode>,
    doc: Res<EditorDoc>,
    view: (Res<SurfaceSize>, Res<ViewportInsets>, Res<CameraZoom>),
    labels: Query<Entity, With<SlotLabel>>,
    mut shown: Local<Option<(Vec<Label>, f32)>>,
) {
    let (size, insets, zoom) = view;
    let changed = mode.is_changed() || doc.is_changed() || size.is_changed() || insets.is_changed();
    if !changed && !zoom.is_changed() {
        return;
    }
    // Space units per screen pixel: the labels' scale and the unit of their distance from the marker.
    let scale = su_per_pixel(*size, *insets, zoom.0);
    let gap = 3.0 * scale;
    let mut wanted = Vec::new();
    if *mode == ClientMode::Editor {
        let map = doc.map();
        let terrain = &map.terrain;
        for (id, slot) in map.slot_ids().zip(&map.slots) {
            let editable = map.is_slot_editable(id);
            if terrain.in_bounds(slot.start) {
                let t = slot.start;
                let bottom = drawn_position(terrain, t.corner() + Vec2::new(SU_PER_GU / 2.0, SU_PER_GU));
                wanted.push(Label {
                    text: format!("S{} start", id.0),
                    slot: id,
                    editable,
                    at: bottom + Vec2::new(0.0, gap),
                    below: true,
                });
            }
            if let Some(p) = slot.camera_start.filter(|p| terrain.contains_point(*p)) {
                wanted.push(Label {
                    text: format!("S{} camera", id.0),
                    slot: id,
                    editable,
                    at: drawn_position(terrain, p) - Vec2::new(0.0, SU_PER_GU * 0.4 + gap),
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
        let alpha = if label.editable { 1.0 } else { GENERATED_ALPHA };
        commands.spawn((
            Name::new("Slot label"),
            SlotLabel,
            Text2d::new(label.text.clone()),
            TextFont::from_font_size(FontSize::Px(LABEL_PX)),
            TextColor(slot_color(label.slot).with_alpha(alpha)),
            anchor,
            Transform::from_translation(view_to_world(label.at).extend(DrawLayer::Overlays.base_z() + 1.0))
                .with_scale(Vec3::splat(scale)),
        ));
    }
    *shown = Some(wanted);
}
