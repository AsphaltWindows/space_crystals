//! The Forge (`factions/syndicate/structures/forge.md`, design change 0367; slices M6-SK and M6-SD): an underground
//! expansion, drawn as the Hideout is, an outline that only its owner sees (`crate::object_render`). A Tier 2 Tunnel
//! builds it, behind a Vehicle Assembly (the ExpandMenu's sixth entry, S), for the Aerospace Assembly behind it. It
//! has no panel ("No commands for now (prerequisite only)"): selected, it shows its InfoPanel alone. Its label is "FE",
//! its ExpandMenu placeholder's until slice M6-SD (`Planned::label`).

use crate::command_ui::interface::templates::PanelTemplate;
use crate::kinds::ClientKind;
use crate::object_render::Look;

pub const KIND: ClientKind = ClientKind {
    label: "FE",
    look: Look::Structure,
    shot: None,
    template: PanelTemplate::NONE,
};
