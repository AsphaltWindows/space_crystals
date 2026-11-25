//! The lingering build ghosts (`core/entities.md > Placement Validation > Lingering build ghosts`, design change
//! 0125): "Every ordered worker build (Drone Deploy, Agent build, Cults Recruits' Construct), including every
//! Shift-queued one, leaves a ghost of the building on its site until construction actually starts", "Visible to its
//! owner. Enemies never see it". The local player's pending worker builds (`Sim::pending_builds`), each the footprint
//! of what it builds on its site, in the game modes: a Drone's Beacon, an Agent's Tunnel (slice M4-E), the Tunnel's
//! with its Tier 1 Tunnel Area (slice M4-J1), which it reserves too (`sc_sim::WholeProblem::TunnelAreaOverlap`:
//! Q0079 point 2). A ghost goes with its order: when the deployment or the site starts, or the order is cancelled,
//! replaced by a new one, or fails. They block no one (the sim's rule: `crates/sim/tests/deploy_gate/`,
//! `pending_sites_block_no_one`) and reserve their sites against the player's further placements, which the placement
//! ghost shows red there (`super::ghost`). They are the local player's own: another player's never show
//! (`sim_driver::LocalPlayer`, the view the client draws).
//!
//! One mesh, under one handle, rebuilt in place only when the sites or the map change. The look, placeholders until
//! the art (Q0001): each footprint tile tinted pale blue on the terrain's surface, as the terrain mesh draws it, and
//! an outline around each footprint; a Tunnel ghost's area fainter, with its own outline, as the Tunnel Area overlay
//! draws an area (`super::overlay::tint_area`), but on the tiles the player hasn't explored; in the terrain's z band,
//! just in front of the build area overlay ([`LINGERING_Z`]): the terrain in front hides it, units stand and walk on
//! it, and its footprints show where the fog is dark too (the player's own orders).

use std::collections::HashSet;

use bevy::prelude::*;
use bevy::sprite_render::AlphaMode2d;
use sc_sim::nav::TileRect;
use sc_sim::{CornerPos, Map, Placement, PlayerId, Sim, TilePos, VisibilityState, tunnel};

use super::overlay::{AREA_Z, tint_area};
use crate::ClientMode;
use crate::coords::DrawLayer;
use crate::fog::LocalSight;
use crate::map_render::{MapView, SURFACE_LIFT_Z, surface_vertex, tile_surface};
use crate::shapes::ColorMesh;
use crate::sim_driver::LocalPlayer;

/// A lingering ghost's tiles, and its outline (placeholder colours).
pub const LINGERING_FILL: Color = Color::srgba(0.55, 0.85, 1.0, 0.30);
pub const LINGERING_EDGE: Color = Color::srgba(0.75, 0.93, 1.0, 0.90);
/// A lingering Tunnel ghost's area, and its outline (placeholder colours).
pub const LINGERING_AREA_FILL: Color = Color::srgba(0.55, 0.85, 1.0, 0.10);
pub const LINGERING_AREA_EDGE: Color = Color::srgba(0.75, 0.93, 1.0, 0.55);
/// The outline's width, space units.
const EDGE_SU: f32 = 3.0;
/// The ghosts' z above the terrain entity's: in front of the build area overlay ([`AREA_Z`]), far behind the ground
/// objects' band.
pub const LINGERING_Z: f32 = AREA_Z + 0.01;

/// What the lingering ghosts show.
#[derive(Resource, Debug, Default)]
pub struct LingeringGhosts {
    /// The footprints shown, in `Sim::pending_builds` order; empty while none shows.
    pub shown: Vec<Placement>,
    /// The Tier 1 Tunnel Areas of the Tunnel ghosts among them, in the same order.
    pub areas: Vec<TileRect>,
    /// The tiles of those areas left out: those the local player hadn't explored.
    unexplored: Vec<TilePos>,
    /// The map meshes' build count (`MapView::builds`) when the mesh was built: a new map redraws it.
    map_builds: u32,
    /// How many times its mesh was built.
    pub builds: u32,
}

/// `player`'s lingering ghosts in `sim` (`Sim::pending_builds`): each site's footprint, and the Tier 1 Tunnel Area of
/// each Tunnel's (`sc_sim::tunnel::area::base_radius`, a type with a tier table).
pub fn lingering_sites(sim: &Sim, player: PlayerId) -> (Vec<Placement>, Vec<TileRect>) {
    let pending = sim.pending_builds(player);
    let areas = pending.iter().filter_map(|pending| {
        let site = pending.site;
        let radius = tunnel::area::base_radius(pending.kind)?;
        Some(tunnel::area::area(TileRect::new(site.origin, site.w, site.h), radius))
    });
    let areas = areas.collect();
    (pending.into_iter().map(|pending| pending.site).collect(), areas)
}

/// The lingering ghosts' mesh.
#[derive(Component)]
pub struct LingeringMesh;

#[derive(Resource)]
pub(super) struct LingeringAssets {
    mesh: Handle<Mesh>,
}

pub(super) fn spawn_lingering(
    mut commands: Commands,
    meshes: Res<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // The mesh asset arrives with the first ghost: the renderer warns about empty meshes.
    let mesh = meshes.reserve_handle();
    let material = ColorMaterial {
        color: Color::WHITE,
        alpha_mode: AlphaMode2d::Blend,
        ..default()
    };
    commands.spawn((
        Name::new("Lingering build ghosts"),
        LingeringMesh,
        Mesh2d(mesh.clone()),
        MeshMaterial2d(materials.add(material)),
        Transform::from_xyz(0.0, 0.0, DrawLayer::Terrain.base_z() + LINGERING_Z),
        Visibility::Hidden,
    ));
    commands.insert_resource(LingeringAssets { mesh });
}

/// Shows the local player's lingering ghosts in the game modes, redrawn when they, the unexplored tiles among their
/// areas or the map changed.
#[expect(clippy::too_many_arguments, reason = "a system's parameters")]
pub(super) fn draw_lingering(
    mode: Res<ClientMode>,
    local: LocalSight,
    player: Res<LocalPlayer>,
    view: Res<MapView>,
    assets: Res<LingeringAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut ghosts: ResMut<LingeringGhosts>,
    mut drawn: Query<&mut Visibility, With<LingeringMesh>>,
) {
    let (sim, sight) = (local.sim(), local.get());
    let (sites, areas) = if mode.is_game() {
        lingering_sites(sim, player.0)
    } else {
        (Vec::new(), Vec::new())
    };
    let unexplored: Vec<TilePos> = (areas.iter())
        .flat_map(|&area| tunnel::area::cells(area))
        .filter(|&t| sim.map().in_bounds(t) && sight.tile(t) == VisibilityState::Unexplored)
        .collect();
    let changed = ghosts.shown != sites || ghosts.areas != areas || ghosts.unexplored != unexplored;
    if (changed || ghosts.map_builds != view.builds) && !sites.is_empty() {
        let hidden: HashSet<TilePos> = unexplored.iter().copied().collect();
        let mesh = lingering_mesh(sim.map(), &sites, &areas, |t| !hidden.contains(&t));
        if let Err(error) = meshes.insert(&assets.mesh, mesh.into_mesh()) {
            error!("lingering ghosts mesh: {error}");
        }
        ghosts.map_builds = view.builds;
        ghosts.builds += 1;
    }
    if changed {
        ghosts.shown = sites;
        ghosts.areas = areas;
        ghosts.unexplored = unexplored;
    }
    let visibility = if ghosts.shown.is_empty() {
        Visibility::Hidden
    } else {
        Visibility::Inherited
    };
    for mut shown in &mut drawn {
        shown.set_if_neq(visibility);
    }
}

/// The ghosts' mesh: each Tunnel Area of `areas` tinted on `map`'s surface with its outline ([`tint_area`]), on its
/// tiles on the map that are `drawn`; then each footprint tile of `sites` tinted on the surface (a quad if it is flat,
/// else four triangles around its centre), and an outline around each footprint. Bevy world coordinates in the
/// terrain's z band, relative to the ghosts' entity.
pub fn lingering_mesh(
    map: &Map,
    sites: &[Placement],
    areas: &[TileRect],
    drawn: impl Fn(TilePos) -> bool,
) -> ColorMesh {
    let mut mesh = ColorMesh::triangles();
    for &area in areas {
        let on_map = |t: TilePos| map.in_bounds(t) && drawn(t);
        let colors = (LINGERING_AREA_FILL, LINGERING_AREA_EDGE);
        tint_area(&mut mesh, map, tunnel::area::cells(area), on_map, colors);
    }
    let vertex = |c: CornerPos| {
        let height = map.corner(c).map_or_else(|| map.height_at(c.pos()), f32::from);
        surface_vertex(c.pos(), height, SURFACE_LIFT_Z)
    };
    for site in sites {
        for t in site.tiles() {
            let surface = tile_surface(map, t);
            let [nw, ne, se, sw, center] =
                surface.map(|(ground, height)| surface_vertex(ground, height, SURFACE_LIFT_Z));
            if surface[..4].iter().all(|&(_, height)| height == surface[0].1) {
                mesh.quad([nw, ne, se, sw], LINGERING_FILL);
            } else {
                for (a, b) in [(nw, ne), (ne, se), (se, sw), (sw, nw)] {
                    mesh.triangle([a, b, center], LINGERING_FILL);
                }
            }
        }
        let (w, h) = (site.w as i32, site.h as i32);
        let TilePos { x, y } = site.origin;
        let corner = |dx: i32, dy: i32| vertex(CornerPos::new(x.saturating_add(dx), y.saturating_add(dy)));
        for dx in 0..w {
            edge(&mut mesh, corner(dx, 0), corner(dx + 1, 0));
            edge(&mut mesh, corner(dx, h), corner(dx + 1, h));
        }
        for dy in 0..h {
            edge(&mut mesh, corner(0, dy), corner(0, dy + 1));
            edge(&mut mesh, corner(w, dy), corner(w, dy + 1));
        }
    }
    mesh
}

/// A line from `a` to `b` [`EDGE_SU`] wide, each end at its own z (the terrain's depth grows southwards).
fn edge(mesh: &mut ColorMesh, a: Vec3, b: Vec3) {
    let along = (b - a).truncate();
    let side = along.normalize_or_zero().perp() * (EDGE_SU / 2.0);
    let side = side.extend(0.0);
    mesh.quad([a - side, b - side, b + side, a + side], LINGERING_EDGE);
}

#[cfg(test)]
mod tests {
    use sc_sim::deploy;

    use super::*;

    /// Each lingering ghost is its Beacon's 2 × 2 footprint: 4 tiles and 8 outline segments, a quad each on flat
    /// ground.
    #[test]
    fn a_ghost_is_its_beacons_footprint() {
        let map = Map::new(20, 20);
        let sites = [deploy::site(TilePos::new(3, 3)), deploy::site(TilePos::new(9, 4))];
        let all = |_| true;
        assert_eq!(lingering_mesh(&map, &sites[..1], &[], all).vertex_count(), (4 + 8) * 4);
        assert_eq!(lingering_mesh(&map, &sites, &[], all).vertex_count(), 2 * (4 + 8) * 4);
        assert_eq!(lingering_mesh(&map, &[], &[], all).vertex_count(), 0);
    }

    /// Slice M4-J1: a Tunnel ghost's Tier 1 area (8 × 8 around its 4 × 4 footprint) is drawn with it, its 64 tiles and
    /// its 32 outline segments, but on the tiles not drawn (unexplored) and off the map.
    #[test]
    fn a_tunnel_ghost_has_its_tier_1_area() {
        let map = Map::new(20, 20);
        let site = Placement {
            origin: TilePos::new(4, 4),
            w: 4,
            h: 4,
            ..deploy::site(TilePos::new(4, 4))
        };
        let area = tunnel::area::area(TileRect::new(site.origin, 4, 4), 2);
        assert_eq!(area, TileRect::new(TilePos::new(2, 2), 8, 8));
        let footprint = (16 + 16) * 4;
        let all = |_| true;
        assert_eq!(
            lingering_mesh(&map, &[site], &[area], all).vertex_count(),
            footprint + (64 + 32) * 4
        );
        // Its west column unexplored: 8 tiles fewer, and no outline along the tiles left out (22 segments).
        let west = |t: TilePos| t.x != 2;
        assert_eq!(
            lingering_mesh(&map, &[site], &[area], west).vertex_count(),
            footprint + (56 + 22) * 4
        );
    }
}
