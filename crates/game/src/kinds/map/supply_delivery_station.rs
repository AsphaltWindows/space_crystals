//! The Supply Delivery Station (`core/map.md > SupplyDeliveryStation`): a map resource with a shape of its own and
//! no panel; its label is its three initials.

use crate::command_ui::interface::templates::PanelTemplate;
use crate::kinds::ClientKind;
use crate::object_render::Look;

pub const KIND: ClientKind = ClientKind {
    label: "SDS",
    look: Look::SupplyDeliveryStation,
    shot: None,
    template: PanelTemplate::NONE,
};
