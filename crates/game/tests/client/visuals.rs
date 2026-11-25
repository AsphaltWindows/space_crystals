//! Visuals (`ARCHITECTURE.md > Client architecture > Visuals`): objects on the terrain and their picking, one
//! visual per object, draw order, facing, the grid overlay, loading another map, and interpolation between
//! ticks.
//!
//! By topic: the map, objects on its terrain and the grid overlay, and another one loaded (`map`); one visual
//! per object, the draw order, and a kind change (`objects`); how each kind is drawn, its facing included
//! (`kinds`); and interpolation between ticks (`interpolation`).

mod interpolation;
mod kinds;
mod map;
mod objects;

use std::collections::BTreeSet;

use bevy::mesh::VertexAttributeValues;
use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;

use sc_sim::{
    Command, ConstructionCommand, DebugCommand, ObjectId, ObjectKind, OrderKind, PlayerCommand, PlayerId, Sim, SpawnAt,
    Target, TilePos, scenarios,
};
use space_crystals::camera::{CameraView, ViewportInsets, cursor_to_ground, viewport_rect};
use space_crystals::coords::{drawn_position, lift};
use space_crystals::hud::selection_panel::short_label;
use space_crystals::input::InputAction;
use space_crystals::map_render::{GridOverlay, MapRenderAssets, MapView};
use space_crystals::object_render::{FacingTick, ObjectVisuals, StructureTag, UnitTag};
use space_crystals::platform::SurfaceSize;
use space_crystals::sim_driver::{PendingCommands, RecordedInputs, RenderAlpha, SimRes, load_scenario};

use super::{
    camera_center, debug, headless_app, patch_at, plateau_scenario, press, sim_ids, small_scenario, tick, updates,
    visual_ids,
};

/// The lowest and the highest corner of the box around a visual's mesh, in the visual's space.
fn mesh_bounds(app: &App, entity: Entity) -> (Vec3, Vec3) {
    let handle = &app.world().get::<Mesh2d>(entity).expect("a mesh").0;
    let mesh = app.world().resource::<Assets<Mesh>>().get(handle).unwrap();
    let Some(VertexAttributeValues::Float32x3(positions)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) else {
        panic!("a 2D mesh has positions");
    };
    let bounds = (Vec3::INFINITY, Vec3::NEG_INFINITY);
    positions
        .iter()
        .map(|&p| Vec3::from(p))
        .fold(bounds, |(lo, hi), p| (lo.min(p), hi.max(p)))
}

/// The structure tags among `entity`'s children: whose they are, and what they say.
fn tags(app: &App, entity: Entity) -> Vec<(ObjectId, String)> {
    let world = app.world();
    let Some(children) = world.get::<Children>(entity) else {
        return Vec::new();
    };
    let tag = |child| {
        Some((
            world.get::<StructureTag>(child)?.0,
            world.get::<Text2d>(child)?.0.clone(),
        ))
    };
    children.iter().filter_map(tag).collect()
}
