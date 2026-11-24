//! The command flow's rules as plain data (`systems/controls/selection_and_hud.md`): the panel's commands, the
//! ObjectInterfaceState, the command panel a selection gets and who receives a command. Unit tested here; the
//! systems apply them to the local player's ControlState. What a click resolves to is [`super::resolution`].
//!
//! - [`commands`]: the panel's commands ([`PanelCommand`]), for units, producing structures and constructors alike
//!   (plan decision T10), and what each gives the sim: an order, an ability, a production or a construction command;
//! - [`templates`]: the ObjectInterfaceState templates ([`InterfaceTemplate`]): which commands a type offers, in
//!   which slots, and how a constructor's follow its state and its build menus ([`state_grid`]), the Beacon's its
//!   Warp In menus ([`menu_grid`]);
//! - [`conditions`]: what the game allows the selection's commands now ([`Conditions`]): the units' modes, the
//!   ActiveGroup's constructor state, whether it transforms, and the command gate's answer to each production and
//!   construction command, with why a button is greyed out ([`WhyUnavailable`]);
//! - [`panel`](mod@panel): the panel a selection gets, what pressing a slot does, and who receives a command;
//! - [`placing`]: AwaitingPlacement's building and who builds it ([`Placing`], [`Builder`]: a constructor, the
//!   Drones, a Beacon), where its ghost stands and how the placement keys turn it, and which Drone a Deploy goes
//!   to ([`deploy_recipient`]);
//! - here: the ObjectInterfaceState itself ([`InterfaceState`]), and whether it still holds ([`still_valid`]).
//!
//! The Peacekeeper's Riot Shield (`factions/gdo/units.md > Peacekeeper > Riot Shield (toggled ability)`) is a
//! command of its own ([`PanelCommand::RiotShield`]), in slot R of its panel. Pressed, it raises the shields it goes
//! to, or lowers them when all of them are up or on their way up ([`Modes`]), and the button says which. While a
//! unit's shield is up it can't attack: Attack is disabled when none of its recipients can, an explicit Attack
//! leaves it out (`super::resolution::unit_target_order`), and a right-click on an enemy sends it after the enemy
//! instead (`super::resolution::unit_right_click`).
//!
//! The Colonists' Legionnaires convert between their two forms with R (`factions/colonists/units.md > Weapon
//! Conversion`, [`PanelCommand::Convert`]), a common command of a selection of both (each converts to its other
//! form), issued at once and never queued. It is disabled while the gate would refuse it: while each of its
//! recipients is converting or waiting to (design change 0153: one pressed while Firing or in Cooldown converts once
//! its Cooldown is over). The converted unit stays selected as its new type
//! (`crate::selection::ControlState::set_kind`) and is drawn anew (`crate::object_render`).
//!
//! A Barracks (`factions/gdo/structures.md > Barracks > ObjectInterfaceState[Barracks]`) trains a Peacekeeper (Q)
//! and cancels the last unit queued (X), at once, and awaits the target of its rally point (C); a Train or a Cancel
//! Production that the command gate would refuse shows disabled ([`Conditions`]: the same predicates as the gate).
//! Every producing structure has this panel, built from its recipes (the Production template, plan decision T23):
//! the Colonists' Robo Portal trains a Drone in Q, the Bio Portal a Sabre Legionnaire
//! (`factions/colonists/structures.md > Robo Portal ObjectInterfaceState`, `> Bio Portal ObjectInterfaceState`,
//! design change 0124).
//!
//! A Deployment Center (`factions/gdo/structures.md > DeploymentCenter > ObjectInterfaceState[DeploymentCenter]`,
//! design change 0104) opens BuildMenu\[Basic\] (Q) and BuildMenu\[Advanced\] (W, unavailable while none of its
//! buildings is in the game); a menu's entry constructs its building and returns to DefaultState; once the building
//! is ready, its entry enters AwaitingPlacement; X cancels the construction, or the building ready to place. An
//! Extraction Facility (`factions/gdo/economy.md > ExtractionFacility > ObjectInterfaceState[ExtractionFacility]`)
//! does the same with its plate from DefaultState: Q builds it, then places it. Their commands go to the one
//! instance whose state the panel shows ([`recipients`]).
//!
//! The Colonists build by transforming (slice M3-J1, design change 0124). A Beacon (`factions/colonists/economy.md >
//! Beacon > Beacon ObjectInterfaceState`) opens Warp In (Basic) (Q) and Warp In (Advanced) (W) like the Deployment
//! Center's menus; a menu's building enters AwaitingPlacement on the Beacon, the ghost centred on it and turned by
//! the placement keys about its centre (design change 0121), and a click warps it in; the buildings not in the game
//! yet show unavailable. A Drone's A (`factions/colonists/units.md > Drone ObjectInterfaceState`) enters
//! AwaitingPlacement of a Beacon under the cursor; each click sends one Drone of the group to deploy there (design
//! change 0125), with Shift the next one that has no Deploy yet. Whatever transforms, a Beacon under deployment or
//! a building under Warp In, shows X Cancel and nothing else (design change 0116). Warp In applies to the Beacon
//! Tabbed to (Beacons are Ungroupable, design change 0122).
//!
//! A greyed-out button, or its hotkey, can still be pressed: nothing is issued, and the player is told why
//! ([`PanelSlot::why`], design change 0162).
//!
//! No smart-casting (`> SelectionGroup > No smart-casting`, design changes 0071 and 0075): a panel command goes to
//! every object it is issued to ([`recipients`]), a Train to every selected Barracks, whose payments the sim orders
//! (design change 0083).
//!
//! While a target is awaited or a building is placed, the other commands are unavailable (`> AwaitingTarget > Other
//! commands while targeting`, design change 0066): the panel shows them disabled, and their hotkeys and buttons do
//! nothing.

pub mod commands;
pub mod conditions;
pub mod panel;
pub mod placing;
pub mod templates;

pub use commands::PanelCommand;
pub use conditions::{Conditions, Modes, WhyUnavailable};
pub use panel::{
    ButtonLook, Panel, PanelButton, PanelSlot, SlotPress, commandable, is_common, panel, press, recipients,
};
pub use placing::{Builder, Placing, deploy_recipient, deploying};
pub use templates::{
    BuildMenu, InterfaceTemplate, Planned, TRANSFORMING_GRID, can_target_ground, commands, grid, menu_grid, menu_of,
    offers, slot_index, state_grid,
};

use sc_sim::deploy;

use crate::input::CommandSlot;
use crate::selection::Selection;

/// `> ObjectInterfaceState[ObjectEnum]`: where the ActiveGroup's command flow stands. It belongs to each player's
/// ControlState (`crate::selection::ControlState::interface`), and goes back to DefaultState when the Selection or
/// the ActiveGroup changes (`crate::selection::LocalControl::update`), or when it no longer holds for the game
/// ([`still_valid`]). The production UI needs no state of its own: Set Rally Point awaits its target like a unit
/// command.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum InterfaceState {
    /// `> DefaultState`.
    #[default]
    DefaultState,
    /// `> AwaitingTarget[CommandType]`: the PendingCommand waits for a left-click on its target: a unit command's,
    /// or a producer's rally point (`factions/gdo/structures.md > Barracks`: "C: Set Rally Point: enters
    /// AwaitingTarget\[SetRallyPoint\]").
    AwaitingTarget {
        /// The PendingCommand.
        pending: PanelCommand,
        /// `> AwaitingTarget > Shift while targeting` (design change 0070): a Shift-click has queued the command
        /// since the target began to be awaited. Targeting then ends when Shift is released; before the first
        /// queued click a release of Shift changes nothing (the design's TBD, its proposed default: "targeting
        /// simply continues").
        queued: bool,
    },
    /// `factions/gdo/structures.md > DeploymentCenter > ObjectInterfaceState[DeploymentCenter]`: one of the
    /// Deployment Center's build menus is open (design change 0104), in whichever state the Deployment Center is:
    /// idle, constructing or holding a building ready; or one of a Beacon's Warp In menus
    /// (`factions/colonists/economy.md > Beacon > Beacon ObjectInterfaceState`).
    BuildMenu(BuildMenu),
    /// `> AwaitingPlacement`: the ghost of a building ready to place follows the cursor, and a left-click on a valid
    /// spot places it (`crate::placement`); the same for a Drone's Beacon, and a Beacon's Warp In, whose ghost stands
    /// on the Beacon ([`Builder`]).
    AwaitingPlacement(Placing),
}

impl InterfaceState {
    /// AwaitingTarget of `pending`, as a command's hotkey or button enters it: no click queued yet.
    pub const fn awaiting(pending: PanelCommand) -> Self {
        InterfaceState::AwaitingTarget { pending, queued: false }
    }

    /// The PendingCommand, while a target is awaited.
    pub fn pending(self) -> Option<PanelCommand> {
        match self {
            InterfaceState::AwaitingTarget { pending, .. } => Some(pending),
            InterfaceState::DefaultState | InterfaceState::BuildMenu(_) | InterfaceState::AwaitingPlacement(_) => None,
        }
    }

    /// The building being placed, in AwaitingPlacement.
    pub fn placing(self) -> Option<Placing> {
        match self {
            InterfaceState::AwaitingPlacement(placing) => Some(placing),
            _ => None,
        }
    }

    /// Whether the state takes the left-clicks on the map as its own, a target or a placement, so that they select
    /// nothing, and a drag on the map does nothing (`> AwaitingTarget > Dragging while targeting`, design change
    /// 0067; the same for a placement). A build menu leaves the map's clicks to the selection.
    pub fn takes_map_clicks(self) -> bool {
        matches!(
            self,
            InterfaceState::AwaitingTarget { .. } | InterfaceState::AwaitingPlacement(_)
        )
    }

    /// The state before this one, which Back (Z), Esc and a right-click return to (`> Standard Slot Assignments >
    /// Z`: "In any multi-stage menu (BuildMenu, ExpandMenu, EjectMenu, AwaitingTarget, AwaitingPlacement), Z returns
    /// to the previous state (StateOnlyTransition). Equivalent to Escape or right-click cancel."): from
    /// AwaitingPlacement of a Deployment Center's building, its build menu (`> AwaitingPlacement`: "Escape/right-click:
    /// returns to BuildMenu"), and from a Beacon's Warp In the Warp In menu its building was chosen in (the menus are
    /// "like the Deployment Center's two build menus": `factions/colonists/economy.md > Beacon > Beacon
    /// ObjectInterfaceState`); from anything else DefaultState: the Extraction Facility's AwaitingPlacement
    /// ("Escape/right-click: returns to DefaultState"), the Drones' (A is pressed in their DefaultState), a build menu
    /// ("Escape/right-click: returns to DefaultState"), AwaitingTarget.
    pub fn back(self, selection: &Selection) -> InterfaceState {
        match self {
            InterfaceState::AwaitingPlacement(placing) => selection
                .active()
                .and_then(|active| menu_of(active.kind, placing.kind))
                .map_or(InterfaceState::DefaultState, InterfaceState::BuildMenu),
            _ => InterfaceState::DefaultState,
        }
    }
}

/// `> ObjectInterfaceState`: whether `interface` still holds for `selection` (validated every frame), while the
/// selection is commandable (a defeated player's and everyone's after the end of the game is not):
/// - AwaitingTarget while its ActiveGroup still offers the PendingCommand, which the game still allows
///   (`conditions`: an Attack while some recipient can attack);
/// - a build menu while the ActiveGroup is a Deployment Center, or a Beacon that isn't being deployed: the menu stays
///   open while its state changes (a construction that completes shows its building ready in the menu);
/// - AwaitingPlacement while the ActiveGroup is the constructor, which still holds the building ready
///   (`conditions`: its state); while it is Drones; while it is the Beacon, which isn't being deployed (the bank is
///   the click's to check: `crate::command_ui::feedback` says why it is refused).
///
/// A selected object that transforms has the panel of its new type, but only its Cancel (design change 0116): what
/// its old type was doing ends (`crate::selection::ControlState::set_kind`), and so does a target awaited for it
/// (the panel offers no command but the Cancel).
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
        InterfaceState::BuildMenu(menu) => {
            commandable
                && !conditions.transforming
                && selection
                    .active()
                    .is_some_and(|active| offers(active.kind, PanelCommand::BuildMenu(menu)))
        }
        InterfaceState::AwaitingPlacement(placing) => {
            commandable
                && match placing.builder {
                    Builder::Constructor(constructor) => {
                        selection.in_active_group(constructor)
                            && conditions.constructor.and_then(|c| c.ready()) == Some(placing.kind)
                    }
                    Builder::Drones { .. } => selection.active().is_some_and(|active| deploy::deploys(active.kind)),
                    Builder::Beacon(beacon) => {
                        selection.in_active_group(beacon)
                            && !conditions.transforming
                            && selection
                                .active()
                                .is_some_and(|active| offers(active.kind, PanelCommand::WarpIn(placing.kind)))
                    }
                }
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
