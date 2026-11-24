//! The lingering build ghosts (`core/entities.md > Placement Validation > Lingering build ghosts`, design change
//! 0125): "Every ordered worker build (Drone Deploy, Agent build, Cults Recruits' Construct), including every
//! Shift-queued one, leaves a ghost of the building on its site until construction actually starts", "Visible to its
//! owner. Enemies never see it". The local player's pending Deploys (`Sim::pending_deploys`), each the Beacon's
//! footprint on its site, in the game modes. A ghost goes with its order: when the deployment starts (the Drone is a
//! Beacon then), or the order is cancelled, replaced by a new one, or fails. They block no one (the sim's rule:
//! `crates/sim/tests/deploy_gate.rs`, `pending_sites_block_no_one`) and reserve their sites against the player's
//! further placements, which the placement ghost shows red there (`super::ghost`).
//!
//! One mesh, under one handle, rebuilt in place only when the sites or the map change. The look, placeholders until
//! the art (Q0001): each footprint tile tinted pale blue on the terrain's surface, as the terrain mesh draws it, and
//! an outline around each footprint; in the terrain's z band, just in front of the build area overlay
//! ([`LINGERING_Z`]): the terrain in front hides it, units stand and walk on it, and it shows where the fog is dark
//! too (the player's own orders).

use bevy::prelude::*;
use bevy::sprite_render::AlphaMode2d;
use sc_sim::{CornerPos, Map, Placement, TilePos};

use super::overlay::AREA_Z;
use crate::ClientMode;
use crate::coords::DrawLayer;
use crate::map_render::{MapView, SURFACE_LIFT_Z, surface_vertex, tile_surface};
use crate::shapes::ColorMesh;
use crate::sim_driver::{LocalPlayer, SimRes};

/// A lingering ghost's tiles, and its outline (placeholder colours).
pub const LINGERING_FILL: Color = Color::srgba(0.55, 0.85, 1.0, 0.30);
pub const LINGERING_EDGE: Color = Color::srgba(0.75, 0.93, 1.0, 0.90);
/// The outline's width, space units.
const EDGE_SU: f32 = 3.0;
/// The ghosts' z above the terrain entity's: in front of the build area overlay ([`AREA_Z`]), far behind the ground
/// objects' band.
pub const LINGERING_Z: f32 = AREA_Z + 0.01;

/// What the lingering ghosts show.
#[derive(Resource, Debug, Default)]
pub struct LingeringGhosts {
    /// The footprints shown, in `Sim::pending_deploys` order; empty while none shows.
    pub shown: Vec<Placement>,
    /// The map meshes' build count (`MapView::builds`) when the mesh was built: a new map redraws it.
    map_builds: u32,
    /// How many times its mesh was built.
    pub builds: u32,
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

/// Shows the local player's lingering ghosts in the game modes, redrawn when they or the map changed.
#[expect(clippy::too_many_arguments, reason = "a system's parameters")]
pub(super) fn draw_lingering(
    mode: Res<ClientMode>,
    sim: Res<SimRes>,
    player: Res<LocalPlayer>,
    view: Res<MapView>,
    assets: Res<LingeringAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut ghosts: ResMut<LingeringGhosts>,
    mut drawn: Query<&mut Visibility, With<LingeringMesh>>,
) {
    let sites: Vec<Placement> = if mode.is_game() {
        sim.pending_deploys(player.0)
            .into_iter()
            .map(|(_, site)| site)
            .collect()
    } else {
        Vec::new()
    };
    if (ghosts.shown != sites || ghosts.map_builds != view.builds) && !sites.is_empty() {
        if let Err(error) = meshes.insert(&assets.mesh, lingering_mesh(sim.map(), &sites).into_mesh()) {
            error!("lingering ghosts mesh: {error}");
        }
        ghosts.map_builds = view.builds;
        ghosts.builds += 1;
    }
    if ghosts.shown != sites {
        ghosts.shown = sites;
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

/// The ghosts' mesh: each footprint tile of `sites` tinted on `map`'s surface (a quad if it is flat, else four
/// triangles around its centre), and an outline around each footprint. Bevy world coordinates in the terrain's z
/// band, relative to the ghosts' entity.
pub fn lingering_mesh(map: &Map, sites: &[Placement]) -> ColorMesh {
    let mut mesh = ColorMesh::triangles();
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
        assert_eq!(lingering_mesh(&map, &sites[..1]).vertex_count(), (4 + 8) * 4);
        assert_eq!(lingering_mesh(&map, &sites).vertex_count(), 2 * (4 + 8) * 4);
        assert_eq!(lingering_mesh(&map, &[]).vertex_count(), 0);
    }
}
