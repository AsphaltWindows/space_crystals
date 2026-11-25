//! The Drone (`factions/colonists/units/drone.md`): an octagon, a HoverVehicle's body, with its label; unarmed. Its
//! panel: Move, Deploy, Set Drop-off and Stop (`crate::command_ui::interface::templates::colonists::drone`).

use crate::command_ui::interface::templates;
use crate::kinds::ClientKind;
use crate::object_render::{Look, UnitShape};

pub const KIND: ClientKind = ClientKind {
    label: "DR",
    look: Look::Unit {
        shape: UnitShape::Octagon,
        tagged: true,
    },
    shot: None,
    template: templates::colonists::drone::TEMPLATE,
};
