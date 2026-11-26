//! Shroud's clouds in the client (slice M5-N4b; `factions/cults/units/disciple.md > Spells > Shroud`: "the cloud is
//! shown to **every player**, over the tiles that are Visible to them, so its edge can be seen even when the Disciple
//! can't"). Display only: the sim keeps the clouds (`Sim::clouds`), whatever the client shows.
//!
//! - **The cloud** is a pale haze over its circles, merged where they overlap, in one translucent layer over the
//!   ground units and under the air units, which no cloud covers (`sc_sim::shroud`: Shroud protects the Ground
//!   domain), drawn only over the tiles Visible to the local player ([`texels`]): an enemy who sees one tile of a
//!   cloud's edge sees that tile's part of it. Without the fog every tile is Visible.
//! - **A release** (`SimEvent::ShroudReleased`, which every player perceives: `crate::viewer_events`) flashes over the
//!   circles it went over, as they were (a join's or a split's old cloud too), for [`RELEASE_TICKS`], fading, over
//!   the same Visible tiles.
//!
//! One mesh and one texture, over the box of tiles the clouds and the flashes cover ([`Area`]), never an entity per
//! cloud or per tile (`ARCHITECTURE.md > Client architecture > Visuals`): the mesh follows the terrain's surface tile
//! by tile, as the fog layer's does (`crate::fog::fog_mesh`), and is rebuilt when the box changes; the texture,
//! [`TEXELS_PER_TILE`] texels to a tile's edge, sampled linearly so the edges soften, is written when what it shows
//! changes: the clouds, the flashes, the local player's Visible tiles. With no cloud and no flash the layer is hidden
//! and nothing is computed.
//!
//! Placeholders (Q0001, the design being silent on the look): the haze's colour and opacity, the flash's, its length.

use bevy::asset::RenderAssetUsages;
use bevy::image::ImageSampler;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::sprite_render::AlphaMode2d;
use sc_sim::{Map, PlayerId, SU_PER_GU, SimEvent, Tick, TilePos, VisibilityState};

use crate::coords::{DrawLayer, drawn_at_height, view_to_world};
use crate::fog::{LocalSight, Revision};
use crate::map_render::{MapView, tile_surface};
use crate::sim_driver::SimRes;
use crate::viewer_events::{FilterEvents, ViewerEvents};
use crate::{ClientMode, ClientSystems};

#[cfg(test)]
mod tests;

/// Texels to a tile's edge: 16 space units each, so that a circle of 3 grid units is 12 texels across its radius.
pub const TEXELS_PER_TILE: i32 = 4;
/// A release flashes for this many ticks, fading (placeholder).
pub const RELEASE_TICKS: Tick = 6;
/// The haze: a pale violet, under half opaque, so that the units under it show (placeholder, Q0001).
const CLOUD: [u8; 4] = [190, 178, 226, 100];
/// A release's flash: near white, up to this opacity as it starts (placeholder, Q0001).
const FLASH: [u8; 3] = [255, 246, 255];
const FLASH_ALPHA: f32 = 210.0;
/// How far under the air units' z band the layer lies: over every ground unit's z (`DrawLayer::GroundUnits`, its
/// depth offsets under 66 for any map), under the air units'.
const UNDER_AIR_Z: f32 = 20.0;

pub struct ShroudViewPlugin;

impl Plugin for ShroudViewPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ShroudView>()
            .add_systems(Startup, spawn_shroud_layer)
            .add_systems(
                Update,
                collect_releases.in_set(ClientSystems::SyncSim).after(FilterEvents),
            )
            .add_systems(Update, update_shroud_layer.in_set(ClientSystems::Present));
    }
}

/// The layer's handles: one mesh, rebuilt in place; one texture, written in place while the box keeps its size, and
/// replaced by a new one when the size changes, which the material is pointed at: a material keeps the texture it
/// was prepared with, and a texture of another size is a new one on the GPU.
#[derive(Resource)]
pub struct ShroudAssets {
    pub mesh: Handle<Mesh>,
    pub texture: Handle<Image>,
    pub material: Handle<ColorMaterial>,
}

/// The layer's entity.
#[derive(Component)]
pub struct ShroudLayer;

/// A box of tiles: its north-west tile and its size.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Area {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Area {
    /// The box of `map`'s tiles that `circles` touch; none for no circle.
    pub fn of<'a>(map: &Map, circles: impl IntoIterator<Item = &'a (Vec2, f32)>) -> Option<Area> {
        let (mut lo, mut hi) = (IVec2::MAX, IVec2::MIN);
        for &(at, radius) in circles {
            lo = lo.min(((at - radius) / SU_PER_GU).floor().as_ivec2());
            hi = hi.max(((at + radius) / SU_PER_GU).floor().as_ivec2());
        }
        let last = IVec2::new(map.width() as i32, map.height() as i32) - 1;
        let (lo, hi) = (lo.max(IVec2::ZERO), hi.min(last));
        (lo.x <= hi.x && lo.y <= hi.y).then(|| Area {
            x: lo.x,
            y: lo.y,
            w: hi.x - lo.x + 1,
            h: hi.y - lo.y + 1,
        })
    }

    /// Its north-west corner and its size, space units.
    fn bounds(self) -> (Vec2, Vec2) {
        let corner = Vec2::new(self.x as f32, self.y as f32) * SU_PER_GU;
        (corner, Vec2::new(self.w as f32, self.h as f32) * SU_PER_GU)
    }

    /// Its tiles, row by row from the north-west.
    fn tiles(self) -> impl Iterator<Item = TilePos> {
        (self.y..self.y + self.h).flat_map(move |y| (self.x..self.x + self.w).map(move |x| TilePos::new(x, y)))
    }
}

/// A release still flashing: its tick and the circles it went over.
#[derive(Clone, Debug, PartialEq)]
struct Release {
    tick: Tick,
    circles: Vec<(Vec2, f32)>,
}

/// What the layer shows, for tests and QA as well.
#[derive(Resource, Default, Debug)]
pub struct ShroudView {
    /// The releases still flashing, and the `SimRes::generation` they belong to.
    releases: Vec<Release>,
    generation: Option<u32>,
    /// The box the mesh covers, and the `MapView::builds` it belongs to.
    area: Option<(Area, u32)>,
    /// What the texture shows.
    shown: Option<Key>,
    /// The tiles where the texture shows any cloud or flash, as last written: none while the layer is hidden.
    pub tiles: Vec<TilePos>,
    pub mesh_builds: u32,
    pub texture_writes: u32,
}

/// Everything the texture's texels depend on.
#[derive(Clone, Debug, PartialEq)]
struct Key {
    generation: u32,
    player: PlayerId,
    fogged: bool,
    revision: Option<Revision>,
    map_build: u32,
    clouds: Vec<Vec<(Vec2, f32)>>,
    flashes: Vec<(Vec<(Vec2, f32)>, u8)>,
}

/// The flash's opacity `age` ticks after its release: [`FLASH_ALPHA`] in its tick, fading to nothing over
/// [`RELEASE_TICKS`].
fn flash_alpha(age: Tick) -> u8 {
    let left = RELEASE_TICKS.saturating_sub(age) as f32 / RELEASE_TICKS as f32;
    (FLASH_ALPHA * left).round() as u8
}

/// How much of the texel `step` space units wide at `point` the circles cover, 0 to 1: its centre's distance inside
/// the edge of the circle it is deepest in, over a texel, so that the edges are smooth however coarse the texels
/// (half covered on the edge, wholly half a texel inside it).
fn coverage(circles: &[(Vec2, f32)], point: Vec2, step: f32) -> f32 {
    (circles.iter())
        .map(|&(at, radius)| ((radius - at.distance(point)) / step + 0.5).clamp(0.0, 1.0))
        .fold(0.0, f32::max)
}

/// The texels of `area` (row by row from its north-west corner, RGBA8, [`TEXELS_PER_TILE`] to a tile's edge, each
/// sampled at its centre), and the tiles where any shows: on a tile `visible` says is Visible, the haze where the
/// circles of `clouds` cover a texel, a flash where those of one of `flashes` do (with its opacity), each as opaque
/// as it covers the texel ([`coverage`]), the more opaque of the two; elsewhere nothing.
pub fn texels(
    area: Area,
    clouds: &[Vec<(Vec2, f32)>],
    flashes: &[(Vec<(Vec2, f32)>, u8)],
    visible: impl Fn(TilePos) -> bool,
) -> (Vec<u8>, Vec<TilePos>) {
    let (corner, _) = area.bounds();
    let step = SU_PER_GU / TEXELS_PER_TILE as f32;
    let (w, h) = (area.w * TEXELS_PER_TILE, area.h * TEXELS_PER_TILE);
    let mut data = vec![0; (w * h * 4) as usize];
    let mut shown = Vec::new();
    for tile in area.tiles() {
        if !visible(tile) {
            continue;
        }
        let mut any = false;
        let (tx, ty) = ((tile.x - area.x) * TEXELS_PER_TILE, (tile.y - area.y) * TEXELS_PER_TILE);
        for j in ty..ty + TEXELS_PER_TILE {
            for i in tx..tx + TEXELS_PER_TILE {
                let point = corner + (Vec2::new(i as f32, j as f32) + 0.5) * step;
                let opacity =
                    |circles: &[(Vec2, f32)], alpha: u8| (alpha as f32 * coverage(circles, point, step)).round() as u8;
                let haze = clouds
                    .iter()
                    .map(|circles| opacity(circles, CLOUD[3]))
                    .max()
                    .unwrap_or(0);
                let flash = (flashes.iter())
                    .map(|(circles, alpha)| opacity(circles, *alpha))
                    .max()
                    .unwrap_or(0);
                let texel = if flash > haze {
                    [FLASH[0], FLASH[1], FLASH[2], flash]
                } else if haze > 0 {
                    [CLOUD[0], CLOUD[1], CLOUD[2], haze]
                } else {
                    continue;
                };
                let at = ((j * w + i) * 4) as usize;
                data[at..at + 4].copy_from_slice(&texel);
                any = true;
            }
        }
        if any {
            shown.push(tile);
        }
    }
    (data, shown)
}

/// The layer's mesh over `area`: each tile's surface as the terrain mesh draws it ([`tile_surface`]), flat in z (the
/// layer's z is its entity's), with texture coordinates from the ground position across the box.
fn area_mesh(map: &Map, area: Area) -> Mesh {
    let (corner, size) = area.bounds();
    let (mut positions, mut uvs, mut indices) = (Vec::new(), Vec::new(), Vec::<u32>::new());
    for tile in area.tiles() {
        let [nw, ne, se, sw, centre] = tile_surface(map, tile);
        let flat = [ne, se, sw].iter().all(|c| c.1 == nw.1);
        let first = positions.len() as u32;
        let points: &[(Vec2, f32)] = if flat {
            &[nw, ne, se, sw]
        } else {
            &[nw, ne, se, sw, centre]
        };
        for &(ground, height) in points {
            positions.push(view_to_world(drawn_at_height(ground, height)).extend(0.0).to_array());
            uvs.push(((ground - corner) / size).to_array());
        }
        if flat {
            indices.extend([first, first + 1, first + 2, first, first + 2, first + 3]);
        } else {
            for i in 0..4 {
                indices.extend([first + i, first + (i + 1) % 4, first + 4]);
            }
        }
    }
    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_indices(Indices::U32(indices))
}

/// A texture of `data` for `area`, sampled linearly.
fn area_image(data: Vec<u8>, area: Area) -> Image {
    let size = Extent3d {
        width: (area.w * TEXELS_PER_TILE) as u32,
        height: (area.h * TEXELS_PER_TILE) as u32,
        depth_or_array_layers: 1,
    };
    let mut image = Image::new(
        size,
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::linear();
    image
}

/// A zero-area stand-in, until the first cloud (a mesh without vertices makes the renderer warn).
fn placeholder_mesh() -> Mesh {
    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vec![[0.0_f32; 3]; 3])
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0_f32; 2]; 3])
        .with_inserted_indices(Indices::U32(vec![0, 1, 2]))
}

/// The layer, hidden, until the first cloud.
fn spawn_shroud_layer(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let one = Area { x: 0, y: 0, w: 1, h: 1 };
    let mesh = meshes.add(placeholder_mesh());
    let texture = images.add(area_image(
        vec![0; (TEXELS_PER_TILE * TEXELS_PER_TILE * 4) as usize],
        one,
    ));
    let material = materials.add(ColorMaterial {
        color: Color::WHITE,
        alpha_mode: AlphaMode2d::Blend,
        texture: Some(texture.clone()),
        ..default()
    });
    commands.spawn((
        Name::new("Shroud clouds"),
        ShroudLayer,
        Mesh2d(mesh.clone()),
        MeshMaterial2d(material.clone()),
        Transform::from_xyz(0.0, 0.0, DrawLayer::Air.base_z() - UNDER_AIR_Z),
        Visibility::Hidden,
    ));
    commands.insert_resource(ShroudAssets {
        mesh,
        texture,
        material,
    });
}

/// Keeps this frame's releases (`SimEvent::ShroudReleased`, which every player perceives) while they flash, and
/// forgets them with the sim they belong to.
fn collect_releases(sim: Res<SimRes>, sight: LocalSight, events: Res<ViewerEvents>, mut view: ResMut<ShroudView>) {
    if view.generation != Some(sim.generation()) {
        view.releases.clear();
        view.generation = Some(sim.generation());
    }
    for (tick, event, _) in events.perceived_by(&sim, sight.get()) {
        if let SimEvent::ShroudReleased { circles, .. } = event {
            view.releases.push(Release {
                tick,
                circles: circles.clone(),
            });
        }
    }
    let now = sim.tick();
    view.releases.retain(|r| r.tick <= now && now - r.tick < RELEASE_TICKS);
}

/// Hides the layer while there is nothing to show; else rebuilds its mesh when the box of the clouds and the flashes
/// changed, writes its texture when what it shows changed, and shows it, in the game and its play-tests (the editor
/// shows no cloud).
#[expect(clippy::too_many_arguments, reason = "a system's parameters")]
fn update_shroud_layer(
    sim_res: Res<SimRes>,
    sight: LocalSight,
    mode: Res<ClientMode>,
    map_view: Res<MapView>,
    assets: Option<ResMut<ShroudAssets>>,
    mut view: ResMut<ShroudView>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut layer: Query<&mut Visibility, With<ShroudLayer>>,
) {
    let Some(mut assets) = assets else {
        return;
    };
    let sim = sight.sim();
    let clouds: Vec<Vec<(Vec2, f32)>> = sim.clouds().list().iter().map(|c| c.circles()).collect();
    let now = sim.tick();
    let flashes: Vec<(Vec<(Vec2, f32)>, u8)> = (view.releases.iter())
        .map(|r| (r.circles.clone(), flash_alpha(now.saturating_sub(r.tick))))
        .filter(|&(_, alpha)| alpha > 0)
        .collect();
    let circles = clouds.iter().flatten().chain(flashes.iter().flat_map(|f| &f.0));
    let area = mode.is_game().then(|| Area::of(sim.map(), circles)).flatten();
    let shows = area.is_some();
    for mut visibility in &mut layer {
        visibility.set_if_neq(if shows {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
    let Some(area) = area else {
        if view.shown.take().is_some() {
            view.tiles.clear();
        }
        return;
    };
    let seen = sight.get();
    let key = Key {
        generation: sim_res.generation(),
        player: seen.player(),
        fogged: seen.is_fogged(),
        revision: seen.revision(),
        map_build: map_view.builds,
        clouds,
        flashes,
    };
    if view.shown.as_ref() == Some(&key) && view.area == Some((area, map_view.builds)) {
        return;
    }
    if view.area != Some((area, map_view.builds)) {
        if let Err(error) = meshes.insert(&assets.mesh, area_mesh(sim.map(), area)) {
            error!("shroud mesh rebuild failed: {error}");
        }
        view.area = Some((area, map_view.builds));
        view.mesh_builds += 1;
    }
    let visible = |tile: TilePos| !seen.is_fogged() || seen.tile(tile) == VisibilityState::Visible;
    let (data, tiles) = texels(area, &key.clouds, &key.flashes, visible);
    let size = UVec2::new(area.w as u32, area.h as u32) * TEXELS_PER_TILE as u32;
    let same_size = images.get(&assets.texture).is_some_and(|image| image.size() == size);
    if same_size {
        if let Some(mut image) = images.get_mut(&assets.texture) {
            image.data = Some(data);
        }
    } else {
        let texture = images.add(area_image(data, area));
        if let Some(mut material) = materials.get_mut(&assets.material) {
            material.texture = Some(texture.clone());
        }
        // The old texture goes with its last handle.
        assets.texture = texture;
    }
    view.tiles = tiles;
    view.shown = Some(key);
    view.texture_writes += 1;
}
