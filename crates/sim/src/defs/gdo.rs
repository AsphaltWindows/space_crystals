//! The GDO — `factions/gdo/`: its own rules, and one module per unit (`units/`, `factions/gdo/units/*.md`) and per
//! structure (`structures/`, `factions/gdo/structures/*.md`, the economy buildings in `factions/gdo/economy.md`).

pub(crate) mod structures;
pub(crate) mod units;

/// `factions/gdo/overview.md > GDOResources > Unit Control`: "Hard cap of 200". A unit that would take the used Unit
/// Control past it doesn't start production until there is room (design change 0143; `economy::army`).
pub const UNIT_CONTROL_CAP: u32 = 200;
