//! The empty-station notice (slice M4-I): a placeholder for design change 0256 point 3's notification
//! (`factions/syndicate/units/agent.md > Gathering > Supplies`: "An empty station: the owner gets a notification
//! (notifications not designed yet)"; 0256's Implementation impact: "M4 may show it in a placeholder form").
//!
//! When the sim reports that a Supply Delivery Station where the local player's Agents gather has run out
//! (`SimEvent::SupplyStationEmpty`, sent to each player whose Agents gather there, and through the filter of events to
//! that player alone, `crate::viewer_events`), [`STATION_EMPTY_TEXT`] shows
//! across the top of the world view, under the top bar, for [`NOTICE_TICKS`] after the latest one. It ages by the
//! sim's ticks, so a QA screenshot of a tick always shows it alike, and a sim replaced (a load, a replay) shows none of
//! the old one's. It never takes the pointer: clicks pass through it to the map.
//!
//! Placeholders, the design giving none of them: the wording, the look, where it shows and for how long.

use bevy::picking::Pickable;
use bevy::prelude::*;
use sc_sim::{SimEvent, Tick};

use super::TOP_BAR_HEIGHT_VH;
use crate::fog::LocalSight;
use crate::sim_driver::SimRes;
use crate::viewer_events::ViewerEvents;
use crate::{ClientMode, ClientSystems};

/// The notice's line. Placeholder wording.
pub const STATION_EMPTY_TEXT: &str = "Supply Delivery Station empty: your Agents wait there for the next delivery";

pub use crate::tuning::feedback::NOTICE_TICKS;

/// Placeholder look: pale green text on a dark band, this far under the top bar (percent of the surface height).
const TEXT_COLOR: Color = Color::srgb(0.70, 0.92, 0.70);
const BAND_COLOR: Color = Color::srgba(0.0, 0.0, 0.0, 0.55);
const TEXT_VH: f32 = 2.2;
const BELOW_TOP_BAR_VH: f32 = 1.5;

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

pub struct NoticesPlugin;

impl Plugin for NoticesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<StationNotice>()
            .add_systems(Startup, spawn_notice)
            .add_systems(Update, show_notice.in_set(ClientSystems::Present));
    }
}

fn spawn_notice(mut commands: Commands) {
    commands.spawn((
        Name::new("Station notice"),
        StationNoticeView,
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            width: percent(100),
            top: vh(TOP_BAR_HEIGHT_VH + BELOW_TOP_BAR_VH),
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
                Text::new(STATION_EMPTY_TEXT),
                TextFont::from_font_size(FontSize::Vh(TEXT_VH)),
                TextColor(TEXT_COLOR),
                Pickable::IGNORE,
            )],
        )],
    ));
}

/// Takes the frame's empty stations that the local player perceives (theirs alone), and shows the notice for
/// [`NOTICE_TICKS`] after the latest.
fn show_notice(
    mode: Res<ClientMode>,
    sim: Res<SimRes>,
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
    let since = (latest.or(earlier)).filter(|&tick| sim.tick().saturating_sub(tick) < NOTICE_TICKS);
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
