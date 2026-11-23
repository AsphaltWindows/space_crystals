//! Things that happened during a tick, for the client (effects, sounds, UI
//! notifications). Events are informational output only: the sim never reads them.

use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::defs::ObjectKind;
use crate::ids::{ObjectId, PlayerId};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SimEvent {
    Spawned(ObjectId),
    /// The object left the game (`Sim::remove_objects`), after its `Died` event if it was destroyed.
    Removed(ObjectId),
    /// A Supply Delivery Station received a delivery.
    SuppliesDelivered {
        station: ObjectId,
        amount: u32,
    },
    /// A unit gave up a move: it found no path (`docs/movement.md > 4`,
    /// `systems/controls/behaviors.md > MovingToLocation > Failure`).
    MoveFailed {
        unit: ObjectId,
    },
    /// An attack took effect: `attacker`'s Firing phase ended on `target` (`systems/combat.md > Firing`). The
    /// positions of both, space units, for drawing the shot. Its damage follows as a [`SimEvent::Hit`].
    Attacked {
        attacker: ObjectId,
        target: ObjectId,
        from: Vec2,
        to: Vec2,
    },
    /// `target` lost `damage` HP to `attacker`'s attack (`systems/combat.md > SingleTargetDamage`).
    Hit {
        attacker: ObjectId,
        target: ObjectId,
        damage: f32,
    },
    /// The object was destroyed: its HP reached 0. What it was and where, since its `Removed` event follows in
    /// the same tick.
    Died {
        id: ObjectId,
        kind: ObjectKind,
        owner: Option<PlayerId>,
        pos: Vec2,
    },
}
