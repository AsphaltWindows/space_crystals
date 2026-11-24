//! Things that happened during a tick, for the client (effects, sounds, UI
//! notifications). Events are informational output only: the sim never reads them.

use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::defs::ObjectKind;
use crate::ids::{ObjectId, PlayerId};
use crate::placement::PlacementCheck;
use crate::scenario::BuildingOrientation;
use crate::units::TilePos;

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
    /// `constructor` finished its CurrentConstruction, a `kind`: it now holds it ready to place (ReadyToPlace;
    /// `factions/gdo/structures.md > DeploymentCenter`, `crate::construction`).
    ConstructionCompleted {
        constructor: ObjectId,
        kind: ObjectKind,
    },
    /// `constructor` placed its ready building, which is now the object `structure` (its `Spawned` event comes
    /// first).
    StructurePlaced {
        constructor: ObjectId,
        structure: ObjectId,
    },
    /// A Place that the placement check refused when it was applied (plan decision T4): `constructor` keeps its
    /// ready `kind`, which was to go with the top-left tile of its footprint at `origin`, turned by `orientation`,
    /// and `problems` says why. The one command the sim reports refusing (`Sim::validate` gives
    /// `Reject::Unplaceable`); the client validates a Place before it sends it, so this tells it that the game
    /// changed in between, e.g. a unit walked onto the footprint.
    PlacementRefused {
        constructor: ObjectId,
        kind: ObjectKind,
        origin: TilePos,
        orientation: BuildingOrientation,
        problems: PlacementCheck,
    },
}
