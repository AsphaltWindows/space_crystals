//! The Deployment Center (`factions/gdo/structures/deployment_center.md`): its panel opens the build menus
//! (`crate::command_ui::interface::templates::gdo::deployment_center`).

use crate::command_ui::interface::templates;
use crate::kinds::ClientKind;
use crate::object_render::Look;

pub const KIND: ClientKind = ClientKind {
    label: "DC",
    look: Look::Structure,
    shot: None,
    template: templates::gdo::deployment_center::TEMPLATE,
};
