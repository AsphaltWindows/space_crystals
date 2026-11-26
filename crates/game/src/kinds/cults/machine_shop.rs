//! The Machine Shop (`factions/cults/structures/machine_shop.md`, slice M5-G2): a structure, labelled with its
//! initials, "MS". "Until its research is designed it has no panel commands and Recruits can't enter it": no panel.

use crate::command_ui::interface::templates::PanelTemplate;
use crate::kinds::ClientKind;
use crate::object_render::Look;

pub const KIND: ClientKind = ClientKind {
    label: "MS",
    look: Look::Structure,
    shot: None,
    template: PanelTemplate::NONE,
};
