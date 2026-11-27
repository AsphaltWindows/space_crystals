//! The First Aid Tent (`factions/gdo/structures/first_aid_tent.md`, design change 0366, slice M6-GK): a structure,
//! labelled "FAT", the initials of its name, its Field Engineer's Construct menu's entry's label
//! (`crate::command_ui::interface::templates::Planned::FirstAidTent`), with no panel. No player sets one up until
//! slice M6-GB, which makes that entry live, and its healing with it.

use crate::command_ui::interface::templates::PanelTemplate;
use crate::kinds::ClientKind;
use crate::object_render::Look;

pub const KIND: ClientKind = ClientKind {
    label: "FAT",
    look: Look::Structure,
    shot: None,
    template: PanelTemplate::NONE,
};
