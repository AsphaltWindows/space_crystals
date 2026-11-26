//! The Tunnel Area display (slices M4-J2 and M4-J3; `factions/syndicate/tunnel_area_and_space.md > Tunnel Area
//! display`, design changes 0299 and 0300, answering Q0086 point 4): when the local player is shown a Tunnel Area on
//! the map (`factions/syndicate/tunnel_area_and_space.md > Tunnel Area`: "a square underground build zone centered on
//! the Tunnel").
//!
//! - **The selected Tunnels' areas** ([`selected_areas`]): "**Selecting one of your Tunnels** shows its Tunnel Area on
//!   the map". A reading: the area of each own Tunnel in the selection, the ActiveGroup or not (a Tunnel is
//!   Ungroupable, so two selected are two groups), a site's too (a Tunnel under construction, whose area counts from
//!   the moment it appears: design change 0289 point 3). Each at its tier now (`sc_sim::tunnel::area::of`): an own
//!   Tunnel that upgrades shows its current area, the one its expansions lie in; the target tier's area it reserves
//!   meanwhile is a rule of the placements (slice M4-G), not of the display. It hides as the selection changes.
//!   "**Placing an expansion** (from a selected Tunnel) shows that Tunnel's area too": that placement is its
//!   Tunnel's, which stays selected all along (slice M4-F).
//! - **The rings of a Tunnel ghost** ([`ghost_rings`], [`rings`]): "**Placing a Tunnel** shows the ghost's Tier 1 area
//!   **and the would-be Tier 2 and Tier 3 areas**", their cells that would block a later upgrade red. The Tier 1 area
//!   is the ghost's own (`super::ghost_area`); the rings around it follow the ghost (`super::ghost`) tile by tile.
//! - **Never an enemy's area**: "**Enemy Tunnel Areas are never shown**", neither one the player sees nor one they
//!   remember. The ghost still judges by them (design change 0237: "The ghost judges enemy Tunnels by what the player
//!   knows"): red over a known enemy Tunnel's area, with the reason its click says (`crate::command_ui::feedback`).
//! - No view modes with toggles ("maybe later").
//!
//! It shows in the game modes only. **The look**, placeholders until the art (Q0001): the GDO build area's
//! (`super::overlay`), each area's tiles tinted on the terrain's surface with a brighter line along its edge, a
//! selected Tunnel's in violet, the rings in theirs ([`rings::RING_COLORS`]); not on the tiles the player hasn't
//! explored, which stay black (`core/vision.md > Unexplored`), nor off the map. One mesh, under one handle, rebuilt in
//! place only when what it shows, the unexplored tiles among it or the map changed: while a Tunnel is placed, when the
//! ghost moves to another tile or a ring's red cells change, never every frame.

pub mod rings;

use std::collections::HashSet;

use bevy::prelude::*;
use bevy::sprite_render::AlphaMode2d;
use sc_sim::nav::TileRect;
use sc_sim::{Map, ObjectId, PlayerId, Sim, TilePos, VisibilityState, tunnel};

use super::ghost::{Ghost, PlacementGhost};
use super::overlay::{AREA_Z, tint_area};
use super::placing_now;
use crate::ClientMode;
use crate::command_ui::Builder;
use crate::coords::DrawLayer;
use crate::fog::LocalSight;
use crate::input::{ActionState, InputAction};
use crate::map_render::MapView;
use crate::selection::{ControlStates, Selection};
use crate::shapes::ColorMesh;
pub use rings::Ring;

/// The tint and the edge of a selected Tunnel's area (placeholder colours).
pub const OWN_FILL: Color = Color::srgba(0.70, 0.45, 1.0, 0.16);
pub const OWN_EDGE: Color = Color::srgba(0.80, 0.60, 1.0, 0.85);

/// The Tunnel Area of a Tunnel of the local player's in their selection ([`selected_areas`]).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct TunnelArea {
    /// The Tunnel whose area it is.
    pub tunnel: ObjectId,
    /// Its tier now.
    pub tier: u8,
    /// The area at that tier (`sc_sim::tunnel::area::of`), which may reach past the map's edge.
    pub area: TileRect,
}

/// The areas of `player`'s own Tunnels in `selection`, sites included, each at its tier now (module docs), in id
/// order; none of another player's Tunnel.
pub fn selected_areas(sim: &Sim, player: PlayerId, selection: &Selection) -> Vec<TunnelArea> {
    let objects = selection.ids().filter_map(|id| sim.object(id));
    let own = objects.filter(|o| o.owner == Some(player));
    own.filter_map(|o| {
        Some(TunnelArea {
            tunnel: o.id,
            tier: o.tunnel.as_ref()?.tier,
            area: tunnel::area::of(o)?,
        })
    })
    .collect()
}

/// The rings of `player`'s Tunnel ghost `ghost` ([`rings::rings`]), none for another kind's: red where they would block
/// an upgrade as far as the player knows the ground (`sc_sim::Vision::state`, as `sc_sim::placement::Mode::Known` has
/// it), and over the areas of their own other Tunnels as the ghost's check counts them ([`rings::own_areas`]): without
/// `queue` (Shift), the click replaces the orders of `builder`, an Agent, whose pending ghosts then don't count.
pub fn ghost_rings(sim: &Sim, player: PlayerId, ghost: &Ghost, builder: Option<Builder>, queue: bool) -> Vec<Ring> {
    if tunnel::def_of(ghost.kind).is_none() {
        return Vec::new();
    }
    let replaced = match builder {
        Some(Builder::Agent(agent)) if !queue => Some(agent),
        _ => None,
    };
    let own = rings::own_areas(sim, player, replaced);
    let known = |t: TilePos| sim.vision().state(player, t) != VisibilityState::Unexplored;
    let (w, h) = ghost.size;
    rings::rings(sim.map(), ghost.kind, TileRect::new(ghost.origin, w, h), known, &own)
}

/// The overlay's mesh: each of the `selected` areas in the own colours ([`tint_area`]), then the `rings`
/// ([`rings::draw`]), on their tiles on `map` that are `drawn`, with no edge where an area goes on past the map's edge
/// or into a tile not drawn. Each area has its own edge, also where two touch (the Tunnel Areas never overlap:
/// `factions/syndicate/tunnel_area_and_space.md > Tunnel Area Non-Overlap Rule`).
pub fn overlay_mesh(map: &Map, selected: &[TunnelArea], rings: &[Ring], drawn: impl Fn(TilePos) -> bool) -> ColorMesh {
    let mut mesh = ColorMesh::triangles();
    let on_map = |t: TilePos| map.in_bounds(t) && drawn(t);
    for shown in selected {
        let cells = tunnel::area::cells(shown.area);
        tint_area(&mut mesh, map, cells, on_map, (OWN_FILL, OWN_EDGE));
    }
    rings::draw(&mut mesh, map, rings, drawn);
    mesh
}

/// What the Tunnel Area display shows.
#[derive(Resource, Debug, Default)]
pub struct TunnelAreaOverlay {
    /// The selected Tunnels' areas its mesh was last built from.
    pub selected: Vec<TunnelArea>,
    /// The Tunnel ghost's rings it was built from.
    pub rings: Vec<Ring>,
    /// The tiles of those areas and rings on the map it left out, in their order: those the local player hadn't
    /// explored.
    pub unexplored: Vec<TilePos>,
    /// Whether that mesh has anything to show.
    drawn: bool,
    /// The map meshes' build count (`MapView::builds`) when it was: a new map redraws it.
    map_builds: u32,
    /// Whether it shows now: something to show, in part on explored tiles at least.
    pub visible: bool,
    /// How many times its mesh was built.
    pub builds: u32,
}

/// The overlay's mesh.
#[derive(Component)]
pub struct TunnelAreaMesh;

#[derive(Resource)]
pub(super) struct TunnelAreaAssets {
    mesh: Handle<Mesh>,
}

pub(super) fn spawn_overlay(
    mut commands: Commands,
    meshes: Res<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // The mesh asset arrives with the first areas to draw: the renderer warns about empty meshes.
    let mesh = meshes.reserve_handle();
    let material = ColorMaterial {
        color: Color::WHITE,
        alpha_mode: AlphaMode2d::Blend,
        ..default()
    };
    commands.spawn((
        Name::new("Tunnel Area overlay"),
        TunnelAreaMesh,
        Mesh2d(mesh.clone()),
        MeshMaterial2d(materials.add(material)),
        Transform::from_xyz(0.0, 0.0, DrawLayer::Terrain.base_z() + AREA_Z),
        Visibility::Hidden,
    ));
    commands.insert_resource(TunnelAreaAssets { mesh });
}

/// Shows the local player's selected Tunnels' areas and their Tunnel ghost's rings in the game modes (module docs), but
/// on the tiles they haven't explored, redrawn when what it shows or those tiles changed. It runs after the ghost's
/// update (`super::ghost::update_ghost`), with Shift as the ghost's check has it.
#[expect(clippy::too_many_arguments, reason = "a system's parameters")]
pub(super) fn draw_overlay(
    mode: Res<ClientMode>,
    local: LocalSight,
    states: Res<ControlStates>,
    ghost: Res<PlacementGhost>,
    actions: Res<ActionState>,
    view: Res<MapView>,
    assets: Res<TunnelAreaAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut overlay: ResMut<TunnelAreaOverlay>,
    mut shown: Query<&mut Visibility, With<TunnelAreaMesh>>,
) {
    let (sim, sight) = (local.sim(), local.get());
    let player = sight.player();
    let (selected, rings) = if mode.is_game() {
        let builder = placing_now(*mode, sim, player, &states).map(|placing| placing.builder);
        let queue = actions.held(InputAction::Queue);
        let rings = (ghost.shown.as_ref()).map_or_else(Vec::new, |g| ghost_rings(sim, player, g, builder, queue));
        (selected_areas(sim, player, &states.of(player).selection), rings)
    } else {
        (Vec::new(), Vec::new())
    };
    let cells =
        (selected.iter().flat_map(|shown| tunnel::area::cells(shown.area))).chain(rings.iter().flat_map(Ring::cells));
    let unexplored: Vec<TilePos> = cells
        .filter(|&t| sim.map().in_bounds(t) && sight.tile(t) == VisibilityState::Unexplored)
        .collect();
    let changed = overlay.selected != selected || overlay.rings != rings || overlay.unexplored != unexplored;
    if changed || overlay.map_builds != view.builds {
        overlay.drawn = false;
        if !selected.is_empty() || !rings.is_empty() {
            let hidden: HashSet<TilePos> = unexplored.iter().copied().collect();
            let mesh = overlay_mesh(sim.map(), &selected, &rings, |t| !hidden.contains(&t));
            overlay.drawn = mesh.vertex_count() > 0;
            if overlay.drawn
                && let Err(error) = meshes.insert(&assets.mesh, mesh.into_mesh())
            {
                error!("Tunnel Area overlay mesh: {error}");
            }
            overlay.builds += 1;
        }
        overlay.selected = selected;
        overlay.rings = rings;
        overlay.unexplored = unexplored;
        overlay.map_builds = view.builds;
    }
    let visible = overlay.drawn;
    if overlay.visible != visible {
        overlay.visible = visible;
    }
    let visibility = if visible {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    for mut mesh in &mut shown {
        mesh.set_if_neq(visibility);
    }
}

#[cfg(test)]
mod tests;
