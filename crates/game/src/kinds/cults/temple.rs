//! The Temple (`factions/cults/structures/temple.md`): a structure, labelled with its first and last letters, "TE", as
//! its Construct menus' entry was (slice M5-J1). Its numbers are in the game since slice M5-G2; its research and its
//! staff come with its slice. Its panel comes with its slice, as the Recruits' build of it does; until then it has
//! none, and the Construct menus hold its slot, greyed (`sc_sim::defs::ConstructEntry::Planned`).

use crate::command_ui::interface::templates::PanelTemplate;
use crate::kinds::ClientKind;
use crate::object_render::Look;

pub const KIND: ClientKind = ClientKind {
    label: "TE",
    look: Look::Structure,
    shot: None,
    template: PanelTemplate::NONE,
};
