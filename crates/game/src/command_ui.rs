//! How the local player commands their units and producing structures (`systems/controls/selection_and_hud.md >
//! ObjectInterfaceState`, `> CommandPanel`, `> DefaultState`, `> RightClickResolution`, `> Production Building
//! Default Right-Click`, `> AwaitingTarget`, `> PointerDisplayType`, `> CommandIndicators`;
//! `systems/controls/commands.md > Unit Command`; `factions/gdo/structures.md > Barracks >
//! ObjectInterfaceState[Barracks]`).
//!
//! - [`interface`]: the rules as plain data: the panel's commands ([`PanelCommand`], for units and producers alike:
//!   plan decision T10), the ObjectInterfaceState of each ControlState, the command panel a selection gets and who
//!   receives a command, the Peacekeeper's Riot Shield toggle in R with what a raised shield allows, and the
//!   production commands the command gate would refuse, which show disabled ([`interface::Conditions`]);
//! - [`resolution`]: what a click on the map resolves to, object by object for a right-click (a unit's order, or a
//!   producer's rally point), and the PointerDisplayType that previews it;
//! - the systems (`systems.rs`): command hotkeys and panel clicks, left-clicks while a target is awaited (at the
//!   press point, within [`TARGETING_CLICK_ALLOWANCE_PX`]; Shift-clicks queue and keep the target awaited until
//!   Shift is let go), right-clicks and Esc change the local player's ObjectInterfaceState and give orders
//!   (`Command::Order` in `PendingCommands`, with `queue` while the Queue modifier, Shift, is held), abilities
//!   (`Command::Ability`) and production commands (`Command::Production`: Train, Cancel Production and the rally
//!   point); every frame the state is checked against the game and falls back to DefaultState when it no longer
//!   holds;
//! - [`indicators`]: markers at the targets of the selected units' orders, a flash where an order was just given,
//!   the queued waypoints as lines while Shift is held, and the rally points of the selected producers;
//! - [`feedback`]: the message after a click on an invalid target;
//! - [`cursor`]: the pointer's look, from the PointerDisplayType.
//!
//! The panel's buttons belong to the HUD (`crate::hud::command_panel`); a click on one is a press of its slot, like
//! its hotkey.

pub mod cursor;
pub mod feedback;
pub mod indicators;
pub mod interface;
pub mod resolution;
mod systems;

use bevy::prelude::*;
use sc_sim::{ObjectId, OrderKind, Tick};

pub use interface::{
    ButtonLook, Conditions, InterfaceState, InterfaceTemplate, Modes, Panel, PanelButton, PanelCommand, PanelSlot,
};
pub use resolution::{CursorTarget, MapTarget, PointerDisplay, Resolved, UnitCapabilities};
pub use systems::TARGETING_CLICK_ALLOWANCE_PX;

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

/// `> AwaitingTarget > Invalid target clicks` (design change 0065): a left-click on a target the pending command
/// can't use, which gave nothing (no order, nor a rally point the sim would take); the target is still awaited. For
/// the feedback ([`feedback`]).
#[derive(Message, Copy, Clone, Debug, PartialEq)]
pub struct InvalidTarget {
    /// The PendingCommand.
    pub command: PanelCommand,
    /// The sim's tick when it was clicked.
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
            .add_message::<InvalidTarget>()
            .init_resource::<systems::TargetPress>()
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
                    systems::end_queued_targeting,
                    systems::cancel_targeting,
                    systems::press_command_slots,
                    systems::click_targets,
                    systems::right_click_orders,
                )
                    .chain()
                    .in_set(CommandUiSystems::Input),
            )
            .add_plugins((
                indicators::IndicatorsPlugin,
                feedback::FeedbackPlugin,
                cursor::CursorPlugin,
            ));
    }
}
