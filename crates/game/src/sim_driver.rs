//! Owns the simulation and steps it (`ARCHITECTURE.md > Client architecture > Sim driver`).
//!
//! One `FixedUpdate` run is one tick at [`TICKS_PER_SECOND`] (`core/scale.md > SimulationFrame`); nothing else
//! steps the sim. Once the game is over (`Sim::outcome`), it isn't stepped again: "the game stops"
//! (`core/game_rules.md > End of Game`). Player intent arrives as `PlayerCommand`s in [`PendingCommands`], which
//! the step drains into that tick's `TickInputs`. Every tick's inputs are kept in [`RecordedInputs`] for replays,
//! and the tick's `SimEvent`s are forwarded as [`SimEventMessage`]s right after the step.

use std::ops::Deref;
use std::time::{Duration, Instant};

use bevy::prelude::*;
use sc_sim::{PlayerCommand, PlayerId, Scenario, Sim, SimEvent, TICKS_PER_SECOND, Tick, TickInputs};

use crate::ClientSystems;

/// The simulation, as a Bevy resource. Read it through `Deref<Target = Sim>`; only the step system mutates it.
#[derive(Resource)]
pub struct SimRes {
    sim: Sim,
    generation: u32,
}

impl SimRes {
    pub fn new(sim: Sim) -> Self {
        Self { sim, generation: 0 }
    }

    pub fn sim(&self) -> &Sim {
        &self.sim
    }

    /// Changes whenever the whole sim is replaced ([`SimRes::replace`]). Views then rebuild from scratch instead of
    /// diffing, because object ids of the new sim can repeat those of the old one.
    pub fn generation(&self) -> u32 {
        self.generation
    }

    /// Swaps in a different simulation, e.g. when the map editor loads or edits a scenario.
    pub fn replace(&mut self, sim: Sim) {
        self.sim = sim;
        self.generation = self.generation.wrapping_add(1);
    }
}

impl Deref for SimRes {
    type Target = Sim;

    fn deref(&self) -> &Sim {
        &self.sim
    }
}

/// The scenario the current sim started from (the map and the game setup, game options included): with
/// [`RecordedInputs`], a replay. The player slots' camera starts come from it too: the sim doesn't keep them.
#[derive(Resource, Clone, Debug)]
pub struct LoadedScenario(pub Scenario);

/// Whose view the client shows (`--player`).
#[derive(Resource, Copy, Clone, Debug, PartialEq, Eq)]
pub struct LocalPlayer(pub PlayerId);

/// Commands waiting for the next tick, in submission order.
#[derive(Resource, Default, Debug)]
pub struct PendingCommands(pub Vec<PlayerCommand>);

/// The `TickInputs` of every tick so far, empty ticks included: with [`LoadedScenario`], a complete replay.
#[derive(Resource, Default, Debug)]
pub struct RecordedInputs {
    /// `ticks[i]` holds the inputs applied on tick `i + 1`.
    pub ticks: Vec<TickInputs>,
}

impl RecordedInputs {
    /// The inputs applied on `tick` (tick 0 is the initial state and has none).
    pub fn inputs_of(&self, tick: Tick) -> Option<&TickInputs> {
        self.ticks.get(tick.checked_sub(1)? as usize)
    }
}

/// Whether the step system may advance the sim. Never once the game is over ([`SimControl::may_step`]).
#[derive(Resource, Default, Debug)]
pub struct SimControl {
    pub paused: bool,
    /// One step while paused (replay playback's single step): the step system clears it once it has stepped.
    pub step_once: bool,
    /// Step up to this tick, then wait. QA holds the sim at each screenshot tick until the capture is saved.
    pub hold_at: Option<Tick>,
}

impl SimControl {
    /// Whether `sim` may advance a tick now: not paused (or one step asked for), not held, and its game not over
    /// (`core/game_rules.md > End of Game`: "the game stops", design change 0101).
    pub fn may_step(&self, sim: &Sim) -> bool {
        (!self.paused || self.step_once) && self.hold_at.is_none_or(|hold| sim.tick() < hold) && sim.outcome().is_none()
    }
}

/// How fast the sim runs, changeable at any time (replay playback's speeds).
#[derive(Resource, Copy, Clone, Debug, PartialEq)]
pub enum Pace {
    /// This many times [`TICKS_PER_SECOND`] ticks per second of real time: 1 is the game's speed.
    Scaled(f32),
    /// As many ticks per frame as fit in [`FAST_FRAME_BUDGET`] (`--fast`). Objects are drawn exactly at their
    /// tick positions, never between ticks.
    Max,
}

/// This frame's interpolation factor between each object's `prev_pos` (0) and `pos` (1). It is 1 while the sim
/// stands still, so a paused or held frame shows the exact tick state.
#[derive(Resource, Copy, Clone, Debug, PartialEq)]
pub struct RenderAlpha(pub f32);

/// A `SimEvent` of the tick that just ran, forwarded for effects, sounds and UI.
#[derive(Message, Clone, Debug, PartialEq)]
pub struct SimEventMessage {
    pub tick: Tick,
    pub event: SimEvent,
}

/// The `FixedUpdate` system that advances the sim.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct StepSim;

/// [`Pace::Max`]: virtual time runs this much faster than real time, so each frame owes the fixed loop far more ticks
/// than it can run in [`FAST_FRAME_BUDGET`].
const FAST_TIME_SCALE: f32 = 100_000.0;
/// The most virtual time one [`Pace::Max`] frame may add (`Time<Virtual>::max_delta`).
const FAST_MAX_DELTA: Duration = Duration::from_secs(3600);
/// The most virtual time a frame may add at a [`Pace::Scaled`] pace: Bevy's default.
const SCALED_MAX_DELTA: Duration = Duration::from_millis(250);
/// Wall-clock time a [`Pace::Max`] frame spends stepping before it presents a frame.
pub const FAST_FRAME_BUDGET: Duration = Duration::from_millis(50);

pub struct SimDriverPlugin {
    pub scenario: Scenario,
    pub player: PlayerId,
    /// Start at [`Pace::Max`]: as many ticks per frame as fit in [`FAST_FRAME_BUDGET`] instead of 16 per second.
    pub fast: bool,
    /// Map editor: show the scenario even if it is invalid ([`Sim::preview`]), paused at tick 0. Otherwise the
    /// scenario must be valid.
    pub preview: bool,
}

impl Plugin for SimDriverPlugin {
    fn build(&self, app: &mut App) {
        let sim = if self.preview {
            Sim::preview(&self.scenario).0
        } else {
            Sim::new(&self.scenario)
        };
        app.insert_resource(Time::<Fixed>::from_hz(f64::from(TICKS_PER_SECOND)))
            .insert_resource(SimRes::new(sim))
            .insert_resource(LoadedScenario(self.scenario.clone()))
            .insert_resource(LocalPlayer(self.player))
            .init_resource::<PendingCommands>()
            .init_resource::<RecordedInputs>()
            .insert_resource(SimControl {
                paused: self.preview,
                ..default()
            })
            .insert_resource(if self.fast { Pace::Max } else { Pace::Scaled(1.0) })
            .init_resource::<FastFrame>()
            .insert_resource(RenderAlpha(1.0))
            .add_message::<SimEventMessage>()
            .add_systems(Startup, apply_pace)
            .add_systems(PreUpdate, begin_fast_frame.run_if(at_max_pace))
            .add_systems(FixedUpdate, (step_sim.in_set(StepSim), limit_fixed_loop.after(StepSim)))
            .add_systems(PostUpdate, apply_pace)
            .add_systems(Update, update_render_alpha.in_set(ClientSystems::SyncSim));
    }
}

/// Restarts the client on `scenario`: a fresh sim with the game options of the running one, no pending or
/// recorded inputs. Views rebuild because [`SimRes::generation`] changes.
pub fn load_scenario(world: &mut World, mut scenario: Scenario) {
    scenario.setup.config = world.resource::<SimRes>().config();
    world.resource_mut::<SimRes>().replace(Sim::new(&scenario));
    world.resource_mut::<PendingCommands>().0.clear();
    world.resource_mut::<RecordedInputs>().ticks.clear();
    world.insert_resource(LoadedScenario(scenario));
}

/// One tick: drain the pending commands into `TickInputs`, step, record, forward the events. A single step
/// ([`SimControl::step_once`]) is used up.
fn step_sim(
    mut sim: ResMut<SimRes>,
    mut pending: ResMut<PendingCommands>,
    mut recorded: ResMut<RecordedInputs>,
    mut control: ResMut<SimControl>,
    mut events: MessageWriter<SimEventMessage>,
) {
    if !control.may_step(&sim) {
        return;
    }
    if control.step_once {
        control.step_once = false;
    }
    let inputs = TickInputs {
        commands: std::mem::take(&mut pending.0),
    };
    sim.sim.step(&inputs);
    recorded.ticks.push(inputs);
    let tick = sim.tick();
    for event in sim.events() {
        events.write(SimEventMessage {
            tick,
            event: event.clone(),
        });
    }
}

/// `ARCHITECTURE.md > Client architecture > Visuals > Interpolation`: `Time<Fixed>::overstep_fraction()`, read in
/// `Update`, unless the sim stands still (paused, or held) or runs at [`Pace::Max`], which draws exact ticks.
fn update_render_alpha(
    fixed: Res<Time<Fixed>>,
    control: Res<SimControl>,
    pace: Res<Pace>,
    sim: Res<SimRes>,
    mut alpha: ResMut<RenderAlpha>,
) {
    let a = if *pace == Pace::Max || control.paused || !control.may_step(&sim) {
        1.0
    } else {
        fixed.overstep_fraction().clamp(0.0, 1.0)
    };
    alpha.set_if_neq(RenderAlpha(a));
}

#[derive(Resource, Default)]
struct FastFrame {
    deadline: Option<Instant>,
}

fn at_max_pace(pace: Res<Pace>) -> bool {
    *pace == Pace::Max
}

/// Sets the speed of virtual time, which drives the fixed loop, when the [`Pace`] changes. It runs in `PostUpdate`,
/// after the systems that change the pace (the playback keys), so that the next frame's time update already goes at
/// the new pace; a pace changed between frames is caught by [`limit_fixed_loop`]. A scaled pace drops the time the
/// loop still owes.
fn apply_pace(pace: Res<Pace>, mut time: ResMut<Time<Virtual>>, mut fixed: ResMut<Time<Fixed>>) {
    if !pace.is_changed() {
        return;
    }
    match *pace {
        Pace::Scaled(factor) => {
            time.set_max_delta(SCALED_MAX_DELTA);
            time.set_relative_speed(factor);
            let owed = fixed.overstep();
            fixed.discard_overstep(owed);
        }
        Pace::Max => {
            time.set_max_delta(FAST_MAX_DELTA);
            time.set_relative_speed(FAST_TIME_SCALE);
        }
    }
}

fn begin_fast_frame(mut frame: ResMut<FastFrame>) {
    frame.deadline = Some(Instant::now() + FAST_FRAME_BUDGET);
}

/// Ends or trims this frame's fixed loop by dropping time it still owes, which makes `Time<Fixed>::expend` fail. At
/// [`Pace::Max`] the loop ends once the frame's budget is spent or the sim may not step. At a scaled pace the loop
/// never owes more than a frame may add ([`SCALED_MAX_DELTA`] of scaled time), so a frame whose time was still taken
/// at `Max` (the pace changed between frames) runs a few ticks, not a burst of them.
fn limit_fixed_loop(
    frame: Res<FastFrame>,
    pace: Res<Pace>,
    sim: Res<SimRes>,
    control: Res<SimControl>,
    mut fixed: ResMut<Time<Fixed>>,
) {
    let owed = fixed.overstep();
    let keep = match *pace {
        Pace::Max => {
            let out_of_time = frame.deadline.is_none_or(|deadline| Instant::now() >= deadline);
            if out_of_time || !control.may_step(&sim) {
                Duration::ZERO
            } else {
                owed
            }
        }
        Pace::Scaled(factor) => owed.min(SCALED_MAX_DELTA.mul_f32(factor.max(0.0))),
    };
    fixed.discard_overstep(owed - keep);
}
