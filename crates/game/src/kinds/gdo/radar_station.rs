//! The Radar Station (`factions/gdo/structures/radar_station.md`, design change 0366, slice M6-GK): a structure,
//! labelled "RS", the initials of its name, with no panel yet: its Scan is slice M6-GR's, which makes its Deployment
//! Center entry live, its BuildMenu\[Basic\]'s A greyed until then
//! (`crate::command_ui::interface::templates::gdo::deployment_center`).

use crate::command_ui::interface::templates::PanelTemplate;
use crate::kinds::ClientKind;
use crate::object_render::Look;

pub const KIND: ClientKind = ClientKind {
    label: "RS",
    look: Look::Structure,
    shot: None,
    template: PanelTemplate::NONE,
};
