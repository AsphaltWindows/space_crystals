//! The Storage (`factions/cults/structures/storage.md`): a structure, labelled with its first and last letters, "SE".
//! "ObjectInterfaceState: None (info display only)".

use crate::command_ui::interface::templates::PanelTemplate;
use crate::kinds::ClientKind;
use crate::object_render::Look;

pub const KIND: ClientKind = ClientKind {
    label: "SE",
    look: Look::Structure,
    shot: None,
    template: PanelTemplate::NONE,
};
