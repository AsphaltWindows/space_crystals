//! The Armory (`factions/cults/structures/armory.md`): a structure, labelled with its first and last letters, "AY". Its
//! panel (`> ObjectInterfaceState[Armory]`, slice M5-F2) is the pool buildings' (design change 0337,
//! `crate::command_ui::interface::templates::cults::armory`): a kind of Recruit first, then Train Militant or Release,
//! and Eject All, Cancel and Set Rally Point; its right-click sets the rally point.

use crate::command_ui::interface::templates::cults::armory::TEMPLATE;
use crate::kinds::ClientKind;
use crate::object_render::Look;

pub const KIND: ClientKind = ClientKind {
    label: "AY",
    look: Look::Structure,
    shot: None,
    template: TEMPLATE,
};
