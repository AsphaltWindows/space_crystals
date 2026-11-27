//! The Syndicate — `factions/syndicate/`: its own rules, and one module per unit (`units/`,
//! `factions/syndicate/units/*.md`) and per structure (`structures/`: the Tunnel, `factions/syndicate/tunnels.md >
//! Tunnel`, and the expansions, `factions/syndicate/structures/*.md`).

pub(crate) mod structures;
pub(crate) mod units;

use super::ObjectKind;

crate::tuning::tunables! {
    /// The Syndicate's own rules (`crate::ruleset::Ruleset::factions`).
    SyndicateRules;

    /// `factions/syndicate/tunnel_area_and_space.md > Tunnel Space`: "capped at a hard maximum of 200"
    /// (`factions/syndicate/overview.md > DisplayHud`: "Tunnel Space: current used / current available (max 200)";
    /// `economy::army`).
    tunnel_space_cap: u32 = 200,

    /// `factions/syndicate/tunnel_area_and_space.md > Tunnel Space`: the unused tunnel-region cells that make one
    /// Tunnel Space ("... / 4) — rounded down to a whole number"; `economy::army`).
    cells_per_tunnel_space: u32 = 4,
}

/// `factions/syndicate/tunnel_menus.md > Tunnel ObjectInterfaceState` (design change 0257, answering Q0068 point 4):
/// the EjectMenu's unit types, in the order they take the grid's slots: "entries take the grid slots in order (Q, W, E,
/// R, A, S, …)", the unit types "(Agent, Guard, Grenadier, …)", that is Agent Q, Guard W, Grenadier E (Q0068). That is
/// `factions/syndicate/units/README.md > Files` (design change 0285: "Listed in panel order: this is the order used by
/// the Tunnel's EjectMenu"): "Agent, Guard, Grenadier, then the vehicles and aircraft in production-building order",
/// the Assault Bike, the Shadebringer, the Laser Battery and the Subterranean Transport, then the Phantom, the Specter
/// and the Prism, Q to C. Each Syndicate unit joins the list with its kind (the Grenadier third, with slice M4-U; the
/// seven others with slice M6-SK), so that the client's menu (slice M4-J2) needs no edit, and each keeps its tile
/// greyed with a count of 0 while none is inside (design change 0299: "**Every Syndicate unit type keeps its
/// tile**"), as every one of the seven is until its slice makes it. A test checks that it lists every Syndicate unit
/// once, but a unit's second type, the Laser Battery (Deployed) and the Subterranean Transport (Burrowed), which never
/// enter the network: a deployed Battery can't move (`factions/syndicate/units/laser_battery.md > Deploy (ability)`:
/// "**cannot move**"), and a burrowed Transport surfaces before it drives in
/// (`factions/syndicate/units/subterranean_transport.md > Entering the Tunnel Network loaded`: "It never enters from
/// below").
pub const EJECT_MENU_ORDER: &[ObjectKind] = &[
    ObjectKind::Agent,
    ObjectKind::Guard,
    ObjectKind::Grenadier,
    ObjectKind::AssaultBike,
    ObjectKind::Shadebringer,
    ObjectKind::LaserBattery,
    ObjectKind::SubterraneanTransport,
    ObjectKind::Phantom,
    ObjectKind::Specter,
    ObjectKind::Prism,
];

/// An entry of the Tunnel's ExpandMenu ([`EXPAND_MENU`], slice M4-F): an expansion a Tunnel builds, or an entry not in
/// the game yet, which holds its slot.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ExpandEntry {
    /// A type a Tunnel builds (its `StructureDef::expansion`).
    Kind(ObjectKind),
    /// An entry no Tunnel builds yet, by its name in the design: an expansion of
    /// `factions/syndicate/structures/README.md > Tech Tree Overview` not in the game (the Laser Turret was one until
    /// slice M4-S2 had a Tunnel build it). It holds its slot, greyed (design change 0291: "Unavailable ones are greyed,
    /// including ones not in the build yet"). The slice that has a Tunnel build it turns it into its `Kind`.
    Planned(&'static str),
}

impl ExpandEntry {
    /// Its name: its type's, or the design's.
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

/// `factions/syndicate/tunnel_menus.md > Tunnel ObjectInterfaceState` (design change 0257): the ExpandMenu's entries,
/// in the order they take the grid's slots ("entries take the grid slots in order (Q, W, E, R, A, S, …): expansions in
/// the Tech Tree table's order (Hideout, Optics Lab, Chemical Plant, Vault, …)"), the one table of its slots (the
/// client's menu reads it). `factions/syndicate/tunnel_menus.md > ExpandMenu` (design change 0291, answering Q0081):
/// "**Fixed slots** ...: **all nine expansions** are always listed, in the Tech Tree table's order, each holding its
/// slot; ones not available (tier, prerequisite, or not yet in the build) are greyed. The **Laser Turret is always the
/// tenth slot**, so no hotkey ever moves": those no Tunnel builds yet are [`ExpandEntry::Planned`], the turret until
/// slice M4-S2. The Hideout, the Optics Lab and the Chemical Plant are in the game (slices M4-F and M4-K), in Q, W and
/// E. Tests check it against the table, and that it lists every type a Tunnel builds, once.
pub const EXPAND_MENU: &[ExpandEntry] = &[
    ExpandEntry::Kind(ObjectKind::Hideout),
    ExpandEntry::Kind(ObjectKind::OpticsLab),
    ExpandEntry::Kind(ObjectKind::ChemicalPlant),
    ExpandEntry::Planned("Vault"),
    ExpandEntry::Planned("Vehicle Assembly"),
    ExpandEntry::Planned("Forge"),
    ExpandEntry::Planned("Null Chamber"),
    ExpandEntry::Planned("Aerospace Assembly"),
    ExpandEntry::Planned("Orbital Command"),
    ExpandEntry::Kind(ObjectKind::LaserTurret),
];

#[cfg(test)]
mod tests;
