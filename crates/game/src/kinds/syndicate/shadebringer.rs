//! The Shadebringer (`factions/syndicate/units/shadebringer.md`, design change 0367; slice M6-SK): the plain unit box,
//! a square, with "SBR" on it, its first and last letters with its B, as the Seeker's "SKR" (its "SR" is the
//! Shrine's); unarmed. Its panel is the BasicCombatUnit template without its attack commands, plus its toggles, R
//! Sludge, and F Cover's slot until slice M6-SC (slice M6-SB:
//! `crate::command_ui::interface::templates::syndicate::shadebringer`). The Vehicle Assembly trains it, its W, behind a
//! Vault (slice M6-SB).

use crate::command_ui::interface::templates::syndicate::shadebringer;
use crate::kinds::ClientKind;
use crate::object_render::{Look, UnitShape};

pub const KIND: ClientKind = ClientKind {
    label: "SBR",
    look: Look::Unit {
        shape: UnitShape::Square,
        tagged: true,
    },
    shot: None,
    template: shadebringer::TEMPLATE,
};
