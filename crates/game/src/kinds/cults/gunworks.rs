//! The Gunworks (`factions/cults/structures/gunworks.md`, slice M5-G2): a structure, labelled with its name's first
//! letter and its last, "GS". "**Until its research is designed it has no panel commands and Recruits can't enter it**
//! (a right-click on it with Recruits is a Move; Q0093)": no panel. It unlocks the Armory's Gunner.

use crate::command_ui::interface::templates::PanelTemplate;
use crate::kinds::ClientKind;
use crate::object_render::Look;

pub const KIND: ClientKind = ClientKind {
    label: "GS",
    look: Look::Structure,
    shot: None,
    template: PanelTemplate::NONE,
};
