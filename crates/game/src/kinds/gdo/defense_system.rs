//! The Defense System (`factions/gdo/structures/defense_system.md`, design change 0366, slice M6-GK): a structure,
//! labelled "DS", the initials of its name, with no panel yet: its Missile Turret is slice M6-GR's, which makes its
//! Deployment Center entry live, its BuildMenu\[Basic\]'s F greyed until then
//! (`crate::command_ui::interface::templates::gdo::deployment_center`), and its own construction with it.

use crate::command_ui::interface::templates::PanelTemplate;
use crate::kinds::ClientKind;
use crate::object_render::Look;

pub const KIND: ClientKind = ClientKind {
    label: "DS",
    look: Look::Structure,
    shot: None,
    template: PanelTemplate::NONE,
};
