//! How the local player orders their units (`systems/controls/selection_and_hud.md > ObjectInterfaceState`,
//! `> CommandPanel`, `> DefaultState`, `> RightClickResolution`, `> AwaitingTarget`, `> PointerDisplayType`,
//! `> CommandIndicators`; `systems/controls/commands.md > Unit Command`).
//!
//! - [`interface`]: the rules as plain data: the unit commands, the ObjectInterfaceState of each ControlState, the
//!   command panel a selection gets and who receives a command, and the Peacekeeper's Riot Shield toggle in R with
//!   what a raised shield allows ([`interface::Modes`]);
//! - [`resolution`]: what a click on the map resolves to, and the PointerDisplayType that previews it;
//! - the systems (`systems.rs`): command hotkeys and panel clicks, left-clicks while a target is awaited,
//!   right-clicks and Esc change the local player's ObjectInterfaceState and give orders (`Command::Order` in
//!   `PendingCommands`, with `queue` while the Queue modifier, Shift, is held) and abilities (`Command::Ability`);
//!   every frame the state is checked against the game and falls back to DefaultState when it no longer holds;
//! - [`indicators`]: markers at the targets of the selected units' orders, a flash where an order was just given,
//!   and the queued waypoints as lines while Shift is held;
//! - [`cursor`]: the pointer's look, from the PointerDisplayType.
//!
//! The panel's buttons belong to the HUD (`crate::hud::command_panel`); a click on one is a press of its slot, like
//! its hotkey.

pub mod cursor;
pub mod indicators;
pub mod interface;
pub mod resolution;
mod systems;

use bevy::prelude::*;
use sc_sim::{ObjectId, OrderKind, Tick};

pub use interface::{ButtonLook, InterfaceState, InterfaceTemplate, Modes, Panel, PanelButton, PanelSlot, UnitCommand};
pub use resolution::{CursorTarget, PointerDisplay};

use crate::ClientSystems;
use crate::selection::SelectionSystems;

/// An order the local player just gave (a CommandIssuingTransition), as it went to the sim: for the indicators.
#[derive(Message, Clone, Debug, PartialEq)]
pub struct OrderIssued {
    pub kind: OrderKind,
    pub units: Vec<ObjectId>,
    pub queue: bool,
    /// The sim's tick when it was given; it applies in the next one.
    pub tick: Tick,
}

/// Order of the command flow's `Update` systems.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum CommandUiSystems {
    /// The local ObjectInterfaceState is checked against the game (in `ClientSystems::SyncSim`, after the
    /// selection has caught up with it).
    Validate,
    /// The frame's input becomes transitions and orders (in `ClientSystems::Actions`, after the selection's input:
    /// a left-click on the map selects only while no target is awaited).
    Input,
}

pub struct CommandUiPlugin;

impl Plugin for CommandUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<OrderIssued>()
            .configure_sets(
                Update,
                CommandUiSystems::Validate
                    .in_set(ClientSystems::SyncSim)
                    .after(SelectionSystems::Sync),
            )
            .configure_sets(
                Update,
                CommandUiSystems::Input
                    .in_set(ClientSystems::Actions)
                    .after(SelectionSystems::Input),
            )
            .add_systems(Update, systems::validate_interface.in_set(CommandUiSystems::Validate))
            .add_systems(
                Update,
                (
                    systems::cancel_targeting,
                    systems::press_command_slots,
                    systems::click_targets,
                    systems::right_click_orders,
                )
                    .chain()
                    .in_set(CommandUiSystems::Input),
            )
            .add_plugins((indicators::IndicatorsPlugin, cursor::CursorPlugin));
    }
}
