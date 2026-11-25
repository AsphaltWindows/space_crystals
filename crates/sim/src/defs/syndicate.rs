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

#[cfg(test)]
mod tests;
