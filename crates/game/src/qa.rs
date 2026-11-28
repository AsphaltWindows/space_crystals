//! QA mode: screenshots of exact ticks, exit at a tick, and a JSON state dump at exit (`docs/architecture/qa-mode.md > QA mode`).
//!
//! The sim is held at each screenshot tick (`SimControl::hold_at`) until the PNG is saved. A held sim draws every
//! object exactly at its tick position, and raw input is off in QA runs, so a screenshot shows the same pixels on
//! every run. The sim is also held at the exit tick, so the state dump never overshoots it, and at each tick of the
//! `--input-script` ([`input_script`]) until its step has been applied and a frame more, in which its input is read.
//!
//! Once the game is over (`Sim::outcome`), the sim stands at its last tick for good (`SimControl::may_step`): the
//! screenshots, input steps and exit scheduled after it happen at that tick ([`stop_at_game_end`]), so a run never
//! waits for a tick that can't come.

pub mod input_script;

use std::collections::VecDeque;
use std::path::{Path, PathBuf};

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
use bevy::window::ExitSystems;
use sc_sim::vision::LastKnown;
use sc_sim::{Command, Map, Object, ObjectId, Player, PlayerCommand, PlayerId, SimEvent, Tick};
use serde::{Deserialize, Serialize};

pub use input_script::{InputStep, load_input_script};

use crate::ClientSystems;
use crate::camera::CameraView;
use crate::fog::FogOfWar;
use crate::input::ReadActions;
use crate::map_render::MapView;
use crate::platform::RenderSurface;
use crate::replay::{Playback, Recorder};
use crate::selection::ControlStates;
use crate::sim_driver::{LoadedScenario, LocalPlayer, PendingCommands, RecordedInputs, SimControl, SimRes, StepSim};

/// Frames to wait at a screenshot tick before capturing. With synchronous pipeline compilation the first frame is
/// already complete; the margin covers window setup and text layout.
pub const SETTLE_FRAMES: u32 = 3;
/// Frames to wait for a capture before giving up on it.
pub const CAPTURE_TIMEOUT_FRAMES: u32 = 600;

#[derive(Clone, Debug, PartialEq)]
pub struct ScreenshotRequest {
    pub tick: Tick,
    pub path: PathBuf,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct QaConfig {
    pub screenshots: Vec<ScreenshotRequest>,
    pub exit_at_tick: Option<Tick>,
    pub dump_state: Option<PathBuf>,
    /// `--commands`: player commands to apply at given ticks.
    pub commands: Vec<TimedCommand>,
    /// `--show-paths`: the path overlay is on from the start (QA runs ignore the keyboard). A view option: it
    /// doesn't make a QA run by itself.
    pub show_paths: bool,
    /// `--no-fog`: the fog of war is off from the start (`crate::fog::FogOfWar`). A view option, like
    /// `show_paths`.
    pub no_fog: bool,
    /// `--input-script`: the virtual keyboard and mouse at given ticks.
    pub input_script: Vec<InputStep>,
}

impl QaConfig {
    /// Any QA flag makes a reproducible run: raw input off, offscreen rendering unless a window is asked for.
    pub fn is_active(&self) -> bool {
        !self.screenshots.is_empty()
            || self.exit_at_tick.is_some()
            || self.dump_state.is_some()
            || !self.commands.is_empty()
            || !self.input_script.is_empty()
    }
}

/// One entry of a `--commands` file: `command` from `player`, applied by the step that advances the sim to `tick`
/// (the first is tick 1). E.g. `{"tick": 2, "player": 1, "command": {"Order": {"units": [18], "kind": {"Move":
/// {"Point": [480, 288]}}, "queue": false}}}`. Commands go through the sim's gate like any other.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TimedCommand {
    pub tick: Tick,
    pub player: PlayerId,
    pub command: Command,
}

/// Reads a `--commands` file: a JSON list of [`TimedCommand`]s, any order (the same tick keeps file order).
pub fn load_commands(path: &Path) -> Result<Vec<TimedCommand>, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("can't read {}: {e}", path.display()))?;
    let commands: Vec<TimedCommand> =
        serde_json::from_str(&text).map_err(|e| format!("{} is not a list of timed commands: {e}", path.display()))?;
    if let Some(early) = commands.iter().find(|c| c.tick == 0) {
        return Err(format!(
            "{}: a command at tick 0 ({:?}) never applies: the first step is tick 1",
            path.display(),
            early.command
        ));
    }
    Ok(commands)
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum CapturePhase {
    Idle,
    /// At a screenshot tick, waiting for the frame to settle.
    Settling {
        frames_left: u32,
    },
    /// The screenshot entity is out; Bevy despawns it after its observers saved the PNG.
    Capturing {
        entity: Entity,
        frames: u32,
    },
}

#[derive(Resource, Debug)]
pub struct QaState {
    /// Screenshots still to take, by tick.
    pending: VecDeque<ScreenshotRequest>,
    phase: CapturePhase,
    exit_at_tick: Option<Tick>,
    dump_state: Option<PathBuf>,
    exit_requested: bool,
    dumped: bool,
    /// `--commands` still to apply, by tick.
    commands: VecDeque<TimedCommand>,
    /// `--input-script` steps still to apply, by tick.
    input_steps: VecDeque<InputStep>,
    /// (Slice M6-S6a) The tick of the input step applied in this frame or the last, while its input may still be read:
    /// the sim waits there a frame after each step ([`input_script`]).
    input_tick: Option<Tick>,
    /// Whether this frame applied an input step.
    input_applied: bool,
    /// Screenshots written so far.
    pub saved: Vec<PathBuf>,
    /// What went wrong. Any failure makes the run exit with an error code.
    pub failures: Vec<String>,
}

impl QaState {
    pub fn new(config: QaConfig) -> Self {
        let mut screenshots = config.screenshots;
        screenshots.sort_by_key(|s| s.tick);
        let mut commands = config.commands;
        commands.sort_by_key(|c| c.tick); // Stable: a tick's commands keep their order.
        let mut input_steps = config.input_script;
        input_steps.sort_by_key(|s| s.tick);
        Self {
            pending: screenshots.into(),
            phase: CapturePhase::Idle,
            exit_at_tick: config.exit_at_tick,
            dump_state: config.dump_state,
            exit_requested: false,
            dumped: false,
            commands: commands.into(),
            input_steps: input_steps.into(),
            input_tick: None,
            input_applied: false,
            saved: Vec::new(),
            failures: Vec::new(),
        }
    }

    /// The tick the sim must wait at next: the next screenshot, input step or the exit tick, whichever comes
    /// first, or (slice M6-S6a) the tick of the last input step while its input may still be read ([`input_script`]).
    pub fn next_stop(&self) -> Option<Tick> {
        let shot = self.pending.front().map(|s| s.tick);
        let step = self.input_steps.front().map(|s| s.tick);
        [shot, step, self.exit_at_tick, self.input_tick]
            .into_iter()
            .flatten()
            .min()
    }

    fn pop_due(&mut self, tick: Tick) -> Vec<ScreenshotRequest> {
        let mut due = Vec::new();
        while self.pending.front().is_some_and(|s| s.tick == tick) {
            due.extend(self.pending.pop_front());
        }
        due
    }

    fn exit_code(&self) -> AppExit {
        if self.failures.is_empty() {
            AppExit::Success
        } else {
            AppExit::error()
        }
    }
}

pub struct QaPlugin {
    pub config: QaConfig,
}

impl Plugin for QaPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(QaState::new(self.config.clone()))
            .add_systems(FixedUpdate, inject_commands.before(StepSim))
            .add_systems(PreUpdate, input_script::apply_input_script.before(ReadActions))
            .add_systems(Startup, hold_at_next_stop)
            .add_systems(
                Update,
                (
                    stop_at_game_end,
                    drive_screenshots,
                    exit_when_done,
                    input_script::input_read,
                    hold_at_next_stop,
                )
                    .chain()
                    .in_set(ClientSystems::Qa),
            )
            .add_systems(Last, dump_state_on_exit.after(ExitSystems));
    }
}

/// `--commands`: hands the commands of the next tick to the step that applies them (a held sim keeps them pending).
/// The computer players' commands come after them (`crate::ai_players`).
pub(crate) fn inject_commands(mut qa: ResMut<QaState>, sim: Res<SimRes>, mut pending: ResMut<PendingCommands>) {
    let next = sim.tick() + 1;
    while let Some(c) = qa.commands.pop_front_if(|c| c.tick <= next) {
        pending.0.push(PlayerCommand {
            player: c.player,
            command: c.command,
        });
    }
}

/// The game is over (`core/game_rules.md > End of Game`): the sim steps no more, so every stop scheduled after its last
/// tick comes down to it. The screenshots are taken there, the input steps applied there, and the run exits there.
fn stop_at_game_end(mut qa: ResMut<QaState>, sim: Res<SimRes>) {
    if sim.outcome().is_none() {
        return;
    }
    let end = sim.tick();
    let qa = &mut *qa;
    let mut moved = false;
    let mut bring = |tick: &mut Tick| {
        if *tick > end {
            *tick = end;
            moved = true;
        }
    };
    qa.pending.iter_mut().for_each(|s| bring(&mut s.tick));
    qa.input_steps.iter_mut().for_each(|s| bring(&mut s.tick));
    if let Some(exit) = &mut qa.exit_at_tick {
        bring(exit);
    }
    if moved {
        info!("QA: the game is over at tick {end}: what was due after it happens at tick {end}");
    }
}

fn hold_at_next_stop(qa: Res<QaState>, mut control: ResMut<SimControl>) {
    let hold = qa.next_stop();
    if control.hold_at != hold {
        control.hold_at = hold;
    }
}

fn drive_screenshots(
    mut commands: Commands,
    mut qa: ResMut<QaState>,
    sim: Res<SimRes>,
    surface: Res<RenderSurface>,
    screenshots: Query<(), With<Screenshot>>,
) {
    let tick = sim.tick();
    let qa = &mut *qa;
    qa.phase = match qa.phase {
        CapturePhase::Idle => match qa.pending.front() {
            Some(next) if next.tick < tick => {
                // Unreachable while the hold works: the sim waits at every screenshot tick.
                for missed in qa.pop_due(next.tick) {
                    qa.failures
                        .push(format!("missed the screenshot of tick {}", missed.tick));
                }
                CapturePhase::Idle
            }
            Some(next) if next.tick == tick => CapturePhase::Settling {
                frames_left: SETTLE_FRAMES,
            },
            _ => CapturePhase::Idle,
        },
        CapturePhase::Settling { frames_left: 0 } => {
            let mut shot = commands.spawn(Screenshot(surface.target.clone()));
            for request in qa.pending.iter().take_while(|s| s.tick == tick) {
                // A file left by an earlier run must not pass for this run's screenshot.
                remove_stale(&request.path);
                shot.observe(save_to_disk(request.path.clone()));
            }
            CapturePhase::Capturing {
                entity: shot.id(),
                frames: 0,
            }
        }
        CapturePhase::Settling { frames_left } => CapturePhase::Settling {
            frames_left: frames_left - 1,
        },
        CapturePhase::Capturing { entity, frames } if screenshots.contains(entity) => {
            if frames < CAPTURE_TIMEOUT_FRAMES {
                CapturePhase::Capturing {
                    entity,
                    frames: frames + 1,
                }
            } else {
                commands.entity(entity).despawn();
                for request in qa.pop_due(tick) {
                    qa.failures
                        .push(format!("screenshot {} timed out", request.path.display()));
                }
                CapturePhase::Idle
            }
        }
        CapturePhase::Capturing { .. } => {
            for request in qa.pop_due(tick) {
                if request.path.is_file() {
                    info!("QA: saved the screenshot of tick {tick} to {}", request.path.display());
                    qa.saved.push(request.path);
                } else {
                    qa.failures
                        .push(format!("screenshot {} was not written", request.path.display()));
                }
            }
            CapturePhase::Idle
        }
    };
}

fn remove_stale(path: &Path) {
    if let Err(error) = std::fs::remove_file(path)
        && error.kind() != std::io::ErrorKind::NotFound
    {
        warn!("QA: can't remove the old {}: {error}", path.display());
    }
}

/// Exits at the exit tick, once its screenshots are saved and (slice M6-S6a) its input read, after writing the state
/// dump.
fn exit_when_done(mut qa: ResMut<QaState>, sources: DumpSources, mut exit: MessageWriter<AppExit>) {
    let tick = sources.sim.tick();
    let Some(exit_at) = qa.exit_at_tick else {
        return;
    };
    let busy =
        qa.phase != CapturePhase::Idle || qa.pending.front().is_some_and(|s| s.tick <= tick) || qa.input_tick.is_some();
    if qa.exit_requested || tick < exit_at || busy {
        return;
    }
    for request in std::mem::take(&mut qa.pending) {
        qa.failures.push(format!(
            "screenshot of tick {} not taken: the run ends at tick {exit_at}",
            request.tick
        ));
    }
    if let Some(path) = qa.dump_state.clone() {
        qa.dumped = true;
        match write_state_dump(&path, &sources) {
            Ok(()) => info!("QA: dumped the state of tick {tick} to {}", path.display()),
            Err(error) => qa.failures.push(error),
        }
    }
    for failure in &qa.failures {
        error!("QA: {failure}");
    }
    qa.exit_requested = true;
    exit.write(qa.exit_code());
}

/// Writes the dump when the app exits some other way, e.g. its window was closed.
fn dump_state_on_exit(mut exits: MessageReader<AppExit>, mut qa: ResMut<QaState>, sources: DumpSources) {
    if exits.read().count() == 0 || qa.dumped {
        return;
    }
    qa.dumped = true;
    if let Some(path) = qa.dump_state.clone()
        && let Err(error) = write_state_dump(&path, &sources)
    {
        error!("QA: {error}");
    }
}

/// Everything the state dump reads.
#[derive(SystemParam)]
pub struct DumpSources<'w> {
    sim: Res<'w, SimRes>,
    scenario: Res<'w, LoadedScenario>,
    player: Res<'w, LocalPlayer>,
    recorded: Res<'w, RecordedInputs>,
    camera: Option<Res<'w, CameraView>>,
    map_view: Option<Res<'w, MapView>>,
    control: Option<Res<'w, ControlStates>>,
    fog: Option<Res<'w, FogOfWar>>,
    playback: Option<Res<'w, Playback>>,
    recorder: Option<Res<'w, Recorder>>,
}

/// The `--dump-state` file: the sim state through its public accessors, plus the client's view of it.
#[derive(Serialize)]
pub struct StateDump<'a> {
    pub tick: Tick,
    pub scenario: &'a str,
    pub players: &'a [Player],
    pub objects: Vec<&'a Object>,
    /// Events of the latest tick, as the sim has them, for every player: QA tooling, the one reader of the sim's events
    /// in the client beside the filter that every view reads them through (`crate::viewer_events`).
    pub events: &'a [SimEvent],
    pub client: ClientDump,
    pub map: &'a Map,
}

#[derive(Serialize)]
pub struct ClientDump {
    pub local_player: PlayerId,
    /// Camera center in sim space units.
    pub camera_center: Option<Vec2>,
    pub grid_visible: bool,
    /// Whether the fog of war is on (`--no-fog`, the debug toggle).
    pub fog_of_war: bool,
    /// Ticks whose inputs are recorded for a replay.
    pub recorded_ticks: usize,
    /// The local player's selection.
    pub selection: Vec<ObjectId>,
    /// The local player's records of the structures and resources it has seen (`PlayerVision::records`), by id:
    /// what its Explored tiles show of them.
    pub last_known: Vec<LastKnown>,
    /// The file the game is recorded to, if it is (`crate::replay::record`).
    pub recording: Option<PathBuf>,
    /// `--replay`: the replay played back (`crate::replay::playback`).
    pub replay: Option<ReplayDump>,
}

/// The replay played back, in the state dump.
#[derive(Serialize)]
pub struct ReplayDump {
    pub file: PathBuf,
    /// Its last tick, where the playback pauses.
    pub end_tick: Tick,
    pub checkpoints: usize,
    /// The first checkpoint the game didn't match (`sc_sim::Desync`), if one didn't.
    pub desync: Option<String>,
}

fn write_state_dump(path: &Path, sources: &DumpSources) -> Result<(), String> {
    let sim = sources.sim.sim();
    let dump = StateDump {
        tick: sim.tick(),
        scenario: sources.scenario.0.name(),
        players: sim.players(),
        objects: sim.objects().collect(),
        events: sim.events(),
        client: ClientDump {
            local_player: sources.player.0,
            camera_center: sources.camera.as_ref().map(|c| c.center),
            grid_visible: sources.map_view.as_ref().is_some_and(|v| v.grid_visible),
            fog_of_war: sources.fog.as_ref().is_some_and(|f| f.enabled),
            recorded_ticks: sources.recorded.ticks.len(),
            selection: sources
                .control
                .as_ref()
                .map_or_else(Vec::new, |c| c.selection(sources.player.0).ids().collect()),
            last_known: sim
                .vision()
                .player(sources.player.0)
                .map_or_else(Vec::new, |v| v.records().cloned().collect()),
            recording: sources.recorder.as_ref().map(|r| r.path.clone()),
            replay: sources.playback.as_ref().map(|p| ReplayDump {
                file: p.file.clone(),
                end_tick: p.end_tick(),
                checkpoints: p.replay.checkpoints.len(),
                desync: p.desync.map(|d| d.to_string()),
            }),
        },
        map: sim.map(),
    };
    let json = serde_json::to_string_pretty(&dump).map_err(|e| format!("can't serialize the state: {e}"))?;
    std::fs::write(path, json).map_err(|e| format!("can't write {}: {e}", path.display()))
}
