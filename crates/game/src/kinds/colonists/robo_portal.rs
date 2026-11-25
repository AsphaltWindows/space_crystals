//! The Robo Portal (`factions/colonists/structures/robo_portal.md`): the Production template, with its Scout, not
//! in the game yet, in its slot (`crate::command_ui::interface::templates::colonists::robo_portal`).

use crate::command_ui::interface::templates;
use crate::kinds::ClientKind;
use crate::object_render::Look;

pub const KIND: ClientKind = ClientKind {
    label: "RP",
    look: Look::Structure,
    shot: None,
    template: templates::colonists::robo_portal::TEMPLATE,
};
