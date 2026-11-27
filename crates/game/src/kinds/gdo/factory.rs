//! The Factory (`factions/gdo/structures/factory.md`, design change 0366, slice M6-GK): a structure, labelled "FY",
//! the first and last letters of its one-word name, with no panel yet. No player constructs one until slice M6-GF,
//! which makes its Deployment Center entry live, its BuildMenu\[Advanced\]'s Q greyed until then
//! (`crate::command_ui::interface::templates::gdo::deployment_center`), and its production with it.

use crate::command_ui::interface::templates::PanelTemplate;
use crate::kinds::ClientKind;
use crate::object_render::Look;

pub const KIND: ClientKind = ClientKind {
    label: "FY",
    look: Look::Structure,
    shot: None,
    template: PanelTemplate::NONE,
};
