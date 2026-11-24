//! Short feedback messages of the command flow, placeholders until the future notification system:
//! - a click on an invalid target (`systems/controls/selection_and_hud.md > AwaitingTarget[CommandType] > Invalid
//!   target clicks`, design change 0065): the click is ignored, the target is still awaited, and the player gets
//!   "short feedback (an 'invalid target' message and/or sound; exact form TBD, ties into the future notification
//!   system)";
//! - a Train that some of its producers didn't queue for want of Space Crystals (`> SelectionGroup > No
//!   smart-casting > Group production with limited resources`, design change 0083): "The others don't queue, and the
//!   player gets insufficient-resources feedback". The sim tells which (`SimEvent::ProductionRefused`); a full queue
//!   is no resource, and gives none. A Train that no producer would queue is never sent: its button is disabled
//!   (`super::interface::Conditions`). The army cap is checked when production starts, not when a unit is queued
//!   (design change 0143), so it refuses nothing;
//! - a producer whose unit at the front waits at 0% for room under the army cap (`SimEvent::ProductionWaiting`,
//!   design change 0143: "A blocked queue raises a notification (not yet designed)"): placeholder for that
//!   notification, the cap's name ([`army_cap_text`]: Unit Control, or the Colonists' Beacon Capacity, plan decision
//!   T20), once as the wait begins; the producer's InfoPanel shows the wait for as long as it lasts
//!   (`crate::hud::info_panel`);
//! - a building that can't be placed (`factions/gdo/structures.md > DeploymentCenter >
//!   ObjectInterfaceState[DeploymentCenter] > AwaitingPlacement`): a left-click where the ghost is invalid sends
//!   nothing and placement goes on (`super::InvalidPlacement`), and a Place that the sim refused as it applied it,
//!   the game having changed since the click (`SimEvent::PlacementRefused`, e.g. a unit walked onto the footprint),
//!   leaves the building ready to place and the player in DefaultState, where the valid click left them (`>
//!   AwaitingPlacement`: it "returns to DefaultState"); the ready building's slot enters placement again. Both say
//!   why: the placement check's first problem ([`placement_text`]). Placeholders: the design gives no message for
//!   either;
//! - a Deploy that failed at the end of the Drone's walk (`SimEvent::DeployFailed`, slice M3-F; `factions/colonists/
//!   economy.md > Beacon Deployment > Placement`): what the check found on arrival, as for a building, or that the
//!   Drone didn't get to its site ([`deploy_failed_text`]). Placeholders too.
//!
//! The message ([`INVALID_TARGET_TEXT`], [`NO_CRYSTALS_TEXT`], an [`army_cap_text`] or a [`placement_text`]) shows
//! over the world view, centred just above the HUD Bar, on a dark band, for [`FEEDBACK_TICKS`] after the latest cause;
//! a new one replaces it. It ages by the sim's ticks, so a QA screenshot of a tick always shows it alike. It never
//! takes the pointer: clicks pass through it to the map.

use bevy::picking::Pickable;
use bevy::prelude::*;
use sc_sim::{ArmyResource, PlacementCheck, PlacementProblem, Refusal, SimEvent, Tick, TileProblem, WholeProblem};

use super::{InvalidPlacement, InvalidTarget};
use crate::hud::HUD_BAR_HEIGHT_VH;
use crate::sim_driver::{LocalPlayer, SimEventMessage, SimRes};
use crate::{ClientMode, ClientSystems};

/// How long the message shows, ticks (2 s). Placeholder: the design says "short".
pub const FEEDBACK_TICKS: u32 = 32;
/// The message of an invalid target. Placeholder wording, the design's.
pub const INVALID_TARGET_TEXT: &str = "Invalid target";
/// The message of a Train that producers didn't queue for want of Space Crystals (design change 0083). Placeholder
/// wording.
pub const NO_CRYSTALS_TEXT: &str = "Not enough Space Crystals";
/// The messages of a producer that waits for room under the army cap (design change 0143), by the faction's cap (plan
/// decision T20: the faction's name in the feedback; [`army_cap_text`]). Placeholder wordings.
pub const UNIT_CONTROL_TEXT: &str = "Unit Control limit reached";
pub const BEACON_CAPACITY_TEXT: &str = "Beacon Capacity limit reached";
/// The message of a building that can't be placed for no rule the check names (the sim refused the command
/// otherwise). Placeholder wording, like those of [`placement_text`].
pub const CANNOT_PLACE_TEXT: &str = "Cannot place there";
/// The message of a Deploy whose Drone stopped short of its site (design change 0117). Placeholder wording.
pub const SITE_UNREACHED_TEXT: &str = "Could not reach the site";
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
/// Space Crystals; none for a full queue.
fn refusal_text(reason: Refusal) -> Option<&'static str> {
    match reason {
        Refusal::Crystals => Some(NO_CRYSTALS_TEXT),
        Refusal::QueueFull => None,
    }
}

/// The message of a producer whose unit waits for room under the army cap `resource` (`SimEvent::ProductionWaiting`,
/// design change 0143; placeholder for its notification), named after it: Unit Control, or the Colonists' Beacon
/// Capacity.
pub fn army_cap_text(resource: ArmyResource) -> &'static str {
    match resource {
        ArmyResource::BeaconCapacity => BEACON_CAPACITY_TEXT,
        ArmyResource::UnitControl => UNIT_CONTROL_TEXT,
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
        },
        Some(PlacementProblem::Tile(_, problem)) => match problem {
            TileProblem::OffMap => "Outside the map",
            TileProblem::NotBuildable => "Cannot build on this ground",
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

/// The message of a Deploy that failed on arrival (`SimEvent::DeployFailed`): the first problem the check of its site
/// found ([`placement_text`]), or, with none, that the Drone stopped short of the footprint.
pub fn deploy_failed_text(problems: &PlacementCheck) -> &'static str {
    match problems.first_problem() {
        Some(problem) => placement_text(Some(problem)),
        None => SITE_UNREACHED_TEXT,
    }
}

/// Takes the frame's invalid clicks (on a target, or a placement), the local player's producers' and constructors'
/// refusals and their producers' waits, and shows the latest one's message for [`FEEDBACK_TICKS`].
#[expect(clippy::too_many_arguments, reason = "a system's parameters")]
fn show_feedback(
    mode: Res<ClientMode>,
    sim: Res<SimRes>,
    player: Res<LocalPlayer>,
    mut invalid: MessageReader<InvalidTarget>,
    mut unplaceable: MessageReader<InvalidPlacement>,
    mut events: MessageReader<SimEventMessage>,
    mut feedback: ResMut<TargetFeedback>,
    mut views: Query<&mut Visibility, With<TargetFeedbackView>>,
    mut texts: Query<&mut Text, With<FeedbackText>>,
) {
    let clicks = invalid.read().map(|click| (click.tick, INVALID_TARGET_TEXT));
    let placements = unplaceable
        .read()
        .map(|click| (click.tick, placement_text(click.problem)));
    let own = |id| sim.object(id).is_some_and(|o| o.owner == Some(player.0));
    let refusals = events.read().filter_map(|message| match &message.event {
        SimEvent::ProductionRefused { producer, reason, .. } if own(*producer) => {
            Some((message.tick, refusal_text(*reason)?))
        }
        SimEvent::ProductionWaiting { producer, reason } if own(*producer) => {
            Some((message.tick, army_cap_text(*reason)))
        }
        SimEvent::PlacementRefused {
            constructor, problems, ..
        } if own(*constructor) => Some((message.tick, placement_text(problems.first_problem()))),
        SimEvent::DeployFailed { drone, problems, .. } if own(*drone) => {
            Some((message.tick, deploy_failed_text(problems)))
        }
        _ => None,
    });
    let latest = clicks.chain(placements).chain(refusals).max_by_key(|&(tick, _)| tick);
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
