//! Play-testing (F5): the edited map runs in game mode, in-process, as the chosen player slot; Esc or F5 returns to
//! the editor with the document, its undo history and the editor's view as they were.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use sc_sim::Sim;

use super::actions::PlayTestRequest;
use super::doc::EditorDoc;
use super::state::EditorStatus;
use super::tools::EditorTools;
use crate::ClientMode;
use crate::camera::{CameraView, CameraZoom, start_center};
use crate::sim_driver::{LoadedScenario, LocalPlayer, PendingCommands, RecordedInputs, SimControl, SimRes};

/// The editor's view while a play-test runs, to come back to.
#[derive(Resource, Default, Debug)]
pub struct PlayTestState {
    saved_view: Option<(Vec2, f32)>,
}

/// The client state a play-test switches.
#[derive(SystemParam)]
pub struct GameState<'w> {
    sim: ResMut<'w, SimRes>,
    loaded: ResMut<'w, LoadedScenario>,
    player: ResMut<'w, LocalPlayer>,
    control: ResMut<'w, SimControl>,
    pending: ResMut<'w, PendingCommands>,
    recorded: ResMut<'w, RecordedInputs>,
    view: ResMut<'w, CameraView>,
    zoom: ResMut<'w, CameraZoom>,
}

/// Starts or ends a play-test when one is requested.
pub fn switch_play_test(
    mut request: ResMut<PlayTestRequest>,
    mut mode: ResMut<ClientMode>,
    mut state: ResMut<PlayTestState>,
    doc: Res<EditorDoc>,
    tools: Res<EditorTools>,
    mut status: ResMut<EditorStatus>,
    mut game: GameState,
) {
    if !std::mem::take(&mut request.0) {
        return;
    }
    match *mode {
        ClientMode::Editor => {
            let scenario = doc.scenario();
            // The game only runs valid scenarios.
            let sim = match Sim::try_new(scenario) {
                Ok(sim) => sim,
                Err(problems) => {
                    status.error(format!(
                        "Can't play-test: {} problem(s), see Problems ({})",
                        problems.len(),
                        problems[0]
                    ));
                    return;
                }
            };
            state.saved_view = Some((game.view.center, game.zoom.0));
            game.sim.replace(sim);
            game.loaded.0 = scenario.clone();
            game.player.0 = tools.slot;
            game.pending.0.clear();
            game.recorded.ticks.clear();
            game.control.paused = false;
            game.zoom.0 = 1.0;
            game.view.center = start_center(scenario, tools.slot, &scenario.map);
            *mode = ClientMode::PlayTest;
            status.info(format!("Play-testing as player {}: Esc or F5 returns", tools.slot.0));
        }
        ClientMode::PlayTest => {
            let scenario = doc.scenario();
            game.sim.replace(Sim::preview(scenario).0);
            game.loaded.0 = scenario.clone();
            game.pending.0.clear();
            game.recorded.ticks.clear();
            game.control.paused = true;
            if let Some((center, zoom)) = state.saved_view.take() {
                game.view.center = center;
                game.zoom.0 = zoom;
            }
            *mode = ClientMode::Editor;
            status.info("Back in the editor");
        }
        ClientMode::Game => {}
    }
}
