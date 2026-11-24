//! Replay playback (`--replay`, `ARCHITECTURE.md > Replays and golden hashes`): the client plays a recorded game
//! instead of a live one. Each step applies the recording's inputs of its tick and nothing else, so the local player
//! can't give commands: the command UI's input doesn't run, and whatever else is pending is dropped. The playback
//! pauses at the end of the recording.
//!
//! The game is checked against the recording's checkpoints at tick 0 and after every step (`Replay::verify`). The
//! first desync is logged as an error, shown in the top bar (`crate::hud`) and fails a QA run; the playback goes on.
//!
//! Controls ([`InputAction`], placeholder keys: the design has no replays yet): pause and resume, play one tick,
//! slower and faster through [`SPEEDS`]. The player switch (F2) and the fog toggle (F3) work as in a game with debug
//! commands, even if the recorded game had none: they only change what the replay shows. Their own systems do it,
//! the selection's (`crate::selection`) and the fog's (`crate::fog`), which allow them while a [`Playback`] runs.

use std::path::PathBuf;

use bevy::prelude::*;
use sc_sim::{Desync, Replay, Tick};

use crate::ClientSystems;
use crate::command_ui::CommandUiSystems;
use crate::input::{ActionState, InputAction};
use crate::qa::QaState;
use crate::sim_driver::{Pace, PendingCommands, SimControl, SimRes, StepSim};

/// The playback speeds, slowest first: half, once, twice and four times the game's speed, and as fast as the client
/// can step (`Pace::Max`). Playback starts at once, or as fast as possible with `--fast`. Placeholders.
pub const SPEEDS: [Pace; 5] = [
    Pace::Scaled(0.5),
    Pace::Scaled(1.0),
    Pace::Scaled(2.0),
    Pace::Scaled(4.0),
    Pace::Max,
];

/// The replay being played back.
#[derive(Resource, Debug)]
pub struct Playback {
    /// The replay file.
    pub file: PathBuf,
    pub replay: Replay,
    /// The first checkpoint that the game played back didn't match.
    pub desync: Option<Desync>,
}

impl Playback {
    /// The last tick of the recording, where the playback pauses.
    pub fn end_tick(&self) -> Tick {
        self.replay.end_tick()
    }

    /// The top bar's note on the playback, at `tick`: its speed, whether it is paused or over, and the desync if
    /// there was one.
    pub fn status(&self, tick: Tick, paused: bool, pace: Pace) -> String {
        let speed = match pace {
            Pace::Scaled(factor) => format!("x{factor}"),
            Pace::Max => "max".to_string(),
        };
        let state = if tick >= self.end_tick() {
            ", end"
        } else if paused {
            ", paused"
        } else {
            ""
        };
        let mut status = format!("Replay {speed}{state}");
        if let Some(desync) = self.desync {
            status.push_str(&format!(" - DESYNC at tick {} (see the log)", desync.tick));
        }
        status
    }
}

/// Plays `replay`, read from `file`, back. The sim must start from the replay's scenario (`AppConfig::playback`).
pub struct PlaybackPlugin {
    pub file: PathBuf,
    pub replay: Replay,
}

impl Plugin for PlaybackPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Playback {
            file: self.file.clone(),
            replay: self.replay.clone(),
            desync: None,
        })
        .configure_sets(Update, CommandUiSystems::Input.run_if(not(resource_exists::<Playback>)))
        .add_systems(Startup, check_checkpoint)
        .add_systems(
            FixedUpdate,
            (feed_recorded_inputs.before(StepSim), check_checkpoint.after(StepSim)),
        )
        .add_systems(Update, playback_keys.in_set(ClientSystems::Actions));
    }
}

/// Gives the step the recorded inputs of its tick, in place of anything else pending. At the end of the recording
/// there are none, and the playback pauses; so it does at the end of the game, where the sim stops
/// (`SimControl::may_step`), whatever a recording made outside the client holds after it.
fn feed_recorded_inputs(
    sim: Res<SimRes>,
    playback: Res<Playback>,
    mut pending: ResMut<PendingCommands>,
    mut control: ResMut<SimControl>,
) {
    let inputs = playback.replay.inputs.get(sim.tick() as usize);
    match inputs.filter(|_| sim.outcome().is_none()) {
        Some(inputs) => pending.0.clone_from(&inputs.commands),
        None => {
            pending.0.clear();
            if !control.paused || control.step_once {
                control.paused = true;
                control.step_once = false;
            }
        }
    }
}

/// Compares the game with the recording's checkpoint of its tick, if it has one, once per tick. Only the first
/// desync is reported: after it, the game differs anyway.
fn check_checkpoint(
    sim: Res<SimRes>,
    mut playback: ResMut<Playback>,
    mut qa: ResMut<QaState>,
    mut checked: Local<Option<Tick>>,
) {
    let tick = sim.tick();
    if *checked == Some(tick) || playback.desync.is_some() {
        return;
    }
    *checked = Some(tick);
    if let Err(desync) = playback.replay.verify(&sim) {
        let message = format!("replay {}: {desync}", playback.file.display());
        error!("{message}");
        qa.failures.push(message);
        playback.desync = Some(desync);
    }
}

/// The playback controls.
fn playback_keys(
    actions: Res<ActionState>,
    sim: Res<SimRes>,
    playback: Res<Playback>,
    mut control: ResMut<SimControl>,
    mut pace: ResMut<Pace>,
) {
    let over = sim.tick() >= playback.end_tick() || sim.outcome().is_some();
    if actions.just_pressed(InputAction::ReplayPause) && !over {
        control.paused = !control.paused;
    }
    if actions.just_pressed(InputAction::ReplayStep) && !over {
        control.paused = true;
        control.step_once = true;
    }
    let speed = SPEEDS.iter().position(|s| *s == *pace).unwrap_or(1);
    if actions.just_pressed(InputAction::ReplaySlower) {
        pace.set_if_neq(SPEEDS[speed.saturating_sub(1)]);
    }
    if actions.just_pressed(InputAction::ReplayFaster) {
        pace.set_if_neq(SPEEDS[(speed + 1).min(SPEEDS.len() - 1)]);
    }
}
