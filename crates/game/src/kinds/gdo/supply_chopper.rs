//! The Supply Chopper (`factions/gdo/units/supply_chopper.md`, design change 0366, slice M6-GK): the plain unit box, a
//! square, with its label on it, "SCR", the initials of its name and its last letter ("SC" being the crystal patch's),
//! drawn in flight as every air unit is (`crate::object_render::flight`); unarmed. Its panel, **placeholder** until
//! slice M6-GS builds the design's (`> ObjectInterfaceState[SupplyChopper]`: Move, Stop, Hold Position and its supply
//! and transport commands) and makes its Supply Tower entry live, is the BasicCombatUnit template without its attack
//! commands (the Prayer-mobile's).

use crate::command_ui::interface::templates::{InterfaceTemplate, PanelTemplate};
use crate::kinds::ClientKind;
use crate::object_render::{Look, UnitShape};

pub const KIND: ClientKind = ClientKind {
    label: "SCR",
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
