//! The Bio Portal (`factions/colonists/structures/bio_portal.md`): the Production template, with its Lancer and
//! its psionic unit, not in the game yet, in their slots
//! (`crate::command_ui::interface::templates::colonists::bio_portal`).

use crate::command_ui::interface::templates;
use crate::kinds::ClientKind;
use crate::object_render::Look;

pub const KIND: ClientKind = ClientKind {
    label: "BP",
    look: Look::Structure,
    shot: None,
    template: templates::colonists::bio_portal::TEMPLATE,
};
