//! The Tunnel Area of a Tunnel being placed (slice M4-J1; `factions/syndicate/tunnels.md > Tunnel Area Non-Overlap
//! Rule`, design change 0242: "a Tunnel can only be placed if every cell of its Tunnel Area is on the map and
//! Drillable"): around the placement ghost (`super::ghost`), the Tier 1 area the Tunnel would have
//! (`sc_sim::tunnel::area::base_radius`), green while the check its click would face finds nothing wrong with the
//! area, red while it does (a cell past the map's edge, on known undrillable ground, or in a Tunnel Area the player
//! knows of: `WholeProblem::TunnelAreaOffMap`, `TunnelAreaNotDrillable`, `TunnelAreaOverlap`; what is wrong is the
//! invalid click's message, `crate::command_ui::feedback`). The areas it must keep clear of show meanwhile
//! (`super::tunnel_areas::shows`).
//!
//! The look, placeholders until the art (Q0001): the Tunnel Area overlay's (`super::overlay::tint_area`) in the
//! ghost's green or red, fainter than the ghost's own tiles; not on the tiles the player hasn't explored, which stay
//! black (`core/vision.md > Unexplored`), nor off the map. One mesh, under one handle, rebuilt in place only when the
//! area, its verdict, the unexplored tiles among it or the map changed.

use std::collections::HashSet;

use bevy::prelude::*;
use bevy::sprite_render::AlphaMode2d;
use sc_sim::nav::TileRect;
use sc_sim::tunnel;
use sc_sim::{PlacementCheck, TilePos, VisibilityState, WholeProblem};

use super::ghost::{Ghost, INVALID_EDGE, PlacementGhost, VALID_EDGE};
use super::overlay::{AREA_Z, tint_area};
use crate::coords::DrawLayer;
use crate::fog::LocalSight;
use crate::map_render::MapView;
use crate::shapes::ColorMesh;

/// The tint of the area while nothing is wrong with it, and while something is (placeholder colours).
pub const AREA_VALID_FILL: Color = Color::srgba(0.25, 0.95, 0.35, 0.16);
pub const AREA_INVALID_FILL: Color = Color::srgba(1.0, 0.22, 0.18, 0.22);
/// Its z above the terrain entity's: in front of the Tunnel Area overlay's, which shows the areas it must keep clear
/// of, under the lingering ghosts'.
const GHOST_AREA_Z: f32 = AREA_Z + 0.005;

/// The Tier 1 area of a Tunnel ghost, and whether nothing is wrong with it.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ShownArea {
    pub area: TileRect,
    pub clear: bool,
}

impl ShownArea {
    /// The area of `ghost`, if it is a Tunnel's (a type with a tier table).
    pub fn of(ghost: &Ghost) -> Option<Self> {
        let radius = tunnel::area::base_radius(ghost.kind)?;
        let (w, h) = ghost.size;
        Some(Self {
            area: tunnel::area::area(TileRect::new(ghost.origin, w, h), radius),
            clear: area_is_clear(&ghost.check),
        })
    }
}

/// Whether `check` finds nothing wrong with the Tunnel Area (design change 0242; the area's rules are rules of the
/// footprint as a whole, `sc_sim::WholeProblem`).
pub fn area_is_clear(check: &PlacementCheck) -> bool {
    !check.whole.iter().any(|problem| {
        matches!(
            problem,
            WholeProblem::TunnelAreaOffMap | WholeProblem::TunnelAreaNotDrillable | WholeProblem::TunnelAreaOverlap
        )
    })
}

/// What the ghost's area shows.
#[derive(Resource, Debug, Default)]
pub struct GhostArea {
    /// The area shown, while a Tunnel ghost shows.
    pub shown: Option<ShownArea>,
    /// The tiles of that area left out: those the local player hadn't explored.
    unexplored: Vec<TilePos>,
    /// The map meshes' build count (`MapView::builds`) when the mesh was built: a new map redraws it.
    map_builds: u32,
    /// How many times its mesh was built.
    pub builds: u32,
}

/// The area's mesh.
#[derive(Component)]
pub struct GhostAreaMesh;

#[derive(Resource)]
pub(super) struct GhostAreaAssets {
    mesh: Handle<Mesh>,
}

pub(super) fn spawn_ghost_area(
    mut commands: Commands,
    meshes: Res<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // The mesh asset arrives with the first area to draw: the renderer warns about empty meshes.
    let mesh = meshes.reserve_handle();
    let material = ColorMaterial {
        color: Color::WHITE,
        alpha_mode: AlphaMode2d::Blend,
        ..default()
    };
    commands.spawn((
        Name::new("Tunnel ghost's area"),
        GhostAreaMesh,
        Mesh2d(mesh.clone()),
        MeshMaterial2d(materials.add(material)),
        Transform::from_xyz(0.0, 0.0, DrawLayer::Terrain.base_z() + GHOST_AREA_Z),
        Visibility::Hidden,
    ));
    commands.insert_resource(GhostAreaAssets { mesh });
}

/// Shows the area of the placement ghost while it is a Tunnel's, but on the tiles the local player hasn't explored,
/// redrawn when it changed. It runs after the ghost's update (`super::ghost::update_ghost`).
pub(super) fn draw_ghost_area(
    local: LocalSight,
    ghost: Res<PlacementGhost>,
    view: Res<MapView>,
    assets: Res<GhostAreaAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut shown: ResMut<GhostArea>,
    mut drawn: Query<&mut Visibility, With<GhostAreaMesh>>,
) {
    let (sim, sight) = (local.sim(), local.get());
    let map = sim.map();
    let area = ghost.shown.as_ref().and_then(ShownArea::of);
    let unexplored: Vec<TilePos> = (area.iter())
        .flat_map(|shown| tunnel::area::cells(shown.area))
        .filter(|&t| map.in_bounds(t) && sight.tile(t) == VisibilityState::Unexplored)
        .collect();
    let mut visible = false;
    if let Some(area) = area {
        if shown.shown != Some(area) || shown.unexplored != unexplored || shown.map_builds != view.builds {
            let hidden: HashSet<TilePos> = unexplored.iter().copied().collect();
            let colors = if area.clear {
                (AREA_VALID_FILL, VALID_EDGE)
            } else {
                (AREA_INVALID_FILL, INVALID_EDGE)
            };
            let mut mesh = ColorMesh::triangles();
            let on_map = |t: TilePos| map.in_bounds(t) && !hidden.contains(&t);
            tint_area(&mut mesh, map, tunnel::area::cells(area.area), on_map, colors);
            if mesh.vertex_count() > 0
                && let Err(error) = meshes.insert(&assets.mesh, mesh.into_mesh())
            {
                error!("Tunnel ghost's area mesh: {error}");
            }
            shown.map_builds = view.builds;
            shown.builds += 1;
        }
        visible = unexplored.len() < tunnel::area::cells(area.area).filter(|&t| map.in_bounds(t)).count();
    }
    if shown.shown != area {
        shown.shown = area;
    }
    if shown.unexplored != unexplored {
        shown.unexplored = unexplored;
    }
    let visibility = if visible {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    for mut mesh in &mut drawn {
        mesh.set_if_neq(visibility);
    }
}
