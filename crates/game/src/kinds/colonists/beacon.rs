//! The Beacon (`factions/colonists/economy.md > Beacon`): its panel opens the two Warp In menus
//! (`crate::command_ui::interface::templates::colonists::beacon`).

use crate::command_ui::interface::templates;
use crate::kinds::ClientKind;
use crate::object_render::Look;

pub const KIND: ClientKind = ClientKind {
    label: "BN",
    look: Look::Structure,
    shot: None,
    template: templates::colonists::beacon::TEMPLATE,
};
