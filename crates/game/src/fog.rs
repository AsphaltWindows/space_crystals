//! Fog of war (`core/vision.md`): the local player's view of the map through its vision at the end of the latest
//! tick (`Sim::vision`). Display only: the sim computes every player's vision whatever the client shows.
//!
//! - **What shows** ([`Sight`]): with the fog on, the local player sees its own objects and everything on its
//!   Visible tiles in real time (`> Visible`); on Explored tiles the terrain, and the structures and resources it
//!   remembers in their last-known state, never the live objects there (`> Explored`: the sim's records,
//!   `PlayerVision::remembered`); on Unexplored tiles nothing (`> Unexplored`). The object visuals
//!   ([`hide_unseen_objects`], `object_render::remembered`), the combat effects, the path overlay, the command
//!   indicators, the InfoPanel and the selection (`selection::pick`) all ask it.
//! - **The fog layer**: one mesh over the whole map and one texture of one texel per tile, never an entity or a
//!   material per tile (`ARCHITECTURE.md > Client architecture > Visuals`). The mesh follows the terrain's surface
//!   tile by tile as the terrain mesh draws it (a quad per flat tile, four triangles around the centre of a sloped
//!   one), in front of it in the terrain's z band ([`FOG_Z`]): objects stand on it. Each texel is black, with [`UNEXPLORED_ALPHA`] for an Unexplored tile, [`EXPLORED_ALPHA`] for an
//!   Explored one and none for a Visible one, sampled linearly so the edges soften across a tile
//!   ([`FOG_SOFT_EDGES`]).
//!   The texture is written only when what it shows changes: the local player's tiles (`PlayerVision::revision`),
//!   the local player, or the sim (`SimRes::generation`); the mesh and the texture's size only with the map.
//! - **Off** ([`FogOfWar`]): `--no-fog` and the debug toggle (F3, a placeholder key, only while debug commands are
//!   on, or in a replay's playback) turn it off for QA: then the client shows and lets the player select
//!   everything, as if every tile were Visible. The map editor never shows fog; its play-tests do.
//!
//! Placeholders, the design being silent on how fog looks: the Explored dimming and the soft edges.

use bevy::asset::RenderAssetUsages;
use bevy::ecs::system::SystemParam;
use bevy::image::ImageSampler;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::sprite_render::AlphaMode2d;
use sc_sim::vision::{LastKnown, PlayerVision};
use sc_sim::{Faction, Map, Object, ObjectId, PlayerId, SU_PER_GU, Sim, TilePos, VisibilityState};

use crate::coords::DrawLayer;
use crate::input::{ActionState, InputAction};
use crate::map_render::{MapView, SURFACE_LIFT_Z, surface_vertex, tile_surface};
use crate::object_render::ObjectVisual;
use crate::replay::Playback;
use crate::sim_driver::{LocalPlayer, SimRes};
use crate::{ClientMode, ClientSystems};

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

/// Whether the fog of war hides what the local player doesn't see. Display only.
#[derive(Resource, Copy, Clone, Debug, PartialEq, Eq)]
pub struct FogOfWar {
    /// On unless `--no-fog`; [`InputAction::DebugToggleFog`] switches it while debug commands are on.
    pub enabled: bool,
}

impl FogOfWar {
    /// Whether the fog applies in `mode`: in a game and in a play-test while it is enabled, never in the editor.
    pub fn applies(self, mode: ClientMode) -> bool {
        self.enabled && mode.is_game()
    }
}

/// What one player sees of the sim, as the client shows it (`core/vision.md`): through its vision while the fog
/// applies, everything as if Visible while it doesn't.
#[derive(Copy, Clone, Debug)]
pub struct Sight<'a> {
    player: PlayerId,
    /// `None` while the fog doesn't apply.
    fog: Option<Fogged<'a>>,
}

#[derive(Copy, Clone, Debug)]
struct Fogged<'a> {
    /// The player's vision; `None` for a player who isn't in the game, who sees nothing.
    vision: Option<&'a PlayerVision>,
    /// The player plays the GDO (`core/vision.md > Explored Selectability`).
    gdo: bool,
}

impl<'a> Sight<'a> {
    /// `player`'s sight in `sim`: through its vision if `fog` applies ([`FogOfWar::applies`]).
    pub fn new(sim: &'a Sim, player: PlayerId, fog: bool) -> Self {
        let fog = fog.then(|| Fogged {
            vision: sim.vision().player(player),
            gdo: sim.player(player).is_some_and(|p| p.faction == Faction::Gdo),
        });
        Self { player, fog }
    }

    pub fn player(&self) -> PlayerId {
        self.player
    }

    /// Whether the fog applies: `false` while everything shows.
    pub fn is_fogged(&self) -> bool {
        self.fog.is_some()
    }

    /// Whether the player plays the GDO; `false` while the fog doesn't apply, which needs no exception.
    pub fn is_gdo(&self) -> bool {
        self.fog.is_some_and(|f| f.gdo)
    }

    /// How the player sees `tile`.
    pub fn tile(&self, tile: TilePos) -> VisibilityState {
        match self.fog {
            None => VisibilityState::Visible,
            Some(f) => f.vision.map_or(VisibilityState::Unexplored, |v| v.state(tile)),
        }
    }

    /// How the player sees the tile holding the ground point `ground`.
    pub fn at(&self, ground: Vec2) -> VisibilityState {
        self.tile(TilePos::containing(ground))
    }

    /// The best state among the tiles `object` covers (`PlayerVision::object_state`).
    pub fn object(&self, object: &Object) -> VisibilityState {
        match self.fog {
            None => VisibilityState::Visible,
            Some(f) => f.vision.map_or(VisibilityState::Unexplored, |v| v.object_state(object)),
        }
    }

    /// Whether the player sees `object` as it is now: its own objects, and the others on a Visible tile. The views
    /// draw only these of the live objects; Explored tiles show [records](Sight::remembered) instead.
    pub fn sees(&self, object: &Object) -> bool {
        object.owner == Some(self.player) || self.object(object) == VisibilityState::Visible
    }

    /// What Explored tiles show (`core/vision.md > Explored`): the records of the structures and resources the
    /// player remembers and doesn't see now, in their last-known state, by id. None while the fog doesn't apply,
    /// when every object shows as it is.
    pub fn remembered(&self) -> impl Iterator<Item = &'a LastKnown> + use<'a> {
        self.fog.and_then(|f| f.vision).into_iter().flat_map(|v| v.remembered())
    }

    /// The record shown in place of object `id` ([`Sight::remembered`]), if any.
    pub fn remembered_record(&self, id: ObjectId) -> Option<&'a LastKnown> {
        self.fog?.vision?.remembered_record(id)
    }
}

/// The local player's [`Sight`], for systems.
#[derive(SystemParam)]
pub struct LocalSight<'w> {
    sim: Res<'w, SimRes>,
    player: Res<'w, LocalPlayer>,
    fog: Res<'w, FogOfWar>,
    mode: Res<'w, ClientMode>,
}

impl LocalSight<'_> {
    pub fn get(&self) -> Sight<'_> {
        Sight::new(&self.sim, self.player.0, self.fog.applies(*self.mode))
    }

    /// The sim it looks at.
    pub fn sim(&self) -> &Sim {
        &self.sim
    }

    /// Whether what it shows may have changed since the system last ran: the sim, the local player, the fog or the
    /// mode changed.
    pub fn is_changed(&self) -> bool {
        self.sim.is_changed() || self.player.is_changed() || self.fog.is_changed() || self.mode.is_changed()
    }
}

/// The fog layer's handles: one mesh, one texture, one material, created once and rebuilt in place.
#[derive(Resource)]
pub struct FogAssets {
    pub mesh: Handle<Mesh>,
    pub texture: Handle<Image>,
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
    /// The player's `PlayerVision::revision`; `None` for a player who isn't in the game.
    revision: Option<u32>,
    map_build: Option<u32>,
}

pub struct FogPlugin {
    /// The fog is on from the start (no `--no-fog`).
    pub enabled: bool,
}

impl Plugin for FogPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(FogOfWar { enabled: self.enabled })
            .init_resource::<FogView>()
            .add_systems(Startup, spawn_fog)
            .add_systems(Update, toggle_fog.in_set(ClientSystems::Actions))
            .add_systems(
                Update,
                (update_fog_layer, hide_unseen_objects).in_set(ClientSystems::Present),
            );
    }
}

/// The alpha of a tile's texel.
pub fn fog_alpha(state: VisibilityState) -> f32 {
    match state {
        VisibilityState::Unexplored => UNEXPLORED_ALPHA,
        VisibilityState::Explored => EXPLORED_ALPHA,
        VisibilityState::Visible => 0.0,
    }
}

/// The texels of a `width` × `height` map as `vision` sees it (row by row from the north-west corner, RGBA8): black,
/// with each tile's [`fog_alpha`]. Without a vision, every tile is Unexplored.
pub fn fog_texels(vision: Option<&PlayerVision>, width: u32, height: u32) -> Vec<u8> {
    let tiles = width as usize * height as usize;
    let texel = |state| [0, 0, 0, (fog_alpha(state) * 255.0).round() as u8];
    match vision {
        Some(v) => v.states().take(tiles).flat_map(texel).collect(),
        None => [texel(VisibilityState::Unexplored)].repeat(tiles).concat(),
    }
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
fn spawn_fog(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mesh = meshes.add(placeholder_mesh());
    let texture = images.add(fog_image(fog_texels(None, 1, 1), 1, 1));
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
        MeshMaterial2d(material),
        Transform::from_xyz(0.0, 0.0, DrawLayer::Terrain.base_z() + FOG_Z),
        Visibility::Hidden,
    ));
    commands.insert_resource(FogAssets { mesh, texture });
}

/// [`InputAction::DebugToggleFog`]: only while the game's debug commands are on (`SimConfig::debug`), since it
/// shows what the player doesn't see; and in a replay's playback, which shows a finished game.
fn toggle_fog(actions: Res<ActionState>, sim: Res<SimRes>, playback: Option<Res<Playback>>, mut fog: ResMut<FogOfWar>) {
    if actions.just_pressed(InputAction::DebugToggleFog) && (sim.config().debug || playback.is_some()) {
        fog.enabled = !fog.enabled;
    }
}

/// Builds the mesh when the map changed (in the first frame too: `MapView::builds` counts the terrain's builds),
/// writes the texture when what it shows changed, resizing it with the map, and shows the layer while the fog
/// applies.
fn update_fog_layer(
    sight: LocalSight,
    map_view: Res<MapView>,
    assets: Option<Res<FogAssets>>,
    mut view: ResMut<FogView>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut layer: Query<&mut Visibility, With<FogLayer>>,
) {
    let Some(assets) = assets else {
        return;
    };
    let fogged = sight.get().is_fogged();
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
    let player = sight.player.0;
    let vision = sim.vision().player(player);
    let key = FogKey {
        generation: sim.generation(),
        player,
        revision: vision.map(PlayerVision::revision),
        map_build,
    };
    if view.shown == Some(key) {
        return;
    }
    let texels = fog_texels(vision, map.width(), map.height());
    let resized = images
        .get(&assets.texture)
        .is_none_or(|image| image.width() != map.width().max(1) || image.height() != map.height().max(1));
    if resized {
        if let Err(error) = images.insert(&assets.texture, fog_image(texels, map.width(), map.height())) {
            error!("fog texture rebuild failed: {error}");
        }
    } else if let Some(mut image) = images.get_mut(&assets.texture) {
        image.data = Some(texels);
    }
    view.shown = Some(key);
    view.texture_writes += 1;
}

/// Hides the visual of every object the local player doesn't see ([`Sight::sees`]): on Explored tiles too, where
/// the records show instead (`object_render::remembered`). Visuals of a headless app have no `Visibility` until this
/// gives them one.
fn hide_unseen_objects(
    mut commands: Commands,
    sight: LocalSight,
    mut visuals: Query<(Entity, &ObjectVisual, Option<&mut Visibility>)>,
) {
    let seen = sight.get();
    for (entity, visual, visibility) in &mut visuals {
        let shown = sight.sim.object(visual.0).is_some_and(|o| seen.sees(o));
        let wanted = if shown {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        match visibility {
            Some(mut visibility) => {
                visibility.set_if_neq(wanted);
            }
            None => {
                commands.entity(entity).insert(wanted);
            }
        }
    }
}
