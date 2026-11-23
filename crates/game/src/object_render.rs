//! Placeholder visuals for sim objects (`ARCHITECTURE.md > Client architecture > Visuals`).
//!
//! The sim is the source of truth. Each visual is a separate entity linked to its object by [`ObjectVisual`]. Visuals
//! are created and despawned by diffing them against `Sim::objects()` (scenario spawns emit no events), and placed
//! from the sim every frame at `prev_pos.lerp(pos, alpha)`, drawn at the terrain height there (a tile-based object
//! at its footprint's height, `core/art_direction.md > Elevation Display`) and y-sorted by its ground position in the
//! ground-object layer. Placeholder shapes stand in until the art direction (Q0001) is settled: a unit is a shape of
//! its own per kind, its silhouette's size, in its owner's colour, with a tick showing its facing. The Guard's square
//! stays axis-aligned, like its collision box, and only its tick turns ([`FacingTick`]).

use std::collections::BTreeMap;

use bevy::prelude::*;
use sc_sim::{Map, ObjectId, ObjectKind, Placement, PlayerId};

use crate::ClientSystems;
use crate::coords::{DrawLayer, facing_to_rotation, footprint_height, ground_translation, lerp_facing};
use crate::shapes::{ColorMesh, darken, vertex_color_material};
use crate::sim_driver::{RenderAlpha, SimRes};

/// Links a visual entity to the sim object it shows.
#[derive(Component, Copy, Clone, Debug, PartialEq, Eq)]
pub struct ObjectVisual(pub ObjectId);

/// The facing tick of a visual whose body doesn't turn ([`body_turns`]): a child of the object's visual, turned by
/// the object's facing.
#[derive(Component, Copy, Clone, Debug, PartialEq, Eq)]
pub struct FacingTick(pub ObjectId);

/// Mesh and material handles, created once per object kind (and owner, for units) and shared by all its visuals.
#[derive(Resource, Default)]
pub struct ObjectVisualAssets {
    material: Option<Handle<ColorMaterial>>,
    meshes: BTreeMap<(ObjectKind, Option<PlayerId>), Handle<Mesh>>,
    ticks: BTreeMap<ObjectKind, Handle<Mesh>>,
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

/// The height an object stands at: its footprint's for a tile-based object, else the terrain's at `pos`.
pub fn object_height(map: &Map, pos: Vec2, placement: Option<&Placement>) -> f32 {
    match placement {
        Some(p) => footprint_height(map, p.origin, p.w, p.h),
        None => map.height_at(pos),
    }
}

/// The transform of a visual anchored at ground position `pos`, standing at `height` levels, with sim facing
/// `facing`.
pub fn object_transform(pos: Vec2, height: f32, facing: f32) -> Transform {
    Transform::from_translation(ground_translation(DrawLayer::GroundObjects, pos, height))
        .with_rotation(facing_to_rotation(facing))
}

/// The placeholder shape of each object kind, centered on the object's position; units in the neutral colour.
pub fn placeholder_mesh(kind: ObjectKind) -> Mesh {
    object_mesh(kind, None)
}

/// The placeholder shape of an object of `kind` owned by `owner`. Units take their owner's colour.
pub fn object_mesh(kind: ObjectKind, owner: Option<PlayerId>) -> Mesh {
    match kind {
        ObjectKind::CrystalPatch => crystal_patch_mesh(),
        ObjectKind::SupplyDeliveryStation => supply_station_mesh(),
        ObjectKind::Peacekeeper => unit_mesh(kind, UnitShape::Round, player_color(owner), true),
        ObjectKind::Guard => unit_mesh(kind, UnitShape::Square, player_color(owner), true),
    }
}

/// Whether a visual of `kind` turns with its object's facing. The Guard's square doesn't: its collision box never
/// turns, and a turned square makes crowded Guards look as if they overlapped. Its tick turns alone ([`FacingTick`]).
fn body_turns(kind: ObjectKind) -> bool {
    kind != ObjectKind::Guard
}

/// The mesh of an object's visual: its placeholder shape, without the tick if its body doesn't turn.
fn visual_mesh(kind: ObjectKind, owner: Option<PlayerId>) -> Mesh {
    match kind {
        ObjectKind::Guard => unit_mesh(kind, UnitShape::Square, player_color(owner), false),
        _ => object_mesh(kind, owner),
    }
}

/// Player colours, by player number: those of the editor's slots, whose players they are by default.
const PLAYER_COLORS: [Color; 8] = [
    Color::srgb(0.30, 0.55, 1.00),
    Color::srgb(1.00, 0.35, 0.30),
    Color::srgb(0.35, 0.85, 0.40),
    Color::srgb(1.00, 0.85, 0.25),
    Color::srgb(0.75, 0.45, 1.00),
    Color::srgb(0.30, 0.90, 0.90),
    Color::srgb(1.00, 0.60, 0.20),
    Color::srgb(1.00, 0.50, 0.80),
];
const NEUTRAL_COLOR: Color = Color::srgb(0.70, 0.70, 0.66);

/// The colour of `owner`'s units: grey for neutral ones.
pub fn player_color(owner: Option<PlayerId>) -> Color {
    match owner {
        Some(p) => PLAYER_COLORS[usize::from(p.0.max(1) - 1) % PLAYER_COLORS.len()],
        None => NEUTRAL_COLOR,
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum UnitShape {
    Round,
    Square,
}

/// A unit (placeholders until Q0001 settles the art): its silhouette's size in a shape of its own, filled with its
/// owner's colour inside a dark rim, and (`with_tick`) a light tick from its center through its front, along local
/// +x, which the visual's rotation turns to the unit's facing.
fn unit_mesh(kind: ObjectKind, shape: UnitShape, color: Color, with_tick: bool) -> Mesh {
    let half = kind.def().half_extents().unwrap_or(Vec2::splat(12.0));
    let outline = |inset: f32| -> Vec<Vec2> {
        let h = half - Vec2::splat(inset);
        match shape {
            UnitShape::Round => (0..16)
                .map(|i| {
                    let a = i as f32 * std::f32::consts::TAU / 16.0;
                    Vec2::new(h.x * a.cos(), h.y * a.sin())
                })
                .collect(),
            UnitShape::Square => vec![
                Vec2::new(-h.x, -h.y),
                Vec2::new(h.x, -h.y),
                Vec2::new(h.x, h.y),
                Vec2::new(-h.x, h.y),
            ],
        }
    };
    let srgba = color.to_srgba();
    let mut mesh = ColorMesh::triangles();
    mesh.polygon(&outline(0.0), 0.0, darken(srgba, 0.45).into());
    mesh.polygon(&outline(2.5), 0.001, color);
    if with_tick {
        facing_tick(&mut mesh, half);
    }
    mesh.into_mesh()
}

/// A unit's facing tick, along local +x, in front of its body.
fn facing_tick(mesh: &mut ColorMesh, half: Vec2) {
    mesh.segment(
        Vec2::ZERO,
        Vec2::new(half.x + 5.0, 0.0),
        3.0,
        0.002,
        Color::srgb(0.96, 0.96, 0.92),
    );
}

/// The facing tick of a `kind` visual whose body doesn't turn, alone.
fn tick_mesh(kind: ObjectKind) -> Mesh {
    let mut mesh = ColorMesh::triangles();
    facing_tick(&mut mesh, kind.def().half_extents().unwrap_or(Vec2::splat(12.0)));
    mesh.into_mesh()
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
        // Only units are drawn in their owner's colour.
        let owner = object.owner.filter(|_| object.kind.def().unit.is_some());
        let mesh = assets
            .meshes
            .entry((object.kind, owner))
            .or_insert_with(|| meshes.add(visual_mesh(object.kind, owner)))
            .clone();
        let height = object_height(sim.map(), object.pos, object.placement.as_ref());
        let turns = body_turns(object.kind);
        let entity = commands
            .spawn((
                Name::new(format!("{:?} #{}", object.kind, object.id.0)),
                ObjectVisual(object.id),
                Mesh2d(mesh),
                MeshMaterial2d(material.clone()),
                object_transform(object.pos, height, if turns { object.facing } else { 0.0 }),
            ))
            .id();
        if !turns {
            let tick = assets
                .ticks
                .entry(object.kind)
                .or_insert_with(|| meshes.add(tick_mesh(object.kind)))
                .clone();
            commands.spawn((
                Name::new("Facing tick"),
                FacingTick(object.id),
                Mesh2d(tick),
                MeshMaterial2d(material.clone()),
                Transform::from_rotation(facing_to_rotation(object.facing)),
                ChildOf(entity),
            ));
        }
        visuals.by_id.insert(object.id, entity);
    }
}

/// `ARCHITECTURE.md > Client architecture > Visuals > Interpolation`.
fn place_object_visuals(
    sim: Res<SimRes>,
    alpha: Res<RenderAlpha>,
    mut visuals: Query<(&ObjectVisual, &mut Transform), Without<FacingTick>>,
    mut ticks: Query<(&FacingTick, &mut Transform), Without<ObjectVisual>>,
) {
    for (visual, mut transform) in &mut visuals {
        let Some(object) = sim.object(visual.0) else {
            continue;
        };
        let pos = object.prev_pos.lerp(object.pos, alpha.0);
        let facing = lerp_facing(object.prev_facing, object.facing, alpha.0);
        let height = object_height(sim.map(), pos, object.placement.as_ref());
        let facing = if body_turns(object.kind) { facing } else { 0.0 };
        transform.set_if_neq(object_transform(pos, height, facing));
    }
    for (tick, mut transform) in &mut ticks {
        let Some(object) = sim.object(tick.0) else {
            continue;
        };
        let facing = lerp_facing(object.prev_facing, object.facing, alpha.0);
        transform.set_if_neq(Transform::from_rotation(facing_to_rotation(facing)));
    }
}
