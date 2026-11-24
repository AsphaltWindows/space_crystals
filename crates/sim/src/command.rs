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
use crate::orders::OrderKind;
use crate::scenario::SpawnAt;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Command {
    /// `systems/controls/commands.md > Unit Command`: the player orders their own `units` (`docs/movement.md >
    /// 3`). With `queue` (Shift) the order is appended to each unit's queue; without, it replaces the queue. Ids
    /// that no longer exist and repeated ids are dropped; the command is rejected if no unit remains or one
    /// belongs to someone else.
    Order {
        units: Vec<ObjectId>,
        kind: OrderKind,
        queue: bool,
    },
    /// Development / QA only — never offered to players in a real match.
    Debug(DebugCommand),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum DebugCommand {
    /// Spawn an object for any owner, bypassing costs and placement rules. A unit's box must be clear of blocked
    /// tiles and of the other units of its layer; a blocking footprint must not cover a unit.
    Spawn {
        kind: ObjectKind,
        owner: Option<PlayerId>,
        at: SpawnAt,
    },
    /// Remove an object outright.
    Remove { id: ObjectId },
    /// Teleport an object (units only make sense; tile objects keep their placement). A unit's box must be clear
    /// there, as for a spawn; a moving unit plans its path again from there.
    SetPosition { id: ObjectId, pos: Vec2 },
    /// Turn auto-targeting off or on for every unit in the game (`Sim::auto_targeting`; on when a game starts).
    /// While it is off no unit acquires a target by itself: not when idle, on Hold Position, on an AttackMove or on
    /// a Patrol (`systems/controls/behaviors.md > BaseAutoTargeting`). Attack orders work as ever, and a target a
    /// unit already engaged stays engaged. For tests and QA runs that look at movement between the units of
    /// different players without the fight that would follow.
    SetAutoTargeting { on: bool },
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
