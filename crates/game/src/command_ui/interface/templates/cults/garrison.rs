//! The Garrison's panel (`factions/cults/structures/garrison.md > Placeholder stats > Panel`, slice M5-N2): "**A:
//! Eject All** (everyone inside leaves); **X: Cancel construction** while being built. Right-click resolution: none
//! (it has no rally point)". A lets everyone inside out at once through its door (`sc_sim::HoldCommand::EjectAll`),
//! greyed by the gate's refusal while it is empty (`sc_sim::Reject::NothingInside`); X is its site's, as every site's
//! (`super::super::SITE_GRID`). It is Groupable (its placeholder stats: "Groupable true"), so an Eject All goes to
//! every Garrison of the selection, a common command (`super::super::super::is_common`), unlike a pool building's.

use super::super::{InterfaceTemplate, PanelCommand, PanelTemplate};

/// The Garrison's panel template.
pub const TEMPLATE: PanelTemplate = PanelTemplate::of(InterfaceTemplate::Garrison);

/// Its panel, slot by slot (`CommandSlot::all` order): A Eject All, in the slot a pool building has it in
/// (`super::armory::GRID`).
#[rustfmt::skip]
pub const GRID: [Option<PanelCommand>; 12] = [
    None,                         None, None, None,
    Some(PanelCommand::EjectAll), None, None, None,
    None,                         None, None, None,
];
