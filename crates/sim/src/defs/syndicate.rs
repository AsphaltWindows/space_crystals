//! The Syndicate — `factions/syndicate/`: its own rules, and one module per unit (`units/`,
//! `factions/syndicate/units/*.md`) and per structure (`structures/`: the Tunnel, `factions/syndicate/tunnels.md >
//! Tunnel`, and the expansions, `factions/syndicate/structures/*.md`).

pub(crate) mod structures;
pub(crate) mod units;

/// `factions/syndicate/tunnels.md > Tunnel Space`: "capped at a hard maximum of 200" (`factions/syndicate/overview.md >
/// DisplayHud`: "Tunnel Space: current used / current available (max 200)"; `economy::army`).
pub const TUNNEL_SPACE_CAP: u32 = 200;

/// `factions/syndicate/tunnels.md > Tunnel Space`: the unused tunnel-region cells that make one Tunnel Space ("... /
/// 4) — rounded down to a whole number"; `economy::army`).
pub const CELLS_PER_TUNNEL_SPACE: u32 = 4;

#[cfg(test)]
mod tests;
