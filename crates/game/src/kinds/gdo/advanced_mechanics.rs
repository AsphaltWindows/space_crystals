//! Advanced Mechanics (`factions/gdo/structures/advanced_mechanics.md`, design change 0366, slice M6-GK): a structure,
//! labelled "AM", the initials of its name, with no panel ("Panel: none yet"). (M6-GF) The Deployment Center
//! constructs it, its BuildMenu\[Advanced\]'s W (`crate::command_ui::interface::templates::gdo::deployment_center`).

use crate::command_ui::interface::templates::PanelTemplate;
use crate::kinds::ClientKind;
use crate::object_render::Look;

pub const KIND: ClientKind = ClientKind {
    label: "AM",
    look: Look::Structure,
    shot: None,
    template: PanelTemplate::NONE,
};
