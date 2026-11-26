//! The Cathedral (`factions/cults/structures/cathedral.md`): a structure, labelled with its first and last letters,
//! "CL", as its Construct menus' entry was (slice M5-J1). Since slice M5-N4a the Recruits build it (Advanced C, greyed
//! "Requires a Temple" while their player owns none) and it trains Disciples from its pool: its panel is the pool
//! buildings' (design change 0337, `crate::command_ui::interface::templates::cults::armory`; `> Placeholder stats`:
//! "Panel like the Armory's (choose a kind with Q/W/E/R first, then; README > Choosing Recruits, Release and Eject
//! All): **Q: Train Disciple**, **A: Release** in the kind menu; **A: Eject All**, **C: Set Rally Point**, **X:
//! Cancel Production**"), its KindMenus its one recipe's, and its right-click sets the rally point.

use crate::command_ui::interface::templates::cults::armory::TEMPLATE;
use crate::kinds::ClientKind;
use crate::object_render::Look;

pub const KIND: ClientKind = ClientKind {
    label: "CL",
    look: Look::Structure,
    shot: None,
    template: TEMPLATE,
};
