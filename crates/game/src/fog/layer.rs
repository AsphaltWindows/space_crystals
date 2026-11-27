//! The fog layer (`core/vision.md`): one mesh over the whole map and one texture of one texel per tile, its mesh
//! rebuilt with the map and its texture written only when what it shows changes, a new texture only when the map's
//! size does; and how each tile's state looks (placeholders, the design being silent: the Explored dimming and the
//! soft edges).

use bevy::asset::RenderAssetUsages;
use bevy::image::ImageSampler;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::sprite_render::AlphaMode2d;
use sc_sim::{Map, PlayerId, SU_PER_GU, TilePos, VisibilityState};

use super::{LocalSight, Revision, Sight};
use crate::coords::DrawLayer;
use crate::map_render::{MapView, SURFACE_LIFT_Z, surface_vertex, tile_surface};

/// `core/vision.md > Unexplored`: "fully black".
pub const UNEXPLORED_ALPHA: f32 = 1.0;
/// `core/vision.md > Explored`: the terrain shows, dimmed this much towards black (blending in linear colour, 0.7
/// leaves about 58 % of the sRGB brightness). Placeholder: the design doesn't say how Explored looks.
pub const EXPLORED_ALPHA: f32 = 0.7;
/// Fog edges fade across a tile: the texture is sampled linearly between tile centres. `false` would step at the
/// tile edges. Placeholder: the design doesn't say.
pub const FOG_SOFT_EDGES: bool = true;
/// The fog layer's z above the terrain entity's, on top of the [`SURFACE_LIFT_Z`] its vertices have: 60 su of
/// ground y (`coords::depth_offset`). It sorts after the grid overlay, so it darkens the grid too, and lies in front
/// of every cliff stratum, which stands out of its face by up to about 50 su of ground y where a 4-level rise
/// flattens the face to a line on screen: no terrain detail shows through the fog. The cost: where a north-facing
/// rise of 5 levels or more folds the terrain over itself, the fog of the hidden face also darkens the strip in
/// front of it.
pub const FOG_Z: f32 = 0.06;

/// The fog layer's handles: one mesh, rebuilt in place; one texture, written in place while the map keeps its size,
/// and replaced by a new one when the size changes, which the material is pointed at (slice M5-Z3): a 2D material
/// keeps the GPU texture it was prepared with, and an image of another size is a new texture on the GPU
/// (`ARCHITECTURE.md > Bevy 0.20: verify, don't recall`). The size changes from the 1 × 1 stand-in at the first
/// frame the fog applies after one it didn't (an editor play-test, F3 after `--no-fog`) and with the map between
/// play-tests.
#[derive(Resource)]
pub struct FogAssets {
    pub mesh: Handle<Mesh>,
    pub texture: Handle<Image>,
    pub material: Handle<ColorMaterial>,
}

/// The fog layer's entity.
#[derive(Component)]
pub struct FogLayer;

/// What the fog layer shows, and how often it was rebuilt (for tests and QA).
#[derive(Resource, Default, Debug)]
pub struct FogView {
    /// The `MapView::builds` the mesh and the texture's size belong to.
    map_build: Option<u32>,
    /// What the texture shows.
    shown: Option<FogKey>,
    pub mesh_builds: u32,
    pub texture_writes: u32,
}

/// Everything the texture's texels depend on.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct FogKey {
    generation: u32,
    player: PlayerId,
    /// The sight's [`Sight::revision`]: whom it watches, and their tiles.
    revision: Option<Revision>,
    map_build: Option<u32>,
}

/// The alpha of a tile's texel.
pub fn fog_alpha(state: VisibilityState) -> f32 {
    match state {
        VisibilityState::Unexplored => UNEXPLORED_ALPHA,
        VisibilityState::Explored => EXPLORED_ALPHA,
        VisibilityState::Visible => 0.0,
    }
}

/// The texels of a `width` × `height` map as `sight` sees it (row by row from the north-west corner, RGBA8): black,
/// with each tile's [`fog_alpha`], its best state among the visions the sight watches ([`Sight::tile_states`]).
/// Watching none, every tile is Unexplored; without the fog, every tile is Visible.
pub fn fog_texels(sight: &Sight, width: u32, height: u32) -> Vec<u8> {
    let texel = |state| [0, 0, 0, (fog_alpha(state) * 255.0).round() as u8];
    sight.tile_states(width, height).into_iter().flat_map(texel).collect()
}

/// A texture of `texels` for a `width` × `height` map ([`FOG_SOFT_EDGES`]).
fn fog_image(texels: Vec<u8>, width: u32, height: u32) -> Image {
    let size = Extent3d {
        width: width.max(1),
        height: height.max(1),
        depth_or_array_layers: 1,
    };
    let mut image = Image::new(
        size,
        TextureDimension::D2,
        texels,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.sampler = if FOG_SOFT_EDGES {
        ImageSampler::linear()
    } else {
        ImageSampler::nearest()
    };
    image
}

/// The fog layer's mesh: every tile's surface as the terrain mesh draws it ([`tile_surface`]), lifted by
/// [`SURFACE_LIFT_Z`], with texture coordinates from the ground position, so each tile's texel lies on its tile.
pub fn fog_mesh(map: &Map) -> Mesh {
    let size = Vec2::new(map.width() as f32, map.height() as f32) * SU_PER_GU;
    let (mut positions, mut uvs, mut indices) = (Vec::new(), Vec::new(), Vec::<u32>::new());
    for y in 0..map.height() as i32 {
        for x in 0..map.width() as i32 {
            let [nw, ne, se, sw, centre] = tile_surface(map, TilePos::new(x, y));
            let flat = [ne, se, sw].iter().all(|c| c.1 == nw.1);
            let first = positions.len() as u32;
            let points: &[(Vec2, f32)] = if flat {
                &[nw, ne, se, sw]
            } else {
                &[nw, ne, se, sw, centre]
            };
            for &(ground, height) in points {
                positions.push(surface_vertex(ground, height, SURFACE_LIFT_Z).to_array());
                uvs.push((ground / size).to_array());
            }
            if flat {
                indices.extend([first, first + 1, first + 2, first, first + 2, first + 3]);
            } else {
                for i in 0..4 {
                    indices.extend([first + i, first + (i + 1) % 4, first + 4]);
                }
            }
        }
    }
    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_indices(Indices::U32(indices))
}

/// A zero-area stand-in, until the first frame builds the map's mesh (a mesh without vertices makes the renderer
/// warn).
fn placeholder_mesh() -> Mesh {
    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vec![[0.0_f32; 3]; 3])
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0_f32; 2]; 3])
        .with_inserted_indices(Indices::U32(vec![0, 1, 2]))
}

/// The fog layer, hidden: [`update_fog_layer`] builds its mesh and texture in the first frame.
pub(super) fn spawn_fog(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mesh = meshes.add(placeholder_mesh());
    let unexplored = [0, 0, 0, (fog_alpha(VisibilityState::Unexplored) * 255.0).round() as u8].to_vec();
    let texture = images.add(fog_image(unexplored, 1, 1));
    let material = materials.add(ColorMaterial {
        color: Color::WHITE,
        alpha_mode: AlphaMode2d::Blend,
        texture: Some(texture.clone()),
        ..default()
    });
    commands.spawn((
        Name::new("Fog of war"),
        FogLayer,
        Mesh2d(mesh.clone()),
        MeshMaterial2d(material.clone()),
        Transform::from_xyz(0.0, 0.0, DrawLayer::Terrain.base_z() + FOG_Z),
        Visibility::Hidden,
    ));
    commands.insert_resource(FogAssets {
        mesh,
        texture,
        material,
    });
}

/// Builds the mesh when the map changed (in the first frame too: `MapView::builds` counts the terrain's builds),
/// writes the texture when what it shows changed, in place while the map keeps its size, else as a new texture that
/// the material is pointed at ([`FogAssets`]), and shows the layer while the fog applies.
#[expect(clippy::too_many_arguments, reason = "a system's parameters")]
pub(super) fn update_fog_layer(
    sight: LocalSight,
    map_view: Res<MapView>,
    assets: Option<ResMut<FogAssets>>,
    mut view: ResMut<FogView>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut layer: Query<&mut Visibility, With<FogLayer>>,
) {
    let Some(mut assets) = assets else {
        return;
    };
    let seen = sight.get();
    let fogged = seen.is_fogged();
    for mut visibility in &mut layer {
        visibility.set_if_neq(if fogged {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
    let sim = &sight.sim;
    let map = sim.map();
    let map_build = Some(map_view.builds);
    if view.map_build != map_build {
        if let Err(error) = meshes.insert(&assets.mesh, fog_mesh(map)) {
            error!("fog mesh rebuild failed: {error}");
        }
        view.map_build = map_build;
        view.mesh_builds += 1;
    }
    if !fogged {
        return;
    }
    let key = FogKey {
        generation: sim.generation(),
        player: seen.player(),
        revision: seen.revision(),
        map_build,
    };
    if view.shown == Some(key) {
        return;
    }
    let texels = fog_texels(&seen, map.width(), map.height());
    let size = UVec2::new(map.width(), map.height()).max(UVec2::ONE);
    let same_size = images.get(&assets.texture).is_some_and(|image| image.size() == size);
    if same_size {
        if let Some(mut image) = images.get_mut(&assets.texture) {
            image.data = Some(texels);
        }
    } else {
        let texture = images.add(fog_image(texels, map.width(), map.height()));
        if let Some(mut material) = materials.get_mut(&assets.material) {
            material.texture = Some(texture.clone());
        }
        // The old texture goes with its last handle.
        assets.texture = texture;
    }
    view.shown = Some(key);
    view.texture_writes += 1;
}
