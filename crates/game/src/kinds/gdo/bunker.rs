//! The Bunker (`factions/gdo/structures/bunker.md`, design change 0366, slice M6-GK): a structure, labelled "BKR", its
//! Field Engineer's Construct menu's entry's label (`crate::command_ui::interface::templates::Planned::Bunker`, "BR"
//! being the Barracks'), with no panel yet. No player builds one until slice M6-GB, which makes that entry live, and
//! its garrison with it.

use crate::command_ui::interface::templates::PanelTemplate;
use crate::kinds::ClientKind;
use crate::object_render::Look;

pub const KIND: ClientKind = ClientKind {
    label: "BKR",
    look: Look::Structure,
    shot: None,
    template: PanelTemplate::NONE,
};
