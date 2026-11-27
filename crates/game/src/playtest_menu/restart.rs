//! The playtest menu's Restart (`core/game_rules.md > Playtest menu (temporary)`: "starts the same game over from the
//! beginning: the same map, player slots, factions and computer players (with their profiles) as the current game
//! was started with"): in-process, as the map editor's play-test swaps its sim (`crate::editor::playtest`), the new
//! game as a fresh launch would start it ([`Launch`]), so that it plays out the same.
//!
//! What a game's client state is, and how a restart starts it over (slice M5-S7, from every resource a game inserts):
//! - **Here**: the sim, a new one of the launch's scenario, whose setup brings the slots' players and factions, the
//!   seed and the game options (the map explored at the start or not: `SimConfig::start_explored`); the scenario
//!   loaded; the local player, the launch's (the debug player switch, F2, may have changed it); the pending and
//!   recorded inputs; the pause, the menu's; the camera: the local player's start view, the zoom, a middle-button grab
//!   under way; the minimap's press; the computer players, new ones of the same slots and profiles
//!   (`AiPlayers::start`); the recording, written one last time and followed by the new game's
//!   (`crate::replay::record`); and the menu, closed, with the input it held.
//! - **By the new sim's generation** (`SimRes::generation`), as for a play-test: the selections, control groups and
//!   interface states (`crate::selection::ControlStates`), the players' cameras of the debug switch, the visuals of
//!   objects and of remembered records, the viewer events, the combat effects, the Shroud's clouds, the command
//!   indicators and the targeting feedback, the notices, the selected doors' arrows, the fog's and the minimap's
//!   layers (the terrain's meshes and the minimap's terrain stay: the map is the same), and (slice M5-MM2) the
//!   minimap's alerts, their pings and the latest that Space jumps to (`crate::hud::minimap::MinimapAlerts`). The
//!   outcome banner and the results screen follow `Sim::outcome`, which the new sim hasn't.
//! - **Derived every frame**: the HUD's panels, the placement ghost and its areas, the pointer.
//! - **Kept**, the client's own settings rather than the game's: the fog toggle (F3), the path overlay (P), the grid
//!   (G), the pace (`--fast`), the window, the bindings, and QA's queue, whose ticks are the running game's.

use bevy::prelude::*;
use sc_sim::Sim;

use super::{Launch, PlaytestMenu};
use crate::ai_players::AiPlayers;
use crate::camera::{CameraView, CameraZoom, GrabPan, start_view};
use crate::hud::minimap::MinimapPointer;
use crate::input::Modal;
use crate::replay::Recorder;
use crate::sim_driver::{LoadedScenario, LocalPlayer, PendingCommands, RecordedInputs, SimControl, SimRes};

/// Starts the game over as it was launched (module docs).
pub(crate) fn restart(world: &mut World) {
    let mut launch = world.resource_mut::<Launch>();
    launch.restarts += 1;
    let launch = launch.clone();
    // The game's recording, complete to its last tick, before its sim goes.
    if world.contains_resource::<Recorder>() {
        world.resource_scope(|world, mut recorder: Mut<Recorder>| {
            recorder.close(world.resource::<SimRes>());
        });
    }
    let scenario = launch.scenario.clone();
    world.resource_mut::<SimRes>().replace(Sim::new(&scenario));
    world.resource_mut::<LocalPlayer>().0 = launch.player;
    world.resource_mut::<PendingCommands>().0.clear();
    world.resource_mut::<RecordedInputs>().ticks.clear();
    let mut control = world.resource_mut::<SimControl>();
    control.paused = false;
    control.step_once = false;
    let center = start_view(&scenario, launch.player, world.resource::<SimRes>().map());
    world.resource_mut::<CameraView>().center = center;
    world.resource_mut::<CameraZoom>().0 = 1.0;
    world.resource_mut::<GrabPan>().grabbed = None;
    world.insert_resource(MinimapPointer::default());
    if world.contains_resource::<AiPlayers>() {
        let players = AiPlayers::start(&launch.ai, &scenario, world.resource::<SimRes>());
        world.insert_resource(players);
    }
    if let Some((first, names)) = &launch.recording {
        let path = names.restart_path(first, launch.restarts, scenario.name());
        let recorder = Recorder::start(path, &scenario, world.resource::<SimRes>());
        world.insert_resource(recorder);
    }
    world.insert_resource(LoadedScenario(scenario));
    world.insert_resource(PlaytestMenu::Closed);
    world.insert_resource(Modal(false));
    info!("The game restarted ({} so far)", launch.restarts);
}
