//! The Industrial Powerplant (`factions/gdo/structures/industrial_powerplant.md`, design change 0366, slice M6-GK): a
//! structure, labelled "IP", the initials of its name, with no panel (its design gives it none). (M6-GF) The
//! Deployment Center constructs it, its BuildMenu\[Advanced\]'s A
//! (`crate::command_ui::interface::templates::gdo::deployment_center`).

use crate::command_ui::interface::templates::PanelTemplate;
use crate::kinds::ClientKind;
use crate::object_render::Look;

pub const KIND: ClientKind = ClientKind {
    label: "IP",
    look: Look::Structure,
    shot: None,
    template: PanelTemplate::NONE,
};
