//! The Recruits' two Construct menus (`factions/cults/units/recruit.md > Recruit ObjectInterfaceState > DefaultState
//! commands`, design change 0327, slice M5-J1), from the sim's table of their slots, the one table of them
//! (`sc_sim::defs::CONSTRUCT_MENU`): "Fixed slots: every building keeps its key and is greyed until its
//! TechPrerequisite is met, so no hotkey ever moves". Each entry takes its slot whatever it holds: a type in the game
//! is its [`PanelCommand::Build`], greyed while the gate would refuse the Recruits' Build (its TechPrerequisite, the
//! bank: `super::super::super::Conditions`). One not in the game yet held its slot as a `Planned` entry until slice
//! M5-N6 made the last of them, the Chop Shop, live. Z is Back (`crate::command_ui::interface::panel`).

use sc_sim::ObjectKind;
use sc_sim::defs::{CONSTRUCT_MENU, ConstructEntry, ConstructMenu};

use super::super::{BuildMenu, PanelCommand, slot_index};
use crate::input::CommandSlot;

/// The command of a Construct menu's entry: the Build of a type in the game. `None` for an entry not in the game yet
/// (`ConstructEntry::Planned`), which the client has no placeholder for since slice M5-N6: the menus hold none, as the
/// sim's tests check (`sc_sim::defs::cults`).
pub fn command(entry: ConstructEntry) -> Option<PanelCommand> {
    match entry {
        ConstructEntry::Kind(kind) => Some(PanelCommand::Build(kind)),
        ConstructEntry::Planned(_) => None,
    }
}

/// The sim's Construct menu that the panel's `menu` is; none for the Field Engineer's, which a Recruit hasn't.
pub fn of(menu: BuildMenu) -> Option<ConstructMenu> {
    match menu {
        BuildMenu::Basic => Some(ConstructMenu::Basic),
        BuildMenu::Advanced => Some(ConstructMenu::Advanced),
        BuildMenu::Construct => None,
    }
}

/// The Construct menu `menu`, slot by slot (`CommandSlot::all` order): each entry of the sim's table in its slot.
pub fn grid(menu: BuildMenu) -> [Option<PanelCommand>; 12] {
    let mut grid = [None; 12];
    for slot in CONSTRUCT_MENU.iter().filter(|slot| Some(slot.menu) == of(menu)) {
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
