//! # space_crystals — the Bevy client
//!
//! Draws the deterministic simulation (`sc_sim`), turns input into commands, and hosts the HUD, the map editor and
//! QA mode. Read `ARCHITECTURE.md > Client architecture` before changing anything here.
//!
//! [`build_app`] assembles one plugin per feature module:
//! - [`platform`]: a window, an offscreen image (QA) or no rendering at all (tests);
//! - [`sim_driver`]: owns the `Sim` and steps it in `FixedUpdate`, 16 ticks per second;
//! - [`input`]: keys, mouse, wheel and the screen edge become [`input::InputAction`]s, clicks and boxes, through
//!   the bindings of the mode's input context (the game's or the editor's);
//! - [`camera`]: the world camera (28 GU wide, between the UI) and the UI camera;
//! - [`map_render`]: the terrain mesh and the grid overlay;
//! - [`object_render`]: placeholder visuals of sim objects;
//! - [`path_overlay`]: the units' remaining paths, a development view;
//! - [`combat_effects`]: placeholder combat feedback: tracers, HP bars, death markers;
//! - [`selection`]: the players' selections and control groups (client-side UI state), and their markers;
//! - [`hud`]: the game mode's HUD frame, with the InfoPanel and the SelectionPanel;
//! - [`editor`]: the map editor (`--editor`), with its own UI, tools and play-test;
//! - [`qa`]: screenshots, exit at a tick, state dump, commands and input at given ticks.
//!
//! Everything except [`hud`], [`selection`] and [`editor`] is independent of the mode ([`ClientMode`]), apart from
//! the input layer, which applies the bindings of the mode's input context ([`ClientMode::input_context`]).

pub mod camera;
pub mod cli;
pub mod combat_effects;
pub mod coords;
pub mod editor;
pub mod hud;
pub mod input;
pub mod map_render;
pub mod object_render;
pub mod path_overlay;
pub mod platform;
pub mod qa;
pub mod selection;
pub mod shapes;
pub mod sim_driver;

use bevy::prelude::*;
use sc_sim::{PlayerId, Scenario, SimConfig};

use crate::camera::GameCameraPlugin;
use crate::combat_effects::CombatEffectsPlugin;
use crate::editor::{EditorConfig, EditorPlugin};
use crate::hud::HudPlugin;
use crate::input::{ActionInputPlugin, InputContext};
use crate::map_render::MapRenderPlugin;
use crate::object_render::ObjectRenderPlugin;
use crate::path_overlay::PathOverlayPlugin;
use crate::platform::{DisplayMode, PlatformPlugin};
use crate::qa::{QaConfig, QaPlugin};
use crate::selection::SelectionPlugin;
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

/// What the client is doing.
#[derive(Resource, Copy, Clone, Debug, PartialEq, Eq)]
pub enum ClientMode {
    /// Playing a scenario.
    Game,
    /// Editing a map (`--editor`): the sim shows the map, paused at tick 0.
    Editor,
    /// Playing the edited map from the editor, which comes back afterwards.
    PlayTest,
}

impl ClientMode {
    /// A game is running, with the game's HUD and camera: a normal game or an editor play-test.
    pub fn is_game(self) -> bool {
        self != ClientMode::Editor
    }

    /// Whose bindings the input layer applies: the game's in a game and in a play-test, the editor's in the editor.
    pub fn input_context(self) -> InputContext {
        if self.is_game() {
            InputContext::Game
        } else {
            InputContext::Editor
        }
    }
}

/// Everything [`build_app`] needs to know.
#[derive(Clone, Debug)]
pub struct AppConfig {
    /// The scenario to play: the map and the game setup, whose game options say whether debug commands are on
    /// (`ARCHITECTURE.md > Commands, events and replays`; on unless `--no-debug`). In the editor: the map to edit,
    /// and the game options its play-tests run with.
    pub scenario: Scenario,
    /// Whose view the client shows. In the editor: the slot play-tests start as (the first slot if the map has no
    /// such slot).
    pub player: PlayerId,
    pub display: DisplayMode,
    /// Physical size of the window or offscreen image.
    pub window_size: UVec2,
    pub qa: QaConfig,
    /// Run as many ticks per frame as possible.
    pub fast: bool,
    /// Edit `scenario`'s map in the map editor instead of playing it.
    pub editor: Option<EditorConfig>,
}

impl AppConfig {
    /// Player 1's view of `scenario` in a 1280×720 window, without QA, with debug commands on (the client's
    /// default, whatever the scenario's setup says).
    pub fn new(mut scenario: Scenario) -> Self {
        scenario.setup.config = SimConfig::DEBUG;
        Self {
            scenario,
            player: PlayerId(1),
            display: DisplayMode::Window,
            window_size: cli::DEFAULT_WINDOW_SIZE,
            qa: QaConfig::default(),
            fast: false,
            editor: None,
        }
    }

    /// The map editor on `scenario`'s map in a 1280×720 window, without QA. Play-tests run with debug commands on.
    pub fn editor(scenario: Scenario, editor: EditorConfig) -> Self {
        Self {
            editor: Some(editor),
            ..Self::new(scenario)
        }
    }
}

/// Builds the client app. Run it with `App::run`, or step it with `App::update` in tests (after `App::finish` and
/// `App::cleanup`, which the runners otherwise call).
pub fn build_app(config: AppConfig) -> App {
    let qa_run = config.qa.is_active();
    let editing = config.editor.is_some();
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
    .insert_resource(if editing { ClientMode::Editor } else { ClientMode::Game })
    // Mode-independent: everything a view of the map needs.
    .add_plugins((
        SimDriverPlugin {
            scenario: config.scenario.clone(),
            player: config.player,
            fast: config.fast,
            preview: editing,
        },
        ActionInputPlugin { raw_input: !qa_run },
        GameCameraPlugin,
        MapRenderPlugin,
        ObjectRenderPlugin,
        PathOverlayPlugin {
            visible: config.qa.show_paths,
        },
        CombatEffectsPlugin,
        QaPlugin { config: config.qa },
        SelectionPlugin,
    ))
    // The game mode; the editor hides the HUD except in its play-tests.
    .add_plugins(HudPlugin);
    if let Some(editor) = config.editor {
        app.add_plugins(EditorPlugin {
            config: editor,
            scenario: config.scenario,
        });
    }
    app
}
