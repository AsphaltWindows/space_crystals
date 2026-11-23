//! # space_crystals — the Bevy client
//!
//! Draws the deterministic simulation (`sc_sim`), turns input into commands, and hosts the HUD and QA mode. Read
//! `ARCHITECTURE.md > Client architecture` before changing anything here.
//!
//! [`build_app`] assembles one plugin per feature module:
//! - [`platform`]: a window, an offscreen image (QA) or no rendering at all (tests);
//! - [`sim_driver`]: owns the `Sim` and steps it in `FixedUpdate`, 16 ticks per second;
//! - [`input`]: raw keys and the screen edge become [`input::InputAction`]s;
//! - [`camera`]: the world camera (28 GU wide, between the UI bars) and the UI camera;
//! - [`map_render`]: the terrain mesh and the grid overlay;
//! - [`object_render`]: placeholder visuals of sim objects;
//! - [`hud`]: the game mode's HUD frame;
//! - [`qa`]: screenshots, exit at a tick, state dump.
//!
//! Everything except [`hud`] is independent of the game mode, so a map editor can reuse it with its own UI.

pub mod camera;
pub mod cli;
pub mod coords;
pub mod hud;
pub mod input;
pub mod map_render;
pub mod object_render;
pub mod platform;
pub mod qa;
pub mod shapes;
pub mod sim_driver;

use bevy::prelude::*;
use sc_sim::{PlayerId, Scenario};

use crate::camera::GameCameraPlugin;
use crate::hud::HudPlugin;
use crate::input::ActionInputPlugin;
use crate::map_render::MapRenderPlugin;
use crate::object_render::ObjectRenderPlugin;
use crate::platform::{DisplayMode, PlatformPlugin};
use crate::qa::{QaConfig, QaPlugin};
use crate::sim_driver::SimDriverPlugin;

/// Order of the client's `Update` systems within a frame.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum ClientSystems {
    /// Catch up with the ticks that just ran: interpolation factor, visuals to spawn or despawn, map meshes.
    SyncSim,
    /// Consume this frame's `InputAction`s: camera pan, toggles.
    Actions,
    /// Derive transforms, viewports and visibility from the sim and the view state.
    Present,
    /// QA bookkeeping, after the frame's state is final: screenshots, exit, holding the sim.
    Qa,
}

/// Everything [`build_app`] needs to know.
#[derive(Clone, Debug)]
pub struct AppConfig {
    pub scenario: Scenario,
    /// Whose view the client shows.
    pub player: PlayerId,
    pub display: DisplayMode,
    /// Physical size of the window or offscreen image.
    pub window_size: UVec2,
    pub qa: QaConfig,
    /// Run as many ticks per frame as possible.
    pub fast: bool,
}

impl AppConfig {
    /// Player 1's view of `scenario` in a 1280×720 window, without QA.
    pub fn new(scenario: Scenario) -> Self {
        Self {
            scenario,
            player: PlayerId(1),
            display: DisplayMode::Window,
            window_size: cli::DEFAULT_WINDOW_SIZE,
            qa: QaConfig::default(),
            fast: false,
        }
    }
}

/// Builds the client app. Run it with `App::run`, or step it with `App::update` in tests (after `App::finish` and
/// `App::cleanup`, which the runners otherwise call).
pub fn build_app(config: AppConfig) -> App {
    let qa_run = config.qa.is_active();
    let mut app = App::new();
    app.add_plugins(PlatformPlugin {
        display: config.display,
        size: config.window_size,
        reproducible: qa_run,
        unpaced: config.fast,
    })
    .configure_sets(
        Update,
        (
            ClientSystems::SyncSim,
            ClientSystems::Actions,
            ClientSystems::Present,
            ClientSystems::Qa,
        )
            .chain(),
    )
    // Mode-independent: everything a view of the map needs.
    .add_plugins((
        SimDriverPlugin {
            scenario: config.scenario,
            player: config.player,
            fast: config.fast,
        },
        ActionInputPlugin { raw_input: !qa_run },
        GameCameraPlugin,
        MapRenderPlugin,
        ObjectRenderPlugin,
        QaPlugin { config: config.qa },
    ))
    // The game mode.
    .add_plugins(HudPlugin);
    app
}
