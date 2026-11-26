//! (M5-S6) The top bar's notes on how this client shows the game ([`TopBarNote`]), each in the bar's row while it
//! applies: "(AI)" after a computer player's name (slice A1), "Fog off (F3)", "Play-test (F5)".

use bevy::prelude::*;

use super::{show_node, text};
use crate::ClientMode;
use crate::ai_players::AiPlayers;
use crate::fog::FogOfWar;
use crate::sim_driver::LocalPlayer;

/// The top bar's notes ([`TopBarNote`]): the amber of the map editor's warnings (placeholder).
const NOTE_COLOR: Color = Color::srgb(0.98, 0.76, 0.32);

/// (M5-S6) A note in the top bar on how this client shows the game, shown only while it applies, in the bar's row:
/// it takes room of its own there, so it never covers the name, the resources or the tick, and it gives that room up
/// first, shrinking and clipped at its end when the bar is short of room, while they keep their size
/// ([`spawn_note`]). Placeholders, the design having no such note: the wording, the colour, the place.
#[derive(Component, Copy, Clone, Debug, PartialEq, Eq)]
pub enum TopBarNote {
    /// (A1) "(AI)", on the left right after the player's name ("Player 2 (AI)"), while a computer plays the local
    /// player (`--ai`, `crate::ai_players`): the client gives that player no commands, and the user, switching sides
    /// with F2, sees whose side it is.
    Computer,
    /// "Fog off (F3)", on the left after the player's name, while the fog of war is off ([`FogOfWar`]: `--no-fog`, or
    /// F3, the debug key next to F2's player switch), in games and replays alike: the user, switching sides with F2 in
    /// a hot-seat game, hit F3 and took the revealed map for a bug.
    FogOff,
    /// "Play-test (F5)", on the right before the tick, where a replay's note is (a play-test is never a replay), in the
    /// map editor's play-test (`ClientMode::PlayTest`), which F5 started and ends, as the editor's "Play-test (F5)"
    /// button says. It replaces the banner the editor laid over the top bar, which covered its figures (found by slice
    /// M5-Z3).
    PlayTest,
}

impl TopBarNote {
    /// What the note says.
    pub fn text(self) -> &'static str {
        match self {
            TopBarNote::Computer => "(AI)",
            TopBarNote::FogOff => "Fog off (F3)",
            TopBarNote::PlayTest => "Play-test (F5)",
        }
    }

    /// Whether the note shows, with the fog of war `fog`, in `mode`, while a computer plays the local player or not
    /// (`computer`).
    pub fn applies(self, fog: FogOfWar, mode: ClientMode, computer: bool) -> bool {
        match self {
            TopBarNote::Computer => computer,
            TopBarNote::FogOff => !fog.enabled,
            TopBarNote::PlayTest => mode == ClientMode::PlayTest,
        }
    }
}

/// The top bar's `note`, hidden until it applies ([`update_top_bar_notes`]): its text on one line, in a node that
/// shrinks first when the bar is short of room, clipping the text at its end (as the map editor's status line gives
/// way to its message).
pub(super) fn spawn_note(side: &mut ChildSpawnerCommands, note: TopBarNote) {
    side.spawn((
        Name::new("Top bar note"),
        note,
        Node {
            display: Display::None,
            flex_shrink: 1.0,
            min_width: px(0),
            overflow: Overflow::clip(),
            ..default()
        },
    ))
    .with_child((text(note.text(), 2.0, NOTE_COLOR), TextLayout::no_wrap()));
}

/// Shows each of the top bar's notes while it applies ([`TopBarNote::applies`]): when the fog of war is switched,
/// when the mode changes (a play-test starts or ends), and when the local player changes (F2), who may be a computer's.
pub(super) fn update_top_bar_notes(
    fog: Res<FogOfWar>,
    mode: Res<ClientMode>,
    player: Res<LocalPlayer>,
    computers: Option<Res<AiPlayers>>,
    mut notes: Query<(&TopBarNote, &mut Node)>,
) {
    if !fog.is_changed() && !mode.is_changed() && !player.is_changed() {
        return;
    }
    let computer = computers.is_some_and(|ais| ais.controls(player.0));
    for (note, mut node) in &mut notes {
        show_node(&mut node, note.applies(*fog, *mode, computer));
    }
}
