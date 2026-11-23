//! Placeholder visuals for sim objects (`ARCHITECTURE.md > Client architecture > Visuals`).
//!
//! The sim is the source of truth. Each visual is a separate entity linked to its object by [`ObjectVisual`]. Visuals
//! are created and despawned by diffing them against `Sim::objects()` (scenario spawns emit no events), and placed
//! from the sim every frame at `prev_pos.lerp(pos, alpha)`, y-sorted in the ground-object layer. Placeholder shapes
//! stand in until the art direction (Q0001) is settled.

use std::collections::BTreeMap;

use bevy::prelude::*;
use sc_sim::{ObjectId, ObjectKind};

use crate::ClientSystems;
use crate::coords::{DrawLayer, anchored_translation, facing_to_rotation, lerp_facing};
use crate::shapes::{ColorMesh, vertex_color_material};
use crate::sim_driver::{RenderAlpha, SimRes};

/// Links a visual entity to the sim object it shows.
#[derive(Component, Copy, Clone, Debug, PartialEq, Eq)]
pub struct ObjectVisual(pub ObjectId);

/// Mesh and material handles, created once per object kind and shared by all its visuals.
#[derive(Resource, Default)]
pub struct ObjectVisualAssets {
    material: Option<Handle<ColorMaterial>>,
    meshes: BTreeMap<ObjectKind, Handle<Mesh>>,
}

/// The visual entity of each object.
#[derive(Resource, Default, Debug)]
pub struct ObjectVisuals {
    pub by_id: BTreeMap<ObjectId, Entity>,
    /// The `SimRes::generation` the visuals belong to.
    generation: Option<u32>,
}

pub struct ObjectRenderPlugin;

impl Plugin for ObjectRenderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ObjectVisualAssets>()
            .init_resource::<ObjectVisuals>()
            .add_systems(Update, sync_object_visuals.in_set(ClientSystems::SyncSim))
            .add_systems(Update, place_object_visuals.in_set(ClientSystems::Present));
    }
}

/// The transform of a visual anchored at sim position `pos` with sim facing `facing`.
pub fn object_transform(pos: Vec2, facing: f32) -> Transform {
    Transform::from_translation(anchored_translation(DrawLayer::GroundObjects, pos))
        .with_rotation(facing_to_rotation(facing))
}

/// The placeholder shape of each object kind, centered on the object's position.
pub fn placeholder_mesh(kind: ObjectKind) -> Mesh {
    match kind {
        ObjectKind::CrystalPatch => crystal_patch_mesh(),
        ObjectKind::SupplyDeliveryStation => supply_station_mesh(),
    }
}

/// `core/map.md > SpaceCrystalsPatch` (1×1): a cluster of three cyan crystals on a dark bed.
fn crystal_patch_mesh() -> Mesh {
    let mut mesh = ColorMesh::triangles();
    let bed: Vec<Vec2> = (0..8)
        .map(|i| {
            let a = i as f32 * std::f32::consts::TAU / 8.0;
            Vec2::new(25.0 * a.cos(), -10.0 + 11.0 * a.sin())
        })
        .collect();
    mesh.polygon(&bed, 0.0, Color::srgb(0.09, 0.14, 0.18));
    crystal_shard(&mut mesh, Vec2::new(-13.0, -8.0), Vec2::new(7.0, 15.0), 0.001);
    crystal_shard(&mut mesh, Vec2::new(13.0, -10.0), Vec2::new(7.0, 13.0), 0.001);
    crystal_shard(&mut mesh, Vec2::new(0.0, -2.0), Vec2::new(10.0, 24.0), 0.002);
    mesh.into_mesh()
}

/// A five-sided shard standing on its base, its facets shaded as if lit from the top left.
fn crystal_shard(mesh: &mut ColorMesh, center: Vec2, half: Vec2, z: f32) {
    let apex = center + Vec2::new(0.0, half.y);
    let upper_right = center + Vec2::new(half.x, half.y * 0.35);
    let lower_right = center + Vec2::new(half.x * 0.7, -half.y);
    let lower_left = center + Vec2::new(-half.x * 0.7, -half.y);
    let upper_left = center + Vec2::new(-half.x, half.y * 0.35);
    let core = center + Vec2::new(0.0, half.y * 0.1);
    let facets = [
        (upper_left, apex, Color::srgb(0.72, 1.0, 1.0)),
        (apex, upper_right, Color::srgb(0.36, 0.90, 0.98)),
        (upper_right, lower_right, Color::srgb(0.07, 0.48, 0.66)),
        (lower_right, lower_left, Color::srgb(0.04, 0.32, 0.47)),
        (lower_left, upper_left, Color::srgb(0.17, 0.72, 0.86)),
    ];
    for (a, b, color) in facets {
        mesh.polygon(&[core, a, b], z, color);
    }
}

/// `core/map.md > SupplyDeliveryStation` (2×2): an amber landing pad with a cross and corner lights.
fn supply_station_mesh() -> Mesh {
    let mut mesh = ColorMesh::triangles();
    mesh.rect(
        Vec2::splat(-60.0),
        Vec2::splat(60.0),
        0.0,
        Color::srgb(0.28, 0.18, 0.07),
    );
    mesh.rect(
        Vec2::splat(-52.0),
        Vec2::splat(52.0),
        0.001,
        Color::srgb(0.85, 0.55, 0.13),
    );
    let mark = Color::srgb(0.40, 0.24, 0.06);
    mesh.rect(Vec2::new(-30.0, -5.0), Vec2::new(30.0, 5.0), 0.002, mark);
    mesh.rect(Vec2::new(-5.0, -30.0), Vec2::new(5.0, 30.0), 0.002, mark);
    for corner in [
        Vec2::new(-1.0, -1.0),
        Vec2::new(1.0, -1.0),
        Vec2::new(1.0, 1.0),
        Vec2::new(-1.0, 1.0),
    ] {
        let c = corner * 42.0;
        mesh.rect(c - 5.0, c + 5.0, 0.002, Color::srgb(1.0, 0.93, 0.55));
    }
    mesh.into_mesh()
}

/// Spawns a visual for every object without one and despawns visuals whose object is gone. A replaced sim (new
/// generation) starts from scratch, because its ids may repeat the old ones.
fn sync_object_visuals(
    mut commands: Commands,
    sim: Res<SimRes>,
    mut visuals: ResMut<ObjectVisuals>,
    mut assets: ResMut<ObjectVisualAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if !sim.is_changed() {
        return;
    }
    let visuals = &mut *visuals;
    if visuals.generation != Some(sim.generation()) {
        for entity in std::mem::take(&mut visuals.by_id).into_values() {
            commands.entity(entity).despawn();
        }
        visuals.generation = Some(sim.generation());
    }
    visuals.by_id.retain(|id, entity| {
        let alive = sim.object(*id).is_some();
        if !alive {
            commands.entity(*entity).despawn();
        }
        alive
    });
    let assets = &mut *assets;
    let material = assets
        .material
        .get_or_insert_with(|| materials.add(vertex_color_material()))
        .clone();
    for object in sim.objects() {
        if visuals.by_id.contains_key(&object.id) {
            continue;
        }
        let mesh = assets
            .meshes
            .entry(object.kind)
            .or_insert_with(|| meshes.add(placeholder_mesh(object.kind)))
            .clone();
        let entity = commands
            .spawn((
                Name::new(format!("{:?} #{}", object.kind, object.id.0)),
                ObjectVisual(object.id),
                Mesh2d(mesh),
                MeshMaterial2d(material.clone()),
                object_transform(object.pos, object.facing),
            ))
            .id();
        visuals.by_id.insert(object.id, entity);
    }
}

/// `ARCHITECTURE.md > Client architecture > Visuals > Interpolation`.
fn place_object_visuals(
    sim: Res<SimRes>,
    alpha: Res<RenderAlpha>,
    mut visuals: Query<(&ObjectVisual, &mut Transform)>,
) {
    for (visual, mut transform) in &mut visuals {
        let Some(object) = sim.object(visual.0) else {
            continue;
        };
        let pos = object.prev_pos.lerp(object.pos, alpha.0);
        let facing = lerp_facing(object.prev_facing, object.facing, alpha.0);
        transform.set_if_neq(object_transform(pos, facing));
    }
}
