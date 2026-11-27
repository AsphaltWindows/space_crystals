//! The Shadebringer (`factions/syndicate/units/shadebringer.md`, design change 0367; slice M6-SK): the plain unit box,
//! a square, with "SBR" on it, its first and last letters with its B, as the Seeker's "SKR" (its "SR" is the
//! Shrine's); unarmed. Its panel is the BasicCombatUnit template without its attack commands
//! (`PanelTemplate::without_attacks`: "Move-type commands like a combat unit without an attack, plus the two
//! toggles"), the toggles, R Sludge and F Cover, coming with slice M6-SB, which has the Vehicle Assembly build it
//! (plan decision T84): until then it is the debug spawn's alone, and the Tunnel's EjectMenu shows its tile, greyed
//! with a count of 0.

use crate::command_ui::interface::templates::{InterfaceTemplate, PanelTemplate};
use crate::kinds::ClientKind;
use crate::object_render::{Look, UnitShape};

pub const KIND: ClientKind = ClientKind {
    label: "SBR",
    look: Look::Unit {
        shape: UnitShape::Square,
        tagged: true,
    },
    shot: None,
    template: PanelTemplate {
        without_attacks: true,
        ..PanelTemplate::of(InterfaceTemplate::BasicCombatUnit)
    },
};
