//! Tints that lie on the terrain, in its z band so that the terrain in front hides them: the slope tint (K) and
//! the dimmed locked part of a symmetric map (`core/map_editor.md > Symmetry > Editing a symmetric map`). Both are
//! rebuilt when the document changes.

use bevy::prelude::*;
use sc_sim::{Map, MapDef, TilePos};

use super::OverlayAssets;
use crate::ClientMode;
use crate::editor::doc::EditorDoc;
use crate::editor::state::EditorUi;
use crate::map_render::{SURFACE_LIFT_Z, surface_vertex, tile_surface};
use crate::shapes::ColorMesh;

/// The slope tint (placeholder colours): flat, sloped within the maximum slope, too steep for the tile's preset.
const FLAT_TINT: Color = Color::srgba(0.25, 0.95, 0.35, 0.22);
const SLOPED_TINT: Color = Color::srgba(1.0, 0.72, 0.1, 0.32);
const STEEP_TINT: Color = Color::srgba(1.0, 0.1, 0.1, 0.5);
/// Dims the locked part of a symmetric map (placeholder).
pub const LOCKED_TINT: Color = Color::srgba(0.02, 0.03, 0.06, 0.5);

/// The slope tint's mesh (K).
#[derive(Component)]
pub struct SlopeTint;

/// The locked part's tint.
#[derive(Component)]
pub struct LockedTint;

/// Lays `color` on tile `t` of `map`'s surface: a quad if it is flat, else four triangles around its center.
fn tile_tint(mesh: &mut ColorMesh, map: &Map, t: TilePos, color: Color) {
    let [nw, ne, se, sw, center] =
        tile_surface(map, t).map(|(ground, height)| surface_vertex(ground, height, SURFACE_LIFT_Z));
    if map.is_flat(t) {
        mesh.quad([nw, ne, se, sw], color);
    } else {
        for (a, b) in [(nw, ne), (ne, se), (se, sw), (sw, nw)] {
            mesh.triangle([a, b, center], color);
        }
    }
}

fn tiles(map: &Map) -> impl Iterator<Item = TilePos> + '_ {
    (0..map.height() as i32).flat_map(move |y| (0..map.width() as i32).map(move |x| TilePos::new(x, y)))
}

/// The slope tint: each tile green if flat, amber if sloped within the maximum slope, red if too steep for its
/// preset (`core/map.md > Elevation`), lying on the surface in the terrain's z band.
pub fn slope_mesh(map: &Map) -> ColorMesh {
    let mut mesh = ColorMesh::triangles();
    for t in tiles(map) {
        let color = if map.is_too_steep(t) {
            STEEP_TINT
        } else if map.is_flat(t) {
            FLAT_TINT
        } else {
            SLOPED_TINT
        };
        tile_tint(&mut mesh, map, t, color);
    }
    mesh
}

/// The locked part's tint: every tile outside the editable part ([`MapDef::region`]), dimmed. Empty without a
/// symmetry.
pub fn locked_mesh(map: &MapDef) -> ColorMesh {
    let mut mesh = ColorMesh::triangles();
    let region = map.region();
    if region.is_everything() {
        return mesh;
    }
    for t in tiles(&map.terrain).filter(|t| !region.tile(*t)) {
        tile_tint(&mut mesh, &map.terrain, t, LOCKED_TINT);
    }
    mesh
}

/// Shows the slope tint while it is on (K), rebuilt when the document changed since it was built.
pub(super) fn draw_slopes(
    mode: Res<ClientMode>,
    doc: Res<EditorDoc>,
    ui: Res<EditorUi>,
    assets: Res<OverlayAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut built: Local<Option<u64>>,
    mut tint: Query<&mut Visibility, With<SlopeTint>>,
) {
    let visible = *mode == ClientMode::Editor && ui.slopes;
    if visible && *built != Some(doc.revision()) {
        let mesh = slope_mesh(&doc.map().terrain);
        if mesh.vertex_count() > 0
            && let Err(error) = meshes.insert(&assets.slope_mesh, mesh.into_mesh())
        {
            error!("editor slope tint: {error}");
        }
        *built = Some(doc.revision());
    }
    for mut visibility in &mut tint {
        visibility.set_if_neq(if visible {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
}

/// Dims the locked part of a symmetric map in the editor, rebuilt when the document changed since it was built.
pub(super) fn draw_locked(
    mode: Res<ClientMode>,
    doc: Res<EditorDoc>,
    assets: Res<OverlayAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut built: Local<Option<(u64, bool)>>,
    mut tint: Query<&mut Visibility, With<LockedTint>>,
) {
    let editing = *mode == ClientMode::Editor;
    if editing && built.is_none_or(|(revision, _)| revision != doc.revision()) {
        let mesh = locked_mesh(doc.map());
        let has_tiles = mesh.vertex_count() > 0;
        if has_tiles && let Err(error) = meshes.insert(&assets.locked_mesh, mesh.into_mesh()) {
            error!("editor locked tint: {error}");
        }
        *built = Some((doc.revision(), has_tiles));
    }
    let visible = editing && built.is_some_and(|(_, has_tiles)| has_tiles);
    for mut visibility in &mut tint {
        visibility.set_if_neq(if visible {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sc_sim::{MapSettings, Symmetry};

    #[test]
    fn the_slope_tint_has_a_quad_per_flat_tile_and_four_triangles_per_sloped_one() {
        let map = Map::from_ascii("...\n\n0011\n0011").unwrap();
        // Tiles (0, 0) and (2, 0) are flat, (1, 0) slopes.
        assert_eq!(slope_mesh(&map).vertex_count(), 4 + 12 + 4);
    }

    #[test]
    fn the_locked_tint_covers_the_locked_part() {
        let settings = |symmetry, width, height| MapSettings {
            symmetry,
            width,
            height,
            slots: 4,
            map_type: sc_sim::MapType::Custom,
            ..MapSettings::default()
        };
        // A flat map: a quad per locked tile.
        let half = settings(Symmetry::Rotate180, 16, 16).create().unwrap();
        assert_eq!(locked_mesh(&half).vertex_count(), 4 * 8 * 16);
        let quarter = settings(Symmetry::MirrorBothAxes, 16, 16).create().unwrap();
        assert_eq!(locked_mesh(&quarter).vertex_count(), 4 * (16 * 16 - 8 * 8));
        let free = settings(Symmetry::None, 16, 16).create().unwrap();
        assert_eq!(locked_mesh(&free).vertex_count(), 0);
    }
}
