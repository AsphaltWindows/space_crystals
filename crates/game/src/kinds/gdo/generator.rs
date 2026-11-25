//! The Generator (`factions/gdo/economy.md > Generator`): "ObjectInterfaceState: None (info display only)".

use crate::command_ui::interface::templates::PanelTemplate;
use crate::kinds::ClientKind;
use crate::object_render::Look;

pub const KIND: ClientKind = ClientKind {
    label: "GN",
    look: Look::Structure,
    shot: None,
    template: PanelTemplate::NONE,
};
