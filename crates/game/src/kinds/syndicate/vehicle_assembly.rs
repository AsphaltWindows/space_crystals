//! The Vehicle Assembly (`factions/syndicate/structures/vehicle_assembly.md`, design change 0367; slices M6-SK and
//! M6-SV): an underground expansion, drawn as the Hideout is, an outline that only its owner sees
//! (`crate::object_render`). A Tier 2 Tunnel builds it, behind a Chemical Plant (the ExpandMenu's fifth entry, R), and
//! it has the Production template with the design's letters (`> Placeholder stats`: "Panel as the Hideout",
//! `crate::command_ui::interface::templates::syndicate::vehicle_assembly`): Q trains the Assault Bike into the Tunnel
//! Network, and W, E and R hold the units it doesn't train yet. Its label is "VA", its ExpandMenu placeholder's until
//! slice M6-SV (`Planned::label`).

use crate::command_ui::interface::templates::syndicate;
use crate::kinds::ClientKind;
use crate::object_render::Look;

pub const KIND: ClientKind = ClientKind {
    label: "VA",
    look: Look::Structure,
    shot: None,
    template: syndicate::vehicle_assembly::TEMPLATE,
};
