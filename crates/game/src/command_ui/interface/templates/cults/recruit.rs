//! The Recruit's panel (`factions/cults/units/recruit.md > Recruit ObjectInterfaceState`, slice M5-J1; design change
//! 0322, answering Q0095 and Q0096: "Same layout as the Agent's panel, plus W for the second build menu"): **Q** Move
//! and **X** Stop, "as in BasicCombatUnitInterfaceState" (their slots there); **A** Basic Construct and **W** Advanced
//! Construct, which enter the two Construct menus ([`super::construct_menu`]: `PanelCommand::BuildMenu`, opened in
//! `InterfaceState::BuildMenu`), whose buildings enter their placement, the Recruits' group Build
//! (`crate::command_ui::interface::Builder::Recruits`); **S** Assist Construction, which awaits an own site
//! (`PanelCommand::Assist`); and **D** Repair, greyed until slice N8 brings it (`Planned::Repair`). Its right-click
//! (`> Right-Click Resolution`: Gather, Drop Off, Enter, Assist, Attack, Move; Repair from slice N8) is the sim's one
//! rule, as every unit's (`crate::command_ui::resolution::unit_right_click`). Recruits are Groupable: all the selected
//! Recruits are the one ActiveGroup that every command goes to ("both right-click and panel commands (Construct,
//! Assist Construction, Repair) are issued to all of them").

use super::super::{BuildMenu, InterfaceTemplate, PanelCommand, PanelTemplate, Planned};

/// The Recruit's panel template, with the names the design gives its menus (`PanelCommand::name_on`): "A: Basic
/// Construct" and "W: Advanced Construct", the Deployment Center's being "Basic Construction" and "Advanced
/// Construction".
pub const TEMPLATE: PanelTemplate = PanelTemplate {
    names: &[
        (PanelCommand::BuildMenu(BuildMenu::Basic), "Basic Construct"),
        (PanelCommand::BuildMenu(BuildMenu::Advanced), "Advanced Construct"),
    ],
    ..PanelTemplate::of(InterfaceTemplate::Recruit)
};

/// `> DefaultState commands`, slot by slot: Q Move, W Advanced Construct; A Basic Construct, S Assist Construction,
/// D Repair; X Stop.
#[rustfmt::skip]
pub const GRID: [Option<PanelCommand>; 12] = {
    use PanelCommand::{Assist, Move, Stop};
    const BASIC: PanelCommand = PanelCommand::BuildMenu(BuildMenu::Basic);
    const ADVANCED: PanelCommand = PanelCommand::BuildMenu(BuildMenu::Advanced);
    const REPAIR: PanelCommand = PanelCommand::Planned(Planned::Repair);
    [
        Some(Move),  Some(ADVANCED), None,         None,
        Some(BASIC), Some(Assist),   Some(REPAIR), None,
        None,        Some(Stop),     None,         None,
    ]
};
