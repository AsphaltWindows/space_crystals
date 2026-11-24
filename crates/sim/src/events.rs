//! Things that happened during a tick, for the client (effects, sounds, UI
//! notifications). Events are informational output only: the sim never reads them.

use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::defs::ObjectKind;
use crate::ids::{ObjectId, PlayerId};
use crate::production::Refusal;

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
    /// `plate`'s last load emptied its Space Crystals patch: from now on it mines the residual rate, and the patch
    /// stays under it with 0 left until the plate is gone (`factions/gdo/economy.md > ExtractionPlate > Depleted
    /// patch under a plate`).
    PatchDepleted {
        patch: ObjectId,
        plate: ObjectId,
    },
    /// `producer` took a `Train` command: a unit of `kind` was paid for and joined its BuildQueue
    /// (`factions/gdo/structures.md > Barracks`).
    ProductionQueued {
        producer: ObjectId,
        kind: ObjectKind,
    },
    /// `producer` received a `Train` command that other producers took, and didn't queue the unit itself (design
    /// change 0083: for the "insufficient resources" feedback).
    ProductionRefused {
        producer: ObjectId,
        kind: ObjectKind,
        reason: Refusal,
    },
    /// A unit of `kind` left `producer`'s BuildQueue unbuilt and `refund` Space Crystals went back to its owner:
    /// cancelled by a command, or refunded as the producer left the game (design changes 0084 and 0085).
    ProductionCancelled {
        producer: ObjectId,
        kind: ObjectKind,
        refund: u32,
    },
    /// `producer` released `unit` into its exit doorway (`core/entities.md > Structure Type > Exit Doorway`), after
    /// the unit's `Spawned` event.
    UnitProduced {
        producer: ObjectId,
        unit: ObjectId,
    },
    /// `player` was defeated in this tick: they own no structure any more (`core/game_rules.md > Defeat Condition`,
    /// plan decision T8), and their remaining units became neutral (`> On Defeat`, design change 0101).
    PlayerDefeated {
        player: PlayerId,
    },
    /// The game ended in this tick, after its `PlayerDefeated` events (`core/game_rules.md > End of Game`):
    /// `winners` see Victory, every other player Defeat (`crate::Sim::outcome`, [`crate::economy::Outcome`]).
    GameOver {
        winners: Vec<PlayerId>,
    },
}
