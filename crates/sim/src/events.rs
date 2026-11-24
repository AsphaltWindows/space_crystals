//! Things that happened during a tick, for the client (effects, sounds, UI
//! notifications). Events are informational output only: the sim never reads them.

use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::defs::ObjectKind;
use crate::ids::{ObjectId, PlayerId};
use crate::placement::PlacementCheck;
use crate::production::Refusal;
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
    /// `target` lost `damage` HP and `shield_damage` Shields to `attacker`'s attack (`systems/combat.md >
    /// SingleTargetDamage`, `> Shields > Shield Damage Resolution`; `combat::resolve`): Shields that take all of it
    /// leave the HP untouched (`damage` 0).
    Hit {
        attacker: ObjectId,
        target: ObjectId,
        damage: f32,
        /// Optional in files (0), so that the states saved before it existed still load.
        #[serde(default)]
        shield_damage: f32,
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
    /// `constructor` finished its CurrentConstruction, a `kind`: it now holds it ready to place (ReadyToPlace;
    /// `factions/gdo/structures.md > DeploymentCenter`, `crate::construction`).
    ConstructionCompleted {
        constructor: ObjectId,
        kind: ObjectKind,
    },
    /// `constructor` gave up its CurrentConstruction or its ReadyToPlace, a `kind`, and `refund` Space Crystals went
    /// back to its owner (`factions/gdo/structures.md > DeploymentCenter > Cancellation`): cancelled by a command (in
    /// full while constructing, 75% rounded down once ready), or refunded as the constructor left the game (design
    /// change 0084; before its `Removed` event).
    ConstructionCancelled {
        constructor: ObjectId,
        kind: ObjectKind,
        refund: u32,
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
    /// changed in between, e.g. a unit walked onto the footprint. A Warp In refused the same way says so too: its
    /// Beacon is the `constructor`, and stays a Beacon, and `origin` is that of the footprint derived from it
    /// (`crate::transform::warp_in_footprint`, design change 0121; `Sim::warp_in_check`, slice M3-E).
    PlacementRefused {
        constructor: ObjectId,
        kind: ObjectKind,
        origin: TilePos,
        orientation: BuildingOrientation,
        problems: PlacementCheck,
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
    /// `worker` finished unloading at `drop_off`, and `amount` Space Crystals entered its owner's bank
    /// (`factions/colonists/economy.md > Egress Node > Drop Off`; `crate::economy::gathering`).
    ResourcesDelivered {
        worker: ObjectId,
        drop_off: ObjectId,
        amount: u32,
    },
    /// Object `id` changed its kind in place, from `from` to `to`, and is still the same object (plan decision T15,
    /// `Sim::change_kind`): a Drone deploying into a Beacon, a Beacon warping in a building, a Legionnaire converting
    /// its weapon, or one of these cancelled. The client draws the object anew and keeps it selected as its new kind
    /// (slice M3-E's response, `ARCHITECTURE.md > Client architecture`).
    KindChanged {
        id: ObjectId,
        from: ObjectKind,
        to: ObjectKind,
    },
    /// Object `id` finished transforming into its kind `kind` (`crate::transform`): a building warped in from a
    /// Beacon (`factions/colonists/economy.md > Warp In`) now works.
    TransformCompleted {
        id: ObjectId,
        kind: ObjectKind,
    },
    /// Object `id` gave up transforming into a `kind` by a command, and `refund` Space Crystals went back to its
    /// owner (`factions/colonists/economy.md > Warp In > Cancel Warp In`: 75% of the cost, rounded down). Its
    /// `KindChanged` back to what it was follows.
    TransformCancelled {
        id: ObjectId,
        kind: ObjectKind,
        refund: u32,
    },
}
