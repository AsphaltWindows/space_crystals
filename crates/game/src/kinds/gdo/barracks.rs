//! The Barracks (`factions/gdo/structures/barracks.md`): the Production template, its units from its
//! `ProductionDef` (plan decision T23).

use crate::command_ui::interface::templates::{InterfaceTemplate, PanelTemplate};
use crate::kinds::ClientKind;
use crate::object_render::Look;

pub const KIND: ClientKind = ClientKind {
    label: "BR",
    look: Look::Structure,
    shot: None,
    template: PanelTemplate::of(InterfaceTemplate::Production),
};
