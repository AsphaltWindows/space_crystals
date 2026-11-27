//! The Minimap (`systems/controls/hud.md > Minimap`, slice M5-MM, design change 0415): the HUD Bar's leftmost section,
//! "a small view of the **whole map**", and how clicks on it move the camera and give orders.
//!
//! - **Where** ([`MinimapFrame`]): the whole map, its aspect kept, as large as fits in the section's content box and
//!   centred there (letterboxed), on whole physical pixels. One tested function maps each way between a point of the
//!   panel and a ground point ([`MinimapFrame::ground_at`], [`MinimapFrame::screen_at`]).
//! - **What it shows** (`systems/controls/hud.md > Minimap > What it shows`): one CPU-side image, one texel per
//!   physical pixel, shown by a UI node ([`MinimapImage`]; [`raster`] paints it): the terrain in simplified colours,
//!   baked once per map and frame; the fog of war over it, as the main map's (`core/vision.md > VisibilityState`); the
//!   objects over that, by the same predicates as the main map's views (`crate::fog::Sight`): records on Explored
//!   tiles, the live objects only where the player sees them, a hidden object never. The fog and the objects are
//!   redrawn only when what they show may have changed ([`LayersKey`]: the sim's tick, the local player, the fog and
//!   the records), at most [`REDRAWS_PER_SECOND`] times a second while the sim runs, at once while it stands still
//!   ([`redraw_due`]). The image is written in place under its handle while the frame keeps its size, and replaced by
//!   a new one, which the node is pointed at, when the size changes.
//! - **The camera's view** "as a rectangle outline", every frame, after the camera is clamped: a UI node's border
//!   ([`CameraOutline`], [`MinimapFrame::camera_rect`]) round the ground the view shows ([`viewed_ground`]), as crisp
//!   as a line drawn into the image and never waiting for a redraw.
//! - **Using it** ([`input`]): a left-click or a drag centres the camera on that ground point; a right-click is the
//!   selection's right-click on that ground point, and while a target command waits a left-click targets it, both
//!   through the command flow's own functions for a click on the map (`crate::command_ui`, ground only: the designer's
//!   placeholder). Placement isn't possible from it.
//! - **Alerts** ([`alerts`], slice M5-MM2, design change 0443, `systems/controls/hud.md > Minimap > Alerts`): a hit on
//!   one of the player's objects off screen, outside [`viewed_ground`], pings there, a red ring over the image that
//!   shrinks and fades, at most one an area every 10 s; Space centres the camera on the latest.
//!
//! Colours and sizes are placeholders (Q0001).

mod alerts;
mod frame;
mod input;
pub mod raster;

use bevy::asset::RenderAssetUsages;
use bevy::ecs::system::SystemParam;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use sc_sim::{Map, PlayerId, Tick};

use super::layout::{SECTION_BORDER_PX, section_rect};
use super::{HudSection, SectionLabel, show_node};
use crate::camera::{CameraView, CameraZoom, ViewportInsets, map_extent_su, viewport_rect, visible_extent_su};
use crate::command_ui::CommandUiSystems;
use crate::coords::ground_under;
use crate::fog::{LocalSight, Revision};
use crate::map_render::MapView;
use crate::platform::SurfaceSize;
use crate::sim_driver::{SimControl, SimRes};
use crate::{ClientMode, ClientSystems};
use raster::{Terrain, marks, paint};

pub use alerts::{MinimapAlerts, Ping, PingLayer, PingRing, ring_at};
pub use frame::MinimapFrame;
pub use input::{MinimapClick, MinimapPointer, MinimapPress};

/// "The fog and object layers are redrawn at most about 15 times a second" (the Director's technical decision for
/// slice M5-MM): often enough for units to move smoothly at the sim's 16 ticks a second, seldom enough to cost
/// nothing. A technical constant.
pub const REDRAWS_PER_SECOND: f64 = 15.0;
/// The camera's outline: its width (logical pixels) and colour. Placeholders (Q0001).
const OUTLINE_PX: f32 = 1.0;
const OUTLINE_COLOR: Color = Color::srgb(0.95, 0.95, 0.95);

/// The minimap's image node, in the Minimap section.
#[derive(Component)]
pub struct MinimapImage;

/// The camera's rectangle on the minimap: a node with a border, in the image node.
#[derive(Component)]
pub struct CameraOutline;

/// What the minimap shows, and how often it was drawn (for tests, QA and the perf figure).
#[derive(Resource, Default)]
pub struct MinimapView {
    /// The image the node shows, made for [`MinimapView::frame`]'s size.
    pub image: Option<Handle<Image>>,
    /// The frame it was drawn for last.
    pub frame: Option<MinimapFrame>,
    /// The camera's rectangle as last placed, logical pixels ([`MinimapFrame::camera_rect`]).
    pub camera_rect: Option<Rect>,
    /// How many times the terrain was baked, and the fog and object layers drawn.
    pub bakes: u32,
    pub redraws: u32,
    /// The terrain baked, and for which map build and image size.
    terrain: Option<((u32, UVec2), Terrain)>,
    /// What the image shows, and when it was drawn (seconds of real time).
    drawn: Option<(LayersKey, f64)>,
}

/// Everything the fog and object layers depend on: the sim and its tick (the objects move every tick), whom the
/// local player is and what their sight shows of the tiles and the records (`Sight::revision` and
/// `Sight::records_revision`, `None` while the fog doesn't apply), and the terrain under them.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct LayersKey {
    generation: u32,
    tick: Tick,
    player: PlayerId,
    revision: Option<Revision>,
    records: Option<Revision>,
    map_build: u32,
    texels: UVec2,
}

/// Whether the layers are redrawn at `now` (seconds of real time), showing `key`, after they were drawn showing what
/// `drawn` says, when: only when what they show changed; then at once the first time and while the sim stands still
/// (`still`: paused, held or over, so that the frame shows the exact state, as QA's screenshots need), else at most
/// [`REDRAWS_PER_SECOND`] times a second.
fn redraw_due<K: PartialEq>(drawn: Option<&(K, f64)>, key: &K, now: f64, still: bool) -> bool {
    match drawn {
        None => true,
        Some((shown, _)) if shown == key => false,
        Some((_, at)) => still || now - at >= 1.0 / REDRAWS_PER_SECOND,
    }
}

pub struct MinimapPlugin;

impl Plugin for MinimapPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MinimapView>()
            .init_resource::<MinimapPointer>()
            .init_resource::<MinimapAlerts>()
            // Before the command flow, which takes the minimap's targeting clicks and right-clicks; the camera
            // follows after it, once it is known whether the command flow took the press, and after the frame's pans,
            // as it does to Space's jump to the latest alert.
            .add_systems(
                Update,
                (
                    input::track_minimap_pointer.before(CommandUiSystems::Input),
                    input::follow_minimap_press
                        .after(CommandUiSystems::Input)
                        .after(crate::camera::grab_pan),
                    alerts::jump_to_alert.after(crate::camera::grab_pan),
                )
                    .in_set(ClientSystems::Actions),
            )
            // The alerts are taken where the camera looks once the frame's pans are clamped, as the outline is placed.
            .add_systems(
                Update,
                (
                    spawn_minimap,
                    draw_minimap,
                    place_camera_outline.after(crate::camera::apply_camera),
                    alerts::take_alerts.after(crate::camera::apply_camera),
                    alerts::place_pings,
                )
                    .chain()
                    .in_set(ClientSystems::Present),
            );
    }
}

/// The minimap's nodes, once the HUD has its Minimap section: the image node, and in it the camera's outline and the
/// alerts' layer over that ([`alerts`]). The section's name gives way to them.
fn spawn_minimap(
    mut commands: Commands,
    spawned: Query<(), With<MinimapImage>>,
    sections: Query<(Entity, &HudSection)>,
    mut labels: Query<(&SectionLabel, &mut Node)>,
) {
    if !spawned.is_empty() {
        return;
    }
    let Some((section, _)) = sections.iter().find(|(_, s)| **s == HudSection::Minimap) else {
        return;
    };
    commands.spawn((
        Name::new("Minimap"),
        MinimapImage,
        Node {
            position_type: PositionType::Absolute,
            ..default()
        },
        ImageNode {
            image_mode: NodeImageMode::Stretch,
            ..default()
        },
        ChildOf(section),
        children![
            (
                Name::new("Minimap camera"),
                CameraOutline,
                Node {
                    position_type: PositionType::Absolute,
                    border: UiRect::all(px(OUTLINE_PX)),
                    display: Display::None,
                    ..default()
                },
                BorderColor::all(OUTLINE_COLOR),
            ),
            alerts::ping_layer(),
        ],
    ));
    for (label, mut node) in &mut labels {
        if label.0 == HudSection::Minimap {
            show_node(&mut node, false);
        }
    }
}

/// Puts an absolute node at `at` (logical pixels from its parent's padding box), `size` large; changes it only when
/// that differs.
fn place(node: &mut Mut<Node>, at: Vec2, size: Vec2) {
    let placed = (px(at.x), px(at.y), px(size.x), px(size.y));
    if (node.left, node.top, node.width, node.height) != placed {
        (node.left, node.top, node.width, node.height) = placed;
    }
}

/// A blank image of `size` texels, RGBA8 sRGB, sampled nearest: one texel per physical pixel.
fn blank_image(size: UVec2) -> Image {
    let extent = Extent3d {
        width: size.x.max(1),
        height: size.y.max(1),
        depth_or_array_layers: 1,
    };
    let data = vec![0; extent.width as usize * extent.height as usize * 4];
    let mut image = Image::new(
        extent,
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::nearest();
    image
}

/// What the minimap is drawn from.
#[derive(SystemParam)]
struct DrawSource<'w> {
    mode: Res<'w, ClientMode>,
    size: Res<'w, SurfaceSize>,
    sim: Res<'w, SimRes>,
    sight: LocalSight<'w>,
    map_view: Res<'w, MapView>,
    control: Res<'w, SimControl>,
    time: Res<'w, Time<Real>>,
}

/// Places the image node at the frame, makes the image (anew when the frame's size changes), bakes the terrain when
/// the map or the size changes, and draws the fog and the objects over it when they are due ([`redraw_due`]). Only in
/// the game modes, the HUD's.
fn draw_minimap(
    source: DrawSource,
    mut view: ResMut<MinimapView>,
    mut images: ResMut<Assets<Image>>,
    mut nodes: Query<(&mut Node, &mut ImageNode), With<MinimapImage>>,
) {
    if !source.mode.is_game() {
        return;
    }
    let sim = source.sight.sim();
    let map = sim.map();
    let frame = MinimapFrame::on(*source.size, map_extent_su(map));
    let view = &mut *view;
    let image = match &view.image {
        Some(image) if view.frame.is_some_and(|f| f.texels == frame.texels) => image.clone(),
        _ => {
            let image = images.add(blank_image(frame.texels));
            view.image = Some(image.clone());
            view.drawn = None;
            image
        }
    };
    view.frame = Some(frame);
    // Absolute children are placed from the section's padding box: inside its border.
    let section = section_rect(HudSection::Minimap, source.size.logical());
    let origin = section.min + Vec2::splat(SECTION_BORDER_PX);
    for (mut node, mut image_node) in &mut nodes {
        place(&mut node, frame.area.min - origin, frame.area.size());
        if image_node.image != image {
            image_node.image = image.clone();
        }
    }
    let terrain_key = (source.map_view.builds, frame.texels);
    if view.terrain.as_ref().is_none_or(|(key, _)| *key != terrain_key) {
        view.terrain = Some((terrain_key, Terrain::bake(map, &frame)));
        view.bakes += 1;
        view.drawn = None;
    }
    let seen = source.sight.get();
    let key = LayersKey {
        generation: source.sim.generation(),
        tick: sim.tick(),
        player: seen.player(),
        revision: seen.revision(),
        records: seen.records_revision(),
        map_build: source.map_view.builds,
        texels: frame.texels,
    };
    let now = source.time.elapsed_secs_f64();
    let still = !source.control.may_step(sim);
    if !redraw_due(view.drawn.as_ref(), &key, now, still) {
        return;
    }
    let Some((_, terrain)) = &view.terrain else {
        return;
    };
    let states = seen.tile_states(map.width(), map.height());
    let marks = marks(sim, &seen);
    if let Some(mut asset) = images.get_mut(&image)
        && let Some(data) = asset.data.as_mut()
    {
        paint(data, &frame, terrain, &states, &marks);
    }
    view.drawn = Some((key, now));
    view.redraws += 1;
}

/// The ground the world view shows, space units: a rectangle of the view's extent at `zoom`
/// ([`visible_extent_su`]), the view centred on the view point `center`, around the ground point drawn there
/// ([`ground_under`]). A reading: the view looks at the ground lifted by its height, so the ground it shows isn't
/// exactly a rectangle; this is the rectangle the view would show of flat ground there. The camera's outline shows it
/// on the minimap ([`MinimapFrame::camera_rect`]), and the alerts call off screen what lies outside it ([`alerts`]), so
/// that the two agree.
pub fn viewed_ground(map: &Map, size: SurfaceSize, insets: ViewportInsets, zoom: f32, center: Vec2) -> Rect {
    let (_, viewport) = viewport_rect(size, insets);
    let visible = visible_extent_su(viewport, size.physical.x, zoom);
    Rect::from_center_size(ground_under(map, center), visible)
}

/// Where the camera looks: what the camera's outline is placed from, and what the alerts call off screen.
#[derive(SystemParam)]
struct CameraSource<'w> {
    mode: Res<'w, ClientMode>,
    size: Res<'w, SurfaceSize>,
    insets: Res<'w, ViewportInsets>,
    zoom: Res<'w, CameraZoom>,
    camera: Res<'w, CameraView>,
    sim: Res<'w, SimRes>,
}

impl CameraSource<'_> {
    /// The ground the world view shows now ([`viewed_ground`]).
    fn viewed_ground(&self) -> Rect {
        let center = self.camera.center;
        viewed_ground(self.sim.map(), *self.size, *self.insets, self.zoom.0, center)
    }
}

/// `systems/controls/hud.md > Minimap > What it shows`: "The camera's view as a rectangle outline", every frame,
/// where the camera looks once the frame's pans are clamped: the ground the view shows ([`viewed_ground`]) on the
/// minimap ([`MinimapFrame::camera_rect`]).
fn place_camera_outline(
    source: CameraSource,
    mut view: ResMut<MinimapView>,
    mut outlines: Query<&mut Node, With<CameraOutline>>,
) {
    if !source.mode.is_game() {
        return;
    }
    let frame = MinimapFrame::on(*source.size, map_extent_su(source.sim.map()));
    let rect = frame.camera_rect(source.viewed_ground());
    if view.camera_rect != rect {
        view.camera_rect = rect;
    }
    for mut node in &mut outlines {
        show_node(&mut node, rect.is_some());
        if let Some(rect) = rect {
            place(&mut node, rect.min - frame.area.min, rect.size());
        }
    }
}

#[cfg(test)]
mod tests;
