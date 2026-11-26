//! The Chop Shop (`factions/cults/structures/chop_shop.md`): a structure, labelled with its initials, "CS", as its
//! Construct menus' entry was (slice M5-J1). Its numbers and its pool's capacity are in the game since slice M5-G2; its
//! vehicles and its pool building's panel (design change 0337) come with its slice, as the Recruits' build of it does;
//! until then it has no panel, and the Construct menus hold its slot, greyed (`sc_sim::defs::ConstructEntry::Planned`).

use crate::command_ui::interface::templates::PanelTemplate;
use crate::kinds::ClientKind;
use crate::object_render::Look;

pub const KIND: ClientKind = ClientKind {
    label: "CS",
    look: Look::Structure,
    shot: None,
    template: PanelTemplate::NONE,
};
