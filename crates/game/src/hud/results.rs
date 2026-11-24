//! The results screen (slice M3-V; design change 0164, `core/game_rules.md > End of Game > Defeated before the end (no
//! team left)`): "a player defeated while the game continues and who has **no undefeated allies** (e.g. in a
//! free-for-all of 3 or more) **leaves to a results screen** and stops watching; the game goes on for the others."
//!
//! - **Who** ([`has_left`]): a player whose whole side, they and their allies (`Sim::allies`), was defeated while
//!   the game went on. They left in the tick the last of their side fell, and stay on the results screen from then on,
//!   after the end too. A side that falls in the last tick of the game never left: its players see the end over the
//!   world like everyone else (design change 0163: a draw if no side is left). A player defeated while an ally still
//!   stands watches with their team's vision instead (design change 0102: `crate::fog::Sight::new`, and the outcome
//!   banner's Spectating, [`super::outcome::Banner`]), and leaves with the last of them. The sim has no teams, so
//!   today every player defeated before the end leaves at once.
//! - **What it shows**, a placeholder ("Results screen contents: TBD"): DEFEATED, the player's name and faction, the
//!   game time and tick of their defeat, and whether the game goes on. It covers everything below the top bar, the
//!   world view and the HUD Bar, opaque, and takes the pointer, so nothing of the world shows and no click reaches
//!   it; the top bar stays (the player's name, the tick, a replay's controls). The end of the game shows over it, in
//!   the outcome banner: Defeat, or Draw, which "every player sees", the players defeated earlier too (design change
//!   0163).
//! - **A client state, derived**: the screen shows whenever [`has_left`] holds for the local player, in a game or a
//!   play-test. The sim runs on as before, and the debug player switch (F2) and a replay's playback show what each
//!   player would see.

use bevy::prelude::*;
use sc_sim::{Faction, PlayerId, Sim, TICKS_PER_SECOND, Tick};

use super::{TEXT_COLOR, TOP_BAR_HEIGHT_VH, set_text};
use crate::editor::objects::faction_name;
use crate::sim_driver::{LocalPlayer, SimRes};
use crate::{ClientMode, ClientSystems};

/// Placeholder look: an opaque background, the title in the outcome banner's Defeat red, and the sizes of the title
/// and the lines under it (percent of the surface height).
const BACKGROUND: Color = Color::srgb(0.05, 0.06, 0.08);
const TITLE_COLOR: Color = Color::srgb(0.95, 0.30, 0.25);
const DIM_COLOR: Color = Color::srgb(0.55, 0.60, 0.66);
const TITLE_VH: f32 = 8.0;
const LINE_VH: f32 = 2.4;
/// The space above the title, below the top bar (percent of the surface height): the texts stay above the outcome
/// banner's band, which shows over the screen at the end.
const TOP_PADDING_VH: f32 = 8.0;
/// Over the HUD (whose nodes have no z-index of their own), under the outcome banner (`outcome::BANNER_Z`).
pub const RESULTS_Z: i32 = 3;

/// Design change 0164: whether `player` has left the game for the results screen: they were defeated, and so was
/// every ally of theirs, the last of them before the game ended or while it goes on ([`left_the_game`]). Never for
/// a player not in the game.
pub fn has_left(sim: &Sim, player: PlayerId) -> bool {
    let Some(defeated_at) = sim.player(player).and_then(|p| p.defeated_at) else {
        return false;
    };
    let allies = sim.allies(player).into_iter();
    let allies = allies.map(|ally| sim.player(ally).and_then(|a| a.defeated_at));
    left_the_game(defeated_at, allies, sim.outcome().map(|end| end.tick))
}

/// The rule of [`has_left`] on its figures: a player defeated in tick `defeated_at`, with allies defeated in the
/// ticks `allies` (`None` for one still standing), in a game that ended in tick `end` (`None` while it goes on). They
/// left once no ally was left standing, in the tick their side's last player fell, if the game went on after it.
pub fn left_the_game(defeated_at: Tick, allies: impl IntoIterator<Item = Option<Tick>>, end: Option<Tick>) -> bool {
    let mut side_fell = defeated_at;
    for ally in allies {
        match ally {
            Some(at) => side_fell = side_fell.max(at),
            None => return false,
        }
    }
    end.is_none_or(|end| side_fell < end)
}

/// What the results screen shows a player who left ([`has_left`]). Placeholder contents (design change 0164:
/// "Results screen contents: TBD").
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Results {
    pub name: String,
    pub faction: Faction,
    /// The tick of their defeat (`Player::defeated_at`).
    pub defeated_at: Tick,
    /// Whether the game is over (`Sim::outcome`).
    pub game_over: bool,
}

impl Results {
    /// The results screen of `player` in `sim`, if they left the game.
    pub fn of(sim: &Sim, player: PlayerId) -> Option<Self> {
        if !has_left(sim, player) {
            return None;
        }
        let p = sim.player(player)?;
        Some(Results {
            name: p.name.clone(),
            faction: p.faction,
            defeated_at: p.defeated_at?,
            game_over: sim.outcome().is_some(),
        })
    }

    /// Its title.
    pub fn title(&self) -> &'static str {
        "DEFEATED"
    }

    /// Who the player was: their name and faction.
    pub fn player_line(&self) -> String {
        format!("{} ({})", self.name, faction_name(self.faction))
    }

    /// When they were defeated: the game time and the tick.
    pub fn defeat_line(&self) -> String {
        format!(
            "Defeated at {} (tick {})",
            game_time(self.defeated_at),
            self.defeated_at
        )
    }

    /// Whether the game goes on without them.
    pub fn status_line(&self) -> &'static str {
        if self.game_over {
            "The game is over"
        } else {
            "The game goes on for the other players"
        }
    }
}

/// The game time of `tick`, minutes and seconds, the seconds rounded down: tick 16 is "0:01", tick 960 "1:00".
pub fn game_time(tick: Tick) -> String {
    let seconds = tick / TICKS_PER_SECOND;
    format!("{}:{:02}", seconds / 60, seconds % 60)
}

/// The results screen, shown while the local player has [`Results`], in the game modes.
#[derive(Component)]
pub struct ResultsScreen;

/// The results screen's texts.
#[derive(Component, Copy, Clone, Debug, PartialEq, Eq)]
pub enum ResultsText {
    /// [`Results::title`].
    Title,
    /// [`Results::player_line`].
    Player,
    /// [`Results::defeat_line`].
    Defeat,
    /// [`Results::status_line`].
    Status,
}

pub struct ResultsPlugin;

impl Plugin for ResultsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_results_screen)
            .add_systems(Update, show_results_screen.in_set(ClientSystems::Present));
    }
}

fn spawn_results_screen(mut commands: Commands) {
    let text = |part: ResultsText, size_vh: f32, color: Color| {
        (
            part,
            Text::new(""),
            TextFont::from_font_size(FontSize::Vh(size_vh)),
            TextColor(color),
        )
    };
    commands.spawn((
        Name::new("Results screen"),
        ResultsScreen,
        Node {
            position_type: PositionType::Absolute,
            top: vh(TOP_BAR_HEIGHT_VH),
            bottom: px(0),
            left: px(0),
            width: percent(100),
            padding: UiRect::top(vh(TOP_PADDING_VH)),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: vh(1.2),
            ..default()
        },
        BackgroundColor(BACKGROUND),
        GlobalZIndex(RESULTS_Z),
        Visibility::Hidden,
        children![
            text(ResultsText::Title, TITLE_VH, TITLE_COLOR),
            text(ResultsText::Player, LINE_VH, TEXT_COLOR),
            text(ResultsText::Defeat, LINE_VH, TEXT_COLOR),
            text(ResultsText::Status, LINE_VH, DIM_COLOR),
        ],
    ));
}

/// Shows the local player's results screen while they have left the game, in a game or a play-test.
fn show_results_screen(
    mode: Res<ClientMode>,
    sim: Res<SimRes>,
    player: Res<LocalPlayer>,
    mut screens: Query<&mut Visibility, With<ResultsScreen>>,
    mut texts: Query<(&ResultsText, &mut Text)>,
) {
    if !mode.is_changed() && !sim.is_changed() && !player.is_changed() {
        return;
    }
    let results = Results::of(&sim, player.0).filter(|_| mode.is_game());
    let shown = if results.is_some() {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    for mut visibility in &mut screens {
        visibility.set_if_neq(shown);
    }
    let Some(results) = results else {
        return;
    };
    for (part, mut text) in &mut texts {
        match part {
            ResultsText::Title => set_text(&mut text, results.title()),
            ResultsText::Player => set_text(&mut text, &results.player_line()),
            ResultsText::Defeat => set_text(&mut text, &results.defeat_line()),
            ResultsText::Status => set_text(&mut text, results.status_line()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Design change 0164 without teams: a player defeated while the game goes on leaves at once; one defeated in
    /// the game's last tick (the draw of design change 0163 among them) never left.
    #[test]
    fn a_player_without_allies_leaves_when_defeated_before_the_end() {
        assert!(left_the_game(100, [], None), "the game goes on");
        assert!(left_the_game(100, [], Some(250)), "and stays left after the end");
        assert!(!left_the_game(250, [], Some(250)), "defeated in the last tick");
    }

    /// Design change 0102 is unchanged by 0164: with an ally still standing, a defeated player spectates with their
    /// team's vision (a 2 v 1 whose first defeat leaves the ally playing), and leaves only once the last of their
    /// side falls before the end; a side whose last player falls in the last tick never left. The sim has no teams,
    /// so this is the rule's only test with allies.
    #[test]
    fn a_player_with_an_ally_standing_spectates_and_leaves_with_the_last_of_them() {
        assert!(!left_the_game(100, [None], None), "2 v 1: the ally plays on");
        assert!(!left_the_game(100, [None], Some(300)), "and won");
        assert!(
            left_the_game(100, [Some(200)], None),
            "the ally fell too, the game goes on"
        );
        assert!(left_the_game(100, [Some(200)], Some(300)));
        assert!(
            !left_the_game(100, [Some(300)], Some(300)),
            "the ally fell in the last tick"
        );
        assert!(!left_the_game(100, [Some(50), None], None), "one ally of two stands");
    }

    #[test]
    fn game_time_counts_whole_seconds() {
        assert_eq!(game_time(0), "0:00");
        assert_eq!(game_time(15), "0:00");
        assert_eq!(game_time(16), "0:01");
        assert_eq!(game_time(960), "1:00");
        assert_eq!(game_time(16 * 754), "12:34");
    }
}
