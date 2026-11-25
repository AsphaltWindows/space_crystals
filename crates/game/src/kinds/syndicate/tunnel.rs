//! The Tunnel (`factions/syndicate/tunnels.md > Tunnel`): a structure labelled by its tier ([`tier_label`]), its
//! sides showing their functions (`crate::object_render`). Its panel comes with slice M4-J2
//! (`factions/syndicate/tunnels.md > Tunnel ObjectInterfaceState`): none yet.

use crate::command_ui::interface::templates::PanelTemplate;
use crate::kinds::ClientKind;
use crate::object_render::Look;

/// Its type's label is Tier 1's, the tier it starts at.
pub const KIND: ClientKind = ClientKind {
    label: TIER_LABELS[0],
    look: Look::Structure,
    shot: None,
    template: PanelTemplate::NONE,
};

/// A Tunnel's label by tier (slice M4-A, `factions/syndicate/tunnels.md > Tunnel > Tiers`): "T1" to "T3".
const TIER_LABELS: [&str; 3] = ["T1", "T2", "T3"];

/// The label of a Tunnel at `tier` ([`TIER_LABELS`]); the type's own for a tier the table hasn't.
pub fn tier_label(tier: u8) -> &'static str {
    let index = usize::from(tier).checked_sub(1);
    index
        .and_then(|i| TIER_LABELS.get(i))
        .copied()
        .unwrap_or(TIER_LABELS[0])
}
