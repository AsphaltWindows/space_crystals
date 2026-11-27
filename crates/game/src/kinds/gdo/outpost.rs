//! The Outpost (`factions/gdo/structures/outpost.md`, design change 0366, slice M6-GK): a structure, labelled "OT", the
//! first and last letters of its one-word name, with no panel yet. Nothing makes one until slice M6-GP, whose
//! Prospector deploys into it.

use crate::command_ui::interface::templates::PanelTemplate;
use crate::kinds::ClientKind;
use crate::object_render::Look;

pub const KIND: ClientKind = ClientKind {
    label: "OT",
    look: Look::Structure,
    shot: None,
    template: PanelTemplate::NONE,
};
