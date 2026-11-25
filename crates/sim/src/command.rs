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
    /// The player's `units` use `ability` at once (`factions/gdo/units.md > Peacekeeper > Riot Shield (toggled
    /// ability)`). An ability is not an order: it doesn't enter or replace a unit's command queue, Shift means nothing
    /// to it, and the unit goes on with its orders meanwhile (design change 0023: a mode isn't a task). Ids that no
    /// longer exist, repeated ids and units without the ability are dropped; the command is rejected if no unit
    /// remains or one belongs to someone else.
    Ability { units: Vec<ObjectId>, ability: Ability },
    /// The player's producing structures among `objects` take `command` (`factions/gdo/structures.md > Barracks >
    /// ObjectInterfaceState[Barracks]`; `production`): train a unit, cancel the last one queued, or set the rally
    /// point. Every producer of the selection receives it (`systems/controls/selection_and_hud.md > SelectionGroup >
    /// No smart-casting`). Ids that no longer exist, repeated ids and the player's objects that produce nothing are
    /// dropped; the command is rejected if no producer remains or one belongs to someone else.
    Production {
        objects: Vec<ObjectId>,
        command: ProductionCommand,
    },
    /// The player's constructors `objects` (the Deployment Center, the Extraction Facility) carry out `command`
    /// (`crate::construction`; `factions/gdo/structures.md > DeploymentCenter`, `factions/gdo/economy.md >
    /// ExtractionFacility`), each in id order that can. Ids that no longer exist, repeated ids and objects that
    /// construct nothing are dropped; the command is rejected if none of the rest can carry it out, or one of the
    /// objects belongs to someone else. A Place names one constructor.
    Construction {
        objects: Vec<ObjectId>,
        command: ConstructionCommand,
    },
    /// Development / QA only — never offered to players in a real match.
    Debug(DebugCommand),
}

/// An ability a unit uses at once ([`Command::Ability`]).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Ability {
    /// `factions/gdo/units.md > Peacekeeper > Riot Shield`: raise the shield (`raise`) or lower it
    /// (`crate::riot_shield`), which makes the unit a Peacekeeper (Shielded) or a Peacekeeper at once, its shield then
    /// working toward it (design changes 0173 and 0198). The command says which, rather than "toggle", so that it
    /// means the same to every unit of a group and when it arrives twice: the Peacekeeper's panel sends Raise Shield,
    /// the Peacekeeper (Shielded)'s Lower Shield.
    RiotShield { raise: bool },
    /// `factions/colonists/units.md > Drone > Assigned Drop-off > Set Drop-off Location command`: the workers'
    /// AssignedDropOff becomes this drop-off of their owner's, an Egress Node. It "does not interrupt the Drone's
    /// current action": a return trip under way turns to it (`crate::orders::gather`).
    SetDropOff(ObjectId),
    /// `factions/colonists/units.md > Weapon Conversion > Convert Command`: each unit's target form flips, a Sabre
    /// Legionnaire's to the Rifle Legionnaire and the other way round, and the unit converts in place to it over 4
    /// frames (`crate::conversion`, plan decision T26; design change 0198: "Pressing R **flips the unit's target
    /// form**"). Its orders stay (`> Weapon Conversion > Orders are kept`, design change 0150).
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
    /// tests and QA runs of the tiers, which nothing else changes until the upgrades (slice M4-G). Rejected for an
    /// object that isn't a Tunnel, or a tier its table hasn't.
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
