//! Replays: the scenario plus the inputs of every tick (`ARCHITECTURE.md > Commands,
//! events and replays`). The sim is deterministic, so playing them back reproduces the
//! game bit for bit; compare [`Sim::state_hash`] to check.

use serde::{Deserialize, Serialize};

use crate::command::TickInputs;
use crate::scenario::{Scenario, ScenarioProblem};
use crate::sim::Sim;
use crate::units::Tick;

/// A recorded game. Serializes as `{ "scenario": …, "inputs": […] }`, the scenario in
/// its versioned file format.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Replay {
    pub scenario: Scenario,
    /// `inputs[t]` is what the step from tick `t` to tick `t + 1` applied, empty ticks
    /// included.
    pub inputs: Vec<TickInputs>,
}

impl Replay {
    /// An empty recording of a game that starts from `scenario`.
    pub fn new(scenario: Scenario) -> Self {
        Self {
            scenario,
            inputs: Vec::new(),
        }
    }

    /// Records one step. Call it for every [`Sim::step`], in order, with the same inputs,
    /// including empty ones.
    pub fn record(&mut self, inputs: TickInputs) {
        self.inputs.push(inputs);
    }

    /// The tick at which playback ends: the number of recorded steps.
    pub fn end_tick(&self) -> Tick {
        Tick::try_from(self.inputs.len()).unwrap_or(Tick::MAX)
    }

    /// Plays back one step: steps `sim`, a sim started from this replay's scenario, with
    /// the inputs recorded for its current tick. Returns `false`, without stepping, once
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

    /// Plays the whole recording from the start. Panics on an invalid scenario, like
    /// [`Sim::new`].
    pub fn run(&self) -> Sim {
        let mut sim = Sim::new(&self.scenario);
        for inputs in &self.inputs {
            sim.step(inputs);
        }
        sim
    }
}
