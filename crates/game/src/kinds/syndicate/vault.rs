//! The Vault (`factions/syndicate/structures/vault.md`, design change 0367; slice M6-SK): an underground
//! expansion, drawn as the Hideout is, an outline that only its owner sees (`crate::object_render`). A Tier 1 Tunnel
//! builds it behind a Hideout, its ExpandMenu's R (slice M6-SB), and it refills the tanks of the units in its owner's
//! Tunnel Network (`sc_sim::network::services`), which it does by itself: it has no panel, and selected, shows its
//! InfoPanel alone. Its label is its ExpandMenu placeholder's, "VT", which it took into the game.

use crate::command_ui::interface::templates::PanelTemplate;
use crate::kinds::ClientKind;
use crate::object_render::Look;

pub const KIND: ClientKind = ClientKind {
    label: "VT",
    look: Look::Structure,
    shot: None,
    template: PanelTemplate::NONE,
};
