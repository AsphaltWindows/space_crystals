//! Player input to the simulation.
//!
//! The client never mutates sim state directly: every player action becomes a
//! [`PlayerCommand`], collected into the [`TickInputs`] for the next tick. The sim
//! validates each command (ownership, cost, …) and ignores invalid ones. Recording
//! the `TickInputs` of every tick is a complete replay.

use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::construction::ConstructionCommand;
use crate::defs::ObjectKind;
use crate::ids::{ObjectId, PlayerId};
use crate::orders::OrderKind;
use crate::production::ProductionCommand;
use crate::scenario::{BuildingOrientation, SpawnAt};
use crate::units::TilePos;

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
    /// The player's `units` use `ability` at once (`factions/gdo/units/peacekeeper.md > Riot Shield (toggled
    /// ability)`). An ability is not an order: it doesn't enter or replace a unit's command queue, Shift means nothing
    /// to it, and the unit goes on with its orders meanwhile (design change 0023: a mode isn't a task). Ids that no
    /// longer exist, repeated ids and units without the ability are dropped; the command is rejected if no unit
    /// remains or one belongs to someone else.
    Ability { units: Vec<ObjectId>, ability: Ability },
    /// The player's producing structures among `objects` take `command` (`factions/gdo/structures/barracks.md >
    /// ObjectInterfaceState[Barracks]`; `production`): train a unit, cancel the last one queued, or set the rally
    /// point. Every producer of the selection receives it (`systems/controls/selection_and_hud.md > SelectionGroup >
    /// No smart-casting`). Ids that no longer exist, repeated ids and the player's objects that produce nothing are
    /// dropped; the command is rejected if no producer remains or one belongs to someone else.
    Production {
        objects: Vec<ObjectId>,
        command: ProductionCommand,
    },
    /// The player's constructors `objects` (the Deployment Center, the Extraction Facility) carry out `command`
    /// (`crate::construction`; `factions/gdo/structures/deployment_center.md`, `factions/gdo/economy.md >
    /// ExtractionFacility`), each in id order that can. Ids that no longer exist, repeated ids and objects that
    /// construct nothing are dropped; the command is rejected if none of the rest can carry it out, or one of the
    /// objects belongs to someone else. A Place names one constructor.
    Construction {
        objects: Vec<ObjectId>,
        command: ConstructionCommand,
    },
    /// The player's Tunnel `tunnel` carries out `command` (`factions/syndicate/tunnels.md > Tunnel
    /// ObjectInterfaceState`, plan decision T36; `crate::network`). Rejected if it doesn't exist, belongs to someone
    /// else, isn't a Tunnel in operation, or can't carry the command out.
    Tunnel { tunnel: ObjectId, command: TunnelCommand },
    /// (M5-F1) The player's buildings that hold units among `objects` take `command` (`crate::holding`;
    /// `factions/cults/structures/armory.md > ObjectInterfaceState[Armory]`: "A: Eject All"). Every one of the
    /// selection receives it (`systems/controls/selection_and_hud.md > SelectionGroup > No smart-casting`). Ids that no
    /// longer exist, repeated ids and the player's objects that hold no units are dropped; the command is rejected if
    /// none of the rest can carry it out, or one of the objects belongs to someone else.
    Holding {
        objects: Vec<ObjectId>,
        command: HoldCommand,
    },
    /// Development / QA only — never offered to players in a real match.
    Debug(DebugCommand),
}

/// A command to a Tunnel ([`Command::Tunnel`], plan decision T36).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TunnelCommand {
    /// `factions/syndicate/tunnels.md > EjectMenu`: "ejects one unit of that type from this Tunnel's Side A", the one
    /// of `kind` "that has been inside the network longest" among those free to leave (design changes 0245 and
    /// 0251). The click reserves that unit on the Tunnel's eject queue, and it comes out of the exit path once the
    /// units queued before it are out and the exit spot is clear (0250).
    Eject { kind: ObjectKind },
    /// `factions/syndicate/tunnels.md > AwaitingPlacement (Expansion)` (slice M4-F): "Left-click valid location:
    /// places expansion, begins construction": the Tunnel builds an expansion of `kind` with the top-left tile of its
    /// footprint at `origin`, turned by `orientation`, in its Tunnel Area, as its one operation, paid at once (design
    /// change 0254: "Paid when the ghost is placed"; `sim/expansions.rs`).
    Expand {
        kind: ObjectKind,
        origin: TilePos,
        orientation: BuildingOrientation,
    },
    /// `factions/syndicate/tunnels.md > Tunnel ObjectInterfaceState`: "**Q** Upgrade Tunnel" (slice M4-G), "upgrades to
    /// next tier (CommandIssuingTransition). Costs Supplies per upgrade cost formula. Unavailable if already Tier 3 or
    /// if currently performing an operation": the Tunnel upgrades to the tier above its own as its one operation, paid
    /// at once (`factions/syndicate/tunnels.md > Cost Scaling > Upgrade Costs`: "**Paid when the upgrade is
    /// ordered**"), if its larger area is clear by the truth (`sim/upgrades.rs`).
    Upgrade,
    /// `factions/syndicate/tunnels.md > Tunnel ObjectInterfaceState`: "**X** Cancel (upgrade, construction or the
    /// current operation)" (design change 0254: "The Tunnel's X cancels whatever operation it is running"): an
    /// expansion under construction is cancelled as by its own Cancel Construction (slice M4-F), an upgrade refunded at
    /// the current top price (slice M4-G, design changes 0232 and 0284: "Cancel Upgrade refunds the current top
    /// price").
    Cancel,
}

/// A command to buildings that hold units ([`Command::Holding`], slice M5-F1).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum HoldCommand {
    /// `factions/cults/structures/armory.md > Eject All`: "Immediate command that ejects all stored Recruits from the
    /// exit side in a rapid stream, one after another. Ejected Recruits are unchanged — they return to the field as
    /// normal Recruits. This only affects StoredRecruits (the unreserved pool)": every unit of each building's pool
    /// comes out through its exit doorway, one at a time (`sim/holding.rs`). "Only available if StoredRecruits is not
    /// empty" (`factions/cults/structures/armory.md > ObjectInterfaceState[Armory]`).
    EjectAll,
}

/// An ability a unit uses at once ([`Command::Ability`]).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Ability {
    /// `factions/gdo/units/peacekeeper.md > Riot Shield`: raise the shield (`raise`) or lower it
    /// (`crate::riot_shield`), which makes the unit a Peacekeeper (Shielded) or a Peacekeeper at once, its shield then
    /// working toward it (design changes 0173 and 0198). The command says which, rather than "toggle", so that it
    /// means the same to every unit of a group and when it arrives twice: the Peacekeeper's panel sends Raise Shield,
    /// the Peacekeeper (Shielded)'s Lower Shield.
    RiotShield { raise: bool },
    /// `factions/colonists/units/drone.md > Assigned Drop-off > Set Drop-off Location command`: the workers'
    /// AssignedDropOff becomes this drop-off of their owner's, an Egress Node. It "does not interrupt the Drone's
    /// current action": a return trip under way turns to it (`crate::orders::gather`).
    SetDropOff(ObjectId),
    /// `factions/colonists/units/weapon_conversion.md > Convert Command`: each unit's target form flips, a Sabre
    /// Legionnaire's to the Rifle Legionnaire and the other way round, and the unit converts in place to it over 4
    /// frames (`crate::conversion`, plan decision T26; design change 0198: "Pressing R **flips the unit's target
    /// form**"). Its orders stay (`factions/colonists/units/weapon_conversion.md > Orders are kept`, design change
    /// 0150).
    Convert,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum DebugCommand {
    /// Spawn an object for any owner, bypassing costs and placement rules. A unit's box must be clear of blocked
    /// tiles and of the other units of its layer; a blocking footprint must not cover a unit; a footprint must not
    /// overlap another, except a plate's its patch's (`ObjectKind::may_overlap`); a structure needs flat ground,
    /// but for an underground expansion (`ObjectDef::needs_flat_ground`, design change 0005).
    Spawn {
        kind: ObjectKind,
        owner: Option<PlayerId>,
        at: SpawnAt,
        /// How a tile-based object is turned (`core/entities.md > Structure Type`); units ignore it. Optional in
        /// files, so that the replays recorded before it existed still load.
        #[serde(default, skip_serializing_if = "BuildingOrientation::is_default")]
        orientation: BuildingOrientation,
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
    /// Set `player`'s banks: Space Crystals and Supplies (`factions/gdo/overview.md > GDOResources`), and Alloys,
    /// Essence and Flux (`factions/colonists/overview.md > ColonistResources`, plan decision T21), whoever issues it.
    /// For tests and QA runs of the economy; a game's starting banks come from the Starting Conditions.
    SetResources {
        player: PlayerId,
        crystals: u32,
        supplies: u32,
        /// Optional in files (0), as are `essence` and `flux`, so that the replays recorded before they existed still
        /// load.
        #[serde(default)]
        alloys: u32,
        #[serde(default)]
        essence: u32,
        #[serde(default)]
        flux: u32,
    },
    /// Set the tier of `id`, a Tunnel (`factions/syndicate/tunnels.md > Tunnel > Tiers`, `crate::tunnel::set_tier`):
    /// its MaxHP becomes the tier's, and its HP follows, the damage it has taken staying (design change 0231). For
    /// tests and QA runs of the tiers, without an upgrade's price or time (slice M4-G); an upgrade under way ends,
    /// unrefunded, as the Tunnel is then at the tier asked for. Rejected for an object that isn't a Tunnel, or a tier
    /// its table hasn't.
    SetTier { id: ObjectId, tier: u8 },
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
