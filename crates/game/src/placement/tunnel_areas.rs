//! The Tunnel Area overlay (slice M4-J2, plan decision T41): the Tunnel Areas the local player knows of
//! (`factions/syndicate/tunnels.md > Tunnel Area`: "a square underground build zone centered on the Tunnel";
//! [`known_areas`]). Their own Tunnels' at the tier they are at now; the others' as the player knows them: those they
//! see at their tier now ("Enemies see a Tunnel's tier", `factions/syndicate/tunnels.md > Tunnel Area Non-Overlap
//! Rule`), those they remember where the record stands, at the tier last seen (design change 0237: "their remembered
//! position and their tier as last seen", `sc_sim::vision::LastKnown::tunnel`). A Tunnel they have never seen, or
//! don't see now and don't remember, has none: the overlay never draws what the player doesn't know
//! (`crate::fog::Sight`).
//!
//! **When it shows** ([`shows`]). Placeholder (not in the design; Q0086 point 4): while the local player's ActiveGroup
//! is a Tunnel of theirs, in a game mode, while they may command it, and while they place a Tunnel (slice M4-J1: the
//! Tier 1 area the ghost needs clear of every area they know of, design change 0242). The placement of an expansion
//! (slice M4-F) is its Tunnel's, the ActiveGroup all along: the area its ghost is held within shows meanwhile.
//!
//! **The look**, placeholders until the art (Q0001): the GDO build area's (`super::overlay`), each area's tiles tinted
//! on the terrain's surface with a brighter line along its edge, the player's own in violet and the others' in red;
//! not on the tiles they haven't explored, which stay black (`core/vision.md > Unexplored`), nor off the map. One mesh,
//! under one handle, rebuilt in place only when the areas, the unexplored tiles among them or the map changed.

use std::collections::HashSet;

use bevy::prelude::*;
use bevy::sprite_render::AlphaMode2d;
use sc_sim::nav::TileRect;
use sc_sim::tunnel::{self, BASE_TIER};
use sc_sim::{Map, ObjectId, PlayerId, Sim, TilePos, VisibilityState};

use super::overlay::{AREA_Z, tint_area};
use crate::ClientMode;
use crate::command_ui::interface::commandable;
use crate::coords::DrawLayer;
use crate::fog::{LocalSight, Sight};
use crate::map_render::MapView;
use crate::selection::ControlStates;
use crate::shapes::ColorMesh;

/// The tint and the edge of the player's own areas, and of the others' (placeholder colours).
pub const OWN_FILL: Color = Color::srgba(0.70, 0.45, 1.0, 0.16);
pub const OWN_EDGE: Color = Color::srgba(0.80, 0.60, 1.0, 0.85);
pub const FOREIGN_FILL: Color = Color::srgba(1.0, 0.35, 0.30, 0.16);
pub const FOREIGN_EDGE: Color = Color::srgba(1.0, 0.45, 0.40, 0.85);

/// A Tunnel Area the local player knows of ([`known_areas`]).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct KnownArea {
    /// The Tunnel whose area it is.
    pub tunnel: ObjectId,
    /// Whether that Tunnel is the player's own.
    pub own: bool,
    /// Its tier as the player knows it: now for a Tunnel they see, as last seen for one they remember.
    pub tier: u8,
    /// The area at that tier (`sc_sim::tunnel::area`), which may reach past the map's edge.
    pub area: TileRect,
}

/// The Tunnel Areas the player whose `sight` this is knows of, in Tunnel id order (module docs): of each Tunnel they
/// see (their own always) at its tier now, of each Tunnel they remember ([`Sight::remembered`]) at its record, at the
/// tier last seen; none of a Tunnel they neither see nor remember.
pub fn known_areas(sim: &Sim, sight: &Sight) -> Vec<KnownArea> {
    let player = sight.player();
    let tunnels = sim.objects().filter(|o| tunnel::def_of(o.kind).is_some());
    let seen = tunnels.filter(|o| sight.sees(o)).filter_map(|o| {
        Some(KnownArea {
            tunnel: o.id,
            own: o.owner == Some(player),
            tier: o.tunnel.as_ref()?.tier,
            area: tunnel::area::of(o)?,
        })
    });
    let remembered = sight.remembered().filter_map(|record| {
        // A record of a Tunnel keeps its tier (`LastKnown::tunnel`); one without, the base tier.
        let tier = record.tunnel.as_ref().map_or(BASE_TIER, |state| state.tier);
        let radius = tunnel::def_of(record.kind)?.tier(tier)?.area_radius;
        let p = record.placement;
        Some(KnownArea {
            tunnel: record.id,
            own: record.owner == Some(player),
            tier,
            area: tunnel::area::area(TileRect::new(p.origin, p.w, p.h), radius),
        })
    });
    let mut areas: Vec<KnownArea> = seen.chain(remembered).collect();
    // A Tunnel seen shows no record (`Sight::remembered_record`); were both there, the stable sort keeps the one seen.
    areas.sort_by_key(|known| known.tunnel);
    areas.dedup_by_key(|known| known.tunnel);
    areas
}

/// Whether the overlay shows for `player` now (module docs): in a game mode, while their selection is commandable
/// (`crate::command_ui::interface::commandable`: never once the game is over or they are defeated) and its ActiveGroup
/// is a Tunnel, or they place one (an Agent's Build Tunnel, slice M4-J1). Placeholder (not in the design).
pub fn shows(mode: ClientMode, sim: &Sim, player: PlayerId, states: &ControlStates) -> bool {
    let state = states.of(player);
    let placing = (state.interface.placing()).is_some_and(|placing| tunnel::def_of(placing.kind).is_some());
    mode.is_game()
        && commandable(sim, player, &state.selection)
        && (placing || (state.selection.active()).is_some_and(|active| tunnel::def_of(active.kind).is_some()))
}

/// The overlay's mesh: each of `areas` ([`tint_area`]) in its owner's colours, its tiles on `map` that are `drawn`,
/// with no edge where it goes on past the map's edge or into a tile not drawn. Each area has its own edge, also where
/// two touch (the Tunnel Areas never overlap: `factions/syndicate/tunnels.md > Tunnel Area Non-Overlap Rule`).
pub fn areas_mesh(map: &Map, areas: &[KnownArea], drawn: impl Fn(TilePos) -> bool) -> ColorMesh {
    let mut mesh = ColorMesh::triangles();
    for known in areas {
        let colors = if known.own {
            (OWN_FILL, OWN_EDGE)
        } else {
            (FOREIGN_FILL, FOREIGN_EDGE)
        };
        let on_map = |t: TilePos| map.in_bounds(t) && drawn(t);
        tint_area(&mut mesh, map, tunnel::area::cells(known.area), on_map, colors);
    }
    mesh
}

/// What the Tunnel Area overlay shows.
#[derive(Resource, Debug, Default)]
pub struct TunnelAreaOverlay {
    /// The areas its mesh was last built from.
    pub shown: Vec<KnownArea>,
    /// The tiles of those areas it left out, in their order: those the local player hadn't explored.
    pub unexplored: Vec<TilePos>,
    /// Whether that mesh has anything to show.
    drawn: bool,
    /// The map meshes' build count (`MapView::builds`) when it was: a new map redraws it.
    map_builds: u32,
    /// Whether it shows now ([`shows`]), with something to show.
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

/// Shows the Tunnel Areas the local player knows of while it [`shows`], but on the tiles they haven't explored,
/// redrawn when either changed.
#[expect(clippy::too_many_arguments, reason = "a system's parameters")]
pub(super) fn draw_overlay(
    mode: Res<ClientMode>,
    local: LocalSight,
    states: Res<ControlStates>,
    view: Res<MapView>,
    assets: Res<TunnelAreaAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut overlay: ResMut<TunnelAreaOverlay>,
    mut shown: Query<&mut Visibility, With<TunnelAreaMesh>>,
) {
    let (sim, sight) = (local.sim(), local.get());
    let showing = shows(*mode, sim, sight.player(), &states);
    if showing {
        let areas = known_areas(sim, &sight);
        let unexplored: Vec<TilePos> = (areas.iter())
            .flat_map(|known| tunnel::area::cells(known.area))
            .filter(|&t| sim.map().in_bounds(t) && sight.tile(t) == VisibilityState::Unexplored)
            .collect();
        if overlay.shown != areas || overlay.unexplored != unexplored || overlay.map_builds != view.builds {
            let hidden: HashSet<TilePos> = unexplored.iter().copied().collect();
            let mesh = areas_mesh(sim.map(), &areas, |t| !hidden.contains(&t));
            overlay.drawn = mesh.vertex_count() > 0;
            if overlay.drawn
                && let Err(error) = meshes.insert(&assets.mesh, mesh.into_mesh())
            {
                error!("Tunnel Area overlay mesh: {error}");
            }
            overlay.shown = areas;
            overlay.unexplored = unexplored;
            overlay.map_builds = view.builds;
            overlay.builds += 1;
        }
    }
    let visible = showing && overlay.drawn;
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
