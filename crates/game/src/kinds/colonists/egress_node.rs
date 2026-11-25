//! The Egress Node (`factions/colonists/economy.md > Egress Node`): its panel's one command is F, Select Returning
//! Drones (`crate::command_ui::interface::templates::colonists::egress_node`).

use crate::command_ui::interface::templates;
use crate::kinds::ClientKind;
use crate::object_render::Look;

pub const KIND: ClientKind = ClientKind {
    label: "EN",
    look: Look::Structure,
    shot: None,
    template: templates::colonists::egress_node::TEMPLATE,
};
