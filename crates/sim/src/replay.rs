//! Replays: the scenario (the map and the game setup, which holds the game options) plus the inputs of every tick
//! (`ARCHITECTURE.md > Commands, events and replays`), and state-hash checkpoints. The sim is deterministic, so
//! playing them back reproduces the game bit for bit; the checkpoints say whether it did ([`Replay::verify`]): a
//! replay recorded by an older build may diverge once the game's rules changed, and its checkpoints show where.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::command::TickInputs;
use crate::scenario::{Scenario, ScenarioProblem};
use crate::sim::Sim;
use crate::units::Tick;

/// How often a recording notes the state hash ([`Replay::record_checked`]): every this many ticks, 4 s of game
/// time, so playback finds a desync within 4 s of where it began. A QA setting, not a design number.
pub const CHECKPOINT_INTERVAL: Tick = 64;

/// The recorded game's state hash ([`Sim::state_hash`]) at the end of one tick.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Checkpoint {
    pub tick: Tick,
    pub hash: u64,
}

/// A recorded game. Serializes as `{ "scenario": { "map": …, "setup": … }, "inputs": […], "checkpoints": […] }`,
/// the map in its versioned file format.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Replay {
    /// The map and the game setup the game started with, game options included.
    pub scenario: Scenario,
    /// `inputs[t]` is what the step from tick `t` to tick `t + 1` applied, empty ticks
    /// included.
    pub inputs: Vec<TickInputs>,
    /// The state hash at some ticks of the recorded game, in ascending tick order and none after
    /// [`Replay::end_tick`]: playback compares the game with them ([`Replay::verify`]). [`Replay::record_checked`]
    /// notes one at tick 0 and every [`CHECKPOINT_INTERVAL`] ticks; a file without any loads as a replay that
    /// can't detect a desync.
    #[serde(default)]
    pub checkpoints: Vec<Checkpoint>,
}

/// The game played back from a replay differs from the recorded one ([`Replay::verify`]).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Desync {
    /// The tick of the checkpoint that differs.
    pub tick: Tick,
    /// The recorded state hash.
    pub expected: u64,
    /// The state hash played back.
    pub actual: u64,
    /// The checkpoint before it, if any. A playback that checked every checkpoint in order found it equal, so the
    /// game diverged in one of the ticks after it, up to [`Desync::tick`].
    pub previous: Option<Tick>,
}

impl fmt::Display for Desync {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "desync at tick {}: the state hash is {:#018x}, the recording has {:#018x}",
            self.tick, self.actual, self.expected
        )?;
        match self.previous {
            Some(previous) => write!(f, " (the checkpoint before, at tick {previous}, matched)"),
            None => write!(f, " (its first checkpoint)"),
        }
    }
}

/// Why a replay can't be played back to its end ([`Replay::run_checked`]).
#[derive(Clone, Debug, PartialEq)]
pub enum ReplayError {
    /// The scenario is invalid: [`Sim::try_new`] rejects it.
    Invalid(Vec<ScenarioProblem>),
    /// The game diverged from the recording.
    Desync(Desync),
}

impl fmt::Display for ReplayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReplayError::Invalid(problems) => {
                let problems: Vec<String> = problems.iter().map(ToString::to_string).collect();
                write!(f, "the replay's scenario is invalid: {}", problems.join("; "))
            }
            ReplayError::Desync(desync) => desync.fmt(f),
        }
    }
}

impl Replay {
    /// An empty recording of a game that starts from `scenario`.
    pub fn new(scenario: Scenario) -> Self {
        Self {
            scenario,
            inputs: Vec::new(),
            checkpoints: Vec::new(),
        }
    }

    /// The game as it starts: the sim to play this replay back on with [`Replay::step`]. Panics on an invalid
    /// scenario, like [`Sim::new`].
    pub fn start(&self) -> Sim {
        Sim::new(&self.scenario)
    }

    /// Records one step. Call it for every [`Sim::step`], in order, with the same inputs,
    /// including empty ones.
    pub fn record(&mut self, inputs: TickInputs) {
        self.inputs.push(inputs);
    }

    /// Notes the state hash of `sim` as the checkpoint of its tick. `sim` is the recorded game at the end of the
    /// recording so far ([`Replay::end_tick`]); nothing is noted if a checkpoint at that tick (or a later one)
    /// exists already, or if `sim` is at another tick.
    pub fn checkpoint(&mut self, sim: &Sim) {
        let tick = sim.tick();
        if tick == self.end_tick() && self.checkpoints.last().is_none_or(|c| c.tick < tick) {
            self.checkpoints.push(Checkpoint {
                tick,
                hash: sim.state_hash(),
            });
        }
    }

    /// The way to record a game as it runs: call it once with the game at tick 0 and `None`, then after every
    /// [`Sim::step`] with its inputs. It records them ([`Replay::record`]) and notes a checkpoint
    /// ([`Replay::checkpoint`]) at tick 0 and every [`CHECKPOINT_INTERVAL`] ticks.
    pub fn record_checked(&mut self, inputs: Option<TickInputs>, sim: &Sim) {
        if let Some(inputs) = inputs {
            self.record(inputs);
        }
        if sim.tick().is_multiple_of(CHECKPOINT_INTERVAL) {
            self.checkpoint(sim);
        }
    }

    /// The tick at which playback ends: the number of recorded steps.
    pub fn end_tick(&self) -> Tick {
        Tick::try_from(self.inputs.len()).unwrap_or(Tick::MAX)
    }

    /// The checkpoint at `tick`, if the recording has one.
    pub fn checkpoint_at(&self, tick: Tick) -> Option<Checkpoint> {
        let i = self.checkpoints.binary_search_by_key(&tick, |c| c.tick).ok()?;
        Some(self.checkpoints[i])
    }

    /// Compares `sim`, this replay played back to its current tick, with the recording: `Err` if the recording has
    /// a checkpoint at that tick and the state hash differs. Ticks without a checkpoint pass.
    pub fn verify(&self, sim: &Sim) -> Result<(), Desync> {
        let tick = sim.tick();
        let Ok(i) = self.checkpoints.binary_search_by_key(&tick, |c| c.tick) else {
            return Ok(());
        };
        let expected = self.checkpoints[i].hash;
        let actual = sim.state_hash();
        if actual == expected {
            return Ok(());
        }
        Err(Desync {
            tick,
            expected,
            actual,
            previous: i.checked_sub(1).map(|j| self.checkpoints[j].tick),
        })
    }

    /// Plays back one step: steps `sim`, a sim started like this replay's game
    /// ([`Replay::start`]), with the inputs recorded for its current tick. Returns `false`, without stepping, once
    /// `sim` has reached [`Replay::end_tick`].
    pub fn step(&self, sim: &mut Sim) -> bool {
        match self.inputs.get(sim.tick() as usize) {
            Some(inputs) => {
                sim.step(inputs);
                true
            }
            None => false,
        }
    }

    /// Plays the whole recording from the start, or lists the scenario's problems.
    pub fn try_run(&self) -> Result<Sim, Vec<ScenarioProblem>> {
        let mut sim = Sim::try_new(&self.scenario)?;
        for inputs in &self.inputs {
            sim.step(inputs);
        }
        Ok(sim)
    }

    /// Plays the whole recording from the start, checking every checkpoint on the way ([`Replay::verify`]): the
    /// game at its end, or the scenario's problems, or the first checkpoint that differs.
    pub fn run_checked(&self) -> Result<Sim, ReplayError> {
        let mut sim = Sim::try_new(&self.scenario).map_err(ReplayError::Invalid)?;
        self.verify(&sim).map_err(ReplayError::Desync)?;
        for inputs in &self.inputs {
            sim.step(inputs);
            self.verify(&sim).map_err(ReplayError::Desync)?;
        }
        Ok(sim)
    }

    /// Plays the whole recording from the start. Panics on an invalid scenario, like
    /// [`Sim::new`].
    pub fn run(&self) -> Sim {
        let mut sim = self.start();
        for inputs in &self.inputs {
            sim.step(inputs);
        }
        sim
    }
}
