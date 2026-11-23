//! Owns the simulation and steps it (`ARCHITECTURE.md > Client architecture > Sim driver`).
//!
//! One `FixedUpdate` run is one tick at [`TICKS_PER_SECOND`] (`core/scale.md > SimulationFrame`); nothing else
//! steps the sim. Player intent arrives as `PlayerCommand`s in [`PendingCommands`], which the step drains into that
//! tick's `TickInputs`. Every tick's inputs are kept in [`RecordedInputs`] for replays, and the tick's
//! `SimEvent`s are forwarded as [`SimEventMessage`]s right after the step.

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

/// The scenario the current sim started from: replays need it, and so do the player slots' camera starts, which
/// the sim itself doesn't keep.
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

/// Whether the step system may advance the sim.
#[derive(Resource, Default, Debug)]
pub struct SimControl {
    pub paused: bool,
    /// Step up to this tick, then wait. QA holds the sim at each screenshot tick until the capture is saved.
    pub hold_at: Option<Tick>,
    /// Draw objects exactly at their tick positions, never between ticks (`--fast`).
    pub exact_render: bool,
}

impl SimControl {
    pub fn may_step(&self, tick: Tick) -> bool {
        !self.paused && self.hold_at.is_none_or(|hold| tick < hold)
    }
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

/// `--fast`: virtual time runs this much faster than real time, so each frame owes the fixed loop far more ticks than
/// it can run in [`FAST_FRAME_BUDGET`].
const FAST_TIME_SCALE: f32 = 100_000.0;
/// The most virtual time one `--fast` frame may add (`Time<Virtual>::max_delta`).
const FAST_MAX_DELTA: Duration = Duration::from_secs(3600);
/// Wall-clock time a `--fast` frame spends stepping before it presents a frame.
pub const FAST_FRAME_BUDGET: Duration = Duration::from_millis(50);

pub struct SimDriverPlugin {
    pub scenario: Scenario,
    pub player: PlayerId,
    /// Run as many ticks per frame as fit in [`FAST_FRAME_BUDGET`] instead of 16 per second.
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
                exact_render: self.fast,
                ..default()
            })
            .insert_resource(RenderAlpha(1.0))
            .add_message::<SimEventMessage>()
            .add_systems(FixedUpdate, step_sim.in_set(StepSim))
            .add_systems(Update, update_render_alpha.in_set(ClientSystems::SyncSim));
        if self.fast {
            app.init_resource::<FastFrame>()
                .add_systems(Startup, speed_up_virtual_time)
                .add_systems(PreUpdate, begin_fast_frame)
                .add_systems(FixedUpdate, end_fast_steps.after(StepSim));
        }
    }
}

/// Restarts the client on `scenario`: a fresh sim, no pending or recorded inputs. Views rebuild because
/// [`SimRes::generation`] changes.
pub fn load_scenario(world: &mut World, scenario: Scenario) {
    world.resource_mut::<SimRes>().replace(Sim::new(&scenario));
    world.resource_mut::<PendingCommands>().0.clear();
    world.resource_mut::<RecordedInputs>().ticks.clear();
    world.insert_resource(LoadedScenario(scenario));
}

/// One tick: drain the pending commands into `TickInputs`, step, record, forward the events.
fn step_sim(
    mut sim: ResMut<SimRes>,
    mut pending: ResMut<PendingCommands>,
    mut recorded: ResMut<RecordedInputs>,
    control: Res<SimControl>,
    mut events: MessageWriter<SimEventMessage>,
) {
    if !control.may_step(sim.tick()) {
        return;
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
/// `Update`, unless the sim stands still or exact rendering is on.
fn update_render_alpha(
    fixed: Res<Time<Fixed>>,
    control: Res<SimControl>,
    sim: Res<SimRes>,
    mut alpha: ResMut<RenderAlpha>,
) {
    let a = if control.exact_render || !control.may_step(sim.tick()) {
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

fn speed_up_virtual_time(mut time: ResMut<Time<Virtual>>) {
    time.set_max_delta(FAST_MAX_DELTA);
    time.set_relative_speed(FAST_TIME_SCALE);
}

fn begin_fast_frame(mut frame: ResMut<FastFrame>) {
    frame.deadline = Some(Instant::now() + FAST_FRAME_BUDGET);
}

/// Ends this frame's fixed loop once the budget is spent or the sim may not step: dropping the owed time makes
/// `Time<Fixed>::expend` fail, so the loop stops after this run.
fn end_fast_steps(frame: Res<FastFrame>, sim: Res<SimRes>, control: Res<SimControl>, mut fixed: ResMut<Time<Fixed>>) {
    let out_of_time = frame.deadline.is_none_or(|deadline| Instant::now() >= deadline);
    if out_of_time || !control.may_step(sim.tick()) {
        let owed = fixed.overstep();
        fixed.discard_overstep(owed);
    }
}
