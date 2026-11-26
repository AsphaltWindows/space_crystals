//! The Field Engineer's panel (`factions/gdo/units/field_engineer.md > Placeholder stats`, design change 0328, slice
//! M5-N9): "**Q** Move; **A** Construct (a menu: **Q** Bunker, **W** First Aid Tent, both greyed until designed; **Z**
//! back); **D** Repair (targeted: a damaged own building or vehicle); **X** Stop". Q and X are in their
//! BasicCombatUnitInterfaceState slots, as the Recruit's and the Agent's; D awaits its target as theirs does
//! (`PanelCommand::Repair`, slice M5-N8); A opens its Construct menu, a build menu of its own
//! ([`BuildMenu::Construct`], opened in `InterfaceState::BuildMenu` as the Recruit's two are), whose two buildings
//! aren't in the game yet ([`Planned`]). Its button reads "Constr", the six letters a label has room for, and its
//! hint "A: Construct" (`PanelCommand::label`, `PanelCommand::name`). Its right-click ("a damaged own building or vehicle: Repair (Enter wins on a building it can enter,
//! as for Recruits)") is the sim's one rule, as every unit's (`crate::command_ui::resolution::unit_right_click`).

use super::super::{BuildMenu, InterfaceTemplate, PanelCommand, PanelTemplate, Planned};

/// Its A, the Construct menu.
const CONSTRUCT: PanelCommand = PanelCommand::BuildMenu(BuildMenu::Construct);

/// The Field Engineer's panel template.
pub const TEMPLATE: PanelTemplate = PanelTemplate::of(InterfaceTemplate::FieldEngineer);

/// Its DefaultState commands, slot by slot: Q Move; A Construct, D Repair; X Stop.
#[rustfmt::skip]
pub const GRID: [Option<PanelCommand>; 12] = {
    use PanelCommand::{Move, Repair, Stop};
    [
        Some(Move),      None,       None,         None,
        Some(CONSTRUCT), None,       Some(Repair), None,
        None,            Some(Stop), None,         None,
    ]
};

/// Its Construct menu, slot by slot: "**Q** Bunker, **W** First Aid Tent, both greyed until designed"; Z is Back
/// (`crate::command_ui::interface::panel`).
#[rustfmt::skip]
pub const CONSTRUCT_MENU: [Option<PanelCommand>; 12] = {
    use PanelCommand::Planned as P;
    [
        Some(P(Planned::Bunker)), Some(P(Planned::FirstAidTent)), None, None,
        None,                     None,                           None, None,
        None,                     None,                           None, None,
    ]
};

/// The entries of its build menu `menu`, slot by slot: its Construct menu, the one its A opens; none for the others,
/// which it hasn't.
pub fn menu(menu: BuildMenu) -> [Option<PanelCommand>; 12] {
    match menu {
        BuildMenu::Construct => CONSTRUCT_MENU,
        BuildMenu::Basic | BuildMenu::Advanced => [None; 12],
    }
}
