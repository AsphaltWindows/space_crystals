//! The Tunnel's EjectMenu (`factions/syndicate/tunnels.md > Tunnel ObjectInterfaceState`, slice M4-J2): "a grid of
//! unit type tiles representing all units currently in the Tunnel Network. Each tile shows the unit type icon and a
//! count of how many of that type are in the network" (`> EjectMenu`). Its tiles take the grid's slots in the order of
//! the sim's list (design change 0257: "entries take the grid slots in order (Q, W, E, R, A, S, …)", "Agent Q, Guard
//! W, Grenadier E" in Q0068's words; `sc_sim::defs::EJECT_MENU_ORDER`), every slot but Z's, which is Back. Each type
//! holds its slot whether or not any of it is inside: one with none shows a count of 0, greyed ("unavailable entries
//! are greyed"), and so does one this Tunnel's tier doesn't let out ("visible but greyed out (disabled)") and one
//! whose every unit only just walked in (design change 0288 point 1: "When every unit of a type is still in the 2 s
//! transit delay, its tile is greyed and the click is refused; nothing is queued"), each with the gate's reason
//! (`crate::command_ui::Conditions`).
//!
//! A unit of a type not in the list is never inside in a game: only a debug game puts another faction's units in the
//! network (the sim's Enter takes any unit of the Tunnel's owner). Placeholder (not in the design): such a type has no
//! tile.

use sc_sim::ObjectKind;
use sc_sim::defs::EJECT_MENU_ORDER;

use super::super::{PanelCommand, slot_index};
use crate::input::CommandSlot;

/// The slots the tiles take, in order: Q, W, E, R, A, S, D, F, X, C, V, every slot of the grid but Back's, Z
/// (`systems/controls/selection_and_hud.md > Standard Slot Assignments > Z`). The ExpandMenu's entries take them alike
/// (`super::expand_menu`, design change 0257: "In the **ExpandMenu** and **EjectMenu**, entries take the grid slots in
/// order").
pub(super) fn tile_slots() -> impl Iterator<Item = usize> {
    CommandSlot::all()
        .filter(|&slot| slot != CommandSlot::BACK)
        .map(slot_index)
}

/// The unit types of the menu, in their slots' order: the sim's list (module docs).
pub fn kinds() -> &'static [ObjectKind] {
    EJECT_MENU_ORDER
}

/// The EjectMenu, slot by slot (`CommandSlot::all` order): an Eject tile for each unit type of the list, from Q on
/// ([`grid_of`]).
pub fn grid() -> [Option<PanelCommand>; 12] {
    grid_of(kinds())
}

/// The EjectMenu of the unit types `kinds`, slot by slot: their tiles in the order of [`tile_slots`], as many as it
/// has.
pub fn grid_of(kinds: &[ObjectKind]) -> [Option<PanelCommand>; 12] {
    let mut grid = [None; 12];
    debug_assert!(
        kinds.len() <= tile_slots().count(),
        "the EjectMenu has 11 tiles at most"
    );
    for (slot, &kind) in tile_slots().zip(kinds) {
        grid[slot] = Some(PanelCommand::Eject(kind));
    }
    grid
}
