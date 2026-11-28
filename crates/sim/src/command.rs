//! Player input to the simulation.
//!
//! The client never mutates sim state directly: every player action becomes a
//! [`PlayerCommand`], collected into the [`TickInputs`] for the next tick. The sim
//! validates each command (ownership, cost, …) and ignores invalid ones. Recording
//! the `TickInputs` of every tick is a complete replay.

use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::augmentation::{Augmentation, RecruitKind};
use crate::construction::ConstructionCommand;
use crate::defs::{AreaAbility, ObjectKind, PatchKind};
use crate::ids::{ObjectId, PlayerId};
use crate::orders::OrderKind;
use crate::production::ProductionCommand;
use crate::ruleset::Ruleset;
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
    /// point. Every producer of the selection receives it (`systems/controls/selection.md > SelectionGroup >
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
    /// selection receives it (`systems/controls/selection.md > SelectionGroup > No smart-casting`). Ids that no
    /// longer exist, repeated ids and the player's objects that hold no units are dropped; the command is rejected if
    /// none of the rest can carry it out, or one of the objects belongs to someone else.
    Holding {
        objects: Vec<ObjectId>,
        command: HoldCommand,
    },
    /// (M5-N5) One of the player's `casters` casts the area ability `ability` at `at`, space units (plan decision T76:
    /// a command, not an order, as a structure has no order queue; `factions/cults/structures/shrine.md > Abilities`:
    /// "Both abilities are cast on an area anywhere the player currently has vision, and spend Shrine energy";
    /// `crate::buffs`). The first of them in id order that casts it now does (`Sim::can_cast`): "each press casts from
    /// one Shrine" (Q0118). Ids that no longer exist and repeated ids are dropped; the command is rejected if none of
    /// the rest can cast it, one of them belongs to someone else, or the player doesn't see `at` now.
    Cast {
        casters: Vec<ObjectId>,
        ability: AreaAbility,
        at: Vec2,
    },
    /// Development / QA only — never offered to players in a real match.
    Debug(DebugCommand),
}

/// A command to a Tunnel ([`Command::Tunnel`], plan decision T36).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TunnelCommand {
    /// `factions/syndicate/tunnel_menus.md > EjectMenu`: "ejects one unit of that type from this Tunnel's Side A", the
    /// one of `kind` "that has been inside the network longest" among those free to leave (design changes 0245 and
    /// 0251). The click reserves that unit on the Tunnel's eject queue, and it comes out of the exit path once the
    /// units queued before it are out and the exit spot is clear (0250).
    Eject { kind: ObjectKind },
    /// `factions/syndicate/tunnel_menus.md > AwaitingPlacement (Expansion)` (slice M4-F): "Left-click valid location:
    /// places expansion, begins construction": the Tunnel builds an expansion of `kind` with the top-left tile of its
    /// footprint at `origin`, turned by `orientation`, in its Tunnel Area, as its one operation, paid at once (design
    /// change 0254: "Paid when the ghost is placed"; `sim/expansions.rs`).
    Expand {
        kind: ObjectKind,
        origin: TilePos,
        orientation: BuildingOrientation,
    },
    /// `factions/syndicate/tunnel_menus.md > Tunnel ObjectInterfaceState`: "**Q** Upgrade Tunnel" (slice M4-G),
    /// "upgrades to next tier (CommandIssuingTransition). Costs Supplies per upgrade cost formula. Unavailable if
    /// already Tier 3 or if currently performing an operation": the Tunnel upgrades to the tier above its own as its
    /// one operation, paid at once (`factions/syndicate/tunnels.md > Cost Scaling > Upgrade Costs`: "**Paid when the
    /// upgrade is ordered**"), if its larger area is clear by the truth (`sim/upgrades.rs`).
    Upgrade,
    /// `factions/syndicate/tunnel_menus.md > Tunnel ObjectInterfaceState`: "**X** Cancel (upgrade, construction or the
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
    /// (M5-N3) A tech building's **A/S/D/F** (`factions/cults/structures/README.md > Choosing Recruits, Release and
    /// Eject All`: "**Tech buildings** (Temple, Gunworks, Machine Shop, Shrine): DefaultState has **one button per
    /// kind** [...]: pressing one sends **one** Recruit of that kind out of the exit at once (no queue)"): the unit of
    /// this kind that entered first leaves through the exit doorway, as an ejected one does (`sim/holding.rs`). A
    /// pool building releases by its queue instead (`ProductionCommand::Release`).
    ReleaseOne(RecruitKind),
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
    /// (M6-GD) `factions/gdo/units/drone_operator.md > Drone Stock`: "**Build Drone** is one of the Drone Operator's
    /// abilities: once its drone has been used, it purchases/builds a replacement" (plan decision T89): each unit buys
    /// one item more for its stock (`crate::stock`), paid at the press, over its type's frames. Its orders stay. Refused
    /// while its stock is full, while it buys one, and while its drone is out (design change 0421).
    Buy,
    /// (M6-CM) `factions/cults/units/disciple.md > Spells > Mirage`: "An **instant** spell: the Disciple creates **4
    /// illusions** of itself" (`crate::mirage`): each unit pays its cost, ends its Shroud channel ("Casting it cancels
    /// Shroud") and makes its new illusions, its old ones vanishing. Its other orders stay. Refused for a unit that
    /// can't pay ("**50 energy**") and for an illusion ("**No spells**: illusions can't cast").
    Mirage,
    /// (M6-SB) `factions/syndicate/units/shadebringer.md > Sludge (toggle)`: switch the Sludge on (`on`) or off
    /// (`crate::patches`): on, each unit lays its trail of sludge and drains its tank, until it is switched off or its
    /// tank is empty. The command says which, as the Riot Shield's does, so that it means the same to every unit of a
    /// group: the panel sends on unless every unit it goes to has it on already. "Toggles follow the mode-switch rule
    /// (keep orders, not queueable)". Switching on is refused for a unit whose tank is empty ("a toggle can be switched
    /// on with **any charge** left in its tank", design change 0369).
    Sludge { on: bool },
    /// (M6-SB) `factions/syndicate/units/shadebringer.md > Cover (toggle)`: switch the Cover on or off, by the
    /// Sludge's rule (`crate::patches`), with its own tank. Slice M6-SC gives the Shadebringer that tank and the smoke
    /// it lays: until then no unit has it, and the gate refuses the press (`Reject::LacksAbility`).
    Cover { on: bool },
}

impl Ability {
    /// (M6-SB) The kind of patch a toggle lays, and whether it switches it on: the Sludge's and the Cover's
    /// (`crate::patches`, one rule for both). `None` for the abilities that aren't a tank's toggle.
    pub fn toggle(self) -> Option<(PatchKind, bool)> {
        match self {
            Ability::Sludge { on } => Some((PatchKind::Sludge, on)),
            Ability::Cover { on } => Some((PatchKind::Smoke, on)),
            Ability::RiotShield { .. } | Ability::SetDropOff(_) | Ability::Convert | Ability::Buy | Ability::Mirage => {
                None
            }
        }
    }

    /// (M6-SB) The toggle of the tank that lays `kind`'s patches, switching it on (`on`) or off: [`Ability::Sludge`] or
    /// [`Ability::Cover`], whose [`Ability::toggle`] this is the other way round. The client's panel sends it.
    pub fn switching(kind: PatchKind, on: bool) -> Ability {
        match kind {
            PatchKind::Sludge => Ability::Sludge { on },
            PatchKind::Smoke => Ability::Cover { on },
        }
    }
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
    /// (M5-F2) Set the augmentations of `id`, a unit on the map (`Object::augmentation`,
    /// `factions/cults/augmentation.md > AugmentationState (per unit instance)`), whoever owns it, and with them the
    /// kind a building that trains it counts it as (design change 0337). For tests and QA runs of the Armory's kinds
    /// and of the augmented stats until the Altar augments (slice M5-N1b). (M5-N1a) They take effect at once, at its
    /// owner's levels (`crate::augmentation`), its HP following its MaxHP (`augmentation::follow_max`: the damage
    /// stays as MaxHP goes up, HP drops to a lower MaxHP only if above it). Rejected for an object that isn't a unit,
    /// and for Augmentations to a unit that can't have them (`Reject::NotAugmentable`).
    SetAugmentation { id: ObjectId, augmentation: Augmentation },
    /// (M5-X1b) Play on with `rules` from the next tick on (plan decision T66, `crate::ruleset`): their numbers apply
    /// as the tick of the command ends, every object keeping its state, its HP and Shields following their new maxima
    /// with the damage kept (`transform::follow_max`) and its energy at most its new MaxEnergy; what is under way goes
    /// on by the new numbers (a refund, for one, at the new price), and what is fixed as it starts (a shot in flight, a
    /// path planned) stays as it started. [`crate::SimEvent::RulesChanged`] tells it. The ruleset travels in the
    /// command, so that a replay plays a tuned game back. Rejected for rules whose structure differs from the game's
    /// (`Ruleset::structural_diff`: `Reject::RulesStructural`), which need a new game, for rules with a number that
    /// isn't finite (`Reject::RulesNotFinite`), (M5-X2) for rules with a number the game isn't made for
    /// (`Ruleset::check`: `Reject::RulesInvalid`), and when the process has no generation left for them
    /// (`Reject::RulesFull`). For the data files' tuning, which the client reads and sends (slice M5-X2), and the tests
    /// of tuned rules.
    SetRules(Box<Ruleset>),
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
