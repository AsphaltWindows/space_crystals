//! The Extraction Plate (`factions/gdo/economy.md > ExtractionPlate`): drawn over the patch it covers, with no
//! panel.

use crate::command_ui::interface::templates::PanelTemplate;
use crate::kinds::ClientKind;
use crate::object_render::Look;

pub const KIND: ClientKind = ClientKind {
    label: "EP",
    look: Look::Structure,
    shot: None,
    template: PanelTemplate::NONE,
};
