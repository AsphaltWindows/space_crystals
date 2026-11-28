//! Why the sim ignores a command ([`Reject`]), and how it says so. Moved out of `sim/gate.rs` whole (slice M5-N3, plan
//! decision T28).

use std::fmt;

use crate::augmentation::{AugmentOrder, RecruitKind};
use crate::defs::{ObjectKind, Tech};
use crate::ids::{ObjectId, PlayerId};
use crate::orders::MAX_ORDER_UNITS;
use crate::placement::PlacementProblem;
use crate::ruleset::RuleField;
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
    /// (`factions/gdo/units/peacekeeper.md > Riot Shield`: "Attack orders are not possible"), or (M5-N4a) they have no
    /// weapon (the Disciple's "AttackAttributes - None"; `combat::takes_attack_orders`).
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
    /// `Sim::build_price`, slice M4-E), or (M6-GD) a unit's (`Recipe::supplies`; `factions/gdo/structures/barracks.md >
    /// ObjectInterfaceState[Barracks]`: "Only available if [...] the player has sufficient Space Crystals and
    /// Supplies").
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
    /// `crate::holding::holds`), or the target of a Tunnel command isn't a Tunnel in operation. (M6-ST1) Or the target
    /// of an Enter is a carrier that doesn't load now (`crate::carry::loads`: the Subterranean Transport underground).
    NotATunnel(ObjectId),
    /// The Tunnel's tier lets none of the units an Enter names into the network, or not a unit of the kind an Eject
    /// names out of it (`factions/syndicate/tunnels.md > Tunnel Network > Transit Tier Requirements`): (M6-SD) an air
    /// unit below Tier 3, whose air doors it goes through.
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
    /// (M6-GF) None of the units an Attack names has a weapon that can hit its target, an object of a domain no
    /// weapon of theirs targets (`systems/combat.md > ValidTarget` 3: "Ground attacks can target Ground and surfaced
    /// Underground units, Air attacks can target Air units, Universal attacks can target any unit";
    /// `combat::domain_allows`): a Twister's anti-air machine gun at a Peacekeeper.
    CannotHit(ObjectId),
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
    /// Armory, Cathedral: Recruits only"; `HoldsDef::accepts`). (M6-ST1) Or the carrier (`crate::carry`): the
    /// Subterranean Transport "carries **infantry only**".
    NotAccepted(ObjectId),
    /// (M5-F1) None of the objects a holding command, or a Release (slice M5-F2), names holds units (or exists any
    /// more): `crate::holding`. (M5-N3) Or none of those that do takes it: a Release at buildings none of which trains
    /// the units inside (`holding::trains`), a `HoldCommand::ReleaseOne` at buildings none of which is a tech building
    /// (`holding::staffed`).
    NoHolders,
    /// (M5-F1) An Eject All to buildings none of which has a unit in its pool
    /// (`factions/cults/structures/armory.md > ObjectInterfaceState[Armory]`: "Only available if StoredRecruits is not
    /// empty"). (M6-ST1) Or an Unload All to carriers none of which has a passenger aboard (`crate::carry`).
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
    /// (`factions/cults/augmentation.md > Altar > Augmentation Orders`: "An order can only target a unit that is
    /// missing every Augmentation the order applies"; `AugmentOrder::fits`).
    AlreadyAugmented(RecruitKind),
    /// (M5-N3) None of the structures of a Research offers the technology (`StructureDef::research`: only a Temple
    /// does).
    CannotResearch(Tech),
    /// (M5-N3) The player has completed the technology (`core/tech_and_research.md > Research > Shared Rules` 4: "Once
    /// completed, it no longer appears in any research menu").
    Researched(Tech),
    /// (M5-N3) The technology is queued or in progress at a structure of the player's (`core/tech_and_research.md >
    /// Research > Shared Rules` 4: "a technology cannot be queued or in progress at more than one structure at the same
    /// time"): what the Temple's key shows as "In progress".
    ResearchUnderWay(Tech),
    /// (M5-N3) The player hasn't completed this technology, the previous level of the one a Research names
    /// (`core/tech_and_research.md > Tech Prerequisites > Research Prerequisites`: "A next level can only be queued
    /// once the previous level has **completed**").
    RequiresTech(Tech),
    /// (M5-N4b) None of the units of a spell's order that cast it has the energy it needs to start
    /// (`defs::SpellDef::needs`, `energy::has`): Shroud's "needs at least **10 energy** to start"
    /// (`factions/cults/units/disciple.md > Placeholder stats > Shroud numbers`). The spells' one refusal for want of
    /// energy (`crate::energy`), and (M5-N5) the casts': none of the casters of a `Command::Cast` that cast its ability
    /// has the energy it costs (`factions/cults/structures/shrine.md > Abilities`: "spend Shrine energy"; plan decision
    /// T76), what greys the Shrine's Q and W.
    NotEnoughEnergy,
    /// (M5-N5) A cast's point isn't in the issuing player's vision now, as the last tick left it
    /// (`factions/cults/structures/shrine.md > Abilities`: "cast on an area anywhere the player currently has vision";
    /// plan decision T76).
    OutOfSight,
    /// (M6-GR) Every caster of a `Command::Cast` that casts its ability, and has the energy it costs, waits out its
    /// cooldown (`Object::cooldown`, `crate::cooldown`; `factions/gdo/structures/radar_station.md > Placeholder stats`:
    /// "**cooldown 60 seconds** (960 frames) [...] Each Radar Station has its own cooldown"): what greys the Radar
    /// Station's Q.
    CoolingDown,
    /// (M5-V2) None of the units a Reverse names can back up (`systems/controls/commands.md > Reverse`: "Only available
    /// to bases with CanReverse = true"; `UnitDef::can_reverse`).
    CannotReverse,
    /// (M5-K) A Target Lock's target is the issuing player's own, or nobody's, as the player knows it
    /// (`factions/gdo/units/rocketman.md > Target Lock`: "The player orders Target Lock on one enemy unit").
    NotAnEnemy(ObjectId),
    /// (M5-X1b) A `DebugCommand::SetRules` whose rules differ from the game's in structure, here first
    /// (`Ruleset::structural_diff`, plan decision T66): they need a new game.
    RulesStructural(RuleField),
    /// (M5-X1b) A `DebugCommand::SetRules` whose rules have a number that isn't finite, here first
    /// (`Ruleset::non_finite`).
    RulesNotFinite(RuleField),
    /// (M5-X1b) A `DebugCommand::SetRules` for which the process has no generation left
    /// (`ruleset::GENERATION_COUNT`): rules other than every generation installed, when all are taken.
    RulesFull,
    /// (M5-X2) A `DebugCommand::SetRules` whose rules have a number the game isn't made for, here first
    /// (`Ruleset::check`): `why` says what it must be ("must be positive", ...).
    RulesInvalid { field: RuleField, why: &'static str },
    /// (M6-GD) Every unit of a Buy that has a stock holds as many items as it can (`crate::stock`;
    /// `factions/gdo/units/drone_operator.md > Drone Stock`: "holds a **maximum of 1**").
    StockFull,
    /// (M6-GD) Every unit of a Buy that has a stock is buying one already (plan decision T89: "refused while a purchase
    /// runs").
    PurchaseUnderWay,
    /// (M6-GD) Every unit of the command has its drone out (`crate::tether`; `factions/gdo/units/drone_operator.md`):
    /// a Buy ("Build Drone can start only once the operator's drone has **ended**", design change 0421), a Launch ("Only
    /// **one drone can be active at a time** per operator"), and an order that would move it ("The Drone Operator
    /// **cannot move** while it has a drone active").
    DroneOut,
    /// (M6-GD) None of the units of a Launch has an item in its stock to launch (`crate::stock`).
    StockEmpty,
    /// (M6-GH1) The Airfield has no room (`crate::aircraft::room`): for the aircraft a Train would queue
    /// (`factions/gdo/structures/airfield.md > Production limit`: "An Airfield **can't queue another aircraft** if its
    /// assignments plus the aircraft already in its production queue would exceed 4"), or for any of the aircraft a
    /// Rebase names that it doesn't hold yet (`factions/gdo/structures/airfield.md > Assignment`: "if that Airfield has
    /// room: its assignments plus the aircraft in its production queue must be **fewer than 4**").
    AirfieldFull,
    /// (M6-GH1) A Rebase's target isn't an Airfield of the player's in operation (`crate::aircraft::is_airfield`).
    NotAnAirfield(ObjectId),
    /// (M6-GH1) None of the aircraft of a Return to Base has a base (`factions/gdo/structures/airfield.md > Returning
    /// and rearming`: "An unassigned aircraft has no base [...] Return to Base is unavailable until it is reassigned").
    NoBase,
    /// (M6-GH1) None of the units of an Attack or an Attack Ground that would take it has ammo left (`crate::aircraft`;
    /// `factions/gdo/units/hornet.md > AttackAttributes`: "Ammo: **5 missiles**, then it must reload at its assigned
    /// Airfield").
    OutOfAmmo,
    /// (M6-GA) None of the units of an order that would take them somewhere can move: their types can't
    /// (`crate::vehicle_deploy::immobile`, plan decision T87: "The deployed kind is immobile by its def";
    /// `factions/gdo/units/artillery_rig.md > Modes`: "one it can't carry out (e.g. **Move**, since a deployed unit
    /// can't move)").
    CannotMove,
    /// (M6-GA) Every unit of a vehicle deploy deploys already (`crate::vehicle_deploy`;
    /// `factions/gdo/units/artillery_rig.md > Modes`: "**Undeploy is only available once deployment is complete**";
    /// "Deploy is available again only once it has").
    Deploying,
    /// (M6-CM) Every unit of a spell's order or of a Mirage whose type casts it is an illusion (`crate::mirage`;
    /// `factions/cults/units/disciple.md > Spells > Mirage`: "**No spells**: illusions can't cast"): what greys an
    /// illusion's R and F. Only its owner, who gives it orders, ever meets it.
    IllusionsCantCast,
    /// (M6-SB) Every unit a toggle would switch on has an empty tank (`crate::patches`;
    /// `factions/syndicate/units/shadebringer.md > Charges and recharging`: "When a tank runs **empty**, that ability
    /// switches off and can't be switched back on until the tank has some charge again"): what greys a Shadebringer's
    /// R once its Sludge's tank is empty.
    TankEmpty,
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
            Reject::CannotAttack => write!(f, "none of the units can attack: shields up, or no weapon"),
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
            Reject::CannotHit(id) => write!(f, "none of the units' weapons can hit object {}", id.0),
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
            Reject::CannotResearch(tech) => write!(f, "none of the structures researches {}", tech.def().name),
            Reject::Researched(tech) => write!(f, "{} is researched already", tech.def().name),
            Reject::ResearchUnderWay(tech) => write!(f, "{} is being researched already", tech.def().name),
            Reject::RequiresTech(tech) => write!(f, "it requires {}", tech.def().name),
            Reject::NotEnoughEnergy => write!(f, "none of the units has the energy the spell needs"),
            Reject::OutOfSight => write!(f, "the point is out of sight"),
            Reject::CoolingDown => write!(f, "every caster waits out its cooldown"),
            Reject::CannotReverse => write!(f, "none of the units can reverse"),
            Reject::NotAnEnemy(id) => write!(f, "object {} isn't an enemy's", id.0),
            Reject::RulesStructural(change) => write!(f, "the rules change {change}, which needs a new game"),
            Reject::RulesNotFinite(field) => write!(f, "{field} isn't a finite number in the rules"),
            Reject::RulesFull => write!(f, "no generation of rules is left; restart the game"),
            Reject::RulesInvalid { field, why } => write!(f, "{field} {why}"),
            Reject::StockFull => write!(f, "the stock is full"),
            Reject::PurchaseUnderWay => write!(f, "a purchase is under way"),
            Reject::DroneOut => write!(f, "its drone is out"),
            Reject::StockEmpty => write!(f, "there is nothing in stock to launch"),
            Reject::AirfieldFull => write!(f, "the Airfield is full"),
            Reject::NotAnAirfield(id) => write!(f, "object {} is no Airfield of the player's in operation", id.0),
            Reject::NoBase => write!(f, "none of the aircraft has an Airfield"),
            Reject::OutOfAmmo => write!(f, "none of the units has ammo left"),
            Reject::CannotMove => write!(f, "none of the units can move"),
            Reject::Deploying => write!(f, "every unit deploys already"),
            Reject::IllusionsCantCast => write!(f, "illusions can't cast"),
            Reject::TankEmpty => write!(f, "every tank is empty"),
        }
    }
}
