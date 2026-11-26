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
//!   wordings but the design's two.
//!
//! The sim's events among these come through the filter of events (`crate::viewer_events`), which gives each to the
//! owner of its producer, constructor or unit alone.
//!
//! The message ([`INVALID_TARGET_TEXT`], [`NO_CRYSTALS_TEXT`], an [`army_cap_text`], a [`placement_text`], an
//! [`unavailable_text`], [`TARGET_TOO_CLOSE_TEXT`] or [`BUILDING_FULL_TEXT`]) shows over the world view, centred just
//! above the HUD Bar, on a dark band, for [`FEEDBACK_TICKS`] after the latest cause; a new one replaces it. It ages by
//! the sim's ticks, so a QA screenshot of a tick always shows it alike. It never takes the pointer: clicks pass
//! through it to the map.

use std::sync::LazyLock;

use bevy::picking::Pickable;
use bevy::prelude::*;
use sc_sim::{
    ArmyResource, ObjectKind, PlacementCheck, PlacementProblem, Refusal, Reject, SimEvent, Tech, Tick, TileProblem,
    WholeProblem,
};

use super::{CommandRefused, InvalidPlacement, InvalidTarget, WhyUnavailable};
use crate::fog::LocalSight;
use crate::hud::HUD_BAR_HEIGHT_VH;
use crate::sim_driver::SimRes;
use crate::viewer_events::ViewerEvents;
use crate::{ClientMode, ClientSystems};

pub use crate::tuning::feedback::FEEDBACK_TICKS;

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
}

impl Default for TargetFeedback {
    fn default() -> Self {
        Self {
            since: None,
            text: INVALID_TARGET_TEXT,
            generation: None,
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
/// Space Crystals, or of a free Recruit of the kind at a pool building (slice M5-F2); none for a full queue.
fn refusal_text(reason: Refusal) -> Option<&'static str> {
    match reason {
        Refusal::Crystals => Some(NO_CRYSTALS_TEXT),
        Refusal::QueueFull => None,
        Refusal::NoRecruit => Some(NO_RECRUIT_TEXT),
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
            WholeProblem::RecruitmentAreaOffMap => "The Recruitment Area reaches off the map",
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
/// ([`reject_text`]), the Riot Shields, a command awaiting its target, the type, or that it isn't in the game yet.
pub fn unavailable_text(why: WhyUnavailable) -> &'static str {
    match why {
        WhyUnavailable::Refused(reject) => reject_text(reject),
        WhyUnavailable::ShieldsUp => SHIELDS_UP_TEXT,
        WhyUnavailable::Waiting => CANCEL_FIRST_TEXT,
        WhyUnavailable::NotForThisType => NOT_FOR_TYPE_TEXT,
        WhyUnavailable::NotYet => NOT_YET_TEXT,
    }
}

/// The message of a command the gate refuses (`sc_sim::Reject`), as a greyed-out button or a Warp In click says it:
/// "Not enough Space Crystals", "Queue is full" (design change 0162), the placement's first problem
/// ([`placement_text`]), an EjectMenu tile's (slice M4-J2), an ExpandMenu entry's and the Tunnel's Cancel's (slice
/// M4-F: its operation, an expansion's Tier Requirement above its tier, nothing to cancel), a prerequisite missing
/// ([`requires_text`], slice M4-K), the Tunnel's Upgrade's (slice M4-G: its top tier, something blocking it), a pool
/// building's (slice M5-F2: no free Recruit of the kind, nothing to eject), a Temple's key's (slice M5-N3: in progress,
/// completed, its previous level missing), and placeholders for the others.
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
/// the waits of the producers, constructors and units that the local player perceives (their own:
/// `crate::viewer_events`), and shows the latest one's message for [`FEEDBACK_TICKS`].
#[expect(clippy::too_many_arguments, reason = "a system's parameters")]
fn show_feedback(
    mode: Res<ClientMode>,
    sim: Res<SimRes>,
    sight: LocalSight,
    mut invalid: MessageReader<InvalidTarget>,
    mut unplaceable: MessageReader<InvalidPlacement>,
    mut unavailable: MessageReader<CommandRefused>,
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
        _ => None,
    });
    let latest = clicks
        .chain(placements)
        .chain(presses)
        .chain(refusals)
        .max_by_key(|&(tick, _)| tick);
    let generation = Some(sim.generation());
    let earlier = feedback
        .since
        .filter(|_| feedback.generation == generation)
        .map(|since| (since, feedback.text));
    let shown = latest
        .or(earlier)
        .filter(|&(tick, _)| sim.tick().saturating_sub(tick) < FEEDBACK_TICKS);
    let (since, text) = shown.map_or((None, feedback.text), |(tick, text)| (Some(tick), text));
    if feedback.since != since || feedback.text != text || feedback.generation != generation {
        feedback.since = since;
        feedback.text = text;
        feedback.generation = generation;
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
