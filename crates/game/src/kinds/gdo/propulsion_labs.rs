//! Propulsion Labs (`factions/gdo/structures/propulsion_labs.md`, design change 0366, slice M6-GK): a structure,
//! labelled "PL", the initials of its name, with no panel ("Panel: none yet"). No player constructs one until slice
//! M6-GL, which makes its Deployment Center entry live, its BuildMenu\[Advanced\]'s R greyed until then
//! (`crate::command_ui::interface::templates::gdo::deployment_center`).

use crate::command_ui::interface::templates::PanelTemplate;
use crate::kinds::ClientKind;
use crate::object_render::Look;

pub const KIND: ClientKind = ClientKind {
    label: "PL",
    look: Look::Structure,
    shot: None,
    template: PanelTemplate::NONE,
};
