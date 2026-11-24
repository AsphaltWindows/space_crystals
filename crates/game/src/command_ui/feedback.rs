//! Feedback on a click on an invalid target (`systems/controls/selection_and_hud.md > AwaitingTarget[CommandType] >
//! Invalid target clicks`, design change 0065): the click is ignored, the target is still awaited, and the player
//! gets "short feedback (an 'invalid target' message and/or sound; exact form TBD, ties into the future notification
//! system)".
//!
//! Placeholder until that system: the message [`INVALID_TARGET_TEXT`] over the world view, centred just above the
//! HUD Bar, on a dark band, for [`FEEDBACK_TICKS`] after the latest invalid click ([`InvalidTarget`]). It ages by the
//! sim's ticks, so a QA screenshot of a tick always shows it alike. It never takes the pointer: clicks pass through
//! it to the map.

use bevy::picking::Pickable;
use bevy::prelude::*;
use sc_sim::Tick;

use super::InvalidTarget;
use crate::hud::HUD_BAR_HEIGHT_VH;
use crate::sim_driver::SimRes;
use crate::{ClientMode, ClientSystems};

/// How long the message shows, ticks (2 s). Placeholder: the design says "short".
pub const FEEDBACK_TICKS: u32 = 32;
/// The message. Placeholder wording, the design's.
pub const INVALID_TARGET_TEXT: &str = "Invalid target";
/// Placeholder look: amber text on a dark band, this far above the HUD Bar (percent of the surface height).
const TEXT_COLOR: Color = Color::srgb(1.0, 0.80, 0.30);
const BAND_COLOR: Color = Color::srgba(0.0, 0.0, 0.0, 0.55);
const TEXT_VH: f32 = 2.4;
const ABOVE_HUD_VH: f32 = 3.0;

/// The latest invalid click whose message still shows.
#[derive(Resource, Default, Debug)]
pub struct TargetFeedback {
    /// The sim's tick of that click; `None` while no message shows.
    pub since: Option<Tick>,
    /// The `SimRes::generation` it belongs to: a replaced sim shows none.
    generation: Option<u32>,
}

/// The message's node, shown while [`TargetFeedback::since`] is set, in the game modes.
#[derive(Component)]
pub struct TargetFeedbackView;

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
                Text::new(INVALID_TARGET_TEXT),
                TextFont::from_font_size(FontSize::Vh(TEXT_VH)),
                TextColor(TEXT_COLOR),
                Pickable::IGNORE,
            )],
        )],
    ));
}

/// Takes the invalid clicks of the frame and shows the message for [`FEEDBACK_TICKS`] after the latest.
fn show_feedback(
    mode: Res<ClientMode>,
    sim: Res<SimRes>,
    mut invalid: MessageReader<InvalidTarget>,
    mut feedback: ResMut<TargetFeedback>,
    mut views: Query<&mut Visibility, With<TargetFeedbackView>>,
) {
    let latest = invalid.read().last().map(|click| click.tick);
    let generation = Some(sim.generation());
    let earlier = feedback.since.filter(|_| feedback.generation == generation);
    let since = latest
        .or(earlier)
        .filter(|&tick| sim.tick().saturating_sub(tick) < FEEDBACK_TICKS);
    if feedback.since != since || feedback.generation != generation {
        feedback.since = since;
        feedback.generation = generation;
    }
    let shown = if since.is_some() && mode.is_game() {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    for mut visibility in &mut views {
        visibility.set_if_neq(shown);
    }
}
