//! The Barracks (`factions/gdo/structures/barracks.md`): the Production template, its units from its
//! `ProductionDef` (plan decision T23), the Field Engineer in E (slice M5-N9:
//! `crate::command_ui::interface::templates::gdo::barracks`).

use crate::command_ui::interface::templates::gdo;
use crate::kinds::ClientKind;
use crate::object_render::Look;

pub const KIND: ClientKind = ClientKind {
    label: "BR",
    look: Look::Structure,
    shot: None,
    template: gdo::barracks::TEMPLATE,
};
