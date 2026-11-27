//! The GDO — `factions/gdo/`: its own rules, and one module per unit (`units/`, `factions/gdo/units/*.md`) and per
//! structure (`structures/`, `factions/gdo/structures/*.md`, the economy buildings in `factions/gdo/economy.md`).

pub(crate) mod structures;
pub(crate) mod units;

crate::tuning::tunables! {
    /// The GDO's own rules (`crate::ruleset::Ruleset::factions`).
    GdoRules;

    /// `factions/gdo/overview.md > GDOResources > Unit Control`: "Hard cap of 200". A unit that would take the used
    /// Unit Control past it doesn't start production until there is room (design change 0143; `economy::army`).
    unit_control_cap: u32 = 200,
}
