//! Placeholder visuals for sim objects (`ARCHITECTURE.md > Client architecture > Visuals`).
//!
//! The sim is the source of truth. Each visual is a separate entity linked to its object by [`ObjectVisual`]. Visuals
//! are created and despawned by diffing them against `Sim::objects()` (scenario spawns emit no events), and placed
//! from the sim every frame at `prev_pos.lerp(pos, alpha)`, drawn at the terrain height there (a tile-based object
//! at its footprint's height, `core/art_direction.md > Elevation Display`) and y-sorted by its ground position in the
//! ground-object layer. Placeholder shapes stand in until the art direction (Q0001) is settled ([`units`]): a unit is
//! a shape of its own per kind, its silhouette's size, in its owner's colour, with a tick showing its facing. A square
//! or octagonal body (the Guard's, the Colonists' units') stays axis-aligned, like its collision box, and only its
//! tick turns ([`FacingTick`]); the Colonists' units carry a tag naming their type ([`UnitTag`]). A Peacekeeper's
//! raised Riot Shield is a pale arc in front of it ([`RiotShieldVisual`]), growing out of its body as the shield goes
//! up. A structure is its footprint as placed, in its owner's colour, with a tag naming its type ([`structures`]).
//! The map resources have shapes of their own ([`resources`]).
//!
//! The fog of war hides the visuals of the objects the local player doesn't see (`crate::fog`); Explored tiles show
//! the records of what it remembers instead, as visuals of their own ([`remembered`]).

mod remembered;
mod resources;
mod structures;
mod units;

use std::collections::BTreeMap;

use bevy::prelude::*;
use sc_sim::{Category, Map, ObjectId, ObjectKind, Placement, PlayerId};

use crate::ClientSystems;
use crate::coords::{DrawLayer, facing_to_rotation, footprint_height, ground_translation, lerp_facing};
use crate::shapes::vertex_color_material;
use crate::sim_driver::{RenderAlpha, SimRes};

/// Links a visual entity to the sim object it shows.
#[derive(Component, Copy, Clone, Debug, PartialEq, Eq)]
pub struct ObjectVisual(pub ObjectId);

/// The facing tick of a visual whose body doesn't turn (`units::body_turns`): a child of the object's visual, turned
/// by the object's facing.
#[derive(Component, Copy, Clone, Debug, PartialEq, Eq)]
pub struct FacingTick(pub ObjectId);

/// The Riot Shield of a unit that carries one (`factions/gdo/units.md > Peacekeeper > Riot Shield`): a child of the
/// unit's visual, hidden while the shield is down, else inheriting the visual's visibility, so the fog hides it with
/// its unit (`crate::fog`). Placeholder until the art exists (Q0001).
#[derive(Component, Copy, Clone, Debug, PartialEq, Eq)]
pub struct RiotShieldVisual(pub ObjectId);

pub use remembered::{RememberedVisual, RememberedVisuals};
pub use structures::StructureTag;
pub use units::UnitTag;

/// What the mesh of an object's visual depends on ([`MeshKey::of`]).
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct MeshKey {
    kind: ObjectKind,
    /// The owner whose colour it is drawn in: a unit's or a structure's; none for map resources.
    owner: Option<PlayerId>,
    /// A structure's size as placed (`Placement::w`, `Placement::h`, turned).
    size: Option<(u32, u32)>,
}

impl MeshKey {
    fn of(kind: ObjectKind, owner: Option<PlayerId>, placement: Option<&Placement>) -> Self {
        let def = kind.def();
        Self {
            kind,
            owner: owner.filter(|_| def.category != Category::Resource),
            size: placement.filter(|_| def.structure.is_some()).map(|p| (p.w, p.h)),
        }
    }
}

/// Mesh and material handles, created once per object kind (and owner, for units and structures, and size as
/// placed, for structures) and shared by all its visuals.
#[derive(Resource, Default)]
pub struct ObjectVisualAssets {
    material: Option<Handle<ColorMaterial>>,
    /// The remembered objects' material ([`remembered`]).
    dimmed: Option<Handle<ColorMaterial>>,
    meshes: BTreeMap<MeshKey, Handle<Mesh>>,
    ticks: BTreeMap<ObjectKind, Handle<Mesh>>,
    shields: BTreeMap<ObjectKind, Handle<Mesh>>,
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
            .init_resource::<RememberedVisuals>()
            .add_systems(Update, sync_object_visuals.in_set(ClientSystems::SyncSim))
            .add_systems(
                Update,
                (
                    place_object_visuals,
                    place_riot_shields,
                    remembered::sync_remembered_visuals,
                )
                    .in_set(ClientSystems::Present),
            );
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

/// The placeholder shape of each object kind, centered on the object's position; units and structures in the
/// neutral colour, structures unturned.
pub fn placeholder_mesh(kind: ObjectKind) -> Mesh {
    object_mesh(kind, None)
}

/// The placeholder shape of an object of `kind` owned by `owner`, a structure unturned. Units and structures take
/// their owner's colour.
pub fn object_mesh(kind: ObjectKind, owner: Option<PlayerId>) -> Mesh {
    match kind {
        ObjectKind::CrystalPatch => resources::crystal_patch_mesh(),
        ObjectKind::SupplyDeliveryStation => resources::supply_station_mesh(),
        ObjectKind::Peacekeeper
        | ObjectKind::Guard
        | ObjectKind::Drone
        | ObjectKind::SabreLegionnaire
        | ObjectKind::RifleLegionnaire => units::unit_mesh(kind, player_color(owner), true),
        ObjectKind::DeploymentCenter
        | ObjectKind::Generator
        | ObjectKind::Barracks
        | ObjectKind::ExtractionFacility
        | ObjectKind::ExtractionPlate
        | ObjectKind::Beacon
        | ObjectKind::EgressNode
        | ObjectKind::RoboPortal
        | ObjectKind::BioPortal => {
            let (w, h) = kind.def().tile_size().unwrap_or((1, 1));
            structures::structure_mesh(w, h, player_color(owner))
        }
    }
}

/// Whether a visual of `kind` turns with its object's facing: only a unit's round body does (`units::body_turns`).
/// Structures and map resources are drawn as placed, whatever their facing (a structure that was a unit keeps the
/// unit's: `Sim::change_kind`).
fn body_turns(kind: ObjectKind) -> bool {
    kind.def().unit.is_some() && units::body_turns(kind)
}

/// Whether a visual of `kind` shows its object's facing by a tick of its own ([`FacingTick`]): a unit whose body
/// doesn't turn.
fn has_tick(kind: ObjectKind) -> bool {
    kind.def().unit.is_some() && !units::body_turns(kind)
}

/// The mesh of an object's visual: its placeholder shape, a unit's without the tick if its body doesn't turn, a
/// structure in its size as placed.
fn visual_mesh(key: MeshKey) -> Mesh {
    let MeshKey { kind, owner, size } = key;
    match size {
        Some((w, h)) => structures::structure_mesh(w, h, player_color(owner)),
        None if kind.def().unit.is_some() => units::unit_mesh(kind, player_color(owner), units::body_turns(kind)),
        None => object_mesh(kind, owner),
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

/// The colour of `owner`'s units and structures: grey for neutral ones.
pub fn player_color(owner: Option<PlayerId>) -> Color {
    match owner {
        Some(p) => PLAYER_COLORS[usize::from(p.0.max(1) - 1) % PLAYER_COLORS.len()],
        None => NEUTRAL_COLOR,
    }
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
        let key = MeshKey::of(object.kind, object.owner, object.placement.as_ref());
        let mesh = assets
            .meshes
            .entry(key)
            .or_insert_with(|| meshes.add(visual_mesh(key)))
            .clone();
        let height = object_height(sim.map(), object.pos, object.placement.as_ref());
        let facing = if body_turns(object.kind) { object.facing } else { 0.0 };
        let entity = commands
            .spawn((
                Name::new(format!("{:?} #{}", object.kind, object.id.0)),
                ObjectVisual(object.id),
                Mesh2d(mesh),
                MeshMaterial2d(material.clone()),
                object_transform(object.pos, height, facing),
            ))
            .id();
        if let Some(size) = key.size {
            commands.spawn((structures::tag(object.id, object.kind, size, 1.0), ChildOf(entity)));
        }
        if units::tagged(object.kind) {
            commands.spawn((units::tag(object.id, object.kind), ChildOf(entity)));
        }
        if has_tick(object.kind) {
            let tick = assets
                .ticks
                .entry(object.kind)
                .or_insert_with(|| meshes.add(units::tick_mesh(object.kind)))
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
        if object.riot_shield.is_some() {
            let shield = assets
                .shields
                .entry(object.kind)
                .or_insert_with(|| meshes.add(units::riot_shield_mesh(object.kind)))
                .clone();
            commands.spawn((
                Name::new("Riot Shield"),
                RiotShieldVisual(object.id),
                Mesh2d(shield),
                MeshMaterial2d(material.clone()),
                Transform::default(),
                Visibility::Hidden,
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

/// Shows each Riot Shield that isn't down, sized by how far up it is (from 60% of its size, close to the body, to
/// all of it), and turned with its unit's facing when the body doesn't turn. Never `Visible`: a shown shield
/// inherits its unit's visibility.
fn place_riot_shields(
    sim: Res<SimRes>,
    alpha: Res<RenderAlpha>,
    mut shields: Query<(&RiotShieldVisual, &mut Transform, &mut Visibility)>,
) {
    for (shield, mut transform, mut visibility) in &mut shields {
        let Some(object) = sim.object(shield.0) else {
            continue;
        };
        let (Some(mode), Some(def)) = (object.riot_shield, sc_sim::riot_shield::def_of(object)) else {
            continue;
        };
        let shown = mode != sc_sim::RiotShield::Lowered;
        visibility.set_if_neq(if shown {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
        let scale = 0.6 + 0.4 * mode.progress(def.toggle_ticks);
        let rotation = if body_turns(object.kind) {
            Quat::IDENTITY
        } else {
            facing_to_rotation(lerp_facing(object.prev_facing, object.facing, alpha.0))
        };
        transform.set_if_neq(Transform::from_rotation(rotation).with_scale(Vec3::new(scale, scale, 1.0)));
    }
}
