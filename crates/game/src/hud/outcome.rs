//! The end of the game in the client (slice M2-F2; `core/game_rules.md > End of Game`, design changes 0101 and
//! 0102), apart from the stop of the sim (`crate::sim_driver::SimControl::may_step`), the saved replay
//! (`crate::replay::record`) and the spectator's fog (`crate::fog::Sight::new`):
//! - **The banner** ([`Banner`]): once the game is over, "every player sees Victory or Defeat": Victory for the
//!   players of the side left standing (`Outcome::winners`), Defeat for everyone else. A player defeated while the
//!   game goes on sees that they are defeated and spectating. The banner is the local player's, so the debug player
//!   switch (F2) shows each player's.
//! - **No command input**: a defeated player "can't command anything", and after the end nobody can. The command
//!   UI's input doesn't run ([`may_command`]); the sim's gate refuses their commands anyway.
//!
//! Placeholders, the design being silent on how the banner looks: its texts, a dark band across the middle of the
//! world view, and the title's colours and sizes. It never takes the pointer.

use bevy::picking::Pickable;
use bevy::prelude::*;
use sc_sim::{Player, PlayerId, Sim};

use super::{TEXT_COLOR, set_text};
use crate::command_ui::CommandUiSystems;
use crate::sim_driver::{LocalPlayer, SimRes};
use crate::{ClientMode, ClientSystems};

/// Placeholder look: the band, the title's colours (gold for Victory, red for Defeat, grey while spectating) and the
/// sizes of the title and the line under it (percent of the surface height).
const BAND_COLOR: Color = Color::srgba(0.0, 0.0, 0.0, 0.6);
const VICTORY_COLOR: Color = Color::srgb(1.0, 0.82, 0.30);
const DEFEAT_COLOR: Color = Color::srgb(0.95, 0.30, 0.25);
const SPECTATING_COLOR: Color = Color::srgb(0.70, 0.73, 0.78);
const TITLE_VH: f32 = 8.0;
const LINE_VH: f32 = 2.4;
/// Where the band's top is, from the top of the surface (percent of its height): the middle of the world view.
const TOP_VH: f32 = 36.0;

/// What the banner tells one player.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Banner {
    /// The game is over, and the player's side won.
    Victory,
    /// The game is over, and the player's side didn't win.
    Defeat,
    /// The player is defeated and the game goes on (design change 0102): they watch, and command nothing.
    Spectating,
}

impl Banner {
    /// The banner `player` sees in `sim`: none while they play, or if they aren't in the game.
    pub fn of(sim: &Sim, player: PlayerId) -> Option<Self> {
        let p = sim.player(player)?;
        match sim.outcome() {
            Some(outcome) if outcome.is_winner(player) => Some(Banner::Victory),
            Some(_) => Some(Banner::Defeat),
            None if p.is_defeated() => Some(Banner::Spectating),
            None => None,
        }
    }

    /// The banner's title. Placeholder wording.
    pub fn title(self) -> &'static str {
        match self {
            Banner::Victory => "VICTORY",
            Banner::Defeat => "DEFEAT",
            Banner::Spectating => "DEFEATED",
        }
    }

    /// The line under the title. Placeholder wording.
    pub fn line(self) -> &'static str {
        match self {
            Banner::Victory | Banner::Defeat => "The game is over",
            Banner::Spectating => "Spectating: you can no longer give commands",
        }
    }

    fn color(self) -> Color {
        match self {
            Banner::Victory => VICTORY_COLOR,
            Banner::Defeat => DEFEAT_COLOR,
            Banner::Spectating => SPECTATING_COLOR,
        }
    }
}

/// The banner's band, shown while the local player has a [`Banner`], in the game modes.
#[derive(Component)]
pub struct OutcomeBanner;

/// The banner's texts.
#[derive(Component, Copy, Clone, Debug, PartialEq, Eq)]
pub enum BannerText {
    /// The title, in the banner's colour.
    Title,
    /// The line under the title.
    Line,
}

pub struct OutcomePlugin;

impl Plugin for OutcomePlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(Update, CommandUiSystems::Input.run_if(may_command))
            .add_systems(Startup, spawn_banner)
            .add_systems(Update, show_banner.in_set(ClientSystems::Present));
    }
}

/// Whether the local player may give commands: the game isn't over and they aren't defeated
/// (`core/game_rules.md > On Defeat`, design changes 0101 and 0102).
pub fn may_command(sim: Res<SimRes>, player: Res<LocalPlayer>) -> bool {
    sim.outcome().is_none() && !sim.player(player.0).is_some_and(Player::is_defeated)
}

fn spawn_banner(mut commands: Commands) {
    commands.spawn((
        Name::new("Outcome banner"),
        OutcomeBanner,
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            width: percent(100),
            top: vh(TOP_VH),
            padding: UiRect::vertical(vh(1.6)),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: vh(0.8),
            ..default()
        },
        BackgroundColor(BAND_COLOR),
        Pickable::IGNORE,
        Visibility::Hidden,
        children![
            (
                BannerText::Title,
                Text::new(""),
                TextFont::from_font_size(FontSize::Vh(TITLE_VH)),
                TextColor(VICTORY_COLOR),
                Pickable::IGNORE,
            ),
            (
                BannerText::Line,
                Text::new(""),
                TextFont::from_font_size(FontSize::Vh(LINE_VH)),
                TextColor(TEXT_COLOR),
                Pickable::IGNORE,
            ),
        ],
    ));
}

/// Shows the local player's banner, if they have one, in a game or a play-test.
fn show_banner(
    mode: Res<ClientMode>,
    sim: Res<SimRes>,
    player: Res<LocalPlayer>,
    mut bands: Query<&mut Visibility, With<OutcomeBanner>>,
    mut texts: Query<(&BannerText, &mut Text, &mut TextColor)>,
) {
    let banner = Banner::of(&sim, player.0).filter(|_| mode.is_game());
    let shown = if banner.is_some() {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    for mut visibility in &mut bands {
        visibility.set_if_neq(shown);
    }
    let Some(banner) = banner else {
        return;
    };
    for (part, mut text, mut color) in &mut texts {
        match part {
            BannerText::Title => {
                set_text(&mut text, banner.title());
                color.set_if_neq(TextColor(banner.color()));
            }
            BannerText::Line => set_text(&mut text, banner.line()),
        }
    }
}
