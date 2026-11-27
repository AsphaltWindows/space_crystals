//! The HUD's notices, across the top of the world view under the top bar: placeholders for notifications the design
//! hasn't designed yet. Each shows for `FeedbackTuning::notice_ticks` after its latest event. It ages by the sim's
//! ticks, so a QA screenshot of a tick always shows it alike, and a sim replaced (a load, a replay) shows none of the
//! old one's. A notice never takes the pointer: clicks pass through it to the map.
//!
//! - **The empty-station notice** (slice M4-I): a placeholder for design change 0256 point 3's notification
//!   (`factions/syndicate/units/agent.md > Gathering > Supplies`: "An empty station: the owner gets a notification
//!   (notifications not designed yet)"; 0256's Implementation impact: "M4 may show it in a placeholder form"). When the
//!   sim reports that a Supply Delivery Station where the local player's Agents gather has run out
//!   (`SimEvent::SupplyStationEmpty`, sent to each player whose Agents gather there, and through the filter of events
//!   to that player alone, `crate::viewer_events`), [`STATION_EMPTY_TEXT`] shows.
//! - **The defeat notice** (slice M5-S1; `core/game_rules.md > On Defeat`: "**Everyone learns of it** (user,
//!   2026-11-26; Q0090): when a player is defeated while the game goes on, **all players** are told", design change
//!   0357): a defeat reaches every player (`SimEvent::PlayerDefeated`, whose audience is everyone in the filter of
//!   events), so every player who hasn't left the game for the results screen (`super::results::has_left`) sees a line,
//!   [`defeat_text`], for each other player defeated in the last `FeedbackTuning::notice_ticks`, in the order they
//!   fell. The debug player switch (F2) shows it to each player in turn. The defeated player sees their own defeat on
//!   the results screen instead, or in the outcome banner's Spectating while an ally plays on. Once the game is over,
//!   the banner tells everyone (`core/game_rules.md > End of Game`: "Every player sees Victory or Defeat"), so the
//!   defeat that ends the game shows no notice, and a notice still showing goes.
//!
//! Placeholders, the design giving none of them: the wording, the look, where each shows and for how long.

use bevy::picking::Pickable;
use bevy::prelude::*;
use sc_sim::{PlayerId, SimEvent, Tick};

use super::{TOP_BAR_HEIGHT_VH, results, set_text};
use crate::fog::LocalSight;
use crate::sim_driver::{LocalPlayer, SimRes};
use crate::tuning::ClientTuning;
use crate::viewer_events::ViewerEvents;
use crate::{ClientMode, ClientSystems};

/// The empty-station notice's line. Placeholder wording.
pub const STATION_EMPTY_TEXT: &str = "Supply Delivery Station empty: your Agents wait there for the next delivery";

/// The defeat notice's line for the player called `name`. Placeholder wording.
pub fn defeat_text(name: &str) -> String {
    format!("{name} has been defeated")
}

/// Placeholder look: pale green text on a dark band, this far under the top bar (percent of the surface height).
const TEXT_COLOR: Color = Color::srgb(0.70, 0.92, 0.70);
const BAND_COLOR: Color = Color::srgba(0.0, 0.0, 0.0, 0.55);
const TEXT_VH: f32 = 2.2;
const BELOW_TOP_BAR_VH: f32 = 1.5;
/// Placeholder look of the defeat notice: pale red text on the same band, in a slot of its own under the
/// empty-station notice's, so that the two never overlap (percent of the surface height).
const DEFEAT_TEXT_COLOR: Color = Color::srgb(0.98, 0.64, 0.58);
const DEFEAT_BELOW_TOP_BAR_VH: f32 = BELOW_TOP_BAR_VH + 4.5;

/// The latest notice that still shows.
#[derive(Resource, Debug, Default)]
pub struct StationNotice {
    /// The sim's tick of the station's emptying; `None` while no notice shows.
    pub since: Option<Tick>,
    /// The `SimRes::generation` it belongs to: a replaced sim shows none.
    generation: Option<u32>,
}

/// The notice's node, shown while [`StationNotice::since`] is set, in the game modes.
#[derive(Component)]
pub struct StationNoticeView;

/// The defeats of the last `FeedbackTuning::notice_ticks` (module docs), every player's: what the defeat notice tells.
#[derive(Resource, Debug, Default)]
pub struct DefeatNotice {
    /// Each player defeated, with the sim's tick of their defeat, in the order they fell.
    pub defeats: Vec<(Tick, PlayerId)>,
    /// The `SimRes::generation` they belong to: a replaced sim shows none.
    generation: Option<u32>,
}

/// The defeat notice's node, shown while it tells the local player of a defeat, in the game modes.
#[derive(Component)]
pub struct DefeatNoticeView;

/// The defeat notice's text: a line for each defeat it tells.
#[derive(Component)]
pub struct DefeatNoticeText;

pub struct NoticesPlugin;

impl Plugin for NoticesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<StationNotice>()
            .init_resource::<DefeatNotice>()
            .add_systems(Startup, (spawn_notice, spawn_defeat_notice))
            .add_systems(Update, (show_notice, show_defeat_notice).in_set(ClientSystems::Present));
    }
}

/// A notice's node: `text` in `color` on a dark band, centred across the world view, `below` the top bar (percent of
/// the surface height), hidden.
fn notice_node(below: f32, text: impl Bundle) -> impl Bundle {
    (
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            width: percent(100),
            top: vh(TOP_BAR_HEIGHT_VH + below),
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
            children![text],
        )],
    )
}

fn spawn_notice(mut commands: Commands) {
    let text = (
        Text::new(STATION_EMPTY_TEXT),
        TextFont::from_font_size(FontSize::Vh(TEXT_VH)),
        TextColor(TEXT_COLOR),
        Pickable::IGNORE,
    );
    commands.spawn((
        Name::new("Station notice"),
        StationNoticeView,
        notice_node(BELOW_TOP_BAR_VH, text),
    ));
}

fn spawn_defeat_notice(mut commands: Commands) {
    let text = (
        DefeatNoticeText,
        Text::new(""),
        TextFont::from_font_size(FontSize::Vh(TEXT_VH)),
        TextColor(DEFEAT_TEXT_COLOR),
        Pickable::IGNORE,
    );
    commands.spawn((
        Name::new("Defeat notice"),
        DefeatNoticeView,
        notice_node(DEFEAT_BELOW_TOP_BAR_VH, text),
    ));
}

/// Takes the frame's empty stations that the local player perceives (theirs alone), and shows the notice for
/// `FeedbackTuning::notice_ticks` after the latest.
fn show_notice(
    mode: Res<ClientMode>,
    sim: Res<SimRes>,
    tuning: Res<ClientTuning>,
    sight: LocalSight,
    events: Res<ViewerEvents>,
    mut notice: ResMut<StationNotice>,
    mut views: Query<&mut Visibility, With<StationNoticeView>>,
) {
    let latest = (events.perceived_by(sight.sim(), sight.get()))
        .filter(|(_, event, _)| matches!(event, SimEvent::SupplyStationEmpty { .. }))
        .map(|(tick, ..)| tick)
        .max();
    let generation = Some(sim.generation());
    let earlier = notice.since.filter(|_| notice.generation == generation);
    let lasts = tuning.feedback.notice_ticks;
    let since = (latest.or(earlier)).filter(|&tick| sim.tick().saturating_sub(tick) < lasts);
    if notice.since != since || notice.generation != generation {
        notice.since = since;
        notice.generation = generation;
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

/// Keeps the defeats of the last `FeedbackTuning::notice_ticks` (the frame's that the local player perceives: every
/// player's), and tells the local player of the others' while the game goes on, unless they have left it (module docs).
#[expect(clippy::too_many_arguments, reason = "a system's parameters")]
fn show_defeat_notice(
    mode: Res<ClientMode>,
    sim: Res<SimRes>,
    tuning: Res<ClientTuning>,
    local: Res<LocalPlayer>,
    sight: LocalSight,
    events: Res<ViewerEvents>,
    mut notice: ResMut<DefeatNotice>,
    mut views: Query<&mut Visibility, With<DefeatNoticeView>>,
    mut texts: Query<&mut Text, With<DefeatNoticeText>>,
) {
    let generation = Some(sim.generation());
    let earlier = (notice.defeats.iter().copied()).filter(|_| notice.generation == generation);
    let new = (events.perceived_by(sight.sim(), sight.get())).filter_map(|(tick, event, _)| match *event {
        SimEvent::PlayerDefeated { player } => Some((tick, player)),
        _ => None,
    });
    let lasts = tuning.feedback.notice_ticks;
    let recent = |&(tick, _): &(Tick, PlayerId)| sim.tick().saturating_sub(tick) < lasts;
    let defeats: Vec<(Tick, PlayerId)> = earlier.chain(new).filter(recent).collect();
    if notice.defeats != defeats || notice.generation != generation {
        notice.defeats = defeats;
        notice.generation = generation;
    }
    let told = mode.is_game() && sim.outcome().is_none() && !results::has_left(&sim, local.0);
    let lines: Vec<String> = (notice.defeats.iter())
        .filter(|&&(_, player)| told && player != local.0)
        .filter_map(|&(_, player)| Some(defeat_text(&sim.player(player)?.name)))
        .collect();
    let visibility = if lines.is_empty() {
        Visibility::Hidden
    } else {
        Visibility::Inherited
    };
    for mut view in &mut views {
        view.set_if_neq(visibility);
    }
    for mut text in &mut texts {
        set_text(&mut text, &lines.join("\n"));
    }
}
