//! The command flow's rules as plain data (`systems/controls/interface.md`): the panel's commands, the
//! ObjectInterfaceState, the command panel a selection gets and who receives a command. Unit tested here; the
//! systems apply them to the local player's ControlState. What a click resolves to is [`super::resolution`].
//!
//! - [`commands`]: the panel's commands ([`PanelCommand`]), for units, producing structures and constructors alike
//!   (plan decision T10), and what each gives the sim: an order, an ability, a production or a construction command;
//! - [`templates`]: the ObjectInterfaceState templates ([`InterfaceTemplate`]): which commands a type offers, in
//!   which slots, and how a constructor's follow its state and its build menus ([`state_grid`]), the Beacon's its
//!   Warp In menus and the Recruit's its Construct menus ([`menu_grid`]);
//! - [`conditions`]: what the game allows the selection's commands now ([`Conditions`]): the units' modes, the
//!   ActiveGroup's constructor state, and the command gate's answer to each production and construction command,
//!   with why a button is greyed out ([`WhyUnavailable`]);
//! - [`panel`](mod@panel): the panel a selection gets, what pressing a slot does, and who receives a command;
//! - [`placing`]: AwaitingPlacement's building and who builds it ([`Placing`], [`Builder`]: a constructor, the Drones,
//!   a Beacon, an Agent, a Tunnel, the Recruits), where its ghost stands and how the placement keys turn it, and which
//!   Drone a Deploy goes to ([`deploy_recipient`]);
//! - here: the ObjectInterfaceState itself ([`InterfaceState`]), and whether it still holds ([`still_valid`]).
//!
//! The Peacekeeper's Riot Shield (`factions/gdo/units/peacekeeper.md > Riot Shield (toggled ability)`): shield
//! down and shield up are two unit types (design change 0173), Peacekeeper and Peacekeeper (Shielded), which form
//! separate SelectionGroups. The Peacekeeper's panel has Raise Shield in slot R ([`PanelCommand::RaiseShield`]), the
//! Peacekeeper (Shielded)'s Lower Shield there ([`PanelCommand::LowerShield`]) and no Attack (design change 0174: R,
//! a placeholder). Each is its type's own, so in a selection of both it goes to the ActiveGroup. A mode toggle sets
//! a target state (design change 0198): the units pressed switch their type at the press, before the sim applies
//! the command (`crate::selection::ControlState::toggle`, and `crate::sim_driver::PendingCommands::kind_of` until
//! it does), including those whose raise waits for their shot, so they move to the other type's group at once,
//! which becomes the ActiveGroup (design change 0199), and a second press reaches them all. Which orders a unit
//! takes follows its type (design change 0202, [`Modes`]): a Peacekeeper (Shielded) takes no Attack (its panel alone
//! has none, [`templates::attack_slot`]; an explicit Attack sends it to the target instead, as every unit that can't
//! attack, `super::resolution::unit_target_order`, design change 0398; and a right-click on an enemy sends it after
//! the enemy, `super::resolution::unit_right_click`), while a Peacekeeper whose shield is on its way down takes one,
//! and the sim holds it until the shield is down. In a selection with other units, an AttackMove takes it along
//! (design change 0076).
//!
//! **Attack goes to everyone** (`systems/controls/interface.md > CommandPanel`, design changes 0397 to 0399): A is
//! Attack on every unit's panel, shown whenever one of the selected units can attack, whichever group is in focus
//! ([`templates::attack_slot`]), and the Attack goes to every selected unit ([`recipients`]): those that can attack
//! carry it out, an Attack on an object or an Attack Move on the ground, and the others move, to the spot with the
//! group (the Attack Move, which plays out as a move for them) or to the object (a Move to it).
//!
//! The Colonists' Legionnaires convert between their two forms with R (`factions/colonists/units/weapon_conversion.md`,
//! [`PanelCommand::Convert`]), each form's own command, so that in a selection of both it goes to the ActiveGroup alone
//! (design change 0198), issued at once and never queued, and never refused: it flips each unit's target form. The
//! units switch their type at the press as the shield's do, joining the other form's group, which becomes the
//! ActiveGroup, and are drawn by their form, the weapon in hand, until their conversion is over
//! (`crate::object_render`, `sc_sim::conversion::form`).
//!
//! A Barracks (`factions/gdo/structures/barracks.md > ObjectInterfaceState[Barracks]`) trains a Peacekeeper (Q) and
//! cancels the last unit queued (X), at once, and awaits the target of its rally point (C); a Train or a Cancel
//! Production that the command gate would refuse shows disabled ([`Conditions`]: the same predicates as the gate).
//! Every producing structure has this panel, built from its recipes (the Production template, plan decision T23): the
//! Colonists' Robo Portal trains a Drone in Q, the Bio Portal a Sabre Legionnaire
//! (`factions/colonists/structures/robo_portal.md > Robo Portal ObjectInterfaceState`,
//! `factions/colonists/structures/bio_portal.md > Bio Portal ObjectInterfaceState`, design change 0124).
//!
//! A Deployment Center (`factions/gdo/structures/deployment_center.md > ObjectInterfaceState[DeploymentCenter]`,
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
//! yet show unavailable. A Drone's W (`factions/colonists/units/drone.md > Drone ObjectInterfaceState`) enters
//! AwaitingPlacement of a Beacon under the cursor; each click sends one Drone of the group to deploy there (design
//! change 0125), with Shift the next one that has no Deploy yet. Whatever transforms, a Beacon under deployment or
//! a building under Warp In, shows X Cancel and nothing else (design change 0116): the unfinished buildings of a type
//! form a SelectionGroup of their own, with that panel, apart from the finished ones and their panel (design change
//! 0185, `crate::selection::GroupType`). Warp In applies to the Beacon Tabbed to (Beacons are Ungroupable, design
//! change 0122).
//!
//! The Syndicate's Tunnel (`factions/syndicate/tunnel_menus.md > Tunnel ObjectInterfaceState`, design change 0257;
//! slice M4-J2) opens its EjectMenu (E): a tile per unit type of the player's Tunnel Network, with its count, in the
//! sim's order, each ejecting the unit of its type inside the longest, at once, the menu staying open (design change
//! 0300, answering Q0086 point 3: "The panel stays in the EjectMenu after a click"); Z goes back. Its W
//! opens the ExpandMenu (slice M4-F): every expansion of the Tech Tree in its slot, in the sim's order
//! (`sc_sim::defs::EXPAND_MENU`), greyed with why while this Tunnel can't build it (its operation, its tier, the bank,
//! or not in the game yet); an entry enters the expansion's placement, its ghost held within the Tunnel Area, where a
//! click builds it and returns to DefaultState, and Z goes back to the menu. Its X cancels its operation, and its Q
//! upgrades it (slice M4-G). The Hideout's Q, W and E train the Agent, the Guard and the Grenadier, each greyed with
//! why while the gate refuses it (slices M4-K and M4-U).
//!
//! The Cults' Armory (`factions/cults/structures/armory.md > ObjectInterfaceState[Armory]`, design change 0337; slice
//! M5-F2) picks a kind of Recruit first: Q Plain, W Constitution, E Metabolism and R Both, each with the count of its
//! free Recruits inside and greyed while there are none, open KindMenu\[kind\], whose Q trains a Militant of a Recruit
//! of the kind and W a Gunner, greyed while the player owns no Gunworks (slice M5-G2: "Requires a Gunworks", the
//! gate's `Reject::Requires`), and A releases one, the menu staying open, Z going back. Its
//! A ejects every free Recruit, X cancels the last entry and C awaits its rally point. Every command goes to the
//! Armory Tabbed to (design change 0348 point 5, `factions/cults/structures/README.md > Choosing Recruits, Release and
//! Eject All`: "a command goes to one building").
//!
//! The Cults' Altar (`factions/cults/augmentation.md > Altar > ObjectInterfaceState[Altar]`, slice M5-N1b) picks a
//! unit type first, Q Recruit, W Militant, E Gunner and R Disciple, each with the count of its free units inside and
//! greyed while there are none, opening AugmentMenu\[UnitType\]: Q Unaugmented, W Enhanced Constitution only, E Improved
//! Metabolism only and R Fully augmented, the same, each opening AugmentMenu\[UnitType, State\], whose orders the units
//! of the state may take queue one each and A releases one, the menu staying open; Z goes back a stage. Its A, X and C
//! are the Armory's, and every command goes to the Altar Tabbed to.
//!
//! The Syndicate's Agent (`factions/syndicate/units/agent.md > Agent ObjectInterfaceState`, design changes 0257,
//! 0359 and 0398; slices M4-J1 and M5-Y) has Q Move, W Build Tunnel, A Attack, S Drop Off, D Repair (slice M5-N8) and
//! X Stop: a Laser Turret is the Tunnel's ExpandMenu's alone. W enters AwaitingPlacement of a Tunnel built by the Agent
//! Tabbed to ([`Builder::Agent`]), greyed while the gate would refuse it (the bank can't pay its price, which its hint
//! shows, [`PanelSlot::price`], design change 0233); a click sends that Agent its Build and returns to DefaultState,
//! Shift held or not. S awaits the Tunnel to drop off at, greyed while no Agent carries a load; D a damaged own Tunnel
//! or Laser Turret, which the Agents repair (`sc_sim::OrderKind::Repair`). A site under construction, of any type,
//! shows X Cancel Construction alone ([`SITE_GRID`], design change 0237), but its C if it will produce or release units
//! (slice M5-S6, design change 0396: a Hideout's).
//!
//! The Cults' Recruit (`factions/cults/units/recruit.md > Recruit ObjectInterfaceState`, design changes 0322, 0327
//! and 0398; slice M5-J1) has Q Move, W Basic Construct, E Advanced Construct, A Attack, S Assist Construction, D
//! Repair (slice M5-N8) and X Stop. W and E open the two Construct menus, fixed slots from the sim's table
//! (`sc_sim::defs::CONSTRUCT_MENU`), every building in its slot, greyed with why until it can be built (its
//! TechPrerequisite, the bank, or not in the game yet); Z goes back. An entry enters AwaitingPlacement of the building
//! that all the selected Recruits build together ([`Builder::Recruits`]: Groupable, they are one ActiveGroup, and the
//! group's Build is paid once); a click sends it and returns to DefaultState, a Shift-click queues it and placement
//! stays while Shift is held, for a chain of buildings. S awaits an own site under construction, which the Recruits
//! assist (`sc_sim::OrderKind::Assist`); D a damaged own building or vehicle, which they repair
//! (`sc_sim::OrderKind::Repair`).
//!
//! A greyed-out button, or its hotkey, can still be pressed: nothing is issued, and the player is told why
//! ([`PanelSlot::why`], design change 0162).
//!
//! No smart-casting (`systems/controls/selection.md > SelectionGroup > No smart-casting`, design changes 0071
//! and 0075): a panel command goes to every object it is issued to ([`recipients`]), a Train to every selected
//! Barracks, whose payments the sim orders (design change 0083).
//!
//! While a target is awaited or a building is placed, the other commands are unavailable
//! (`systems/controls/interface.md > AwaitingTarget > Other commands while targeting`, design change 0066): the
//! panel shows them disabled, and their hotkeys and buttons do nothing.

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
    BuildMenu, InterfaceTemplate, Planned, SITE_GRID, TRANSFORMING_GRID, can_target_ground, commands, grid, menu_grid,
    menu_of, offers, slot_index, state_grid,
};

use sc_sim::{ObjectKind, RecruitKind, deploy};

use crate::selection::Selection;

/// `systems/controls/interface.md > ObjectInterfaceState[ObjectEnum]`: where the ActiveGroup's command flow
/// stands. It belongs to each player's ControlState (`crate::selection::ControlState::interface`), and goes back to
/// DefaultState when the Selection or the ActiveGroup changes (`crate::selection::LocalControl::update`), or when it no
/// longer holds for the game ([`still_valid`]). The production UI needs no state of its own: Set Rally Point awaits its
/// target like a unit command.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum InterfaceState {
    /// `systems/controls/interface.md > DefaultState`.
    #[default]
    DefaultState,
    /// `systems/controls/interface.md > AwaitingTarget[CommandType]`: the PendingCommand waits for a left-click
    /// on its target: a unit command's, or a producer's rally point (`factions/gdo/structures/barracks.md`: "C: Set
    /// Rally Point: enters AwaitingTarget\[SetRallyPoint\]").
    AwaitingTarget {
        /// The PendingCommand.
        pending: PanelCommand,
        /// `systems/controls/interface.md > AwaitingTarget > Shift while targeting` (design change 0070): a
        /// Shift-click has queued the command since the target began to be awaited. Targeting then ends when Shift is
        /// released; before the first queued click a release of Shift changes nothing (the design's TBD, its proposed
        /// default: "targeting simply continues").
        queued: bool,
    },
    /// `factions/gdo/structures/deployment_center.md > ObjectInterfaceState[DeploymentCenter]`: one of the
    /// Deployment Center's build menus is open (design change 0104), in whichever state the Deployment Center is:
    /// idle, constructing or holding a building ready; or one of a Beacon's Warp In menus
    /// (`factions/colonists/economy.md > Beacon > Beacon ObjectInterfaceState`).
    BuildMenu(BuildMenu),
    /// `systems/controls/hud.md > AwaitingPlacement`: the ghost of a building ready to place follows the
    /// cursor, and a left-click on a valid spot places it (`crate::placement`); the same for a Drone's Beacon, and a
    /// Beacon's Warp In, whose ghost stands on the Beacon ([`Builder`]).
    AwaitingPlacement(Placing),
    /// `factions/syndicate/tunnel_menus.md > EjectMenu`: the Tunnel's EjectMenu is open (E, design change 0257): a tile
    /// per unit type of the player's Tunnel Network, which ejects one from this Tunnel and leaves the menu open
    /// (`PanelCommand::Eject`; design change 0300: "**The panel stays in the EjectMenu after a click**", so repeated
    /// clicks eject more); "**Z**: returns to DefaultState (StateOnlyTransition)". Like a build menu, it leaves
    /// the map's clicks to the selection (slice M4-J2).
    EjectMenu,
    /// `factions/syndicate/tunnel_menus.md > ExpandMenu`: the Tunnel's ExpandMenu is open (W, design change 0257): the
    /// expansions it may build, each entering its placement in the Tunnel Area (`PanelCommand::Expand`); "**Z**:
    /// returns to DefaultState (StateOnlyTransition)". Like a build menu, it leaves the map's clicks to the selection
    /// (slice M4-F).
    ExpandMenu,
    /// `factions/cults/structures/README.md > Choosing Recruits, Release and Eject All`: a pool building's
    /// KindMenu\[kind\] is open (design change 0337: "Q/W/E/R pick a kind"): "the building's production options, using
    /// Recruits of that kind [...], plus **A: Release**; **Z** back. Clicks stay in the menu so they can be repeated."
    /// Like a build menu, it leaves the map's clicks to the selection (slice M5-F2).
    KindMenu(RecruitKind),
    /// (M5-N1b) `factions/cults/augmentation.md > Altar > ObjectInterfaceState[Altar] > AugmentMenu[UnitType]`: an
    /// Altar's AugmentationState tiles for the unit type chosen in its DefaultState ("**Z**: returns to DefaultState
    /// (StateOnlyTransition)"). Like a build menu, it leaves the map's clicks to the selection.
    AugmentMenu(ObjectKind),
    /// (M5-N1b) `... > AugmentMenu[UnitType, State]`: the orders a unit of the type and the state may take, and Release
    /// ("Each click queues one order (CommandIssuingTransition) and stays in this menu so further clicks queue more";
    /// "**Z**: returns to AugmentMenu[UnitType] (StateOnlyTransition)").
    AugmentStateMenu(ObjectKind, RecruitKind),
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
            InterfaceState::DefaultState
            | InterfaceState::BuildMenu(_)
            | InterfaceState::AwaitingPlacement(_)
            | InterfaceState::EjectMenu
            | InterfaceState::ExpandMenu
            | InterfaceState::KindMenu(_)
            | InterfaceState::AugmentMenu(_)
            | InterfaceState::AugmentStateMenu(..) => None,
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
    /// nothing, and a drag on the map does nothing (`systems/controls/interface.md > AwaitingTarget > Dragging
    /// while targeting`, design change 0067; the same for a placement). A build menu leaves the map's clicks to the
    /// selection. These are the states a right-click cancels, straight to DefaultState (design change 0400: "Only when
    /// the next step is a **left-click on a location or target** [...] does a right-click cancel").
    pub fn takes_map_clicks(self) -> bool {
        matches!(
            self,
            InterfaceState::AwaitingTarget { .. } | InterfaceState::AwaitingPlacement(_)
        )
    }

    /// The state before this one, which Back (Z) and Esc return to (`systems/controls/interface.md > Standard Slot
    /// Assignments > Z`: "In any multi-stage menu (BuildMenu, ExpandMenu, EjectMenu, AwaitingTarget,
    /// AwaitingPlacement), Z returns to the previous state (StateOnlyTransition). Equivalent to Escape."; a right-click
    /// no longer does, design change 0400: `super::systems::right_click_orders`): from AwaitingPlacement of a
    /// Deployment Center's building, its build menu (`factions/gdo/structures/deployment_center.md >
    /// AwaitingPlacement`: "Escape: returns to BuildMenu"), but DefaultState from one entered with its E: Place
    /// (`Placing::shortcut`, design change 0388: "Escape/right-click from there returns to DefaultState"), and from a
    /// Beacon's Warp In the Warp In menu its building was
    /// chosen in (the menus are "like the Deployment Center's two build menus": `factions/colonists/economy.md >
    /// Beacon > Beacon ObjectInterfaceState`), and from an expansion's the Tunnel's ExpandMenu
    /// (`factions/syndicate/tunnel_menus.md > Expansions under construction`: "**Z**: returns to ExpandMenu"); from
    /// anything else DefaultState: the Extraction Facility's AwaitingPlacement ("Escape/right-click: returns to
    /// DefaultState"), the Drones' (W is pressed in their DefaultState), a build menu ("Escape: returns to
    /// DefaultState"), AwaitingTarget, and the Tunnel's EjectMenu and ExpandMenu (`factions/syndicate/tunnel_menus.md >
    /// EjectMenu`, `factions/syndicate/tunnel_menus.md > ExpandMenu`: "**Z**: returns to DefaultState"), and a pool
    /// building's KindMenu ("**Z** back"). An Altar's AugmentMenu\[UnitType, State\] goes back to AugmentMenu\[UnitType\],
    /// that to DefaultState (`factions/cults/augmentation.md > Altar > ObjectInterfaceState[Altar]`: "Z returns to the
    /// previous stage at every stage").
    pub fn back(self, selection: &Selection) -> InterfaceState {
        match self {
            InterfaceState::AugmentStateMenu(unit, _) => InterfaceState::AugmentMenu(unit),
            InterfaceState::AwaitingPlacement(Placing {
                builder: Builder::Tunnel(_),
                ..
            }) => InterfaceState::ExpandMenu,
            InterfaceState::AwaitingPlacement(placing) if placing.shortcut => InterfaceState::DefaultState,
            InterfaceState::AwaitingPlacement(placing) => selection
                .active()
                .and_then(|active| menu_of(active.kind, placing.kind))
                .map_or(InterfaceState::DefaultState, InterfaceState::BuildMenu),
            _ => InterfaceState::DefaultState,
        }
    }
}

/// `systems/controls/interface.md > ObjectInterfaceState`: whether `interface` still holds for `selection`
/// (validated every frame), while the selection is commandable (a defeated player's and everyone's after the end of the
/// game is not):
/// - AwaitingTarget while its ActiveGroup's DefaultState panel still has the PendingCommand, which the game still
///   allows (`conditions`): the selection's Attack while one of the selected units can attack, whichever group is in
///   focus (design change 0399);
/// - a build menu while the ActiveGroup is a Deployment Center, a Beacon that isn't being deployed, or Recruits (slice
///   M5-J1): the menu stays open while its state changes (a construction that completes shows its building ready in the
///   menu);
/// - AwaitingPlacement while the ActiveGroup is the constructor, which still holds the building ready
///   (`conditions`: its state); while it is Drones; while it is the Beacon, which isn't being deployed (the bank is
///   the click's to check: `crate::command_ui::feedback` says why it is refused); while it is the Agent placing, the
///   bank the click's to check as well (slice M4-J1); while it is the Tunnel, whose type builds the expansion (its
///   operation, its tier and the bank are the click's to check, as a Warp In's bank; slice M4-F); while it is
///   Recruits, whose type builds the building (the bank and the ground are the click's to check; slice M5-J1);
/// - the EjectMenu and the ExpandMenu while the ActiveGroup is a Tunnel: the menu stays open while the network's
///   contents or the Tunnel's operation change, its entries greyed while they can't be used;
/// - a KindMenu while the ActiveGroup is a pool building: the menu stays open while its pool changes, its kind's
///   last Recruit used or gone out included (its entries greyed then), so that clicks can be repeated;
/// - an AugmentMenu while the ActiveGroup is an Altar with its tile: the menu stays open while its units change, as a
///   KindMenu does (slice M5-N1b).
///
/// A selected object that transforms has the panel of its new type's unfinished group, only its Cancel (design
/// changes 0116 and 0185): what its old type was doing ends (`crate::selection::ControlState::set_type`), and so does
/// a target awaited for it (the panel offers no command but the Cancel), but (M5-S6) a rally point's, on a building
/// that will produce or release units, which offers its C (design change 0396).
pub fn still_valid(
    interface: InterfaceState,
    selection: &Selection,
    commandable: bool,
    conditions: &Conditions,
) -> bool {
    match interface {
        InterfaceState::DefaultState => true,
        InterfaceState::AwaitingTarget { pending, .. } => {
            let default = panel(selection, InterfaceState::DefaultState, conditions);
            commandable && (default.slots()).any(|(slot, _)| press(&default, slot) == SlotPress::AwaitTarget(pending))
        }
        InterfaceState::BuildMenu(menu) => {
            commandable
                && selection
                    .active()
                    .is_some_and(|active| offers(active.group_type(), PanelCommand::BuildMenu(menu)))
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
                            && selection
                                .active()
                                .is_some_and(|active| offers(active.group_type(), PanelCommand::WarpIn(placing.kind)))
                    }
                    Builder::Agent(agent) => {
                        selection.in_active_group(agent)
                            && selection
                                .active()
                                .is_some_and(|active| offers(active.group_type(), PanelCommand::Build(placing.kind)))
                    }
                    Builder::Tunnel(tunnel) => {
                        selection.in_active_group(tunnel)
                            && selection
                                .active()
                                .is_some_and(|active| offers(active.group_type(), PanelCommand::Expand(placing.kind)))
                    }
                    Builder::Recruits { .. } => selection
                        .active()
                        .is_some_and(|active| offers(active.group_type(), PanelCommand::Build(placing.kind))),
                }
        }
        InterfaceState::EjectMenu
        | InterfaceState::ExpandMenu
        | InterfaceState::KindMenu(_)
        | InterfaceState::AugmentMenu(_)
        | InterfaceState::AugmentStateMenu(..) => {
            let menu = match interface {
                InterfaceState::EjectMenu => PanelCommand::EjectMenu,
                InterfaceState::KindMenu(kind) => PanelCommand::KindMenu(kind),
                InterfaceState::AugmentMenu(unit) => PanelCommand::AugmentMenu(unit),
                InterfaceState::AugmentStateMenu(unit, state) => PanelCommand::AugmentState(unit, state),
                _ => PanelCommand::ExpandMenu,
            };
            commandable
                && selection
                    .active()
                    .is_some_and(|active| offers(active.group_type(), menu))
        }
    }
}

#[cfg(test)]
mod tests;
