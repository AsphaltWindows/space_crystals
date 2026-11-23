//! Where frames go: a window, an offscreen image (QA screenshots without a window), or nowhere (headless tests and
//! sim-only QA runs).
//!
//! Every other plugin reads [`RenderSurface`] (what cameras render to) and [`SurfaceSize`] (its size), so none of
//! them cares which of the three is active.

use std::time::Duration;

use bevy::app::ScheduleRunnerPlugin;
use bevy::camera::RenderTarget;
use bevy::input::InputPlugin;
use bevy::prelude::*;
use bevy::render::RenderPlugin;
use bevy::render::render_resource::TextureFormat;
use bevy::window::{ExitCondition, PresentMode, PrimaryWindow, WindowCloseRequested, WindowRef, WindowResolution};
use bevy::winit::WinitPlugin;

/// Present while closing the window needs a confirmation: the map editor asks a second time when there are unsaved
/// changes, and exits itself. Without it, a close request (`WindowCloseRequested`) exits the app at once.
#[derive(Resource, Default, Debug)]
pub struct ConfirmClose;

/// `WindowPlugin::close_when_requested` is off, so that the editor can confirm a close: here a close request exits
/// the app, unless [`ConfirmClose`] says someone else decides.
fn exit_on_close_request(
    mut requests: MessageReader<WindowCloseRequested>,
    confirm: Option<Res<ConfirmClose>>,
    mut exit: MessageWriter<AppExit>,
) {
    if requests.read().count() > 0 && confirm.is_none() {
        exit.write(AppExit::Success);
    }
}

/// How the client presents frames.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DisplayMode {
    /// A normal window on the desktop.
    Window,
    /// No window: cameras render into an offscreen image, which screenshots capture.
    Offscreen,
    /// No window and no renderer at all. Used by tests and sim-only QA runs.
    Headless,
}

/// The render target every camera of the client uses.
#[derive(Resource, Clone, Debug)]
pub struct RenderSurface {
    pub target: RenderTarget,
}

/// Size of the render surface, kept current every frame.
#[derive(Resource, Copy, Clone, Debug, PartialEq)]
pub struct SurfaceSize {
    pub physical: UVec2,
    pub scale_factor: f32,
}

impl SurfaceSize {
    pub fn logical(&self) -> Vec2 {
        self.physical.as_vec2() / self.scale_factor
    }
}

/// Wait between frames of the offscreen and headless loops: about 60 frames per second.
const FRAME_WAIT: Duration = Duration::from_micros(16_667);

pub struct PlatformPlugin {
    pub display: DisplayMode,
    /// Physical size of the window or offscreen image.
    pub size: UVec2,
    /// Reproducible rendering for QA: scale factor 1 (screenshots are exactly `size`) and synchronous pipeline
    /// compilation, so the first frame already draws everything.
    pub reproducible: bool,
    /// Run frames back to back instead of pacing them (`--fast`).
    pub unpaced: bool,
}

impl Plugin for PlatformPlugin {
    fn build(&self, app: &mut App) {
        let size = self.size.max(UVec2::ONE);
        let render = RenderPlugin {
            synchronous_pipeline_compilation: self.reproducible,
            ..default()
        };
        let frame_wait = if self.unpaced { Duration::ZERO } else { FRAME_WAIT };
        match self.display {
            DisplayMode::Window => {
                let mut resolution = WindowResolution::new(size.x, size.y);
                if self.reproducible {
                    resolution = resolution.with_scale_factor_override(1.0);
                }
                // QA and `--fast` runs don't wait for vsync: they want frames back to back, and with the display
                // asleep X11 paces a vsynced window at one frame per second.
                let present_mode = if self.reproducible || self.unpaced {
                    PresentMode::AutoNoVsync
                } else {
                    PresentMode::AutoVsync
                };
                let window = Window {
                    title: "Space Crystals".into(),
                    resolution,
                    present_mode,
                    ..default()
                };
                app.add_plugins(
                    DefaultPlugins
                        .set(WindowPlugin {
                            primary_window: Some(window),
                            close_when_requested: false,
                            ..default()
                        })
                        .set(render),
                );
                app.insert_resource(RenderSurface {
                    target: RenderTarget::Window(WindowRef::Primary),
                })
                .insert_resource(SurfaceSize {
                    physical: size,
                    scale_factor: 1.0,
                })
                .add_systems(First, track_window_size)
                .add_systems(Update, exit_on_close_request);
            }
            DisplayMode::Offscreen => {
                app.add_plugins(
                    DefaultPlugins
                        .set(WindowPlugin {
                            primary_window: None,
                            exit_condition: ExitCondition::DontExit,
                            ..default()
                        })
                        .set(render)
                        .disable::<WinitPlugin>(),
                )
                .add_plugins(ScheduleRunnerPlugin::run_loop(frame_wait));
                let image = offscreen_image(size);
                let handle = app.world_mut().resource_mut::<Assets<Image>>().add(image);
                app.insert_resource(RenderSurface {
                    target: RenderTarget::Image(handle.into()),
                })
                .insert_resource(SurfaceSize {
                    physical: size,
                    scale_factor: 1.0,
                });
            }
            DisplayMode::Headless => {
                app.add_plugins(MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(frame_wait)))
                    .add_plugins((TransformPlugin, AssetPlugin::default(), InputPlugin))
                    // No window ever asks to close, but tests send the message to drive the editor's confirmation.
                    .add_message::<WindowCloseRequested>()
                    .add_systems(Update, exit_on_close_request)
                    .init_asset::<Mesh>()
                    .init_asset::<ColorMaterial>()
                    .init_asset::<Image>()
                    .insert_resource(RenderSurface {
                        target: RenderTarget::Window(WindowRef::Primary),
                    })
                    .insert_resource(SurfaceSize {
                        physical: size,
                        scale_factor: 1.0,
                    });
            }
        }
    }
}

/// An sRGB colour target. It keeps its default `MAIN_WORLD | RENDER_WORLD` usage: cameras size an image target
/// from the main-world asset. Screenshots redirect the frame into their own readable texture, so no extra usage
/// flags are needed.
fn offscreen_image(size: UVec2) -> Image {
    Image::new_target_texture(size.x, size.y, TextureFormat::Rgba8UnormSrgb, None)
}

fn track_window_size(window: Query<&Window, With<PrimaryWindow>>, mut size: ResMut<SurfaceSize>) {
    let Ok(window) = window.single() else {
        return;
    };
    let current = SurfaceSize {
        physical: window.physical_size().max(UVec2::ONE),
        scale_factor: window.scale_factor(),
    };
    size.set_if_neq(current);
}
