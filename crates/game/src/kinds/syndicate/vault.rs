//! The Vault (`factions/syndicate/structures/vault.md`, design change 0367; slice M6-SK): an underground
//! expansion, drawn as the Hideout is, an outline that only its owner sees (`crate::object_render`). No Tunnel builds
//! one yet (plan decision T84): the ExpandMenu holds its slot with a placeholder (`Planned`), and slice M6-SB brings
//! what it stores and refills (`factions/syndicate/structures/vault.md`). Until then it has no panel: a debug spawn's, selected, shows its InfoPanel alone. Its label is its
//! placeholder's, "VT" (`Planned::label`), which it takes into the game.

use crate::command_ui::interface::templates::PanelTemplate;
use crate::kinds::ClientKind;
use crate::object_render::Look;

pub const KIND: ClientKind = ClientKind {
    label: "VT",
    look: Look::Structure,
    shot: None,
    template: PanelTemplate::NONE,
};
