//! The Recruits' two Construct menus (`factions/cults/units/recruit.md > Recruit ObjectInterfaceState > DefaultState
//! commands`, design change 0327, slice M5-J1), from the sim's table of their slots, the one table of them
//! (`sc_sim::defs::CONSTRUCT_MENU`): "Fixed slots: every building keeps its key and is greyed until its
//! TechPrerequisite is met, so no hotkey ever moves". Each entry takes its slot whatever it holds: a type in the game
//! is its [`PanelCommand::Build`], greyed while the gate would refuse the Recruits' Build (its TechPrerequisite, the
//! bank: `super::super::super::Conditions`), and one not in the game yet its [`Planned`] entry. Z is Back
//! (`crate::command_ui::interface::panel`).

use sc_sim::ObjectKind;
use sc_sim::defs::{CONSTRUCT_MENU, ConstructEntry, ConstructMenu};

use super::super::{BuildMenu, PanelCommand, Planned, slot_index};
use crate::input::CommandSlot;

/// The command of a Construct menu's entry: the Build of a type in the game, or the placeholder of one not in it yet
/// (by its name, `Planned::construct`). `None` for a name the client has no placeholder for, which a test rules out.
pub fn command(entry: ConstructEntry) -> Option<PanelCommand> {
    match entry {
        ConstructEntry::Kind(kind) => Some(PanelCommand::Build(kind)),
        ConstructEntry::Planned(name) => Planned::construct(name).map(PanelCommand::Planned),
    }
}

/// The sim's Construct menu that the panel's `menu` is.
pub fn of(menu: BuildMenu) -> ConstructMenu {
    match menu {
        BuildMenu::Basic => ConstructMenu::Basic,
        BuildMenu::Advanced => ConstructMenu::Advanced,
    }
}

/// The Construct menu `menu`, slot by slot (`CommandSlot::all` order): each entry of the sim's table in its slot.
pub fn grid(menu: BuildMenu) -> [Option<PanelCommand>; 12] {
    let mut grid = [None; 12];
    for slot in CONSTRUCT_MENU.iter().filter(|slot| slot.menu == of(menu)) {
        if let Some(at) = CommandSlot::new(slot.row, slot.column) {
            grid[slot_index(at)] = command(slot.entry);
        }
    }
    grid
}

/// The types the Construct menus build now, in the table's order: their entries in the game.
pub fn kinds() -> impl Iterator<Item = ObjectKind> {
    CONSTRUCT_MENU.iter().filter_map(|slot| slot.entry.kind())
}
