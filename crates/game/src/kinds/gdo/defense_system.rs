//! The Defense System (`factions/gdo/structures/defense_system.md`, design change 0366, slice M6-GK): a structure,
//! labelled "DS", the initials of its name. (M6-GR) Its panel builds and places its Missile Turrets, one at a time, as
//! the Extraction Facility's its plate (`crate::command_ui::interface::templates::gdo::defense_system`), its
//! Deployment Center entry live.

use crate::command_ui::interface::templates;
use crate::kinds::ClientKind;
use crate::object_render::Look;

pub const KIND: ClientKind = ClientKind {
    label: "DS",
    look: Look::Structure,
    shot: None,
    template: templates::gdo::defense_system::TEMPLATE,
};
