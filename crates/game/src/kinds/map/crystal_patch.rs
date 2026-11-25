//! The Space Crystals Patch (`core/map.md > SpaceCrystalsPatch`): a map resource, drawn as shards of its own, with
//! no panel.

use crate::command_ui::interface::templates::PanelTemplate;
use crate::kinds::ClientKind;
use crate::object_render::Look;

pub const KIND: ClientKind = ClientKind {
    label: "SC",
    look: Look::CrystalPatch,
    shot: None,
    template: PanelTemplate::NONE,
};
