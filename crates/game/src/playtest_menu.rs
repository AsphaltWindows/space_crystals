//! The playtest menu (slice M5-S7; design change 0414, `core/game_rules.md > Playtest menu (temporary)`): "A simple
//! **temporary** menu for playtests, until a real in-game menu is designed", in games only. The map editor, its
//! play-tests and a replay's playback keep their own controls, and have no such menu (`crate::build_app`).
//!
//! - **It opens** "when the player presses **F10** during the game" ([`InputAction::PlaytestMenu`]), and "while the
//!   F10 menu is open the game is paused" (the designer's placeholder): the sim doesn't step (`SimControl::paused`), so
//!   nothing advances, the computer players don't think (`crate::ai_players`), QA's `--commands` wait for their tick,
//!   and no command reaches the sim, the command flow's input not running while the menu is open
//!   ([`menu_closed`]). It opens too "when the **game ends** (Victory, Defeat or Draw, shown with it)", over the
//!   stopped game, the outcome banner (`crate::hud::outcome`) telling the result above it ([`PlaytestMenu::GameEnd`]),
//!   and stays until Restart or Quit.
//! - **Its entries** ([`MenuButton`]): Resume, in F10's menu only, which closes it, as F10 again does; Restart; Quit.
//!   "No confirmation step (designer's placeholder): the menu itself is the deliberate step." A click on a button
//!   chooses it.
//! - **Restart** ([`restart`]): "the same map, player slots, factions and computer players (with their profiles) as
//!   the current game was started with" ([`Launch`]), from 0:00, in-process: a new sim, and every piece of the game's
//!   client state started over. The new game keeps its own replay (`crate::replay::record`).
//! - **Quit**: "closes the game", as closing the window does (`AppExit::Success`); the recording is written as on any
//!   exit.
//! - **While it is open it holds the input** (`crate::input::Modal`): no selection, command, control group, camera key,
//!   wheel or edge pan reaches the game; F10, the full-screen toggle (F11, Alt + Enter) and the debug keys, which only
//!   change what the client shows (F2, F3), still work. The press that closes it does nothing else.
//! - **Its look** ([`panel`]), a placeholder: a simple centred panel with buttons, in the HUD's style.

mod panel;
pub(crate) mod restart;

use std::path::PathBuf;

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use sc_sim::{PlayerId, Scenario};

use crate::ClientSystems;
use crate::ai_players::AiSlot;
use crate::command_ui::CommandUiSystems;
use crate::input::{ActionState, InputAction, Modal, ReadActions};
use crate::platform::SurfaceSize;
use crate::replay::RecordingNames;
use crate::sim_driver::{SimControl, SimRes};
pub use panel::{MenuLayout, MenuPart, MenuRoot};

/// Whether the playtest menu is open, and which (module docs).
#[derive(Resource, Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum PlaytestMenu {
    #[default]
    Closed,
    /// F10's, during the game: the game is paused.
    Paused,
    /// The game is over: the menu shows under its result.
    GameEnd,
}

impl PlaytestMenu {
    /// The menu's entries, top down.
    pub fn buttons(self) -> &'static [MenuButton] {
        match self {
            PlaytestMenu::Closed => &[],
            PlaytestMenu::Paused => &[MenuButton::Resume, MenuButton::Restart, MenuButton::Quit],
            PlaytestMenu::GameEnd => &[MenuButton::Restart, MenuButton::Quit],
        }
    }
}

/// One of the menu's entries (`core/game_rules.md > Playtest menu (temporary) > Options`), a button.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MenuButton {
    Resume,
    Restart,
    Quit,
}

impl MenuButton {
    pub const ALL: [MenuButton; 3] = [MenuButton::Resume, MenuButton::Restart, MenuButton::Quit];

    /// The button's label. Placeholder wording.
    pub fn label(self) -> &'static str {
        match self {
            MenuButton::Resume => "Resume (F10)",
            MenuButton::Restart => "Restart",
            MenuButton::Quit => "Quit",
        }
    }
}

/// The game as it was launched: what Restart starts over (module docs).
#[derive(Resource, Clone, Debug)]
pub struct Launch {
    /// The map and the game setup: the slots' players and factions, the seed and the game options.
    pub scenario: Scenario,
    /// The local player at launch (the debug player switch, F2, may have changed it since).
    pub player: PlayerId,
    /// The computer players, with their profiles (`--ai`).
    pub ai: Vec<AiSlot>,
    /// The launch's recording, if the game is recorded: its file, and how the restarts' are named.
    pub recording: Option<(PathBuf, RecordingNames)>,
    /// How many times the game was restarted.
    pub restarts: u32,
}

/// The playtest menu of a game launched as `launch` says. Add it after `SimDriverPlugin` and `HudPlugin`, in games
/// only.
pub struct PlaytestMenuPlugin {
    pub launch: Launch,
}

impl Plugin for PlaytestMenuPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.launch.clone())
            .init_resource::<PlaytestMenu>()
            .configure_sets(Update, CommandUiSystems::Input.run_if(menu_closed))
            .add_systems(Startup, panel::spawn_menu)
            // Right after the frame's input is read, so that the pause holds from this frame's step on, and the frame's
            // systems already see the input the menu holds.
            .add_systems(PreUpdate, use_menu.after(ReadActions))
            .add_systems(Update, panel::show_menu.in_set(ClientSystems::Present));
    }
}

/// Whether the playtest menu is closed: only then does the command flow's input run.
pub fn menu_closed(menu: Res<PlaytestMenu>) -> bool {
    *menu == PlaytestMenu::Closed
}

/// What the menu changes: itself, the pause and the input it holds.
#[derive(SystemParam)]
struct MenuState<'w> {
    menu: ResMut<'w, PlaytestMenu>,
    control: ResMut<'w, SimControl>,
    modal: ResMut<'w, Modal>,
}

/// Opens and closes the menu, and does what its buttons say (module docs). The menu holds the input from the next
/// frame on (`Modal`).
fn use_menu(
    actions: Res<ActionState>,
    size: Res<SurfaceSize>,
    sim: Res<SimRes>,
    state: MenuState,
    mut exit: MessageWriter<AppExit>,
    mut commands: Commands,
) {
    let MenuState {
        mut menu,
        mut control,
        mut modal,
    } = state;
    let opened = *menu;
    let clicked = (actions.modal_pressed(InputAction::Select))
        .then(|| actions.cursor())
        .flatten()
        .and_then(|cursor| MenuLayout::of(opened, size.logical())?.button_at(cursor));
    if sim.outcome().is_some() && opened != PlaytestMenu::GameEnd {
        *menu = PlaytestMenu::GameEnd;
    } else if actions.just_pressed(InputAction::PlaytestMenu) && opened != PlaytestMenu::GameEnd {
        // F10 opens the menu, and closes it again; the game end's stays.
        *menu = match opened {
            PlaytestMenu::Closed => PlaytestMenu::Paused,
            _ => PlaytestMenu::Closed,
        };
    } else if let Some(button) = clicked {
        match button {
            MenuButton::Resume => *menu = PlaytestMenu::Closed,
            MenuButton::Restart => {
                info!("Playtest menu: restarting the game");
                commands.queue(restart::restart);
            }
            MenuButton::Quit => {
                info!("Playtest menu: quitting");
                exit.write(AppExit::Success);
            }
        }
    }
    // The game pauses while F10's menu is open; the game end's stops it anyway (`SimControl::may_step`).
    if *menu != opened {
        control.paused = *menu == PlaytestMenu::Paused;
    }
    modal.set_if_neq(Modal(*menu != PlaytestMenu::Closed));
}
