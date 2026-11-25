//! The Hideout (`factions/syndicate/structures/hideout.md`): an underground structure, drawn as an outline that only
//! its owner sees (`crate::object_render`). It has the Production template (slice M4-C: it produces into the Tunnel
//! Network, plan decision T23: "every type that produces units has the Production template"), with the design's
//! letters (slice M4-J2, `crate::command_ui::interface::templates::syndicate::hideout`).

use crate::command_ui::interface::templates::syndicate;
use crate::kinds::ClientKind;
use crate::object_render::Look;

pub const KIND: ClientKind = ClientKind {
    label: "HI",
    look: Look::Structure,
    shot: None,
    template: syndicate::hideout::TEMPLATE,
};
