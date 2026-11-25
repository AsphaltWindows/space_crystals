//! The Syndicate — `factions/syndicate/`: its own rules, and one module per unit (`units/`,
//! `factions/syndicate/units/*.md`) and per structure (`structures/`: the Tunnel, `factions/syndicate/tunnels.md >
//! Tunnel`, and the expansions, `factions/syndicate/structures/*.md`).

pub(crate) mod structures;
pub(crate) mod units;

use super::ObjectKind;

/// `factions/syndicate/tunnels.md > Tunnel Space`: "capped at a hard maximum of 200" (`factions/syndicate/overview.md >
/// DisplayHud`: "Tunnel Space: current used / current available (max 200)"; `economy::army`).
pub const TUNNEL_SPACE_CAP: u32 = 200;

/// `factions/syndicate/tunnels.md > Tunnel Space`: the unused tunnel-region cells that make one Tunnel Space ("... /
/// 4) — rounded down to a whole number"; `economy::army`).
pub const CELLS_PER_TUNNEL_SPACE: u32 = 4;

/// `factions/syndicate/tunnels.md > Tunnel ObjectInterfaceState` (design change 0257, answering Q0068 point 4): the
/// EjectMenu's unit types, in the order they take the grid's slots: "entries take the grid slots in order (Q, W, E, R,
/// A, S, …)", the unit types "(Agent, Guard, Grenadier, …)", that is Agent Q, Guard W, Grenadier E (Q0068). The
/// section's own list is followed where `factions/syndicate/units/README.md > Files`, which it names as the order,
/// lists the Guard and the Grenadier before the Agent; the README's order goes on after the Grenadier (Assault Bike,
/// Shadebringer, …). Each Syndicate unit joins the list with its kind (the Grenadier with slice M4-U), so that the
/// client's menu (slice M4-J2) needs no edit: a test checks that it lists every Syndicate unit once.
pub const EJECT_MENU_ORDER: &[ObjectKind] = &[ObjectKind::Agent, ObjectKind::Guard];

/// An entry of the Tunnel's ExpandMenu ([`EXPAND_MENU`], slice M4-F): an expansion a Tunnel builds, or one of the Tech
/// Tree's not in the game yet, which holds its slot.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ExpandEntry {
    /// A type a Tunnel builds (its `StructureDef::expansion`).
    Kind(ObjectKind),
    /// An expansion of `factions/syndicate/structures/README.md > Tech Tree Overview` not in the game yet, by its
    /// name there: it holds its slot, greyed (Placeholder: Q0081, its proposed option A, "so that the Laser Turret's
    /// slot never moves"). The slice that brings it turns it into its `Kind`.
    Planned(&'static str),
}

impl ExpandEntry {
    /// Its name: its type's, or the Tech Tree's.
    pub fn name(self) -> &'static str {
        match self {
            ExpandEntry::Kind(kind) => kind.def().name,
            ExpandEntry::Planned(name) => name,
        }
    }

    /// The type it builds, if it is in the game.
    pub fn kind(self) -> Option<ObjectKind> {
        match self {
            ExpandEntry::Kind(kind) => Some(kind),
            ExpandEntry::Planned(_) => None,
        }
    }
}

/// `factions/syndicate/tunnels.md > Tunnel ObjectInterfaceState` (design change 0257): the ExpandMenu's entries, in the
/// order they take the grid's slots ("entries take the grid slots in order (Q, W, E, R, A, S, …): expansions in the
/// Tech Tree table's order (Hideout, Optics Lab, Chemical Plant, Vault, …)"), the one table of its slots (the client's
/// menu reads it). Every expansion of `factions/syndicate/structures/README.md > Tech Tree Overview` holds its slot,
/// those not in the game yet as [`ExpandEntry::Planned`] (Placeholder: Q0081), so that no letter moves as they come;
/// the Laser Turret takes the slot after them (`factions/syndicate/tunnels.md > ExpandMenu`: "also offers the Laser
/// Turret"), with slice S2. Tests check it against the table, and that it lists every type a Tunnel builds, once.
pub const EXPAND_MENU: &[ExpandEntry] = &[
    ExpandEntry::Kind(ObjectKind::Hideout),
    ExpandEntry::Planned("Optics Lab"),
    ExpandEntry::Planned("Chemical Plant"),
    ExpandEntry::Planned("Vault"),
    ExpandEntry::Planned("Vehicle Assembly"),
    ExpandEntry::Planned("Forge"),
    ExpandEntry::Planned("Null Chamber"),
    ExpandEntry::Planned("Aerospace Assembly"),
    ExpandEntry::Planned("Orbital Command"),
];

#[cfg(test)]
mod tests;
