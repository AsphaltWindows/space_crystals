//! The Supply Chopper (`factions/gdo/units/supply_chopper.md`, design change 0366, slice M6-GK): the plain unit box, a
//! square, with its label on it, "SCR", the initials of its name and its last letter ("SC" being the crystal patch's),
//! drawn in flight as every air unit is (`crate::object_render::flight`), and down on a Tower's or a station's spot
//! (`sc_sim::landing::of`); unarmed. Its panel is the design's (`factions/gdo/units/supply_chopper.md >
//! ObjectInterfaceState[SupplyChopper]`, `crate::command_ui::interface::templates::gdo::supply_chopper`) without the
//! Unit Transport's commands, as it carries Supplies alone (`sc_sim::supply_run::Cargo`).

use crate::command_ui::interface::templates::gdo::supply_chopper;
use crate::kinds::ClientKind;
use crate::object_render::{Look, UnitShape};

pub const KIND: ClientKind = ClientKind {
    label: "SCR",
    look: Look::Unit {
        shape: UnitShape::Square,
        tagged: true,
    },
    shot: None,
    template: supply_chopper::TEMPLATE,
};
