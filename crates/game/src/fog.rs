//! Fog of war (`core/vision.md`): the local player's view of the map through its vision at the end of the latest
//! tick (`Sim::vision`). Display only: the sim computes every player's vision whatever the client shows.
//!
//! - **What shows** ([`Sight`]): with the fog on, the local player sees its own objects and everything on its
//!   Visible tiles in real time (`> Visible`); on Explored tiles the terrain, and the structures and resources it
//!   remembers in their last-known state, never the live objects there (`> Explored`: the sim's records,
//!   `PlayerVision::remembered`); on Unexplored tiles nothing (`> Unexplored`). The object visuals
//!   ([`hide_unseen_objects`], `object_render::remembered`), the combat effects, the path overlay, the command
//!   indicators, the InfoPanel and the selection (`selection::pick`) all ask it.
//! - **Spectating** (`core/game_rules.md > End of Game`, "Defeated before the end", design change 0102): a defeated
//!   player watches with their allies' vision, never a full map reveal. A sight looks through the union of the
//!   visions it watches ([`Sight::watching`]): a tile shows in the best state any of them gives it, and an object
//!   remembered by several shows as last seen. The sim has no teams yet, so a defeated player has no allies and
//!   watches their own vision alone, which has no Visible tile left: they own nothing.
//! - **The fog layer**: one mesh over the whole map and one texture of one texel per tile, never an entity or a
//!   material per tile (`ARCHITECTURE.md > Client architecture > Visuals`). The mesh follows the terrain's surface
//!   tile by tile as the terrain mesh draws it (a quad per flat tile, four triangles around the centre of a sloped
//!   one), in front of it in the terrain's z band ([`FOG_Z`]): objects stand on it. Each texel is black, with [`UNEXPLORED_ALPHA`] for an Unexplored tile, [`EXPLORED_ALPHA`] for an
//!   Explored one and none for a Visible one, sampled linearly so the edges soften across a tile
//!   ([`FOG_SOFT_EDGES`]).
//!   The texture is written only when what it shows changes: the tiles of the visions the local player watches
//!   (`Sight::revision`), the local player, or the sim (`SimRes::generation`); the mesh and the texture's size only
//!   with the map.
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
use sc_sim::{Faction, Map, Object, ObjectId, Player, PlayerId, SU_PER_GU, Sim, TilePos, VisibilityState};

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

/// What one player sees of the sim, as the client shows it (`core/vision.md`): while the fog applies, through the
/// visions they watch ([`Sight::new`]); everything as if Visible while it doesn't.
#[derive(Copy, Clone, Debug)]
pub struct Sight<'a> {
    player: PlayerId,
    /// `None` while the fog doesn't apply.
    fog: Option<Fogged<'a>>,
}

#[derive(Copy, Clone, Debug)]
struct Fogged<'a> {
    /// Every player's vision, in the sim's player order (`Vision::players`).
    visions: &'a [PlayerVision],
    /// The ones the player watches through. None for a player who isn't in the game: they see nothing.
    watched: Watched,
    /// The player plays the GDO (`core/vision.md > Explored Selectability`).
    gdo: bool,
}

/// Which of the sim's visions a [`Sight`] watches through: bit `i` for the vision of the sim's `i`-th player
/// (`Vision::players`), for as many players as a game can have. Part of [`Sight::revision`], so that a view keyed on
/// it redraws when whom the sight watches changes.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Watched([u64; 4]);

impl Watched {
    fn contains(self, i: usize) -> bool {
        self.0.get(i / 64).is_some_and(|word| (word >> (i % 64)) & 1 == 1)
    }

    fn insert(&mut self, i: usize) {
        if let Some(word) = self.0.get_mut(i / 64) {
            *word |= 1 << (i % 64);
        }
    }
}

/// A [`Sight`]'s revision ([`Sight::revision`], [`Sight::records_revision`]): whom it watches, and the sum (wrapping)
/// of a counter of each vision it watches. It changes whenever what those visions show may have changed.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Revision {
    pub watched: Watched,
    pub sum: u32,
}

impl<'a> Sight<'a> {
    /// `player`'s sight in `sim`, through the visions they watch if `fog` applies ([`FogOfWar::applies`]): their
    /// own, and while they are defeated their allies' too (design change 0102: "their fog is the union of their
    /// allies' vision", [`Sim::allies`]). A reading: their own vision stays in the union. Since they own nothing it
    /// has no Visible tile, so it adds only what they explored and remember themselves, and a defeated player
    /// without allies (every player until the sim has teams) keeps their map rather than seeing it go black.
    pub fn new(sim: &'a Sim, player: PlayerId, fog: bool) -> Self {
        if sim.player(player).is_some_and(Player::is_defeated) {
            let mut viewers = vec![player];
            viewers.extend(sim.allies(player));
            Self::watching(sim, player, &viewers, fog)
        } else {
            Self::watching(sim, player, &[player], fog)
        }
    }

    /// `player`'s sight through the visions of `viewers` if `fog` applies: what their union shows (design change
    /// 0102). A tile is in the best state any of them gives it, an object in the best state of its tiles, and an
    /// object out of sight of all of them shows as the latest record any of them keeps of it.
    pub fn watching(sim: &'a Sim, player: PlayerId, viewers: &[PlayerId], fog: bool) -> Self {
        let fog = fog.then(|| {
            let visions = sim.vision().players();
            let mut watched = Watched::default();
            for (i, vision) in visions.iter().enumerate() {
                if viewers.contains(&vision.player()) {
                    watched.insert(i);
                }
            }
            Fogged {
                visions,
                watched,
                gdo: sim.player(player).is_some_and(|p| p.faction == Faction::Gdo),
            }
        });
        Self { player, fog }
    }

    pub fn player(&self) -> PlayerId {
        self.player
    }

    /// The visions it watches through; none while the fog doesn't apply.
    fn visions(&self) -> impl Iterator<Item = &'a PlayerVision> + use<'a> {
        self.fog.into_iter().flat_map(|f| {
            f.visions
                .iter()
                .enumerate()
                .filter(move |&(i, _)| f.watched.contains(i))
                .map(|(_, vision)| vision)
        })
    }

    /// Changes whenever the state of a tile it shows may have changed: whom it watches, and the sum of their
    /// `PlayerVision::revision`s (each only grows). `None` while the fog doesn't apply.
    pub fn revision(&self) -> Option<Revision> {
        let f = self.fog?;
        let sum = self.visions().fold(0u32, |sum, v| sum.wrapping_add(v.revision()));
        Some(Revision {
            watched: f.watched,
            sum,
        })
    }

    /// [`Sight::revision`] for the records it shows: whom it watches and the sum of their
    /// `PlayerVision::records_revision`s.
    pub fn records_revision(&self) -> Option<Revision> {
        let f = self.fog?;
        let sum = self
            .visions()
            .fold(0u32, |sum, v| sum.wrapping_add(v.records_revision()));
        Some(Revision {
            watched: f.watched,
            sum,
        })
    }

    /// Whether the fog applies: `false` while everything shows.
    pub fn is_fogged(&self) -> bool {
        self.fog.is_some()
    }

    /// Whether the player plays the GDO; `false` while the fog doesn't apply, which needs no exception.
    pub fn is_gdo(&self) -> bool {
        self.fog.is_some_and(|f| f.gdo)
    }

    /// How the player sees `tile`: its best state among the visions watched.
    pub fn tile(&self, tile: TilePos) -> VisibilityState {
        match self.fog {
            None => VisibilityState::Visible,
            Some(_) => self.best(|v| v.state(tile)),
        }
    }

    /// The best of `state` among the visions watched; Unexplored with none.
    fn best(&self, state: impl Fn(&PlayerVision) -> VisibilityState) -> VisibilityState {
        let mut best = VisibilityState::Unexplored;
        for vision in self.visions() {
            best = best.max(state(vision));
            if best == VisibilityState::Visible {
                break;
            }
        }
        best
    }

    /// How the player sees the tile holding the ground point `ground`.
    pub fn at(&self, ground: Vec2) -> VisibilityState {
        self.tile(TilePos::containing(ground))
    }

    /// The best state among the tiles `object` covers, or Visible if revealed (`PlayerVision::object_state`, 0177).
    pub fn object(&self, object: &Object) -> VisibilityState {
        match self.fog {
            None => VisibilityState::Visible,
            Some(_) => self.best(|v| v.object_state(object)),
        }
    }

    /// Whether the player sees `object` as it is now: its own objects, and the others on a Visible tile. The views
    /// draw only these of the live objects; Explored tiles show [records](Sight::remembered) instead.
    pub fn sees(&self, object: &Object) -> bool {
        object.owner == Some(self.player) || self.object(object) == VisibilityState::Visible
    }

    /// What Explored tiles show (`core/vision.md > Explored`): the records of the structures and resources the
    /// player remembers and doesn't see now, in their last-known state, one per object
    /// ([`Sight::remembered_record`]). None while the fog doesn't apply, when every object shows as it is.
    pub fn remembered(&self) -> impl Iterator<Item = &'a LastKnown> + use<'a> {
        let sight = *self;
        self.visions().flat_map(PlayerVision::remembered).filter(move |record| {
            sight
                .remembered_record(record.id)
                .is_some_and(|shown| std::ptr::eq(shown, *record))
        })
    }

    /// The record shown in place of object `id` ([`Sight::remembered`]), if any: the latest of the records the
    /// visions watched keep of it (the first of them on a tie), shown while its footprint's best tile is Explored:
    /// not while one sees it, nor while none has explored it (Unexplored is "fully black", design change 0159).
    pub fn remembered_record(&self, id: ObjectId) -> Option<&'a LastKnown> {
        let latest = self
            .visions()
            .filter_map(|v| v.record(id))
            .reduce(|latest, record| if record.seen > latest.seen { record } else { latest })?;
        let best = (latest.placement.tiles()).map(|tile| self.tile(tile)).max()?;
        (best == VisibilityState::Explored).then_some(latest)
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
    /// The sight's [`Sight::revision`]: whom it watches, and their tiles.
    revision: Option<Revision>,
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

/// The texels of a `width` × `height` map as `sight` sees it (row by row from the north-west corner, RGBA8): black,
/// with each tile's [`fog_alpha`], its best state among the visions the sight watches. Watching none, every tile is
/// Unexplored; without the fog, every tile is Visible.
pub fn fog_texels(sight: &Sight, width: u32, height: u32) -> Vec<u8> {
    let tiles = width as usize * height as usize;
    let texel = |state| [0, 0, 0, (fog_alpha(state) * 255.0).round() as u8];
    if !sight.is_fogged() {
        return [texel(VisibilityState::Visible)].repeat(tiles).concat();
    }
    let mut states = vec![VisibilityState::Unexplored; tiles];
    for vision in sight.visions() {
        for (best, state) in states.iter_mut().zip(vision.states()) {
            *best = (*best).max(state);
        }
    }
    states.into_iter().flat_map(texel).collect()
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
