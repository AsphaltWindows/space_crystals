//! The Tunnel's ExpandMenu (`factions/syndicate/tunnel_menus.md > Tunnel ObjectInterfaceState > ExpandMenu`, slice
//! M4-F): "Displays available underground expansion types for this Tunnel's current tier". Its entries take the grid's
//! slots in the order of the sim's list, the one table of its slots (design change 0257: "entries take the grid slots
//! in order (Q, W, E, R, A, S, …): expansions in the Tech Tree table's order (Hideout, Optics Lab, Chemical Plant,
//! Vault, …)"; `sc_sim::defs::EXPAND_MENU`), every slot but Z's, which is Back, as the EjectMenu's tiles do
//! (`super::eject_menu::tile_slots`). Every expansion of the Tech Tree holds its slot, those not in the game yet too,
//! greyed (design change 0291, answering Q0081: "every expansion is listed in table order. Unavailable ones are
//! greyed, including ones not in the build yet"), so that no letter moves as they come: a type in the game is its
//! [`PanelCommand::Expand`], greyed while this Tunnel can't build it now (its operation, its tier, the bank: the gate's
//! reasons, `sc_sim::Sim::can_expand`), and one not in the game yet its [`Planned`] entry. The Laser Turret takes the
//! slot after them, the tenth (0291: "The Laser Turret is always the tenth slot"): its Expand too, since slice M4-S2
//! (`factions/syndicate/structures/laser_turret.md > Construction`: "**By a Tunnel**: from the Tunnel's ExpandMenu"),
//! greyed by the same gate: "Requires an Optics Lab" until the player owns one, and the Tunnel's one operation slot,
//! which its turret takes like an expansion (design change 0267).

use sc_sim::defs::{EXPAND_MENU, ExpandEntry};

use super::super::PanelCommand;
use super::eject_menu::tile_slots;

/// The command of an ExpandMenu entry: the Expand of its type. `None` for an entry not in the game, of which the sim's
/// menu has none (a test checks).
pub fn command(entry: ExpandEntry) -> Option<PanelCommand> {
    match entry {
        ExpandEntry::Kind(kind) => Some(PanelCommand::Expand(kind)),
        ExpandEntry::Planned(_) => None,
    }
}

/// The ExpandMenu, slot by slot (`CommandSlot::all` order): the sim's entries in the order of the tile slots, Q on,
/// each in its slot whatever it holds.
pub fn grid() -> [Option<PanelCommand>; 12] {
    grid_of(EXPAND_MENU)
}

/// The ExpandMenu of `entries`, slot by slot: their commands in the order of the tile slots, as many as it has.
pub fn grid_of(entries: &[ExpandEntry]) -> [Option<PanelCommand>; 12] {
    let mut grid = [None; 12];
    debug_assert!(
        entries.len() <= tile_slots().count(),
        "the ExpandMenu has 11 entries at most"
    );
    for (slot, &entry) in tile_slots().zip(entries) {
        grid[slot] = command(entry);
    }
    grid
}

/// The types the ExpandMenu builds now, in its order: its entries in the game.
pub fn kinds() -> impl Iterator<Item = sc_sim::ObjectKind> {
    EXPAND_MENU.iter().filter_map(|entry| entry.kind())
}
