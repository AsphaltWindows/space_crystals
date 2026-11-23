//! Player input to the simulation.
//!
//! The client never mutates sim state directly: every player action becomes a
//! [`PlayerCommand`], collected into the [`TickInputs`] for the next tick. The sim
//! validates each command (ownership, cost, …) and ignores invalid ones. Recording
//! the `TickInputs` of every tick is a complete replay.

use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::defs::ObjectKind;
use crate::ids::{ObjectId, PlayerId};
use crate::scenario::SpawnAt;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Command {
    /// Development / QA only — never offered to players in a real match.
    Debug(DebugCommand),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum DebugCommand {
    /// Spawn an object for any owner, bypassing costs and placement rules.
    Spawn {
        kind: ObjectKind,
        owner: Option<PlayerId>,
        at: SpawnAt,
    },
    /// Remove an object outright.
    Remove { id: ObjectId },
    /// Teleport an object (units only make sense; tile objects keep their placement).
    SetPosition { id: ObjectId, pos: Vec2 },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlayerCommand {
    pub player: PlayerId,
    pub command: Command,
}

/// Everything players did that the sim applies at the start of one tick, in order.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct TickInputs {
    pub commands: Vec<PlayerCommand>,
}

impl TickInputs {
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
}
