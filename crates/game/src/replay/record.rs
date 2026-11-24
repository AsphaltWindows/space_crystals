//! Recording the game (`ARCHITECTURE.md > Replays and golden hashes`). The client records every game it plays, but
//! not the map editor, its play-tests or a replay's playback: the scenario the game started from, the inputs of every
//! step, and a state-hash checkpoint every `CHECKPOINT_INTERVAL` ticks (`Replay::record_checked`). The file is
//! written when the game starts, every [`AUTOSAVE_TICKS`] ticks of game time and when the app exits, each time with
//! a checkpoint of the latest tick, so a crash loses a few seconds at most. Where it goes is `AppConfig::record`:
//! `--record <file>`, else `replays/<UTC time>_<map>.json` (`super::recording_path`); QA runs record only with
//! `--record`, and `--no-record` turns recording off.
//!
//! A recording belongs to the sim it started with: should the sim be replaced (`SimRes::generation`), the recording
//! is written once more and stops.
//!
//! When the game is over (`Sim::outcome`, `core/game_rules.md > End of Game`: "the replay is saved"), the recording
//! is complete: it is written at once, with a checkpoint of the last tick, and the client steps no more
//! (`sim_driver::SimControl::may_step`). Playing it back ends where the game did, in the same outcome.

use std::path::PathBuf;

use bevy::prelude::*;
use bevy::window::ExitSystems;
use sc_sim::{Replay, Tick};

use super::save_replay;
use crate::sim_driver::{LoadedScenario, RecordedInputs, SimRes, StepSim};

/// How often the recording is written while the game runs: every this many ticks of game time (10 s). A QA setting,
/// not a design number.
pub const AUTOSAVE_TICKS: Tick = 160;

/// The game being recorded.
#[derive(Resource, Debug)]
pub struct Recorder {
    /// The file the recording is written to.
    pub path: PathBuf,
    replay: Replay,
    /// The `SimRes::generation` of the recorded sim.
    generation: u32,
    /// The sim was replaced: the recording ends where it was.
    stopped: bool,
    /// The game is over: the recording is complete.
    finished: bool,
    /// The last tick of the recording when the file was last written, if it was.
    saved_at: Option<Tick>,
    /// The latest error writing the file, so that a failing disk is reported once, not at every save.
    error: Option<String>,
}

impl Recorder {
    /// The recording so far.
    pub fn replay(&self) -> &Replay {
        &self.replay
    }

    /// The last tick of the recording when the file was last written, if it was.
    pub fn saved_at(&self) -> Option<Tick> {
        self.saved_at
    }

    /// Whether the recording is complete: its game is over.
    pub fn is_finished(&self) -> bool {
        self.finished
    }

    fn stop(&mut self, reason: &str) {
        warn!(
            "The recording {} stops at tick {}: {reason}",
            self.path.display(),
            self.replay.end_tick()
        );
        self.stopped = true;
    }

    /// Whether the file is due: the first time, after [`AUTOSAVE_TICKS`] more ticks, and with anything new when the
    /// app exits, the recording stopped or the game is over.
    fn save_due(&self, exiting: bool) -> bool {
        let end = self.replay.end_tick();
        match self.saved_at {
            None => true,
            Some(saved) if saved == end => false,
            Some(saved) => exiting || self.stopped || self.finished || end >= saved.saturating_add(AUTOSAVE_TICKS),
        }
    }
}

/// Records the game to `path`. Add it after `SimDriverPlugin`, whose game it records from tick 0.
pub struct RecordPlugin {
    pub path: PathBuf,
}

impl Plugin for RecordPlugin {
    fn build(&self, app: &mut App) {
        let world = app.world();
        let (Some(sim), Some(scenario)) = (world.get_resource::<SimRes>(), world.get_resource::<LoadedScenario>())
        else {
            panic!("RecordPlugin records the game of SimDriverPlugin: add it after that one");
        };
        let mut replay = Replay::new(scenario.0.clone());
        replay.record_checked(None, sim);
        let recorder = Recorder {
            path: self.path.clone(),
            replay,
            generation: sim.generation(),
            stopped: false,
            finished: false,
            saved_at: None,
            error: None,
        };
        app.insert_resource(recorder)
            .add_systems(FixedUpdate, record_step.after(StepSim))
            .add_systems(Last, save_recording.after(ExitSystems));
    }
}

/// Records the step that just ran with the inputs it applied (`RecordedInputs`), and a checkpoint at every
/// `CHECKPOINT_INTERVAL`-th tick. Runs after every step, so it sees each one; the step that ends the game completes
/// the recording.
fn record_step(sim: Res<SimRes>, recorded: Res<RecordedInputs>, mut recorder: ResMut<Recorder>) {
    if recorder.stopped {
        return;
    }
    if sim.generation() != recorder.generation {
        recorder.stop("the game was replaced");
        return;
    }
    let (tick, end) = (sim.tick(), recorder.replay.end_tick());
    if tick == end {
        return; // No step ran: paused or held.
    }
    match recorded.inputs_of(tick).filter(|_| tick == end + 1) {
        Some(inputs) => {
            let inputs = inputs.clone();
            recorder.replay.record_checked(Some(inputs), &sim);
            if sim.outcome().is_some() && !recorder.finished {
                info!(
                    "The game is over at tick {tick}: the recording {} is complete",
                    recorder.path.display()
                );
                recorder.finished = true;
            }
        }
        None => recorder.stop(&format!("tick {tick} came without the steps since tick {end}")),
    }
}

/// Writes the file when it is due ([`Recorder::save_due`]), with a checkpoint of the latest tick.
fn save_recording(mut exits: MessageReader<AppExit>, sim: Res<SimRes>, mut recorder: ResMut<Recorder>) {
    let exiting = exits.read().count() > 0;
    if !recorder.save_due(exiting) {
        return;
    }
    let recorder = &mut *recorder;
    if !recorder.stopped && sim.generation() == recorder.generation {
        recorder.replay.checkpoint(&sim);
    }
    let end = recorder.replay.end_tick();
    if recorder.saved_at.is_none() {
        info!("Recording the game to {}", recorder.path.display());
    }
    // A failed save waits for the next one too, rather than trying again every frame.
    recorder.saved_at = Some(end);
    match save_replay(&recorder.path, &recorder.replay) {
        Ok(()) => {
            if recorder.error.take().is_some() {
                info!("Recording: {} written again", recorder.path.display());
            }
        }
        Err(error) => {
            if recorder.error.as_ref() != Some(&error) {
                error!("Recording: {error}");
            }
            recorder.error = Some(error);
        }
    }
}
