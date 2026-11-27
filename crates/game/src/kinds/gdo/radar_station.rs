//! The Radar Station (`factions/gdo/structures/radar_station.md`, design change 0366, slice M6-GK): a structure,
//! labelled "RS", the initials of its name. (M6-GR) Its panel scans: Q awaits a point anywhere on the map
//! (`crate::command_ui::interface::templates::gdo::radar_station`), its Deployment Center entry live.

use crate::command_ui::interface::templates;
use crate::kinds::ClientKind;
use crate::object_render::Look;

pub const KIND: ClientKind = ClientKind {
    label: "RS",
    look: Look::Structure,
    shot: None,
    template: templates::gdo::radar_station::TEMPLATE,
};
