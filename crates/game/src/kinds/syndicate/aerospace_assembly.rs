//! The Aerospace Assembly (`factions/syndicate/structures/aerospace_assembly.md`, design change 0367; slices M6-SK and
//! M6-SD): an underground expansion, drawn as the Hideout is, an outline that only its owner sees
//! (`crate::object_render`). A Tier 3 Tunnel builds it, behind a Forge (the ExpandMenu's eighth entry, F), and it has
//! the Production template with the design's letters (`> Placeholder stats`: "Panel: **Q** Phantom, **W** Specter,
//! **E** Prism, **X** Cancel Production, **C** Set Rally Point",
//! `crate::command_ui::interface::templates::syndicate::aerospace_assembly`): Q trains the Phantom into the Tunnel
//! Network, behind an Optics Lab, and W and E hold the units it doesn't train yet. Its label is "AA", its ExpandMenu
//! placeholder's until slice M6-SD (`Planned::label`).

use crate::command_ui::interface::templates::syndicate;
use crate::kinds::ClientKind;
use crate::object_render::Look;

pub const KIND: ClientKind = ClientKind {
    label: "AA",
    look: Look::Structure,
    shot: None,
    template: syndicate::aerospace_assembly::TEMPLATE,
};
