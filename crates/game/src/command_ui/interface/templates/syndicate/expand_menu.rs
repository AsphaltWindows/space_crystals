//! The Tunnel's ExpandMenu (`factions/syndicate/tunnels.md > Tunnel ObjectInterfaceState > ExpandMenu`, slice M4-F):
//! "Displays available underground expansion types for this Tunnel's current tier". Its entries take the grid's slots
//! in the order of the sim's list, the one table of its slots (design change 0257: "entries take the grid slots in
//! order (Q, W, E, R, A, S, …): expansions in the Tech Tree table's order (Hideout, Optics Lab, Chemical Plant, Vault,
//! …)"; `sc_sim::defs::EXPAND_MENU`), every slot but Z's, which is Back, as the EjectMenu's tiles do
//! (`super::eject_menu::tile_slots`). Every expansion of the Tech Tree holds its slot, those not in the game yet too,
//! greyed (design change 0291, answering Q0081: "every expansion is listed in table order. Unavailable ones are
//! greyed, including ones not in the build yet"), so that no letter moves as they come: a type in the game is its
//! [`PanelCommand::Expand`], greyed while this Tunnel can't build it now (its operation, its tier, the bank: the gate's
//! reasons, `sc_sim::Sim::can_expand`), and one not in the game yet its [`Planned`] entry. The Laser Turret takes the
//! slot after them, the tenth (0291: "The Laser Turret is always the tenth slot"): a placeholder too
//! ([`Planned::LaserTurret`]), greyed with its prerequisite, until slice M4-S2 brings a Tunnel's turret to the
//! ExpandMenu (an Agent builds one since slice M4-S1).

use sc_sim::defs::{EXPAND_MENU, ExpandEntry};

use super::super::{PanelCommand, Planned};
use super::eject_menu::tile_slots;

/// The command of an ExpandMenu entry: the Expand of a type in the game, or the placeholder of one not in it yet (by
/// its name, `Planned::expansion`). `None` for a name the client has no placeholder for, which a test rules out.
pub fn command(entry: ExpandEntry) -> Option<PanelCommand> {
    match entry {
        ExpandEntry::Kind(kind) => Some(PanelCommand::Expand(kind)),
        ExpandEntry::Planned(name) => Planned::expansion(name).map(PanelCommand::Planned),
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
