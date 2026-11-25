//! Things that happened during a tick, for the client (effects, sounds, UI
//! notifications). Events are informational output only: the sim never reads them.

use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::defs::{ObjectKind, Resource};
use crate::economy::ArmyResource;
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
    /// A pick-up took the last Supply from `station` (`factions/syndicate/units/agent.md > Gathering > Supplies`,
    /// design change 0256 point 3: "An empty station: the owner gets a notification (notifications not designed yet),
    /// and every Agent picking up there stands next to it indefinitely, keeping its job, waiting for the next supply
    /// delivery"): one event for each player whose workers gather there, `owner`, in id order. The client shows a
    /// placeholder for the notification (slice M4-I, `crate::economy::stations`).
    SupplyStationEmpty {
        station: ObjectId,
        owner: PlayerId,
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
    /// (`factions/gdo/structures/barracks.md`).
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
    /// `producer`'s unit at the front of its queue can't start: its owner's army cap, `reason` (the GDO's Unit
    /// Control, the Colonists' Beacon Capacity), has no room for it, and the producer waits at 0% until there is
    /// (design change 0143: "notification (system TBD)"). Placeholder for that notification: raised once, in the tick
    /// the wait begins (`sim/producers/step.rs`).
    ProductionWaiting {
        producer: ObjectId,
        reason: ArmyResource,
    },
    /// `producer` released `unit` into its exit doorway (`core/entities.md > Structure Type > Exit Doorway`), after
    /// the unit's `Spawned` event.
    UnitProduced {
        producer: ObjectId,
        unit: ObjectId,
    },
    /// `constructor` finished its CurrentConstruction, a `kind`: it now holds it ready to place (ReadyToPlace;
    /// `factions/gdo/structures/deployment_center.md`, `crate::construction`).
    ConstructionCompleted {
        constructor: ObjectId,
        kind: ObjectKind,
    },
    /// `constructor` gave up its CurrentConstruction or its ReadyToPlace, a `kind`, and `refund` Space Crystals went
    /// back to its owner (`factions/gdo/structures/deployment_center.md > Cancellation`): cancelled by a command (in
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
    /// `winners` see Victory, every other player Defeat; none for a draw, which everyone sees (design change 0163;
    /// `crate::Sim::outcome`, [`crate::economy::Outcome`]).
    GameOver {
        winners: Vec<PlayerId>,
    },
    /// `worker` finished unloading at `drop_off`, and `amount` of `resource` entered its owner's bank
    /// (`factions/colonists/economy.md > Egress Node > Drop Off`; `crate::economy::gathering`): Space Crystals, or
    /// Supplies at a Tunnel's Supplies hatch (slice M4-I).
    ResourcesDelivered {
        worker: ObjectId,
        drop_off: ObjectId,
        amount: u32,
        #[serde(default)]
        resource: Resource,
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
    /// Beacon (`factions/colonists/economy.md > Warp In`) now works, a Beacon deployed from a Drone (`> Beacon
    /// Deployment`, `crate::deploy`) is one for good.
    TransformCompleted {
        id: ObjectId,
        kind: ObjectKind,
    },
    /// Object `id` gave up transforming into a `kind` by a command, and `refund` Space Crystals went back to its
    /// owner (`factions/colonists/economy.md > Warp In > Cancel Warp In`: 75% of the cost, rounded down; nothing for
    /// a deployment, which costs nothing, `> Cancel Deployment`). Its `KindChanged` back to what it was follows.
    TransformCancelled {
        id: ObjectId,
        kind: ObjectKind,
        refund: u32,
    },
    /// Drone `drone`'s Deploy failed at the end of its walk (`factions/colonists/economy.md > Beacon Deployment >
    /// Placement`: "If the site is invalid on arrival, the order is cancelled and the Drone stops and idles";
    /// `crate::deploy`, slice M3-F): the Beacon was to go on the footprint whose top-left tile is `origin`.
    /// `problems` are those the check of the site found on arrival, on the truth (design change 0120: a structure
    /// hidden in the fog, say; 0118: a ground unit on the footprint), and none if the Drone stopped short of the
    /// footprint (design change 0117: "If it stops without overlapping the footprint, the order fails"). For the
    /// feedback line (plan decision on notifications: M2's line until they are designed).
    DeployFailed {
        drone: ObjectId,
        origin: TilePos,
        problems: PlacementCheck,
    },
    /// `unit` walked into `tunnel`'s entry path and left the map for its owner's Tunnel Network
    /// (`factions/syndicate/tunnels.md > Tunnel Network > Walking in`; `crate::network`, `Sim::step_entries`). It
    /// keeps its id, and is no longer among the sim's objects: no `Removed` follows. The client drops it from the
    /// selection and keeps it in its control groups (design change 0249).
    EnteredNetwork {
        unit: ObjectId,
        tunnel: ObjectId,
    },
    /// `unit` came out of the Tunnel Network on `tunnel`'s exit path, and is among the sim's objects again, with the
    /// id it had (`> EjectMenu`; `Sim::step_ejections`): ejected by its player, or sent out by a surface rally point
    /// (`factions/syndicate/structures/README.md > Rally Point Behavior`). No `Spawned` comes with it.
    LeftNetwork {
        unit: ObjectId,
        tunnel: ObjectId,
    },
    /// `unit`'s Enter order on `tunnel` ended without it going in (`> Tunnel Network > Walking in`: "if the Tunnel a
    /// unit is heading for is destroyed, or its tier doesn't suit the unit, the Enter order ends and the unit idles";
    /// "if the way is genuinely blocked, the normal Failure rule applies"): its walk ended short of the entry path, or
    /// the Tunnel no longer suits it. For the feedback line, and for counting those that give up.
    EnterFailed {
        unit: ObjectId,
        tunnel: ObjectId,
    },
    /// `unit`'s Attack order on `target` failed: the target is inside the unit's MinRange and the unit can't move to
    /// back off (`systems/controls/behaviors.md > AttackingObject > Target inside MinRange`, design change 0182: "the
    /// order fails with feedback ("Target too close"; same channel as other refusals)"; `Sim::step_too_close`). The
    /// unit stands, with its next queued order, if any, from the next tick.
    TargetTooClose {
        unit: ObjectId,
        target: ObjectId,
    },
}
