//! Short feedback messages of the command flow, placeholders until the future notification system:
//! - a click on an invalid target (`systems/controls/interface.md > AwaitingTarget[CommandType] > Invalid
//!   target clicks`, design change 0065): the click is ignored, the target is still awaited, and the player gets
//!   "short feedback (an 'invalid target' message and/or sound; exact form TBD, ties into the future notification
//!   system)";
//! - a Train that some of its producers didn't queue for want of Space Crystals
//!   (`systems/controls/selection.md > SelectionGroup > No smart-casting > Group production with limited
//!   resources`, design change 0083): "The others don't queue, and the player gets insufficient-resources feedback".
//!   The sim tells which (`SimEvent::ProductionRefused`); a full queue is no resource, and gives none. A Train that no
//!   producer would queue is never sent: its button is disabled (`super::interface::Conditions`). The army cap is
//!   checked when production starts, not when a unit is queued (design change 0143), so it refuses nothing;
//! - a producer whose unit at the front waits at 0% for room under the army cap (`SimEvent::ProductionWaiting`,
//!   design change 0143: "A blocked queue raises a notification (not yet designed)"): placeholder for that
//!   notification, the cap's name ([`army_cap_text`]: Unit Control, or the Colonists' Beacon Capacity, plan decision
//!   T20), once as the wait begins; the producer's InfoPanel shows the wait for as long as it lasts
//!   (`crate::hud::info_panel`);
//! - a building that can't be placed (`factions/gdo/structures/deployment_center.md >
//!   ObjectInterfaceState[DeploymentCenter] > AwaitingPlacement`): a left-click where the ghost is invalid sends
//!   nothing and placement goes on (`super::InvalidPlacement`), and a Place that the sim refused as it applied it,
//!   the game having changed since the click (`SimEvent::PlacementRefused`, e.g. a unit walked onto the footprint),
//!   leaves the building ready to place and the player in DefaultState, where the valid click left them
//!   (`factions/gdo/structures/deployment_center.md > AwaitingPlacement`: it "returns to DefaultState"); the ready
//!   building's slot enters placement again. Both say why: the placement check's first problem ([`placement_text`]).
//!   Placeholders: the design gives no message for either;
//! - a Deploy that failed at the end of the Drone's walk (`SimEvent::DeployFailed`, slice M3-F; `factions/colonists/
//!   economy.md > Beacon Deployment > Placement`), and a Build at the end of its worker's (`SimEvent::BuildFailed`,
//!   slice M4-E): what the check found on arrival, as for a building, or that the worker didn't get to its site
//!   ([`deploy_failed_text`]). Placeholders too;
//! - an Attack whose unit can't move, or can't back off further (design change 0204 point 3), and whose target is
//!   inside its MinRange (`SimEvent::TargetTooClose`; `systems/controls/command_behaviors.md > AttackingObject > Target
//!   inside MinRange`, design change 0182: "the order fails with feedback ("Target too close"; same channel as other
//!   refusals)"): [`TARGET_TOO_CLOSE_TEXT`]; and an Attack Ground whose location is (`SimEvent::LocationTooClose`,
//!   slice M5-O3; `systems/controls/command_behaviors.md > AttackingLocation`, as AttackingObject), the same;
//! - an Enter whose unit got to a building's entrance with its pool full (`SimEvent::EnterFailed` with `full`; slice
//!   M5-F1, design change 0347 point 2): [`BUILDING_FULL_TEXT`];
//! - a greyed-out button pressed, or its hotkey (`systems/controls/interface.md > CommandPanel > Pressing an
//!   unavailable command`, design change 0162: "nothing is issued, and the player gets the same feedback as when an
//!   order is refused, naming the reason (e.g. "Not enough Space Crystals", "Queue is full")"), and a Warp In whose
//!   click the sim refused (`super::CommandRefused`): the button's reason ([`unavailable_text`]), placeholder
//!   wordings but the design's two;
//! - (M6-GD) a right-click that moves nothing for the units' drones (`factions/gdo/units/drone_operator.md > Launching
//!   and piloting`: "The Drone Operator **cannot move** while it has a drone active"; plan §GD's acceptance: "The
//!   operator refuses Move, with feedback, until the drone is gone"): every order it gives refused, one of them for a
//!   drone out (`sc_sim::Reject::DroneOut`), said as a greyed Move is ([`DRONE_OUT_TEXT`], `super::CommandRefused`);
//! - (M6-GH1) a right-click on an own Airfield that reassigns no aircraft, the Airfield full
//!   (`systems/mechanics/aircraft_landing.md > Assigned base`: "if that Airfield has room";
//!   `sc_sim::Reject::AirfieldFull`): [`AIRFIELD_FULL_TEXT`], as a greyed Q says it;
//! - (M6-GA) an order a vehicle deploy dropped as it ended (`SimEvent::OrderDropped`; `factions/gdo/units/
//!   artillery_rig.md > Modes`: "one it can't carry out (e.g. **Move**, since a deployed unit can't move) is **dropped**
//!   with the usual unavailable-command feedback"): the gate's refusal of that order to the unit as it is now
//!   ([`dropped_text`]), as its greyed button says it ([`CANNOT_MOVE_TEXT`] for a Move); and a right-click on the ground
//!   that moves none of the units for their types (`sc_sim::Reject::CannotMove`), the same.
//!
//! The sim's events among these come through the filter of events (`crate::viewer_events`), which gives each to the
//! owner of its producer, constructor or unit alone.
//!
//! The message ([`INVALID_TARGET_TEXT`], [`NO_CRYSTALS_TEXT`], an [`army_cap_text`], a [`placement_text`], an
//! [`unavailable_text`], [`TARGET_TOO_CLOSE_TEXT`] or [`BUILDING_FULL_TEXT`]) shows over the world view, centred just
//! above the HUD Bar, on a dark band, for `FeedbackTuning::feedback_ticks` after the latest cause; a new one replaces
//! it. (M5-X2) A data directory's refusal (`DataRefused`, `crate::data`) shows there too, for
//! `FeedbackTuning::refusal_ticks`, long enough to read its file and member. It ages by the sim's ticks, so a QA
//! screenshot of a tick always shows it alike. It never takes the pointer: clicks pass through it to the map.

use std::sync::LazyLock;

use bevy::picking::Pickable;
use bevy::prelude::*;
use sc_sim::{
    ArmyResource, Command, ObjectId, ObjectKind, OrderKind, PlacementCheck, PlacementProblem, PlayerCommand, PlayerId,
    Refusal, Reject, Sim, SimEvent, Tech, Tick, TileProblem, WholeProblem,
};

use super::{CommandRefused, DataRefused, InvalidPlacement, InvalidTarget, WhyUnavailable};
use crate::fog::LocalSight;
use crate::hud::HUD_BAR_HEIGHT_VH;
use crate::sim_driver::SimRes;
use crate::tuning::ClientTuning;
use crate::viewer_events::ViewerEvents;
use crate::{ClientMode, ClientSystems};

/// The message of an invalid target. Placeholder wording, the design's.
pub const INVALID_TARGET_TEXT: &str = "Invalid target";
/// The message of a Train that producers didn't queue for want of Space Crystals (design change 0083). Placeholder
/// wording.
pub const NO_CRYSTALS_TEXT: &str = "Not enough Space Crystals";
/// The message of a Build its player can't pay for (`factions/syndicate/tunnels.md > Cost Scaling > Construction
/// Cost`, design change 0233: "The order is refused, with feedback, if the player can't pay"). Placeholder wording,
/// after the design's "Not enough Space Crystals" (0162).
pub const NO_SUPPLIES_TEXT: &str = "Not enough Supplies";
/// The messages of a producer that waits for room under the army cap (design change 0143), by the faction's cap (plan
/// decision T20: the faction's name in the feedback; [`army_cap_text`]). Placeholder wordings.
pub const UNIT_CONTROL_TEXT: &str = "Unit Control limit reached";
pub const BEACON_CAPACITY_TEXT: &str = "Beacon Capacity limit reached";
pub const TUNNEL_SPACE_TEXT: &str = "Tunnel Space limit reached";
/// The message of a building that can't be placed for no rule the check names (the sim refused the command
/// otherwise). Placeholder wording, like those of [`placement_text`].
pub const CANNOT_PLACE_TEXT: &str = "Cannot place there";
/// The message of a Deploy whose Drone stopped short of its site (design change 0117). Placeholder wording.
pub const SITE_UNREACHED_TEXT: &str = "Could not reach the site";
/// The message of a Train pressed with every queue it would go to full (design change 0162). The design's wording.
pub const QUEUE_FULL_TEXT: &str = "Queue is full";
/// The message of a Cancel pressed with nothing to cancel. Placeholder wording.
pub const NOTHING_TO_CANCEL_TEXT: &str = "Nothing to cancel";
/// The message of a building to construct pressed while its constructor constructs or holds a building ready
/// (`factions/gdo/structures/deployment_center.md > ObjectInterfaceState[DeploymentCenter]`: "Other build options
/// unavailable"). Placeholder wording.
pub const BUSY_TEXT: &str = "Already building";
/// The message of an Attack pressed while every recipient is a Peacekeeper (Shielded), its shield up or on its way
/// up (design change 0202). Placeholder wording, after `factions/gdo/units/peacekeeper.md > Riot Shield`: "the
/// player must lower the shield before ordering an attack".
pub const SHIELDS_UP_TEXT: &str = "Lower the Riot Shield first";
/// The message of another command pressed while one awaits its target or a building is placed (design change 0066:
/// "To change commands, the player cancels first"). Placeholder wording.
pub const CANCEL_FIRST_TEXT: &str = "Cancel the current command first";
/// The message of a command the type can't carry out (Reverse, AttackGround). Placeholder wording.
pub const NOT_FOR_TYPE_TEXT: &str = "Not available for this unit";
/// The message of a command or building that isn't in the game yet (design change 0124: "shown but unavailable
/// until defined"). Placeholder wording.
pub const NOT_YET_TEXT: &str = "Not available yet";
/// The message of a Place button pressed while its building is under construction, greyed (design changes 0388 and
/// 0392). Placeholder wording.
pub const UNDER_CONSTRUCTION_TEXT: &str = "Still under construction";
/// The message of a refusal the others don't name. Placeholder wording.
pub const CANNOT_DO_TEXT: &str = "Cannot do that now";
/// The message of an Eject of a unit type none of which is in the Tunnel Network free to leave
/// (`Reject::NotInNetwork`; `factions/syndicate/tunnel_menus.md > EjectMenu`). Placeholder wording.
pub const NONE_INSIDE_TEXT: &str = "None in the Tunnel Network";
/// The message of an Eject while every unit of the type only just walked in (`Reject::InTransit`;
/// `factions/syndicate/tunnel_menus.md > EjectMenu`, design change 0288 point 1: "its tile is greyed and the click is
/// refused; nothing is queued"). Placeholder wording.
pub const IN_TRANSIT_TEXT: &str = "Still in transit";
/// The message of an Eject of a unit type this Tunnel's tier doesn't let out (`Reject::NoTransit`;
/// `factions/syndicate/tunnel_menus.md > EjectMenu`: "Unit types whose base category exceeds this Tunnel's tier are
/// visible but greyed out (disabled)"). Placeholder wording.
pub const TIER_TOO_LOW_TEXT: &str = "This Tunnel's tier is too low";
/// The message of an expansion pressed or placed while its Tunnel performs an operation (`Reject::OperationUnderWay`;
/// `factions/syndicate/tunnel_menus.md > ExpandMenu`: "begins construction if the Tunnel is not already performing an
/// operation"). Placeholder wording.
pub const OPERATION_UNDER_WAY_TEXT: &str = "This Tunnel is busy";
/// The message of an Attack its unit can't carry out, the target inside its MinRange and the unit unable to move
/// (`SimEvent::TargetTooClose`), and of an Attack Ground whose location is (`SimEvent::LocationTooClose`, slice M5-O3).
/// The design's wording (design change 0182).
pub const TARGET_TOO_CLOSE_TEXT: &str = "Target too close";
/// The message of an Enter that ended at a building's entrance with no room in its pool (`SimEvent::EnterFailed` with
/// `full`; slice M5-F1; design change 0347 point 2, `factions/cults/structures/README.md > Entering pool buildings`:
/// "a unit arriving when the building is full has its Enter **end** and idles beside the building"). Placeholder
/// wording.
pub const BUILDING_FULL_TEXT: &str = "No room inside";
/// The message of a pool building's training or Release, or a kind button, pressed while no Recruit of its kind is
/// free inside (`Reject::NoRecruit`, `sc_sim::Refusal::NoRecruit`; design change 0337: "it is greyed if there are
/// none", "the second press is greyed"; slice M5-F2), and of an Altar's order, Release or tile while no unit of its
/// type and state is (slice M5-N1b: a Militant, a Gunner or a Disciple as well as a Recruit). Placeholder wording.
pub const NO_RECRUIT_TEXT: &str = "No free unit of that kind";
/// The message of an Eject All pressed with nothing in the pool (`Reject::NothingInside`;
/// `factions/cults/structures/armory.md > ObjectInterfaceState[Armory]`: "Only available if StoredRecruits is not
/// empty"), of any building that holds units. Placeholder wording.
pub const NOTHING_INSIDE_TEXT: &str = "No free unit inside";
/// (M5-N1b) The message of an Altar's order for units that have an Augmentation it applies (`Reject::AlreadyAugmented`;
/// `factions/cults/augmentation.md > Altar > Augmentation Orders`: "An order can only target a unit that is missing
/// every Augmentation the order applies"), which the panel never shows. Placeholder wording.
pub const ALREADY_AUGMENTED_TEXT: &str = "Already has that Augmentation";
/// (M5-N3) The message of a Temple's key while its line's next level is queued or in progress at one of the player's
/// structures (`Reject::ResearchUnderWay`; `factions/cults/structures/temple.md > Placeholder stats`: "its key shows
/// "In progress" meanwhile"). The design's words.
pub const IN_PROGRESS_TEXT: &str = "In progress";
/// (M5-N3) The message of a technology the player has completed (`Reject::Researched`; `core/tech_and_research.md >
/// Research > Shared Rules` 4: "Once completed, it no longer appears in any research menu"), which the panel never
/// shows. Placeholder wording.
pub const RESEARCHED_TEXT: &str = "Already researched";
/// (M5-N4b) The message of a spell pressed while none of its casters has the energy it needs (`Reject::NotEnoughEnergy`;
/// `factions/cults/units/disciple.md > Placeholder stats > Shroud numbers`: "needs at least **10 energy** to start").
/// Placeholder wording.
pub const NOT_ENOUGH_ENERGY_TEXT: &str = "Not enough energy";
/// (M6-GR) The message of a Scan pressed while its station cools down (`Reject::CoolingDown`;
/// `factions/gdo/structures/radar_station.md > Placeholder stats`: "**cooldown 60 seconds** (960 frames)").
/// Placeholder wording.
pub const COOLING_DOWN_TEXT: &str = "Cooling down";
/// The message of a Drop Off pressed while none of its workers carries a load (`Reject::NothingToDeliver`;
/// `factions/syndicate/units/agent.md > Agent ObjectInterfaceState`: "greyed out when Agent is not carrying
/// resources"). Placeholder wording.
pub const NOTHING_CARRIED_TEXT: &str = "Not carrying resources";
/// The message of a Tunnel's Upgrade pressed at its top tier (`Reject::TopTier`, slice M4-G;
/// `factions/syndicate/tunnel_menus.md > Tunnel ObjectInterfaceState`: "Unavailable if already Tier 3"). Placeholder
/// wording.
pub const TOP_TIER_TEXT: &str = "Already at the top tier";
/// The message of a Tunnel's Upgrade that something blocks, by the truth (`Reject::UpgradeBlocked`, slice M4-G;
/// `factions/syndicate/tunnel_area_and_space.md > Tunnel Area Non-Overlap Rule`: "the upgrading Tunnel's owner learns
/// that something blocks it"): it never says what, as the owner may not see it. Placeholder wording (the brief's
/// "something blocks it").
pub const UPGRADE_BLOCKED_TEXT: &str = "Something blocks the upgrade";
/// (M6-GD) The message of an order that would move a Drone Operator whose drone is out (`Reject::DroneOut`;
/// `factions/gdo/units/drone_operator.md > Launching and piloting`: "The Drone Operator **cannot move** while it has a
/// drone active"), of a second Launch Drone ("Only **one drone can be active at a time** per operator") and of a Build
/// Drone meanwhile (design change 0421: "Build Drone can start only once the operator's drone has **ended**").
/// Placeholder wording.
pub const DRONE_OUT_TEXT: &str = "The drone is out";
/// (M6-GD) The message of a Build Drone with the stock full (`Reject::StockFull`; `factions/gdo/units/drone_operator.md >
/// Drone Stock`: "holds a **maximum of 1**"). Placeholder wording.
pub const STOCK_FULL_TEXT: &str = "Already carries a drone";
/// (M6-GD) The message of a Build Drone while one is bought (`Reject::PurchaseUnderWay`). Placeholder wording.
pub const PURCHASE_UNDER_WAY_TEXT: &str = "Already building a drone";
/// (M6-GD) The message of a Launch Drone with none in stock (`Reject::StockEmpty`; `factions/gdo/units/drone_operator.md
/// > Drone Stock`: "once its drone has been used, it purchases/builds a replacement"). Placeholder wording.
pub const STOCK_EMPTY_TEXT: &str = "No drone to launch";
/// (M6-GH1) The message of an Airfield's Q with its assignments and its queue at its capacity, and of a Rebase onto it
/// (`Reject::AirfieldFull`, `Refusal::AirfieldFull`; `m6-plan.md > GH1 > Acceptance`: "Q is refused ("Airfield
/// full")"; `factions/gdo/structures/airfield.md > Aircraft Support`).
pub const AIRFIELD_FULL_TEXT: &str = "Airfield full";
/// (M6-GH1) The message of a Return to Base for aircraft without a base (`Reject::NoBase`;
/// `systems/mechanics/aircraft_landing.md > Assigned base`: "An unassigned aircraft has no base [...] Return to
/// Base is unavailable until it is reassigned"). Placeholder wording.
pub const NO_BASE_TEXT: &str = "No Airfield assigned";
/// (M6-GH1) The message of an Attack or an Attack Ground for aircraft with no ammo left (`Reject::OutOfAmmo`;
/// `factions/gdo/units/hornet.md > AttackAttributes`: "Ammo: **5 missiles**, then it must reload at its assigned
/// Airfield"). Placeholder wording.
pub const OUT_OF_AMMO_TEXT: &str = "Out of ammo";
/// (M6-GH1) The message of a Rebase onto what isn't one of the player's Airfields (`Reject::NotAnAirfield`).
/// Placeholder wording; no click gives one.
pub const NOT_AN_AIRFIELD_TEXT: &str = "Not an Airfield";
/// (M6-GA) The message of an order that would move units of a type that can't move (`Reject::CannotMove`;
/// `factions/gdo/units/artillery_rig.md > Modes`: "**Deployed:** cannot move"), and of such an order a deploy dropped
/// (`SimEvent::OrderDropped`). Placeholder (not in the design: the wording).
pub const CANNOT_MOVE_TEXT: &str = "Cannot move while deployed";
/// (M6-GA) The message of a Deploy or an Undeploy while every one of its units deploys already (`Reject::Deploying`;
/// `factions/gdo/units/artillery_rig.md > Modes`: "**Undeploy is only available once deployment is complete**").
/// Placeholder (not in the design: the wording).
pub const DEPLOYING_TEXT: &str = "Still switching modes";
/// (M6-CM) The message of an illusion's R or F, its spells (`Reject::IllusionsCantCast`;
/// `factions/cults/units/disciple.md > Spells > Mirage`: "**No spells**: illusions can't cast"). Placeholder wording,
/// which only the illusion's owner, who commands it, ever reads.
pub const ILLUSIONS_CANT_CAST_TEXT: &str = "Illusions can't cast";
/// (M6-SB) The message of a toggle switched on with its tank empty (`Reject::TankEmpty`;
/// `factions/syndicate/units/shadebringer.md > Charges and recharging`: "When a tank runs **empty**, that ability
/// switches off and can't be switched back on until the tank has some charge again"). Placeholder wording.
pub const TANK_EMPTY_TEXT: &str = "Tank empty";
/// The message of a Schedule Deliveries pressed while none of its Supply Towers has a chopper attached
/// (`Reject::NoChopperAttached`; `factions/gdo/economy.md > SupplyTower > ObjectInterfaceState[SupplyTower]`: "Only
/// available if tower has an attached chopper"). Placeholder wording.
pub const NO_CHOPPER_TEXT: &str = "No chopper attached";
/// The message of a chopper's order on a Supply Tower another chopper is attached to (`Reject::TowerTaken`;
/// `factions/gdo/economy.md > SupplyTower > Landing and Attachment`: "When a Supply Tower has an attached Supply
/// Chopper, no other Supply Chopper may land on it"), and of a Pick Up Supplies or a Drop Off Supplies on what isn't a
/// station or an own Supply Tower (`Reject::NotAStation`, `Reject::NotASupplyTower`), which no click gives.
/// Placeholder wording.
pub const TOWER_TAKEN_TEXT: &str = "Another chopper is attached";
pub const NOT_A_STATION_TEXT: &str = "Not a Supply Delivery Station";
pub const NOT_A_TOWER_TEXT: &str = "Not one of your Supply Towers";
/// Placeholder look: amber text on a dark band, this far above the HUD Bar (percent of the surface height).
const TEXT_COLOR: Color = Color::srgb(1.0, 0.80, 0.30);
const BAND_COLOR: Color = Color::srgba(0.0, 0.0, 0.0, 0.55);
const TEXT_VH: f32 = 2.4;
const ABOVE_HUD_VH: f32 = 3.0;

/// The latest message that still shows.
#[derive(Resource, Debug)]
pub struct TargetFeedback {
    /// The sim's tick of its cause; `None` while no message shows.
    pub since: Option<Tick>,
    /// What it says.
    pub text: &'static str,
    /// The `SimRes::generation` it belongs to: a replaced sim shows none.
    generation: Option<u32>,
    /// (M5-X2) How many ticks it shows: a refusal of the data files' longer than the rest.
    lasts: u32,
    /// (M5-Mb) The player whose message it is: the local player its cause came from (their click or press, or an
    /// event of theirs, `crate::viewer_events`). It shows to them alone, so the debug player switch (F2) takes it off
    /// the line: another player's refusal ("Requires …", "Not enough …") is none of the new local player's. `None` for
    /// a data directory's refusal, which is nobody's and shows to whoever plays.
    owner: Option<PlayerId>,
}

impl Default for TargetFeedback {
    fn default() -> Self {
        Self {
            since: None,
            text: INVALID_TARGET_TEXT,
            generation: None,
            lasts: 0,
            owner: None,
        }
    }
}

/// The message's node, shown while [`TargetFeedback::since`] is set, in the game modes.
#[derive(Component)]
pub struct TargetFeedbackView;

/// The message's text.
#[derive(Component)]
struct FeedbackText;

pub struct FeedbackPlugin;

impl Plugin for FeedbackPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TargetFeedback>()
            .add_systems(Startup, spawn_feedback)
            .add_systems(Update, show_feedback.in_set(ClientSystems::Present));
    }
}

fn spawn_feedback(mut commands: Commands) {
    commands.spawn((
        Name::new("Target feedback"),
        TargetFeedbackView,
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            width: percent(100),
            bottom: vh(HUD_BAR_HEIGHT_VH + ABOVE_HUD_VH),
            justify_content: JustifyContent::Center,
            ..default()
        },
        Pickable::IGNORE,
        Visibility::Hidden,
        children![(
            Node {
                padding: UiRect::axes(vh(1.2), vh(0.4)),
                ..default()
            },
            BackgroundColor(BAND_COLOR),
            Pickable::IGNORE,
            children![(
                FeedbackText,
                Text::new(INVALID_TARGET_TEXT),
                TextFont::from_font_size(FontSize::Vh(TEXT_VH)),
                TextColor(TEXT_COLOR),
                Pickable::IGNORE,
            )],
        )],
    ));
}

/// The message of a producer that didn't queue a unit others of its command took (design change 0083): for want of
/// Space Crystals, (M6-GD) of Supplies (the Drone Operator's: design change 0441, "greyed without it, with a feedback
/// line that names Supplies"), or of a free Recruit of the kind at a pool building (slice M5-F2); none for a full
/// queue.
fn refusal_text(reason: Refusal) -> Option<&'static str> {
    match reason {
        Refusal::Crystals => Some(NO_CRYSTALS_TEXT),
        Refusal::Supplies => Some(NO_SUPPLIES_TEXT),
        Refusal::QueueFull => None,
        Refusal::NoRecruit => Some(NO_RECRUIT_TEXT),
        Refusal::AirfieldFull => Some(AIRFIELD_FULL_TEXT),
    }
}

/// The message of a producer whose unit waits for room under the army cap `resource` (`SimEvent::ProductionWaiting`,
/// design change 0143; placeholder for its notification), named after it: Unit Control, the Colonists' Beacon
/// Capacity, or the Syndicate's Tunnel Space (slice M4-B). The Cults' Unit Control makes nothing wait
/// (`ArmyResource::reserves`, slice M5-C), and would read as the GDO's.
pub fn army_cap_text(resource: ArmyResource) -> &'static str {
    match resource {
        ArmyResource::BeaconCapacity => BEACON_CAPACITY_TEXT,
        ArmyResource::UnitControl | ArmyResource::CultsUnitControl => UNIT_CONTROL_TEXT,
        ArmyResource::TunnelSpace => TUNNEL_SPACE_TEXT,
    }
}

/// The message of a building that can't be placed, by the first rule its placement breaks
/// (`sc_sim::PlacementCheck::first_problem`; `core/entities.md > Placement Validation`). Placeholder wordings: the
/// design gives none.
pub fn placement_text(problem: Option<PlacementProblem>) -> &'static str {
    match problem {
        Some(PlacementProblem::Whole(problem)) => match problem {
            WholeProblem::NotAStructure => CANNOT_PLACE_TEXT,
            WholeProblem::OutsideBuildArea => "Outside the build area",
            WholeProblem::NotOnPatch => "Must be placed on a crystal patch",
            WholeProblem::TunnelAreaOffMap => "The Tunnel Area reaches off the map",
            WholeProblem::TunnelAreaNotDrillable => "The Tunnel Area must be drillable ground",
            WholeProblem::TunnelAreaOverlap => "The Tunnel Area overlaps another Tunnel's",
            WholeProblem::OutsideTunnelArea => "Must be inside a Tunnel Area",
        },
        Some(PlacementProblem::Tile(_, problem)) => match problem {
            TileProblem::OffMap => "Outside the map",
            TileProblem::NotBuildable => "Cannot build on this ground",
            TileProblem::NotDrillable => "Cannot drill this ground",
            TileProblem::NotFlat => "Ground is not flat",
            TileProblem::NotVisible => "Cannot build where you cannot see",
            TileProblem::Structure => "Blocked by a structure",
            TileProblem::Resource => "Blocked by a resource",
            TileProblem::Unit => "Blocked by a unit",
            TileProblem::Reserved => "A building is already ordered there",
        },
        None => CANNOT_PLACE_TEXT,
    }
}

/// The message of a greyed-out button pressed (design change 0162), by why it is greyed out: the gate's refusal
/// ([`reject_text`]), the Riot Shields, a command awaiting its target, the type, that it isn't in the game yet, or a
/// Place button's building under construction.
pub fn unavailable_text(why: WhyUnavailable) -> &'static str {
    match why {
        WhyUnavailable::Refused(reject) => reject_text(reject),
        WhyUnavailable::ShieldsUp => SHIELDS_UP_TEXT,
        WhyUnavailable::Waiting => CANCEL_FIRST_TEXT,
        WhyUnavailable::NotForThisType => NOT_FOR_TYPE_TEXT,
        WhyUnavailable::NotYet => NOT_YET_TEXT,
        WhyUnavailable::UnderConstruction => UNDER_CONSTRUCTION_TEXT,
    }
}

/// The message of a command the gate refuses (`sc_sim::Reject`), as a greyed-out button or a Warp In click says it:
/// "Not enough Space Crystals", "Queue is full" (design change 0162), the placement's first problem
/// ([`placement_text`]), an EjectMenu tile's (slice M4-J2), an ExpandMenu entry's and the Tunnel's Cancel's (slice
/// M4-F: its operation, an expansion's Tier Requirement above its tier, nothing to cancel), a prerequisite missing
/// ([`requires_text`], slice M4-K), the Tunnel's Upgrade's (slice M4-G: its top tier, something blocking it), a pool
/// building's (slice M5-F2: no free Recruit of the kind, nothing to eject), a Temple's key's (slice M5-N3: in progress,
/// completed, its previous level missing), (M6-GD) a Drone Operator's (its drone out, its stock full, empty or being
/// filled; Supplies short for the Barracks' R, "a feedback line that names Supplies", design change 0441), (M6-GR) a
/// Scan's (its station cooling down), (M6-GH1) an Airfield's and its aircraft's (the Airfield full, no base, out of
/// ammo, not an Airfield), (M6-GA) a vehicle's (it can't move, it deploys already), (M6-CM) an illusion's spells'
/// ([`ILLUSIONS_CANT_CAST_TEXT`]), (M6-SB) a tank's toggle's ([`TANK_EMPTY_TEXT`]), the supply run's (no
/// chopper attached, a Tower taken, not a station, not a Tower), and placeholders for the others.
pub fn reject_text(reject: Reject) -> &'static str {
    match reject {
        Reject::InsufficientCrystals => NO_CRYSTALS_TEXT,
        Reject::InsufficientSupplies => NO_SUPPLIES_TEXT,
        Reject::QueueFull => QUEUE_FULL_TEXT,
        Reject::NothingQueued
        | Reject::NotConstructing
        | Reject::NothingReady
        | Reject::NotTransforming
        | Reject::NoOperation(_) => NOTHING_TO_CANCEL_TEXT,
        Reject::ConstructorBusy(_) => BUSY_TEXT,
        Reject::CannotAttack => SHIELDS_UP_TEXT,
        Reject::Unplaceable(problem) => placement_text(Some(problem)),
        Reject::NotInNetwork(_) => NONE_INSIDE_TEXT,
        Reject::InTransit(_) => IN_TRANSIT_TEXT,
        Reject::NoTransit { .. } | Reject::TierRequirement { .. } => TIER_TOO_LOW_TEXT,
        Reject::OperationUnderWay(_) => OPERATION_UNDER_WAY_TEXT,
        Reject::NothingToDeliver => NOTHING_CARRIED_TEXT,
        Reject::Requires(kind) => requires_text(kind),
        Reject::TopTier(_) => TOP_TIER_TEXT,
        Reject::UpgradeBlocked(_) => UPGRADE_BLOCKED_TEXT,
        Reject::NoRecruit(_) => NO_RECRUIT_TEXT,
        Reject::NothingInside => NOTHING_INSIDE_TEXT,
        Reject::AlreadyAugmented(_) => ALREADY_AUGMENTED_TEXT,
        Reject::ResearchUnderWay(_) => IN_PROGRESS_TEXT,
        Reject::Researched(_) => RESEARCHED_TEXT,
        Reject::RequiresTech(tech) => requires_tech_text(tech),
        Reject::NotEnoughEnergy => NOT_ENOUGH_ENERGY_TEXT,
        Reject::DroneOut => DRONE_OUT_TEXT,
        Reject::StockFull => STOCK_FULL_TEXT,
        Reject::PurchaseUnderWay => PURCHASE_UNDER_WAY_TEXT,
        Reject::StockEmpty => STOCK_EMPTY_TEXT,
        Reject::CoolingDown => COOLING_DOWN_TEXT,
        Reject::AirfieldFull => AIRFIELD_FULL_TEXT,
        Reject::NoBase => NO_BASE_TEXT,
        Reject::OutOfAmmo => OUT_OF_AMMO_TEXT,
        Reject::NotAnAirfield(_) => NOT_AN_AIRFIELD_TEXT,
        Reject::CannotMove => CANNOT_MOVE_TEXT,
        Reject::Deploying => DEPLOYING_TEXT,
        Reject::IllusionsCantCast => ILLUSIONS_CANT_CAST_TEXT,
        Reject::TankEmpty => TANK_EMPTY_TEXT,
        Reject::NoChopperAttached => NO_CHOPPER_TEXT,
        Reject::TowerTaken(_) => TOWER_TAKEN_TEXT,
        Reject::NotAStation(_) => NOT_A_STATION_TEXT,
        Reject::NotASupplyTower(_) => NOT_A_TOWER_TEXT,
        _ => CANNOT_DO_TEXT,
    }
}

/// The message of an entry whose prerequisite the player doesn't own (`Reject::Requires`, slice M4-K;
/// `core/tech_and_research.md > Tech Prerequisites`: "Options whose TechPrerequisite is not met are unavailable in the
/// build menu"; `factions/syndicate/structures/hideout.md > ObjectInterfaceState[Hideout]`: "Only available if the
/// player owns at least one Optics Lab"): "Requires", the type's name and its article, "Requires an Optics Lab",
/// "Requires a Chemical Plant". The M4 plan's wording. Each type's is made once, the first time one is asked for, and
/// lives as long as the game, as the messages do (`TargetFeedback::text`).
pub fn requires_text(kind: ObjectKind) -> &'static str {
    static TEXTS: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
        (ObjectKind::ALL.iter())
            .map(|kind| {
                let name = kind.def().name;
                let article = if name.starts_with(['A', 'E', 'I', 'O', 'U']) {
                    "an"
                } else {
                    "a"
                };
                &*format!("Requires {article} {name}").leak()
            })
            .collect()
    });
    TEXTS[kind as usize]
}

/// (M5-N3) The message of a technology whose previous level the player hasn't completed (`Reject::RequiresTech`;
/// `core/tech_and_research.md > Tech Prerequisites > Research Prerequisites`: "A next level can only be queued once
/// the previous level has **completed**"), which the panel never shows, as its key shows the previous level until
/// then: "Requires" and the technology's name, as a building's prerequisite says it ([`requires_text`]).
pub fn requires_tech_text(tech: Tech) -> &'static str {
    static TEXTS: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
        (Tech::ALL.iter())
            .map(|tech| &*format!("Requires {}", tech.def().name).leak())
            .collect()
    });
    TEXTS[tech as usize]
}

/// (M6-GA) The message of an order of `unit`'s that its vehicle deploy dropped as it ended (`SimEvent::OrderDropped`):
/// the gate's refusal of that order, unqueued, to the unit as it is now ([`reject_text`]), as the order's greyed button
/// or a refused click says it (`factions/gdo/units/artillery_rig.md > Modes`: "the usual unavailable-command feedback";
/// design change 0162: "the same feedback as when an order is refused, naming the reason"): "Cannot move while
/// deployed" for a Move. An Attack or an Attack Ground the unit's type can't carry out (`Reject::CannotAttack`,
/// `Reject::CannotTargetGround`: a Rig undeployed) is [`NOT_FOR_TYPE_TEXT`], as a greyed Attack Ground says it, not a
/// Riot Shield's [`SHIELDS_UP_TEXT`]; so is an order the gate would take now, or one of a unit that is gone.
pub fn dropped_text(sim: &Sim, unit: ObjectId, order: OrderKind) -> &'static str {
    let Some(player) = sim.object(unit).and_then(|o| o.owner) else {
        return NOT_FOR_TYPE_TEXT;
    };
    let command = Command::Order {
        units: vec![unit],
        kind: order,
        queue: false,
    };
    match sim.validate(&PlayerCommand { player, command }) {
        Err(Reject::CannotAttack | Reject::CannotTargetGround) | Ok(()) => NOT_FOR_TYPE_TEXT,
        Err(reject) => reject_text(reject),
    }
}

/// The message of a Deploy or a Build that failed on arrival (`SimEvent::DeployFailed`, `SimEvent::BuildFailed`): the
/// first problem the check of its site found ([`placement_text`]), or, with none, that the worker stopped short of the
/// footprint.
pub fn deploy_failed_text(problems: &PlacementCheck) -> &'static str {
    match problems.first_problem() {
        Some(problem) => placement_text(Some(problem)),
        None => SITE_UNREACHED_TEXT,
    }
}

/// Takes the frame's invalid clicks (on a target, or a placement), its greyed-out buttons pressed, the refusals and
/// the waits of the producers, constructors and units, (M6-GA) the orders their deploys dropped, that the local player
/// perceives (their own:
/// `crate::viewer_events`), and (M5-X2) the data files' refusals, and shows the latest one's message for
/// `FeedbackTuning::feedback_ticks` (a data file's for `FeedbackTuning::refusal_ticks`). (M5-Mb) A message shows to
/// the player it belongs to ([`TargetFeedback::owner`]): once F2 switches to another player, an earlier one of the
/// player before goes, and doesn't come back on the switch back.
#[expect(clippy::too_many_arguments, reason = "a system's parameters")]
fn show_feedback(
    mode: Res<ClientMode>,
    sim: Res<SimRes>,
    sight: LocalSight,
    tuning: Res<ClientTuning>,
    mut invalid: MessageReader<InvalidTarget>,
    mut unplaceable: MessageReader<InvalidPlacement>,
    mut unavailable: MessageReader<CommandRefused>,
    mut data: MessageReader<DataRefused>,
    events: Res<ViewerEvents>,
    mut feedback: ResMut<TargetFeedback>,
    mut views: Query<&mut Visibility, With<TargetFeedbackView>>,
    mut texts: Query<&mut Text, With<FeedbackText>>,
) {
    let clicks = invalid.read().map(|click| (click.tick, INVALID_TARGET_TEXT));
    let placements = unplaceable
        .read()
        .map(|click| (click.tick, placement_text(click.problem)));
    let presses = unavailable
        .read()
        .map(|press| (press.tick, unavailable_text(press.why)));
    let refusals = (events.perceived_by(sight.sim(), sight.get())).filter_map(|(tick, event, _)| match event {
        SimEvent::ProductionRefused { reason, .. } => Some((tick, refusal_text(*reason)?)),
        SimEvent::ProductionWaiting { reason, .. } => Some((tick, army_cap_text(*reason))),
        SimEvent::PlacementRefused { problems, .. } => Some((tick, placement_text(problems.first_problem()))),
        SimEvent::DeployFailed { problems, .. } | SimEvent::BuildFailed { problems, .. } => {
            Some((tick, deploy_failed_text(problems)))
        }
        SimEvent::TargetTooClose { .. } | SimEvent::LocationTooClose { .. } => Some((tick, TARGET_TOO_CLOSE_TEXT)),
        SimEvent::EnterFailed { full: true, .. } => Some((tick, BUILDING_FULL_TEXT)),
        SimEvent::OrderDropped { unit, order } => Some((tick, dropped_text(&sim, *unit, *order))),
        _ => None,
    });
    let lasts = tuning.feedback.feedback_ticks;
    let local = Some(sight.get().player());
    let files = (data.read()).map(|refused| (refused.tick, refused.text, tuning.feedback.refusal_ticks, None));
    let latest = (clicks.chain(placements).chain(presses).chain(refusals))
        .map(|(tick, text)| (tick, text, lasts, local))
        .chain(files)
        .max_by_key(|&(tick, _, _, _)| tick);
    let generation = Some(sim.generation());
    let earlier = feedback
        .since
        .filter(|_| feedback.generation == generation && feedback.owner.is_none_or(|owner| Some(owner) == local))
        .map(|since| (since, feedback.text, feedback.lasts, feedback.owner));
    let shown = latest
        .or(earlier)
        .filter(|&(tick, _, lasts, _)| sim.tick().saturating_sub(tick) < lasts);
    let (since, text, lasts, owner) = shown.map_or(
        (None, feedback.text, feedback.lasts, feedback.owner),
        |(tick, text, lasts, owner)| (Some(tick), text, lasts, owner),
    );
    if feedback.since != since
        || feedback.text != text
        || feedback.generation != generation
        || feedback.lasts != lasts
        || feedback.owner != owner
    {
        feedback.since = since;
        feedback.text = text;
        feedback.generation = generation;
        feedback.lasts = lasts;
        feedback.owner = owner;
    }
    for mut shown_text in &mut texts {
        if shown_text.0 != text {
            shown_text.0 = text.to_string();
        }
    }
    let visibility = if since.is_some() && mode.is_game() {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    for mut view in &mut views {
        view.set_if_neq(visibility);
    }
}
