//! The Null Chamber (`factions/syndicate/structures/null_chamber.md`, design change 0367; slices M6-SK and M6-SN): an
//! underground expansion, drawn as the Hideout is, an outline that only its owner sees (`crate::object_render`). A Tier
//! 2 Tunnel builds it, behind a Hideout (the ExpandMenu's seventh entry, D). It has no commands: selected, it shows its
//! InfoPanel alone. What it does is the sim's (`sc_sim::network::services`): the units that leave the network buffed
//! show the buff's mark beside their HP bar (`crate::combat_effects::buffs`). Its label is "NC", its ExpandMenu
//! placeholder's until slice M6-SN (`Planned::label`).

use crate::command_ui::interface::templates::PanelTemplate;
use crate::kinds::ClientKind;
use crate::object_render::Look;

pub const KIND: ClientKind = ClientKind {
    label: "NC",
    look: Look::Structure,
    shot: None,
    template: PanelTemplate::NONE,
};
