//! The Field Engineer (`factions/gdo/units/field_engineer.md`, design change 0328, slice M5-N9): a LightInfantry's
//! round body, the Peacekeeper's, the one that turns with its unit's facing, with its label on it, "FEN": its
//! initials, "FE", are the Forge's. It carries no weapon, so it has no shots. Its panel: Move, its Construct menu,
//! Repair and Stop (`crate::command_ui::interface::templates::gdo::field_engineer`).

use crate::command_ui::interface::templates::gdo;
use crate::kinds::ClientKind;
use crate::object_render::{Look, UnitShape};

pub const KIND: ClientKind = ClientKind {
    label: "FEN",
    look: Look::Unit {
        shape: UnitShape::Round,
        tagged: true,
    },
    shot: None,
    template: gdo::field_engineer::TEMPLATE,
};
