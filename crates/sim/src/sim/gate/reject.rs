//! Why the sim ignores a command ([`Reject`]), and how it says so. Moved out of `sim/gate.rs` whole (slice M5-N3, plan
//! decision T28).

use std::fmt;

use crate::augmentation::{AugmentOrder, RecruitKind};
use crate::defs::ObjectKind;
use crate::ids::{ObjectId, PlayerId};
use crate::orders::MAX_ORDER_UNITS;
use crate::placement::PlacementProblem;
use crate::scenario::SpawnProblem;

/// Why the sim ignores a command (`Sim::validate`).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Reject {
    /// The issuing player, or the player a debug command names, is not in the game.
    UnknownPlayer(PlayerId),
    /// The game is over (`core/game_rules.md > End of Game`, `Sim::outcome`): the sim accepts no command any more.
    GameOver,
    /// The issuing player is defeated, and nothing they had can be commanded again (`core/game_rules.md > On
    /// Defeat`, design change 0101); or a debug spawn would give a defeated player an object.
    Defeated(PlayerId),
    /// Debug commands are off in this game ([`crate::sim::SimConfig::debug`]).
    DebugDisabled,
    /// No object has this id: it never existed, or it is gone.
    UnknownObject(ObjectId),
    /// The object belongs to someone else, and only its owner may order it.
    NotOwned(ObjectId),
    /// The object can't be created as asked.
    Spawn(SpawnProblem),
    /// Every object id is used up.
    OutOfIds,
    /// A tile-based object keeps its placement: only units move.
    TileObject(ObjectId),
    /// The position is not finite or lies off the map.
    OffMap,
    /// None of the units an order names exists any more (or takes orders).
    NoUnits,
    /// An order names more than [`MAX_ORDER_UNITS`] units.
    TooManyUnits,
    /// The order's target can't be attacked: it is indestructible
    /// (`systems/controls/commands.md > Attack`: a destructible object).
    NotAttackable(ObjectId),
    /// The order's target isn't in the issuing player's sight (`core/vision.md > Explored Selectability`; question
    /// Q0017, point 2).
    NotVisible(ObjectId),
    /// None of the units an ability names has that ability.
    LacksAbility,
    /// None of the units an Attack order names can attack now: their Riot Shields are up
    /// (`factions/gdo/units/peacekeeper.md > Riot Shield`: "Attack orders are not possible").
    CannotAttack,
    /// A unit's box there would overlap a blocked tile, a blocking footprint or the map's edge
    /// (`docs/movement.md > 3`).
    Blocked,
    /// A unit's box there would overlap this unit of its layer, or a blocking footprint would cover this unit
    /// (`docs/movement.md > 3`).
    Occupied(ObjectId),
    /// A footprint there would overlap this object's (`core/entities.md > Placement Validation`): only a plate
    /// shares its tile, with the uncovered patch it is placed onto (`ObjectKind::may_overlap`).
    Overlaps(ObjectId),
    /// None of the objects a production command names produces units (or exists any more).
    NoProducers,
    /// None of the producers can produce a unit of this kind (`factions/gdo/structures/barracks.md > Produces`).
    CannotProduce(ObjectKind),
    /// The BuildQueue of every producer that could train the unit is full (design change 0081: 20 with the unit in
    /// production).
    QueueFull,
    /// The player's Space Crystals don't cover the unit, or the building to construct ("Only available if player
    /// has sufficient Space Crystals", `factions/gdo/structures/deployment_center.md >
    /// ObjectInterfaceState[DeploymentCenter]`). The army cap refuses nothing: design change 0143 checks it when
    /// production starts (`Production::blocked`), and M2's `UnitControlCap` went with the check at queue time.
    InsufficientCrystals,
    /// The player's Supplies don't cover a Build order's price (`factions/syndicate/tunnels.md > Cost Scaling >
    /// Construction Cost`, design change 0233: "The order is refused, with feedback, if the player can't pay";
    /// `Sim::build_price`, slice M4-E).
    InsufficientSupplies,
    /// Cancel Production: no producer has anything queued.
    NothingQueued,
    /// None of the objects a construction command names is one of the player's constructors that exists
    /// (`crate::construction`); for a Warp In, one of the player's Beacons (`crate::transform`).
    NoConstructor,
    /// A Place names more than one constructor: one building is placed by one constructor (design change 0071); a
    /// Warp In more than one Beacon: one Beacon becomes the building (plan decision T22); a Deploy more than one
    /// Drone: one Drone becomes the Beacon (`crate::deploy`).
    SeveralConstructors,
    /// The building isn't on the constructor's `Constructs` list (`factions/gdo/structures/deployment_center.md >
    /// Constructs`, `factions/gdo/economy.md > ExtractionFacility > Constructs`), or on the Beacon's
    /// (`factions/colonists/economy.md > Warp In > Constructs List`).
    CannotConstruct(ObjectKind),
    /// The constructor is constructing, or holds a building ready to place: "Other build options unavailable"
    /// (`factions/gdo/structures/deployment_center.md > ObjectInterfaceState[DeploymentCenter]`); the Beacon of a Warp
    /// In is transforming itself (a Beacon still being deployed, slice M3-F).
    ConstructorBusy(ObjectId),
    /// A Cancel Construction to a constructor that isn't constructing.
    NotConstructing,
    /// A Cancel Ready Building or a Place to a constructor that holds no building ready to place.
    NothingReady,
    /// None of the units a Gather or a Drop Off order names is a worker (`factions/colonists/units/drone.md`), none
    /// of those a Gather on a Supply Delivery Station names gathers Supplies (`Sim::gatherers_among`, slice M4-I), or
    /// none of those a Deploy names deploys (`crate::deploy`).
    NoWorkers,
    /// A Gather's target isn't a Space Crystals patch that workers can gather, nor a Supply Delivery Station: another
    /// object, or a patch under a mining structure (design change 0137).
    NotGatherable(ObjectId),
    /// The target of a Drop Off order or of Set Drop-off Location isn't a drop-off of the player's, an own Egress
    /// Node (`factions/colonists/units/drone.md > Assigned Drop-off`).
    NotDropOff(ObjectId),
    /// The ready building can't be placed there (`core/entities.md > Placement Validation`, `crate::placement`): the
    /// first problem the placement check finds. When a Place is refused for this as it applies, the sim says why
    /// (`SimEvent::PlacementRefused`); a Warp In the same.
    Unplaceable(PlacementProblem),
    /// A Cancel Warp In to objects none of which is transforming (`factions/colonists/economy.md > Warp In > Cancel
    /// Warp In`, `crate::transform`).
    NotTransforming,
    /// A Drop Off order to workers none of which carries anything (design change 0140: "Drop Off is only resolved
    /// for a Drone that is carrying something"; the empty ones among loaded ones are left out).
    NothingToDeliver,
    /// A debug SetTier to an object without that tier: it isn't a Tunnel, or its tier table hasn't the tier
    /// (`factions/syndicate/tunnels.md > Tunnel > Tiers`: 1 to 3).
    NoSuchTier { id: ObjectId, tier: u8 },
    /// The target of an Enter order is neither a Tunnel in operation nor a building that holds units
    /// (`systems/controls/commands.md > Enter`: "the structure being entered"; `crate::network::is_exit`,
    /// `crate::holding::holds`), or the target of a Tunnel command isn't a Tunnel in operation.
    NotATunnel(ObjectId),
    /// The Tunnel's tier lets none of the units an Enter names into the network, or not a unit of the kind an Eject
    /// names out of it (`factions/syndicate/tunnels.md > Tunnel Network > Transit Tier Requirements`); no air unit
    /// passes yet (`tuning::tunnel::AIR_DOORS`).
    NoTransit { tunnel: ObjectId, kind: Option<ObjectKind> },
    /// An Eject for a kind of which the player has no unit in the Tunnel Network that isn't queued to come out
    /// already (`factions/syndicate/tunnel_menus.md > EjectMenu`: "each click reserves a specific unit").
    NotInNetwork(ObjectKind),
    /// An Eject for a kind whose units in the network all walked in less than the transit delay ago
    /// (`factions/syndicate/tunnels.md > Tunnel Network > Transit delay`, design change 0245: "For 2 s it can't be
    /// ejected anywhere").
    InTransit(ObjectKind),
    /// (M4-O) None of the units an Attack Ground names has an attack that can target the ground
    /// (`systems/controls/commands.md > AttackGround`: "Only available to units whose AttackType has CanTargetGround";
    /// `combat::can_target_ground`).
    CannotTargetGround,
    /// (M4-F) The Tunnel performs an operation already (`factions/syndicate/tunnels.md > Construction and Upgrade
    /// Rules`: "A Tunnel can only perform one operation at a time"; `tunnel::Operation`).
    OperationUnderWay(ObjectId),
    /// (M4-F) The building Tunnel's tier is below the expansion's Tier Requirement, `tier`
    /// (`factions/syndicate/tunnel_menus.md > ExpandMenu`: "available only if (1) its Tier Requirement is at or below
    /// **this** Tunnel's tier").
    TierRequirement { kind: ObjectKind, tier: u8 },
    /// (M4-F) A Tunnel's Cancel while it performs no operation ("**X** Cancel (upgrade, construction or the current
    /// operation)", `factions/syndicate/tunnel_menus.md > Tunnel ObjectInterfaceState`).
    NoOperation(ObjectId),
    /// (M4-K) The player owns no complete structure of this type, which what the command starts requires: a unit's
    /// Unit Prerequisite as it is queued, a building's TechPrerequisite as its construction starts
    /// (`core/tech_and_research.md > Tech Prerequisites`; `crate::tech`, plan decision T37). The first one missing.
    Requires(ObjectKind),
    /// (M4-G) An Upgrade to a Tunnel at its table's last tier (`factions/syndicate/tunnels.md > Tunnel
    /// ObjectInterfaceState`: "Unavailable if already Tier 3").
    TopTier(ObjectId),
    /// (M4-G) Something blocks the Tunnel's upgrade, by the truth: its area at the target tier would overlap another
    /// Tunnel's (any player's, as it reserves it), or reach a cell off the map or not Drillable
    /// (`factions/syndicate/tunnel_area_and_space.md > Tunnel Area Non-Overlap Rule`). It never says what: "the upgrade
    /// check uses the truth; the upgrading Tunnel's owner learns that something blocks it".
    UpgradeBlocked(ObjectId),
    /// (M5-F1) The target of an Enter is a building that holds units, under construction (design change 0347 point 4,
    /// `factions/cults/structures/README.md > Entering pool buildings`: "a building under construction **can't be
    /// entered**; a right-click on it Assists").
    UnderConstruction(ObjectId),
    /// (M5-F1) The building takes none of the units an Enter names (`systems/controls/commands.md > Enter`: "Cults —
    /// Armory, Cathedral: Recruits only"; `HoldsDef::accepts`).
    NotAccepted(ObjectId),
    /// (M5-F1) None of the objects a holding command, or a Release (slice M5-F2), names holds units (or exists any
    /// more): `crate::holding`.
    NoHolders,
    /// (M5-F1) An Eject All to buildings none of which has a unit in its pool
    /// (`factions/cults/structures/armory.md > ObjectInterfaceState[Armory]`: "Only available if StoredRecruits is not
    /// empty").
    NothingInside,
    /// (M5-E) The target of an Assist is no site of the player's that its builders build from outside
    /// (`factions/cults/economy.md > Assist Construction Command`: "Left Click on an in-progress building"; design
    /// change 0339, `factions/cults/economy.md > Cults Building Mechanics > Group construction rules`: "**Assist**
    /// targets only a building **already under construction**, not a ghost whose order hasn't started").
    NotASite(ObjectId),
    /// (M5-F2) None of the buildings a TrainFrom or a Release names has enough free units of this kind inside: the
    /// unit's Unit Control cost of them, one for a Release (`Refusal::NoRecruit`; `factions/cults/structures/README.md
    /// > Choosing Recruits, Release and Eject All`: a kind's button "is greyed if there are none").
    NoRecruit(RecruitKind),
    /// (M5-N8) None of the units of a Repair can repair its target now (`Sim::can_repair`, `crate::repair`): it is
    /// no damaged own object, complete, of a kind their types repair.
    CantRepair(ObjectId),
    /// (M5-N1a) A `DebugCommand::SetAugmentation` that gives Augmentations to a unit of a kind that can't have them
    /// (`factions/cults/augmentation.md > Eligibility`: "Only Cults infantry can be augmented";
    /// `augmentation::is_augmentable`).
    NotAugmentable(ObjectId),
    /// (M5-N1b) None of the producers of an Augment takes the order (`ProductionDef::augment`: only an Altar does).
    CannotAugment(AugmentOrder),
    /// (M5-N1b) An Augment for units of this kind, which have an Augmentation the order applies
    /// (`factions/cults/augmentation.md > Altar > Augmentation Orders`: "An order can only target a unit that is missing
    /// every Augmentation the order applies"; `AugmentOrder::fits`).
    AlreadyAugmented(RecruitKind),
}

impl fmt::Display for Reject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Reject::UnknownPlayer(p) => write!(f, "player {} is not in the game", p.0),
            Reject::GameOver => write!(f, "the game is over"),
            Reject::Defeated(p) => write!(f, "player {} is defeated", p.0),
            Reject::DebugDisabled => write!(f, "debug commands are off"),
            Reject::UnknownObject(id) => write!(f, "object {} doesn't exist", id.0),
            Reject::NotOwned(id) => write!(f, "object {} belongs to someone else", id.0),
            Reject::Spawn(problem) => write!(f, "can't spawn: {problem}"),
            Reject::OutOfIds => write!(f, "every object id is used up"),
            Reject::TileObject(id) => write!(f, "object {} is tile-based and can't move", id.0),
            Reject::OffMap => write!(f, "the position is off the map"),
            Reject::NoUnits => write!(f, "none of the ordered units exists"),
            Reject::TooManyUnits => write!(f, "an order may name at most {MAX_ORDER_UNITS} units"),
            Reject::NotAttackable(id) => write!(f, "object {} can't be attacked", id.0),
            Reject::NotVisible(id) => write!(f, "object {} is out of sight", id.0),
            Reject::LacksAbility => write!(f, "none of the units has the ability"),
            Reject::CannotAttack => write!(f, "none of the units can attack with its shield up"),
            Reject::Blocked => write!(f, "the unit doesn't fit there: blocked ground"),
            Reject::Occupied(id) => write!(f, "unit {} is in the way", id.0),
            Reject::Overlaps(id) => write!(f, "the footprint overlaps object {}", id.0),
            Reject::NoProducers => write!(f, "none of the objects produces units"),
            Reject::CannotProduce(kind) => write!(f, "none of the producers trains a {}", kind.def().name),
            Reject::QueueFull => write!(f, "the build queue is full"),
            Reject::InsufficientCrystals => write!(f, "not enough Space Crystals"),
            Reject::InsufficientSupplies => write!(f, "not enough Supplies"),
            Reject::NothingQueued => write!(f, "nothing is queued"),
            Reject::NoConstructor => write!(f, "none of the objects is a constructor of the player's"),
            Reject::SeveralConstructors => write!(f, "a building is placed by one constructor"),
            Reject::CannotConstruct(kind) => write!(f, "the constructor doesn't construct a {}", kind.def().name),
            Reject::ConstructorBusy(id) => write!(f, "constructor {} is busy", id.0),
            Reject::NotConstructing => write!(f, "nothing is under construction"),
            Reject::NothingReady => write!(f, "no building is ready to place"),
            Reject::NoWorkers => write!(f, "none of the units is a worker"),
            Reject::NotGatherable(id) => write!(f, "object {} can't be gathered", id.0),
            Reject::NotDropOff(id) => write!(f, "object {} is no drop-off of the player's", id.0),
            Reject::Unplaceable(problem) => write!(f, "the building can't go there: {problem}"),
            Reject::NotTransforming => write!(f, "nothing is transforming"),
            Reject::NothingToDeliver => write!(f, "none of the workers carries anything"),
            Reject::NoSuchTier { id, tier } => write!(f, "object {} has no tier {tier}", id.0),
            Reject::NotATunnel(id) => write!(f, "object {} is no Tunnel in operation", id.0),
            Reject::NoTransit { tunnel, kind: None } => {
                write!(f, "Tunnel {}'s tier lets none of the units in", tunnel.0)
            }
            Reject::NoTransit {
                tunnel,
                kind: Some(kind),
            } => write!(f, "Tunnel {}'s tier lets no {} out", tunnel.0, kind.def().name),
            Reject::NotInNetwork(kind) => write!(f, "no {} in the Tunnel Network is free to leave", kind.def().name),
            Reject::InTransit(kind) => write!(f, "the {} only just entered the Tunnel Network", kind.def().name),
            Reject::CannotTargetGround => write!(f, "none of the units can attack the ground"),
            Reject::OperationUnderWay(id) => write!(f, "Tunnel {} performs an operation already", id.0),
            Reject::TierRequirement { kind, tier } => write!(f, "a {} needs a Tier {tier} Tunnel", kind.def().name),
            Reject::NoOperation(id) => write!(f, "Tunnel {} performs no operation", id.0),
            Reject::Requires(kind) => write!(f, "it requires a complete {} of the player's", kind.def().name),
            Reject::TopTier(id) => write!(f, "Tunnel {} is at its top tier", id.0),
            Reject::UpgradeBlocked(id) => write!(f, "something blocks Tunnel {}'s upgrade", id.0),
            Reject::UnderConstruction(id) => write!(f, "building {} is under construction", id.0),
            Reject::NotAccepted(id) => write!(f, "building {} takes none of the units in", id.0),
            Reject::NoHolders => write!(f, "none of the objects holds units"),
            Reject::NothingInside => write!(f, "nothing is inside"),
            Reject::NotASite(id) => write!(f, "object {} is no building of the player's under construction", id.0),
            Reject::NoRecruit(kind) => write!(f, "no {} Recruit is free inside", kind.name()),
            Reject::CantRepair(id) => write!(f, "none of the units can repair object {} now", id.0),
            Reject::NotAugmentable(id) => write!(f, "object {} can't be augmented", id.0),
            Reject::CannotAugment(order) => write!(f, "none of the producers applies {}", order.name()),
            Reject::AlreadyAugmented(kind) => write!(f, "a {} unit has an Augmentation the order applies", kind.name()),
        }
    }
}
