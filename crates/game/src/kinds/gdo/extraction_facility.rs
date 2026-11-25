//! The Extraction Facility (`factions/gdo/economy.md > ExtractionFacility`): its panel builds and places its plate
//! (`crate::command_ui::interface::templates::gdo::extraction_facility`).

use crate::command_ui::interface::templates;
use crate::kinds::ClientKind;
use crate::object_render::Look;

pub const KIND: ClientKind = ClientKind {
    label: "EF",
    look: Look::Structure,
    shot: None,
    template: templates::gdo::extraction_facility::TEMPLATE,
};
