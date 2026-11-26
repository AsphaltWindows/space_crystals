//! The Cults — `factions/cults/`: its own rules, and one module per unit (`units/`, `factions/cults/units/*.md`) and
//! per structure (`structures/`: the buildings of `factions/cults/structures/*.md`, the Recruitment Hall, the Storage
//! and the Armory so far).
//!
//! Slice M5-A put the faction's six designed kinds in the game's model with their numbers (plan decision T55): the
//! Recruit, the Militant, the Gunner, the Recruitment Hall, the Storage and the Armory. Each system's slice adds its
//! own field to their defs (plan §3.2, "one field per slice"): the Recruit's gathering and building, the Hall's
//! production, the Armory's pool and training, and every structure's site. A Cults unit's Unit Control cost is the
//! Recruits consumed to make it (`factions/cults/structures/recruitment_hall.md > Unit Control Tracking`), kept in
//! `UnitDef::unit_control_cost`; each point of it counts against the Hall its Recruit came from (`Object::lineage`,
//! plan decision T56). The faction's own rule so far: the Recruits' two Construct menus ([`CONSTRUCT_MENU`], slice
//! M5-J1), which the client's panel reads.

pub(crate) mod structures;
pub(crate) mod units;

#[cfg(test)]
mod tests;

use crate::defs::ObjectKind;

/// One of the Recruits' two build menus (`factions/cults/units/recruit.md > Recruit ObjectInterfaceState >
/// DefaultState commands`, design change 0327: "Two build menus ... "basic and advanced", as the GDO's Deployment
/// Center"): BuildMenu[Basic], which Basic Construct (A) enters, and BuildMenu[Advanced], which Advanced Construct (W)
/// enters.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ConstructMenu {
    Basic,
    Advanced,
}

/// An entry of a Construct menu ([`CONSTRUCT_MENU`]): a building the Recruits build, or one not in the game yet, which
/// holds its slot.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ConstructEntry {
    /// A type the Recruits build (their `WorkerDef::builds`).
    Kind(ObjectKind),
    /// A building of `factions/cults/structures/README.md > Tech Tree Overview` not in the game, by its name in the
    /// design. It holds its slot, greyed (design change 0327: "every building greyed until available, including ones
    /// not in the build yet"). The slice that brings the building turns it into its `Kind`.
    Planned(&'static str),
}

impl ConstructEntry {
    /// Its name: its type's, or the design's.
    pub fn name(self) -> &'static str {
        match self {
            ConstructEntry::Kind(kind) => kind.def().name,
            ConstructEntry::Planned(name) => name,
        }
    }

    /// The type it builds, if it is in the game.
    pub fn kind(self) -> Option<ObjectKind> {
        match self {
            ConstructEntry::Kind(kind) => Some(kind),
            ConstructEntry::Planned(_) => None,
        }
    }
}

/// A slot of a Construct menu: the menu, the entry, and where the entry stands on the panel's grid of 3 rows of 4
/// (`systems/controls/interface.md > CommandPanel`): the Q W E R row is row 0, the A S D F row 1 and the
/// Z X C V row 2, each from column 0.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ConstructSlot {
    pub menu: ConstructMenu,
    pub row: u8,
    pub column: u8,
    pub entry: ConstructEntry,
}

const fn slot(menu: ConstructMenu, row: u8, column: u8, entry: ConstructEntry) -> ConstructSlot {
    ConstructSlot {
        menu,
        row,
        column,
        entry,
    }
}

/// `factions/cults/units/recruit.md > Recruit ObjectInterfaceState > DefaultState commands` (design change 0327,
/// answering Q0096): the slots of the two Construct menus, the one table of them (the client's menus read it). "Fixed
/// slots: every building keeps its key and is greyed until its TechPrerequisite is met, so no hotkey ever moves. Rows
/// follow the branches: top row shared base, **A S D F row Tech/Weaponry, X C V row Mystical**; **Z** returns to
/// DefaultState": "**BuildMenu[Basic]**: **Q** Recruitment Hall, **W** Storage; **A** Armory, **S** Gunworks, **D**
/// Garrison; **X** Altar", and "**BuildMenu[Advanced]**: **A** Chop Shop, **S** Machine Shop, **D** Airstrip, **F**
/// Scrap Hangar; **X** Temple, **C** Cathedral, **V** Shrine". The buildings not in the game are
/// [`ConstructEntry::Planned`]. Tests check it against the design's lines, and that it lists every type the Recruits
/// build, once.
pub const CONSTRUCT_MENU: &[ConstructSlot] = {
    use ConstructEntry::{Kind, Planned};
    use ConstructMenu::{Advanced, Basic};
    &[
        slot(Basic, 0, 0, Kind(ObjectKind::RecruitmentHall)), // Q
        slot(Basic, 0, 1, Kind(ObjectKind::Storage)),         // W
        slot(Basic, 1, 0, Kind(ObjectKind::Armory)),          // A
        slot(Basic, 1, 1, Planned("Gunworks")),               // S
        slot(Basic, 1, 2, Planned("Garrison")),               // D
        slot(Basic, 2, 1, Planned("Altar")),                  // X
        slot(Advanced, 1, 0, Planned("Chop Shop")),           // A
        slot(Advanced, 1, 1, Planned("Machine Shop")),        // S
        slot(Advanced, 1, 2, Planned("Airstrip")),            // D
        slot(Advanced, 1, 3, Planned("Scrap Hangar")),        // F
        slot(Advanced, 2, 1, Planned("Temple")),              // X
        slot(Advanced, 2, 2, Planned("Cathedral")),           // C
        slot(Advanced, 2, 3, Planned("Shrine")),              // V
    ]
};
