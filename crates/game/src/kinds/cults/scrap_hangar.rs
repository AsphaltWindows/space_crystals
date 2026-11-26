//! The Scrap Hangar (`factions/cults/structures/scrap_hangar.md`, slice M5-G2): a structure, labelled with its
//! initials, "SH". "No panel commands (it only unlocks)."

use crate::command_ui::interface::templates::PanelTemplate;
use crate::kinds::ClientKind;
use crate::object_render::Look;

pub const KIND: ClientKind = ClientKind {
    label: "SH",
    look: Look::Structure,
    shot: None,
    template: PanelTemplate::NONE,
};
