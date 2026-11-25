//! The Armory (`factions/cults/structures/armory.md`): a structure, labelled with its first and last letters, "AY". No
//! panel yet: its Eject All, its training and its rally point (`> ObjectInterfaceState[Armory]`) come with slices
//! M5-F1 and M5-F2.

use crate::command_ui::interface::templates::PanelTemplate;
use crate::kinds::ClientKind;
use crate::object_render::Look;

pub const KIND: ClientKind = ClientKind {
    label: "AY",
    look: Look::Structure,
    shot: None,
    template: PanelTemplate::NONE,
};
