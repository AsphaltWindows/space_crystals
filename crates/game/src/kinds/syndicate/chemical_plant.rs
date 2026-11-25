//! The Chemical Plant (`factions/syndicate/structures/chemical_plant.md`, design change 0260; slice M4-K): an
//! underground expansion, drawn as the Hideout is, an outline that only its owner sees (`crate::object_render`).
//! "Commands - none for now: it is only a prerequisite", so no panel: a selected one shows its InfoPanel alone. Its
//! label is its initials, "CP", as the ExpandMenu's placeholder for it was (slice M4-F).

use crate::command_ui::interface::templates::PanelTemplate;
use crate::kinds::ClientKind;
use crate::object_render::Look;

pub const KIND: ClientKind = ClientKind {
    label: "CP",
    look: Look::Structure,
    shot: None,
    template: PanelTemplate::NONE,
};
