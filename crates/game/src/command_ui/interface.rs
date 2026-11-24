//! The command flow's rules as plain data (`systems/controls/selection_and_hud.md`): the panel's commands, the
//! ObjectInterfaceState, the command panel a selection gets and who receives a command. Unit tested here; the
//! systems apply them to the local player's ControlState. What a click resolves to is [`super::resolution`].
//!
//! - [`commands`]: the panel's commands ([`PanelCommand`]), for units and producing structures alike (plan decision
//!   T10), and what each gives the sim: an order, an ability or a production command;
//! - [`templates`]: the ObjectInterfaceState templates ([`InterfaceTemplate`]): which commands a type offers, in
//!   which slots;
//! - [`conditions`]: what the game allows the selection's commands now ([`Conditions`]): the units' modes, and the
//!   command gate's answer to each production command;
//! - [`panel`]: the panel a selection gets, what pressing a slot does, and who receives a command;
//! - here: the ObjectInterfaceState itself ([`InterfaceState`]), and whether it still holds ([`still_valid`]).
//!
//! The Peacekeeper's Riot Shield (`factions/gdo/units.md > Peacekeeper > Riot Shield (toggled ability)`) is a
//! command of its own ([`PanelCommand::RiotShield`]), in slot R of its panel. Pressed, it raises the shields it goes
//! to, or lowers them when all of them are up or on their way up ([`Modes`]), and the button says which. While a
//! unit's shield is up it can't attack: Attack is disabled when none of its recipients can, an explicit Attack
//! leaves it out (`super::resolution::unit_target_order`), and a right-click on an enemy sends it after the enemy
//! instead (`super::resolution::order_without_attack`).
//!
//! A Barracks (`factions/gdo/structures.md > Barracks > ObjectInterfaceState[Barracks]`) trains a Peacekeeper (Q)
//! and cancels the last unit queued (X), at once, and awaits the target of its rally point (C); a Train or a Cancel
//! Production that the command gate would refuse shows disabled ([`Conditions`]: the same predicates as the gate).
//!
//! No smart-casting (`> SelectionGroup > No smart-casting`, design changes 0071 and 0075): a panel command goes to
//! every object it is issued to ([`recipients`]), a Train to every selected Barracks, whose payments the sim orders
//! (design change 0083).
//!
//! While a target is awaited, the other commands are unavailable (`> AwaitingTarget > Other commands while
//! targeting`, design change 0066): the panel shows them disabled, and their hotkeys and buttons do nothing.

pub mod commands;
pub mod conditions;
pub mod panel;
pub mod templates;

pub use commands::PanelCommand;
pub use conditions::{Conditions, Modes};
pub use panel::{ButtonLook, Panel, PanelButton, PanelSlot, SlotPress, commandable, panel, press, recipients};
pub use templates::{InterfaceTemplate, can_target_ground, grid, offers, slot_index};

use crate::input::CommandSlot;
use crate::selection::Selection;

/// `> ObjectInterfaceState[ObjectEnum]`: where the ActiveGroup's command flow stands. It belongs to each player's
/// ControlState (`crate::selection::ControlState::interface`), and goes back to DefaultState when the Selection or
/// the ActiveGroup changes (`crate::selection::LocalControl::update`), or when it no longer holds for the game
/// ([`still_valid`]).
///
/// The construction UI (slice M2-H2) adds its states beside these: the Deployment Center's build menus
/// (`factions/gdo/structures.md > DeploymentCenter`, design change 0104) and the placement of a building ready to
/// place. The production UI needs none of its own: Set Rally Point awaits its target like a unit command.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum InterfaceState {
    /// `> DefaultState`.
    #[default]
    DefaultState,
    /// `> AwaitingTarget[CommandType]`: the PendingCommand waits for a left-click on its target: a unit command's,
    /// or a producer's rally point (`factions/gdo/structures.md > Barracks`: "C: Set Rally Point: enters
    /// AwaitingTarget[SetRallyPoint]").
    AwaitingTarget {
        /// The PendingCommand.
        pending: PanelCommand,
        /// `> AwaitingTarget > Shift while targeting` (design change 0070): a Shift-click has queued the command
        /// since the target began to be awaited. Targeting then ends when Shift is released; before the first
        /// queued click a release of Shift changes nothing (the design's TBD, its proposed default: "targeting
        /// simply continues").
        queued: bool,
    },
}

impl InterfaceState {
    /// AwaitingTarget of `pending`, as a command's hotkey or button enters it: no click queued yet.
    pub const fn awaiting(pending: PanelCommand) -> Self {
        InterfaceState::AwaitingTarget { pending, queued: false }
    }

    /// The PendingCommand, while a target is awaited.
    pub fn pending(self) -> Option<PanelCommand> {
        match self {
            InterfaceState::DefaultState => None,
            InterfaceState::AwaitingTarget { pending, .. } => Some(pending),
        }
    }
}

/// `> ObjectInterfaceState`: whether `interface` still holds for `selection` (validated every frame): AwaitingTarget
/// while the selection is commandable and its ActiveGroup still offers the PendingCommand, which the game still
/// allows (`conditions`: an Attack while some recipient can attack).
pub fn still_valid(
    interface: InterfaceState,
    selection: &Selection,
    commandable: bool,
    conditions: &Conditions,
) -> bool {
    match interface {
        InterfaceState::DefaultState => true,
        InterfaceState::AwaitingTarget { pending, .. } => {
            commandable
                && press(
                    &panel(selection, InterfaceState::DefaultState, conditions),
                    command_slot(selection, pending),
                ) == SlotPress::AwaitTarget(pending)
        }
    }
}

/// The slot of `command` in the ActiveGroup's grid (Back's when it has none, which no command is pressed from).
fn command_slot(selection: &Selection, command: PanelCommand) -> CommandSlot {
    selection
        .active()
        .and_then(|active| {
            CommandSlot::all()
                .zip(grid(active.kind))
                .find(|(_, c)| *c == Some(command))
                .map(|(slot, _)| slot)
        })
        .unwrap_or(CommandSlot::BACK)
}

#[cfg(test)]
mod tests;
