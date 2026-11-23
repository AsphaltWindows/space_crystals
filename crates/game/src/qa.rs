//! QA mode: screenshots of exact ticks, exit at a tick, and a JSON state dump at exit (`ARCHITECTURE.md > QA mode`).
//!
//! The sim is held at each screenshot tick (`SimControl::hold_at`) until the PNG is saved. A held sim draws every
//! object exactly at its tick position, and raw input is off in QA runs, so a screenshot shows the same pixels on
//! every run. The sim is also held at the exit tick, so the state dump never overshoots it.

use std::collections::VecDeque;
use std::path::{Path, PathBuf};

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
use bevy::window::ExitSystems;
use sc_sim::{Map, Object, Player, PlayerId, SimEvent, Tick};
use serde::Serialize;

use crate::ClientSystems;
use crate::camera::CameraView;
use crate::map_render::MapView;
use crate::platform::RenderSurface;
use crate::sim_driver::{LoadedScenario, LocalPlayer, RecordedInputs, SimControl, SimRes};

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
}

impl QaConfig {
    /// Any QA flag makes a reproducible run: raw input off, offscreen rendering unless a window is asked for.
    pub fn is_active(&self) -> bool {
        !self.screenshots.is_empty() || self.exit_at_tick.is_some() || self.dump_state.is_some()
    }
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
    /// Screenshots written so far.
    pub saved: Vec<PathBuf>,
    /// What went wrong. Any failure makes the run exit with an error code.
    pub failures: Vec<String>,
}

impl QaState {
    pub fn new(config: QaConfig) -> Self {
        let mut screenshots = config.screenshots;
        screenshots.sort_by_key(|s| s.tick);
        Self {
            pending: screenshots.into(),
            phase: CapturePhase::Idle,
            exit_at_tick: config.exit_at_tick,
            dump_state: config.dump_state,
            exit_requested: false,
            dumped: false,
            saved: Vec::new(),
            failures: Vec::new(),
        }
    }

    /// The tick the sim must wait at next: the next screenshot or the exit tick, whichever comes first.
    pub fn next_stop(&self) -> Option<Tick> {
        let shot = self.pending.front().map(|s| s.tick);
        match (shot, self.exit_at_tick) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (a, b) => a.or(b),
        }
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
            .add_systems(Startup, hold_at_next_stop)
            .add_systems(
                Update,
                (drive_screenshots, exit_when_done, hold_at_next_stop)
                    .chain()
                    .in_set(ClientSystems::Qa),
            )
            .add_systems(Last, dump_state_on_exit.after(ExitSystems));
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

/// Exits at the exit tick, once its screenshots are saved, after writing the state dump.
fn exit_when_done(mut qa: ResMut<QaState>, sources: DumpSources, mut exit: MessageWriter<AppExit>) {
    let tick = sources.sim.tick();
    let Some(exit_at) = qa.exit_at_tick else {
        return;
    };
    let busy = qa.phase != CapturePhase::Idle || qa.pending.front().is_some_and(|s| s.tick <= tick);
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
}

/// The `--dump-state` file: the sim state through its public accessors, plus the client's view of it.
#[derive(Serialize)]
pub struct StateDump<'a> {
    pub tick: Tick,
    pub scenario: &'a str,
    pub players: &'a [Player],
    pub objects: Vec<&'a Object>,
    /// Events of the latest tick.
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
    /// Ticks whose inputs are recorded for a replay.
    pub recorded_ticks: usize,
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
            recorded_ticks: sources.recorded.ticks.len(),
        },
        map: sim.map(),
    };
    let json = serde_json::to_string_pretty(&dump).map_err(|e| format!("can't serialize the state: {e}"))?;
    std::fs::write(path, json).map_err(|e| format!("can't write {}: {e}", path.display()))
}
