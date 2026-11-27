//! How the local player commands their units, producing structures and constructors (`systems/controls/interface.md >
//! ObjectInterfaceState`, `systems/controls/interface.md > CommandPanel`, `systems/controls/interface.md >
//! DefaultState`, `systems/controls/interface.md > RightClickResolution`, `systems/controls/interface.md > Production
//! Building Default Right-Click`, `systems/controls/interface.md > AwaitingTarget`, `systems/controls/hud.md >
//! PointerDisplayType`, `systems/controls/hud.md > CommandIndicators`; `systems/controls/commands.md > Unit Command`;
//! `factions/gdo/structures/barracks.md > ObjectInterfaceState[Barracks]`,
//! `factions/gdo/structures/deployment_center.md > ObjectInterfaceState[DeploymentCenter]`; `factions/gdo/economy.md >
//! ExtractionFacility > ObjectInterfaceState[ExtractionFacility]`; `factions/colonists/economy.md > Beacon > Beacon
//! ObjectInterfaceState`, `factions/colonists/units/drone.md > Drone ObjectInterfaceState`).
//!
//! - [`interface`]: the rules as plain data: the panel's commands ([`PanelCommand`], for units, producers and
//!   constructors alike: plan decision T10), the ObjectInterfaceState of each ControlState (with the Deployment
//!   Center's build menus and AwaitingPlacement), the command panel a selection gets and who receives a command, the
//!   Peacekeeper's Riot Shield toggle in R with what a raised shield allows, the Colonists' Beacon with its Warp In
//!   menus, the Drone's Deploy and the Cancel of whatever transforms, and the production and construction commands the
//!   command gate would refuse, which show disabled, with why ([`interface::Conditions`]);
//! - [`resolution`]: what a click on the map resolves to, object by object for a right-click (a unit's order, or a
//!   producer's rally point), and the PointerDisplayType that previews it;
//! - the systems (`systems.rs`): command hotkeys and panel clicks, left-clicks while a target is awaited (at the press
//!   point, within `InputTuning::targeting_click_allowance_px`; Shift-clicks queue and keep the target awaited until
//!   Shift is let go) or while a building is placed, the placement keys, right-clicks and Esc change the local player's
//!   ObjectInterfaceState and give orders (`Command::Order` in `PendingCommands`, with `queue` while the Queue
//!   modifier, Shift, is held), abilities (`Command::Ability`), production commands (`Command::Production`: Train,
//!   Cancel Production and the rally point) and construction commands (`Command::Construction`: Construct, the Cancels,
//!   Place and Warp In), and a Drone's Deploy (`OrderKind::Deploy`, one Drone a click: design change 0125); every frame
//!   the state is checked against the game and falls back to DefaultState when it no longer holds, and the input layer
//!   takes the placement keys while a building is placed (`crate::input::Placing`, the editor's keys);
//! - [`indicators`]: markers at the targets of the selected units' orders, a flash where an order was just given, the
//!   queued waypoints as lines while Shift is held, and the rally points of the selected producers;
//! - [`feedback`]: the message after a click on an invalid target, a placement that can't be made, a greyed-out button
//!   pressed ([`CommandRefused`]), and a refusal;
//! - [`cursor`]: the pointer's look, from the PointerDisplayType; none while a building is placed, whose ghost is the
//!   cursor (`crate::placement`).
//!
//! The panel's buttons belong to the HUD (`crate::hud::command_panel`); a click on one is a press of its slot, like its
//! hotkey.

pub mod cursor;
pub mod feedback;
pub mod indicators;
pub mod interface;
pub mod resolution;
mod systems;

use bevy::prelude::*;
use sc_sim::{ObjectId, OrderKind, PlacementProblem, Tick};

pub use interface::{
    BuildMenu, Builder, ButtonLook, Conditions, InterfaceState, InterfaceTemplate, Modes, Panel, PanelButton,
    PanelCommand, PanelSlot, Placing, Planned, WhyUnavailable,
};
pub use resolution::{CursorTarget, MapTarget, PointerDisplay, Resolved, UnitCapabilities};

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

/// `systems/controls/interface.md > AwaitingTarget > Invalid target clicks` (design change 0065): a left-click
/// on a target the pending command can't use, which gave nothing (no order, nor a rally point the sim would take); the
/// target is still awaited. For the feedback ([`feedback`]).
#[derive(Message, Copy, Clone, Debug, PartialEq)]
pub struct InvalidTarget {
    /// The PendingCommand.
    pub command: PanelCommand,
    /// The sim's tick when it was clicked.
    pub tick: Tick,
}

/// `systems/controls/hud.md > AwaitingPlacement`: a left-click where the building can't be placed now,
/// which sent nothing; placement goes on. For the feedback ([`feedback`]).
#[derive(Message, Copy, Clone, Debug, PartialEq)]
pub struct InvalidPlacement {
    /// Why not: the placement check's first problem (`sc_sim::PlacementCheck::first_problem`); `None` when the check
    /// allowed it but the sim wouldn't take the command.
    pub problem: Option<PlacementProblem>,
    /// The sim's tick when it was clicked.
    pub tick: Tick,
}

/// Design change 0162 (`systems/controls/interface.md > CommandPanel > Pressing an unavailable command`): a
/// greyed-out button pressed, or its hotkey, which issued nothing; or a Warp In whose click the sim wouldn't take
/// (the bank short since its entry was pressed). "The player gets the same feedback as when an order is refused,
/// naming the reason" ([`feedback`]).
#[derive(Message, Copy, Clone, Debug, PartialEq)]
pub struct CommandRefused {
    /// Why: the button's own reason (`PanelSlot::why`), or the gate's refusal.
    pub why: WhyUnavailable,
    /// The sim's tick when it was pressed.
    pub tick: Tick,
}

/// (Slice M5-X2) A data directory's files refused (`crate::data`): a file, a member of it and why ("tuning/combat.json:
/// minimum_damage: expected a number"), or the game's refusal of their rules. The game keeps the rules it had. For the
/// feedback ([`feedback`]), which shows it longer than a click's (`FeedbackTuning::refusal_ticks`).
#[derive(Message, Copy, Clone, Debug, PartialEq)]
pub struct DataRefused {
    /// What the feedback line says.
    pub text: &'static str,
    /// The sim's tick when the files were read.
    pub tick: Tick,
}

/// Order of the command flow's `Update` systems.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum CommandUiSystems {
    /// The local ObjectInterfaceState is checked against the game (in `ClientSystems::SyncSim`, after the
    /// selection has caught up with it).
    Validate,
    /// The frame's input becomes transitions and orders (in `ClientSystems::Actions`, after the selection's input:
    /// a left-click on the map selects only while no target is awaited and no building placed).
    Input,
}

pub struct CommandUiPlugin;

impl Plugin for CommandUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<crate::tuning::ClientTuning>();
        app.add_message::<OrderIssued>()
            .add_message::<InvalidTarget>()
            .add_message::<InvalidPlacement>()
            .add_message::<CommandRefused>()
            .add_message::<DataRefused>()
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
                    systems::end_queued_placement,
                    systems::cancel_on_escape,
                    systems::press_command_slots,
                    systems::turn_ghost,
                    systems::click_targets,
                    systems::place_on_click,
                    systems::right_click_orders,
                )
                    .chain()
                    .in_set(CommandUiSystems::Input),
            )
            // Not in `CommandUiSystems::Input`, which stops when the player may no longer command
            // (`crate::hud::outcome::may_command`) and during a playback: the placement keys end with the placement.
            .add_systems(
                Update,
                systems::report_placing
                    .in_set(ClientSystems::Actions)
                    .after(CommandUiSystems::Input),
            )
            .add_plugins((
                indicators::IndicatorsPlugin,
                feedback::FeedbackPlugin,
                cursor::CursorPlugin,
            ));
    }
}
