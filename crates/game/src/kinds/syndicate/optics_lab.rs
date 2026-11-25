//! The Optics Lab (`factions/syndicate/structures/optics_lab.md`, design change 0229; slice M4-K): an underground
//! expansion, drawn as the Hideout is, an outline that only its owner sees (`crate::object_render`). "Commands - none
//! for now: it is only a prerequisite", so no panel: a selected one shows its InfoPanel alone. Its label is its
//! initials, "OL", as the ExpandMenu's placeholder for it was (slice M4-F).

use crate::command_ui::interface::templates::PanelTemplate;
use crate::kinds::ClientKind;
use crate::object_render::Look;

pub const KIND: ClientKind = ClientKind {
    label: "OL",
    look: Look::Structure,
    shot: None,
    template: PanelTemplate::NONE,
};
